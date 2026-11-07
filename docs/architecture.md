# Doc4Me architecture

> **Last reviewed:** 2026-11-07 · **Last updated:** 2026-11-07 · **Change:** minor (current source metrics, 3 active CI workflows, Windows readiness run 42274814600; previously: `plan/architecture.md` is local-only) · **Target:** Microsoft Word (Microsoft 365) for Windows (manual Word comparison was on Word for Mac 16.113.4 through 2026-10-21; the Windows corpus readiness run now measures the current source, not a manual Word comparison)

The latest official Windows readiness report measured source ed4284d; the current working tree also contains an unmeasured xtask boundary-word diagnostic. The structure metrics below count the current working tree as of 2026-11-07.
The generated report is commit 29e6672. The longer design notes in
`plan/architecture.md` are gitignored and local to the original author's machine; this file is the
committed, current summary. Rules for working in the code are in [`AGENTS.md`](../AGENTS.md).

## Layers

216,629 Rust source lines across
20 workspace packages (including apps and `xtask`) against the dependency rules:
a crate may only depend on crates in lower layers, and nothing below L6 depends on egui, eframe,
winit or rfd.

| Layer | Crate | Lines | Job |
|---|---|---|---|
| L0 | `geom` | 0.3k | Units (points, twips, EMU, inches, cm), rectangles, measurement parsing |
| L1 | `doc` | 18.0k | Document model: stories, paragraphs, runs, tables, sections, styles, numbering, fields, notes, comments, revisions, equations (OMML-shaped math tree, linear/LaTeX input); edit primitives; bidi |
| L1 | `fonts` | 5.5k | Font database (bundled OFL + system fonts), metrics, outlines, shaping with HarfRust, Word font substitution (`word.rs`: Carlito ↔ Calibri, Caladea ↔ Cambria, Liberation ↔ Arial/Times/Courier), OpenType MATH |
| L1 | `metafile` | 2.9k | Windows Metafile (WMF/EMF) parser to neutral paths and bitmap model |
| L1 | `proof` | 5.0k | Proofing per language (`ProofLang` from a run's `w:lang`): English (public-domain word list), Polish (installed Hunspell dictionary through a Hunspell-style affix engine) and Thai (common misspellings) spelling and suggestions, rule-based grammar for English and Polish, hyphenation dictionary and Liang patterns (English, Polish) |
| L2 | `layout` | 21.1k | Shaping runs, line breaking, tabs, lists, tables (row splitting, floating tables), pagination, headers/footers, notes, floating objects and wrap, math layout, fields, hit testing; produces a display list |
| L2 | `docx` | 24.3k | OOXML WordprocessingML reader and writer (Transitional and Strict namespaces, `.docx/.docm/.dotx/.dotm`), OMML, custom properties, VBA parts preserved |
| L2 | `docbin` | 5.9k | Word 97-2003 `.doc` reader ([MS-DOC]: FIB, piece table, FKPs, SPRMs, stylesheet, lists, tables, notes, pictures) over a CFB container |
| L2 | `formats` | 12.5k | TXT, Markdown, HTML, RTF, ODT and LaTeX import/export through a shared intermediate model |
| L3 | `render` | 2.1k | vello_cpu rasteriser: pages and thumbnails from the display list; PNG/JPEG |
| L3 | `pdf` | 2.4k | krilla PDF export: font subsets, selectable text, links, outline, tagging |
| L4 | `engine` | 51.5k | `Session` (document, selection, undo, layout cache, view state), **command registry** (727 registered engine command specs, verified by `doc4me-cli commands --json`), Word feature catalog (`catalog.rs`), samples, file I/O dispatch (`io.rs`), Read Aloud sentence model |
| L4 | `zotero` | 1.7k | Zotero's word-processor integration protocol answered against a `Session` |
| L4 | `control-key` | 0.1k | Per-window control-port key files shared by the app and the MCP bridge |
| L5 | `mcp` | 1.2k | MCP server: every command as a tool; headless or bridged to the running app |
| L6 | `ui-egui` | 54.1k | Word-style front end: ribbon, keytips, mini-toolbar, Backstage, canvas, rulers, panes, dialogs, Equation tab, i18n catalogs, themes, control channel, headless `ui_shot` example |
| apps | `doc4me` | 3.7k | Desktop app (eframe + wgpu; logging, window geometry, file associations) |
| apps | `doc4me-cli` | 0.7k | `convert`, `text`, `inspect`, `run`, `commands`, `parity`, `mcp`, `zotero` |
| apps | `doc4me-web` | 0.7k | Browser build (eframe web runner, WebGPU with WebGL2 fallback) |
| xtask | `xtask` | 2.9k | Workspace automation, CI, readiness measurements, reports and development tools |

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

- 2,071
  input, `.doc`, formats, typing parity (`engine/src/tests_typing.rs`), bidi, i18n catalogs, UI
  interaction (egui_kittest), MCP acceptance; `hostile_params_never_panic` runs every command with
  junk params.
- `cargo xtask ci`: fmt, clippy `-D warnings`, tests, asset attribution, layering, wasm build.
- `cargo xtask parity` regenerates [`parity-checklist.md`](parity-checklist.md); a test keeps the
  catalog percentage above a floor that only rises.
- GitHub Actions has three active workflows: `windows-x64-installer.yml` builds the Windows x64
  MSI and portable zip on `v*` tags or manual dispatch; `platform-windows-arm64-readiness.yml`
  checks ARM64 packages on manual dispatch; and `readiness-windows.yml` measures the full corpus
  and updates the generated report on manual dispatch. The installer runs `doc4me-cli --version`
  but no fmt, clippy or tests, so run `cargo xtask ci` before pushing. See [`releasing.md`](releasing.md).

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-10 | minor | `proof`: proofing languages (English, Polish), affix engine, Polish patterns; layout and engine route each run by its language |
| 2026-10-10 | major | First committed architecture doc, from the crates' sources and `AGENTS.md` |
