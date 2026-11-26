# Fixed-layout tables take their first row's cell widths · 2026-11-18

Corpus-breaks follow-up. Criteria, accepted, excluded and deferred rows are unchanged.

## Finding

zenodo-115's two-column "Theoretical Contribution" table is `tblLayout fixed` with grid columns
5976/3096 twips (298.8/154.8 pt) but both cells `tcW` 4608 twips (230.4 pt). Word's PDF puts the
right column's text at x = 314.3 pt, 219.5 pt right of the left column's (94.8 pt): two equal
230.4 pt columns, the cell widths. Doc4Me used the grid (right column at 386.7 pt), so the bullets
wrapped differently.

`crates/layout/src/table.rs` now uses the first row's cell widths for a fixed-layout table when
that row has one cell per grid column, each with a width in points (no spans, no percentages), and
any of them differs from the grid by more than 1 pt. Other tables keep the grid.

The corpus has 24 fixed tables whose first-row cell widths disagree with their grid (govuk-009,
-048, -061; zenodo-051, -080, -092, -108, -115, -118, -137).

## Results (Linux, local)

438 source hashes verified, pinned prepared craft-fonts, release CLI (`af0f1d84…9af4110`),
`cargo xtask readiness --corpus --dry-run --diagnostics --jobs 2`, at `b1d5a87` plus this change:
open/round-trip/stability 219/219, page counts 218/219 (zenodo-019, as before), boundaries
**133/219**. Against the previous local run (`bad5ec5`, 131/219): govuk-045 (the concurrent
`b1d5a87` table-mark fix) and **zenodo-137** pass; nothing that passed fails. Every row:
[`fixed-table-widths-comparison-2026-11-18.tsv`](fixed-table-widths-comparison-2026-11-18.tsv).

Isolation: the same CLI with the rule switched off (a temporary environment switch, since removed)
gives zenodo-137 7 pages with pages 6 and 7 starting or ending differently; with the rule all seven
match Word's.

zenodo-115 still fails: its abstract wraps "authority," | "and adaptive" where Word keeps "and" on
the line (justified text; not analysed further).

Test: `a_fixed_table_takes_its_first_rows_cell_widths_over_a_disagreeing_grid`.

## Page-1 cases set aside

- zenodo-006: Word's page 1 starts with a journal header ("SINAU: Jurnal Ilmiah…") that is not in
  the `.docx` at all (no header part or body text contains it): the PDF comes from a later edition.
- zenodo-126, govuk-048: the same words in a different extraction order at the top of page 1.
