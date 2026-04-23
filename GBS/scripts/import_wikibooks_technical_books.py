from __future__ import annotations

import json
import sys
import time
import urllib.parse
import urllib.request
from urllib.error import HTTPError
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from gbs.cli import build_chunks_from_text, normalize_text, slugify, write_chunks


API = "https://en.wikibooks.org/w/api.php"
USER_AGENT = "GBS/0.1 (https://github.com/alto4truth/projects)"
CATEGORY_TITLES = [
    "Category:Subject:Computer science/all books",
    "Category:Subject:Computer programming/all books",
    "Category:Subject:Computer software/all books",
    "Category:Subject:Information technology/all books",
    "Category:Subject:Mathematics/all books",
    "Category:Subject:Engineering/all books",
]
SKIP_SUBPAGE_SUFFIXES = {
    "/Printable version",
    "/Print version",
    "/Copyright",
}
MIN_REQUEST_INTERVAL = 1.0
REQUEST_TIMEOUT = 12
MAX_SUBPAGES_PER_BOOK = 40
MAX_SECONDS_PER_BOOK = 45
_LAST_REQUEST_AT = 0.0


def api_get(params: dict[str, str]) -> dict:
    global _LAST_REQUEST_AT
    query = urllib.parse.urlencode(params)
    last_error: Exception | None = None
    for attempt in range(6):
        try:
            now = time.monotonic()
            wait = MIN_REQUEST_INTERVAL - (now - _LAST_REQUEST_AT)
            if wait > 0:
                time.sleep(wait)
            req = urllib.request.Request(f"{API}?{query}", headers={"User-Agent": USER_AGENT})
            with urllib.request.urlopen(req, timeout=REQUEST_TIMEOUT) as response:
                _LAST_REQUEST_AT = time.monotonic()
                return json.load(response)
        except HTTPError as exc:
            last_error = exc
            _LAST_REQUEST_AT = time.monotonic()
            if exc.code == 429 and attempt < 5:
                time.sleep(10 + attempt * 5)
                continue
            raise
        except Exception as exc:
            last_error = exc
            _LAST_REQUEST_AT = time.monotonic()
            if attempt < 5:
                time.sleep(1 + attempt)
    if last_error is None:
        raise RuntimeError("api_get failed without an exception")
    raise last_error


def fetch_category_titles(category_title: str, limit: int = 200) -> list[str]:
    titles: list[str] = []
    cont: dict[str, str] = {}
    while len(titles) < limit:
        params = {
            "action": "query",
            "list": "categorymembers",
            "cmtitle": category_title,
            "cmlimit": "500",
            "format": "json",
        }
        params.update(cont)
        data = api_get(params)
        titles.extend(item["title"] for item in data["query"]["categorymembers"])
        cont = data.get("continue", {})
        if not cont:
            break
    return titles[:limit]


def clean_book_title(title: str) -> bool:
    if ":" in title:
        return False
    if "/" in title:
        return False
    if title.startswith("Shelf:"):
        return False
    return True


def select_books(limit: int) -> list[str]:
    selected: list[str] = []
    seen: set[str] = set()
    for category in CATEGORY_TITLES:
        print(f"collect {category}")
        for title in fetch_category_titles(category):
            if not clean_book_title(title):
                continue
            if title in seen:
                continue
            seen.add(title)
            selected.append(title)
            if len(selected) >= limit:
                return selected
    return selected


def fetch_page_extract(title: str) -> str:
    data = api_get(
        {
            "action": "query",
            "prop": "extracts",
            "explaintext": "1",
            "titles": title,
            "format": "json",
        }
    )
    page = next(iter(data["query"]["pages"].values()))
    return normalize_text(page.get("extract", ""))


def fetch_subpages(title: str) -> list[str]:
    subpages: list[str] = []
    cont: dict[str, str] = {}
    prefix = f"{title}/"
    while True:
        params = {
            "action": "query",
            "generator": "allpages",
            "gapprefix": prefix,
            "gapnamespace": "0",
            "gaplimit": "500",
            "prop": "info",
            "format": "json",
        }
        params.update(cont)
        data = api_get(params)
        pages = data.get("query", {}).get("pages", {})
        subpages.extend(page["title"] for page in pages.values())
        cont = data.get("continue", {})
        if not cont:
            break
    subpages.sort()
    return [page for page in subpages if not any(page.endswith(suffix) for suffix in SKIP_SUBPAGE_SUFFIXES)]


