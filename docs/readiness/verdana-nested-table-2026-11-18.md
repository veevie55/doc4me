# Verdana forms, nested tables and PDF word text · 2026-11-18

Status: Linux validated; Windows confirmed by run 44224006895 (source `cedff9f`, report `ce70748`). Criteria and accepted/excluded/deferred decisions unchanged.

Windows confirmation: page counts **219/219**, strict page boundaries **129/219** (previously 127/219), **90 boundary failures remain**. Readiness is **95.0%**, or **96.9% without the 22 deferred rows**. The generated report is committed separately in `ce70748`; the local Linux result below remains a separate measurement.

## Evidence and changes

The baseline is `df1ec6c` (manual-break fix), with the same pinned prepared fonts and compiler profile as the candidate. Windows run **44212279192** measured that source: counts **219/219**, boundaries **127/219**, readiness **95.0% / 96.8% without deferred**. Report commit `2541e0c` updates generated reports only. The local Linux baseline is counts **216/219**, boundaries **128/219**, open/roundtrip/stability **219/219**. Linux results do not replace Windows results.

### The final mark after a nested table

In govuk-017 page 1, the labels within table A already sit close to Word's, but its outer bottom border is at **523.17 pt** instead of Word's **509.64 pt**. The last empty paragraph after the nested table adds a blank line to the cell. In the observed modern layout, Word's outer border ends at the last nested row. Omitting that final empty mark from the cell's height brings “Please turn over” back onto page 1. The mark is retained for caret navigation using the existing `hideMark` layout path. Visible text, extra blank paragraphs and legacy compatibility modes retain their previous height rules.

The isolated table probe preserves all 219 page counts and all document pass/fail statuses. Exact Rust-token comparison of **2,941** corresponding pages finds **10 endpoint improvements, zero losses** (govuk-017/018 and zenodo-053). govuk-017 then matches every endpoint except page 9's first word.

### Verdana advance widths with Inter outlines

The remaining govuk-017 failure is line wrapping: Inter's narrower advances pull “Alternatively…” onto page 8, while Word starts page 9 with it. The document requests 11 pt Verdana in this passage. `tools/verdana_pdf_metrics.py` reads only public `/FirstChar`, `/Widths` and WinAnsi encoding facts from the two reference PDFs, checks their SHA-256 against the existing manifest and rejects conflicting measurements. No font program or outline is read or copied.

| Source | PDF SHA-256 |
|---|---|
| govuk-017 | `eff41e0f10ca1c5dd4cbf629dd02ef70028df95525e5f4843db0856fb815f2b5` |
| govuk-018 | `e84a71043ddb82aed9d79ff1246b12662e712c114030a859f1e83744b672b7d0` |

Measured regular/italic/bold/bold-italic advances apply only when Inter substitutes for Verdana. Unmeasured characters retain fallback advances; other requested fonts and other drawing faces retain their own rules. Verdana's existing line metrics stay unchanged. The regression passage wraps at “year”, as in Word, at 11 pt and 541.8 pt text width.

### Logical words in positioned PDF text

Larger advances with narrower open outlines can make PDF readers infer spaces inside a word (“Manage m en t”). The layout display list now identifies runs whose font metrics were fitted and have no intentional character tracking, carrying that intent through drawing transforms to PDF. Export preserves only forward-ordered, known ASCII words in those runs when an internal glyph advance exceeds the native advance (a 0.001 em tolerance excludes numerical noise). Ordinary words, intentional tracking, complex/reversed clusters and inter-word spaces retain their original path. Tabs remain in the logical display text without adding a visible glyph.

Replacement text is emitted as a non-structural `/ActualText` span **inside BT/ET**, while the original per-glyph Unicode maps, glyph IDs and positions stay unchanged. Both tagged and untagged PDF tests also read every character through Hayro's glyph interpreter, which does not use replacement spans. This guards readers that use native font mappings. Words are capped at 128 bytes and parsed-word allocation at the glyph count.

The existing krilla 0.8.2 dependency lacks this scoped API. `vendor/krilla` is the published package pinned to upstream `3ffdf0588cf98050aad6edba51ca70162e1fb5b5`, with a small `draw_glyphs_with_actual_text` addition in `src/surface.rs`/`src/content.rs`. It stores replacement text only for the draw, writes it inside the text object, and restores the prior state. Native Unicode mapping is untouched. MIT/Apache licenses, upstream notices (including the inherited MPL-2.0 method), MPL text and the four CC0 color-profile attributions are retained. [Patch record](../../vendor/krilla/DOC4ME_PATCH.md). No font program is vendored.

