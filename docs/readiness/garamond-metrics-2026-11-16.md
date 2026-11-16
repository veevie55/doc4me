# Garamond widths and line · 2026-11-16

## Evidence

Garamond is a commercial font, and Doc4Me draws it with Tinos (Times New Roman on Windows). In **zenodo-115** Doc4Me fit fewer words on every page than Word (350 against 396 on page 1, 457 against 510 on page 2) and ran to 10 pages instead of 9.

- **Line.** The body is Garamond 11 pt at 283/240 line spacing, justified. Word steps those lines 14.64 or 14.52 pt apart, **14.59 pt** on average: 1.125 em of line. That equals the 862 + 263 per 1000 ascent and descent Word declares in the embedded font descriptor. Tinos' hhea line, which a substitute uses, is 1.150 em.
- **Widths.** Garamond is narrower than Times New Roman. Word's PDFs of zenodo-115 and zenodo-118 declare its advance widths in each simple font's `/Widths` array (WinAnsi codes, 1000 units per em). The two agree on every shared character.

Not every document asking for Garamond was laid out with it: Word set **zenodo-114**'s Garamond in Times New Roman (its PDF embeds no Garamond), so the author evidently lacked the font. A document doesn't say which fonts its author had, so this change follows the documents where Word did have Garamond.

## Change

- `tools/garamond_pdf_metrics.py` reads those widths from the two manifest-pinned PDFs (hash checked; never a font program or outline) and prints the `GARAMOND_REGULAR`, `GARAMOND_ITALIC` and `GARAMOND_BOLD` tables now in `crates/fonts/src/latin_metrics.rs`.
- `metric_fit` applies them to Garamond regular, italic and bold when Tinos or Times New Roman stands in for it, as it already does for Arial Narrow, Gotham and Nyala. Bold italic has no measured widths and is unchanged. Characters without a measured width keep the stand-in's advance.
- `requested_line` gives Garamond a 1.125 em line (0.862 + 0.263) when another face stands in for it.

## Validation

- Font unit test: Garamond's line is 1.125 em; with Tinos, the regular, italic and bold fits exist, the space is 0.25 em and unmeasured characters return none; bold italic has no fit. `doc4me-fonts` 83/83; full `cargo xtask ci` 6/6 (native 2,564 pass / one existing ignored).
- [220-document Linux comparison](garamond-metrics-comparison-2026-11-16.tsv), prepared-font release CLIs, baseline `3668e3c` against this change; all 440 hashes match `corpus.tsv`.
  - Page counts **197 → 198/220**: **zenodo-115** 10 → 9 matches Word. Its first page boundary still differs, so boundaries stay **104/220**.
  - **zenodo-118** (also Garamond) still matches, and **zenodo-114** (Garamond requested, Times New Roman drawn by Word) is unchanged. No other row changes.
- On the Windows readiness runner Garamond is expected to resolve to Times New Roman (the first serif stand-in there), which the fit also covers; the next Windows run decides.

## ภาษาไทย

Garamond เป็นฟอนต์เชิงพาณิชย์ Doc4Me จึงใช้ Tinos (บน Windows คือ Times New Roman) แทน ใน zenodo-115 Doc4Me ใส่คำได้น้อยกว่า Word ทุกหน้าและได้ 10 หน้าแทน 9 หน้า เพราะ Garamond แคบกว่า Times และ Word เว้นบรรทัด Garamond 1.125 em (ตรงกับ ascent/descent ใน descriptor) ขณะที่ Tinos ใช้ 1.150 em

แก้โดยดึงความกว้างตัวอักษร Garamond (Regular/Italic/Bold) ที่ Word ประกาศใน /Widths ของ PDF สองฉบับที่ตรวจ hash แล้ว (ไม่อ่านโปรแกรมฟอนต์) มาใช้เมื่อ Tinos หรือ Times New Roman วาดแทน และกำหนดระยะบรรทัด 1.125 em ส่วน zenodo-114 ที่ขอ Garamond แต่ Word วาดด้วย Times New Roman (ผู้สร้างไม่มีฟอนต์) ไม่เปลี่ยนผล

ผลเทียบบน Linux ครบ 220 ฉบับ: จำนวนหน้าตรง **197 → 198** (zenodo-115 ตรง Word) ขอบเขตหน้าคงที่ **104** zenodo-118 ยังผ่าน zenodo-114 ไม่เปลี่ยน ไม่มีรายการที่เคยผ่านกลับมาตก
