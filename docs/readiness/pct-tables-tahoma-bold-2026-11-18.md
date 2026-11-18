# Percentage-width tables before Word 2013, and Tahoma Bold widths · 2026-11-18

Corpus-breaks follow-up. Criteria, accepted, excluded and deferred rows are unchanged.

## 1. A percentage table width includes the cell side margins (compatibility mode ≤ 14)

govuk-044 page 8 gained a page once Tahoma Bold widths were tried (the reason an earlier session
dropped them): its answer tables wrapped "Don’t know" and "identified at" where Word didn't. The
tables are autofit with `tblW` 4888 pct (97.76%) in a compatibility-mode-14 document whose text is
453.15 pt wide. Doc4Me drew them 443.0 pt wide (97.76% of the text width); Word's PDF puts the
columns 75.26 pt apart, as the file's grid (4553 + 3 × 1506 = 9071 twips = 453.56 pt). That is
97.76% of the text width **plus** both 5.4 pt cell side margins: before Word 2013 a table stands
out into the margin by its cells' side margins, and a percentage is of that wider room.

Every top-level percentage-width table in a mode ≤ 14 corpus document, comparing its saved grid
with each reading: [`pct-tables-mode14-scan-2026-11-18.tsv`](pct-tables-mode14-scan-2026-11-18.tsv).
Of 44 tables, 28 match only the wider room (govuk-010, -043, -044, -054, one centred), 5 match both
(no margins or above 100%), none match only the text width; govuk-048's grids are 10–30 twips off
both (it has zero cell margins, so both readings agree) and zenodo-108's four centred tables match
neither (grid 10,428 against 10,368 and 10,584 twips).

`crates/layout/src/table.rs`: in compatibility mode 14 and earlier a percentage table width is of
the available width plus the table's default left and right cell margins. Mode 15 is unchanged.

Test: `a_percentage_table_width_counts_the_cell_margins_before_word_2013`.

## 2. Tahoma Bold takes Tahoma Bold's widths with Arimo Bold

`crates/fonts/src/latin_metrics.rs` gains `TAHOMA_BOLD` (62 advances) from the existing audit
[`tahoma-width-audit-2026-11-15.tsv`](tahoma-width-audit-2026-11-15.tsv) (public `/Widths` facts
of the pinned reference PDFs; no font program read), and `metric_fit` applies it to bold Tahoma
(italic still keeps Arimo's). govuk-043's bold answer text ("You can skip any questions…") was
2–3% narrower than Word's.

## Results (Linux, local)

438 source hashes verified, pinned prepared craft-fonts, release CLI (`a4f38440…ae278821`, built
with temporary environment switches since removed, run with both rules on),
`cargo xtask readiness --corpus --dry-run --diagnostics --jobs 2`: open/round-trip/stability
219/219, page counts 218/219 (zenodo-019 only; govuk-044 stays at Word's 10), boundaries 136/219,
no row changes status. Every row: [`pct-tables-tahoma-bold-comparison-2026-11-18.tsv`](pct-tables-tahoma-bold-comparison-2026-11-18.tsv).

Lines identical to Word's (116,977): 105,998 → 106,155. Gains: govuk-043 +77, govuk-038 +42,
govuk-067 +35, govuk-044 +8, govuk-054 +2, govuk-060 +1; losses: zenodo-108 −6 (its centred tables,
above), govuk-061 −1, zenodo-054 −1. Tahoma Bold alone (no table rule) gave govuk-044 11 pages; the
table rule alone gave +10 lines.
