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
commit = capture(r"at commit `([^`]+)`")
date = capture(r"readiness` on ([0-9-]+)")

dimensions = {
    "Features": "ฟีเจอร์", "File formats": "รูปแบบไฟล์",
    "Layout and pagination": "การจัดหน้าและแบ่งหน้า", "UI/UX": "UI/UX",
    "Stability": "ความเสถียร", "Performance": "ประสิทธิภาพ",
    "Localization": "ภาษา", "Platforms": "แพลตฟอร์ม",
    "Ecosystem": "ระบบนิเวศ", "Hardware": "ฮาร์ดแวร์",
}
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
    "มีรายการ layout ที่ไม่ผ่านจำนวนหน้า 32 รายการ และไม่ผ่านขอบเขตหน้า 112 รายการ รายการครบทุกข้อพร้อมผลที่วัดได้ การตรวจ font substitution และเหตุผลที่ยืนยันได้หรือยังแยกสาเหตุไม่ได้อยู่ใน [อุปสรรค readiness ฉบับภาษาไทย](readiness/blockers-2026-11-14.th.md) และ [ฉบับภาษาอังกฤษ](readiness/blockers-2026-11-14.md)", "",
    "การยังแยกสาเหตุไม่ได้ไม่ทำให้รายการผ่าน ต้องตรงกับ Word ตามเกณฑ์เดิมก่อนจึงนับผ่าน ไม่มีการลดเกณฑ์หรือเพิ่มข้อยกเว้น", "",
    "## รายการ deferred", "",
    "มี 22 รายการ deferred ซึ่งยังนับเป็นไม่ผ่านในคะแนนรวมและไม่รวมในตัวเลข ‘ไม่นับ deferred’ ตามวิธีคำนวณเดิม รายการและ prerequisite ที่บันทึกไว้ยังคงเดิม ดู [รายงาน readiness ฉบับเต็ม](readiness.md) และ [แผน deferred](readiness/deferred-plan.md)", "",
    "## วิธีอ่านผล", "",
    "รายงานนี้วัด source ที่ระบุด้านบนจาก Windows runner; เป้าหมาย readiness 100% ยังไม่ถึง เพราะ layout comparison ที่ยังไม่ตรงจะคงสถานะไม่ผ่านจนกว่าจะมีผลวัดที่ตรงเกณฑ์ คำแปลมิติต่าง ๆ แสดงคู่กับชื่อชุดตรวจและจำนวนจริงจากรายงานต้นทาง", "",
]
out.write_text("\n".join(doc), encoding="utf-8")
