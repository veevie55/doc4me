# Layout and pagination parity with Microsoft Word

> **Last reviewed:** 2026-11-14 · **Last updated:** 2026-11-14 · **Change:** minor (Table/clipboard audit: 53 commands, 32 engine regressions plus raster and UI checks; CI 6/6, 2,543 native passed; 2,552 test attributes, 237,580 Rust lines, 730 commands, ribbon 504/504. Latest measured Windows source 6127a73: 95.1% without deferred, 188/220 pages and 108/220 boundaries; table source awaits measurement.) · **Target:** Microsoft Word (Microsoft 365) for Windows

**Table/clipboard audit · 2026-11-14:** ตรวจคำสั่งตารางครบ 53 คำสั่ง เพิ่มการเลือกคอลัมน์/เซลล์ว่าง เส้นขอบ 8 แบบและปากกาเส้นขอบ แก้หัวตารางหลายแถว การรวม/แยก/ขนาด/ช่วงเลือก และ rich clipboard ข้ามเอกสาร รวมรูป กล่องข้อความ กลุ่มวัตถุ แผนภูมิ SmartArt และ OLE พร้อมทรัพยากรที่อ้างอิง ดู [audit](readiness/table-clipboard-audit-2026-11-13.md) ผล CI ผ่าน 6/6; native 2,543, ignored 1 เดิม ไม่มี tag/Release คะแนน Windows ล่าสุดของ body source `6127a73` / report `527a832` / run 43442685451 ยัง **95.1% ไม่นับ deferred**; รอวัดซอร์สงานตาราง ไม่เปลี่ยนเกณฑ์ 220 / 22 deferred / 24 excluded


On Linux, a section-marker candidate was rejected after comparing all 15 corpus documents whose empty markers would change: page-count passes moved 10/15 → 11/15, but boundary passes fell 5/15 → 4/15. zenodo-053 improved from 59 to Word's 58 pages; zenodo-010 moved its body another 12.65 pt away from the reference and changed its first extracted word from Guuboo to GAWP. The candidate and its unproven global regressions were reverted. Investigate column balancing before changing marker height; official Windows pagination remains 188/220 and 108/220.

A document only "opens correctly" if its pages break where Word breaks them: the same lines, the
same page count, footnotes and pictures on the same pages. This checklist covers line breaking,
pagination and object placement in `crates/layout`. Typing behaviour is in
[`typing-parity.md`](typing-parity.md); the reading and writing of the properties is in
[`file-format-parity.md`](file-format-parity.md).

**Dimension: ~55% (estimated), 90–145 h to full** (the sum of the rows below). The engine implements
most of Word's layout rules; the latest official Windows corpus measurement (run 43333680851,
source `92025e8`, report commit `c098032`) matches Word's page count for 188/220 documents and every
page's first and last word for 108/220. The current code can use DOCX `w:altName` fonts when the
primary family is unavailable, with paragraph, graphic-text and DOCX round-trip regressions.
Run 43011248444 had 182/222 pages and 99/222 boundaries before zenodo-104 and govuk-003 (reference PDFs not made from their .docx) were excluded; the latest has 188/220 and 108/220. In run 42334133340, govuk-005
matched Word's 36 pages but failed page 12: Doc4Me starts with “do”, Word starts with “yes”,
and both end with “account”. This is now recorded by the first/last-word diagnostic. Windows
font diagnostics captured substitutions but did not establish a safe broad fix; MS Gothic did
not appear in the sampled results.
The `w:gridBefore` and
`w:gridAfter` row properties preserve omitted table columns; govuk-009 now passes both pagination
checks. A page break at the start of a table row no longer creates a blank page; govuk-003 now
matches the reference's five-page count, but its page-boundary check still differs. The zero-size
Word frame marker fix prevents later paragraphs in zenodo-030 from being clipped; that document
still lays out to 34 pages versus Word's 32. Direct visual comparison on the remaining cases is
still needed.


