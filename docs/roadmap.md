# Doc4Me roadmap: milestones and what's next

> **Last reviewed:** 2026-11-12 · **Last updated:** 2026-11-12 · **Change:** minor (Windows run 43194683237 on `a466dff`, report `67d6323`: 92.7% / 94.6% without 22 deferred; Unicode fallback confirmed; contextual spacing before a same-style in-flow table fixes govuk-034 on Linux, pending Windows) · **Target:** Microsoft Word (Microsoft 365) for Windows

Forward-looking plan. The summary is [`ROADMAP.md`](../ROADMAP.md); the ranked work list is
[`gaps.md`](gaps.md); the numbers are in [`target-app-parity.md`](target-app-parity.md). Hours are
Opus 5.5 agent wall-clock.

## Current focus (toward beta, ~150–230 h)

งานล่าสุด 2026-11-12: รักษาลำดับซ้อนของวัตถุลอย `wp:relativeHeight` รวมข้อความในกล่องและการเลือก; Bring Forward/Send Backward ย้ายหนึ่งชั้นโดยไม่เปลี่ยน wrapping; ตัวแบ่งหน้าล้วนก่อนย่อหน้าว่างปิดส่วนที่ส่วนถัดไปเป็นแบบ Continuous ต้องวางบรรทัดให้พอก่อนแบ่งหน้า เทียบ Linux ครบ 220 ฉบับแล้ว: จำนวนหน้า 181 → 182/220 ขอบเขตหน้า 95/220 คงเดิม ไม่ถดถอย; รอ Windows ก่อนอ้างคะแนนที่เพิ่มขึ้น เป้าหมาย 100% ไม่นับ deferred ยังไม่สำเร็จ

1. **Review relevant upstream WordCraft changes before duplicating work.** Check individual patches
   against the remaining gaps (including Hebrew RTL, IME behavior and locale coverage); the upstream
   open-PR count is not maintained as a current project metric.
