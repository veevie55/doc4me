# Where Doc4Me falls short of Word

> **Last reviewed:** 2026-11-12 · **Last updated:** 2026-11-12 · **Change:** minor (preserve DOCX floating-drawing stacking order and a standalone page break before an empty continuous-section end; Windows readiness remains 92.7% / 94.5% without 22 deferred, pending a new measurement) · **Target:** Microsoft Word (Microsoft 365) for Windows

The work list. One entry per shortfall, ranked by how much it stops a Word user from switching
(**B** = blocks beta). Each says what's missing, the evidence, who it hurts, an Opus 5.5 agent
wall-clock estimate, and which parity document tracks it. Agents: pick from the top, check open PRs
first (`gh pr list`), and update this file and the parity doc when a gap closes.

Native save reliability: synthetic cases on macOS reproduced loss through the predictable temporary-file name. Saves now use owned, exclusively created siblings, synchronize before replacement and clean up on failure; originals survive pre-replacement errors. This narrows save-path reliability under gap #3; the listed field failures and broader stability work remain open. See [save-path guarantees](file-format-parity.md#native-save-path-guarantees).

**ความคืบหน้า gap #4 (2026-11-12):** แก้การวางวัตถุที่อิงย่อหน้าและไม่บังคับให้ข้อความหลบ เมื่อจุดยึดอยู่หลังตัวแบ่งหน้า/คอลัมน์ (`govuk-001`, `zenodo-097`); ยังไม่ปิด gap และยังไม่เปลี่ยนคะแนนทางการจนกว่าจะวัดบน Windows

งานต่อในวันเดียวกัน: รักษา `wp:relativeHeight` ตลอดการอ่าน/บันทึกและการวาด รวมข้อความของกล่องกับชั้นของรูปร่าง; Bring Forward/Send Backward จัดลำดับในหน้า/story เดียวกันและคง wrapping เดิม แก้ตัวแบ่งหน้าล้วนก่อนย่อหน้าว่างปิดส่วนที่ส่วนถัดไปเป็นแบบ Continuous; ผล Linux 220 ฉบับเพิ่มจำนวนหน้าที่ตรงจาก 181 เป็น 182 (`govuk-037`: 78 → 79 หน้า) ขอบเขตหน้า 95/220 คงเดิม เปิด/round-trip/เสถียร 220/220 ทั้งสองรอบ ไม่พบการถดถอย; ยังต้องแก้ pagination ที่เหลือและยืนยันบน Windows ก่อนปิดรายการ

## Ranked