Rejected probes explain the scope: whole-run spans regressed zenodo-074/091; fake many-to-many glyph ranges broke native glyph-map reading; spans outside BT/ET gave Poppler incorrect, zero-height geometry; applying word spans to all text regressed three previously passing documents. Expanded-only spans repaired those three, but also changed intentionally tracked footer extraction in zenodo-060. The final source-aware flag restricts the correction to metric fitting without intentional tracking.

The final 219-document candidate gains **govuk-017 (all nine pages)** and **zenodo-027**, with **no previously passing document failing** and **no changed page counts**. [All 219 document comparisons](verdana-nested-table-comparison-2026-11-18.tsv). The exact Rust-token comparison checks **2,941** corresponding pages: **23 endpoint improvements and 3 losses**, with strict document results **128→130/219**. [Every changed endpoint](verdana-nested-table-endpoints-2026-11-18.tsv). The whole-PDF/form-feed reader was cross-checked against the earlier page-by-page harness and produced identical endpoint rows; the production verifier itself is unchanged. The original production verifier completed from a fresh work directory: **open/roundtrip/stability 219/219**, **page counts 216/219**, **boundaries 130/219**. There are no missing inputs or extractor/work-folder errors. [Every document in all five sets](verdana-nested-table-verifier-2026-11-18.tsv). Candidate prepared CLI SHA-256: `68fb15a87ae87bb79add51df6d9e784c11de065c7af0a5cc1d49ce2b5bc49198`.

The three losses are first words in already-failing zenodo-096 page 8 and zenodo-147 pages 7/11. Their page word-multiset distances change 0→9 and 0→3/3 respectively. Across all pages, the sum of those distances improves **534→36** in zenodo-096 and **463→63** in zenodo-147. Exact page multisets change **6→10** and **22→18**, so the smaller early-page spills are explicitly retained as limitations. These diagnostics are **not pass criteria**; both documents still fail. [Page-by-page diagnostic](verdana-flow-audit-2026-11-18.tsv).

## Validation

Focused layout regressions cover the observed Verdana wrap and the final nested-table mark (including caret access, real text, extra blanks and unchanged legacy layout). A font regression covers all four styles, measured spaces, missing characters and the expected drawing face. Both layout regressions fail with their production changes removed and pass when restored. The PDF replacement-text regression also fails when the metadata call is removed. PDF tests cover native per-glyph reading, tagged/untagged output, span placement inside BT/ET, native versus fitted words, intentional tracking, tabs, long words and reversed mappings. `cargo xtask ci` passes **6/6**: **2,621 native passed, zero failed, one existing ignored**, assets **145**, layers **20 crates**, WASM **16 crates**. It uses the project’s opt-level 1, serial tests and disabled incremental compilation; performance limits and ignored tests are unchanged. The opt-level-0 prepared CLI is used only for paired corpus correctness comparisons. All **438** original input hashes were reverified. The final native code review adds no panic/unsafe path; local vendor changes preserve the original glyph maps and restore scoped state. Main advanced to `1581bea` during validation with only three build-workflow files changed relative to the report commit; those owner changes were fast-forwarded intact and do not alter the tested renderer.

## ภาษาไทย

แก้ตารางชั้นนอกที่นับย่อหน้าว่างท้ายตารางย่อยเป็นความสูงเพิ่ม ทั้งที่ Word จบขอบตารางตรงแถวสุดท้าย และใช้ค่าความกว้าง Verdana ที่ประกาศใน PDF อ้างอิงกับเส้นตัวอักษร Inter ที่เป็นโอเพนซอร์ส ไม่คัดลอกโปรแกรมฟอนต์ เพิ่มข้อความเชิงตรรกะรายคำใน PDF เพื่อไม่ให้ตัวอ่านแทรกช่องว่างกลางคำ ผลทดลองทั้ง corpus ทำให้ govuk-017 และ zenodo-027 ผ่านเพิ่ม โดยจำนวนหน้าทุกไฟล์ไม่เปลี่ยนและไม่มีเอกสารที่เคยผ่านกลับมาตก ยังไม่อ้างว่างานขอบเขตหน้าที่เหลือเสร็จทั้งหมด

ผล Windows run 44224006895 ยืนยันแล้วว่า จำนวนหน้าผ่าน 219/219 และขอบเขตหน้าผ่านเพิ่มจาก 127 เป็น 129/219 ยังเหลือ 90 กรณี ความพร้อมรวม 95.0% หรือ 96.9% เมื่อไม่นับ deferred ทั้ง 22 รายการ โดยเกณฑ์ตรวจรับคงเดิม
