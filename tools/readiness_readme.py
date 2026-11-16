#!/usr/bin/env python3
"""Keep the readiness numbers in README.md (Thai) and README.en.md equal to docs/readiness.md.

docs/readiness.th.md is rendered from the same report (tools/readiness_th.py), so all four files
carry the same run, source commit, scores, page-count and page-boundary results and failure
counts. The Windows readiness workflow runs this after each measurement; run it by hand after a
report changes. `--check` changes nothing and exits 1 when a README is out of date.

The run comes from GITHUB_RUN_ID or READINESS_RUN_ID, else from the current blocker inventory's
heading (tools/readiness_blockers.py writes it from the same run).
"""
from __future__ import annotations

import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "docs/readiness.md"
BLOCKERS = ROOT / "docs/readiness/blockers-2026-11-14.md"
REPO_RUNS = "https://github.com/veevie55/doc4me/actions/runs"


def require(pattern: str, text: str, label: str) -> re.Match[str]:
    match = re.search(pattern, text, re.MULTILINE)
    if not match:
        raise ValueError(f"missing {label}")
    return match


def numbers() -> dict[str, str]:
    report = REPORT.read_text(encoding="utf-8")
    run = os.environ.get("GITHUB_RUN_ID") or os.environ.get("READINESS_RUN_ID")
    if not run:
        heading = BLOCKERS.read_text(encoding="utf-8")
        run = require(r"^# Readiness blockers — Windows run (\d+)$", heading, "the blocker inventory's run").group(1)
    if not run.isdecimal():
        raise ValueError(f"run id {run!r} is not a number")
    pages = require(r"corpus-pages (\d+)/(\d+)", report, "corpus-pages")
    breaks = require(r"corpus-breaks (\d+)/(\d+)", report, "corpus-breaks")
    return {
        "run": run,
        "source": require(r"at commit `([0-9a-f]{7,40})`", report, "source commit").group(1),
        "overall": require(r"^\*\*Measured readiness: ([0-9.]+%)\*\*", report, "overall readiness").group(1),
        "ahead": require(r"^\*\*Without the deferred rows: ([0-9.]+%)\*\*", report, "readiness without deferred").group(1),
        "deferred": require(r"Without the deferred rows: [0-9.]+%\*\* — (\d+) rows", report, "deferred row count").group(1),
        "pages": pages.group(1),
        "breaks": breaks.group(1),
        "total": pages.group(2),
        "page_failures": str(int(pages.group(2)) - int(pages.group(1))),
        "break_failures": str(int(breaks.group(2)) - int(breaks.group(1))),
    }


def lines(n: dict[str, str]) -> dict[Path, list[tuple[str, str]]]:
    url = f"{REPO_RUNS}/{n['run']}"
    return {
        ROOT / "README.md": [
            ("ผลวัด Windows ทางการล่าสุด (", f"ผลวัด Windows ทางการล่าสุด (run {n['run']}, source `{n['source']}`): **{n['overall']}** หรือ **{n['ahead']} เมื่อไม่นับ {n['deferred']} รายการ deferred**"),
            (f"คลังเอกสาร {n['total']} ไฟล์เปิดและบันทึกกลับได้ครบ", f"คลังเอกสาร {n['total']} ไฟล์เปิดและบันทึกกลับได้ครบ จำนวนหน้าตรง {n['pages']}/{n['total']} และขอบเขตหน้าตรง {n['breaks']}/{n['total']}"),
            (
                "ผลวัด Windows ล่าสุด (run [",
                f"ผลวัด Windows ล่าสุด (run [{n['run']}]({url}), source `{n['source']}`): readiness {n['overall']} รวม /{n['ahead']} ไม่รวม {n['deferred']} deferred, "
                f"จำนวนหน้า {n['pages']}/{n['total']} และขอบเขตหน้า {n['breaks']}/{n['total']}; ยังมี page-count {n['page_failures']} รายการและ boundary {n['break_failures']} รายการที่ไม่ผ่าน "
                "ตาม [รายงาน](docs/readiness.th.md) และ [การวิเคราะห์ blocker ไทย/อังกฤษ](docs/readiness/blockers-2026-11-14.th.md).",
            ),
        ],
        ROOT / "README.en.md": [
            ("The latest official Windows measurement (", f"The latest official Windows measurement (run {n['run']}, source `{n['source']}`) is **{n['overall']}**, or **{n['ahead']} excluding {n['deferred']} deferred checks**."),
            (
                f"All {n['total']} corpus documents open and round-trip;",
                f"All {n['total']} corpus documents open and round-trip; page counts match for {n['pages']}/{n['total']} and page boundaries for {n['breaks']}/{n['total']}. "
                "See the [readiness report](docs/readiness.md) and [bilingual blocker assessment](docs/readiness/blockers-2026-11-14.md) for remaining failures and their evidence.",
            ),
            (
                "Latest Windows measurement: [run ",
                f"Latest Windows measurement: [run {n['run']}]({url}), source `{n['source']}`; readiness is {n['overall']} overall / {n['ahead']} without {n['deferred']} deferred rows, "
                f"with page counts {n['pages']}/{n['total']} and page boundaries {n['breaks']}/{n['total']}. There are still {n['page_failures']} page-count and {n['break_failures']} boundary failures. "
                "See the [report](docs/readiness.md) and [bilingual blocker assessment](docs/readiness/blockers-2026-11-14.md).",
            ),
        ],
    }


def refresh(path: Path, wanted: list[tuple[str, str]]) -> tuple[str, str]:
    """The file's text before and after replacing each line that starts with a known prefix."""
    before = path.read_text(encoding="utf-8")
    out = before.split("\n")
    for prefix, line in wanted:
        hits = [i for i, l in enumerate(out) if l.startswith(prefix)]
        if len(hits) != 1:
            raise ValueError(f"{path.name}: expected one line starting {prefix!r}, found {len(hits)}")
        out[hits[0]] = line
    return before, "\n".join(out)


def main() -> int:
    check = sys.argv[1:] == ["--check"]
    if sys.argv[1:] and not check:
        print("usage: readiness_readme.py [--check]", file=sys.stderr)
        return 2
    stale = []
    for path, wanted in lines(numbers()).items():
        before, after = refresh(path, wanted)
        if before != after:
            stale.append(path.name)
            if not check:
                path.write_text(after, encoding="utf-8")
    if check and stale:
        print(f"out of date with docs/readiness.md: {', '.join(stale)}", file=sys.stderr)
        return 1
    print(("updated " + ", ".join(stale)) if stale and not check else "README readiness numbers match docs/readiness.md")
    return 0


if __name__ == "__main__":
    sys.exit(main())
