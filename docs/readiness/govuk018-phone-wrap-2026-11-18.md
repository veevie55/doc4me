# govuk-018 phone-row investigation · 2026-11-18

Status: diagnostic only; the document still fails strict page-boundary verification. No renderer, source corpus file, criterion or accepted/excluded/deferred decision changed.

The source has compatibility mode 15. The fixed nested table in section A has a 3,312-twip column for a cell containing 39 ordinary leading spaces followed by “Fax”, in nominal 10 pt Verdana. Its 165.6 pt grid width minus the two default 5.4 pt cell margins leaves 154.8 pt. The measured PDF width table used by the Inter substitution gives space=352, F=575, a=601 and x=592 units per 1,000 em: the total advance is **154.96 pt**, 0.16 pt beyond the available width. Doc4Me wraps the word and grows the row.

Using the rebuilt prepared-font CLI from unchanged `cedff9f` renderer source, a scratch copy removes **one leading space from only the first Fax field**. The original DOCX and the second Fax field are unchanged. This counterfactual removes the extra line and moves subsequent text up **12.153 pt**, confirming the source of the row-height jump. Section C moves back to page 1 (baseline 782.417 pt, reference 779.760 pt), restoring page 1’s last word and page 2’s first word. It does not restore the complete document’s layout: page 2’s last word remains “continued” versus “9”, and page 4’s last word remains “documents” versus “i”. Exact Rust tokenization is copied unchanged from the production verifier for this diagnostic; all six page counts remain unchanged. The scratch DOCX/PDF are not committed or used to score readiness.

| Page 1 text baseline (pt from top) | Word reference | Current | Scratch: 38 spaces |
|---|---:|---:|---:|
| Section A Fax | 438.240 | 449.834 | 437.681 |
| Section A Email | 463.560 | 475.641 | 463.487 |
| Section B Fax | 686.760 | 699.727 | 687.574 |
| Section B Email | 712.080 | 725.534 | 713.381 |

Reference observations read only public PDF text operators and font dictionaries, never embedded font programs or outlines. The reference draws both Fax labels at an effective 9.96 pt size; the text matrix, character/word spacing and TJ adjustments must be considered together. A PDF drawing size alone does not establish the layout advance. Both observed Verdana documents also use 11.04 pt for nominal 11 pt text, so applying a global scale inferred from the 10 pt example is unsupported.

Next: distinguish fixed-table cell geometry from layout advance rounding with controlled observations. Do not increase the global line-fit tolerance or change every font size merely to accommodate this 0.16 pt example. Preserve the original boundary criteria. Official Windows run 44224006895 remains **219/219 page counts, 129/219 boundaries, 90 failures**.

The same exact tokenizer finds identical per-page word multisets on pages 1–4 and the blank page 6 of the scratch comparison. Page 5 still has five multiset differences from a fragmented “environmental” word. This separates remaining extraction/word-metadata leads from the measured extra row; equal multisets do not make any failed endpoint pass. Inter’s open regular, semibold, bold and italic faces have identical hhea and OS/2 typo ascent/descent (1984/−494 at 2048 units/em, USE_TYPO_METRICS set), so switching those metric sources cannot resolve these Inter reading-order differences.

## Verification of this investigation

The prepared-font CLI rebuild has the same SHA-256 as the previous validated final binary: `68fb15a87ae87bb79add51df6d9e784c11de065c7af0a5cc1d49ce2b5bc49198`. A fresh run of the original five-set verifier reproduces every committed result/reason across all 219 documents: open/roundtrip/stability 219/219, Linux page counts 216/219 and boundaries 130/219. All 438 original input hashes were reverified. The scratch comparison uses the unchanged Rust tokenizer and positive page-count/form-feed checks; it is not a readiness measurement.

`cargo xtask ci` passes all six gates: 2,621 native passed, zero failed, one existing ignored; all 152 current assets attributed, 20 crates respecting layering and 16 WASM crates checked. The project's opt-level 1 is retained for performance tests. The prepared opt-level-0 binary is used only for corpus correctness. Logs: `/workspace/work/ci-windows-confirmation.log` and `/workspace/work/cedff9f-baseline-restored.log`. CI ran at `88a7d41` with only these documentation edits; the subsequent owner commit `e2bf0ae` deletes two README lines and changes no tested code or assets. Generated readiness reports were not edited.

## ภาษาไทย

ยืนยันแล้วว่าช่อง Fax ของตาราง A ขึ้นบรรทัดใหม่เพราะความกว้างรวมเกินพื้นที่ประมาณ 0.16 จุด การลดช่องว่างหนึ่งตัวในสำเนาทดลองลดความสูงแถวลง 12.153 จุด และย้ายหัวข้อ C กลับหน้าหนึ่งได้ แต่คำท้ายหน้าสองและหน้าสี่ยังไม่ตรง จึงยังไม่ทำให้เอกสารผ่านทั้งฉบับ จึงยังไม่แก้กฎจัดบรรทัดหรือเปลี่ยนเกณฑ์ตรวจรับ ต้องตรวจการคำนวณพื้นที่เซลล์และการปัดเศษความกว้างต่อก่อนแก้โค้ด เอกสารต้นฉบับไม่ถูกแก้ไข
