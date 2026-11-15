#!/usr/bin/env python3
"""Refresh the current bilingual readiness blocker inventory from the generated report."""
from __future__ import annotations

import csv
import os
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "docs/readiness.md"
EN = ROOT / "docs/readiness/blockers-2026-11-14.md"
TH = ROOT / "docs/readiness/blockers-2026-11-14.th.md"
TSV = ROOT / "docs/readiness/blockers-2026-11-14.tsv"
BEGIN = "<!-- BEGIN CURRENT READINESS BLOCKERS -->"
END = "<!-- END CURRENT READINESS BLOCKERS -->"


def require(pattern: str, text: str, label: str) -> re.Match[str]:
    match = re.search(pattern, text, re.MULTILINE)
    if not match:
        raise ValueError(f"readiness report is missing {label}")
    return match


def failures(section: str, kind: str) -> list[tuple[str, str]]:
    result: list[tuple[str, str]] = []
    for line in section.splitlines():
        if not line.startswith("- `"):
            continue
        match = re.match(r"- `([^`]+)`\s+(.+)$", line)
        if match:
            result.append((match.group(1), match.group(2)))
    header = require(r"^### " + re.escape(kind) + r" \((\d+) of (\d+)\)$", section, f"{kind} count")
    expected, total = map(int, header.groups())
    if len(result) != expected:
        raise ValueError(f"{kind} has {len(result)} listed failures, expected {expected}")
    return result


def report_section(text: str, kind: str) -> str:
    start = text.find(f"### {kind} (")
    if start < 0:
        raise ValueError(f"readiness report is missing {kind}")
    next_section = text.find("\n### ", start + 1)
    return text[start:] if next_section < 0 else text[start:next_section]


def replace_current(path: Path, title: str, summary: str, body: str, historical_heading: str, source_url: str) -> None:
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    lines[0] = title
    for index, line in enumerate(lines):
        if line.startswith("> Latest measurement:") or line.startswith("> Latest completed measurement:") or line.startswith("> ผลวัดล่าสุด:") or line.startswith("> ผลวัดล่าสุดที่เสร็จแล้ว:"):
            lines[index] = summary
        if line.startswith("Evidence: [Windows run log]"):
            evidence = f"[Windows run log]({source_url})" if source_url else "Local measurement"
            lines[index] = f"Evidence: {evidence} and [reference audit](reference-audit-2026-11-12.md). Each current item states measured facts; no check was removed or relaxed."
        elif line.startswith("หลักฐาน: [Windows run log]"):
            evidence = f"[Windows run log]({source_url})" if source_url else "ผลวัดในเครื่อง"
            lines[index] = f"หลักฐาน: {evidence} และ [ตรวจสอบ reference](reference-audit-2026-11-12.md); รายการปัจจุบันระบุผลที่วัดได้ ไม่มีการลบหรือผ่อนเกณฑ์"
    text = "\n".join(lines) + "\n"
    if BEGIN in text and END in text:
        before, remainder = text.split(BEGIN, 1)
        _, after = remainder.split(END, 1)
        text = before + BEGIN + "\n\n" + body.rstrip() + "\n\n" + END + after
    else:
        anchor = text.find(historical_heading)
        if anchor < 0:
            raise ValueError(f"{path} is missing historical section {historical_heading!r}")
        text = text[:anchor] + BEGIN + "\n\n" + body.rstrip() + "\n\n" + END + "\n\n" + text[anchor:]
    path.write_text(text, encoding="utf-8")


