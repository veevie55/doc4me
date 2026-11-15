# Page-count follow-up · 2026-11-15

Scope: the 32 failing `corpus-pages` rows in Windows run [43701689470](https://github.com/veevie55/doc4me/actions/runs/43701689470), source `fd465e8`. This audit is not a new readiness measurement. The official score remains 94.8% excluding 22 deferred rows, with 188/220 page-count passes.

## Original fonts and redistribution

Copyright and permission are different: OFL fonts are copyrighted but permit use, modification and redistribution. For this task, an openly licensed original is used directly; a commercial original is not downloaded or bundled without redistribution permission.

- **Roboto**: the original is available under [OFL-1.1](https://github.com/google/fonts/blob/cc39c4c2c304bc368c9b7c311ea2bcc6013299fe/ofl/roboto/OFL.txt). The prior resolver forced it to Arimo even when Roboto was present. It now resolves to Roboto, and the corpus font preparation includes the original regular and italic variable files from the pinned Google Fonts revision, with SHA-256 checks and the OFL notice. `zenodo-136` requests Roboto and its Word PDF contains Roboto; this fixes that font decision, but does not by itself prove a page-count pass.
- **Overpass** and **Source Sans Pro / Source Sans 3**: openly licensed originals are already used (Source Sans 3 is the renamed family). Overpass and its named weights are prepared from the pinned upstream OFL files. Their presence does not make `zenodo-022`, `zenodo-067` or `zenodo-069` pass automatically.
- **Arial, Arial Narrow, Arial Nova Cond Light, Tahoma, Calibri, Cambria, Aptos, Garamond, Gotham, Apercu, Arno Pro, Myriad Pro, Frutiger, Amasis MT Pro and the Windows script families**: no open redistribution permission has been established for these originals. Existing open substitutes remain necessary. In particular Arimo is not a verified metric-equivalent replacement for Arial Narrow/Arial Nova Condensed/Tahoma/Gotham; Tinos is not a verified equivalent for Garamond. Their widths, kerning, vertical metrics and style weights require separate validation before claiming equivalence. No blanket scaling was introduced from a target document's page count.

The [32-row inventory](page-count-audit-2026-11-15.tsv) records the exact DOCX/PDF hashes, declared run fonts and font names observed in the PDF. A font declaration can be unused or overridden by a theme/style; it is evidence to investigate, not proof of an active substitution or its cause.

## Reference documents

All 32 source/reference pairs were checked for normalized word overlap using body `w:t` text and Poppler PDF extraction; the inventory records the source-word overlap. This is a screening measure, not a substitute for visual or layout comparison: repeated words, omitted headers and imperfect PDF extraction can change it.

`govuk-026` was checked against the [live publisher page](https://www.gov.uk/government/publications/monthly-bulk-notification-form). It still links exactly the DOCX and PDF URLs in the manifest. All normalized DOCX body words occur in the PDF (overlap 1.000), including the Schedule 3 form sections. The 2025 DOCX timestamp versus the 2012 PDF timestamp and the 4-versus-7 page count therefore do not establish that it is the wrong document. There is no verified replacement reference and no evidence warranting exclusion for this pair.

No pair was excluded or replaced in this audit. A verified mismatch still requires searching the publisher/record for a matching source or PDF; only a demonstrated mismatch without an available matching document justifies exclusion under the user's instruction. The low overlap of multilingual `govuk-068` also requires checking extraction and scripts before treating it as a source mismatch.

## ภาษาไทย

ตรวจเฉพาะ 32 รายการจำนวนหน้าที่ไม่ผ่านจาก Windows run ข้างต้น ฟอนต์ OFL มีลิขสิทธิ์แต่อนุญาตใช้งานและแจกจ่ายได้ จึงใช้ต้นฉบับได้โดยตรง แก้ Roboto ที่เคยถูกบังคับใช้ Arimo และเพิ่มต้นฉบับ Roboto พร้อมใบอนุญาตและตรวจ SHA-256 ในชุด build แล้ว ยังไม่ถือว่ารายการจำนวนหน้าผ่านจนกว่าจะวัดผลจริง

ฟอนต์เชิงพาณิชย์ยังต้องปรับและตรวจฟอนต์แทนด้านความกว้างตัวอักษร kerning ระยะบรรทัดและน้ำหนัก การมีฟอนต์แทนไม่ได้ยืนยันว่าใช้งานเท่าต้นฉบับแล้ว และไม่ปรับ scale ตามจำนวนหน้าที่ต้องการโดยไม่มีหลักฐาน

ตรวจคำในเนื้อหาและ hash ของ DOCX/PDF ครบ 32 คู่แล้ว สำหรับ govuk-026 หน้าเผยแพร่ปัจจุบันยังเชื่อมไปไฟล์คู่เดิม และคำในเนื้อหา DOCX พบใน PDF ครบ วันที่และจำนวนหน้าต่างกันจึงยังไม่พิสูจน์ว่าเอกสารผิดคู่ รอบนี้ไม่มีการ excluded หรือเปลี่ยน reference; ตารางแนบเก็บหลักฐานสำหรับตรวจต่อ ไม่ใช่ผล readiness รอบใหม่

## Validation

The font crate tests pass 76/76 with the prepared pinned font checkout, including the original-Roboto resolution regression. All 32 source/reference hashes match the corpus manifest. The preparation helper verifies all declared font hashes and OFL notices. The release CLI built successfully and probed all 32 documents; the TSV includes Linux page counts and actual reported substitutions. Roboto is absent from the substitution list for `zenodo-136`, confirming direct use, but that document still produces 26 pages versus the 25-page reference. `zenodo-114` produces 71 pages on Linux, whereas the current Windows report says 70; this platform-specific probe does not supersede Windows readiness. Full CI passed fmt and then stopped in the `alsa-sys` build because this environment lacks ALSA development metadata (`alsa.pc`); the remaining full CI stages were skipped, not passed.

ผล CLI ใหม่บน Linux: zenodo-136 ใช้ Roboto โดยตรงแล้วแต่ยังได้ 26 หน้า (reference 25) จึงยังไม่ผ่าน ส่วน zenodo-114 ได้ 71 หน้าใน Linux ต่างจาก Windows เดิม 70 หน้า ข้อมูลนี้เป็น probe รายเอกสาร ไม่ใช่คะแนน readiness ทางการรอบใหม่ Full CI หยุดเพราะขาด ALSA development package ไม่ได้ผ่านครบทุก gate

Focused font clippy (`-D warnings`), formatting, asset attribution (141 files) and crate dependency layers (20 crates) also pass.
