from __future__ import annotations

import argparse
import html
import json
import re
import unicodedata
from dataclasses import asdict, dataclass
from html.parser import HTMLParser
from pathlib import Path
from typing import Iterable


SUPPORTED_TEXT_EXTENSIONS = {".txt", ".md", ".html", ".htm"}


class HTMLTextExtractor(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.parts: list[str] = []
        self._skip_depth = 0

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag in {"script", "style"}:
            self._skip_depth += 1
        elif tag in {"p", "div", "section", "article", "br", "li", "h1", "h2", "h3", "h4"}:
            self.parts.append("\n")

    def handle_endtag(self, tag: str) -> None:
        if tag in {"script", "style"} and self._skip_depth > 0:
            self._skip_depth -= 1
        elif tag in {"p", "div", "section", "article", "li"}:
            self.parts.append("\n")

    def handle_data(self, data: str) -> None:
        if self._skip_depth == 0:
            self.parts.append(data)

    def text(self) -> str:
        return "".join(self.parts)


@dataclass
class Chunk:
    title: str
    source: str
    year: str
    document_id: str
    page: int
    section: str
    chunk: int
    source_path: str
    text: str


def slugify(value: str) -> str:
    normalized = unicodedata.normalize("NFKD", value).encode("ascii", "ignore").decode("ascii")
    lowered = normalized.lower()
    collapsed = re.sub(r"[^a-z0-9]+", "-", lowered).strip("-")
    return collapsed or "document"


def normalize_text(raw: str) -> str:
    raw = html.unescape(raw)
    raw = raw.replace("\r\n", "\n").replace("\r", "\n")
    raw = re.sub(r"-\n(?=\w)", "", raw)
    raw = re.sub(r"[ \t]+", " ", raw)
    raw = re.sub(r"\n{3,}", "\n\n", raw)
    lines = [line.strip() for line in raw.splitlines()]
    return "\n".join(lines).strip()


def extract_text(path: Path) -> str:
    ext = path.suffix.lower()
    if ext not in SUPPORTED_TEXT_EXTENSIONS:
        raise ValueError(
            f"Unsupported input format: {ext or '<none>'}. "
            "Current built-in support is txt/md/html/htm."
        )

    raw = path.read_text(encoding="utf-8")
    if ext in {".html", ".htm"}:
        parser = HTMLTextExtractor()
        parser.feed(raw)
        raw = parser.text()
    return normalize_text(raw)


def split_into_paragraphs(text: str) -> list[str]:
    paragraphs = [part.strip() for part in re.split(r"\n\s*\n", text) if part.strip()]
    return paragraphs


def chunk_text(text: str, max_chars: int, overlap_chars: int) -> list[str]:
    paragraphs = split_into_paragraphs(text)
    if not paragraphs:
        return []

    chunks: list[str] = []
    current = ""
    for paragraph in paragraphs:
        candidate = paragraph if not current else f"{current}\n\n{paragraph}"
        if len(candidate) <= max_chars:
            current = candidate
            continue

        if current:
            chunks.append(current)
            overlap = current[-overlap_chars:].strip()
            current = f"{overlap}\n\n{paragraph}" if overlap else paragraph
        else:
            start = 0
            while start < len(paragraph):
                end = min(start + max_chars, len(paragraph))
                chunks.append(paragraph[start:end].strip())
                if end >= len(paragraph):
                    break
                start = max(0, end - overlap_chars)
            current = ""

    if current:
        chunks.append(current)
    return [chunk for chunk in chunks if chunk]


def infer_sections(text: str) -> Iterable[tuple[str, str]]:
    heading_pattern = re.compile(r"^(#{1,6}\s+.+|[A-Z][A-Z0-9 \-:]{4,})$", re.MULTILINE)
    matches = list(heading_pattern.finditer(text))
    if not matches:
        yield "document", text
        return

    for idx, match in enumerate(matches):
        start = match.end()
        end = matches[idx + 1].start() if idx + 1 < len(matches) else len(text)
        heading = match.group(0).lstrip("#").strip().lower()
        section_text = text[start:end].strip()
        if section_text:
            yield heading, section_text


def build_chunks(
    *,
    input_path: Path,
    source: str,
    year: str,
    title: str,
    max_chars: int,
    overlap_chars: int,
) -> list[Chunk]:
    text = extract_text(input_path)
    document_id = slugify(title)

    built: list[Chunk] = []
    page = 1
    chunk_number = 1
    for section_name, section_text in infer_sections(text):
        section_slug = slugify(section_name)
        for chunk_text_value in chunk_text(section_text, max_chars=max_chars, overlap_chars=overlap_chars):
            built.append(
                Chunk(
                    title=title,
                    source=source,
                    year=year,
                    document_id=document_id,
                    page=page,
                    section=section_slug,
                    chunk=chunk_number,
                    source_path=str(input_path.resolve()),
                    text=chunk_text_value,
                )
            )
            chunk_number += 1
        page += 1
    return built


def write_chunks(chunks: list[Chunk], output_dir: Path) -> Path:
    if not chunks:
        raise ValueError("No chunks were produced from the input document.")

    first = chunks[0]
    base_dir = output_dir / slugify(first.source) / first.year / first.document_id
    manifest: list[dict[str, str | int]] = []

    for chunk in chunks:
        page_dir = base_dir / f"p{chunk.page:04d}"
        page_dir.mkdir(parents=True, exist_ok=True)
        filename = f"{chunk.chunk:04d}-{chunk.section}.txt"
        chunk_path = page_dir / filename
        header = (
            f"title: {chunk.title}\n"
            f"source: {chunk.source}\n"
            f"year: {chunk.year}\n"
            f"document_id: {chunk.document_id}\n"
            f"page: {chunk.page}\n"
            f"section: {chunk.section}\n"
            f"chunk: {chunk.chunk}\n"
            f"source_path: {chunk.source_path}\n\n"
        )
        chunk_path.write_text(header + chunk.text + "\n", encoding="utf-8")
        manifest.append(
            {
                "path": str(chunk_path.relative_to(output_dir)),
                "page": chunk.page,
                "section": chunk.section,
                "chunk": chunk.chunk,
            }
        )

    manifest_path = base_dir / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    return base_dir


def build_command(args: argparse.Namespace) -> int:
    input_path = Path(args.input)
    output_dir = Path(args.output)
    title = args.title or input_path.stem

    chunks = build_chunks(
        input_path=input_path,
        source=args.source,
        year=str(args.year),
        title=title,
        max_chars=args.max_chars,
        overlap_chars=args.overlap_chars,
    )
    written_dir = write_chunks(chunks, output_dir)
    print(f"Wrote {len(chunks)} chunks to {written_dir}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Build a GitHub-code-search-friendly text corpus.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    build_parser = subparsers.add_parser("build", help="Convert one document into a chunked corpus layout.")
    build_parser.add_argument("--input", required=True, help="Path to the source document.")
    build_parser.add_argument("--output", default="corpus", help="Directory where the chunked corpus is written.")
    build_parser.add_argument("--source", required=True, help="Corpus source slug, for example arxiv or openstax.")
    build_parser.add_argument("--year", required=True, help="Document year.")
    build_parser.add_argument("--title", help="Override the document title used in metadata and slugs.")
    build_parser.add_argument("--max-chars", type=int, default=1200, help="Maximum chunk size.")
    build_parser.add_argument("--overlap-chars", type=int, default=150, help="Text overlap between chunks.")
    build_parser.set_defaults(func=build_command)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
