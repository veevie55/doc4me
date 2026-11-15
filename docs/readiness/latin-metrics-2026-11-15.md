# Font metrics and page counts · 2026-11-15

This follows the [32-row font/reference audit](page-count-audit-2026-11-15.md). The comparison below is a Linux probe, not a new official Windows readiness score. No corpus member, reference, exclusion, deferred row, weight or exact-match criterion changed.

## Accepted changes

- Arial Nova Cond Light and Arial Narrow use OFL Roboto at `wdth:75`, respecting light/bold and italic styles. Condensed outlines avoid crowding ordinary-width Arimo into narrow advances.
- Gotham Book/Medium use OFL Montserrat at weights 400/500; Gotham Bold uses 700. Overpass Bold now selects 700 rather than ExtraBold 800.
- Documents requesting Source Sans Pro directly use its OFL original in regular/bold/italic/bold-italic, rather than the later Source Sans 3. Existing Light/Semibold aliases are unchanged. The original 2.045 line metrics match the reference's 2.021; this does not prove identical glyphs across versions.
- Selected narrow, Gotham Book/Medium and Nyala advances are fitted from public PDF layout dictionaries. Nyala uses Noto Sans Ethiopic, with measured original line metrics; Gotham Book/Medium also use measured original line metrics. Other characters retain substitute advances. Original kerning, unmeasured styles and complete visual equivalence remain unverified. The existing Thai fit retains its advances, scales, kerning and line metrics.
- The legacy `*Tahoma-Bold-4304` face resolves as bold. Tahoma width fitting remains disabled because its trial regressed another document.
- An empty paragraph used solely to end a section before a next-page section no longer moves to a blank page because of inherited `keep_next`/`keep_lines`. The regression covers all four keep-flag combinations and multiple page-boundary positions. Other section markers retain their behavior.

`tools/prepare_corpus_fonts.py` verifies hashes and OFL notices for Roboto, Montserrat, Overpass, Noto Sans Ethiopic and the original Source Sans Pro. Google Fonts is pinned to `cc39c4c2c304bc368c9b7c311ea2bcc6013299fe`; the original Source Sans Pro uses official adobe-fonts/source-sans revision `ce77773581f4d454f0fa985c073bb25c721bfcf5`. No new font binaries are stored in this repository.

`tools/narrow_pdf_metrics.py` verifies source PDF hashes against the immutable corpus manifest and reads public `/Widths`, `/W` and `/ToUnicode` dictionaries. It generates numeric advance tables without extracting or redistributing font programs or outlines. Separate inspection of public PDF font metadata established original line metrics. The [ambiguity audit](font-width-ambiguities-2026-11-15.tsv) records two Nyala space advances, 293 and 234 per 1000, in different mappings. U+0020 is omitted from the fitted Nyala table; the native substitute advance is retained until context can be resolved. Regeneration is deterministic and avoids rewriting unchanged data.

## Corpus comparison

The [220-row comparison](latin-metrics-comparison-2026-11-15.tsv) uses unchanged corpus inputs and prepared OFL fonts on Linux. Baseline source is `6d257ac`. Page-count matches rise **187 → 189/220**, with **no previously passing page-count row regressing**. Only two counts change:

| Document | Before | Accepted | Reference | Result |
|---|---:|---:|---:|---|
| zenodo-095 | 9 | 10 | 10 | pass |
| zenodo-149 | 40 | 28 | 28 | pass |

