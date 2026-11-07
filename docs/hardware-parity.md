# Hardware parity with Microsoft Word

> **Last reviewed:** 2026-11-07 · **Last updated:** 2026-11-07 · **Change:** minor (the current branch adds a scale-aware multi-monitor window-geometry comparison and regression tests; physical Windows multi-monitor verification remains deferred) · **Target:** Microsoft Word (Microsoft 365) for Windows

Deferred work was explicitly resumed. Native printing, pressure-sensitive ink and scanner/camera acquisition still have implementation gaps in addition to unavailable physical devices; multi-monitor and microphone checks require physical Windows verification. See [the 22-row completion plan](readiness/deferred-plan.md).

Word is not a hardware-heavy app, but a few devices matter to its users: printers above all, then
pens, microphones (Dictate) and speakers (Read Aloud), and displays.

**Dimension: ~40% (estimated), 20–40 h to full** (pen/ink hours are counted with the Draw tab in
[`target-app-parity.md`](target-app-parity.md)).

| Hardware | Word | Doc4Me macOS | Windows | Linux / BSD | Web | Parity | Hours |
|---|---|---|---|---|---|---|---|
| Printers: native print dialog, printer choice, copies, duplex, page ranges, tray, scaling | ✅ (Word for Windows: File › Print in Backstage with printer, printer properties, copies, pages, one- or two-sided, collation, orientation, paper size, margins and pages per sheet; Word for Mac adds `WordPDE.plugin` to the system print dialog) | 🟡 PDF opens in Preview to print (#286) | 🟡 PDF opens in the default viewer (#286) | 🟡 PDF opens in the default viewer (#286) | ✅ browser print dialog (#209) | 35% | 5–8 |
| GPU rendering, HiDPI/Retina, pixel-aligned text | ✅ | ✅ wgpu + vello_cpu rasteriser, whole-pixel text (#110, #165) | ✅ DirectX 12 default (#50), OpenGL when DirectX 12 has no GPU driver (#316); startup crash on Intel UHD (#170) | ✅ (soft text on KDE at some zooms, #140) | ✅ | 80% | 3–5 (bugs) |
| Multiple monitors, per-monitor DPI, remembered window position | ✅ | ✅ (#38, #76) | 🟡 Scale-aware monitor-size comparison and regression tests are in the current branch; actual multi-monitor Windows verification remains deferred (`hw-003`) | ✅ | — | 85% | 1 |
| Pen / stylus: ink, pressure, eraser, ink to shape/math | ✅ (Draw tab) | 🟡 Draw tab (#307, #395; Ink to Math 2026-10-25): pens, pencil, highlighter, eraser, lasso, ink to shape and math, with the pen as a pointer; no pressure | 🟡 same | 🟡 same | 🟡 same | not re-measured | 4–8 (pressure, Word ink `w14:contentPart`) |
| Microphone: Dictate | ✅ (cloud speech service) | 🟡 whisper.cpp on the computer (2026-10-25): text after each pause, no voice commands; microphone permission asked | 🟡 same (not tried with a real microphone yet) | 🟡 same (ALSA/PulseAudio) | ❌ | not re-measured | 4–8 (voice commands, streaming) |
| Speakers: Read Aloud | ✅ | ✅ system speech, speed, sentence skip (#190) | ✅ | 🟡 | 🟡 | 75% | 1–2 |
| Touch screens and touchpads: pinch zoom, inertial scroll | ✅ | ✅ pinch (#179) | ✅ | ✅ inertial scroll (#252) | ✅ | 85% | 0–1 |
| Scanners/cameras (Continuity Camera) | Word for Mac only; Word for Windows has no scanner or camera command (pictures come in through Windows) | ❌ | ❌ | ❌ | ❌ | not a gap for Word for Windows | 0 (2–4 for the macOS build) |

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-11-07 | minor | Deferred work was explicitly resumed. Native printing, pressure-sensitive ink and scanner/camera acquisition still have implementation gaps in addition to unavailable physical devices; multi-monitor and microphone checks require physical Windows verification. See [the 22-row completion plan](readiness/deferred-plan.md). |
| 2026-11-06 | minor | Treat monitor dimensions as scale-dependent when comparing restored window geometry; synthetic regression tests cover DPI-only changes. Readiness `hw-003` remains deferred until tested on the owner's multiple-monitor Windows setup. |
| 2026-10-25 | trivial | Pen / stylus row: Draw tab and Ink to Math; Dictate row: whisper.cpp on the desktop |
| 2026-10-10 | trivial | Windows falls back to OpenGL when DirectX 12 offers only the software adapter (#316) |
| 2026-10-10 | trivial | Desktop File › Print opens the PDF in the system viewer (#286) |
| 2026-10-10 | trivial | Inertial touchpad scrolling landed (#252) |
| 2026-10-10 | major | First version |
