# Cambria Math lines over fallback letters · 2026-11-16

## Evidence

`zenodo-082` sets its equations as ordinary paragraphs in a custom `Equation` style: Cambria Math, 10.5 pt, 1.15 lines (document default), 4 pt before and after. Word's reference PDF embeds Cambria Math and steps consecutive equation lines **18.12 pt** (equations 11 → 12 → 13), 14.15 pt of line plus 4 pt of spacing. That is Cambria's single line, 1946 + 455 per 2048 em (1.172 em × 10.5 pt × 1.15), the line Doc4Me already used for Cambria Math drawn by Caladea.

Doc4Me stepped the same lines **26.38 pt**. Caladea, the open stand-in for Cambria and Cambria Math, has no Greek block (fontconfig charset: nothing in U+0370–03FF), so δ, μ, ζ, ε, π and Δ were drawn by fallback faces, and each fallback face brought its own line. The tallest, Sarabun's Word-for-Windows line (1.853 em), set every equation line to 22.37 pt. The document ran onto 6 pages instead of Word's 5, with page 2 ending three equations early.

## Change

- `doc4me_fonts::word::covered_fallback_line` gives the requested font's line for letters a fallback face draws when the requested font itself holds them in Word. It is limited to **Cambria and Cambria Math**. Other families keep the fallback face's own line, because which letters their originals contain is not established here.
- `crates/layout/src/para.rs` applies it only to fallback segments that are neither complex-script nor East Asian text. Word draws East Asian text in the run's East Asian font and complex scripts in the complex-script font, so those segments keep their fallback face's line. The style cache now also compares the line, so a face shared by both kinds of segment cannot hand one line to the other.
- `single_line` keeps Cambria's line for an **installed** Cambria Math too, as it already did for Aptos. Word's PDF above uses the real Cambria Math, so this is the observed spacing, not Cambria Math's own much taller math metrics. Whether a Windows build previously used the taller metrics is not measured here; the next Windows readiness run decides the official result.
- OMML math layout (`crates/layout/src/math.rs`) does not call `single_line` and is unchanged. No fonts, corpus files, references, criteria, exclusions or deferred rows change.

## Validation

- New regression `cambria_math_lines_keep_cambrias_spacing_over_fallback_letters`: Cambria Math and Cambria lines with and without Greek letters are all 12.31 pt at 10.5 pt single spacing; East Asian letters are not held to Cambria's line. With the `para.rs` change reverted, the test fails (Greek line 19.46 pt against 12.31 pt). Font unit tests cover installed Cambria Math, case and face-name handling, and families that are excluded.
- `doc4me-fonts` 83/83 and `doc4me-layout` 326/326 pass.
- Full `cargo xtask ci`: fmt, clippy, test (native **2,558 pass / one existing ignored**), assets 141 and layers 20 crates pass. The wasm step first failed because this machine lacked the `wasm32-unknown-unknown` target. After installing the target, a rerun of `cargo xtask wasm` on the same source passed all 16 crates. No code or threshold changed between the two runs.
- [220-document Linux comparison](cambria-line-comparison-2026-11-16.tsv), prepared-font release CLIs (craft-fonts `c1db6d1`, `tools/prepare_corpus_fonts.py`), baseline `7e50321` against this change. All 440 DOCX/PDF hashes match `corpus.tsv`.
  - Page counts **191 → 192/220**: only **zenodo-082** changes, 6 → 5 pages (Word 5).
  - Page boundaries **99 → 102/220**: **zenodo-032**, **zenodo-073** and **zenodo-082** newly match every page's first and last word. **zenodo-056** (p1 → p2) and **zenodo-057** (p11 → p26) still fail but match further. All five use Cambria Math. No other row changes, and no previously passing count or boundary regresses.
- zenodo-082 line positions now track Word's: heading 3.4 at 401.91 pt (Word 401.93), equation pitch 18.16 pt (Word 18.12), heading 3.5 at 501.38 pt (Word 501.43).
- Linux counts differ from Windows on known rows (for example zenodo-114), so this is not a readiness score. The official Windows baseline remains [run 43785505707](https://github.com/veevie55/doc4me/actions/runs/43785505707): 192/220 page counts, 99/220 boundaries.

## ภาษาไทย

zenodo-082 เขียนสมการเป็นย่อหน้าธรรมดาด้วยฟอนต์ Cambria Math ขนาด 10.5 pt PDF ของ Word เว้นบรรทัดสมการ 18.12 pt ซึ่งตรงกับระยะบรรทัดของ Cambria (1.172 em) แต่ Doc4Me เว้น 26.38 pt สาเหตุคือ Caladea ซึ่งใช้แทน Cambria ไม่มีอักษรกรีก อักษรเหล่านั้นจึงไปใช้ฟอนต์สำรอง แล้วบรรทัดก็สูงตามฟอนต์สำรอง (Sarabun 1.853 em) ทำให้ได้ 6 หน้าแทน 5 หน้า

แก้โดยให้อักษรที่ฟอนต์สำรองวาดแทนในข้อความ Cambria/Cambria Math ใช้ระยะบรรทัดของ Cambria เฉพาะอักษรที่ไม่ใช่ภาษาเอเชียตะวันออกและไม่ใช่ complex script (รวมภาษาไทย) ฟอนต์อื่นยังคงพฤติกรรมเดิม และให้ Cambria Math ที่ติดตั้งในเครื่องใช้ระยะบรรทัดของ Cambria ตามหลักฐานจาก PDF ของ Word ไม่เปลี่ยนการจัดสมการ OMML ฟอนต์ เอกสาร corpus เกณฑ์ หรือรายการ excluded/deferred

ผลเทียบบน Linux ครบ 220 ฉบับ (ตรวจ hash ครบ 440 ไฟล์): จำนวนหน้าตรง **191 → 192** (เปลี่ยนเฉพาะ zenodo-082 จาก 6 เป็น 5 หน้า) ขอบเขตหน้าตรง **99 → 102** (zenodo-032, zenodo-073, zenodo-082 ผ่านเพิ่ม ส่วน zenodo-056/057 ยังไม่ผ่านแต่ตรงไกลขึ้น) ไม่มีรายการที่เคยผ่านกลับมาตก ทุกฉบับที่เปลี่ยนใช้ Cambria Math เทสต์ใหม่ล้มเมื่อถอดการแก้ออก ต้องรอผลวัด Windows รอบใหม่ก่อนเปลี่ยนคะแนนทางการ
