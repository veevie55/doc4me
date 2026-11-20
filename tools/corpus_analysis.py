#!/usr/bin/env python3
"""Analyse corpus page-boundary failures: Doc4Me's PDFs against Word's reference PDFs.

Doc4Me's PDFs are the ones `cargo xtask readiness --corpus --dry-run --diagnostics` leaves in
target/readiness/corpus/<id>/doc4me.pdf (or pass --ours DIR holding <id>.pdf); Word's are
corpus/readiness/<id>.pdf. Needs poppler-utils (pdftotext, pdfinfo); `desc` and `pitch` read the
PDF itself (pitch needs pdfminer.six). Read-only: it writes nothing.

  triage LOG          each corpus-breaks failure in a readiness log: ORDER (same words on the page,
                      only pdftotext's order differs), small (a few words cross the boundary), layout
  drift LOG           on the page before each failure, where line positions start to drift
  lines ID PAGE       every line's top on PAGE, Doc4Me beside Word (matched by order)
  match ID PAGE       lines matched by text: Word's top and Doc4Me's offset from it
  ends ID PAGE        the text around the end of PAGE-1 and PAGE in both
  pitch ID PAGES      the most common steps between consecutive lines of one font/size/x
  desc PDF            each embedded font's /Ascent, /Descent and /FontBBox
"""
import argparse, collections, html, re, statistics, subprocess, sys, unicodedata, zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORD = ROOT / "corpus" / "readiness"
OURS = ROOT / "target" / "readiness" / "corpus"


def ours(doc, base):
    return str(Path(base) / f"{doc}.pdf") if base else str(OURS / doc / "doc4me.pdf")


def word(doc):
    return str(WORD / f"{doc}.pdf")


def text(pdf, page, *opts):
    r = subprocess.run(["pdftotext", "-q", "-enc", "UTF-8", *opts, "-f", str(page), "-l", str(page), pdf, "-"], capture_output=True, text=True)
    return r.stdout


def words(pdf, page):
    t = unicodedata.normalize("NFKC", text(pdf, page)).lower()
    return collections.Counter(re.findall(r"\w+", t))


def bbox_lines(pdf, page):
    out = []
    for m in re.finditer(r'<line xMin="([\d.]+)" yMin="([\d.]+)" xMax="[\d.]+" yMax="([\d.]+)">(.*?)</line>', text(pdf, page, "-bbox-layout"), re.S):
        ws = re.findall(r">([^<]*)</word>", m.group(4))
        out.append((float(m.group(2)), float(m.group(3)), html.unescape(" ".join(ws))))
    return sorted(out)


def failures(log):
    for line in open(log, encoding="utf-8"):
        m = re.match(r"corpus-breaks\t(\S+)\tpage (\d+)", line)
        if m:
            yield m.group(1), int(m.group(2))


def triage(a):
    for doc, p in failures(a.log):
        x, y = words(ours(doc, a.ours), p), words(word(doc), p)
        extra, miss = sum((x - y).values()), sum((y - x).values())
        kind = "ORDER" if extra == 0 and miss == 0 else ("small" if extra + miss <= 6 else "layout")
        print(f"{kind:7} {doc:12} p{p:<3} ours+{extra:<4} word+{miss:<4} {' '.join(list((y - x))[:6])} | {' '.join(list((x - y))[:6])}")


def drift(a):
    for doc, p in failures(a.log):
        q = max(1, p - 1)
        x, y = bbox_lines(ours(doc, a.ours), q), bbox_lines(word(doc), q)
        at = {}
        for top, _, t in y:
            at.setdefault(t, top)
        pairs = [(top, at[t], t) for top, _, t in x if t in at and len(t) > 8]
        if len(pairs) < 3:
            print(f"{doc:12} p{q}: few matches")
            continue
        base = statistics.median([pa - pb for pa, pb, _ in pairs[:3]])
        first = next(((pa, t) for pa, pb, t in pairs if abs((pa - pb) - base) > 2.5), None)
        end = pairs[-1][0] - pairs[-1][1] - base
        print(f"{doc:12} p{q}: drift_end={end:+.1f} first_off={'%.0f %s' % (first[0], first[1][:40]) if first else '-'}")


