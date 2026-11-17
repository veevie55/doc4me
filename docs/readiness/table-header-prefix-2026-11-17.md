# Initial table headers and the first body row — 2026-11-17

Status: all six CI steps and the complete 219-document pagination comparison pass. Windows confirmation pending. Corpus-breaks is still incomplete.

## Problem and behavior

In govuk-052, Question 2's repeated table header fitted at the bottom of page 2 but the first body row, ANSWER, did not. The initial header was placed before checking whether the body could start under it; the next page repeated the stranded header. Word moves the question header to page 3 with ANSWER. The document matches nine pages before and after the change; the baseline has only one failing boundary, at page 2.

`table_header_start_need` measures a table's initial contiguous header rows and the first body row's start, respecting cantSplit and a finite at-least row minimum. When that prefix does not fit the current page but fits a fresh page, it moves before the header is placed. A keep-with-next caption reserves the same prefix, so the move does not strand its caption. Tables without a body after their header rows, ordinary non-header tables, and combinations too tall for a page retain their existing paths. The existing row first-line measurement is reused; this does not replace the row-splitting algorithm.

The Apache-2.0 WordCraft implementation independently corroborates initial-header preflight ([table_flow.rs at d7215f4](https://github.com/storytold/wordcraft/blob/d7215f4441ee9371442fcd6343c8e85ab74d1348/crates/layout/src/table_flow.rs)); its comment records Word's behavior. The local implementation and caption helper use Doc4Me's current row layout and overflow rules.

## Validation

Baseline renderer: AutoFit tree committed as e4277bf, Linux, pinned prepared craft-fonts c1db6d1 plus the existing font preparation script; 438 source hashes were verified earlier in this session and inputs were not changed. Corpus membership, accepted/excluded/deferred rows and pass criteria are unchanged.

- All 219 document page counts remain identical: 213/219 match, including the existing accepted row.
- Boundaries improve 119→120/219. Only govuk-052 newly passes; every page's first and last word now matches Word. No previously passing document or page becomes failing.
- zenodo-108 page 18 and zenodo-173 pages 14/15/18 have different endpoint words within pages already failing; their page counts and counts of differing pages stay unchanged. In zenodo-173 page 15, its previously matching first word changes; this partial endpoint regression is recorded rather than hidden. The document already has 23 pages versus Word's 25. No claim of complete fidelity is made for either document.
- [All 219 comparison rows](table-header-prefix-comparison-2026-11-17.tsv).
- New tests cover a header stranded above a non-splittable body row, a prefix with enough room, and a keep-with-next caption. Existing oversized repeated-header tests also pass.
- `cargo xtask ci`: all six steps pass. Native tests: 2,600 passed, zero failed, one existing ignored; layout 358/358. Assets 141/141; layers 20 crates; WASM 16 crates. No new ignored tests.

No DOCX serialization, text content, fonts, assets, installation behavior or Thai shaping is changed. This is a pagination change only. The official Windows report is not regenerated here.

## งานที่ยังค้าง

ยังเหลือ corpus-breaks 99 เอกสาร ผล Linux 120/219 ยังรอยืนยันบน Windows การตรวจเพิ่มเติมพบ 5 เอกสารที่คำหลัง normalize Unicode NFKC ตรงกันทุกหน้าแต่ลำดับที่ PDF extractor อ่านต่างกัน ได้แก่ govuk-024 และ zenodo-020/030/050/070 ทั้งห้ายังเป็นไม่ผ่านตามเกณฑ์เดิม ไม่มีการเปลี่ยนตัวตรวจเพื่อเพิ่มคะแนน
