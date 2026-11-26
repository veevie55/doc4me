# Slashes, joined brackets and list-label text in the PDF · 2026-11-18

Corpus-breaks follow-up. Criteria, accepted, excluded and deferred rows are unchanged.

## 1. More places Word never breaks

From the split census (where a line of Word's or Doc4Me's PDF ends inside a source token; see
[query-dash-list-indent-2026-11-18.md](query-dash-list-indent-2026-11-18.md)), the remaining
splits only Doc4Me made: after a slash before a non-letter (`/` | `~/media`, zenodo-022;
`/` | `_Remarks`, zenodo-146; `Han/` | `[4orncp]`, zenodo-146) and before an opening bracket joined
to a non-space character other than a closing bracket (`Ch.` | `[arl]es`, zenodo-168). Word: none.
`crates/layout/src/para.rs` now keeps a break after "/" only before whitespace, and no opening
bracket joined to the character before it starts a line.

Test: `no_line_break_after_a_slash_before_a_sign_or_before_a_joined_bracket`.

## 2. List labels keep their text map in the PDF

zenodo-096 page 8 and zenodo-147 page 7 start with the list label "13." / "12.". The label is set
on Verdana's measured widths with Inter's narrower glyphs centred in them, so `pdftotext` saw a
gap and read "1 3." / "1 2.": the page's first word became "1". Body text already gives the PDF
writer each glyph's text range, which lets it mark such words with ActualText; labels passed none,
so the writer fell back. `crates/layout/src/display.rs` now passes one range per character when a
label run has one glyph per character. Drawing is unchanged.

Test: `a_list_label_passes_its_text_map_to_the_pdf`.

## Results (Linux, local)

438 source hashes verified, pinned prepared craft-fonts, release CLI (`e4cfaa7d…`),
`cargo xtask readiness --corpus --dry-run --diagnostics --jobs 2`: open/round-trip/stability
219/219, page counts 218/219 (zenodo-019 only), boundaries 136 → **138/219** (zenodo-096,
zenodo-147), nothing that passed fails. Every row:
[`slash-bracket-labels-comparison-2026-11-18.tsv`](slash-bracket-labels-comparison-2026-11-18.tsv).
Lines identical to Word's (116,977): 106,155 → 106,174 (rule 1: zenodo-146 +14, zenodo-022 +3,
zenodo-168 +2) → 106,217 (labels: zenodo-147 +23, zenodo-096 +13, govuk-017 +4, govuk-018 +3).
No document has fewer matching lines.
