# GBS

`GBS` is a zero-infra MVP for searchable document corpora on top of GitHub code search.

The core idea is simple:

1. Convert legal source documents into clean text.
2. Split them into search-friendly chunks.
3. Write each chunk as a small `.txt` file in a deterministic repo layout.
4. Push the corpus to GitHub and use GitHub code search as the full-text engine.

This is not a perfect search engine. It is a pragmatic hack that works with:

- `0$` infrastructure cost
- public, legal corpora
- GitHub's existing indexing and search UI

## Why this exists

General web search is bad at:

- exact full-text matching inside long documents
- showing all occurrences
- searching across books and papers as text
- filtering by source, section, page, year, or document id

GitHub already solves indexing and query execution for public text repositories. The trick is to make books and papers look like code search inputs.

## Repository layout

```text
corpus/
  source_slug/
    year/
      document_slug/
        p0001/
          0001-introduction.txt
          0002-introduction.txt
        p0002/
          0001-methods.txt
```

Each chunk file starts with a metadata header:

```text
title: Example Paper
source: arxiv
year: 2024
document_id: example-paper
page: 1
section: introduction
chunk: 1
source_path: /abs/path/to/file.txt

Actual searchable text starts here.
```

## Current scope

Supported now:

- `txt`
- `md`
- `html`
- `htm`
- `arXiv` PDF import via `scripts/import_arxiv.py`
- `Project Gutenberg` bulk import via `scripts/import_gutendex_technical_books.py`

Optional later:

- `pdf` via `pymupdf`
- `epub` via `ebooklib` + `beautifulsoup4`

## Quick start

```bash
cd /root/projects/GBS
python -m gbs build \
  --input examples/example_paper.txt \
  --output corpus \
  --source demo \
  --year 2026 \
  --title "Example Paper"
```

Bulk import open corpora:

```bash
cd /root/projects/GBS
python scripts/import_arxiv.py 'cat:cs.AI OR cat:cs.LG OR cat:cs.CL OR cat:cs.CV' 25
python -u scripts/import_gutendex_technical_books.py
```

`import_arxiv.py` now produces two outputs:

- `corpus/arxiv/...` for full-text search
- `feeds/arxiv_daily/YYYYMMDD.json` for a ranked metadata feed

## Next steps

1. Add `pdf` and `epub` extraction.
2. Add a metadata manifest per document.
3. Add a script that pushes the generated corpus to a public GitHub repo.
4. Add a tiny frontend that generates GitHub code search links and filters.