The other 218 page counts are unchanged. This probe does not measure page boundaries or establish full font equivalence. The latest completed official Windows result remains run [43701689470](https://github.com/veevie55/doc4me/actions/runs/43701689470), source `fd465e8`: pages 188/220, boundaries 99/220; 92.9% overall / 94.8% without 22 deferred. A new official Windows measurement is required after pushing this arc. Its workflow now probes font resolution for every current page-count failure, plus fixed comparison documents.

A width-only Arimo experiment reduced zenodo-149 to 31 pages but crowded letters. Condensed Roboto reached 29; the empty section-marker correction removes the remaining blank page and reaches 28. These intermediate trials are not the shipped result.

## Tahoma experiment withdrawn

Regular/bold widths agree across public PDF dictionaries for govuk-038, govuk-043 and govuk-067, recorded in the [Tahoma width audit](tahoma-width-audit-2026-11-15.tsv). Activating them globally made govuk-067 match 16 pages (from 15), but changed previously passing govuk-044 from 10 to 11. That runtime adjustment was withdrawn. The table is research evidence, not an active mapping or a complete replacement. The layout/font interaction requires further investigation.

## Build environment and validation

ALSA development package `libasound2-dev` 1.2.14-1+deb13u1 is installed under `/workspace/deps/alsa` without requiring root. Its pkg-config prefix and development symlink use local headers and the installed system runtime. `pkg-config --modversion alsa` returns 1.2.14; `cargo check -p alsa-sys` passes. Package SHA-256: `dfa8e8a133ef8704093d03464d86a3e86d26232baf1e3827aa030cb68c437e1c`.

CMake 4.4.5 and the GCC 14 header path are configured for whisper.cpp/bindgen in `/workspace/deps/env.sh`. Existing table-test loops in engine/formats were rewritten as iterators for Rust 1.100 clippy without changing assertions. Local build prerequisites are ephemeral workspace setup, not repository dependencies.

- `cargo xtask ci`: **6/6 passed** (fmt, clippy, tests, assets, layers, wasm); **2,554 native tests pass**, one pre-existing ignored test. CI uses standard bundled fonts. Tests run sequentially with CPU affinity for reproducible timing; no performance threshold was relaxed.
- Separate prepared-font tests: **83/83 pass**; prepared CLI build succeeds and probes all 220 corpus documents.
- Assets: 141; layers: 20 crates; wasm: 16 crates. Workflow actionlint passes. Code statistics: 238,588 Rust lines / 2,563 test attributes; attributes are distinct from executed harness tests.
- Logs: `/workspace/work/font-metrics-ci-final.log`, `/workspace/work/prepared-font-metrics-tests.log`, `/workspace/work/prepared-font-metrics-cli.log`; final machine-readable comparison: `/workspace/work/font-metrics-final-comparison.json`.

## ภาษาไทย

ปรับฟอนต์แทน Arial แบบแคบให้ใช้ Roboto condensed, Gotham ให้ใช้ Montserrat ตามน้ำหนัก, แก้ Overpass Bold และใช้ Source Sans Pro ต้นฉบับที่เปิดเผยภายใต้ OFL สำหรับชื่อที่ตรงกัน เพิ่มความกว้างอักษรที่วัดได้และระยะบรรทัดบางตระกูล โดยไม่แจกจ่ายโปรแกรมฟอนต์กรรมสิทธิ์หรือเส้นร่าง glyph ยังไม่อ้างว่า kerning, glyph และทุกรูปแบบเทียบเท่าต้นฉบับครบทั้งหมด ช่องว่าง Nyala ที่พบค่าขัดกันยังใช้ค่าของฟอนต์แทน ไม่เลือกค่าตามที่ทำให้คะแนนดีขึ้น

แก้ย่อหน้าว่างที่ใช้ปิด section ซึ่งสืบทอด keep flags แล้วสร้างหน้าว่าง ผลเทียบ Linux ครบ 220 ฉบับเพิ่มจาก **187 เป็น 189 รายการที่จำนวนหน้าตรง**: zenodo-095 จาก 9 เป็น 10 หน้า และ zenodo-149 จาก 40 เป็น 28 หน้า ตรง reference ทั้งคู่ อีก 218 รายการจำนวนหน้าคงเดิม ไม่มีรายการที่เคยผ่านกลับมาตก ผลนี้ยังไม่ใช่คะแนน Windows และไม่ยืนยัน page boundaries ผล Windows ทางการล่าสุดยัง **94.8% ไม่นับ deferred** ต้องวัดซอร์สรอบใหม่ก่อนเปลี่ยนคะแนน

ถอนการเปิดใช้ความกว้าง Tahoma เพราะ govuk-044 ถดถอย แม้ govuk-067 จะผ่าน เก็บข้อมูลและเหตุผลไว้ตรวจต่อ ไม่เพิ่ม excluded ไม่แก้ reference และไม่เปลี่ยนเกณฑ์ ติดตั้ง ALSA development package ใน workspace แล้ว ตรวจ alsa-sys ผ่าน และ full CI ผ่าน **6/6**, native **2,554 tests** พร้อมทดสอบชุดฟอนต์เตรียมจริง **83/83** งาน page count ที่เหลือยังต้องแก้ต่อ ไม่ถือว่างานทั้งหมดเสร็จ
