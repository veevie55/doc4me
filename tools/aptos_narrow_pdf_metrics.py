#!/usr/bin/env python3
"""Print Aptos Narrow advance widths from the pinned corpus PDFs, as Rust tables.

Reads only the public `/FirstChar`, `/Widths` and WinAnsi encoding facts of the embedded
"Aptos Narrow" and "Aptos Narrow,Bold" simple fonts in Word's reference PDFs (no font program or
outline is read or copied), checks each PDF's SHA-256 against docs/readiness/corpus.tsv, keeps the
printable ASCII codes (identical in WinAnsi) with a width, and refuses conflicting measurements.
Requires pypdf.

    python3 tools/aptos_narrow_pdf_metrics.py > /tmp/aptos_narrow.rs
"""

import csv
import hashlib
import pathlib
import sys

from pypdf import PdfReader

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCES = ["zenodo-154", "zenodo-097"]
STYLES = {"Aptos Narrow": "APTOS_NARROW_REGULAR", "Aptos Narrow,Bold": "APTOS_NARROW_BOLD"}


def manifest():
    lines = (line for line in (ROOT / "docs/readiness/corpus.tsv").read_text(encoding="utf-8").splitlines() if not line.startswith("#"))
    return {r["id"]: r for r in csv.DictReader(lines, delimiter="\t")}


def main():
    rows = manifest()
    widths = {style: {} for style in STYLES.values()}
    for doc in SOURCES:
        path = ROOT / f"corpus/readiness/{doc}.pdf"
        if hashlib.sha256(path.read_bytes()).hexdigest() != rows[doc]["pdf_sha256"]:
            sys.exit(f"{path} does not match the corpus manifest")
        for page in PdfReader(path).pages:
            fonts = page["/Resources"].get("/Font", {})
            for ref in fonts.values():
                font = ref.get_object()
                base = str(font.get("/BaseFont", "")).lstrip("/")
                name = base.split("+", 1)[-1]
                style = STYLES.get(name)
                if style is None:
                    continue
                if str(font.get("/Encoding")) != "/WinAnsiEncoding" or str(font.get("/Subtype")) != "/TrueType":
                    sys.exit(f"{doc}: {base} is not a WinAnsi TrueType simple font")
                first = int(font["/FirstChar"])
                for i, w in enumerate(font["/Widths"].get_object()):
                    code, w = first + i, int(w)
                    if w <= 0 or not 32 <= code <= 126:
                        continue
                    c = chr(code)
                    old = widths[style].setdefault(c, w)
                    if old != w:
                        sys.exit(f"{doc}: {name} {c!r} is {w}, elsewhere {old}")
    for style, table in widths.items():
        print(f"pub(crate) const {style}: &[(char, u16)] = &[")
        for c in sorted(table):
            print(f"    ('\\u{{{ord(c):x}}}', {table[c]}),")
        print("];")


if __name__ == "__main__":
    main()
