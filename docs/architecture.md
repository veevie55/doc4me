# Doc4Me architecture

**Earlier font/section-marker study · 2026-11-15 (Windows confirmed):** [Font/section-marker audit](readiness/latin-metrics-2026-11-15.md): prepared-font corpus page-count matches improve 187→189/220, with zenodo-095 and zenodo-149 newly matching and no page-count regression among previously passing documents. Full CI passes 6/6 (2,554 native tests; one existing ignored); prepared-font tests 83/83. ALSA development setup is repaired. Windows run 43727351543 confirms 190/220 page-count matches and 95.1% excluding deferred; no criteria, references, exclusions or deferred rows changed.

> **Last reviewed:** 2026-11-16 · **Last updated:** 2026-11-16 · **Change:** added manual Windows x86 and x64 artifact workflows; latest official Windows run 43792165925, source `6d4976c`, report `5f362a3`: 93.2% overall / 95.1% without 22 deferred; page counts 193/220, boundaries 102/220. Criteria, corpus membership and exclusions unchanged.

**Windows verification · 2026-11-16:** ผลทางการจาก [run 43792165925](https://github.com/veevie55/doc4me/actions/runs/43792165925), source `6d4976c`, report `5f362a3`: **93.2% รวม / 95.1% ไม่นับ 22 deferred**; pages **193/220**, boundaries **102/220**, open/round-trip/stability **220/220**. ไม่ผ่าน pagination 27 page-count และ 118 page-boundary rows; [รายการพร้อมผลวัดภาษาไทย](readiness/blockers-2026-11-14.th.md) และ [English](readiness/blockers-2026-11-14.md) ตรงกับ run นี้. เกณฑ์, deferred และ excluded คงเดิม; เป้าหมาย 100% ยังไม่ถึง.


The latest official Windows readiness report is run 43792165925, source `6d4976c`, report commit `5f362a3`. The readiness renderer now lists every failing corpus row and links to the bilingual blocker assessment. Structural code metrics below are a separate snapshot and are not readiness scores. The longer design notes in
`plan/architecture.md` are gitignored and local to the original author's machine; this file is the
committed, current summary. Rules for working in the code are in [`AGENTS.md`](../AGENTS.md).

## Layers

237,589 Rust source lines across
20 workspace packages (including apps and `xtask`) against the dependency rules:
a crate may only depend on crates in lower layers, and nothing below L6 depends on egui, eframe,
winit or rfd.

| Layer | Crate | Lines | Job |
|---|---|---|---|
| L0 | `geom` | 1.0k | Units (points, twips, EMU, inches, cm), rectangles, measurement parsing |
| L1 | `doc` | 20.5k | Document model: stories, paragraphs, runs, tables, sections, styles, numbering, fields, notes, comments, revisions, equations (OMML-shaped math tree, linear/LaTeX input); edit primitives; bidi |
| L1 | `fonts` | 7.3k | Font database (bundled OFL + system fonts), metrics, outlines, shaping with HarfRust, Word font substitution (`word.rs`: Carlito ↔ Calibri, Caladea ↔ Cambria, Liberation ↔ Arial/Times/Courier), OpenType MATH |
| L1 | `metafile` | 3.6k | Windows Metafile (WMF/EMF) parser to neutral paths and bitmap model |
| L1 | `proof` | 5.1k | Proofing per language (`ProofLang` from a run's `w:lang`): English (public-domain word list), Polish (installed Hunspell dictionary through a Hunspell-style affix engine) and Thai (common misspellings) spelling and suggestions, rule-based grammar for English and Polish, hyphenation dictionary and Liang patterns (English, Polish) |
| L2 | `layout` | 23.6k | Shaping runs, line breaking, tabs, lists, tables (row splitting, floating tables), pagination, headers/footers, notes, floating objects and wrap, math layout, fields, hit testing; produces a display list |
| L2 | `docx` | 26.3k | OOXML WordprocessingML reader and writer (Transitional and Strict namespaces, `.docx/.docm/.dotx/.dotm`), OMML, custom properties, VBA parts preserved |
| L2 | `docbin` | 7.3k | Word 97-2003 `.doc` reader ([MS-DOC]: FIB, piece table, FKPs, SPRMs, stylesheet, lists, tables, notes, pictures) over a CFB container |
| L2 | `formats` | 14.1k | TXT, Markdown, HTML, RTF, ODT and LaTeX import/export through a shared intermediate model |
| L3 | `render` | 2.5k | vello_cpu rasteriser: pages and thumbnails from the display list; PNG/JPEG |
| L3 | `pdf` | 2.6k | krilla PDF export: font subsets, selectable text, links, outline, tagging |
| L4 | `engine` | 56.6k | `Session` (document, selection, undo, layout cache, view state), **command registry** (727 registered engine command specs, verified by `doc4me-cli commands --json`), Word feature catalog (`catalog.rs`), samples, file I/O dispatch (`io.rs`), Read Aloud sentence model |
| L4 | `zotero` | 2.1k | Zotero's word-processor integration protocol answered against a `Session` |
| L4 | `control-key` | 0.1k | Per-window control-port key files shared by the app and the MCP bridge |
| L5 | `mcp` | 1.6k | MCP server: every command as a tool; headless or bridged to the running app |
| L6 | `ui-egui` | 55.8k | Word-style front end: ribbon, keytips, mini-toolbar, Backstage, canvas, rulers, panes, dialogs, Equation tab, i18n catalogs, themes, control channel, headless `ui_shot` example |
| apps | `doc4me` | 3.9k | Desktop app (eframe + wgpu; logging, window geometry, file associations) |
| apps | `doc4me-cli` | 0.7k | `convert`, `text`, `inspect`, `run`, `commands`, `parity`, `mcp`, `zotero` |
| apps | `doc4me-web` | 0.7k | Browser build (eframe web runner, WebGPU with WebGL2 fallback) |
| xtask | `xtask` | 3.0k | Workspace automation, CI, readiness measurements, reports and development tools |

## Data flow

1. **Open:** `engine::io` picks a reader by extension (`docx`, `docbin`, `formats`, JSON) and
   produces a `doc::Document`.
2. **Edit:** every user-visible action is a command id run through `Session::run` (ribbon,
   shortcuts, command search, CLI, control channel and MCP all dispatch by id). `Session::run`
   snapshots for undo (blocks are `Arc`-shared, so snapshots are cheap) and catches an escaped
   panic, restoring the document (never-crash standard).
3. **Layout:** `layout` turns the document into pages of positioned glyphs, boxes and images
   (display list), incrementally: an edit relayouts only what changed (1.4 ms on a 188-page
   document, measured 2026-10-06).
4. **Draw:** the UI paints pages through `render` (CPU raster, cached per page, pixel-aligned) on
   an egui/wgpu canvas; PDF export walks the same display list through `pdf`.
5. **Save:** `engine::io` picks a writer by extension.

## Testing and gates

- 2,557 native tests passed (2,566 source test attributes), covering
  input, `.doc`, formats, typing parity (`engine/src/tests_typing.rs`), bidi, i18n catalogs, UI
  interaction (egui_kittest), MCP acceptance; `hostile_params_never_panic` runs every command with
  junk params.
- `cargo xtask ci`: fmt, clippy `-D warnings`, tests, asset attribution, layering, wasm build.
- `cargo xtask parity` regenerates [`parity-checklist.md`](parity-checklist.md); a test keeps the
  catalog percentage above a floor that only rises.
- GitHub Actions includes manual Windows x86 and x64 package workflows that upload MSI/portable ZIP
  artifacts for a selected source ref, plus Windows ARM64, macOS, Linux, FreeBSD and Web App builds.
  The separate `windows-x64-installer.yml` can publish a Release only when explicitly requested;
  `readiness-windows.yml` measures the corpus and updates the generated report. Run `cargo xtask ci`
  before pushing. See [`releasing.md`](releasing.md).

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-11-16 | minor | Windows run 43792165925, engine 6d4976c, report 5f362a3: 193/220 page counts (zenodo-082 newly matches after the Cambria Math line fix), 102/220 boundaries (zenodo-032/073/082 newly match), 93.2% overall / 95.1% without deferred; no newly failing row versus report 7e50321. Criteria, corpus, exclusions and deferred unchanged. |
| 2026-11-16 | minor | Windows run 43785505707, engine 55d0c37, report 7e50321: 192/220 page counts (govuk-036 newly matches after the restart-recto fix), 99/220 boundaries, 93.0% overall / 94.9% without deferred; no newly failing row versus report b6e1076. Criteria, corpus, exclusions and deferred unchanged. |
| 2026-11-16 | minor | Windows run 43776538181, engine e00daf1, report b6e1076: 191/220 page counts, 99/220 boundaries, 93.0% overall / 94.9% without deferred; zenodo-136 newly matches, no newly failing corpus row. Prepare v1.1.4 refresh with unchanged criteria. |
| 2026-11-15 | minor | Update current readiness documentation; publication cancelled by user. Windows run 43727351543 (f01f7b6, report 4d70b3c): 93.0% overall / 94.9% without deferred, pages 190/220, boundaries 99/220; bilingual blockers match; no criteria/exclusion changes. |
| 2026-11-15 | minor | Added separate manual Windows x86 and x64 artifact workflows, with source SHA provenance; corrected the current workflow inventory. |
| 2026-11-14 | minor | Table/clipboard and popup layering verified; CI 6/6, 2,543 native passed; Windows run 43541894863, source 97e1d6f, report 7a38383: 95.1% without 22 deferred; pages 188/220, boundaries 108/220. Target 100% remains open. |
| 2026-11-14 | minor | Table/clipboard audit: 53 commands, 32 engine regressions plus raster and UI checks; CI 6/6, 2,543 native passed; 2,552 test attributes, 237,580 Rust lines, 730 commands, ribbon 504/504. Latest measured Windows source `8797e66` in run 43609090751: 95.1% without deferred, 188/220 pages and 108/220 boundaries; table changes are included in this measurement. |
| 2026-10-10 | minor | `proof`: proofing languages (English, Polish), affix engine, Polish patterns; layout and engine route each run by its language |
| 2026-10-10 | major | First committed architecture doc, from the crates' sources and `AGENTS.md` |