def main() -> None:
    report = REPORT.read_text(encoding="utf-8")
    run = os.environ.get("GITHUB_RUN_ID") or os.environ.get("READINESS_RUN_ID") or "local"
    source = require(r"at commit `([^`]+)`", report, "source commit").group(1)
    overall = require(r"\*\*Measured readiness: ([^*]+)\*\*", report, "overall score").group(1)
    without = require(r"\*\*Without the deferred rows: ([^*]+)\*\*", report, "without-deferred score").group(1)
    pages_section = report_section(report, "corpus-pages")
    breaks_section = report_section(report, "corpus-breaks")
    page_failures = failures(pages_section, "corpus-pages")
    break_failures = failures(breaks_section, "corpus-breaks")
    page_total = int(require(r"^### corpus-pages \(\d+ of (\d+)\)$", pages_section, "page denominator").group(1))
    break_total = int(require(r"^### corpus-breaks \(\d+ of (\d+)\)$", breaks_section, "boundary denominator").group(1))

    source_url = f"https://github.com/veevie55/doc4me/actions/runs/{run}" if run.isdecimal() else ""
    run_ref = f"[{run}]({source_url})" if source_url else f"`{run}`"
    measurement = f"Windows run {run_ref}" if source_url else f"local measurement {run_ref}"
    context = "Windows" if source_url else "local"
    en_summary = (
        f"> Latest measurement: {measurement}, source `{source}`: {overall} overall / "
        f"{without} excluding deferred; page counts {page_total - len(page_failures)}/{page_total}, "
        f"boundaries {break_total - len(break_failures)}/{break_total}; {len(page_failures)} page-count and "
        f"{len(break_failures)} boundary failures. Check rules and corpus membership are unchanged."
    )
    th_summary = (
        f"> ผลวัดล่าสุด: {measurement}, source `{source}`: {overall} รวม / {without} "
        f"เมื่อไม่นับ deferred; page count {page_total - len(page_failures)}/{page_total}, "
        f"page boundary {break_total - len(break_failures)}/{break_total}; จำนวนหน้าไม่ผ่าน {len(page_failures)} แถว "
        f"และขอบเขตหน้าไม่ผ่าน {len(break_failures)} แถว. ไม่เปลี่ยนเกณฑ์หรือสมาชิก corpus."
    )

    en: list[str] = [
        f"## Current {context} failures — run {run}",
        "",
        f"Measured readiness is {overall} overall and {without} excluding deferred rows. The corpus has "
        f"{page_total - len(page_failures)}/{page_total} page-count passes and "
        f"{break_total - len(break_failures)}/{break_total} page-boundary passes. Each item below failed "
        "the existing exact-match check; this measurement alone does not establish its root cause. No row "
        "was removed, excluded, or relaxed.",
        "",
        f"### corpus-pages ({len(page_failures)} of {page_total})",
        "",
    ]
    en.extend(f"- `{identifier}` {observed}" for identifier, observed in page_failures)
    en.extend(["", f"### corpus-breaks ({len(break_failures)} of {break_total})", ""])
    en.extend(f"- `{identifier}` {observed}" for identifier, observed in break_failures)

    th: list[str] = [
        f"## รายการที่ยังไม่ผ่าน ({context}) — run {run}",
        "",
        f"ผล readiness คือ {overall} รวม และ {without} เมื่อไม่นับ deferred; corpus ผ่าน page count "
        f"{page_total - len(page_failures)}/{page_total} รายการ และ page boundary "
        f"{break_total - len(break_failures)}/{break_total} รายการ. ทุกรายการด้านล่างไม่ผ่านการเทียบตรงตามเกณฑ์เดิม; "
        "ผลวัดนี้เพียงอย่างเดียวยังยืนยันสาเหตุรากไม่ได้ ไม่มีการลบแถว ตั้งเป็น excluded หรือผ่อนเกณฑ์",
        "",
        f"### corpus-pages ({len(page_failures)} จาก {page_total})",
        "",
    ]
    th.extend(f"- `{identifier}` {observed}" for identifier, observed in page_failures)
    th.extend(["", f"### corpus-breaks ({len(break_failures)} จาก {break_total})", ""])
    th.extend(f"- `{identifier}` {observed}" for identifier, observed in break_failures)

    replace_current(
        EN,
        f"# Readiness blockers — Windows run {run}" if source_url else "# Readiness blockers — latest local measurement",
        en_summary,
        "\n".join(en),
        "## Original target cohort:",
        source_url,
    )
    replace_current(
        TH,
        f"# อุปสรรค readiness — Windows run {run}" if source_url else "# อุปสรรค readiness — ผลวัดในเครื่องล่าสุด",
        th_summary,
        "\n".join(th),
        "## ชุดเป้าหมายเดิม:",
        source_url,
    )

    with TSV.open("w", encoding="utf-8", newline="") as file:
        writer = csv.writer(file, delimiter="\t", lineterminator="\n")
        writer.writerow(["check", "id", "observed", "font_substitutions", "blocker_en", "blocker_th", "run", "source"])
        for check, items in (("corpus-pages", page_failures), ("corpus-breaks", break_failures)):
            for identifier, observed in items:
                detail_en = f"Run {run} measured {observed}. The strict comparison failed; this check does not identify a verified root cause or justify an exclusion."
                detail_th = f"run {run} วัดพบ {observed} การเทียบตามเกณฑ์เคร่งครัดไม่ผ่าน; ชุดตรวจนี้ยังไม่ยืนยันสาเหตุรากหรือเหตุผลให้ excluded."
                writer.writerow([check, identifier, observed, "", detail_en, detail_th, run, source])


if __name__ == "__main__":
    main()
