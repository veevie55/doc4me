# Overpass line spacing · 2026-11-16

## Evidence

Four corpus documents ask for Overpass: zenodo-022, zenodo-046, zenodo-067 and zenodo-069. The pinned open font input (`tools/prepare_corpus_fonts.py`) provides the variable Overpass from Google Fonts, whose OS/2 win metrics are (2316 + 756) / 2000 = **1.536 em**. Because the family name matches, Doc4Me treated it as the installed original and spaced its lines by those win metrics, on Linux and on the Windows readiness runner alike.

- **zenodo-067** is the evidence. Word had Overpass there: its PDF embeds the original static Overpass (Regular, Bold, ExtraBold, ExtraLight, SemiBold), whose font descriptors declare a 0.982 em ascent. Its body is 10.5 pt Overpass, single spacing. Word places those lines 13.32 or 13.20 pt apart (142 and 40 times), **13.294 pt** on average, which is 1.266 em. Doc4Me placed them 16.13 pt apart, so the document ran to 12 pages instead of 11.
- **zenodo-069** and **zenodo-022** share a template whose document default font is Overpass. Their cover has 48 empty Overpass 11 pt paragraphs. **Word never drew Overpass in them:** their PDFs embed only Calibri (and Times New Roman), and their font tables give Overpass the alternative name Calibri. The machine that made them evidently lacked Overpass. At Overpass' 1.536 em the cover took 811 pt of a 792 pt page, so its last paragraph, with the green bar and logo anchored in it, moved to a second page. At 1.266 em, closer to Calibri's 1.22 em, it fits as in Word. The rest of these two documents still differs, because Doc4Me draws Overpass where Word drew Calibri. Nothing in a document tells which fonts the author had, so that difference is not changed here.

## Change

`crates/fonts/src/word.rs`: `requested_line` gives Overpass and its named weights (Light, ExtraLight, SemiBold, ExtraBold) a 1.266 em line: the descriptor's 0.982 em ascent plus the 0.284 em descent the measured pitch implies. `single_line` applies it even when the face drawing the text is called Overpass, as it already did for Aptos and Cambria Math, because the installed open build's win metrics are not the ones Word spaced the original by. No font file, outline or font program changes; no corpus, reference, criteria, exclusion or deferred row changes.

## Validation

- Font unit test: Overpass, Overpass Light and Overpass ExtraBold step 1.266 em whatever face draws them; 13.293 pt at 10.5 pt.
- [220-document Linux comparison](overpass-line-comparison-2026-11-16.tsv), prepared-font release CLIs, baseline `df4f160` (Symbol/Wingdings bullets) against this change; all 440 DOCX/PDF hashes match `corpus.tsv`.
  - Page counts **196 → 197/220**: **zenodo-067** 12 → 11, and every page boundary now matches (boundaries **103 → 104/220**).
  - **zenodo-069** 33 → 26 pages (Word 24) and **zenodo-022** 28 → 22 pages (Word 20) move closer but still fail: their covers now fit one page, as in Word, while their remaining text is drawn in Overpass where Word drew Calibri (see Evidence).
  - **zenodo-046** (Overpass headings) still matches. No previously passing count or boundary regresses.
- The Windows readiness runner uses the same pinned variable Overpass, so this change should reach the official measurement too; the next Windows run decides.

## ภาษาไทย

เอกสาร corpus สี่ฉบับใช้ฟอนต์ Overpass ชุดฟอนต์เปิดที่ใช้ (Overpass variable จาก Google Fonts) มี win metrics 1.536 em เพราะชื่อฟอนต์ตรงกัน Doc4Me จึงใช้ค่านี้ทั้งบน Linux และบนเครื่อง Windows ที่วัดคะแนน หลักฐานมาจาก zenodo-067 ซึ่ง Word มี Overpass ต้นฉบับจริง (ฝังไว้ใน PDF) เนื้อหาเป็น Overpass 10.5 pt Word เว้นบรรทัดเฉลี่ย 13.294 pt (1.266 em) แต่ Doc4Me เว้น 16.13 pt จึงได้ 12 หน้าแทน 11 หน้า

ส่วน zenodo-069 และ zenodo-022 ใช้ template ที่ฟอนต์เริ่มต้นเป็น Overpass แต่ **Word ไม่ได้ใช้ Overpass เลย** PDF ฝังแค่ Calibri และตารางฟอนต์ระบุ altName ของ Overpass เป็น Calibri แสดงว่าเครื่องที่สร้างไฟล์ไม่มี Overpass หน้าปกที่มีย่อหน้าว่าง 48 ย่อหน้าเคยสูง 811 pt เกินหน้า 792 pt ตอนนี้พอดีหนึ่งหน้าเหมือน Word แต่ส่วนที่เหลือยังต่างเพราะ Doc4Me วาด Overpass ขณะที่ Word วาด Calibri ในเอกสารไม่มีข้อมูลว่าผู้เขียนมีฟอนต์อะไร จึงไม่เปลี่ยนส่วนนั้น

แก้โดยกำหนดระยะบรรทัดของ Overpass ทุกน้ำหนักเป็น 1.266 em (ascent 0.982 จาก descriptor ที่ Word ฝัง และ descent 0.284 จากระยะบรรทัดที่วัดได้) แม้ฟอนต์ที่ใช้วาดจะชื่อ Overpass ก็ตาม เหมือนที่ทำกับ Aptos และ Cambria Math ไม่เปลี่ยนไฟล์ฟอนต์หรือเกณฑ์ใด

ผลเทียบบน Linux ครบ 220 ฉบับ (hash ครบ 440 ไฟล์): จำนวนหน้าตรง **196 → 197** (zenodo-067 ตรง Word ทุกหน้า) ขอบเขตหน้า **103 → 104** zenodo-069 จาก 33 เหลือ 26 หน้า (Word 24) และ zenodo-022 จาก 28 เหลือ 22 หน้า (Word 20) ใกล้ Word ขึ้นแต่ยังไม่ผ่านด้วยเหตุผลข้างต้น ไม่มีรายการที่เคยผ่านกลับมาตก เครื่อง Windows ใช้ Overpass ชุดเดียวกัน ผลจึงน่าจะถึงคะแนนทางการด้วย ต้องรอยืนยันจากการวัดรอบถัดไป
