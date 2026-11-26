# ตรวจ upstream layout · 2026-11-12

ต้นทาง [storytold/wordcraft](https://github.com/storytold/wordcraft) เข้าถึงได้แล้ว (snapshot `07f977372d3767e4abf7f3b0cc77c351289013f2`, 2026-11-11) แม้ `veevie55/wordcraft` ที่บันทึกก่อนหน้าตอบ 404 ตรวจ LICENSE-MIT แล้วตรงกับ copyright/dual licensing ที่ fork เก็บไว้ ไม่เปลี่ยนใบอนุญาต

| Commit | เรื่อง | ผลตรวจ |
|---|---|---|
| [5c0a62a7](https://github.com/storytold/wordcraft/commit/5c0a62a7) | ไม่แบ่งระหว่างจุดกับวงเล็บเปิดที่ติดกัน | พอร์ต พร้อม regression ที่มี/ไม่มีช่องว่าง |
| [7830333c](https://github.com/storytold/wordcraft/commit/7830333c) | pageBreakBefore ที่ต้น Continuous section กลางหน้า | พอร์ต โดยแยกต้นหน้าจริงจากต้น section/คอลัมน์; regression มี Continuous/NextPage/หน้าแรก |
| [76345539](https://github.com/storytold/wordcraft/commit/76345539) | ปัดขนาด superscript/subscript เป็นครึ่ง pt จากผล Word for Mac | ทดลองแล้วถอน; คลัง Linux ขอบเขต 97 → 96/220 เพราะ zenodo-093 หน้า 23 ถดถอย แม้จำนวนหน้าคง 184/220 |
| [855a0fe9](https://github.com/storytold/wordcraft/commit/855a0fe9) | nested Can't Split row | อ่าน patch แล้ว; สแกน explicit w:cantSplit ใน nested rows ของเอกสารที่ยังไม่ผ่าน ไม่พบกรณีตรงเงื่อนไขนี้ ยังไม่ได้ backport โครงสร้าง row/cell alignment ใหม่ |
| [a891d861](https://github.com/storytold/wordcraft/commit/a891d861) | contextual spacing หลังย่อหน้า Normal สุดท้ายในเซลล์ | ยังต้องพอร์ต compatibility prerequisite allowSpaceOfSameStyleInTable/ขอบเขต legacy modes และตรวจ split-cell alignment ก่อนนำกฎท้ายเซลล์มาใช้; upstream เองระบุว่า Word comparison ที่แยกสาเหตุยังเปิดอยู่ |

การถอน superscript ไม่ได้ซ่อน failure: ลบทั้งการเปลี่ยน draw_size และเทสต์เฉพาะทดลอง กลับไปใช้พฤติกรรมเดิม ไม่เพิ่ม ignored/excluded/deferred `zenodo-093` มี section เดียว คอลัมน์เดียว ไม่มีข้อความ `.(` แต่มี superscript 43 แห่ง; หลังถอนแล้วกลับมาตรงทั้ง 33 หน้าและขอบเขตทุกหน้า เทียบคลังครบอีกครั้งทั้งห้าชุดตรวจตรงกับก่อนทดลองทั้งหมด

ซอร์สสุดท้ายมีเพียงสองกฎ break ที่พอร์ต คลัง Linux 184/220 pages, 97/220 boundaries; เปิด/round-trip/เสถียร 220/220 ไม่มีกรณีถดถอย CI 6/6, native 2,499 ผ่าน, ignored 1 เดิม ไม่ใช้ผล Linux แทนคะแนน Windows และไม่อ้างว่า replay ทุก commit ของ upstream แล้ว งาน pagination ที่เหลือยังต้องดำเนินต่อโดยคงเกณฑ์เดิม

## CI timing and isolation

Default parallel CI failed the existing `proof_text_walks_the_runs_once` timing assertion at ratio 33.6 (limit 32). That function and its test were unchanged. Isolated execution also varied around the limit; CPU 0-pinned single-thread runs passed at 30.4, 30.4, 28.8. The complete final gate passed with `RUST_TEST_THREADS=1 taskset -c 0 cargo xtask ci`: all 6 steps, native 2,499 passed / 1 originally ignored. No assertion threshold or test was disabled. Failed evidence remains in `/workspace/work/ci-upstream-refined-parallel-failed.log`, with isolated/pinned samples in `proof-linear-{isolated,pinned}.log`.

## Margin and picture follow-up

The preceding two break-rule ports were confirmed by Windows run [43252905246](https://github.com/veevie55/doc4me/actions/runs/43252905246) on `da21a99` (report `d6cd2fb`), unchanged from `e9b354b` in all five corpus sets. Subsequent review of `627ded32` (margin leading) and `dc388cfa` (picture lines) is recorded in [the follow-up audit](margin-leading-2026-11-12.md), including header wrapping, the blank-paragraph Keep with next case, deliberate scope limits and six regressions. That source needs its own Windows measurement.

## Windows confirmation and further review · 2026-11-13

Windows run 43333680851 on `92025e8` (report `c098032`) confirms the margin/picture/header/blank-line arc at 95.1% without deferred, 188/220 pages and 108/220 boundaries. The subsequent [justification review](justification-review-2026-11-13.md) tested and withdrew four `88d936c3` variants; the published implementation and tests remain unchanged.
