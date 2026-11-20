# Corpus rows whose pagination is deferred · 2026-11-20

On 2026-11-20 the owner decided that a corpus row whose page boundary can't be matched from its
`.docx`, or whose pages hold every word Word's do while pdftotext reads them in another order, may
have its pagination deferred (`check` = `deferred` in `corpus.tsv`, reason in `note`). A deferred
row's failing page-count or page-boundary check counts as **deferred**: not passing in the measured
number, left out of the number without deferred rows. A passing check still passes; opening, round
trip and stability are still measured. No row is removed and no reference file changes.

Each row was checked before it was deferred. "Baselines" below compares every word's baseline
(from the PDF text matrix, which doesn't depend on the font descriptor) in Doc4Me's PDF with
Word's, words matched by text and position; "content order" compares `pdftotext -raw`.

## The reference can't be matched from the `.docx` (owner's group B)

| Row | Evidence |
|---|---|
| zenodo-006 | Page 1 of the reference starts with "SINAU: Jurnal Ilmiah Pendidikan dan Pembelajaran", volume and E-ISSN; none of it is in any part of the `.docx` (another edition). |
| govuk-066 | The footer's `DATE` field holds 21/09/2022; Word printed the day it made the PDF (08/02/2024). |
| zenodo-018 | The header is gradient WordArt ("LECTURE NOTE … NOT FOR SELLING"); Word's PDF has no characters for it, Doc4Me keeps its text, so page 12's last extracted word differs. |
| govuk-068 | Word's PDF writes the Arabic text as presentation forms (U+FExx); the same text compares as other words. |
| zenodo-149 | Word's PDF has no character for its "ti" ligature ("Introduc�on", page 6). |
| zenodo-022 | Page 15 starts at the same table row in both; only the order of its two-column bullets (and the ordinal "3e") differs in extraction. |
| zenodo-160 | Word's PDF was printed with a review comment in a markup pane ("Commented [SP1]: I updated this table … added bp units etc."): its pages are shrunk to about 77% beside a grey pane, and page 8 carries the balloon's 20 words. The comment is in no part of the `.docx`; page 8's body words are the same in both PDFs, and Word's last extracted word there ("etc") is the balloon's. (Added 2026-11-20 while checking group D.) |
| zenodo-126 | Page 1's first words in Word's PDF, "SAFER wearables study", are text Word drew from the EPS logo picture (`wearables_logo.eps`); the `.docx` holds no such text. Every baseline of the page's own text is within 0.32 pt of Word's. (Found while checking group C; deferred the same way.) |

## Every word is on the page, only the reading order differs (owner's group A)

`tools/corpus_analysis.py triage` found these ten pages with the same multiset of words as Word's.

| Row | Page | Baselines | Content order | Cause of the order |
|---|---|---|---|---|
| zenodo-020 | 1 | within 0.39 pt | same | line grouping by declared glyph boxes |
| zenodo-029 | 2 | within 0.20 pt | same | Adobe PDFMaker declares Arial's head box (1018/−376); the page number's box overlaps the first heading's line |
| zenodo-030 | 16 | within 0.05 pt | — | line grouping by declared glyph boxes |
| zenodo-050 | 2 | within 0.17 pt | — | line grouping by declared glyph boxes |
| zenodo-070 | 10 | within 0.17 pt | — | line grouping by declared glyph boxes |
| zenodo-072 | 6 | within 0.14 pt | — | line grouping by declared glyph boxes |
| govuk-024 | 2 | within 0.4 pt after the fix below | same | PDFMaker declares Arial Black's head box (ascent 1083) |
| govuk-048 | 1 | within 0.4 pt to mid-page after the fixes below, drifting to 1.42 pt higher across the stacked boxed tables at the foot | — | the nested "Impact Assessment" box (top 34.9 pt) is read before "ANNEX A" (top 37.3 pt); the same in both PDFs within 0.25 pt; the page ends on the same word |
| zenodo-173 | 5 | within 0.19 pt after the fix below | — | the page number and the last cell swap |
| govuk-005 | 12 | 3.6 pt apart (not fixed) | — | Word keeps 3.55 pt space before at the top of a page opened by a next-page section break; not suppressing it there fails zenodo-022, -069, -081 (page counts) and -109 and still leaves this page apart |
| govuk-054 | 11 | the same except one text box | — | the figure's text box turned −90° keeps its text upright in Doc4Me (by design); Word turns it |

Matching the descriptor style per producer isn't possible: the `.docx` doesn't say which program
will print it, and writing head-box descriptors for every font was measured on 2026-11-19
(+4/−6 documents) and withdrawn.

### Fixes made while checking

- **govuk-048** (added 2026-11-20): an empty paragraph whose mark is hidden (holding a bookmark)
  between two tables took a 13.8 pt line in Doc4Me and none in Word; a line ending in a 12 pt space
  and an empty form field's en spaces ("Contact for enquiries:") was set 2.12 pt lower than Word's.
  Both are fixed; the page's first 600 pt now match Word's baselines within 0.4 pt.

- **govuk-024**: Arial Black, drawn by a stand-in, now keeps its own line (win/hhea 2254/634 per
  2048): its 24 pt heading's baseline is 26.4 pt below the row's top in Word's PDF, and was 3.55 pt
  higher in Doc4Me. The page's baselines now match Word's.
- **zenodo-173**: every cell's text in a row starts below the row's largest top margin. Its first
  two columns have 4 pt top margins and the other three none; Word puts all five on the same
  baselines, Doc4Me put the last three 4.02 pt higher. A split row's continuation keeps that
  offset and its height. Pages 5, 12, 18 and 22 now match within 0.2 pt.
