# Corpus boundaries: extraction failures and reading order — 2026-11-17

Status: investigation and verifier correction; no pagination score improvement is claimed.

## Current baseline

The latest committed Windows measurement is run [44078179776](https://github.com/veevie55/doc4me/actions/runs/44078179776), source `019b0ff`: 219/219 page counts, 127/219 boundary passes, 95.0% overall and 96.8% without deferred rows. There are 92 remaining boundary failures. The older Linux figures at the top of `handoff.md` predate several fixes and this Windows run.

## Failed extraction must not pass

`xtask/src/readiness.rs::page_words` previously ignored the exit status of `pdftotext` and converted a process-launch error into an empty word vector. When extraction failed on both PDFs, the two empty vectors could match and the boundary check could pass. A failure on one PDF could also match a genuinely blank page on the other. `pdfinfo` likewise did not check exit status, and a zero page count could skip every boundary comparison.

Both tools now require successful exit and UTF-8 output; both commands explicitly request UTF-8. `pdfinfo` must report a positive page count. Failures report the tool, and the comparison identifies the PDF side and page where applicable. A text-extraction failure leaves a successfully measured page-count result intact and fails the boundary result. Successful blank pages still match. Stderr included in diagnostics is limited to 240 characters.

Three regression tests cover process launch failure, unsuccessful exit with empty or partial output, invalid UTF-8, missing/zero/negative/overflowed page counts, errors on either or both sides, matching blank pages and unchanged exact endpoint comparison. Corpus membership, accepted/excluded/deferred decisions and the word-comparison rule are unchanged. This change does not establish that any earlier official result was affected by an extractor failure.

## Five same-page-text cases

Using renderer `aba659a`, the pinned craft-fonts checkout `c1db6d1a8ed772191241c3d185e2104be84d8691` and `tools/prepare_corpus_fonts.py`, all ten DOCX/reference inputs were SHA-256 verified against `corpus.tsv`. The five documents total 52 pages. Each page has the same lowercase alphanumeric word multiset as its reference, without NFKC normalization. Six pages still differ under the original first/last-word criterion:

| Document | Page | Doc4Me first / last | Word first / last |
|---|---:|---|---|
| govuk-024 | 2 | annex / n | rule / n |
| zenodo-020 | 1 | bay / 2023 | bay / memo |
| zenodo-030 | 16 | available / containers | certificate / containers |
| zenodo-050 | 2 | brief / ca | storage / ca |
| zenodo-050 | 3 | brief / ca | ethics / ca |
| zenodo-070 | 10 | infraestructura / banco | infraestructura / 80 |

[Endpoint and word-count TSV](extraction-order-2026-11-17.tsv). These are diagnostic observations, not replacement pass criteria. Equal multisets do not prove identical geometry, text order or complete visual fidelity. All five documents remain failing.

`pdftotext -bbox-layout` gives useful leads:

- govuk-024 page 2: the reference extracts the right-hand “Rule 15.18” block first (yMin 21.408), then left-hand “Annex A” (24.042). Doc4Me extracts Annex A first (21.458). This is sensitive to font bounds and cell geometry; the reference's Arial Black is not embedded. Descriptor ascent/descent alone cannot establish Word's line-spacing rule.
- zenodo-020 page 1: Doc4Me's final extracted block is “June 2023”, near yMin 696.649. The reference's final extracted block is the table entry ending “OSTP Memo”, near yMin 472.938. The footer's words are present in both page word multisets.
- zenodo-030 page 16: the small “Available from USP website” block is extracted before the left-hand “Certificate of Analysis” heading in Doc4Me. Both are present on the same page in the reference.
- zenodo-050 pages 2/3: Doc4Me extracts right-hand “Brief Guide” first (yMin 95.054), before the left-hand body headings. The reference extracts those headings first (yMin 118.614). Sorting everything simply by top coordinate would not reproduce the reference's reading order.
- zenodo-070 page 10: Doc4Me extracts the short “Banco” block last (yMin 525.659); the reference finishes in a larger reference-list block that extends to yMax 700.161. The same words occur on the page in both files.

No font, renderer, layout or PDF draw-order patch was made based on these leads. Next work should compare these blocks' geometry and reading-order grouping, then test a general rule against the full corpus before changing pagination or export.

## Validation and limitations

- `cargo test -p xtask`: 38 passed, including the three new verifier regressions.
- A standalone smoke harness compiled the final comparison functions directly from `xtask/src/readiness.rs` and used real Poppler: matching blank PDFs pass; missing files and out-of-range page extraction fail; all five audited documents retain their strict boundary failures. A second harness using the same Rust `words` function confirms all 52 page word multisets and the six mismatches in the TSV.
- Workspace validation exposed an existing engine test that assumed a comment's Word `w:date` equals `dateUtc`. The program intentionally stores local wall-clock time in `w:date` and UTC in `dateUtc`. The test now bounds both values by the creation interval after applying the local offset, allowing second and daylight-saving boundaries. The focused test passes in separate processes under `TZ=Asia/Bangkok`, `TZ=UTC` and `TZ=America/New_York`. This changes test expectations, not comment storage or rendering.
- All 438 corpus inputs were eventually downloaded and hash-verified after an initial Zenodo HTTP 429 response. Runs interrupted by resource exhaustion were discarded. Local Linux results do not replace the official Windows measurement.

`cargo xtask ci` passes all six steps: **2,613 native tests passed, zero failed, one existing ignored**; assets **141/141**, layering **20 crates**, WASM **16 crates**. Validation uses the project's opt-level **1**, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0` and `RUST_TEST_THREADS=1`. Performance thresholds and ignored-test lists are unchanged. The initial opt-level-0 experiments are not used to assess the performance gates.

The clean full Linux corpus run uses the frozen prepared-font renderer built from `aba659a` (debug CLI, opt-level 0, SHA-256 `24aee41ff665564abea3e9b76c6ca38f1252261f406a8b665332633f00d63151`) and the final verifier. With a fresh work directory, `readiness --corpus --dry-run --diagnostics --jobs 2 --cli <frozen CLI>` gives:

| Check set | Passed / checked |
|---|---:|
| Open | 219/219 |
| Round trip | 219/219 |
| Page count | 216/219 |
| Exact page endpoints | 128/219 |
| Stability | 219/219 |

[All 219 results](verifier-corpus-2026-11-17.tsv), including the manifest's accepted-pagination flag; a dash in a reason column means no failure. Pass counts retain the existing owner-accepted pagination of govuk-026; that row is not a new exact-match measurement. No source pair is missing and no Poppler or work-folder error occurs. The three Linux count failures are govuk-068 (16 versus 15), zenodo-019 (10 versus 11) and zenodo-114 (72 versus 71). These Linux observations are not a comparison against the Windows font environment, nor evidence of a renderer improvement: rendering code did not change. The frozen debug CLI is used for corpus correctness only; its performance measurements are not used to replace the official Windows readiness score.

The committed Windows reports remain unchanged. No Release or Pages deployment is part of this work.

## Revision history

| Date | Change |
|---|---|
| 2026-11-17 | Recorded the fail-open extraction bug, its regression coverage and the 52-page reading-order audit. |
