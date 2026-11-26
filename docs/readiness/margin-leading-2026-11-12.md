# Margin leading, picture lines and header wrapping · 2026-11-12

Source changes are Unreleased. The fixed ruler remains 220 included documents, 22 deferred rows and 24 excluded rows. No source DOCX, reference PDF, comparator, TSV, dependency, asset or version changed.

The prior Windows run [43252905246](https://github.com/veevie55/doc4me/actions/runs/43252905246) measured `da21a99` (report `d6cd2fb`): **92.9% overall / 94.8% without deferred**, **184/220 pages, 102/220 boundaries**. Its five corpus failure sets are identical to run 43245507907 on `e9b354b`. Windows later measured the new source as recorded below; Linux results are a separate environment.

## Rules and evidence

- [WordCraft 627ded32](https://github.com/storytold/wordcraft/commit/627ded32c7ea5a03d5bf81038c21553fab8bdbcf) reports 38 black-box Word probes for multiple-spacing text fitting at the bottom margin. Store descent separately from trailing leading and share a bounded, panic-free fit calculation. Footnotes and a footer above the bottom margin require full boxes. zenodo-114 previously matched baselines and line widths but lost a text line solely because the extra leading did not fit.
- [WordCraft dc388cfa](https://github.com/storytold/wordcraft/commit/dc388cfaab0c05f7e1569273ee6ca435854e1700) reports measured picture lines in Word: add the font's share of multiple spacing, without scaling the image. Whitespace and break markers do not need text descent; actual text/math do. A small picture ends at the font's bottom. Apply this to ungridded multiple-spaced lines outside cells; preserve table, At least, Exactly and grid rules pending separate validation.
- zenodo-121's header contains wrapping pictures reaching into the body area. Passing their contours to new body paragraphs removes the roughly 12 pt top displacement from the Word reference. Header choice respects first/even/default and linked references, with per-page caching and isolated layout state. A continued paragraph also clears a full-width header. Partial-width rebreaking of continued text remains open.
- zenodo-093 page 11 exposes a related Keep with next interaction: the heading keeps with an empty paragraph before the next body paragraph. Correct picture leading matches the reference's body baselines, but a full-box look-ahead moves the heading needlessly. Let only a final plain blank paragraph leave trailing leading in the margin; keep complete starting boxes for nonempty text. The final PDF retains all 33 pages and page boundaries. Its page 11 was visually checked against the reference (reference image compression differs).

## Verification

| Set | Linux before (`da21a99` behavior) | Final working tree |
|---|---:|---:|
| Open | 220/220 | 220/220 |
| Round-trip measured structure | 220/220 | 220/220 |
| Page counts | 184/220 | **189/220** |
| Every page's first/last word | 97/220 | **103/220** |
| Stability | 220/220 | 220/220 |

No document that passed a set becomes failing in that set. This does not claim every page of the still-failing documents is now correct. Five new page-count passes: zenodo-004 (24→23), -011 (33→32), -070 (13→12), -114 (75→71), -168 (21→20). Six new boundary passes: govuk-019, zenodo-042, -058, -065, -096, -162. zenodo-039 (37→36, reference 35) and -053 (60→59, reference 58) move closer but still fail.

Full `RUST_TEST_THREADS=1 taskset -c 0 cargo xtask ci`: **6/6**, **2,505 native passed**, one originally ignored, layout 319, assets 141, wasm 16 crates. Six new test attributes; stats **2,514 attributes / 234,769 Rust lines**; CLI registry **729**. CPU pinning retains the existing timing assertions and follows the prior arc's documented proof benchmark variability. No test threshold was reduced.

Evidence in the execution workspace: `ci-leading-blank.log`, `corpus-leading-blank.log`, baseline `corpus-upstream-refined.log`, `leading-blank-cli`, and `zenodo-093-blank.pdf`. Reference files and rendered PNGs are not committed.

## Windows confirmation · 2026-11-13

Run [43333680851](https://github.com/veevie55/doc4me/actions/runs/43333680851), source `92025e8`, report `c098032`: **93.2% overall / 95.1% without 22 deferred**, **188/220 page counts, 108/220 boundaries**. Against `da21a99`, all five sets have no newly failing document: four page-count gains (zenodo-004, -011, -070, -168) and six boundary gains (govuk-019, zenodo-042, -058, -065, -093, -162). Open/round-trip/stability stay 220/220. The actual measurement source and report header agree. Windows still has **32 page-count and 112 boundary failures**; 100% is not reached.

The final source was rechecked with full CI: 2,505 native passed, one originally ignored; six steps passed. Tests run single-threaded on CPU 0 through `/workspace/work/cargo-test-cpu0.sh` (the xtask launcher keeps normal compiler affinity), with unchanged timing assertions. Command: `RUST_TEST_THREADS=1 CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/workspace/work/cargo-test-cpu0.sh cargo xtask ci`; log `ci-windows-92025-docs.log`. This workspace helper is not shipped.

### zenodo-114: count agreement can hide two separate offsets

Linux reaches 71 pages but Windows reaches 70, against the reference's 71. The reference has a genuinely blank physical page 8; its quote is on page 9 at baselines 326.47 / 372.43 / 418.51 pt. Our quote is on physical page 8 at 326.35 / 372.35 / 418.34 pt. The source has `mirrorMargins`, a single-column `nextColumn` section start and a restart at page number 1. A mirrored-restart parity rule needs a discriminating Word probe; this is an actual layout investigation, not evidence of a bad reference.

At the biography, Linux falls back to Noto Naskh Arabic for the Arabic name and its first line is taller. Later lines are roughly 14.4 pt below the reference, pushing the final `man` onto another page. That extra page compensates for the missing blank in Linux's page count. The reference uses Times New Roman for that line. Word probes should vary mirror margins, starts 0/1/2, next-page/next-column/odd/even starts, first-page stories and SECTIONPAGES. Do not force a blank or suppress text by document id merely to pass.

The [justification review](justification-review-2026-11-13.md) records the additional experiments that were withdrawn.

## Rejected experiments and limits

Unrestricted margin fitting alone regressed zenodo-121 and -122: existing header/picture geometry had previously hidden those errors. A complete-start paragraph restriction reduced the gains and did not solve the cause, so both that restriction and its experimental test were withdrawn. A broader picture change was not accepted for cells. Trimming the kept starting lines of nonempty paragraphs regressed zenodo-043; it was withdrawn and the blank/nonempty distinction is tested. Final five-set comparison has no new failures.

This closes neither full pagination nor full Word parity. Linux still has 31 page-count failures and 117 boundary failures. The [reference audit](reference-audit-2026-11-12.md#zenodo-023-source-and-reference-content) retains zenodo-023's explicit empty last header/footer versus reference text and zenodo-149's unavailable Arial Nova Cond Light/Narrow fonts. Fresh Word export of the exact source and suitable installed fonts remain unconfirmed; these were not converted to deferred or excluded rows. Translation key coverage is not native-speaker review. Do not issue a tag/Release or claim 100% from these changes.
