#!/usr/bin/env python3
"""Print a font's advance widths from the pinned corpus PDFs, as Rust tables (Aptos Narrow by default).

Reads only public layout facts of the embedded fonts in Word's reference PDFs: `/FirstChar` and
`/Widths` of WinAnsi simple fonts, and `/W` with the `/ToUnicode` map of Identity-H composite ones
(no font program or outline is read or copied), checks each PDF's SHA-256 against docs/readiness/corpus.tsv, keeps Latin
letters, digits and punctuation with a width, and refuses conflicting measurements.
Requires pypdf.

    python3 tools/aptos_narrow_pdf_metrics.py > /tmp/aptos_narrow.rs
    python3 tools/aptos_narrow_pdf_metrics.py DENGXIAN DengXian zenodo-056 > /tmp/dengxian.rs

With arguments: the constant prefix, the PDF base font name (regular; "<name>,Bold" or "<name>-Bold"
is the bold), then the corpus ids to read.
"""

import csv
import hashlib
import pathlib
import re
import sys

from pypdf import PdfReader

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCES = ["zenodo-154", "zenodo-097"]
STYLES = {"Aptos Narrow": "APTOS_NARROW_REGULAR", "Aptos Narrow,Bold": "APTOS_NARROW_BOLD"}


def manifest():
    lines = (line for line in (ROOT / "docs/readiness/corpus.tsv").read_text(encoding="utf-8").splitlines() if not line.startswith("#"))
    return {r["id"]: r for r in csv.DictReader(lines, delimiter="\t")}


def unicode_map(font):
    """CID → Unicode string from a composite font's /ToUnicode bfchar/bfrange entries."""
    cmap = {}
    tu = font.get("/ToUnicode")
    if tu is None:
        return cmap
    text = tu.get_object().get_data().decode("latin-1")
    for block in re.findall(r"beginbfchar(.*?)endbfchar", text, re.S):
        for a, b in re.findall(r"<([0-9A-Fa-f]+)>\s*<([0-9A-Fa-f]+)>", block):
            cmap[int(a, 16)] = bytes.fromhex(b).decode("utf-16-be", "replace")
    for block in re.findall(r"beginbfrange(.*?)endbfrange", text, re.S):
        for a, b, c in re.findall(r"<([0-9A-Fa-f]+)>\s*<([0-9A-Fa-f]+)>\s*<([0-9A-Fa-f]+)>", block):
            start = int(c, 16)
            for k, cid in enumerate(range(int(a, 16), int(b, 16) + 1)):
                cmap[cid] = chr(start + k)
    return cmap


def cid_widths(font):
    """CID → width from a composite font's descendant /W array."""
    desc = font["/DescendantFonts"].get_object()[0].get_object()
    w = desc.get("/W")
    out = {}
    if w is None:
        return out
    w = [x.get_object() if hasattr(x, "get_object") else x for x in w.get_object()]
    i = 0
    while i < len(w):
        first = int(w[i])
        nxt = w[i + 1]
        if isinstance(nxt, list) or hasattr(nxt, "__iter__"):
            for k, v in enumerate(nxt):
                out[first + k] = int(v)
            i += 2
        else:
            for cid in range(first, int(nxt) + 1):
                out[cid] = int(w[i + 2])
            i += 3
    return out


def main():
    global SOURCES, STYLES
    if len(sys.argv) >= 4:
        prefix, name = sys.argv[1], sys.argv[2]
        SOURCES = sys.argv[3:]
        STYLES = {name: f"{prefix}_REGULAR", f"{name},Bold": f"{prefix}_BOLD", f"{name}-Bold": f"{prefix}_BOLD"}
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
                if str(font.get("/Subtype")) == "/Type0" and str(font.get("/Encoding")) == "/Identity-H":
                    uni = unicode_map(font)
                    pairs = [(uni[cid], w) for cid, w in cid_widths(font).items() if len(uni.get(cid, "")) == 1]
                elif str(font.get("/Encoding")) == "/WinAnsiEncoding" and str(font.get("/Subtype")) == "/TrueType":
                    first = int(font["/FirstChar"])
                    pairs = [(chr(first + i), int(w)) for i, w in enumerate(font["/Widths"].get_object()) if 32 <= first + i <= 126]
                else:
                    continue
                for c, w in pairs:
                    if w <= 0 or not (32 <= ord(c) <= 126 or 0xA0 <= ord(c) <= 0x24F or ord(c) in (0x2013, 0x2014, 0x2018, 0x2019, 0x201C, 0x201D, 0x2026) or 0x370 <= ord(c) <= 0x3FF or 0x2070 <= ord(c) <= 0x209F):
                        continue
                    old = widths[style].setdefault(c, w)
                    if old != w:
                        sys.exit(f"{doc}: {name} {c!r} is {w}, elsewhere {old}")
    for style, table in widths.items():
        if not table:
            continue
        print(f"pub(crate) const {style}: &[(char, u16)] = &[")
        for c in sorted(table):
            print(f"    ('\\u{{{ord(c):x}}}', {table[c]}),")
        print("];")


if __name__ == "__main__":
    main()