| # | Gap | B | Evidence | Impact | Hours | Doc |
|---|---|---|---|---|---|---|
| 1 | **Real-world DOCX fidelity remains incomplete.** The 220-document Word-reference corpus is opened, round-tripped and compared for page count/page boundaries; all documents preserve measured structure, but formatting still is not compared directly with Word | B | Latest official Windows run 43051188273: 220/220 open, remain stable and preserve measured structure; 181/220 match Word's page count and 99/220 match every page's first/last word. Earlier fixes include zero-size frame markers, row grid omissions and leading table-row breaks; corpus limits are in [file-format-parity.md](file-format-parity.md) | Everyone exchanging complex documents; pagination-sensitive forms and reports | 30–50 | [file-format-parity.md](file-format-parity.md) |
| 2 | **Word ink is dropped on open and lost on save, silently**; OLE objects show only their picture | B | No Word ink (`w14:contentPart`) handling in `crates/docx`; drawing canvases (`wpc`) and group shapes (`wpg`) are read and written (#267, #344). Charts and SmartArt are drawn (#292) and, with OLE objects, written back with all their parts on save (#319) | Documents with pen notes lose content without warning | 2–4 to preserve ink; 45–70 to render and edit charts and SmartArt (#13) | [file-format-parity.md](file-format-parity.md) |
| 3 | **Crashes and freezes reported in the field**: startup crash on Intel UHD Graphics (#170) (lag on GPUs without a DirectX 12 driver fixed, #316), freezes (#77, #59 Thai fonts, #31 font selector), no writes without admin on Windows 11 (#218), broken shape tool (#142), images can't be moved (#82); these reports still need reproduction against current code | B | The last available issue inventory recorded 62 open reports, about 15 involving crashes, hangs or data-path failures. GitHub issue tracking is currently disabled, so this is not a live issue count | Anyone hit loses work or trust; the never-crash standard covers panics, not hangs or GPU driver faults | 20–30 | [target-app-parity.md](target-app-parity.md) (Stability) |
| 4 | **Pagination differs from Word**: the page-by-page comparison harness is in `cargo xtask readiness`; legacy compatibility and font fallback gaps remain (Aptos and Aptos Display are drawn with the bundled metric-compatible Intos and Intos Display since 2026-10-25: same advances, kerning and line spacing) | B | Latest official Windows run 43051188273 passes 181/220 page-count and 99/220 page-boundary checks ([readiness.md](readiness.md)); the DOCX `w:altName` change has focused tests, but aggregate results (boundaries 95/222 in run 42961322752 → 99/220 now) do not establish font-substitution causality. Windows diagnostics captured several substitutions in nine sampled documents; MS Gothic did not appear in those results. No safe broad pagination fix emerged, so investigate failures per document before changing layout or fallback rules; [layout-parity.md](layout-parity.md); [typing-parity.md](typing-parity.md) Known gaps | Page counts and line breaks differ, which matters for forms, legal and academic work | 30–50 | [layout-parity.md](layout-parity.md) |
| 5 | **Content controls** landed (#381): `w:sdt` kept, shown and edited (Developer tab, Design Mode, Properties). Move tracking landed (#400): `w:moveFrom`/`w:moveTo` with their named ranges round-trip, a tracked cut and paste of a paragraph or sentence records a move, either end accepts/rejects both, green double strike/underline markup and "Moved from/to" balloons; still missing: drag-and-drop moves (no text drag yet) and move options in Track Changes Options. Formatting revisions landed (#41): `w:rPrChange`/`w:pPrChange`/`w:tblPrChange`/`w:trPrChange`/`w:tcPrChange`/`w:sectPrChange`/`w:numberingChange` round-trip, are recorded while tracking, accepted/rejected and shown in balloons; still missing: tracked style-definition changes and table/section changes made in Doc4Me. Opaque custom XML backing stores now survive saves; binding evaluation remains absent. | B | Issues #41, #400; source grep | Legal and editorial review workflows; templates with form controls | 5–8 | [file-format-parity.md](file-format-parity.md) |
| 6 | **No native printing** on desktop: File › Print opens the PDF in the system viewer, which prints (#286); File › Print has a preview, page range, odd/even, copies and collate (#641); printer choice, duplex and pages per sheet are still the system's; web prints through the browser (#209) | B | `ui.print` → `apps/doc4me/src/print.rs`; issue #15 | Everyone who prints | 5–8 | [hardware-parity.md](hardware-parity.md) |
| 7 | **Dialog depth**: 60 modal dialogs vs Word's ~100 | B | `Dialog` enum in `crates/ui-egui/src/dialogs.rs` | Power users reach for dialogs the ribbon alone doesn't cover | 11–19 | [ui-parity.md](ui-parity.md) |
| 8 | **Proofing covers English, Polish and Thai only** (Thai: spelling only, no grammar); no dictionaries, grammar or hyphenation for the other ~47 languages Word proofs (text in them is left unmarked) | | `crates/proof`: English and Polish, a language per run (2026-10-10); Thai spelling with 105 common misspellings (`crates/proof/src/th`, Doc4Me 1.1.0); Word ships 120 proofing tools; issues #25, #40, #100 | Every writer outside English, Polish and Thai | 20–40 | [localization-parity.md](localization-parity.md) |
| 9 | **Password-protected `.doc` files** can't be opened | | Encrypted .docx (agile encryption) opens and saves (`crates/docx/src/crypt.rs`); `docbin` detects and refuses RC4/XOR `.doc` (#55) | Corporate and legal users | 6–10 | [file-format-parity.md](file-format-parity.md) |
| 10 | **Objects**: text in a freely rotated text box (free rotation and flips landed, #332; text direction, alignment and linked text boxes landed, #369; contour wrap and wrap side landed, #354), shape effects, WordArt 3-D effects and connector routing around shapes (WordArt effects and warps and glued connectors landed, #416), dragging members inside a Drawing Canvas (the canvas landed, #344) | | [parity-checklist.md](parity-checklist.md) (Shape Format 6/6; Layout › Group landed, `arrange.group`) | Newsletters, flyers, reports with diagrams | 20–30 | [target-app-parity.md](target-app-parity.md) |
| 11 | **Interface languages**: every culture row now has a registered catalog (Windows run 43051188273: 95/95 catalog-key coverage), but 22 of them, Hindi, Arabic and Korean among the twelve key languages, are AI drafts without native-speaker review. Bundled Noto fallbacks cover their glyphs, but epaint still lacks shaping and bidi and there is no mirrored RTL interface, so the 19 RTL/Indic languages warn before use. The check does not measure whether values translate the English source or assess accuracy | | [localization-parity.md](localization-parity.md) | Most of the world's writers | 50–80 | [localization-parity.md](localization-parity.md) |
| 12 | **Draw tab / ink**: pens, pencil, highlighter, eraser, Select and Hide Ink landed (#307); Lasso Select, Add Pen, Ink to Shape and Ink Replay landed (#395); Ink to Math and Ink Equation landed (2026-10-25, Doc4Me's own $P-based recogniser, a small symbol set); missing: arrows in Ink to Shape, pressure, Word's own ink (`w14:contentPart`/InkML), recognition as good as Microsoft's | | [parity-checklist.md](parity-checklist.md) (Draw 11/11) | Pen and tablet users | 4–8 | [hardware-parity.md](hardware-parity.md) |
| 13 | **Charts and SmartArt editing** (editing charts and SmartArt from Word, more chart types and formatting) | | `insert.chart` and Chart Design basics landed (#409): Doc4Me's own charts are editable; charts from Word stay read-only (no embedded-workbook editing), no combo/3D/radar/stock charts, axis or series formatting; Insert › SmartArt landed (#497) for graphics Doc4Me makes | Reports and business documents | 35–55 | [target-app-parity.md](target-app-parity.md) |
| 14 | **`.doc` can't be written; Word 6/95 files refused** | | `crates/docbin` is read-only | Users exchanging files with old Word or legacy systems | 25–40 | [file-format-parity.md](file-format-parity.md) |
| 15 | **References depth**: no EndNote/Mendeley desktop integration | | Sources are read from and written to the DOCX `b:Sources` custom XML part (`crates/docx/src/read/mod.rs`, `write/mod.rs`) | Students and researchers | 6–10 | [target-app-parity.md](target-app-parity.md) |
| 16 | **Right-to-left completeness**: kashida, RTL in HTML/ODT/RTF; eight older RTL bug reports to re-verify after #207 (#215, #211, #199, #66, #63, #48, #19). RTL tables and sections landed (#362) | | [layout-parity.md](layout-parity.md) | Arabic, Persian, Hebrew writers | 7–11 | [layout-parity.md](layout-parity.md) |
| 17 | **East Asian typography**: Asian Typography options (Phonetic Guide landed, #288; Enclose Characters landed, #297; document grid landed, #391; vertical text landed, #491) | | `para.asianTypography` is in the catalog (`crates/engine/src/catalog.rs`); depth against Word's Asian layout options unmeasured | Chinese, Japanese and Korean documents | 11–19 | [layout-parity.md](layout-parity.md) |
| 18 | **Screen-reader access** to the document canvas: exposed, not yet tried with real screen readers | | The canvas describes paragraphs, headings, lists, tables, links, pictures and the caret to AccessKit (#488), tested headlessly; no NVDA, VoiceOver or Orca run yet | Blind and low-vision users; public-sector procurement | 3–6 | [ui-parity.md](ui-parity.md) |
| 19 | **RTF and ODT depth**: notes, comments, revisions, sections, RTL | | ~1,500 lines each in `crates/formats` | Users exchanging with LibreOffice or older tools | 18–27 | [file-format-parity.md](file-format-parity.md) |
| 20 | **Performance on large real-world documents** unmeasured (500+ pages, many pictures) | | Only the 188-page sample benchmark | Thesis and book authors | 10–20 | [layout-parity.md](layout-parity.md) |
| 21 | **Mail merge data sources**: no Outlook/Contacts, Access or database sources, no email merge; CSV, typed lists (#247) and Excel/OpenDocument spreadsheets (#334) only | | Select Recipients reads .csv/.tsv, .xlsx/.xlsm and .ods | Office administrators | 4–8 | [target-app-parity.md](target-app-parity.md) |
| 22 | **Ecosystem**: Microsoft's own systems are out of scope (owner decision, 2026-10-25): VBA execution, Office (Office.js) add-ins and the Office Store, OneDrive/SharePoint storage and Microsoft's real-time co-authoring. Open replacements in place: Doc4Me macros (record and play commands; VBA in a `.docm` is kept in the file, never run), Doc4Me's own add-in catalog (Get Add-ins), and cloud storage through a folder that Nextcloud Desktop or Syncthing keeps in sync | | `tools.macros`, `tools.recordMacro`; `addins.get` with `addins/catalog.json`; `.docm` keeps macros (#172) | Enterprise users with macro templates; teams | — | [target-app-parity.md](target-app-parity.md) |
| 23 | **PDF Reflow** (open a PDF as an editable document, Word for Windows) | | | Users who edit PDFs they receive | 30–50 | [file-format-parity.md](file-format-parity.md) |
| 24 | **Default shortcuts**: latest official Windows check passes 276/280 rows; all 276 non-deferred rows pass, while four OS-owned Alt-numpad/IME rows remain deferred. The engine registry currently exposes 187 commands with shortcut strings and 215 declared chord alternatives; this differs from the readiness checklist (#330, #339, #343; Customize Keyboard #368, Customize Ribbon and the Quick Access Toolbar #379 landed) | | [ui-parity.md](ui-parity.md) | Power users | 0 for non-deferred rows; owner/platform review for the deferred rows | [ui-parity.md](ui-parity.md) |
| 25 | ~~In-app AI assistant~~: cut (owner decision, 2026-10-25); Doc4Me has no AI provider in the program. External agents drive it through MCP and the CLI ([mcp.md](mcp.md)) | | | | — | [target-app-parity.md](target-app-parity.md) |
| 26 | **Minor formats**: MHT, Works/WordPerfect, saving Word 2003 XML | | Word XML Document (Flat OPC) opens and saves, Word 2003 XML opens (#548) | Rare | 15–28 | [file-format-parity.md](file-format-parity.md) |
| 27 | **View windows** (landed 2026-10-25: New Window, Split, Arrange All, Side by Side, Synchronous Scrolling, Switch Windows): a new window shows the document without its own ribbon, and all windows share one selection | | `crates/ui-egui/src/windows.rs`; [parity-checklist.md](parity-checklist.md) (View 29/29) | Comparing documents | 1–2 | [ui-parity.md](ui-parity.md) |
| 28 | **Dictate**: works (2026-10-25) with whisper.cpp on the computer; text comes after each pause, no voice commands | | `tools.dictate`, `apps/doc4me/src/dictate.rs` | Dictation users | 4–8 (voice commands, streaming) | [hardware-parity.md](hardware-parity.md) |
| 29 | **Equations**: handwritten equations are recognised from a small symbol set (Insert › Equation › Ink Equation and Draw › Ink to Math landed 2026-10-25, `crates/engine/src/ink_math.rs`: digits, `a b c n x y z`, `+ − = < > / ( )`, `π`, `∫`, `√`): no matrices, other Greek letters or letters beyond those by hand | | #191 notes; long display equations wrap at operators (#326) | Maths-heavy documents | 2–3 | [layout-parity.md](layout-parity.md) |
| 30 | **Thai depth**: no Thai grammar checking; line breaks (ICU4X dictionary segmentation) may differ from Word's; TH Sarabun's optional ligatures (fi, ฤๅ) are not simulated | | `crates/proof/src/th` checks spelling only; Thai fonts without measured metrics are drawn with Sarabun at its own size; a document in TH SarabunIT๙ says so when it opens (`cmd::file::unmatched_fonts`) | Thai government and academic writers | 10–20 | [localization-parity.md](localization-parity.md) |

**Beta needs #1–#7** (~150–230 h with the stability and layout work they imply): they are the
blocking gaps, and closing them lifts layout, stability and UI, where the measured full number
(92.7% in the latest completed official report, [readiness.md](readiness.md)) loses most of its points. The report includes the ARM64 pass (`plat-002`, verified in run 42262956358) and records 22 deferred rows. Pagination remains the largest measured gap (catalog-key coverage is 95/95); language key presence is not translation quality. Everything else is depth on the way to full parity.

## By kind

The same gaps grouped the way the parity documents are, for agents working in one area.

- **Feature gaps:** #2 (preserve), #5, #10, #12, #13, #15, #17, #21, #27, #28, #29 (Clipboard
  pane, Manage Styles and Group landed: `edit.clipboardPane`, `styles.manage`, `arrange.group`). Paste Special, Advanced Find, Column Selection, Style Inspector, custom table
  styles, and Draw Table and Eraser (#303) landed on 2026-10-10.
- **UI/UX gaps:** #7, #18, #24; crop handles on the canvas (rotation handle landed, #332); table column markers on the
  ruler (#49; dragging borders and the resize handle on the page landed, #217); context menus for objects and tables; Linux title bar theming (#78); File ›
  Options › Animation effects › Use system setting reads macOS (Reduce motion) and the Linux/BSD
  desktops (`apps/doc4me/src/animations.rs`) but hasn't been tried on a Mac or in a real
  GNOME or KDE session yet.
- **File-format gaps:** #1, #2, #5, #9, #14, #19, #23, #26; embedded fonts; glossary/building blocks.
- **Hardware gaps:** #6, #12, #28; scanner/Continuity Camera insert; KDE soft text (#140).
- **Localization gaps:** #8, #11, #16, #17, #30; native-speaker review of every catalog (human).
- **Stability and performance gaps:** #3, #20.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-11-12 | minor | Preserve floating drawing stacking order across DOCX/JSON, paint and Arrange commands; handle standalone breaks before empty continuous-section ends; readiness criteria and excluded/deferred rows unchanged; new Windows measurement pending |
| 2026-11-12 | minor | แก้วัตถุที่อิงตำแหน่งย่อหน้าและอยู่หน้าหรือหลังข้อความให้ตามตัวแบ่งหน้า/คอลัมน์ก่อนจุดยึด; ผล Windows ล่าสุดยังเป็น run 43051188273 (92.7% / 94.5% ไม่นับ 22 deferred), รอวัดโค้ดรอบนี้ |
| 2026-11-10 | minor | Windows run 42836913553 measured `1352238`; report commit `8fc9173`: 92.2% / 94.1% without 22 deferred, page counts 174/222, boundaries 95/222 and languages 95/95 (22 AI-drafted catalogs added, awaiting native review). Release 1.1.3. |
| 2026-11-11 | minor | Windows run 42961322752 measured `9abf2d6`; report commit `1df7aae`: 92.2% / 94.1% without 22 deferred, page counts 175/222 (zenodo-103 now matches Word: table text no longer snaps to the document grid unless `w:adjustLineHeightInTable` is set), boundaries 95/222 and languages 95/95. v1.1.3 was deleted and released again from the latest code, adding the nested-table depth cap (18), control/MCP refusal to discard unsaved work, shared undo style sheets, the AccessKit focus guard, the Font spacing ▲▼ arrows and an About window that names the developer instead of linking the source. Headline numbers refreshed (503/503 catalog, 729 commands, 2,478 tests, 233,264 lines of Rust; 3,034 keys per catalog). |
| 2026-11-11 | minor | Windows run 43051188273 measured `bd2d2d6`; report commit `5292abd`: 92.7% / 94.5% without 22 deferred on 220 documents (zenodo-104 and govuk-003 excluded by owner decision: their reference PDFs aren't of their .docx); page counts 181/220, boundaries 99/220, languages 95/95. Changes since run 42961322752: theme fonts, Helvetica/Times substitution, bundled metric-compatible fonts (Carlito, Caladea, Arimo, Tinos, Cousine; OFL 1.1) behind the platform's own fallbacks, mode-14 page-break mark, page numbering from 0, continuous sections below floats, digit-hyphen line breaks, grid in tables, Font spacing ▲▼ at 0.1 pt. v1.1.3 was deleted and released again from this code. |
| 2026-11-07 | minor | Windows run 42334133340 measured `1868cb4`; report commit `ed787f9` confirms 91.3% / 93.2% without 22 deferred, page counts 174/222, boundaries 96/222 and languages 72/95, unchanged from the preceding official report. |
| 2026-11-07 | minor | Latest official Windows run 42274814600 on `ed4284d`: readiness 91.3% / 93.2% without 22 deferred; page counts 174/222, boundaries 96/222, languages 72/95, dialogs 216/228, shortcuts 276/280 and platforms 5/5. No safe broad pagination fix emerged; diagnostics sampled substitutions without showing MS Gothic or identifying a cause. |
| 2026-11-06 | minor | Previous official Windows run 42211564189: readiness 90.5% / 93.1% without 23 deferred; page counts 175/222, boundaries 97/222, languages 68/95, dialogs 210/222, shortcuts 276/280. At that time the ARM64 report refresh was pending; run 42274814600 later measured the `plat-002` pass. |
| 2026-11-01 | minor | Map classic New Toolbar, Customize Menu Bar and Customize Menus to the tested Customize Ribbon replacement; dialogs 187/228, readiness 86.0% (88.5% without deferred) |
| 2026-11-01 | minor | Map legacy `wdDialogToolsAdvancedSettings` to the existing, tested File › Options › Advanced pane; dialogs 184/228, readiness 85.8% (88.3% without deferred) |
| 2026-11-01 | minor | Verify Syncthing two-peer DOCX save-in-place and sync/reopen; ecosystem 5/6, readiness 85.7% (88.2% without deferred) |
| 2026-11-01 | minor | Leading page break no longer creates an empty page in table rows; page counts improve to 162/222, page-boundary checks remain 87/222; readiness 85.4% |
| 2026-11-01 | minor | `w:gridBefore`/`w:gridAfter` support fixes govuk-009 pagination; complete corpus improves to 161/222 page counts and 87/222 page breaks; readiness 85.4% (87.8% without deferred). |
| 2026-11-01 | minor | Zero-size Word frame markers no longer collapse following paragraphs into one float; zenodo-030 improves 16→34 pages (Word: 32), but aggregate page-count/page-break checks remain 160/222 and 86/222. Exclude Macintosh-only dlg-040, raising measured readiness to 85.3% (87.7% without deferred). Syncthing peers connected but cluster config was rejected; eco-003 remains untested. |
| 2026-11-01 | minor | Readiness 85.2% on the complete corpus; DOCX round-trip 222/222; pagination harness confirmed, 160/222 page counts and 86/222 page-break checks pass |
| 2026-10-25 | trivial | Beta note: measured readiness is 73.4% now |
| 2026-10-25 | trivial | Beta note: the full number is measured (66.1%) instead of estimated (~60% → ~75%) |
| 2026-10-25 | trivial | #4: Aptos and Aptos Display drawn with the bundled Intos and Intos Display; owner-bound font item removed |
| 2026-10-25 | trivial | #22: VBA, Office add-ins, OneDrive/SharePoint and co-authoring out of scope (owner); open replacements listed. #25 in-app AI provider cut (owner) |
| 2026-10-25 | trivial | #30: TH SarabunIT๙ no longer uses inferred metrics; Sarabun draws it and opening the document says so; the owner-bound font-file item dropped |
| 2026-10-25 | trivial | #12 Ink to Math / Ink Equation, #22 Get Add-ins (Doc4Me's own add-in catalog), #28 Dictate (whisper.cpp) landed; #29 narrowed to the recogniser's symbol set; catalog parity 499/499 |
| 2026-10-11 | trivial | #8 narrowed: runs in a proofing language without a dictionary are skipped instead of flagged against the English dictionary (#100); dictionaries still English-only |
| 2026-10-11 | trivial | #16: DOCX open/save preserves independent complex-script formatting inheritance; RTL sections, tables and typography remain open |
| 2026-10-11 | trivial | `.doc` pictures, shapes and text boxes read (#551); gap #14 narrowed to the writer and Word 6/95 files |
| 2026-10-11 | trivial | Table column and row borders and the resize handle drag on the page (#217); UI/UX gap narrowed to the ruler's column markers |
| 2026-10-11 | trivial | UI/UX gaps list: the system's animation setting is now read on macOS and the Linux/BSD desktops; trying it on real machines remains |
| 2026-10-11 | trivial | Animation effects setting landed (File › Options › Accessibility); UI/UX gaps list: the system's animation setting isn't read on macOS or Linux/BSD yet |
| 2026-10-11 | trivial | Word XML Document (Flat OPC) and Word 2003 XML opening landed (#548); gap #26 narrowed to 15–28 h |
| 2026-10-11 | trivial | #13 narrowed: Insert › SmartArt with eight layouts, Text Pane and SmartArt Design landed (#497); 25–40 h left for charts |
| 2026-10-11 | trivial | File › Options panes landed (#485); removed from the dialog-depth list |
| 2026-10-11 | trivial | #18: the document canvas describes its text, structure and caret to screen readers through AccessKit (#488); real screen-reader testing remains |
| 2026-10-11 | trivial | Compare / Combine Documents and AutoCorrect Options dialogs and the Restrict Editing pane landed (#412); removed from the dialog-depth list; duplicate #7 rows merged |
| 2026-10-11 | trivial | #10: WordArt text effects and Transform warps, connectors glued to shapes (#416) |
| 2026-10-11 | trivial | #5 narrowed: content controls kept, shown and edited (#381); move tracking remains |
| 2026-10-11 | trivial | Insert › Chart landed (#409); gap #13 narrowed to editing Word's charts, more types and SmartArt, 35–55 h |
| 2026-10-11 | trivial | Line Numbers, Hyphenation, Language, Envelopes and Labels dialogs landed (#407); gap #7 at 30 dialogs, 16–24 h; duplicated #7 rows merged |
| 2026-10-11 | trivial | Caption, Index, Mark Index Entry, Custom Table of Contents, Source Manager and Create/Edit Source dialogs landed (#402); removed from the dialog-depth list |
| 2026-10-11 | trivial | Document grid landed (#391); removed from the East Asian gap |
| 2026-10-11 | trivial | Move tracking landed (#400); gap #5 narrowed to content controls, 5–8 h |
| 2026-10-11 | trivial | #4: note options per section (below text, end of section, restart each section/page) and the Footnote and Endnote dialog (#385) |
| 2026-10-11 | trivial | #10: Shape Format › Text: text direction, align text and linked text boxes (#369) |
| 2026-10-11 | trivial | #16: right-to-left tables (`w:bidiVisual`) and sections (`w:bidi`) laid out mirrored, with `table.rtl` and `layout.sectionRtl` (#362) |
| 2026-10-11 | trivial | #4: columns balance before a continuous section break (#350) |
| 2026-10-11 | trivial | #21: Excel (.xlsx/.xlsm) and OpenDocument (.ods) spreadsheets as recipient lists (#334); hours 6–10 → 4–8 |
| 2026-10-11 | trivial | Insert › Icons landed (#309); removed from the objects gap |
| 2026-10-12 | minor | Preserve opaque custom XML data/properties through saves; gap #5 and readiness remain unchanged |
| 2026-10-11 | minor | Imported TOC boundaries preserved in synthetic checks; real-world corpus gap and readiness unchanged |
| 2026-10-11 | trivial | #24 narrowed: Customize Ribbon and Quick Access Toolbar customization landed (#379); the default shortcuts remain |
| 2026-10-11 | trivial | #24 narrowed: Customize Keyboard landed (#368); Customize Ribbon and the Quick Access Toolbar remain |
| 2026-10-11 | trivial | Native saves protect existing temporary-name entries and originals on pre-replacement failure; synthetic macOS evidence, explicit durability/platform limits; readiness unchanged |
| 2026-10-11 | trivial | #24: Word's field keys (Alt+F9, Shift+F9, Ctrl+Shift+F9, Ctrl+F11, Ctrl+Shift+F11, F11, Shift+F11) (#339) |
| 2026-10-10 | trivial | #24: 125 of ~250 keyboard shortcuts after #330 |
| 2026-10-11 | trivial | Collapsed duplicate rows left by parallel merges |
| 2026-10-11 | trivial | All 12 bibliography styles landed (#383); removed from References depth |
| 2026-10-11 | trivial | Formatting revisions landed (#41); gap #5 narrowed to move tracking and content controls, 8–12 h |
| 2026-10-11 | trivial | Omitted `w:pPrDefault` stays omitted across saves; explicit empty/default properties and later spacing edits remain writable. Synthetic preservation checks only; readiness estimates unchanged |
| 2026-10-11 | trivial | #12 narrowed: Lasso Select, Add Pen, Ink to Shape and Ink Replay landed (#395) |
| 2026-10-10 | trivial | Draw Table and Eraser landed (#303); removed from the feature-gap list |
| 2026-10-10 | trivial | Tabs and Borders and Shading dialogs landed (#320); removed from the dialog-depth list |
| 2026-10-10 | trivial | Columns, Symbol and Field dialogs landed (#321); removed from the dialog-depth list |
| 2026-10-10 | trivial | #2 narrowed: charts and SmartArt drawn (#292), and with OLE objects kept on save (#319); group shapes landed (#267); ink and drawing canvases remain |
| 2026-10-10 | trivial | #4: a document's own footnote separators and continuation notice are drawn and kept on save (#352) |
| 2026-10-10 | trivial | #4: long footnotes continue onto the next page (#352) |
| 2026-10-10 | trivial | Define New Multilevel List and Track Changes Options dialogs landed (#328); removed from the dialog-depth list |
| 2026-10-11 | trivial | Enclose Characters landed (#297); removed from the East Asian typography gap |
| 2026-10-11 | trivial | Phonetic Guide (ruby) landed (#288): gap 17 narrowed |
| 2026-10-10 | trivial | #10: wrap side (both sides, left, right, largest) for Square, Tight and Through objects (#354) |
| 2026-10-10 | trivial | #10: Tight and Through wrapping follow the object's outline (#354) |
| 2026-10-10 | trivial | #10: free rotation and flips for pictures, shapes and text boxes, with a rotation handle (#332); text boxes rotate their frame; their text stays upright |
| 2026-10-10 | trivial | #10: Drawing Canvas landed (#344); in-canvas dragging of members remains |
| 2026-10-10 | trivial | #29: long display equations wrap across lines (#326) |
| 2026-10-10 | trivial | #3: Windows lag on GPUs without a DirectX 12 driver fixed with an OpenGL fallback (#316) |
| 2026-10-10 | minor | #8: Polish proofing (spelling, grammar, hyphenation) and a proofing language per run landed; the gap is now the other languages (20–40 h) |
| 2026-10-10 | trivial | #6: desktop File › Print opens the system PDF viewer (#286) |
| 2026-10-10 | trivial | Table Properties dialog landed (#44); removed from the dialog-depth list |
| 2026-10-10 | trivial | Merged main: landed features removed from the feature-gap list |
| 2026-10-10 | minor | Beta note restated: full number back to the additive ~60% (method aligned with the standard, no new evidence) |
| 2026-10-11 | trivial | #491: vertical sections landed; vertical text removed from the East Asian gap |
| 2026-10-10 | major | First version, from the full re-measure: 29 ranked gaps with evidence, impact and hours |
| 2026-10-10 | trivial | #307: Draw tab pens, eraser and Hide Ink landed; ink gap narrowed |
