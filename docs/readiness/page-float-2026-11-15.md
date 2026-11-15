# Page-sized blank floating tables · 2026-11-15

## Problem and accepted behavior

`zenodo-136` begins with an empty floating table used as a blue cover panel. Its minimum row height is 841.8 pt, with page/margin positioning and a negative vertical offset. The old decision compared its painted row height with the body height and converted it to an in-flow table, pushing the cover text and images onto page 2. Merely comparing painted height with physical page height still fails because cell/border insets push the row slightly past the page edge.

`table::TableLayout` now records a fit height from natural cell height and requested row minima, separately from the painted height that includes minimum-row insets. Vertical-merge growth is retained in both measures. Only a page/margin-anchored panel whose cell blocks are empty paragraphs without objects may use this fit height against physical page height. It remains floating and may bleed across the edge. The full painted height is unchanged. Content-bearing tables, nested-table content, paragraph anchors, and tables exceeding their existing fit limit retain the existing pagination path; no table text is admitted through the blank-panel exception.

No font, dependency, document schema, source/reference file, criterion, exclusion or deferred row changed. The extra row-fit vector grows linearly with the rows already laid out. No new input-controlled recursion, unchecked index or panic path was added.

## Validation

- Focused floating-table tests: **6/6 pass**. A new cover regression covers page/margin anchors, a nearly page-high minimum plus 10 pt insets, and a content-bearing control. The existing over-height table regression now covers paragraph/page/margin anchors and verifies all 80 rows remain paginated and drawn.
- [All 220 page counts](page-float-comparison-2026-11-15.tsv), using prepared OFL fonts on Linux against accepted source `f01f7b6`: **189→190/220**, with no previously passing row regressing. **Only zenodo-136 changes, 26→25, matching reference 25.** The other 219 counts are unchanged. All 440 DOCX/PDF SHA-256 values match the immutable corpus manifest.
- An earlier broad physical-page-height experiment changed zenodo-053 from 59 to 57 against reference 58. That broader behavior was rejected. The accepted blank-panel guard leaves it at 59. Neither intermediate result is marked pass.
- Candidate cover rendering was inspected against the reference. The cover content is now on page 1 and remains legible. The painted blue panel still extends farther down the page than in the reference, and the horizontal rule differs; this does not establish pixel or complete page-boundary parity. Generated PDFs/PNGs stay outside the repository.
- Full CI passes **6/6**, native **2,555 passed / one existing ignored**, assets 141, layers 20 crates, wasm 16 crates. Fresh official Windows confirmation follows after push. This local page-count comparison is not a new overall readiness score. The current official baseline remains Windows run 43727351543, source `f01f7b6`, report `4d70b3c`: 190/220 page counts, 99/220 boundaries, 93.0% overall / 94.9% without 22 deferred.

Logs: `/workspace/work/page-float-tests-final-guard.log`, `/workspace/work/page-float-cli-final-guard.log`, `/workspace/work/page-float-comparison-final.log`, `/workspace/work/page-float-ci-final-guard.log`. Baseline CLI is saved as `/workspace/work/page-float-baseline-cli`; comparison JSON is `/workspace/work/page-float-comparison.json`. Earlier experiment logs are not evidence for the accepted code.

## ภาษาไทย

แก้กรณีตารางลอยว่างที่ใช้เป็นพื้นหลังปกและสูงเกือบเต็มหน้า เดิมระบบเทียบกับพื้นที่ข้อความแล้วเปลี่ยนเป็นตารางในเนื้อหา ทำให้ข้อความและรูปบนปกไปอยู่หน้าถัดไป แยกความสูงที่ใช้ตัดสินว่าพื้นหลังวางในหน้าได้ออกจากความสูงที่วาดรวมระยะขอบเซลล์ โดยไม่ลดความสูงที่วาดจริง ใช้ข้อยกเว้นเฉพาะพื้นหลังว่างที่อิงหน้า/ขอบกระดาษ ตารางที่มีข้อความหรือวัตถุ ตารางซ้อน และ anchor แบบย่อหน้ายังคงกฎแบ่งหน้าเดิม

ผล Linux ครบ 220 ฉบับผ่านจำนวนหน้าเพิ่มจาก **189 เป็น 190 รายการ** เปลี่ยนเฉพาะ `zenodo-136` จาก **26 เป็น 25 หน้า** ตรง reference อีก 219 รายการจำนวนหน้าคงเดิม ไม่มีรายการที่เคยผ่านกลับมาตก ตรวจ SHA-256 ของไฟล์ต้นฉบับและ reference ครบ 440 ไฟล์แล้วตรง manifest และทดสอบตารางลอยผ่าน **6/6** ไม่เปลี่ยนเกณฑ์ ไม่แก้ corpus และไม่เพิ่ม excluded/deferred

การตรวจภาพยืนยันว่าเนื้อหาปกกลับมาอยู่หน้าแรกและอ่านได้ แต่ความสูงแถบสีน้ำเงินและรูปแบบเส้นยังต่างจาก reference จึงไม่อ้างว่า pixel หรือขอบเขตหน้าตรงทั้งหมด ต้องรอผล CI และ Windows ของโค้ดชุดนี้ก่อนเปลี่ยนคะแนน readiness ทางการ
