# Justification review · 2026-11-13

[WordCraft 88d936c3](https://github.com/storytold/wordcraft/commit/88d936c3c5729079e0443f69ab5669dcea77ddca) reports Word-for-Mac compatibility-mode-15 measurements: spaces may shrink 25%, with an additional 0.355 overrun bound for the final word and its preceding space. The published Doc4Me source uses the prior 20% rule. Four candidate scopes were compared against the complete 220-document Linux corpus, using a separate immutable CLI for each. All four candidates were withdrawn.

| Candidate | Page-count passes | Boundary passes | New failure against 189/103 baseline |
|---|---:|---:|---|
| Full 25% + word-overrun rule | 188/220 | 109/220 | zenodo-165: 25 vs 24 pages, and boundaries fail |
| Same rule outside cells; retain prior cell policy | 188/220 | 109/220 | same document and failures |
| Only the 25% cap | 190/220 | 108/220 | zenodo-165 page 15 ends with `000`, reference `sig` |
| Only 25% outside cells; prior cap inside cells | 190/220 | 108/220 | same page-15 failure |

Every candidate kept open, round-trip and stability at 220/220, but aggregate gains did not establish that the Mac limits are correct for Word for Windows. zenodo-165's reference reports Microsoft Word 2016 / iLovePDF. Limiting the change to body paragraphs did not remove the regression. No document-specific exception, exclusion, deferred status, comparator or score weight was added. Both the implementation change and the experimental synthetic test were removed; `git diff --exit-code -- crates apps xtask` is clean against published source `92025e8` after the report-only commit.

Published readiness remains the Windows measurement of `92025e8`: **95.1% without deferred**, **188/220 pages / 108/220 boundaries**, run [43333680851](https://github.com/veevie55/doc4me/actions/runs/43333680851), report `c098032`. The source was rechecked with full CI after restoration (2,505 native passed; all six steps).

Evidence retained in the execution workspace: `corpus-space-{shrink,shrink-body,quarter,quarter-body}.log`, experiment patches, and the original `corpus-leading-blank.log`. Generated corpus files, reference PDFs and candidate binaries are not committed. The final app cap is unchanged at 20%.

Next: reproduce long/short-word cases in Word for Windows, with explicit fonts and compatibility modes, before adopting either limit. The GitHub integration could not list self-hosted runners (HTTP 403); no claim is made that a Word runner is available or that these Word probes ran. The earlier questions about a Windows/font environment and an exact Word re-export for zenodo-023 remain unanswered. No tag or Release was created.
