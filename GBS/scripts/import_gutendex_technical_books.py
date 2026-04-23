from __future__ import annotations

import json
import re
import shutil
import ssl
import sys
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from gbs.cli import build_chunks_from_text, normalize_text, write_chunks


SEARCH_TERMS = [
    "mathematics",
    "physics",
    "chemistry",
    "astronomy",
    "geology",
]

TECHNICAL_KEYWORDS = {
    "algebra",
    "anatomy",
    "astronomy",
    "botany",
    "calculus",
    "chemistry",
    "electric",
    "electricity",
    "engineering",
    "geometry",
    "geology",
    "machine",
    "mathemat",
    "mechanic",
    "medicine",
    "mineral",
    "natural philosophy",
    "physics",
    "science",
    "technical",
    "technology",
    "zoology",
}

FORMAT_PREFERENCE = [
    "text/plain; charset=utf-8",
    "text/plain; charset=us-ascii",
    "text/plain",
]


def fetch_json(url: str) -> dict:
    last_error: Exception | None = None
    candidates = [url]
    if url.startswith("https://"):
        candidates.append("http://" + url[len("https://") :])

    for candidate in candidates:
        for _ in range(3):
            try:
                request = urllib.request.Request(
                    candidate,
                    headers={"User-Agent": "GBS/0.1 (+https://github.com/alto4truth/projects)"},
                )
                with urllib.request.urlopen(
                    request,
                    timeout=20,
                    context=ssl._create_unverified_context() if candidate.startswith("https://") else None,
                ) as response:
                    return json.load(response)
            except Exception as exc:
                last_error = exc
                continue

    if last_error is None:
        raise RuntimeError(f"Failed to fetch {url}")
    raise last_error


def pick_text_url(formats: dict[str, str]) -> str | None:
    for key in FORMAT_PREFERENCE:
        url = formats.get(key)
        if url:
            return url
    for key, url in formats.items():
        if key.startswith("text/plain") and url and not url.endswith(".zip"):
            return url
    return None


def looks_technical(book: dict) -> bool:
    haystack = " ".join(
        [book.get("title", "")]
        + book.get("subjects", [])
        + book.get("bookshelves", [])
    ).lower()
    return any(keyword in haystack for keyword in TECHNICAL_KEYWORDS)


def strip_gutenberg_boilerplate(text: str) -> str:
    start_markers = [
        r"\*\*\* START OF THE PROJECT GUTENBERG EBOOK .* \*\*\*",
        r"\*\*\* START OF THIS PROJECT GUTENBERG EBOOK .* \*\*\*",
        r"START OF THE PROJECT GUTENBERG EBOOK",
    ]
    end_markers = [
        r"\*\*\* END OF THE PROJECT GUTENBERG EBOOK .* \*\*\*",
        r"END OF THE PROJECT GUTENBERG EBOOK",
    ]

    for marker in start_markers:
        match = re.search(marker, text, flags=re.IGNORECASE)
        if match:
            text = text[match.end() :]
            break

    for marker in end_markers:
        match = re.search(marker, text, flags=re.IGNORECASE)
        if match:
            text = text[: match.start()]
            break

    return text.strip()


def fetch_text(url: str) -> str:
    last_error: Exception | None = None
    candidates = [url]
    if url.startswith("https://"):
        candidates.append("http://" + url[len("https://") :])

    for candidate in candidates:
        for _ in range(3):
            try:
                request = urllib.request.Request(
                    candidate,
                    headers={"User-Agent": "GBS/0.1 (+https://github.com/alto4truth/projects)"},
                )
                with urllib.request.urlopen(
                    request,
                    timeout=20,
                    context=ssl._create_unverified_context() if candidate.startswith("https://") else None,
                ) as response:
                    raw = response.read()
                    encoding = response.headers.get_content_charset() or "utf-8"
                text = raw.decode(encoding, errors="replace")
                return normalize_text(strip_gutenberg_boilerplate(text))
            except Exception as exc:
                last_error = exc
                continue
    if last_error is None:
        raise RuntimeError(f"Failed to download {url}")
    raise last_error


