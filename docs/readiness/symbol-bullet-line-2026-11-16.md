# Symbol and Wingdings bullets drawn by other faces · 2026-11-16

## Evidence

`zenodo-156` is a two-page table whose left column holds bulleted lists. Its third-level bullets are Wingdings U+F0A7 (`lvlText`, level `rPr` Wingdings, 11 pt from the Calibri paragraph mark). Word's reference PDF steps every one of those lines **13.44 pt**, Calibri 11 pt's line, as if there were no bullet: Wingdings' own line, (1841 + 432) / 2048 em, is shorter than Calibri's.

Where Wingdings is not installed, Doc4Me draws the private-use bullet as the standard character ▪ (U+25AA), which a fallback face draws. With the prepared corpus fonts that face is Source Sans 3, and its line made each bulleted first line **14.59 pt** (ascent 11.0, descent 3.59, from a temporary layout dump). About ten such lines put the left column 10 pt lower than Word's by the second table row. The row (minimum height 272 pt) then no longer fit on page 1, so the document took 3 pages instead of 2.

## Change

`doc4me_fonts::word::covered_fallback_line` ([Cambria Math audit](cambria-line-2026-11-16.md)) now also covers **Symbol, Wingdings and Wingdings 2**, the symbol fonts with a measured Word line in `requested_line`. Word draws their bullets in the symbol font itself, so a fallback face drawing the standard character keeps that font's line. As before, it applies only to letters outside East Asian and complex scripts, and other families keep the fallback face's own line. Wingdings 3 and Webdings have no measured line here and are unchanged.

**Platform note:** The official Windows readiness runner has Symbol and Wingdings installed, and its diagnostics reported no substitution for these documents. There the bullets are drawn by the real fonts, so this change may not alter Windows results. It makes Linux, macOS without Office, and the web build lay these bullets out as Word does.

## Validation

- Font unit test: Wingdings 11 pt → 12.208 pt, Symbol 1.225 em, Wingdings 2 1.054 em; Wingdings 3, Webdings and unrelated families → none.
- Layout regression `a_wingdings_bullet_keeps_wingdings_line`: ▪, U+F0A7 and • in Wingdings before 11 pt Calibri text leave the line Calibri's height. It is decisive where a taller face draws the ▪. With the prepared fonts it fails without the change (14.586 pt against 13.428 pt) and passes with it; with the bundled fonts alone the unit test pins the rule. `doc4me-fonts` 83/83 and `doc4me-layout` 329/329 pass.
- Full `cargo xtask ci` passes **6/6** in one run: native **2,561 pass / one existing ignored**.
- [220-document Linux comparison](symbol-bullet-line-comparison-2026-11-16.tsv), prepared-font release CLIs, baseline `02a6aa9` (tab and box spacing) against this change; all 440 DOCX/PDF hashes match `corpus.tsv`.
  - Page counts **194 → 196/220**: **zenodo-156** 3 → 2 (and every page boundary now matches) and **govuk-043** 26 → 24 match Word.
  - Page boundaries **102 → 103/220** (zenodo-156). **govuk-044** still fails but matches further (first difference p4 → p7).
  - **govuk-038 moves 47 → 45 pages (Word 46)**: it was one page long and is now one page short. It stays failing; the cause of the remaining difference is not established.
  - No previously passing count or boundary regresses.

**Windows confirmation:** [run 43796305956](https://github.com/veevie55/doc4me/actions/runs/43796305956) measured engine `df4f160`, report `3fca7cd`: page counts **195 → 197/220** and boundaries **102 → 103/220**, with zenodo-156 (count and boundaries) and govuk-043 newly matching, as on Linux, despite the platform note above; readiness **93.3% overall / 95.2% without 22 deferred**. Compared row by row with report `12eed2c`, no row is newly failing. Why the Windows runner changes too is not established here.

## ภาษาไทย

zenodo-156 เป็นตารางสองหน้าที่คอลัมน์ซ้ายมีรายการ bullet ระดับที่ 3 ใช้ Wingdings U+F0A7 PDF ของ Word เว้นบรรทัดเหล่านั้น 13.44 pt เท่าบรรทัด Calibri 11 pt เพราะระยะบรรทัดของ Wingdings เล็กกว่า Calibri แต่บนเครื่องที่ไม่มี Wingdings Doc4Me แปลง bullet เป็น ▪ แล้วให้ฟอนต์สำรอง (Source Sans 3) วาด บรรทัดจึงสูง 14.59 pt สะสมราว 10 pt จนแถวตารางล้นไปหน้า 2 และได้ 3 หน้าแทน 2 หน้า

แก้โดยให้ bullet ของ Symbol, Wingdings และ Wingdings 2 ที่ฟอนต์อื่นวาดแทนยังคงระยะบรรทัดของฟอนต์ symbol นั้น ใช้กลไกเดียวกับ Cambria Math เครื่อง Windows ที่ใช้วัดคะแนนทางการมี Symbol/Wingdings ติดตั้งอยู่แล้ว ผล Windows จึงอาจไม่เปลี่ยน แต่ Linux, macOS ที่ไม่มี Office และเว็บจะจัด bullet เหล่านี้เหมือน Word

ผลเทียบบน Linux ครบ 220 ฉบับ (hash ครบ 440 ไฟล์): จำนวนหน้าตรง **194 → 196** (zenodo-156 และ govuk-043 ตรง Word) ขอบเขตหน้า **102 → 103** govuk-044 ตรงไกลขึ้น แต่ **govuk-038 เปลี่ยนจาก 47 เป็น 45 หน้า (Word 46)** จากเดิมเกินหนึ่งหน้า เป็นขาดหนึ่งหน้า ยังไม่ผ่านเหมือนเดิมและยังไม่ทราบสาเหตุที่เหลือ ไม่มีรายการที่เคยผ่านกลับมาตก เทสต์ layout ใหม่ล้มเมื่อถอดการแก้ออกในสภาพที่ใช้ฟอนต์ prepared

ผล Windows run 43796305956 ยืนยัน: จำนวนหน้าตรง **197/220** ขอบเขตหน้า **103/220** (zenodo-156 และ govuk-043 ผ่านเพิ่มเหมือนบน Linux แม้เครื่อง Windows จะมีฟอนต์ symbol ติดตั้งอยู่ ยังไม่ได้ตรวจหาสาเหตุ) คะแนน **93.3% / 95.2%** ไม่มีรายการใหม่ที่ไม่ผ่าน
