# Body contextual spacing · 2026-11-13

ECMA-376 / ISO 29500-1 §17.3.1.9's contextual-spacing example subtracts the paragraph's contribution from the merged gap: after 10 pt and before 12 pt leave max(10,12)−10 = **2 pt**, not 12 pt. [WordCraft 8e0f42e4](https://github.com/storytold/wordcraft/commit/8e0f42e4bc3afe0995cce4ee773c0aa7be076911) also reports discriminating Word-for-Mac probes. Doc4Me's body placement removed the preceding after-space but retained the complete next before-space. It now retains only the residual. The next paragraph's contextual flag controls its own before-space; a different style retains the ordinary merged gap. The additive compatibility option keeps its previous per-side behaviour.

Only body placement changes. Negative before-spacing keeps the existing bounds; no new input indexing, recursion, allocations, panics or unsafe code. Existing float repositioning follows the actual paragraph top. DOCX/JSON models, version, assets, fonts and all benchmark inputs and rules stay unchanged.

## Verification

Three native regressions cover one/both contextual flags, different styles, additive compatibility and a four-paragraph run. Exactly 24 pt lines isolate gaps from font metrics. Focused spacing tests: **12 passed**. Full CI: **all 6 steps passed**, **2,508 native passed**, one originally ignored, layout 322, assets 141 and wasm 16. Tests run single-threaded on CPU 0 through the workspace runner helper; timing assertions remain unchanged. Stats: 2,517 test attributes, 234,851 Rust lines; CLI registry unchanged at 729.

Complete Linux corpus comparison against published source `92025e8`: **189/220 pages**, **103/220 boundaries**, open/round-trip/stability **220/220**. Every row and reason of all five failure sets is identical, not just the aggregate. This correction improves the synthetic measured gap; it adds no corpus pass on Linux. Logs: `/workspace/work/contextual-body-tests.log`, `ci-contextual-body.log`, `corpus-contextual-body.log`; baseline `corpus-leading-blank.log`.

The latest official Windows result remains **95.1% without 22 deferred** on `92025e8`, run [43333680851](https://github.com/veevie55/doc4me/actions/runs/43333680851), report `c098032`: 188/220 page counts and 108/220 boundaries. This new source still needs its own Windows measurement. The 100% target remains open.

## Withdrawn broader trial

An initial trial also changed ordinary/contextual gaps inside `layout_blocks` (cells, headers and other boxes). Its synthetic spacing matrix passed 12 tests, but the full corpus fell from **189/103** to **187/102**: govuk-020 grew to 5 rather than 4 pages and zenodo-067 fell to 10 rather than 11. Open, round-trip and stability stayed 220/220. The complete box change was removed before final verification; no document-specific condition or test exclusion was added. The trial patch/log remain in the workspace as `merged-spacing-all-experiment.patch` and `corpus-merged-spacing.log`. Further box-spacing changes need discriminating Windows/reference checks.

The legacy `allowSpaceOfSameStyleInTable` prerequisite was reviewed separately ([bfacaf36](https://github.com/storytold/wordcraft/commit/bfacaf36274750171db9ed0d597c349baf5515e6)). Only govuk-010 has the enabled flag below mode 15 in this corpus, and it has no contextual-spacing declaration. That setting and the inferred end-of-cell Normal-style rule were not added in this arc.