def iter_books_for_term(term: str) -> list[dict]:
    url = "https://gutendex.com/books/?search=" + urllib.parse.quote(term)
    collected: list[dict] = []
    page = 1
    while url and len(collected) < 80:
        print(f"select term={term} page={page}")
        payload = fetch_json(url)
        collected.extend(payload.get("results", []))
        url = payload.get("next")
        page += 1
    return collected


def select_books(limit: int) -> list[dict]:
    selected: dict[int, dict] = {}

    for term in SEARCH_TERMS:
        before = len(selected)
        for book in iter_books_for_term(term):
            if "en" not in book.get("languages", []):
                continue
            if not looks_technical(book):
                continue
            text_url = pick_text_url(book.get("formats", {}))
            if not text_url:
                continue

            book_id = book["id"]
            current = selected.get(book_id)
            candidate = {
                "id": book_id,
                "title": book["title"],
                "authors": [author["name"] for author in book.get("authors", [])],
                "download_count": book.get("download_count", 0),
                "subjects": book.get("subjects", []),
                "bookshelves": book.get("bookshelves", []),
                "languages": book.get("languages", []),
                "text_url": text_url,
                "gutendex_url": f"https://gutendex.com/books/{book_id}",
            }
            if current is None or candidate["download_count"] > current["download_count"]:
                selected[book_id] = candidate
        print(f"selected after {term}: {len(selected)} (+{len(selected) - before})")

    books = sorted(selected.values(), key=lambda item: (-item["download_count"], item["title"]))
    return books[:limit]


def import_one_book(book: dict, rank: int, corpus_dir: Path) -> dict | None:
    text = fetch_text(book["text_url"])
    if len(text) < 20_000:
        return None

    chunks = build_chunks_from_text(
        text=text,
        source="project-gutenberg",
        year="undated",
        title=book["title"],
        authors=book["authors"],
        source_ref=book["text_url"],
        max_chars=1200,
        overlap_chars=150,
    )
    if not chunks:
        return None

    output_dir = write_chunks(chunks, corpus_dir)
    return {
        "rank": rank,
        "id": book["id"],
        "title": book["title"],
        "authors": book["authors"],
        "download_count": book["download_count"],
        "text_url": book["text_url"],
        "gutendex_url": book["gutendex_url"],
        "corpus_path": str(output_dir.relative_to(ROOT)),
        "chunk_count": len(chunks),
    }


def import_books(limit: int) -> None:
    corpus_dir = ROOT / "corpus"
    catalog_dir = ROOT / "catalogs"
    catalog_dir.mkdir(parents=True, exist_ok=True)
    shutil.rmtree(corpus_dir / "project-gutenberg" / "undated", ignore_errors=True)

    books = select_books(limit + 20)
    if len(books) < limit:
        raise RuntimeError(f"Only found {len(books)} eligible technical books, expected at least {limit}.")

    imported: list[dict] = []
    with ThreadPoolExecutor(max_workers=4) as pool:
        futures = {
            pool.submit(import_one_book, book, rank, corpus_dir): (rank, book["title"])
            for rank, book in enumerate(books, start=1)
        }
        for future in as_completed(futures):
            rank, title = futures[future]
            try:
                item = future.result()
            except Exception as exc:
                print(f"skip rank {rank}: {title} ({exc})")
                continue
            if item is None:
                print(f"skip rank {rank}: {title}")
                continue
            print(f"imported rank {rank}: {title} [{item['chunk_count']} chunks]")
            imported.append(item)

    if len(imported) < limit:
        raise RuntimeError(f"Imported {len(imported)} books, expected at least {limit}.")

    imported.sort(key=lambda item: item["rank"])
    extras = imported[limit:]
    imported = imported[:limit]

    for item in extras:
        shutil.rmtree(ROOT / item["corpus_path"], ignore_errors=True)

    catalog_path = catalog_dir / "project_gutenberg_technical_books.json"
    for item in imported:
        item.pop("rank", None)
        item.pop("chunk_count", None)
    catalog_path.write_text(json.dumps(imported, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"Wrote catalog: {catalog_path}")


if __name__ == "__main__":
    import_books(limit=100)
