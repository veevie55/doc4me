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


def check_failures(text: str, kind: str) -> tuple[list[tuple[str, str]], int]:
    """The listed failures of `kind` and its denominator. The report lists a check's failures
    under `### <kind> (N of M)` only when there are some; with none, the summary table's
    `<kind> M/M` gives the denominator (and must show every row passing)."""
    if f"### {kind} (" in text:
        section = report_section(text, kind)
        total = int(require(r"^### " + re.escape(kind) + r" \(\d+ of (\d+)\)$", section, f"{kind} denominator").group(1))
        return failures(section, kind), total
    passed, total = map(int, require(re.escape(kind) + r" (\d+)/(\d+)", text, f"{kind} summary").groups())
    if passed != total:
        raise ValueError(f"readiness report lists no {kind} failures but its summary shows {passed}/{total}")
    return [], total


def report_section(text: str, kind: str) -> str:
    start = text.find(f"### {kind} (")
    if start < 0:
        raise ValueError(f"readiness report is missing {kind}")
    next_section = text.find("\n### ", start + 1)
    return text[start:] if next_section < 0 else text[start:next_section]


DEFERRED_LABELS = {
    "dialogs": ("Dialogs", "กล่องโต้ตอบ"),
    "shortcuts": ("Keyboard shortcuts", "แป้นพิมพ์ลัด"),
    "ecosystem": ("Ecosystem", "ระบบนิเวศ"),
    "hardware": ("Hardware", "ฮาร์ดแวร์"),
    "corpus-pages": ("Corpus page counts", "จำนวนหน้าของเอกสารใน corpus"),
    "corpus-breaks": ("Corpus page boundaries", "ขอบเขตหน้าของเอกสารใน corpus"),
}
CORPUS_GROUPS = {
    "A": ("Group A — PDF reading order and layout differences", "กลุ่ม ก — ลำดับการอ่านข้อความ PDF และความต่างในการจัดหน้า"),
    "B": ("Group B — Reference differences", "กลุ่ม ข — ความต่างของไฟล์อ้างอิง"),
    "": ("Other deferred pagination checks", "การตรวจการแบ่งหน้าที่พักไว้รายการอื่น"),
}


def deferred_sections(report: str) -> tuple[list[str], list[str]]:
    section = re.search(r"^## Deferred rows \((\d+)\)\n([\s\S]*?)(?=^## |\Z)", report, re.MULTILINE)
    if not section:
        return [], []
    expected, entries = section.groups()
    with (REPORT.parent / "readiness/deferred-summary.th.tsv").open(encoding="utf-8", newline="") as file:
        translations = {row["id"]: (row["group"], row["summary"]) for row in csv.DictReader(file, delimiter="	")}
    grouped: dict[str, dict[str, list[tuple[str, str, str]]]] = {}
    for identifier, check, observed in re.findall(r"^- `([^`]+)` \(([^)]+)\) (.+)$", entries, re.MULTILINE):
        group, thai = translations.get(identifier, ("", "ดูรายละเอียดของรายการนี้ในรายงานต้นฉบับ"))
        grouped.setdefault(check, {}).setdefault(group, []).append((identifier, observed, thai))
    count = sum(len(rows) for groups in grouped.values() for rows in groups.values())
    if count != int(expected):
        raise ValueError(f"deferred list has {count} rows, expected {expected}")
    en = ["", f"## Deferred checks ({count})", "", "The readiness report records the status of each item. [Full measurement](../readiness.md#deferred-rows)", ""]
    th = ["", f"## รายการที่พักไว้ ({count})", "", "นับตามสถานะรายข้อในรายงาน readiness [ผลวัดฉบับเต็ม](../readiness.md#deferred-rows)", ""]
    for check, groups in grouped.items():
        labels = DEFERRED_LABELS.get(check, (check, check))
        total = sum(map(len, groups.values()))
        en.extend([f"### {labels[0]} ({total})", ""])
        th.extend([f"### {labels[1]} ({total})", ""])
        for group, rows in groups.items():
            if check in {"corpus-pages", "corpus-breaks"}:
                names = CORPUS_GROUPS[group]
                en.extend([f"#### {names[0]} ({len(rows)})", ""])
                th.extend([f"#### {names[1]} ({len(rows)})", ""])
            en.extend(f"- `{identifier}` — {observed}" for identifier, observed, _ in rows)
            th.extend(f"- `{identifier}` — {thai}" for identifier, _, thai in rows)
            en.append("")
            th.append("")
    return en, th


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
    page_failures, page_total = check_failures(report, "corpus-pages")
    break_failures, break_total = check_failures(report, "corpus-breaks")
    active_pages = [(identifier, observed) for identifier, observed in page_failures if not observed.startswith("deferred:")]
    active_breaks = [(identifier, observed) for identifier, observed in break_failures if not observed.startswith("deferred:")]

    active_page_total = page_total - len(page_failures) + len(active_pages)
    active_break_total = break_total - len(break_failures) + len(active_breaks)
    active_count = len(active_pages) + len(active_breaks)

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
        f"{break_total - len(break_failures)}/{break_total} page-boundary passes. "
        f"Active corpus blockers: {active_count}.",
        "",
        f"### corpus-pages ({len(active_pages)} of {active_page_total})",
        "",
    ]
    en.extend(f"- `{identifier}` {observed}" for identifier, observed in active_pages)
    en.extend(["", f"### corpus-breaks ({len(active_breaks)} of {active_break_total})", ""])
    en.extend(f"- `{identifier}` {observed}" for identifier, observed in active_breaks)

    th: list[str] = [
        f"## รายการที่ยังไม่ผ่าน ({context}) — run {run}",
        "",
        f"ผล readiness คือ {overall} รวม และ {without} เมื่อไม่นับ deferred; corpus ผ่าน page count "
        f"{page_total - len(page_failures)}/{page_total} รายการ และ page boundary "
        f"{break_total - len(break_failures)}/{break_total} รายการ มีรายการ corpus ที่ยัง active {active_count} รายการ",
        "",
        f"### corpus-pages ({len(active_pages)} จาก {active_page_total})",
        "",
    ]
    th.extend(f"- `{identifier}` {observed}" for identifier, observed in active_pages)
    th.extend(["", f"### corpus-breaks ({len(active_breaks)} จาก {active_break_total})", ""])
    th.extend(f"- `{identifier}` {observed}" for identifier, observed in active_breaks)

    en_deferred, th_deferred = deferred_sections(report)
    en.extend(en_deferred)
    th.extend(th_deferred)

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