def lines(a):
    x, y = bbox_lines(ours(a.id, a.ours), a.page), bbox_lines(word(a.id), a.page)
    for i in range(max(len(x), len(y))):
        p = x[i] if i < len(x) else (0, 0, "")
        q = y[i] if i < len(y) else (0, 0, "")
        print(f"{p[0]:7.1f} {p[1] - p[0]:5.1f} {p[2][:38]:38} | {q[0]:7.1f} {q[1] - q[0]:5.1f} {q[2][:38]:38} d={p[0] - q[0]:+.1f}")


def match(a):
    x = {}
    for top, _, t in bbox_lines(ours(a.id, a.ours), a.page):
        x.setdefault(t[:30], top)
    for top, _, t in bbox_lines(word(a.id), a.page):
        k = t[:30]
        print(f"{top:7.1f} {(x[k] - top) if k in x else float('nan'):+6.1f} {k}")


def ends(a):
    for name, pdf in (("doc4me", ours(a.id, a.ours)), ("word", word(a.id))):
        info = subprocess.run(["pdfinfo", pdf], capture_output=True, text=True).stdout
        pages = re.search(r"Pages:\s+(\d+)", info)
        print(f"=== {name} {pages.group(1) if pages else '?'} pages")
        for label, page, cut in ((f"p{a.page - 1} end", a.page - 1, slice(-4, None)), (f"p{a.page} start", a.page, slice(0, 4)), (f"p{a.page} end", a.page, slice(-3, None))):
            body = [l[:120] for l in text(pdf, page, "-layout").splitlines() if l.strip()]
            print(f"-- {label}")
            print("\n".join(body[cut]))


def pitch(a):
    from pdfminer.high_level import extract_pages
    from pdfminer.layout import LTChar, LTTextBox, LTTextLine

    for name, pdf in (("doc4me", ours(a.id, a.ours)), ("word", word(a.id))):
        found = []
        for pi, pg in enumerate(extract_pages(pdf, page_numbers=list(range(a.pages)))):
            for box in pg:
                if not isinstance(box, LTTextBox):
                    continue
                for line in box:
                    chars = [c for c in line if isinstance(c, LTChar)] if isinstance(line, LTTextLine) else []
                    if chars:
                        c = chars[0]
                        found.append((pi, round(pg.height - c.y0, 2), round(c.size, 1), c.fontname.split("+")[-1][:18], round(c.x0)))
        found.sort()
        steps = collections.Counter()
        for p, q in zip(found, found[1:]):
            if p[0] == q[0] and p[2:] == q[2:] and 0 < q[1] - p[1] < 20:
                steps[(p[3], p[2], round(q[1] - p[1], 1))] += 1
        print(name, steps.most_common(8))


def desc(a):
    data = Path(a.pdf).read_bytes()
    chunks = [data]
    for m in re.finditer(rb"stream\r?\n(.*?)endstream", data, re.S):
        try:
            chunks.append(zlib.decompress(m.group(1)))
        except zlib.error:
            pass
    seen = set()
    for c in chunks:
        for m in re.finditer(rb"/Type\s*/FontDescriptor", c):
            seg = c[max(0, m.start() - 600) : m.start() + 600]
            seg = seg[seg.rfind(b"<<", 0, 600) :]
            get = lambda k: (re.search(k, seg) or [None, b"?"])[1].decode("latin-1")
            seen.add((get(rb"/FontName\s*/([^\s/<>]+)").split("+")[-1], get(rb"/Ascent\s+(-?[\d.]+)"), get(rb"/Descent\s+(-?[\d.]+)"), get(rb"/FontBBox\s*\[([^\]]*)\]")))
    for s in sorted(seen):
        print(*s, sep="\t")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--ours", help="directory with Doc4Me's <id>.pdf (default: the readiness run's)")
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name, f in (("triage", triage), ("drift", drift)):
        s = sub.add_parser(name)
        s.add_argument("log")
        s.set_defaults(f=f)
    for name, f in (("lines", lines), ("match", match), ("ends", ends)):
        s = sub.add_parser(name)
        s.add_argument("id")
        s.add_argument("page", type=int)
        s.set_defaults(f=f)
    s = sub.add_parser("pitch")
    s.add_argument("id")
    s.add_argument("pages", type=int)
    s.set_defaults(f=pitch)
    s = sub.add_parser("desc")
    s.add_argument("pdf")
    s.set_defaults(f=desc)
    a = ap.parse_args()
    try:
        a.f(a)
    except BrokenPipeError:
        # Output piped into `head`: stop quietly.
        sys.stderr.close()


if __name__ == "__main__":
    sys.exit(main())
