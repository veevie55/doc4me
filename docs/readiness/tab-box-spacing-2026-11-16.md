# Tabs and paragraph spacing inside boxes · 2026-11-16

## Evidence

`zenodo-091` puts its contents on page 1 under a cover table. Word fits the whole contents there; Doc4Me pushed its last line to page 2 and ran to 12 pages instead of 11. Line positions from Word's PDF (`pdftotext -bbox-layout`) show two separate causes:

- **Tabs in a bigger font.** Each contents line is 11 pt Arial text with a tab run formatted 12 pt Calibri (the theme minor font). Word steps these lines **12.6 pt**, Arial 11 pt's line. Doc4Me stepped them 14.65 pt, Calibri 12 pt's line, because a tab's font counted towards the line's height.
- **Space between paragraphs in table cells.** Two 11 pt Arial paragraphs in a cell, each with 3 pt before and after, are **15.72 pt** apart in Word: one line plus 3 pt. Doc4Me put them 18.65 pt apart, adding both sides. The body already uses the larger of the space after and the space before ([changelog entry for Word's paragraph spacing](../../CHANGELOG.md), `place_para`), but `frame_box`, which lays out table cells, text boxes and frames, still added them.

## Change

- `crates/layout/src/para.rs`: a tab doesn't raise a line that holds text (a text cluster or an equation). A line of only tabs, spaces and objects keeps their heights, as does the existing rule that a last line of only tabs and spaces is as tall as a bigger paragraph mark.
- `crates/layout/src/lib.rs` `frame_box`: paragraphs after the first are separated by the larger of the space after and the space before, unless the document sets Don't use HTML paragraph auto spacing (`w:doNotUseHTMLParagraphAutoSpacing`), as in the body. Contextual spacing works as before. A later paragraph's negative space before is still clamped at zero inside a box, as it was.
- No fonts, corpus files, references, criteria, exclusions or deferred rows change.

## Validation

- New regressions: `a_bigger_tab_does_not_raise_a_line_of_text` (a 24 pt tab between 11 pt Arial words leaves the line Arial's height; a line of only 24 pt tabs stays taller) and `table_cells_put_the_larger_of_space_after_and_space_before_between_paragraphs` (3/3 → 3 pt, 8/3 → 8 pt, 3/12 → 12 pt; with the additive setting, 3/3 → 6 pt and 8/3 → 11 pt). Both fail on the previous source and pass with the change. `doc4me-layout` passes 328/328.
- Full `cargo xtask ci` passes **6/6** in one run: native **2,560 pass / one existing ignored**, assets 141, layers 20 crates, wasm 16 crates.
- [220-document Linux comparison](tab-box-spacing-comparison-2026-11-16.tsv), prepared-font release CLIs, baseline `6d4976c` (Cambria Math line change) against this change; all 440 DOCX/PDF hashes match `corpus.tsv`.
  - Page counts **192 → 194/220**: **govuk-068** 16 → 15 and **zenodo-091** 12 → 11 now match Word.
  - Page boundaries stay **102/220**. **govuk-046** still fails but matches further (first difference p8 → p11).
  - **zenodo-069** moves 34 → 33 pages (Word 24). **zenodo-022 moves 27 → 28 pages (Word 20)**, further from Word: a table row on page 8 now splits differently. That document already differs from page 2, where Doc4Me's page holds 41 words against Word's 286; the cause there is not established. No previously passing count or boundary regresses.
  - Each change was also measured alone. A first version of the tab rule, which also counted line breaks as text, changed only zenodo-091; the final rule counts only text and equations. The box spacing rule alone changed only govuk-046, govuk-068, zenodo-022 and zenodo-069. The combined comparison above uses the final code.
- zenodo-091 page 1 now tracks Word within 0.3 pt (contents heading 488.89 pt against Word's 488.70). Its first boundary difference is now page 4.
- These are Linux comparisons, not a readiness score. A Windows readiness run decides the official result.

## ภาษาไทย

zenodo-091 มีสารบัญอยู่ใต้ตารางหน้าปก Word วางสารบัญได้ครบในหน้า 1 แต่ Doc4Me ดันบรรทัดสุดท้ายไปหน้า 2 จนได้ 12 หน้าแทน 11 หน้า เทียบตำแหน่งบรรทัดกับ PDF ของ Word แล้วพบสาเหตุสองข้อ ข้อแรก tab ในบรรทัดสารบัญใช้ Calibri 12 pt ขณะที่ข้อความเป็น Arial 11 pt Word เว้นบรรทัดตาม Arial (12.6 pt) แต่ Doc4Me นับความสูงของ tab ด้วย (14.65 pt) ข้อสอง ระยะห่างระหว่างย่อหน้าในเซลล์ตาราง Word ใช้ค่าที่มากกว่าระหว่าง space after กับ space before เหมือนใน body แต่ Doc4Me ยังบวกกันเฉพาะในเซลล์ ตารางและกล่องข้อความ

แก้โดยไม่ให้ tab ทำให้บรรทัดที่มีข้อความสูงขึ้น (บรรทัดที่มีแต่ tab ยังคงความสูงเดิม) และให้เซลล์ ตาราง กล่องข้อความ และ frame ใช้ค่าที่มากกว่าเหมือน body เว้นแต่เอกสารเปิด "Don't use HTML paragraph auto spacing"

ผลเทียบบน Linux ครบ 220 ฉบับ (hash ครบ 440 ไฟล์): จำนวนหน้าตรง **192 → 194** (govuk-068 และ zenodo-091 ตรง Word แล้ว) ขอบเขตหน้าคงที่ **102** govuk-046 ตรงไกลขึ้น zenodo-069 ใกล้ Word ขึ้น แต่ **zenodo-022 ห่างจาก Word เพิ่มอีก 1 หน้า (28 เทียบกับ 20)** เพราะแถวตารางหน้า 8 ตัดต่างไป เอกสารนี้คลาดจาก Word มากอยู่แล้วตั้งแต่หน้า 2 และยังไม่ทราบสาเหตุ ไม่มีรายการที่เคยผ่านกลับมาตก เทสต์ใหม่ทั้งสองล้มบน source เดิม ต้องรอผลวัด Windows ก่อนเปลี่ยนคะแนนทางการ
