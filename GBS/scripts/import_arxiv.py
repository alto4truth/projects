from __future__ import annotations

import io
import json
import shutil
import sys
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from pathlib import Path

from pypdf import PdfReader

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from gbs.cli import build_chunks_from_text, normalize_text, write_chunks


ARXIV_API = "https://export.arxiv.org/api/query"
DEFAULT_QUERY = "cat:cs.AI OR cat:cs.LG OR cat:cs.CL OR cat:cs.CV"


def fetch_atom(query: str, start: int, max_results: int) -> ET.Element:
    params = urllib.parse.urlencode(
        {
            "search_query": query,
            "start": start,
            "max_results": max_results,
            "sortBy": "submittedDate",
            "sortOrder": "descending",
        }
    )
    with urllib.request.urlopen(f"{ARXIV_API}?{params}", timeout=120) as response:
        payload = response.read()
    return ET.fromstring(payload)


def parse_entries(root: ET.Element) -> list[dict]:
    ns = {"atom": "http://www.w3.org/2005/Atom"}
    entries: list[dict] = []
    for entry in root.findall("atom:entry", ns):
        links = entry.findall("atom:link", ns)
        pdf_url = None
        for link in links:
            if link.attrib.get("title") == "pdf":
                pdf_url = link.attrib.get("href")
                break
        if not pdf_url:
            continue

        identifier = entry.findtext("atom:id", default="", namespaces=ns).rstrip("/").split("/")[-1]
        entries.append(
            {
                "id": identifier,
                "title": " ".join(entry.findtext("atom:title", default="", namespaces=ns).split()),
                "authors": [
                    author.findtext("atom:name", default="", namespaces=ns)
                    for author in entry.findall("atom:author", ns)
                ],
                "published": entry.findtext("atom:published", default="", namespaces=ns),
                "updated": entry.findtext("atom:updated", default="", namespaces=ns),
                "summary": " ".join(entry.findtext("atom:summary", default="", namespaces=ns).split()),
                "pdf_url": pdf_url,
            }
        )
    return entries


def fetch_pdf_text(pdf_url: str) -> str:
    with urllib.request.urlopen(pdf_url, timeout=180) as response:
        pdf_bytes = response.read()
    reader = PdfReader(io.BytesIO(pdf_bytes))
    parts: list[str] = []
    for page in reader.pages:
        text = page.extract_text() or ""
        if text.strip():
            parts.append(text)
    return normalize_text("\n\n".join(parts))


def import_entry(entry: dict, corpus_dir: Path) -> dict | None:
    text = fetch_pdf_text(entry["pdf_url"])
    if len(text) < 15_000:
        text = normalize_text(entry["summary"])
    if len(text) < 3_000:
        return None

    year = entry["published"][:4] or "undated"
    chunks = build_chunks_from_text(
        text=text,
        source="arxiv",
        year=year,
        title=entry["title"],
        authors=entry["authors"],
        source_ref=entry["pdf_url"],
        max_chars=1200,
        overlap_chars=150,
    )
    if not chunks:
        return None

    output_dir = write_chunks(chunks, corpus_dir)
    return {
        "id": entry["id"],
        "title": entry["title"],
        "authors": entry["authors"],
        "published": entry["published"],
        "updated": entry["updated"],
        "pdf_url": entry["pdf_url"],
        "corpus_path": str(output_dir.relative_to(ROOT)),
        "chunk_count": len(chunks),
    }


def main() -> None:
    query = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_QUERY
    limit = int(sys.argv[2]) if len(sys.argv) > 2 else 25

    corpus_dir = ROOT / "corpus"
    catalog_dir = ROOT / "catalogs"
    catalog_dir.mkdir(parents=True, exist_ok=True)

    root = fetch_atom(query=query, start=0, max_results=limit)
    entries = parse_entries(root)
    imported: list[dict] = []

    for idx, entry in enumerate(entries[:limit], start=1):
        print(f"[{idx}/{limit}] {entry['id']} {entry['title']}")
        try:
            item = import_entry(entry, corpus_dir)
        except Exception as exc:
            print(f"  skipped: {exc}")
            continue
        if not item:
            print("  skipped: no usable text")
            continue
        imported.append(item)
        print(f"  imported [{item['chunk_count']} chunks]")

    catalog_path = catalog_dir / "arxiv_articles.json"
    for item in imported:
        item.pop("chunk_count", None)
    catalog_path.write_text(json.dumps(imported, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"Wrote catalog: {catalog_path}")


if __name__ == "__main__":
    main()
