# Bracket line breaks and unwrapped text boxes · 2026-11-18

Two corpus-breaks fixes, each found from a page-1 boundary failure and checked against the whole
corpus. Criteria, accepted, excluded and deferred rows are unchanged; the official Windows result
is still the latest workflow run until the readiness workflow measures this source.

## Results (Linux, local)

All 438 source hashes verified (`cargo xtask readiness --download`: nothing to fetch), pinned
prepared craft-fonts (`tools/prepare_corpus_fonts.py`, revision
`c1db6d1a8ed772191241c3d185e2104be84d8691`), release `doc4me-cli` built with
`CRAFT_FONTS_REQUIRED=1`, `cargo xtask readiness --corpus --dry-run --diagnostics --jobs 2`:

| Run | CLI SHA-256 | Open / round-trip / stability | Page counts | Boundaries |
|---|---|---|---|---|
| Baseline `ce70748` | `5eccf04f…2d9a55` | 219/219 each | 218/219 | 129/219 |
| + bracket rule | — | 219/219 each | 218/219 | 130/219 (+zenodo-092) |
| + bracket rule + unwrapped boxes | `45f45708…8bc14` | 219/219 each | 218/219 | 131/219 (+zenodo-163) |

No document that passed before fails now. Every row: [`bracket-nowrap-comparison-2026-11-18.tsv`](bracket-nowrap-comparison-2026-11-18.tsv).
The only page-count failure on Linux is zenodo-019 (10 vs 11), as before.

## 1. No line break between a closing and an opening bracket

zenodo-092 page 1 ended one line early: Doc4Me broke "conflict(Adams, 2022)" | "(Adams, 2022)",
where Word broke "conflict(Adams," | "2022)(Adams, 2022)". UAX #14 allows a break between `)` and
`(` (no rule joins CP and OP), so `unicode-linebreak` offers one; Word doesn't use it.

Evidence: every source token with `)` or `]` directly followed by `(` or `[` (from `doc4me-cli
text`), looked up in the reference PDFs' text (`pdftotext`): Word keeps all 36 occurrences whole and
splits none; the baseline Doc4Me PDFs split one (zenodo-092). `crates/layout/src/para.rs` now drops
that break opportunity; a space between the brackets still breaks.

Test: `no_line_break_between_a_closing_and_an_opening_bracket` (fails with the fix removed).

## 2. Text boxes that don't wrap their text (`wrap="none"`)

zenodo-163's footer is a 207.7 pt floating text box whose `wps:bodyPr` says `wrap="none"`, holding
the one-line "“RAQAMLI IQTISODIYOT” ILMIY-ELEKTRON JURNALI | 15-SON | 2026". Doc4Me ignored the
attribute, wrapped the line in the box and hid the overflow, so page 1 ended with "ELEKTRON" where
Word's ends with "2026".

- `TextBody::no_wrap` (`crates/doc/src/lib.rs`), read from and written back to `wrap="none"`
  (`crates/docx/src/read/story.rs`, `crates/docx/src/write/story.rs`); a box that wraps writes no
  attribute, as before.
- `crates/layout/src/textbox.rs` lays such a box's text out in lines long enough for its longest
  line (measured at 4,000 pt), so each paragraph keeps its own lines and runs past the right margin.
- Five corpus documents have boxes with `wrap="none"` (govuk-002, govuk-057, zenodo-134, zenodo-141,
  zenodo-163); only zenodo-163 changes status.

Tests: `a_text_box_that_does_not_wrap_keeps_each_paragraph_on_one_line` (fails with the layout
change removed) and `a_text_box_that_does_not_wrap_keeps_wrap_none_through_save_and_reopen`.

Known limit: unwrapped text starts at the box's left margin whatever the paragraph alignment; Word
centres or right-aligns an overflowing centred or right-aligned line on the box.

## Leads looked at, not changed

- govuk-068 page 1 differs only because Word's PDF uses Arabic presentation forms (`وﻋﻼج`) where
  Doc4Me writes base letters (`وعالج`); the words are the same. Changing the comparison would change
  the measure, so it was not done.
- zenodo-119 page 1: Word starts page 2 with "Zusammenfassung:" (1.5-line spacing, 12 pt), Doc4Me
  fits it on page 1: vertical drift, cause not found.
- zenodo-020 page 1: same visible text; extraction-order case (see
  [extraction-order-2026-11-17.md](extraction-order-2026-11-17.md)).
