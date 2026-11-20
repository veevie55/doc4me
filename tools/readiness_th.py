#!/usr/bin/env python3
"""Render a Thai companion to the generated Windows readiness report."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
source = (ROOT / "docs/readiness.md").read_text(encoding="utf-8")
out = ROOT / "docs/readiness.th.md"

def capture(pattern: str, default: str = "ไม่พบข้อมูล") -> str:
    match = re.search(pattern, source)
    return match.group(1) if match else default

measured = capture(r"\*\*Measured readiness: ([^*]+)\*\*")
ahead = capture(r"\*\*Without the deferred rows: ([^*]+)\*\*")
deferred = capture(r"Without the deferred rows: [0-9.]+%\*\* — (\d+) rows", "?")
commit = capture(r"at commit `([^`]+)`")
date = capture(r"readiness` on ([0-9-]+)")

dimensions = {
    "Features": "ฟีเจอร์", "File formats": "รูปแบบไฟล์",
    "Layout and pagination": "การจัดหน้าและแบ่งหน้า", "UI/UX": "UI/UX",
    "Stability": "ความเสถียร", "Performance": "ประสิทธิภาพ",
    "Localization": "ภาษา", "Platforms": "แพลตฟอร์ม",
    "Ecosystem": "ระบบนิเวศ", "Hardware": "ฮาร์ดแวร์",
}
page_match = re.search(r"^### corpus-pages \((\d+) of \d+\)$", source, re.MULTILINE)
break_match = re.search(r"^### corpus-breaks \((\d+) of \d+\)$", source, re.MULTILINE)
page_failures = page_match.group(1) if page_match else "0"
break_failures = break_match.group(1) if break_match else "0"
rows = []
for line in source.splitlines():
    m = re.match(r"\| ([^|]+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \|$", line)
    if m and m.group(1).strip() in dimensions:
        name, weight, score, without, checks = (x.strip() for x in m.groups())
        rows.append(f"| {dimensions[name]} | {weight} | {score} | {without} | {checks} |")

doc = [
    "# ผล readiness ที่วัดได้ (ภาษาไทย)", "",
    f"> สร้างจาก [`readiness.md`](readiness.md) เมื่อ {date} ที่ source commit `{commit}`; ตัวเลขและเกณฑ์มาจากรายงาน Windows ทางการ ไม่แก้เกณฑ์ น้ำหนัก corpus สถานะ deferred หรือ excluded", "",
    f"**Readiness รวม: {measured}**  ",
    f"**เมื่อไม่นับรายการ deferred: {ahead}**", "",
    "| มิติ | น้ำหนัก | คะแนน | เมื่อไม่นับ deferred | ชุดตรวจ (ผ่าน / ทั้งหมด) |",
    "|---|---:|---:|---:|---|", *rows, "",
    "## รายการที่ยังไม่ผ่านและเหตุผลรายข้อ", "",
    f"มีรายการ corpus-pages ที่ไม่ผ่าน {page_failures} รายการ และ corpus-breaks ที่ไม่ผ่าน {break_failures} รายการ ผลล่าสุดพร้อม observations อยู่ใน [อุปสรรค readiness ฉบับภาษาไทย](readiness/blockers-2026-11-14.th.md) และ [ฉบับภาษาอังกฤษ](readiness/blockers-2026-11-14.md); workflow จะปรับ blockers จากผลวัดทุกครั้ง", "",
    "การยังแยกสาเหตุไม่ได้ไม่ทำให้รายการผ่าน ต้องตรงกับ Word ตามเกณฑ์เดิมก่อนจึงนับผ่าน ไม่มีการลดเกณฑ์ ข้อยกเว้นมีเฉพาะที่เจ้าของสั่ง defer ไว้พร้อมหลักฐาน", "",
    "## รายการ deferred", "",
    f"มี {deferred} รายการ deferred ซึ่งยังนับเป็นไม่ผ่านในคะแนนรวมและไม่รวมในตัวเลข ‘ไม่นับ deferred’ ได้แก่รายการที่รอ prerequisite (ฮาร์ดแวร์ แพลตฟอร์ม บริการภายนอก) และหน้าของ corpus ที่เจ้าของให้ defer การเทียบการแบ่งหน้า ดู [รายงาน readiness ฉบับเต็ม](readiness.md), [แผน deferred](readiness/deferred-plan.md) และ [หลักฐานของ corpus ที่ defer](readiness/deferred-pagination-2026-11-20.md)", "",
    "## วิธีอ่านผล", "",
    "รายงานนี้วัด source ที่ระบุด้านบนจาก Windows runner; เป้าหมาย readiness 100% ยังไม่ถึง เพราะ layout comparison ที่ยังไม่ตรงจะคงสถานะไม่ผ่านจนกว่าจะมีผลวัดที่ตรงเกณฑ์ คำแปลมิติต่าง ๆ แสดงคู่กับชื่อชุดตรวจและจำนวนจริงจากรายงานต้นทาง", "",
]
out.write_text("\n".join(doc), encoding="utf-8")
