# Manual-break line height · 2026-11-18

## What changed

A manual line break after text no longer contributes its font metrics to that text line. Empty lines still use their break's formatting; the following text retains its own size. The same rule already applies to tabs, page breaks and column breaks after text. No document text, font substitution, PDF metadata, or readiness criterion changes.

In govuk-024 page 1, an 8 pt line is followed by a break in a 24 pt run. The former renderer let that break raise the first line. Baselines measured from the PDF text drawing positions (points from page top):

| Text | Before | After | Word reference |
|---|---:|---:|---:|
| Insolvency (England and Wales) Rules 2016 | 43.852 | 28.504 | 28.380 |
| Rule 15.18 | 73.148 | 53.051 | 56.580 |

The small line now sits close to Word's. The heading still differs; this patch does not claim complete font or line-spacing parity. The separate page-2 Annex/Rule extraction-order failure remains.

## Regression coverage

- `a_manual_break_uses_its_font_height_only_on_a_blank_line`: body and table-cell paragraphs, compatibility modes 14/15, larger next text, blank initial line and consecutive breaks. Removing the production change makes the test fail (8 pt text rises from 9.199 to 27.598 pt line height); restoring it passes.
- `shift_enter_in_a_larger_font_keeps_the_previous_line_small`: commands `format.font`, `format.size`, character-by-character `text.insert`, and `text.lineBreak`; the first line stays small, the next stays large, both stay in one paragraph.

## Corpus comparison

Baseline: main `1d48e99`, using the frozen prepared-font CLI from `aba659a`. Candidate uses the same compiler/profile (debug opt-level 0, correctness comparisons only) and pinned prepared font set. All 438 DOCX/PDF input hashes were checked against `corpus.tsv`. Opt-level 1 is used for the full CI and its performance gates.

The initial 219-document comparison finds no changed page counts and no newly passing or failing document under the existing endpoint criterion. Extracted text changes only in zenodo-044 and zenodo-109; word-box geometry changes on 12 pages across seven documents: govuk-024 (1), govuk-072 (6/7/10/12), zenodo-044 (9/10), zenodo-092 (1), zenodo-096 (5), zenodo-109 (3), zenodo-147 (6/11). A coordinate change need not change plain extracted text. Per-document counts, endpoint results and changed word-box pages: [comparison TSV](manual-break-comparison-2026-11-18.tsv). The completed full verifier confirms open/roundtrip/stability **219/219**, counts **216/219**, boundaries **128/219**, no missing inputs or extractor/work-folder errors. [All five check sets, per document](manual-break-verifier-2026-11-18.tsv). A separate harness using the exact Rust tokenizer compares both endpoints on **2,941** corresponding pages: **one endpoint improves (zenodo-044 page 10 first word), zero endpoints regress**. zenodo-044 still fails elsewhere, so no document score increases. The three Linux count failures remain govuk-068 (16/15), zenodo-019 (10/11), zenodo-114 (72/71).

The official Windows result remains run 44078179776 (source `019b0ff`): 219/219 counts, 127/219 boundaries, **92 boundary failures**. Local Linux measurements do not replace it. The five equal-word-multiset cases still fail the original first/last-word criterion. Accepted/excluded/deferred decisions are unchanged.

## Workspace validation

`cargo xtask ci` passes all **6/6** gates: **2,615 native passed, zero failed, one existing ignored**, assets **141**, layers **20 crates**, WASM **16 crates**. Both new regressions pass in the full workspace run. Existing thresholds and ignored tests are unchanged. The interrupted initial run was discarded; the completed log is `/workspace/work/ci-linebreak-resumed.log`. Candidate CLI SHA-256: `57159d9e1e0f66a4551eb07d32a41644ea91f3fabc0e62c3a7ee56ec8e70f6b2`.

## Rejected experiments

A read-only PDF metadata probe replaced krilla's OS/2 typo ascender/descender descriptors with the rendered face's selected hhea/USE_TYPO metrics. Across 219 documents this gained govuk-024, zenodo-030 and zenodo-070 but lost govuk-072 page 18. Font-size quantization did not remove that regression. No such production patch was retained. Word PDFs also vary between embedded and unembedded Arial, whose Poppler bounds differ; a reference font descriptor alone is insufficient evidence for line height.

## ภาษาไทย

แก้ตัวขึ้นบรรทัดใหม่ที่มีฟอนต์ใหญ่กว่าข้อความก่อนหน้าไม่ให้ดันบรรทัดนั้นสูงเกินจริง บรรทัดว่างและข้อความถัดไปยังรักษาขนาดของตนเอง เทียบ corpus เดิมครบ 219 ฉบับโดยไม่เปลี่ยนเกณฑ์หรือรายการยกเว้น งานขอบเขตหน้ายังไม่เสร็จทั้งหมด และไม่อ้างคะแนนใหม่จากการแก้จุดนี้
