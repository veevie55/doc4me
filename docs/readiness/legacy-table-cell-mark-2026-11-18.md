# Trailing paragraphs in legacy table cells · 2026-11-18

The third page of govuk-045 has a Duration table row whose right cell ends with an empty 14 pt paragraph and has `w:hideMark`. The document uses compatibility mode 14. The Duration text starts at the same height in the reference and Doc4Me PDFs, but the next row begins 17.17 pt earlier in Doc4Me. That extra space lets the first line of the postal address fit on page 3. In Word it begins page 4.

Removing only that cell's `hideMark` in a scratch copy adds 17.09 pt and restores all 23 pages' first and last words. The reference and corpus DOCX were preserved. The same scratch intervention on six comparable cells in govuk-043 produces a byte-identical PDF.

The layout change counts the trailing empty paragraph in a populated cell before compatibility mode 15. A legacy cell containing only an end mark still honors `hideMark`. Modern cells retain the existing behavior observed in govuk-066. The layout regression covers these three cases and caret access to the mark.

The isolated comparison uses renderer 9eecfa2 and the same pinned prepared fonts for both binaries. The original five-set verifier reports 219/219 opens, round trips and stability checks, 216/219 page counts, and 130→131/219 strict page boundaries. The exact comparison of 2,941 pages finds one improvement, the first word on govuk-045 page 4, with no losses. Every other failure reason matches the baseline. Candidate CLI SHA-256: `0f8f9f540742490d25681d488889f0432a90f9cf7272db2c6311d49a5fa452a4`. Canonical CI passes all six gates on the isolated change.

While those checks ran, main advanced to `da3799e`, which includes independent changes for bracket wrapping and unwrapped text boxes. The isolated numbers above measure the table change alone. The latest official Windows report at this point remains run 44224006895.

The integrated prepared-font CLI (`d2f344e44cfc9a411da795303f4a6dc9b24a5f89326bed63bec1e73776e56eef`) passes open, round trip and stability on 219/219 originals. Page counts match 216/219 and boundaries 133/219. Relative to the 130/219 baseline, govuk-045, zenodo-092 and zenodo-163 newly pass. Failure reasons for all remaining rows match. The exact comparison across 2,941 pages finds seven improved endpoints and no losses. All 438 input hashes were rechecked. The govuk-045 PDF is byte-identical to the isolated correction.

Integrated canonical CI passes all six gates. Native tests report 2,624 passes, zero failures and the existing ignored test. Asset attribution covers 152 files, layering covers 20 crates and WASM checks 16 crates.
