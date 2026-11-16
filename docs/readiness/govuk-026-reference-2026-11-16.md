# govuk-026: the reference PDF predates the .docx · 2026-11-16

**For the owner's decision; no row has been excluded.** govuk-026 fails the page-count check
(Doc4Me 4 pages, the reference PDF 7). The evidence says the PDF was made from an earlier
revision of the form, not from this `.docx`:

| Evidence | Value |
|---|---|
| Reference PDF (`RAIB_N2.pdf`) | Creator Acrobat PDFMaker 9.1 for Word, Producer Acrobat Distiller 9.3.3, created 2012-01-11 |
| `.docx` (`RAIBN2.docx`) | `docProps/app.xml` AppVersion 16.0000, `<Pages>4</Pages>`; modified 2025-01-30; compatibility mode 15 |
| Word's own page breaks saved in the `.docx` | 3 `w:lastRenderedPageBreak`, all at section starts (3(2) breakage, 3(3), 3(5)): 4 pages, as Doc4Me lays it out |
| The PDF | starts a page at every "3(x)" heading row (7 pages), including 3(2) buckling, 3(4) and 3(6), which sit mid-table with 400+ pt free on the page before; no page break, `pageBreakBefore`, `cantSplit`, header row or style in the `.docx` asks for one |

The Word that last saved the `.docx` laid it out on 4 pages, as Doc4Me does. The file names
differ only by an underscore, so the corpus rule (`docs/readiness/README.md`: another file name
means `excluded`) does not exclude it automatically. Excluding it as a mismatched reference, as
the owner did for govuk-003 and zenodo-104, is the owner's decision.

How checked: `pdfinfo`, `unzip` of the pinned files (hashes match `corpus.tsv`), the `.docx`
XML (rows, `trHeight`, section properties, styles), and `pdftotext -bbox` of both layouts.
