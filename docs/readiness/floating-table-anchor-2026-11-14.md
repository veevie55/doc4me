# Floating table vertical anchor · 2026-11-14

The DOCX reader treated an omitted `w:tblpPr/@w:vertAnchor` as `text` (paragraph). Word defaults it to `margin`; an explicitly specified `text` anchor remains relative to the paragraph. The public [MS-OE376 §2.1.163 implementation note](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/cb9c4d92-d57f-4b2c-ab2e-6200db92fd83) documents this distinction. Horizontal defaults remain text/column. No positioning offsets or other floating-table rules change.

In the unchanged corpus `zenodo-157`, two body-level floating tables omit this attribute and request top/bottom alignment. Before the fix, the bottom table's contact details appeared above the title. With the margin default it sits near the bottom of the page, as in the reference PDF. This does **not** fix the document's page count: both versions still produce six pages against seven in the reference. The reference has non-embedded Times New Roman faces, so the Linux reference preview is not evidence of identical font appearance.

A handwritten DOCX fixture checks the omitted vertical anchor, explicit text, page and margin anchors, the unchanged horizontal defaults, bottom alignment and a save/reopen round-trip. The focused regression passed. Input-controlled parsing remains bounded and no new panic path, dependency or resource allocation is introduced.

The isolated experiment is based on release source `af59bac`, with a saved CLI from before the fix and a separate candidate CLI. Both complete Linux corpus runs exited successfully. Their five failure maps match exactly, including IDs and reasons:

| Check | Before | After |
|---|---:|---:|
| Open | 220/220 | 220/220 |
| Round-trip | 220/220 | 220/220 |
| Page count | 189/220 | 189/220 |
| Page boundaries | 103/220 | 103/220 |
| Stability | 220/220 | 220/220 |

Evidence in the execution workspace: `floating-anchor-test.log`, `floating-anchor-build.log`, `corpus-1.1.4-baseline.log`, `corpus-floating-anchor.log`, `zen157-1.1.4-before.pdf`, `zen157-floating-anchor.pdf` and first-page PNG/text comparisons. Generated corpus outputs, documents and PDFs are not committed. Full CI passed all six steps: 2,544 native tests passed, one originally ignored, assets 141 and wasm 16; Windows must measure the landed source before a new official score is claimed.

The latest official Windows report remains source `97e1d6f`, run [43541894863](https://github.com/veevie55/doc4me/actions/runs/43541894863): **95.1% without deferred**, pages 188/220, boundaries 108/220. No generated report, corpus input, TSV, comparator, weight, deferred or excluded row is changed. Target 100% remains open, alongside the existing exact-source Word-export and font prerequisites.