def build_book_text(title: str) -> str:
    parts: list[str] = []
    started_at = time.monotonic()

    root_text = fetch_page_extract(title)
    if root_text:
        parts.append(f"# {title}\n\n{root_text}")

    subpages = fetch_subpages(title)
    if len(subpages) > MAX_SUBPAGES_PER_BOOK:
        print(f"  truncating subpages: {len(subpages)} -> {MAX_SUBPAGES_PER_BOOK}")
        subpages = subpages[:MAX_SUBPAGES_PER_BOOK]

    for offset, subpage in enumerate(subpages, start=1):
        if time.monotonic() - started_at > MAX_SECONDS_PER_BOOK:
            print(f"  time budget reached after {offset - 1} subpages")
            break
        text = fetch_page_extract(subpage)
        if not text:
            continue
        heading = subpage.split("/", 1)[1]
        parts.append(f"## {heading}\n\n{text}")
        if offset % 10 == 0:
            print(f"  fetched {offset}/{len(subpages)} subpages")

    return normalize_text("\n\n".join(parts))


def import_book(title: str, corpus_dir: Path) -> dict | None:
    existing_dir = corpus_dir / "wikibooks" / "undated" / slugify(title)
    manifest_path = existing_dir / "manifest.json"
    if manifest_path.exists():
        data = json.loads(manifest_path.read_text(encoding="utf-8"))
        chunks = data.get("chunks", [])
        return {
            "title": title,
            "source_url": f"https://en.wikibooks.org/wiki/{urllib.parse.quote(title.replace(' ', '_'), safe=':_()/')}",
            "corpus_path": str(existing_dir.relative_to(ROOT)),
            "chunk_count": len(chunks),
        }

    text = build_book_text(title)
    if len(text) < 3_000:
        return None

    chunks = build_chunks_from_text(
        text=text,
        source="wikibooks",
        year="undated",
        title=title,
        authors=[],
        source_ref=f"https://en.wikibooks.org/wiki/{urllib.parse.quote(title.replace(' ', '_'), safe=':_()/')}",
        max_chars=1200,
        overlap_chars=150,
    )
    if not chunks:
        return None

    output_dir = write_chunks(chunks, corpus_dir)
    return {
        "title": title,
        "source_url": f"https://en.wikibooks.org/wiki/{urllib.parse.quote(title.replace(' ', '_'), safe=':_()/')}",
        "corpus_path": str(output_dir.relative_to(ROOT)),
        "chunk_count": len(chunks),
    }


def main() -> None:
    limit = int(sys.argv[1]) if len(sys.argv) > 1 else 100
    corpus_dir = ROOT / "corpus"
    catalog_dir = ROOT / "catalogs"
    catalog_dir.mkdir(parents=True, exist_ok=True)
    output = catalog_dir / "wikibooks_technical_books.json"

    selected = select_books(limit + 40)
    if len(selected) < limit:
        raise RuntimeError(f"Only selected {len(selected)} books from Wikibooks")

    imported: list[dict] = json.loads(output.read_text(encoding="utf-8")) if output.exists() else []
    imported_titles = {item["title"] for item in imported}
    for index, title in enumerate(selected, start=1):
        if len(imported) >= limit:
            break
        if title in imported_titles:
            continue
        print(f"[{index}/{len(selected)}] {title}")
        try:
            item = import_book(title, corpus_dir)
        except Exception as exc:
            print(f"  skipped: {exc}")
            continue
        if not item:
            print("  skipped: too short")
            continue
        imported.append(item)
        imported_titles.add(title)
        print(f"  imported [{item['chunk_count']} chunks]")
        persisted = [{k: v for k, v in item.items() if k != "chunk_count"} for item in imported]
        output.write_text(json.dumps(persisted, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    if len(imported) < limit:
        raise RuntimeError(f"Imported {len(imported)} books, expected {limit}")

    imported = imported[:limit]
    for item in imported:
        item.pop("chunk_count", None)
    output.write_text(json.dumps(imported, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"Wrote catalog: {output}")


if __name__ == "__main__":
    main()
