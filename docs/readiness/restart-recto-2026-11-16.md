# Restarting page numbering at 1 with odd/even headers · 2026-11-16

## Evidence and scope

`govuk-036` enables `w:evenAndOddHeaders`. Its cover is followed by a Next Page section (the omitted `w:type` default) that restarts numbering at 1. The reference has an entirely blank physical page 2 and starts Contents, numbered 1, on physical page 3. Doc4Me previously started Contents on physical page 2 and had 39 pages instead of 40. Of the 220 immutable corpus documents, this is the only one with a subsequent section page-number restart and odd/even headers enabled.

A [Word MVP observation on Microsoft Q&A](https://learn.microsoft.com/en-us/answers/questions/5706899/section-break-in-word-causing-ghost-pagination) distinguishes odd/even headers alone from a numbering restart at 1, which can force an Odd Page start. This is supporting observed behavior, not a normative Open XML rule. This patch is deliberately limited to Next Page, explicit restart at 1, odd/even headers, and a preceding odd displayed number. Other restart values and explicit Odd/Even Page rules are unchanged; no broader equivalence is claimed without evidence.

## Implementation

Before applying the restart, retain whether a parity page is needed. Create the real new page through the existing path, then insert a content-free parity page immediately before it, with an empty drawing layer. It uses the preceding physical page's dimensions and section, and its next displayed number. The real page retains the requested number 1. This preserves note finalization and continuation through the existing page-start path.

A `Page::parity_filler` marker suppresses headers and footers on this generated blank. Assigning it to the preceding section also preserves the following section's first-page header/footer selection. Existing content, fonts, source formats, dependencies, corpus inputs and comparison criteria are unchanged. Insertion shifts only the newly created last page; it does not scan prior pages or add input-controlled recursion. Insert indices are bounded and there is no new panic or unsafe path.

## Validation

- Two new regression tests cover 40 combinations of preceding numbers (including 0 and u32::MAX), odd/even settings and restart values. They check final numbering, generated blank content and section ownership. A header/footer/footnote test checks that the actual page keeps its First Page stories and note caret indices remain on the cover and body pages; Continuous, Next Column and web cases do not gain a filler.
- The initial prepared-font restart filter passes 4/4 (two new regressions and two existing zero-start/note-restart tests). The final full CI also exercises the added footnote assertions.
- [220-document Linux comparison](restart-recto-comparison-2026-11-16.tsv) against accepted source `c694538`: **190→191/220** exact page-count matches. **Only govuk-036 changes, 39→40 pages**, matching reference 40. Other 219 counts are unchanged, with no previously passing page-count row regressing. All 440 DOCX/PDF hashes match the corpus manifest.
- Candidate PDF has 40 pages; physical page 2 has no text or images and was visually inspected as entirely white. Contents starts on physical page 3, numbered 1.
- [Per-page endpoint comparison](restart-recto-boundaries-2026-11-16.tsv), using the readiness normalizer and poppler: **37/40** pages match both first and last word. Pages 8, 19 and 37 still differ. Page 8 ends with `individuals` here and `6` in the reference; pages 19/37 start with `sample`/`come` rather than `the`/`inclusion`. This remains a page-boundary failure; page-count success does not imply full layout parity. No root cause is asserted from these endpoints alone.
- Initial full CI stopped at the unchanged `proof_text_walks_the_runs_once` timing check: ratio 32.0 against strict <32. The function tests proof-text extraction directly, without pagination. An isolated recheck of the same source passed at ratio 28.0; no threshold or test was changed. Final full CI passed **6/6**, native **2,557 pass / one existing ignored**, assets 141, layers 20 crates, wasm 16 crates. The unchanged proof-text check passed in that complete rerun.

**Windows confirmation:** [run 43785505707](https://github.com/veevie55/doc4me/actions/runs/43785505707) measured engine `55d0c37`, report `7e50321`: page counts **191→192/220**, with only govuk-036 newly matching (40 pages); boundaries stay **99/220**, readiness **93.0% overall / 94.9% without 22 deferred**. A row-by-row comparison of the failure lists in reports `b6e1076` and `7e50321` finds no newly failing row. govuk-036 still fails the boundary check at page 8 (Doc4Me ends with `individuals`, Word with `6`), matching the Linux endpoint audit. The workflow regenerated the EN/TH reports, current blockers and TSV together (28 page-count and 121 boundary failures). The previous baseline was run 43776538181, engine `e00daf1`, report `b6e1076` (191/220).

Logs: `/workspace/work/restart-recto-{tests-final,cli,comparison,ci,proof-recheck}.log`; candidate PDF/PNGs and full endpoint JSON remain outside the repository. No new exclusions or deferred rows. Existing v1.1.4 at `c694538` remains the published release; this arc does not republish it.

## ภาษาไทย

แก้เงื่อนไข Next Page ที่เริ่มเลขหน้าใหม่ที่ 1 และเปิดหัวกระดาษคู่/คี่ ซึ่ง Word สามารถเว้นหน้าคู่เปล่าเพื่อให้เริ่มส่วนใหม่ด้านขวาได้ กรณี govuk-036 เดิมระบบไม่มีหน้าว่างนี้ จึงได้ 39 หน้าแทน 40 หน้า แทรกหน้าว่างที่ไม่มีเนื้อหา/หัวกระดาษ/ท้ายกระดาษ โดยคงเลข 1 และหัวกระดาษหน้าแรกให้ section จริง รวมถึงตำแหน่ง footnote ไม่ใช้เพียงการเปิดหัวกระดาษคู่/คี่อย่างเดียวเป็นเหตุเพิ่มหน้า และไม่เปลี่ยนค่าการเริ่มเลขหน้าอื่นที่ยังไม่มีหลักฐาน

ผล Linux ครบ 220 ฉบับผ่านจำนวนหน้าเพิ่มจาก **190 เป็น 191** เปลี่ยนเฉพาะ **govuk-036 จาก 39 เป็น 40** อีก 219 รายการจำนวนหน้าคงเดิม ตรวจ hash ต้นฉบับ/reference ครบ 440 ไฟล์ตรง manifest ภาพหน้าว่างตรวจแล้ว และสารบัญเริ่มหน้าเลข 1 ถูกต้อง แต่ขอบเขตข้อความยังตรงเพียง **37/40 หน้า** จึงไม่อ้างว่าผ่าน page boundary หรือ layout ทั้งหมด

CI รอบแรกติด benchmark proof-text ที่ ratio 32.0 ขณะที่เกณฑ์ต้องน้อยกว่า 32 ตรวจแยกโค้ดเดิมผ่านที่ ratio 28.0 โดยไม่ลด threshold หรือแก้เทสต์ full CI รอบสุดท้ายผ่าน **6/6** (native **2,557** ผ่าน / ignored 1 เดิม) แล้ว ไม่เพิ่ม excluded/deferred และไม่เปลี่ยนเอกสารหรือเกณฑ์ Release v1.1.4 ที่เผยแพร่แล้วคงเดิม

ผล Windows run 43785505707 (source `55d0c37`, report `7e50321`) ยืนยันจำนวนหน้าตรง **192/220** เพิ่มจาก 191 โดยมีเพียง govuk-036 ที่ผ่านเพิ่ม (40 หน้า) เทียบรายการไม่ผ่านกับรายงาน `b6e1076` แล้วไม่มีรายการใดกลับมาตก ขอบเขตหน้าคงที่ **99/220** คะแนน **93.0% รวม / 94.9% ไม่นับ deferred** govuk-036 ยังไม่ผ่าน page boundary ที่หน้า 8 ตรงกับผลตรวจบน Linux