2. **DOCX real-world fidelity** (gap #1, 30–50 h): the 220-document corpus is measured for open,
   round-trip, stability and pagination; all documents preserve measured structure after save and
   reopen, but formatting still needs direct comparison with Word. Omitted/empty paragraph defaults
   also have synthetic save/reopen coverage.
3. **Preserve remaining unsupported content** (gap #2, 2–4 h): charts, SmartArt, OLE, groups and
   drawing canvases survive round-trip (some are read-only or shown as fallback pictures); Word ink
   (w14:contentPart) remains unsupported and is lost on open/save.
4. **Stability sweep** (gap #3, 20–30 h): the open crash, hang and write-failure issues, plus an
   autosave/recovery soak test.
5. **Pagination fidelity** (gap #4, 30–50 h): compat options (column balancing and footnote
   continuation landed), a local page-by-page comparison harness (Aptos is drawn with the bundled
   metric-compatible Intos since 2026-10-25).
6. **Formatting revisions and content controls** (gap #5, 12–18 h), **native printing** (gap #6,
   6–10 h), **dialog depth** (gap #7, 20–30 h).

## Alpha gate (core workflows)

The core-workflow gate from the craftrules progress-docs standard: what a typical Word user does
every day, checked end to end on the main platform, including save and reopen. The main platform is
now Windows desktop (Word for Windows is the target); the results below were checked on macOS
desktop in WordCraft and have not been re-run on Windows yet.
"Partial" here means rough but not blocking: the workflow completes.

| Core workflow | Works end to end? | Evidence | Hours to pass |
|---|---|---|---|
| Write and format a document: type, fonts, paragraph formatting, styles, lists, undo, find/replace; save as .docx and reopen | yes | 727 registered engine command specs (confirmed by `doc4me-cli commands --json`), typing parity pinned by `tests_typing.rs`, DOCX round-trip tests (`crates/docx/tests/roundtrip.rs`) | 0 |
| Open a .docx someone sent, edit it, send back a .docx Word opens | partial (not blocking) | Word opens our files; all 220 real-world documents open and preserve measured structure through save and reopen (formatting not compared yet). Charts, SmartArt, OLE, groups and drawing canvases are preserved; Word-created charts/SmartArt remain read-only and OLE shows its picture. Move tracking supports tracked cut/paste; drag-and-drop moves remain missing (gaps #1, #2, #5) | 0 (beta work: 50–80) |
| Build a structured document: tables, pictures, headers/footers, page numbers, sections, columns | yes | Tables with row splitting and floating tables (#137), floating pictures (#136), first/odd/even headers; columns don't balance (gap #4) | 0 |
| Review with others: comments, track changes, accept/reject, compare | yes | Margin balloons, tracked insert/delete incl. paragraph marks (#125, #244), formatting revisions (#41) and move tracking (#400); compare | 0 |
| References for reports and papers: TOC, footnotes, citations and bibliography, captions | yes | Updatable TOC field (#52), notes with continuation across pages (#352), APA/MLA/Chicago/IEEE, Zotero (#189), and bibliography-source DOCX round-trip (#532) | 0 |
| Share or print: export PDF, print | partial (not blocking) | PDF export solid (krilla); desktop printing goes through a PDF, no native print dialog (gap #6) | 0 (beta work: 6–10) |

**Result: historical macOS gate passes.** Those WordCraft workflows completed and saved/reopened on
macOS; the current Windows readiness run measures the document corpus, not every UI workflow. The
partial rows are fidelity gaps (beta work), not broken workflows. Doc4Me stays **alpha**.

## Milestones

| # | Milestone | State | Remaining |
|---|---|---|---|
| M0 | Skeleton and vertical slice (model, layout, render, engine, Word-style UI, CLI, MCP, web) | done | — |
| M1 | DOCX I/O | corpus measured: 220/220 open and preserve measured structure; direct formatting comparison remains | 30–50 h |
| M2 | Home tab | done (Paste Special, Clipboard pane, Style Inspector, Manage Styles and Asian Typography landed; Dictate with whisper.cpp on the computer, 2026-10-25) | — |
| M3 | Insert tab | done: equations (#191), icons (#309), charts, SmartArt, Screenshot › Screen Clipping, Online Video, 3D Models (as a picture), Online Pictures (Openverse), Get Add-ins (Doc4Me's own catalog) and Ink Equation landed | — |
| M4 | Layout and Design tabs | done except column balancing | 6–10 h |
| M5 | Tables | done (row splitting, floating tables, style formatting); custom table styles (#256); Draw Table and Eraser (#303) | 6–10 h |
| M6 | References | first version done (TOC fields, cross-references, Zotero); footnote continuation (#352), DOCX bibliography-source round-trip (#532) and all 12 bibliography styles (#383) landed; deeper Word compatibility remains | 15–25 h |
| M7 | Review | done (balloons, paragraph-mark revisions, formatting revisions #41 and move tracking #400); multilingual proofing remains missing | 35–60 h |
| M8 | View | done; window commands (New Window, Arrange All, Split, View Side by Side, Synchronous Scrolling, Switch Windows) landed 2026-10-25 | — |
| M9 | Mailings | done (first version; Excel and OpenDocument sheet sources supported); Outlook/Access/Contacts sources and email merge remain missing | 6–10 h |
| M10 | File/Backstage | mostly done; native printing, Options depth missing | 12–20 h |
| M11 | Draw and objects | text wrap, text boxes, canvas handles, rotation (#332), ink pens and eraser (#307), contour wrap (#354), lasso, Add Pen, Ink to Shape and Ink Replay (#395) done; grouping (`arrange.group`) and Ink to Math (own recogniser, 2026-10-25) done | 30–50 h |
| M12 | Formats breadth (PDF, ODT, RTF, HTML, MD, TXT, LaTeX, `.doc` import) | done (first versions); encrypted `.docx` opens and saves (agile encryption); `.doc` write, encrypted `.doc`, minor formats missing | 60–100 h |
| M13 | Performance budgets | on track (1.4 ms relayout); large real-world documents unmeasured | 10–20 h |
| M14 | 1.0 polish, packaging, signing | Doc4Me v1.1.3 published on the fork [`veevie55/doc4me`](https://github.com/veevie55/doc4me/releases) on 2026-11-11 (the first v1.1.3 of 2026-11-10 was deleted and released again from the latest code), its only GitHub Release (the v1.0.0–v1.1.2 releases and tags published there on 2026-11-02 no longer exist) (Windows x64 MSI and portable zip, unsigned; CI signs once the signing secrets are set); WordCraft, upstream, shipped v0.1.0–v0.4.0 on every platform | — |
| M15 | Localization (12 key languages, RTL interface, proofing languages) | Latest report: 95/95 culture rows pass (100%; 26 catalogs are AI drafts awaiting native review, and the 19 RTL/Indic languages warn before use because the interface doesn't shape or mirror them). The check does not validate translated values or quality. English, Polish and Thai proofing, a proofing language per run | 60–100 h |
| M16 | Right-to-left and East Asian typography | RTL paragraphs done (#207) | 30–45 h |

## Path to 100% measured readiness

The readiness number in [`ROADMAP.md`](../ROADMAP.md) was an estimate (~60%): a weighted sum of
estimated dimension parities. Since stage B it is measured: **66.1%** at first, **73.4%** in the
2026-10-25 report, and **92.7%** in the latest official Windows report (run 43194683237, source
`a466dff`, report commit `67d6323`; ribbon 503/503, dialogs 216/228, pages 182/220, boundaries
100/220, languages 95/95, shortcuts 276/280; all 220 documents; DOCX open and round-trip 220/220),
7.3 points short of 100% overall. That report says 94.6% without its 22 deferred rows, 5.4 points
short, all of them pagination. Current `manual.tsv` has 22 deferred rows and records the ARM64
check as pass after run 42262956358 built and verified the MSI and portable ZIP. The source TSVs
map 95/95 language rows and the dialog TSV/source maps 216/228 rows (12 deferred; 7 excluded),
matching the report. Language readiness measures catalog key-row coverage, not translated content
or accuracy. Doc4Me 1.1.3 is released from this state.
The target is **100% ready for real work, measured**
(raised from 80% to 95% by the owner on 2026-10-25, and to 100% on 2026-10-27): every dimension
comes from a fixed pass/fail check set, and one command computes the number. Checks may be added,
never removed, so the number can't drift by editing the list.

**A. Build the measurement first** (done 2026-10-25: `cargo xtask readiness`, check sets in
[`readiness/`](readiness/README.md), 222 corpus documents; result in
[`readiness.md`](readiness.md)):

1. `cargo xtask readiness`: reads every check set's results and computes the weighted pass rate
   with the existing weights (features 0.30, file formats 0.20, layout 0.15, UI/UX 0.10,
   stability 0.10, performance 0.05, localization 0.04, platforms 0.03, ecosystem 0.02, hardware
   0.01); writes `docs/readiness.md`, whose number replaces the estimate in `ROADMAP.md`.
2. Real-world DOCX corpus: at least 200 files that may be reused (public-domain government
   documents and openly licensed ones), each with a reference PDF that Microsoft Word produced
   (PDF Producer "Microsoft Word"), so no Word installation is needed. Files stay out of git
   (`/corpus`); a manifest in the repo lists URL, licence and SHA-256.
3. File-format checks per corpus file: opens without error; save and reopen keep text, styles,
   tables, pictures and unknown parts.
4. Pagination checks against the reference PDFs: page count, and the first and last words of
   every page.
5. Stability checks: open, lay out and save every corpus file under a timeout with no panic or
   hang; the autosave soak test; the existing hostile-parameter fuzzing.
6. Feature and UI checklist: a fixed table of Word for Windows features, dialogs and options and
   Microsoft's published default shortcut list, each row tied to a test or a manual check on
   Windows.
7. Performance thresholds (open a 500-page document, typing latency, scrolling), localization
   (catalog completeness per language against Word's 30 interface languages), platforms (build
   and install per platform), hardware and ecosystem rows as pass/fail.

**B. Measure once** and record the first measured number in `ROADMAP.md` (done 2026-10-25: 66.1%
replaces ~60% in `ROADMAP.md`, `target-app-parity.md` and `gaps.md`).

**C. Fix what the checks fail**, highest weight per hour first, until `cargo xtask readiness`
reports 100.0% on the latest commit: every row of every check set passing. The latest official
Windows measurement 43194683237 passes stability (220/220 corpus; 5/5 hand checks),
performance (6/6) and language catalog-key coverage (95/95; translation quality is not tested);
the remaining measured area without deferred rows is pagination (182/220 page-count and 100/220
boundary checks). Dialogs pass 216/228 with 12 deferred, and shortcuts pass 276/280 with four deferred;
all non-deferred dialog and shortcut rows pass. The current source adds regression tests for OS
text/IME delivery and keypad behavior; this Windows run confirms the same 276/280 shortcut result.
Earlier work fixed contextual trailing spacing between paragraphs in table cells. An earlier
Windows report had 175/222 page-count passes and 95/222 boundary passes; the trial section-marker
change was reverted after a corpus boundary regression. Deferred work is now active under the owner's 2026-11-07 request;
[the completion plan](readiness/deferred-plan.md) lists the remaining implementation and verification prerequisites.
Some rows can only pass
on the owner's hardware: the printer, pen, microphone, scanner and multi-monitor checks. Windows
ARM64 packaging passed after the earlier run 42213688289 link failure: run 42262956358
built and verified the MSI and portable ZIP. The artifacts are unsigned and were not uploaded or
published. Linux and macOS package checks already pass in
`manual.tsv`; touch and other Windows session checks pass under the recorded owner rule. 100% further needs every
corpus document to paginate exactly as Word does, an interface catalog for each of the 95 languages
Microsoft 365 offers, and every Word dialog and shortcut in the lists, including the ones tied to
Microsoft services (owner rule: out of scope), so those rows decide how close the number can get.
Those rows wait as `deferred` until the owner asks for them (owner, 2026-10-27), and
[`readiness.md`](readiness.md) reports a second number beside the measured one: readiness without
the deferred rows, also aimed at 100% ([rules](readiness/README.md)). The work: real-world DOCX failures (gap #1), pagination
(gap #4), crashes and hangs on Windows (gap #3), Word ink round trip (gap #2), native printing
(gap #6), remaining dialog depth (gap #7), the four deferred OS-owned shortcut rows (gap #24), editing Word's charts and
SmartArt (gap #13), and the alpha gate re-run on Windows.

## Next after beta

Charts and SmartArt editing, the Draw tab, `.doc` writing, PDF Reflow, the remaining interface
languages, screen-reader access.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-11-12 | minor | Suppress contextual space after a paragraph before an in-flow table starting with the same paragraph style, including nested tables. Linux govuk-034 now matches 14 pages and all boundaries: corpus pages 182→183/220, boundaries 95→96/220, no regressions; two new regressions, CI all six steps pass (2,496 native tests). Windows run 43194683237 confirmed the earlier Unicode fix (94.6% without 22 deferred); table-spacing change awaits Windows measurement. |
| 2026-11-12 | minor | Windows run 43190908818 measured `c030b15`, report `383646e`: 92.7% / 94.6% without 22 deferred, pages 182/220 and boundaries 100/220. Fix Unicode sans names selecting monospace; CI all 6 steps passed (2,494 native tests); Linux 220-document results unchanged. Audit reference metadata without changing criteria or excluded/deferred rows. Font change awaits Windows measurement. |
| 2026-11-12 | minor | Preserve floating drawing stacking order across DOCX/JSON, paint and Arrange commands; handle standalone breaks before empty continuous-section ends; readiness criteria and excluded/deferred rows unchanged; new Windows measurement pending |
| 2026-11-12 | minor | แก้วัตถุที่อิงตำแหน่งย่อหน้าและอยู่หน้าหรือหลังข้อความให้ตามตัวแบ่งหน้า/คอลัมน์ก่อนจุดยึด; ผล Windows ล่าสุดยังเป็น run 43051188273 (92.7% / 94.5% ไม่นับ 22 deferred), รอวัดโค้ดรอบนี้ |
| 2026-11-10 | minor | Windows run 42836913553 measured `1352238`; report commit `8fc9173`: 92.2% / 94.1% without 22 deferred, page counts 174/222, boundaries 95/222 and languages 95/95 (22 AI-drafted catalogs added, awaiting native review). Release 1.1.3. |
| 2026-11-11 | minor | Windows run 42961322752 measured `9abf2d6`; report commit `1df7aae`: 92.2% / 94.1% without 22 deferred, page counts 175/222 (zenodo-103 now matches Word: table text no longer snaps to the document grid unless `w:adjustLineHeightInTable` is set), boundaries 95/222 and languages 95/95. v1.1.3 was deleted and released again from the latest code, adding the nested-table depth cap (18), control/MCP refusal to discard unsaved work, shared undo style sheets, the AccessKit focus guard, the Font spacing ▲▼ arrows and an About window that names the developer instead of linking the source. Headline numbers refreshed (503/503 catalog, 729 commands, 2,478 tests, 233,264 lines of Rust; 3,034 keys per catalog). |
| 2026-11-11 | minor | Windows run 43051188273 measured `bd2d2d6`; report commit `5292abd`: 92.7% / 94.5% without 22 deferred on 220 documents (zenodo-104 and govuk-003 excluded by owner decision: their reference PDFs aren't of their .docx); page counts 181/220, boundaries 99/220, languages 95/95. Changes since run 42961322752: theme fonts, Helvetica/Times substitution, bundled metric-compatible fonts (Carlito, Caladea, Arimo, Tinos, Cousine; OFL 1.1) behind the platform's own fallbacks, mode-14 page-break mark, page numbering from 0, continuous sections below floats, digit-hyphen line breaks, grid in tables, Font spacing ▲▼ at 0.1 pt. v1.1.3 was deleted and released again from this code. |
| 2026-11-07 | minor | Windows run 42334133340 measured `1868cb4`; report commit `ed787f9` confirms 91.3% / 93.2% without 22 deferred, page counts 174/222, boundaries 96/222 and languages 72/95, unchanged from the preceding official report. |
| 2026-11-07 | minor | Latest official Windows run 42274814600 measured `ed4284d`; generated report commit `29e6672` records 91.3% / 93.2% without 22 deferred rows, pages 174/222, boundaries 96/222, languages 72/95, dialogs 216/228 and shortcuts 276/280; platforms pass 5/5. Pagination is one page-count and one boundary pass below the previous report. |
| 2026-11-06 | minor | Previous official Windows run 42211564189: readiness 90.5% / 93.1% without 23 deferred; dialogs 210/222, pagination 175/222 pages and 97/222 boundaries, languages 68/95 and shortcuts 276/280. A later report supersedes these values. |
| 2026-11-06 | minor | At the time, ARM64 `plat-002` had passed after run 42262956358 verified the MSI and portable ZIP; run 42262955486 was canceled before measuring the pass. Replacement run 42274814600 later completed and refreshed the report. |
| 2026-11-05 | minor | Windows run 42025088010: 89.8% total / 92.5% without 26 deferred; legacy printer setup dialog mapped to File › Print; dialogs 205/220, pagination 174/222 pages and 97/222 boundary checks, languages 64/95. |
| 2026-11-05 | minor | Windows run 42020535590: 89.8% total / 92.4% without 26 deferred; eight legacy dialogs excluded with reasons; dialogs 204/220, pagination 174/222 pages and 97/222 boundary checks, languages 64/95. |
| 2026-11-05 | minor | Windows run 42017927963: 89.3% total / 91.8% without deferred; languages 64/95 (Catalan Valencia and Serbian Cyrillic Bosnia alias existing complete catalogs); pagination 174/222 pages and 97/222 boundary checks. |
| 2026-11-05 | minor | Windows readiness run 42012615601: 89.2% total / 91.7% without 26 deferred rows; pagination 174/222 page counts and 97/222 page-boundary checks; dialogs 204/228; languages 62/95. Contextual spacing between paragraphs in table cells fixed and tested. |
| 2026-11-02 | minor | Hidden table rows and `w:hideMark` kept and laid out as in Word (govuk-066 6 pages like Word); a page break ending a section's last paragraph leaves no blank page (zenodo-029, zenodo-140); Linux corpus 171/222 page counts, 91/222 page breaks |
| 2026-11-02 | minor | M14: v1.0.0–v1.1.2 published on the fork `veevie55/doc4me` (v1.1.2 the latest), after Actions of `chatraweechai/doc4me` stopped over a billing lock; a page break before an empty paragraph ending a section leaves no blank page (Linux corpus: page counts 166 → 168, page breaks 89 → 90) |
| 2026-11-01 | minor | Map classic New Toolbar and menu customization dialogs to the tested Customize Ribbon replacement; dialogs 187/228, readiness 86.0% (88.5% without deferred) |
| 2026-11-01 | minor | Map the legacy Advanced Settings dialog to the tested File › Options › Advanced pane; dialogs 184/228, readiness 85.8% (88.3% without deferred) |
| 2026-11-01 | minor | Verify eco-003 with Syncthing 2.1.7: save-in-place on peer B synced to peer A and both edits reopened; ecosystem 5/6, readiness 85.7% (88.2% without deferred) |
| 2026-11-01 | minor | Skip only leading table-row split pieces containing page-break clusters; govuk-003 page count now matches (5) without changing its still-failing page-boundary check; corpus page counts 162/222, page breaks 87/222; readiness 85.4% (87.8% without deferred) |
| 2026-11-01 | minor | `w:gridBefore`/`w:gridAfter` support fixes govuk-009 pagination; corpus passes rise to 161/222 page counts and 87/222 page breaks; readiness 85.4% (87.8% without 26 deferred) |
| 2026-11-01 | minor | Readiness 85.3% (87.7% without 26 deferred); exclude Macintosh-only dlg-040; zero-size frame markers stay in page flow; Syncthing ecosystem check remains untested |
| 2026-11-01 | minor | Full readiness 85.2% on 222 documents (87.6% without 26 deferred checks); DOCX round-trip 222/222; shortcuts 268/280 after Alt+Shift+F10 available-actions menu; remaining score is blocked by pagination, dialogs and localization |
| 2026-10-27 | trivial | Target raised from 95% to 100% measured (owner); what 100% needs |
| 2026-10-25 | trivial | Target raised from 80% to 95% measured (owner); the rows only the owner can check |
| 2026-10-25 | trivial | Stage C so far: measured 73.4% |
| 2026-10-25 | trivial | Path to 80%: stage B done, 66.1% measured replaces the ~60% estimate; alpha-gate row for received .docx from the corpus |
| 2026-10-25 | trivial | Path to 80%: stage A done (measurement and a 222-document corpus); the number goes into `ROADMAP.md` in stage B |
| 2026-10-25 | trivial | Path to 80% measured readiness: build the measurement (xtask readiness, DOCX corpus with Word-made PDFs, pagination, stability, feature checklist), measure, fix to 80% |
| 2026-10-25 | trivial | Beta list item 5 no longer waits on an Aptos-metric font (Intos bundled) |
| 2026-10-25 | trivial | Next after beta no longer lists ecosystem decisions or an in-app assistant (out of scope, owner decision) |
| 2026-10-25 | trivial | M2, M3 and M11 done: Dictate, Online Pictures, Get Add-ins, Ink Equation and Ink to Math landed with free and open-source parts; catalog parity 499/499 |
| 2026-10-25 | trivial | M1, M2, M3, M8, M11 and M12 rows checked against the code; M3 and M8 done apart from service-bound items; alpha gate's command count 618 |
| 2026-10-11 | trivial | M15: Polish interface and Polish proofing landed |
| 2026-10-11 | trivial | M3: icons landed (#309) |
| 2026-10-11 | trivial | M11: contour (Tight/Through) wrap done (#354) |
| 2026-10-11 | trivial | M6: the eight missing bibliography styles landed (#383) |
| 2026-10-11 | trivial | M7: formatting revisions landed (#41) |
| 2026-10-11 | trivial | Omitted `w:pPrDefault` stays omitted across saves; explicit empty/default properties and later spacing edits remain writable. Synthetic preservation checks only; readiness estimates unchanged |
| 2026-10-10 | trivial | M5: Draw Table and Eraser landed (#303) |
| 2026-10-10 | trivial | Merged main: PRs that landed removed from the Current focus list |
| 2026-10-10 | minor | Current focus heading back to ~150–230 h to beta: full number restored to the additive weighted sum (~60%), method aligned with the standard, no new evidence |
| 2026-10-10 | minor | Alpha gate table added (core-workflow gate from the progress-docs standard): all six workflows pass, stage stays alpha |
| 2026-10-10 | major | Created: milestones moved from ROADMAP.md with remaining hours; Current focus from gaps.md; M15–M16 added |
| 2026-10-10 | trivial | M11: ink pens and eraser landed (#307) |
| 2026-10-11 | trivial | M11: lasso, Add Pen, Ink to Shape and Ink Replay landed (#395) |