ผลเทียบ Linux ในรอบ z-order/Continuous section (220 ฉบับ, SHA-256 ตรง manifest ทั้ง 440 ไฟล์): เปิด/round-trip/เสถียร 220/220; จำนวนหน้า 181 → 182/220 โดย `govuk-037` เพิ่มจาก 78 เป็น 79 ตรง Word; ขอบเขตหน้า 95/220 คงเดิม ไม่พบการถดถอยทั้งห้าชุดตรวจ หน้า 60 ของ `zenodo-097` ที่เรนเดอร์ด้วย Poppler สูงสุด 900 px มี SHA-256 ตรงกับภาพจาก PDF อ้างอิง แถบสีเขียวจึงไม่ถูกพื้นสีขาวบังแล้ว ผลนี้ไม่แทนคะแนนทางการบน Windows

## Checklist

**ลำดับซ้อนและตัวแบ่งหน้าก่อน section (2026-11-12):** อ่าน/เขียน `wp:relativeHeight` และเรียงวัตถุข้ามย่อหน้าตามค่าที่อ่าน ภายในชั้นหน้า/หลังข้อความ โดยข้อความในกล่องและ hit area อยู่กับรูปร่าง; ค่าซ้ำรักษาลำดับเอกสาร คำสั่ง Bring Forward/Send Backward ย้ายทีละหนึ่งตำแหน่งโดยไม่เปลี่ยน wrapping และแยกกรอบวาดของแต่ละเซลล์ตาราง ตัวแบ่งหน้าล้วนก่อนย่อหน้าว่างปิดส่วนที่ส่วนถัดไปเป็นแบบ Continuous ต้องวางบรรทัดให้พอก่อนแบ่งหน้า (`govuk-037`); กรณีทั่วไปและส่วนที่เริ่มหน้าใหม่คงกฎเดิม Windows run [43180980471](https://github.com/veevie55/doc4me/actions/runs/43180980471) ของซอร์สเดิม `e65459b` ล้มก่อนเริ่มวัด เพราะ GitHub จัดหา hosted runner ไม่ได้ จึงไม่มีคะแนนใหม่จาก run นี้

✅ matches Word as far as our tests show · 🟡 implemented, differs or untested against Word · ❌ missing

| Feature | State | Evidence / gap | Hours |
|---|---|---|---|
| Line breaking (UAX #14 with Word's exceptions: no break after `/`) | ✅ | `layout::tests::no_line_break_right_after_a_slash` | — |
| Justification, Word 2013+ space shrinking (compat mode 15) | ✅ | #109 | — |
| Complex scripts (Myanmar, Khmer, Indic, Thai, Lao): shaped per script run with OpenType fallback fonts, one cluster per syllable, Myanmar wraps between syllables, Thai, Lao and Khmer wrap between dictionary words (ICU4X) | 🟡 | `layout::tests_complex` (#604), `thai_without_spaces_wraps_between_dictionary_words`; Thai text draws with the bundled Sarabun by default. Word for Windows may break Thai between other words than ICU4X's dictionary | 2–4 |
| Single line spacing from font metrics | ✅ | Word for Windows' measure on every platform: OS/2 usWinAscent + usWinDescent plus the external leading (the hhea line gap the win metrics don't cover) above the ascent, `fonts::word::tests::single_spacing_uses_word_for_windows_metrics`, `layout::tests_complex::sarabun_lines_are_spaced_like_word_for_windows`: Times New Roman 1.15 em, Calibri 1.22 em, Sarabun 1.853 em (Word for Mac's hhea measure gives Sarabun 1.30 em, so Word for Mac paginates a Sarabun document shorter). A substitute keeps the line of the font it stands in for: Cambria 1.172 em and Courier New 1.133 em from a table, other substitutes their hhea line (equal to the original's for metric-compatible fonts; Angsana New, Cordia New… drawn with Sarabun 1.30 em), `fonts::word::tests::substitutes_keep_the_requested_fonts_line` | — |
| Kerning and ligatures | ✅ | Pair kerning only with "Kerning for fonts" (`w:kern`) at that size and above, standard/contextual/historical/discretional ligatures only when `w14:ligatures` asks (Word's defaults: neither); complex scripts (Arabic, Hebrew, Thai, Indic) keep the font's own ligatures, so Arabic لأ always forms, `layout::tests_complex::letters_are_kerned_only_when_the_run_asks`, `ligatures_only_when_the_run_asks`; `w14:ligatures` is read and saved | — |
| TH Sarabun New / TH SarabunPSK when not installed | ✅ | Drawn with the bundled Sarabun scaled to their letter height (ก 400/586, x 340/500; Bold ก 407/586), set on their advance widths and kerning and spaced 1.331 em like them, so lines and pages break as with the real font (`fonts::thsarabun`, measured from TH Sarabun New 1.3 / TH SarabunPSK; `layout::tests_complex::th_sarabun_new_keeps_its_size_widths_and_line_spacing_when_drawn_with_sarabun`). Glyph shapes are Sarabun's, centred on the original widths. Thai fonts without measured metrics are drawn with Sarabun at its own size; a document in TH SarabunIT๙ says so when it opens. Their optional ligatures (fi, ฤๅ) aren't reproduced | — |
| Kashida justification (Arabic) | ❌ | no kashida in `crates/layout` | 3–5 |
| Distributed / Thai distributed alignment | ✅ | `layout::tests_complex::thai_distributed_spreads_full_lines_but_not_the_last`, `distributed_lines_ending_in_a_space_still_reach_the_margin` | — |
| Default font metrics (Aptos, Calibri, Cambria…) | ✅ | Carlito/Caladea/Liberation are metric-compatible for Calibri/Cambria/Arial/Times/Courier; Aptos and Aptos Display (Word's defaults since 2024) are drawn with the bundled Intos and Intos Display (OFL, github.com/muglug/intos), which have their advances, kerning and vertical metrics: measured against Microsoft's Aptos files (download package 4.40, font version 2.01), all 987 shared characters have Aptos's advance in all four styles and 387 of 387 line breaks of 40,000 characters of English prose at 39 em fall where Aptos puts them; single lines keep Aptos's 1.2847 em (`fonts::word::requested_line`; `layout::tests_complex::aptos_is_drawn_with_intos_on_aptos_widths_and_line_spacing`). Aptos Narrow and Serif use Intos Narrow and Intos Serif when installed (not bundled) | — |
| Indents in characters (`w:firstLineChars`, `w:leftChars`…), measured in the paragraph mark's font size | ✅ | #534, `layout::tests::character_indents_use_the_paragraph_mark_size`; spacing in lines (`w:beforeLines`/`w:afterLines`) uses the stored points (needs the document grid) | — |
| Line spacing (single, 1.5, double, at least, exactly, multiple), spacing before/after, contextual spacing | 🟡 | Word puts multiple-spacing extra below the text and collapses spacing differently (#135, #108 open) | 3–5 |
| Widow/orphan control, keep with next, keep lines together, page break before | ✅ | `lib.rs` keep_next/keep_lines; a line holding only a page break stays at the bottom of a full page instead of leaving a blank page (#534) | — |
| Tabs (left, center, right, decimal, bar), leaders | 🟡 | Bar tabs untested | 1–2 |
| Hyphenation (automatic, zone from the document, words in capitals, soft hyphens, consecutive limit, Manual Hyphenation, #407) | 🟡 | English patterns only; other languages hyphenate wrongly or not at all | in localization |
| Drop caps | ✅ | 2026-10-06 | — |
| Columns, column breaks, separators, balancing | ✅ | Columns balance on a section's last page when the next section starts continuous (also at the document end, which Word only balances when the document ends with a continuous break); not after next-page/even/odd breaks or on a page with a column break; keep-together, keep-with-next, widow/orphan control and unsplittable rows can leave them slightly uneven (#350) | — |
| Section breaks (next page, continuous, even, odd), different first page, odd/even headers | ✅ | Section breaks don't show in Draft view (#42) | 1–2 |
| List labels: number formats (decimal, letters, roman, ordinal text, Chinese/Japanese/Korean counting and legal, 一〇一 digits, 甲乙丙, ①…⑳, full-width, Thai digits ๑๒๓, letters ก ข ค and counting หนึ่ง สอง … สิบเอ็ด … หนึ่งร้อยเอ็ด), `%n` text such as `第%1章` / `（%1）`, legal numbering | ✅ | `doc::cjk_numbers` tests, `layout::tests::chinese_counting_list_labels_and_their_font_runs`; a label mixing faces (digit + CJK comma) is drawn run by run (#534). `layout::tests::thai_list_labels`. Hebrew, Arabic, Hindi, Vietnamese and Russian formats show decimal | — |
| Body top below a tall header, footer pushes body up | ✅ | #138; a footer taller than the bottom margin moves the body bottom up (#453), and a header or footer linked to the previous section takes its room too (#454) | — |
| Footnotes at the page bottom, endnotes at the end, note numbering | 🟡 | A long note continues onto the next page at a line boundary (its first line stays with the reference), first in that page's note area under a full-width continuation separator (#352); the document's own separator, continuation separator and continuation notice are drawn (the notice under a note that continues); a long endnote continues on the next column or page at a line boundary (#456), and the caret finds note text on its continuation page (#457); per-section restart partial | 1–3 |
| Body top below a tall header, footer pushes body up | ✅ | #138 | — |
| Footnotes at the page bottom, endnotes at the end, note numbering | 🟡 | A long note continues onto the next page at a line boundary (its first line stays with the reference), first in that page's note area under a full-width continuation separator (#352); the document's own separator, continuation separator and continuation notice are drawn (the notice under a note that continues; endnotes use the document's separator but don't continue onto a page of their own); each section's note options: footnotes below the text, endnotes at the end of each section, start at, numbering restarted each section or each page (counted from where the notes land, at most three passes) (#385) | 1–2 |
| Tables: row heights (at least/exact), rows split across pages, header rows repeat, Can't Split | ✅ | 2026-10-06, #138 | — |
| Tables: autofit to contents/window, fixed widths, Word 2013 edge | 🟡 | #137; #44: AutoFit Contents measures each column's narrowest and widest text and shares the width like Word, once (not live as you type); cell preferred widths and spans still differ | 3–5 |
| Floating tables (`w:tblpPr`) | 🟡 | #137; one placed on the page or margin that would cover the text before it and doesn't fit below it moves to the next page (#534); overlap rules untested | 2–3 |
| Nested tables | 🟡 | render; polish missing | 2–4 |
| Floating pictures/shapes placement (relative to page/margin/column/paragraph/line/character) | ✅ | #136 | — |
| Text wrap: square, top-and-bottom, behind, in front | ✅ | 2026-10-06 | — |
| Text wrap: tight, through (contour, wrap polygon) | 🟡 | #354: lines follow the wrap polygon, or the outline worked out from the object (shape geometry; a picture's opaque pixels as a convex hull; otherwise the rectangle), turned and flipped with it, kept the left/right distance from text; Through also uses gaps inside the outline. Wrap side (`wrapText`: both sides, left only, right only, largest side per line) applies to square, tight and through wraps, turned objects included; picture outlines are convex; not compared page-for-page with Word | 1–2 |
| Text boxes, overflow, linked text boxes | ✅ | #46; linked text boxes (#369): the text runs on through each box in turn at that box's line length, overflow past the last box hidden; Align Text top / middle / bottom | — |
| Rotated objects and text | 🟡 | #332: pictures, shapes, charts and groups drawn turned and flipped about their centre (screen and PDF); square wrap and inline lines keep clear of the rotated bounds; hit testing on the turned shape. Text in a rotated text box stays upright | 2–3 |
| Page borders (from page edge or text), page colour, watermark | ✅ | | — |
| Line numbers (restart per page/section, count by) | ✅ | | — |
| Vertical page alignment | ✅ | | — |
| Gutter, mirror margins, book fold, 2 pages per sheet | 🟡 | gutter and mirror margins (even pages put the inside margin and gutter on the right, #452; page-relative floats move with the text); book fold missing | 2–3 |
| Document grid (`w:docGrid`, lines per page, characters per line) | ❌ | East Asian documents paginate differently without it | 4–6 |
| Vertical text (`tbRl`) in sections and text boxes; cell text direction | ✅ | cell text direction (#245); text-box text direction (#369); vertical sections (`tbRl`/`btLr`, #491): the page's text area turned, lines top to bottom stacking right to left, ideographs, kana and Hangul upright, Latin on its side, caret and clicks through the turn. Missing: OpenType `vert` alternates (punctuation is moved, not substituted), arrow keys along vertical lines, upright pictures take turned room | 2–4 |
| Gutter, mirror margins, book fold, 2 pages per sheet | 🟡 | gutter and mirror margins; book fold missing | 2–3 |
| Document grid (`w:docGrid`, lines per page, characters per line) | 🟡 | #391: line grid snaps line heights to whole pitches (text centred in them; `w:snapToGrid` off opts a paragraph out); character grid puts East Asian characters on the character pitch (`linesAndChars`) or all text (`snapToChars`), Latin text proportional; View › Gridlines draws it. Table cells keep their natural line height unless the document sets `w:adjustLineHeightInTable` (as Word lays out the corpus's zenodo-103: 29 pages, was 36). Not yet: run-level `w:snapToGrid`, grid in headers/footers, paragraph spacing in grid lines, Word's exact glyph placement in a cell | 1–2 |
| Ruby / phonetic guide, enclose characters, combined characters | 🟡 | ruby (Phonetic Guide) landed (#288): ruby text over its base by alignment, line grows to fit, in PDF; enclose and combined characters missing; ruby not yet compared with Word page by page | 2–4 |
| Bidirectional paragraphs, mixed-direction lines; RTL tables and sections | 🟡 | #207; RTL tables (#362): grid columns from the right edge, start alignment and indent from the right margin, cell margins and left/right borders mirrored, merged and turned cells and nested tables included; RTL sections (#362): newspaper columns right to left with the line between mirrored. Right-hand page gutter (`w:rtlGutter`) missing | 1 |
| Equations: display layout, numbering, wrapping long display equations | ✅ | #191; long display equations break at top-level operators (`m:brkBin`, `m:brkBinSub`), manual breaks and `m:alnAt`, continuation lines indented by `m:wrapIndent` or set right (`m:wrapRight`) (#326) | — |
| Hidden text excluded from breaking and hyphenation | ✅ | #173 | — |
| Fields: PAGE, NUMPAGES, SECTIONPAGES, TOC page numbers | ✅ | #52; totals in the body need a second pass once the page count is known (#451) | — |
| Compatibility modes (Word 2003/2007/2010 layout for old files: `w:compat` options) | 🟡 | mode 15 behaviour; the ~60 legacy compat options are mostly ignored | 10–20 |
| Page-by-page comparison harness against Word | ❌ | Run Word locally on our own synthetic documents, export PDF, compare line breaks; Word output stays under `plan/` (never committed) | 8–12 |

## Performance

| Measure | Doc4Me | Word | Kind |
|---|---|---|---|
| Cold layout, 188-page document | 61 ms | not measured | measured 2026-10-06 (not re-run this pass) |
| Relayout after an edit | 1.4 ms | not measured | measured 2026-10-06 |
| Font picker with many installed fonts | fixed freezes (#233) | instant | issue #31, #59 |
| Large real-world documents (500+ pages, many pictures) | not measured | | gap |









**Margin leading and header wrapping (2026-11-12, Unreleased):** [The audit](readiness/margin-leading-2026-11-12.md) records the tested scope. Windows run [43333680851](https://github.com/veevie55/doc4me/actions/runs/43333680851) on `92025e8` (report `c098032`) confirms **95.1% without deferred**, **188/220 page counts and 108/220 boundaries**, from 184/102. All 220 open, round-trip and remain stable; no newly failing document in five sets. Linux is separately 189/220 pages and 103/220 boundaries. Six regressions and full CI 6/6 (2,505 native) cover the source changes. Table picture metrics, explicit spacing and grid metrics retain their prior rules; partial-width header reflow of a continued paragraph remains open. DOCX/JSON formats, assets and version are unchanged.

The [justification review](readiness/justification-review-2026-11-13.md) records four rejected Mac-measured spacing trials. None of that code or its experimental test was retained; the published layout source remains `92025e8`. Native Windows probes are needed before changing those limits.

**Body contextual spacing (2026-11-13, Unreleased):** [The audit](readiness/contextual-spacing-2026-11-13.md) records the ECMA-376 merged-gap correction, additive-spacing controls and three regressions. Full CI 6/6, native 2,508; Linux remains 189/220 pages and 103/220 boundaries, with all five failure sets identical. The broader box-spacing trial was withdrawn after corpus regressions, so table/header spacing retains its published behaviour. Latest confirmed Windows readiness is still 95.1% without deferred on `92025e8`; this source awaits measurement.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-11-14 | minor | Table/clipboard audit: 53 commands, 32 engine regressions plus raster and UI checks; CI 6/6, 2,543 native passed; 2,552 test attributes, 237,580 Rust lines, 730 commands, ribbon 504/504. Latest measured Windows source 6127a73: 95.1% without deferred, 188/220 pages and 108/220 boundaries; table source awaits measurement. |
| 2026-11-13 | minor | Body contextual spacing follows ECMA-376 17.3.1.9: max(10,12)−10 leaves 2 pt, with additive-spacing and four-paragraph controls. Three regressions; CI 6/6, native 2,508; Linux corpus unchanged in all five sets. Full box-spacing trial withdrawn after govuk-020/zenodo-067 regressions; latest Windows source `92025e8` is still 95.1% without deferred pending a new measurement. |
| 2026-11-13 | minor | Windows run 43333680851 measured margin-leading/picture/header/blank-line Keep with next source `92025e8` (report `c098032`): 95.1% without deferred; pages 184→188/220, boundaries 102→108/220, no newly failing document in five sets. Recheck full CI (2,505 native); withdraw all four Mac justification experiments after zenodo-165 regressed; retain fixed criteria and document the mirrored-restart/Arabic-fallback investigation. |
| 2026-11-13 | minor | M14/G4 margin leading: allow trailing multiple-spacing leading in the page margin; font-leading for pictures outside cells and grids; header contours wrap body starts and clear full-width continuations; Keep with next fits a final plain blank mark. Six new regressions, CI 6/6 / native 2,505; Linux pages 184→189/220, boundaries 97→103/220, no newly failing documents. Latest source awaits Windows measurement; criteria and deferred/excluded unchanged. |
| 2026-11-12 | minor | Port full-stop/open-parenthesis line breaking and pageBreakBefore at a continuous section’s top mid-page from accessible storytold/wordcraft. Two regressions; CI all 6 steps pass, native 2,499; Linux corpus unchanged with no regressions. Superscript half-point rounding was withdrawn after zenodo-093 regressed; no criteria or excluded/deferred changes. |
| 2026-11-12 | minor | Alignment tabs do not add text descent to a pictures-only line, matching the existing image-only baseline rule. zenodo-156 now matches 2 pages and all boundaries; Linux corpus pages 183→184/220, boundaries 96→97/220, no regressions. One new regression retains descent for actual text; CI all six steps pass, native 2,497. Windows remeasurement pending; fixed criteria/excluded/deferred unchanged. |
| 2026-11-12 | minor | Suppress contextual space after a paragraph before an in-flow table starting with the same paragraph style, including nested tables. Linux govuk-034 now matches 14 pages and all boundaries: corpus pages 182→183/220, boundaries 95→96/220, no regressions; two new regressions, CI all six steps pass (2,496 native tests). Windows run 43194683237 confirmed the earlier Unicode fix (94.6% without 22 deferred); table-spacing change awaits Windows measurement. |
| 2026-11-12 | minor | Windows run 43190908818 measured `c030b15`, report `383646e`: 92.7% / 94.6% without 22 deferred, pages 182/220 and boundaries 100/220. Fix Unicode sans names selecting monospace; CI all 6 steps passed (2,494 native tests); Linux 220-document results unchanged. Audit reference metadata without changing criteria or excluded/deferred rows. Font change awaits Windows measurement. |
| 2026-11-12 | minor | Preserve floating drawing stacking order across DOCX/JSON, paint and Arrange commands; handle standalone breaks before empty continuous-section ends; readiness criteria and excluded/deferred rows unchanged; new Windows measurement pending |
| 2026-11-12 | minor | แก้วัตถุที่อิงตำแหน่งย่อหน้าและอยู่หน้าหรือหลังข้อความให้ตามตัวแบ่งหน้า/คอลัมน์ก่อนจุดยึด; ผล Windows ล่าสุดยังเป็น run 43051188273 (92.7% / 94.5% ไม่นับ 22 deferred), รอวัดโค้ดรอบนี้ |
| 2026-11-10 | minor | Windows run 42836913553 measured `1352238`; report commit `8fc9173`: 92.2% / 94.1% without 22 deferred, page counts 174/222, boundaries 95/222 and languages 95/95 (22 AI-drafted catalogs added, awaiting native review). Release 1.1.3. |
| 2026-11-11 | minor | Windows run 42961322752 measured `9abf2d6`; report commit `1df7aae`: 92.2% / 94.1% without 22 deferred, page counts 175/222 (zenodo-103 now matches Word: table text no longer snaps to the document grid unless `w:adjustLineHeightInTable` is set), boundaries 95/222 and languages 95/95. v1.1.3 was deleted and released again from the latest code, adding the nested-table depth cap (18), control/MCP refusal to discard unsaved work, shared undo style sheets, the AccessKit focus guard, the Font spacing ▲▼ arrows and an About window that names the developer instead of linking the source. Headline numbers refreshed (503/503 catalog, 729 commands, 2,478 tests, 233,264 lines of Rust; 3,034 keys per catalog). |
| 2026-11-11 | minor | Windows run 43051188273 measured `bd2d2d6`; report commit `5292abd`: 92.7% / 94.5% without 22 deferred on 220 documents (zenodo-104 and govuk-003 excluded by owner decision: their reference PDFs aren't of their .docx); page counts 181/220, boundaries 99/220, languages 95/95. Changes since run 42961322752: theme fonts, Helvetica/Times substitution, bundled metric-compatible fonts (Carlito, Caladea, Arimo, Tinos, Cousine; OFL 1.1) behind the platform's own fallbacks, mode-14 page-break mark, page numbering from 0, continuous sections below floats, digit-hyphen line breaks, grid in tables, Font spacing ▲▼ at 0.1 pt. v1.1.3 was deleted and released again from this code. |
| 2026-11-07 | minor | Windows run 42334133340 measured `1868cb4`; report commit `ed787f9` confirms 91.3% / 93.2% without 22 deferred, page counts 174/222, boundaries 96/222 and languages 72/95, unchanged from the preceding official report. |
| 2026-11-07 | minor | On Linux, a section-marker candidate was rejected after comparing all 15 corpus documents whose empty markers would change: page-count passes moved 10/15 → 11/15, but boundary passes fell 5/15 → 4/15. zenodo-053 improved from 59 to Word's 58 pages; zenodo-010 moved its body another 12.65 pt away from the reference and changed its first extracted word from Guuboo to GAWP. The candidate and its unproven global regressions were reverted. Investigate column balancing before changing marker height; official Windows pagination remains 174/222 and 96/222. |
| 2026-11-07 | minor | Latest official Windows run 42274814600 on `ed4284d` matches 174/222 page counts and 96/222 page boundaries; each is one below the preceding report. Font diagnostics sampled substitutions but did not identify a cause or safe broad fix. |
| 2026-11-06 | minor | Previous official Windows run 42211564189 matched 175/222 page counts and 97/222 page boundaries. DOCX `w:altName` font substitution passed all six CI steps before the next corpus run. |
| 2026-11-01 | minor | Avoid a blank page for a leading page break in a table row; corpus rises to 162/222 page counts while page-boundary checks remain 87/222; govuk-003 page count matches but page boundaries still differ |
| 2026-11-01 | minor | Preserve omitted row grid columns through DOCX read/write and layout; govuk-009 passes; corpus pagination rises to 161/222 page counts and 87/222 page-break checks |
| 2026-11-01 | minor | Treat Word's zero-size `w:framePr` marker as page flow, not a floating frame; regression test covers pagination; zenodo-030 improves from 16 to 34 pages (Word: 32), with aggregate checks unchanged at 160/222 and 86/222 |
| 2026-11-01 | minor | Recorded complete-corpus pagination measurement: 160/222 page counts and 86/222 page-break checks pass; remaining fidelity work is measured, not an unstarted harness |
| 2026-10-25 | trivial | Default font metrics row: Aptos and Aptos Display drawn with the bundled Intos and Intos Display (measured: same advances, 387/387 line breaks) |
| 2026-10-25 | trivial | TH SarabunIT๙ row removed from the metric-matched fonts: drawn with Sarabun at its own size, announced on open |
| 2026-10-15 | minor | Single line spacing from Word for Windows' metrics (usWin + external leading; Sarabun 1.853 em); kerning only with `w:kern`, ligatures only with `w14:ligatures`; TH Sarabun New/PSK/IT๙ drawn with Sarabun at their own height, widths, kerning and line spacing; Thai counting list labels; Thai Distributed tested |
| 2026-10-14 | trivial | Thai, Lao and Khmer wrap between dictionary words (ICU4X); Thai digits and letters in list labels; Sarabun is the default font |
| 2026-10-11 | trivial | Chinese documents (#534): indents in characters, a page-anchored floating table after a full page moves to the next page, no blank page from a page break at the bottom of a full page |
| 2026-10-11 | trivial | East Asian list number formats (一、二、三, 第一章, ①); list labels whose characters come from two fonts draw each part with its own font (the digits of "1、" no longer show as other letters); a list level without indents leaves the label at the paragraph edge (#534) |
| 2026-10-11 | trivial | Vertical sections: `w:textDirection` in `w:sectPr`, upright East Asian glyphs (#491) |
| 2026-10-11 | trivial | Mirror indents (`w:mirrorIndents`): left/right indents swap on even pages; suppress line numbers inherited from styles (#512) |
| 2026-10-11 | trivial | NUMPAGES/SECTIONPAGES in the body show the real totals (the body is laid out again once the page count is known, at most three extra passes, #451); footnotes referenced in table cells go to the page bottom (#450); PDF links to a bookmark go to the page its start is on, not its paragraph's first page (#455) |
| 2026-10-11 | minor | Complex scripts: script itemisation, OpenType fallback fonts, syllable clusters, Myanmar syllable breaks (#604) |
| 2026-10-11 | trivial | Mirrored margins on even pages, tall and linked footers/headers push the body, long endnotes continue, caret in continued notes (#452, #453, #454, #456, #457) |
| 2026-10-11 | trivial | Hyphenation zone, words in capitals and consecutive-hyphen limit follow the document settings; Manual Hyphenation (#407) |
| 2026-10-11 | trivial | Document grid: line and character pitch (#391) |
| 2026-10-11 | trivial | Linked text boxes, Align Text and text-box text direction (#369) |
| 2026-10-11 | trivial | Right-to-left tables and sections laid out mirrored (#362) |
| 2026-10-10 | trivial | The document's own note separators and continuation notice are drawn (#352) |
| 2026-10-11 | trivial | Newspaper columns balance before a continuous section break (#350) |
| 2026-10-11 | trivial | Group and Ungroup keep turned members' orientation and placement (#441, #443); Size resizes a group (#442); the eraser hits turned ink where it is drawn (#445) |
| 2026-10-10 | trivial | Long footnotes continue onto the next page with a continuation separator (#352) |
| 2026-10-11 | trivial | `EQ \o` fields overstrike their arguments; enclosed characters draw their circle, square, triangle or diamond as vectors (#297) |
| 2026-10-11 | trivial | Ruby (Phonetic Guide) layout landed (#288) |
| 2026-10-10 | trivial | Wrap side: text on both sides, left only, right only or the largest side of an object (#354) |
| 2026-10-10 | trivial | Tight and Through wrapping follow the object's outline (#354) |
| 2026-10-10 | trivial | Rotated and flipped objects laid out and drawn (#332) |
| 2026-10-11 | trivial | Footnotes below text, endnotes at the end of each section, note numbering restarted per section or page (#385) |
| 2026-10-10 | trivial | Long display equations wrap across lines (#326) |
| 2026-10-10 | trivial | AutoFit Contents measures the text (#44) |
| 2026-10-10 | major | First version: layout and pagination checklist with evidence from `crates/layout`, PRs and issues |
