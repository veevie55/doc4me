# Query marks, joined em dashes and list-style indents · 2026-11-18

Corpus-breaks follow-up. Criteria, accepted, excluded and deferred rows are unchanged.

## Method

A corpus-wide line comparison (`pdftotext` of Word's PDF against Doc4Me's, lines aligned with
Python's `difflib`) and a split census: for every source token that is not plain letters (from
`doc4me-cli text`), where a line of Word's or Doc4Me's PDF ends inside it, and at which character.

## 1. No break after "?"/"!" before a letter, nor before a joined em dash

Splits inside a token that Doc4Me made and Word never did: after "?" or "!" with a letter or digit
straight after (13 + 2, all web addresses such as `watch?` | `v=…`; UAX #14 EX ÷ AL), and before
an em dash joined to the word before it (7, `scenarios` | `—adversarial`; UAX #14 B2). Word: 0 of
each. `crates/layout/src/para.rs` drops both break opportunities. zenodo-012 passes (its page 22
reference wrapped a YouTube address at "watch?").

Test: `no_line_break_after_a_query_mark_in_an_address_or_before_a_joined_em_dash`.

## 2. A list style's own indents stay over its list level's

govuk-046's body paragraphs use style "Number 3", which brings list 13 and indents 720 twips with a
720 hanging indent; the list's level 0 says 1131/774. Word's wrapped lines start 36 pt in (x = 108
on a 72 pt margin); Doc4Me used the level (x = 128.55) and wrapped 36 lines shorter than Word.

Rule now: list-level indents apply unless the paragraph sets its own (as before) **and**, for a
paragraph in a list only through its style, unless the style that brings the list (or a style based
on it) sets them. Indents inherited from further up the chain stay under the level's: zenodo-067's
"Liste puces - Noir" brings list 4 while its base style "Liste numéros - Rouge" sets 568/284, and
Word's lines follow list 4's level (x = 329.47; a first version that let any style in the chain win
moved them and lost 29 matching lines there, which is how this was found).

Test: `a_list_style_with_its_own_indents_keeps_them_over_the_list_levels` (both cases).

## Results (Linux, local)

438 source hashes verified, pinned prepared craft-fonts, release CLI (`3e00b25d…bc85046b`),
`cargo xtask readiness --corpus --dry-run --diagnostics --jobs 2`: open/round-trip/stability
219/219, page counts 218/219 (zenodo-019, as before), boundaries 135 → **136/219** (+zenodo-012),
nothing that passed fails. Every row: [`query-dash-list-indent-comparison-2026-11-18.tsv`](query-dash-list-indent-comparison-2026-11-18.tsv).

Lines identical to Word's, whole corpus (116,977 Word lines): 105,246 (`576a7b1`) → 105,294 with
rule 1 → 105,998 with rule 2. No document has fewer matching lines with either rule; the largest
gains are govuk-046 (850 → 1,231 of 1,248), govuk-038 (+90), govuk-002 (+88), zenodo-147 (+76) and
zenodo-096 (+36). govuk-046 still differs from page 11 (a table's first rows).
