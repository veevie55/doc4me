#!/usr/bin/env python3
"""Print Calibri Light advance widths declared by Word in pinned corpus PDFs, as Rust tables.

Reads only each simple TrueType font's /FirstChar, /Widths and WinAnsi encoding (public layout
numbers, 1000 units per em); never an embedded font program or outline. Every PDF is checked
against the corpus manifest, and a character whose width differs between them stops the script.
Requires pypdf. Output: the CALIBRI_LIGHT_* tables in crates/fonts/src/latin_metrics.rs. Carlito,
the open stand-in for Calibri, is wider than Calibri Light (A 579 vs 563, / 386 vs 362 per 1000 em).
"""
from __future__ import annotations

import csv
import hashlib
from pathlib import Path

from pypdf import PdfReader

ROOT = Path(__file__).resolve().parents[1]
SOURCES = ["govuk-033", "zenodo-006", "zenodo-008", "zenodo-046", "zenodo-049", "zenodo-052", "zenodo-053", "zenodo-055", "zenodo-096", "zenodo-122", "zenodo-138", "zenodo-143", "zenodo-147", "zenodo-149", "zenodo-151", "zenodo-158"]
STYLES = {"Calibri-Light": "CALIBRI_LIGHT_REGULAR", "Calibri-LightItalic": "CALIBRI_LIGHT_ITALIC"}


def obj(x):
    return x.get_object() if hasattr(x, "get_object") else x


def main() -> None:
    lines = (ROOT / "docs/readiness/corpus.tsv").read_text(encoding="utf-8").splitlines()
    manifest = {r["id"]: r for r in csv.DictReader((l for l in lines if not l.startswith("#")), delimiter="\t")}
    widths: dict[str, dict[str, int]] = {name: {} for name in STYLES.values()}
    for doc in SOURCES:
        path = ROOT / f"corpus/readiness/{doc}.pdf"
        if hashlib.sha256(path.read_bytes()).hexdigest() != manifest[doc]["pdf_sha256"]:
            raise ValueError(f"{doc}.pdf does not match the corpus manifest")
        for page in PdfReader(path).pages:
            fonts = obj(obj(page.get("/Resources", {})).get("/Font", {}))
            for ref in fonts.values():
                font = obj(ref)
                base = str(font.get("/BaseFont", "")).split("+")[-1]
                if base not in STYLES or str(font.get("/Subtype")) != "/TrueType":
                    continue
                if str(obj(font.get("/Encoding"))) != "/WinAnsiEncoding":
                    raise ValueError(f"{doc}: {base} is not WinAnsi encoded")
                first = int(font["/FirstChar"])
                table = widths[STYLES[base]]
                for i, w in enumerate(obj(font["/Widths"])):
                    w = int(w)
                    if w <= 0:
                        continue
                    try:
                        c = bytes([first + i]).decode("cp1252")
                    except UnicodeDecodeError:
                        continue
                    if table.get(c, w) != w:
                        raise ValueError(f"{base} {c!r}: {table[c]} in one PDF, {w} in another")
                    table[c] = w
    for name, table in widths.items():
        print(f"pub(crate) const {name}: &[(char, u16)] = &[")
        for c in sorted(table):
            print(f"    ('\\u{{{ord(c):x}}}', {table[c]}),")
        print("];")


if __name__ == "__main__":
    main()
