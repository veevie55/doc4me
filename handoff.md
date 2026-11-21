# ความคืบหน้า session 2026-11-20 (ช่วงบ่าย) และสิ่งที่พบ

สถานะล่าสุดบน Windows: run 44793856355 (main 889fe68 = a00c04f + รายงาน) **96.3%** รวม / **98.7%** ไม่นับ 40 แถว deferred, corpus-pages 219/219, corpus-breaks **166/219** (ช่วงเย็นผ่านเพิ่ม: govuk-009, govuk-060 จาก header/footer ว่างตาม AppVersion; zenodo-056 จาก 等线→Carlito, ความสูงบรรทัด CJK ของ DengXian และตัวแบ่งบรรทัดท้ายสมการ)

ผ่านเพิ่มบน Windows ในช่วงนี้: zenodo-017, -071, -113, -146 (ตัวห้อย, keep with next), zenodo-155 (ย่อหน้าจบ section ใต้ตาราง), zenodo-085 (ช่องว่างในสมการ), zenodo-051 (แถวตาราง keep with next ไปกับย่อหน้าหลังตาราง); defer govuk-048 (กลุ่ม ก ตามกติกาเจ้าของ หลักฐานใน docs/readiness/deferred-pagination-2026-11-20.md)

แก้แล้วแต่ฉบับยังไม่ผ่าน (ไม่ตรงที่จุดถัดไป): zenodo-108 (เส้นขอบล่างเซลล์, ขนาดตัวยก/ห้อยครึ่งพอยต์, ตัวห้อยของอักษรมีหมวก), zenodo-138 (ข้อความหมุนตัดตามความสูงขั้นต่ำของแถว)

## สิ่งที่วิเคราะห์แล้วแต่ยังไม่แก้ (พร้อมหลักฐาน)

- **ความสูงบรรทัดของสมการ (zenodo-044, -108):** บรรทัดที่มีขอบเขตบนของ ∑ หรือหมวก/ขีดบนในสมการสูงกว่าของเรา 1.3–2.1 pt; ตำแหน่ง ink ทุกตัวตรงกับ Word (ตัวยก/ห้อยตรงแล้ว) ต่างเฉพาะความสูงบรรทัด ส่วนเกินเหนือ ink ใน Word: 1.77 pt (ขีดบน 12 pt), 2.01 pt (n ขอบเขตบน 8.5 pt), 1.26 pt (N ขอบเขตบน 7 pt) ยังหากฎเดียวที่อธิบายได้ไม่เจอ (ลองแล้ว: ascent ของฟอนต์ 0.779 em, ink + ช่องว่างคงที่) ค่าเมตริกอ่านได้จาก Cambria Math ที่ฝังใน PDF ของ Word (subset มีตาราง MATH) ด้วย fontTools เพื่อวัดเท่านั้น
- **zenodo-053 หน้า 20:** ตารางลอย (tblpPr vertAnchor=page) หลังตัวแบ่งหน้า Word แบ่งตารางข้ามหน้า 19/20 ต้องทำการแบ่งตารางลอยข้ามหน้า
- **zenodo-153 หน้า 1:** ย่อหน้าแรกเป็น Heading 1 ว่าง (space before 12 pt) ที่มีแต่กล่องข้อความลอย (VML, relative paragraph) Word วางกล่องเหมือนไม่มี space before ของเราต่ำไป 11.2 pt สแกนทั้ง corpus แล้วไม่ใช่กฎทั่วไปของย่อหน้าแรก
- **zenodo-056 หน้า 1:** ฟอนต์ธีม 等线 (DengXian) บน Linux แมปเป็น Carlito + ความกว้าง DengXian แล้ว แต่บน Windows ของเราวาดด้วย Arial (ดู pdffonts ของ PDF จาก artifact) น่าจะเป็นการแมปฟอนต์บน Windows ที่ยังไม่ใช้ alias เดียวกัน ควรตรวจต่อ
- **govuk-009 หน้า 1:** Word เริ่มเนื้อหาที่ 41.0 pt = ระยะ header 28.35 + บรรทัดว่าง Arial 11 pt ของ header ว่าง (เกินขอบบน 21.3) แต่ govuk-018 ที่โครงสร้าง header เหมือนกันทุกอย่าง (ว่าง, compat 15, เกินขอบบนพอๆ กัน) Word ไม่ดันเนื้อหา ยังหาตัวแยกไม่ได้ ห้ามเปลี่ยน `blank_story` จนกว่าจะเจอ
- **govuk-048 หน้า 1:** วาดเส้นแนวนอนเกินมาทับข้อความ (ไม่กระทบจุดแบ่งหน้า)
- **govuk-059 หน้า 6 (หลัง a00c04f จุดแบ่งหน้าย้ายไปหน้า 11):** กลุ่มเชิงอรรถของ Word จบที่ 756 pt (เส้นคั่นที่ 652.8) ของเราจบที่ขอบล่าง 770 pt (เส้นคั่น 665.4) สูงกว่ากัน 13.7 pt พอดีหนึ่งบรรทัด Arial 12 pt ตรวจแล้วไม่ใช่ footer (footer3 บรรทัดเดียวไม่เกินขอบล่าง), ไม่ใช่ beneathText, เชิงอรรถไม่มีย่อหน้าว่างต่อท้าย และสไตล์ไม่มี spacing ยังไม่ทราบสาเหตุ
- **govuk-067 หน้า 6:** ตาราง "Role details" สูงกว่า Word 14.5 pt (ตามหลังตารางลอย tblpY=1 twip) และ checkbox ของ Word เป็น glyph ใหญ่กว่า
- **zenodo-160 หน้า 7:** Word เริ่มหน้าต่ำกว่า 75 pt กราฟเล็กกว่า และมีรูปทรงสีเทาข้างข้อความ ยังไม่ได้วิเคราะห์
- **ปัญหาโครงสร้าง:** การวัดบน Windows ครั้งหนึ่งล้มเพราะ Chocolatey ตอบ 503 ตอนติดตั้ง poppler (สั่งใหม่แล้วผ่าน) ถ้าเจออีกให้สั่งใหม่
- **zenodo-068 หน้า 2→3:** เซลล์ผสานแนวตั้ง (vMerge restart ในคอลัมน์ซ้าย) ที่เนื้อหาสูงกว่าแถวที่ผสานในหน้านั้น Word ไหลเนื้อหาส่วนที่เหลือของเซลล์ผสานไปหน้าถัดไป ของเราเนื้อหาเซลล์ผสานเป็นของแถวแรกทั้งหมด จึงล้นเลยขอบล่างหน้า 2 ทับ footer (ต้องตัดเนื้อหาเซลล์ผสานข้ามหน้าใน `place_table`/`table::RowLayout::merged`) หน้า 1–2 ตรงกับ Word แล้วหลังแก้ tblpY (4ad0283)
- **zenodo-136:** แก้หน้า 15 แล้ว (แถวที่มีเซลล์จัดกึ่งกลางตัดข้ามหน้า) จุดต่างถัดไปอยู่หน้า 17: ของเราจบที่ "yes" ส่วน Word จบที่ "25" ยังไม่ได้วิเคราะห์
- **zenodo-115 หน้า 1:** หัวข้อ "Introduction" (Heading 1, before 10 pt) เป็นย่อหน้าแรกของ section 2 คอลัมน์แบบต่อเนื่อง หลังย่อหน้าว่างที่จบ section (after ของย่อหน้าก่อนหน้า 2 pt; Normal after 6 pt) ของเราต่ำกว่า Word 4.2 pt (คอลัมน์ขวาต่างแค่ 1.1 pt) สมมติฐาน: Word ไม่ใส่ space before ของย่อหน้าแรกบนสุดของคอลัมน์ใน section ต่อเนื่องใหม่ แต่ใช้ space after ของย่อหน้าจบ section แทน ต้องหาฉบับอื่นยืนยันก่อนเปลี่ยน (`continuous_top` ใน `place_para`) ทดลองกฎ "บนสุดคอลัมน์ของ section ต่อเนื่องใหม่ ใช้ space after ของย่อหน้าจบ section แทน space before" กับ corpus ทั้งชุด (env toggle ชั่วคราว ไม่ได้ commit): ไม่มีฉบับผ่าน/ตกเปลี่ยน แต่ตำแหน่งขัดกันเอง: ดีขึ้นใน zenodo-115 และ govuk-005 หน้า 12 (ห่าง Word 5.4 → 0.8 pt) แย่ลงใน govuk-037 หน้า 3 (เดิมตรง 102.71 เทียบ 102.72 → 99.66) และ zenodo-155 หน้า 4 (Linux) ยังหาตัวแยกไม่ได้

## เครื่องมือที่เพิ่มใน scratchpad (ไม่ได้ commit)

- อ่าน PDF จาก Windows: artifact `readiness-mismatch-pdfs` ของแต่ละ run (`list_workflow_run_artifacts` แล้ว curl `/actions/artifacts/<id>/zip`)
- triage ด้วยตำแหน่งกระโดดครั้งแรกต่อฉบับ เทียบ baseline ด้วย pymupdf (`pip install pymupdf`) และภาพเทียบสองฝั่ง

# งานที่เหลือก่อน readiness ครบ 100% และแนวทาง · 2026-11-20

## สถานะ (Windows run 44653897562, commit 951613d)

- **95.8%** รวม / **97.7%** ไม่นับ 22 แถว deferred; corpus-pages 219/219, corpus-breaks **153/219** (ไม่ผ่าน 66), open/roundtrip/stability 219/219, ribbon 504/504, performance/localization/platforms ครบ
- ส่วนที่ขาด 4.2 จุด มาจากสองแหล่งเท่านั้น
  1. **corpus-breaks 66 ฉบับ: ประมาณ 2.3 จุด** (Layout น้ำหนัก 0.15, คะแนน = ค่าเฉลี่ย pages กับ breaks = 84.9%) ผ่านเพิ่มหนึ่งฉบับได้ประมาณ 0.034 จุด
  2. **22 แถว deferred: ประมาณ 1.9 จุด** (dialogs 12, shortcuts 4, ecosystem 1, hardware 5) เจ้าของสั่งไว้ว่า "skip until asked" และต้องใช้ฮาร์ดแวร์/แพลตฟอร์ม/บริการภายนอก session ทำให้ผ่านเองไม่ได้ (ดูรายการใน docs/readiness.md › Deferred rows) **ถ้าจะให้ readiness ครบ 100% จริง เจ้าของต้องสั่งปลด deferred และจัดหาสิ่งที่ต้องใช้** (เครื่องพิมพ์, จอหลายจอ DPI ต่างกัน, ปากกา, ไมโครโฟน, สแกนเนอร์/กล้อง, IME ญี่ปุ่น/เกาหลี/จีนบน Windows, Mendeley, OLE server/Outlook/Exchange ซึ่งบางรายการเป็นของ Microsoft ที่ repo นี้ห้ามพึ่ง) ส่วน 97.7% → 100% ต้องให้ corpus-breaks ผ่าน 219/219

## corpus-breaks ที่เหลือ แบ่งกลุ่ม (triage บน Linux, `python3 tools/corpus_analysis.py triage LOG`)

Windows กับ Linux ไม่ผ่านที่หน้าเดียวกันแทบทุกฉบับ ยกเว้น zenodo-085 (Linux ไม่มีฟอนต์คณิต) zenodo-141 (ไม่ผ่านเฉพาะ Windows) และ zenodo-019 (จำนวนหน้าผิดเฉพาะ Linux)

### ก. ORDER: คำบนหน้าครบ ต่างแค่ลำดับที่ pdftotext อ่าน (10)
govuk-005 p12, govuk-024 p2, govuk-054 p11, zenodo-020 p1, zenodo-029 p2, zenodo-030 p16, zenodo-050 p2, zenodo-070 p10, zenodo-072 p6, zenodo-173 p5
- สาเหตุส่วนใหญ่: PDF อ้างอิงทำด้วยผู้ผลิตต่างกัน (Adobe PDFMaker 51/219 ใช้ head bbox ใน FontDescriptor, Word Save as PDF ใช้ hhea) และลำดับที่ข้อความถูกเขียนลง content stream ต่างจากของเรา ลองเปลี่ยน descriptor ทั้งชุดแล้ว +4/−6 จึงถอนออก
- แนวทาง: (1) ทีละฉบับ เทียบ `pdftotext -raw` และ bbox ของบรรทัดท้าย/หัวหน้า ดูว่าความต่างมาจากตำแหน่งจริง (ต่างไม่กี่ pt จนกลุ่มบรรทัดสลับ) หรือจากลำดับการวาด (header/footer ก่อนหรือหลังเนื้อหา, เซลล์ตาราง, กล่องข้อความลอย) ถ้าเป็นตำแหน่งจริง แก้ layout ต่อ ถ้าเป็นลำดับการวาด ให้วาดตามลำดับที่ Word ใช้กับ PDF ของ Word เอง (2) เลือก descriptor ต่อผู้ผลิตไม่ได้ เพราะ .docx ไม่บอกผู้ผลิต (3) การเปลี่ยนวิธีเทียบของ verifier (เช่น เทียบชุดคำของบรรทัดแรก/สุดท้าย) **เป็นการเปลี่ยนเกณฑ์ ต้องได้รับอนุมัติจากเจ้าของก่อน**

### ข. ฝั่งเราแก้ไม่ได้หรือต้องให้เจ้าของตัดสิน (6)
- zenodo-006 p1: header ใน PDF ไม่มีใน .docx (เป็นคนละ edition)
- govuk-066 p5: ฟิลด์ DATE ใน footer Word พิมพ์วันที่ตอนนั้น (08/02/2024)
- zenodo-018 p12: header เป็น WordArt gradient ที่ Word ไม่เขียนข้อความลง PDF
- govuk-068 p1: PDF ของ Word ใช้ Arabic presentation forms (การเทียบด้วย NFKC จะแก้ได้ แต่เป็นการเปลี่ยนเกณฑ์)
- zenodo-149 p6: ligature "ti" ใน PDF ของ Word ไม่มี Unicode ("Introduc�on")
- zenodo-022 p15: ลำดับ bullet ในตารางสองคอลัมน์ (layout ตรงแล้ว)
- แนวทาง: เสนอเจ้าของพร้อมหลักฐาน ให้เลือกระหว่าง defer/exclude หรือ normalize การเทียบ ห้ามเปลี่ยนเองแม้เหตุผลดี

### ค. small: คำไม่กี่คำข้ามขอบหน้า มีแนวทางแล้ว (14)
- govuk-006 p2, govuk-020 p2: ฟอร์มที่มีช่องติ๊ก Word วาดกล่อง sizeAuto ใหญ่ราว 15 pt ให้ข้อความ 11 pt (วัดจากภาพเท่านั้น) และยังต่างอยู่ราว 2 pt ในตารางสองคอลัมน์ของ govuk-020 ไล่ด้วย `match govuk-020 2`
- govuk-036 p8: บรรทัดสุดท้ายของ Word ล้นเข้าตัวคั่นเชิงอรรถ 0.26 pt (ผ่านที่ p7 แล้ว) p8 ยังต้องไล่
- govuk-038 p1: ข้อความหมุนในกล่อง ("DRAFT") pdftotext ได้ตัวอักษรแยก
- govuk-046 p11: แถวคำบรรยายตารางที่ keepNext ใน Word ย้ายไปหน้าถัดไป แม้มันกับแถวถัดไปพอดีหน้า
- zenodo-003 p13: เซลล์รวมแนวตั้งที่ข้อความหมุน ("CONTRIBUTION PERCENTAGE", "OTHER") Word ตัดบรรทัดตามส่วนที่อยู่ในหน้า ต้อง layout ใหม่ตอนแบ่งหน้า
- zenodo-056 p1: DengXian กับ ω/ตัวห้อยที่ Word วาดด้วยฟอนต์อื่น
- zenodo-119 p1: Arimo ใช้ kerning คู่ "ff"/"fi" แต่ Arial ไม่ใช้ (ต้องได้ตาราง kern ของ Arial เอง เช่นวัดจาก TJ ใน PDF ของ Word)
- zenodo-125 p3: marker ของ continuous section ว่างหลังหน้าเต็ม ลองให้ค้างท้าย p2 แล้วการเกลี่ยคอลัมน์ของ section ก่อนหน้าเปลี่ยน
- zenodo-126 p1, govuk-048 p1: ลำดับการอ่านในหน้า 1
- zenodo-140 p9: Word ย้ายแถว 3 บรรทัดที่บรรทัดแรกพอดีและเหลือที่ราว 3 pt (widowControl ปิด)
- zenodo-149 p6, zenodo-022 p15: ดูกลุ่ม ข

### ง. layout: เลื่อนหลายบรรทัด (41) แยกตามสาเหตุที่รู้
- **คณิตศาสตร์** (zenodo-017, -044, -051, -053, -070, -085, -108, -113, -155): บน Linux ไม่มีฟอนต์ที่มีตาราง MATH บน Windows มี Cambria Math และไม่ผ่านที่หน้าหลังกว่า (เช่น zenodo-085 p11) **แนวทาง: ต้องเห็นผลลัพธ์ฝั่ง Windows** ให้ workflow readiness-windows อัปโหลด PDF ของฉบับที่ไม่ผ่านเป็น artifact (แก้ workflow ต้องขอเจ้าของก่อน) หรือเพิ่มฟอนต์คณิต OFL (STIX Two Math/Latin Modern Math) ลง craft-fonts เพื่อประมาณบน Linux
- **เมตริกฟอนต์แทน**:
  - Arial Nova Cond Light ใน Word สูง 1.209 em (Roboto ที่ใช้แทนสูง 1.173 em) แต่แก้แล้ว zenodo-149 ได้ 29 หน้า เพราะแถวรายการหน้า 10 ของ Word ถี่กว่า ต้องแก้สองจุดพร้อมกัน
  - Tahoma: govuk-067
  - Cambria→Caladea บรรทัดยาวกว่า: govuk-020, govuk-043
  - Bookman Old Style: zenodo-157
  - วิธี: วัดความกว้างและเมตริกจาก PDF ของ Word ด้วยเครื่องมือใน tools/*_pdf_metrics.py แล้วใส่ใน crates/fonts/src/latin_metrics.rs หรือ word.rs `requested_line` ต้องได้ทั้ง corpus ไม่ใช่แค่ฉบับเดียว
- **ตาราง/วัตถุลอย**:
  - zenodo-157 p2: รูป Square ที่ anchor ท้ายย่อหน้าในแถวที่ถูกแบ่ง Word เก็บ 3 บรรทัดไว้ แล้วย้ายบรรทัดที่เหลือไปพร้อมรูป
  - govuk-031 p7: เริ่มสูงกว่า Word 1.9 pt จากหัวหน้า (ตารางต่อจากหน้าก่อน)
- **header**: govuk-009 p2 PDF ทำด้วย Word 2013 ซึ่งดัน body ลงใต้ header ว่าง ส่วน Word รุ่นใหม่ (govuk-017/018) ไม่ดัน ยังไม่รู้สิ่งที่ใช้แยกสองกรณีใน .docx ไล่หาความต่างที่เหลือ เช่น app.xml AppVersion หรือ compat settings อื่น ห้ามเดาจากผู้ผลิต PDF
- **ยังไม่ได้วิเคราะห์** (เริ่มด้วย `drift LOG`):
  - govuk: 028, 029, 037, 043, 047, 053, 058, 059, 060
  - zenodo: 008, 011, 023, 047, 048, 060, 068, 069, 071, 095, 103, 115 (บทคัดย่อ justify ตัดบรรทัดต่าง), 136, 138, 146, 153, 160
- zenodo-141 ไม่ผ่านเฉพาะบน Windows ต้องใช้ผลจาก Windows

## วิธีทำงานที่ได้ผล (ทำซ้ำได้ทุก session)

1. เตรียมฟอนต์: craft-fonts pin ที่ c1db6d1 แล้วรัน `tools/prepare_corpus_fonts.py`
2. build CLI โดย**ต้องตั้ง** `CRAFT_FONTS_DIR` (build.rs ฝังฟอนต์แทน ถ้าไม่ตั้ง จะไม่มี Roboto/Montserrat และ zenodo-095/149 จำนวนหน้าผิด หลัง `cargo xtask ci` ต้อง build ใหม่):
   `CRAFT_FONTS_DIR=… CRAFT_FONTS_REQUIRED=1 CARGO_INCREMENTAL=0 cargo build --release -p doc4me-cli`
3. วัด: `CRAFT_FONTS_DIR=… cargo xtask readiness --corpus --dry-run --diagnostics --jobs 4 --cli <cli> > log` ใช้เวลาราว 10 นาที
4. เทียบชุดที่ไม่ผ่านกับรอบก่อน: `grep -E "^corpus-(breaks|pages|open|roundtrip|stability)\s" log | awk '{print $1,$2}' | sort` แล้วใช้ `comm` **รับเฉพาะการแก้ที่ไม่มีฉบับใหม่ตก** ถ้ามีฉบับตก ให้หาสาเหตุ (หลายครั้งเป็นการชดเชยกันที่ไม่ถูก เช่น descent ของ bullet ที่บังเมตริก Arial Nova)
5. วิเคราะห์ฉบับหนึ่ง:
   - `tools/corpus_analysis.py ends ID PAGE`
   - `match ID PAGE` (หาจุดที่เริ่มเลื่อน)
   - `pitch ID N` (ระยะบรรทัดต่อฟอนต์)
   - `desc PDF` (เมตริกที่ Word ประกาศ)
   - เปิด XML ของย่อหน้า/ตาราง/สไตล์ที่ตรงกันแล้วหากฎ ข้อมูลที่ดีที่สุดคือ PDF ของ Word เอง (path ของกล่อง, ระยะเส้นขอบ, baseline)
   - ทดลองหลายสมมติฐานด้วย env toggle ชั่วคราว แล้วลบออกก่อน commit
6. ก่อน commit:
   - หาหลักฐานจากหลายฉบับ (เช่นสแกนทั้ง corpus ว่ากฎนี้กระทบใคร)
   - เพิ่มเทสต์ที่ไม่ผ่านถ้าไม่มีการแก้ (ฟอนต์ในเทสต์ไม่ใช่ฟอนต์จริง ให้ออกแบบเทสต์ไม่ให้พึ่งเมตริกเฉพาะ)
   - เขียน CHANGELOG ใต้ [Unreleased] พร้อมหลักฐาน
   - รัน `cargo xtask ci` ให้ผ่านครบ 6 ขั้น
   - commit แล้ว `git fetch && git merge origin/main` (workflow commit รายงานลง main) แล้ว push ไป main และ branch
7. สั่ง readiness-windows.yml หลัง push (เจ้าของอนุญาตเฉพาะ workflow นี้) แล้วอ่านผลที่ workflow commit กลับมาใน docs/readiness.md ห้ามสั่ง Release หรือ Pages
8. ลำดับที่คุ้มที่สุดต่อไป:
   - กลุ่ม ค ที่รู้กฎแล้ว (zenodo-003, -140, -125, govuk-046)
   - "ยังไม่ได้วิเคราะห์" ในกลุ่ม ง
   - ขอเจ้าของเรื่อง artifact ของ Windows สำหรับกลุ่มคณิต
   - เสนอกลุ่ม ก/ข ให้เจ้าของตัดสินพร้อมหลักฐาน

# Form check boxes, table-style cell spacing · 2026-11-20

- FORMCHECKBOX is now an em space in the run's font with a drawn square (`Enclosed`, `cross` when checked), like Word's PDF (paths, no text); a box bigger than its text adds descent up to that font's line at the box size. Table styles' `w:tblCellSpacing` is read/saved/used; with spacing each row holds its cells' bottom borders and the first/last rows the frame border. Exact lines taller than their text take the paragraph mark font's ascent share. Linux 153/219 (govuk-018, govuk-021), no new failure.
- Tried along the way: drawing the box line from the fallback face (old), from the run's line at box size (govuk-031 rows right but govuk-017 extraction order flipped), and at the run size alone (govuk-031 rows 1.1 pt short). Word's drawn sizeAuto box looks ~15 pt for 11 pt text in govuk-020 (render measurement only; not applied).
- govuk-031 p7 still starts 1.9 pt high from the page top (table continuing at the top of the page).

# Duplicate style ids · 2026-11-19

- List labels no longer add their descent to a line (Courier New "o" bullets: 7 documents now step like Word): zenodo-083 passes, govuk-009 fails (its page 1 starts 20 pt high: Word 2013 pushed the body below an empty header, the newer Word behind govuk-017/018 doesn't; a fix in either direction swaps which of them fails). Next lead: Arial Nova Cond Light's line is 1.209 em in Word (ascent 963, descent 224 per 1000 declared; Roboto stand-in 1.173), but setting it makes zenodo-149 29 pages (other compensations there: list rows on p10 are denser in Word).
- Contextual spacing in a cell compares with the cells beside it in its row (not the row above); text taller than its exact line takes a baseline at height × ascent/(ascent+descent) of the unshifted font, then its `w:position` moves it: govuk-044 passes, Linux 151/219.
- Not fixed: zenodo-157 p2 (a Square picture anchored at the end of a paragraph in a split table row: Word keeps 3 lines on the page and moves the rest with the picture to the next page; we keep the lines wrapped around the picture's old place); zenodo-085 fails on Linux at p5 only because no math font with a MATH table is installed (Windows, with Cambria Math, fails at p11).
- Merged centred/bottom cells cut by a page break align their text in the part on the page (zenodo-003 "Name" now at Word's y). zenodo-003 still fails: its turned merged cells ("CONTRIBUTION PERCENTAGE", "OTHER") are laid along the whole merge; Word wraps them to the part on the page (needs a relayout at pagination). zenodo-149 p6 and zenodo-022 p15 are extraction-only (Word's "ti" ligature has no Unicode; bullet order).
- A styleId defined twice in styles.xml takes its later definition (Word does): zenodo-026 passes, Linux 150/219, no new failure. Earlier this day: renamed embedded fonts (`___WRD_EMBED_SUB_NN` → font table name), no line number on the empty rest after a page break, footer top ignores the footer's own trailing multiple-spacing leading: zenodo-106 passes (716eb11).
- Build note: the release CLI must be built with `CRAFT_FONTS_DIR=/home/user/craft-fonts` set (fonts crate build.rs embeds the stand-ins); a build without it (e.g. after `cargo xtask ci`) loses Roboto/Montserrat and fails zenodo-095/zenodo-149 pages.

# Line fit tolerance; PDF font descriptor finding · 2026-11-19 (continued)

- Owner: keep going until readiness is 100%; sessions may dispatch readiness-windows.yml themselves. Windows run for 3b118a6: 145/219, 95.6% / 97.5%.
- Lines may overrun the room left by up to 0.3 pt (LINE_FIT_TOLERANCE): zenodo-049 passes, Linux 146/219, no regressions.
- Tried and reverted: PDF FontDescriptor Ascent/Descent/FontBBox from the font's head box (as Adobe PDFMaker writes): +4 (govuk-024, zenodo-030/050/070) −6 (govuk-072, zenodo-054/061/080/118/127). The corpus's reference PDFs come from different producers: Word's own Save as PDF writes hhea ascent/descent (what krilla writes now), Adobe PDFMaker (51 of 219 references) writes the head box, Mac Word (Creator "Microsoft Word") hhea too. The "ORDER" failures (same words, pdftotext order differs) are largely PDFMaker references; no single descriptor style matches all producers.
- govuk-036 p7: Word's last body line reaches 0.26 pt into the footnote separator; now fits with the tolerance, but the document still fails on p8's end.

# Split rows keep keep-with-next paragraphs; table cell spacing · 2026-11-19

- Embedded DOCX fonts (odttf, ECMA-376 §17.8.1) are now read and registered at open: zenodo-129 passes (Linux 145/219). Cell floats placed from the margin inside the cell grow the cell (govuk-021 p1 now differs only in pdftotext order).

- List items ending a cell keep automatic space after; STYLEREF to a style no paragraph uses shows Word's error text: zenodo-021 passes, Linux 144/219.
- Leads not fixed: zenodo-125 (empty continuous-section marker after a full page starts p3 in ours; letting it stay at p2's bottom changed column balancing of the section before — reverted); govuk-005 p12 (Word keeps 3.55 pt space before at a new section's page top, contrary to the after-break rule built on 8 other docs); govuk-046 p10 (keepNext caption row of a table moves on in Word though it and the next row fit); zenodo-140 p8 (Word moves a 3-line row whose first line fits with ~3 pt to spare; widowControl off).

- A split row whose text cell would keep nothing on the page (widow control) moves on whole: govuk-031, zenodo-123; Linux 141 → 143/219, no new failure.
- Tried and reverted: giving FORMCHECKBOX boxes drawn by a fallback face (Noto Sans Symbols ☐) the run's line height. govuk-006 p2 drift drops from +7 pt to about 1 pt, but govuk-017 and govuk-031 then fail: Word's box does make the line somewhat taller than plain Arial. Needs the real box height rule (sizeAuto box ≈ font size?) before retrying.

- `w:tblCellSpacing` is now read/saved and laid out (each cell inset by it, the table content by it again): zenodo-036 passes; with the keep fix Linux 139 → 141/219, no new failure. govuk-031 (also uses spacing) still fails on p8 for another reason.
- Triage of the rest (/home/user/work/triage.py): 10 ORDER (same words, only extraction order), 18 small (one line/paragraph across the boundary, e.g. zenodo-026 heading: we have slightly more height before it), 50 layout.

- The owner now allows sessions to dispatch `readiness-windows.yml` themselves (only that workflow). Run 44453166606 (`6b106e9`, clipboard work) gave 95.4% / 97.3%, breaks 139/219 unchanged.
- A row split leaves a keepNext paragraph (whose next paragraph in the same cell keeps no line) for the next page; a row starting with one moves on whole; rows already at a page top split as before. zenodo-144 passes; Linux 139 → 140/219, no new failure. Linux setup this session: craft-fonts at /home/user/craft-fonts (pinned c1db6d1, prepared), CLIs /home/user/base-cli and /home/user/cand-cli, `cargo xtask readiness --corpus --dry-run --diagnostics --jobs 4 --cli <cli>`, compare `corpus-breaks` lines of the logs. Helpers: /home/user/work/cmp.sh ID PAGE, /home/user/work/triage.py LOG (ORDER = same words on the page, only extraction order differs).
- Note: the readiness workflow builds only the CLI, so the Windows clipboard code in apps/doc4me is still unbuilt on Windows.

# DengXian widths · 2026-11-18

- 等线/DengXian → Carlito with DENGXIAN widths from zenodo-056's composite fonts (tool now reads Identity-H /W + ToUnicode and takes `PREFIX Name ids…`). zenodo-056 lines 19 → 110/133, still fails page 1 (ω, subscripts drawn with other fonts in Word). zenodo-018's header "LECTURE NOTE…" has w14 gradient fill + outline: Word's PDF has no text for it, Doc4Me keeps invisible text over outlines by design (not changed).

# Section vertical alignment on its last page · 2026-11-18

- A page ended by the next section's page break is aligned with its own section's `vAlign` (was the next section's): zenodo-081's bottom-aligned title page; Linux 138 → 139/219. docs/readiness/section-valign-2026-11-18.md. Windows run 44364914820 confirmed 138/219 for 2b0f9bd.

# Partial cell margins, lowered text, header-row start, Aptos Narrow · 2026-11-18

- zenodo-119 (not fixed): its Keywords line 3 fits in Doc4Me because Arimo kerns "ff"/"fi" ("Efficiency," 53.58 pt) where Word's Arial doesn't (54.58 pt; document has `w:kern 2`). Turning kerning off for Arimo/Tinos/Cousine stand-ins cost 256 matching lines and zenodo-012/-065; Word kerns other pairs. Needs Arial's own pair list, which we don't have.

- Missing tcMar/tblCellMar sides inherit (cell ← table ← style ← Word 0/5.4 pt) instead of 0; negative `w:position` lowers the line's ascent too; the header-rows-plus-first-line check adds the row's bottom cell margin when that row would split; Aptos Narrow → Roboto wdth 75 with measured widths (tools/aptos_narrow_pdf_metrics.py). Status unchanged at 138/219 Linux, lines 106,207 → 106,603. The margin fix alone failed govuk-052 and zenodo-154; the other three fixed what it exposed.

# VML WordArt watermarks, centred unwrapped boxes · 2026-11-18

- `v:textpath` shapes (Word watermarks) are now read as unwrapped, centred, unfilled text boxes (font from the textpath, size ≈ box width / (0.55 em × letters), colour = fill lightened by opacity); before, they were dropped (not shown, lost on save). Fixed a bug in my own `no_wrap` layout: centred lines were measured by their end x in the 4,000 pt measuring length and ended up outside the box. Corpus status unchanged (138/219); zenodo-060 −10 matching lines (Word drew its "DO NOT SELL" as paths). Text in turned boxes stays upright by design; a trial turning it gave only "T" in pdftotext for govuk-038 and changes editing of every turned box, not kept.
- Tried and rejected: first-row cell widths for autofit tables (68 tables; govuk-008 failed, several docs lost lines).

# Slashes, joined brackets, list-label text map · 2026-11-18

- No break after "/" unless whitespace follows; no opening bracket joined to the previous character starts a line. List labels pass per-character text ranges to the PDF writer so stand-in-width labels get ActualText ("12." no longer extracts as "1 2."). Linux: boundaries 136 → 138/219 (zenodo-096, zenodo-147), lines 106,155 → 106,217, no document worse. docs/readiness/slash-bracket-labels-2026-11-18.md.
- Remaining ideas from the split census: Word breaks long URLs character by character where they overflow (zenodo-054); "[ info ]" in zenodo-147 still extracts with spaces (body brackets in another run?).

# Percentage tables before Word 2013, Tahoma Bold widths · 2026-11-18

- Mode ≤ 14: a pct table width is of avail + default left/right cell margins (govuk-044's grid; 28 of 44 mode-14 pct tables match only this, none only the old reading; zenodo-108's centred tables match neither, −6 lines there). Tahoma Bold now uses TAHOMA_BOLD widths from the 2026-11-15 audit; govuk-044 no longer gains a page with them. Linux: boundaries 136/219 unchanged, pages 218/219, lines matching Word 105,998 → 106,155. docs/readiness/pct-tables-tahoma-bold-2026-11-18.md.
- Lead not taken: zenodo-149 needs Arial Nova Cond Light kerning (Word's line is 442 pt natural vs 432 pt room; kern pairs could be read from Word's TJ offsets, not done).

# Query marks, joined em dashes, list-style indents · 2026-11-18

- No break after "?"/"!" before alphanumerics (URLs; Doc4Me 15 splits, Word 0) or before an em dash joined to a word (7 vs 0); list-level indents yield to the indents of the style that brings the list (or a style based on it), not to inherited base-style indents (govuk-046 vs zenodo-067). Linux local: boundaries 135 → 136/219 (zenodo-012), lines matching Word 105,246 → 105,998 of 116,977 with no document worse. docs/readiness/query-dash-list-indent-2026-11-18.md.
- Tools in /home/user/work: firstdiff.py (first differing lines per failing doc), splits.py (where lines split inside tokens, Word vs Doc4Me), wrapstat.py (lines longer/shorter than Word per doc), eqlines.py A B (matching lines per doc between two corpus result dirs). Leads from wrapstat: zenodo-149 (37 lines shorter than Word), govuk-020/govuk-043 (lines longer: Cambria→Caladea docs).
- Not fixable: govuk-066 footer DATE field (Word printed 08/02/2024), zenodo-006 (header not in the .docx).

# Line breaks ignore zero-width markers · 2026-11-18

- Field/bookmark/comment/permission/control boundary objects are left out of the text used for break opportunities (like hidden text): a Zotero field end between ")" and "," no longer lets a line start with the comma. Linux local at 1168b3b + change: boundaries 133 → 135/219 (zenodo-166, zenodo-174), 219/219 open/roundtrip/stability, pages 218/219, no regressions. docs/readiness/field-marker-breaks-2026-11-18.md. Dispatch readiness-windows.yml after pushing (Windows run 44306256566 for 1168b3b was in progress when this was written).

# Fixed-table cell widths · 2026-11-18

- Fixed-layout tables take the first row's cell widths (one cell per grid column, all with point widths, any >1 pt off the grid); zenodo-115's table now has Word's equal columns and **zenodo-137 passes**. Linux local at b1d5a87 + change: 219/219 open/roundtrip/stability, pages 218/219, boundaries 133/219, no regressions vs 131 (bad5ec5); see docs/readiness/fixed-table-widths-2026-11-18.md. zenodo-115 still fails on its abstract's justified wrap ("authority," | "and").
- Official Windows: run 44294389579 (source b1d5a87, includes bracket/no-wrap) gave 132/219, 95.1/97.0. A duplicate run I dispatched on the report-only commit 490355d (44300450827) was cancelled. Dispatch readiness-windows.yml after pushing this change (owner allows sessions to dispatch it).
- Set aside: zenodo-006 (journal header not in the .docx: later edition), zenodo-126 and govuk-048 (extraction order on page 1). Remaining page-1 leads: govuk-021, govuk-038, zenodo-081, zenodo-119, zenodo-174, zenodo-115 abstract.

# Legacy table cell height · 2026-11-18

- The mode-14 govuk-045 Duration row retains its trailing empty paragraph in a populated cell even with hideMark. It restores the first postal address line to page 4 and passes all 23 boundaries. Isolated Linux comparison against 9eecfa2 yields 130→131/219 boundaries, one endpoint improvement across 2,941 pages, zero losses. See docs/readiness/legacy-table-cell-mark-2026-11-18.md.
- Fast-forwarded the concurrent bracket/no-wrap changes through da3799e. Integrated original verifier passes 133/219 boundaries, 216/219 page counts and 219/219 open/roundtrip/stability. Only govuk-045, zenodo-092 and zenodo-163 gain strict passes versus the 130 baseline. All438 source hashes match. Candidate CLI /dev/shm/hidemark-integrated-cli, corpus /dev/shm/hidemark-integrated-full, prepared build cache /dev/shm/hidemark-build.
- Next concrete finding: zenodo-144 p8 heading “IMPORTANT” inherits keepNext from its style, but the split row strands it when the next paragraph starts on p9. Scratch cantSplit on only that row restores all12 boundaries. Validate keeping adjacent paragraphs together during automatic row splits, including progress for groups larger than one page and authoritative explicit breaks. zenodo-049 has a separate footer-clearance difference between 0.05 and 0.10 pt.

# Bracket line breaks and unwrapped text boxes · 2026-11-18

- Scope: the corpus-breaks milestone left in this file (the form arc's Windows confirmation is done: run 44224006895). Two fixes from page-1 failures: no break between `)`/`]` and `(`/`[` with no space (Word keeps all 36 corpus pairs whole; zenodo-092), and text boxes with `wps:bodyPr wrap="none"` keep each paragraph on one line (`TextBody::no_wrap`, read/write/layout; zenodo-163 footer). Linux local, 438 hashes verified, pinned craft-fonts, release CLI: open/roundtrip/stability 219/219, pages 218/219 (zenodo-019 only), boundaries 129 → 131/219, no regressions. See docs/readiness/bracket-nowrap-2026-11-18.md and TSV.
- Leads not changed: govuk-068 (Word PDF Arabic presentation forms; changing the comparison would change the measure), zenodo-119 (Word starts page 2 with "Zusammenfassung:", vertical drift, cause unknown), zenodo-020 (extraction order). Other page-1 failures still to analyse: govuk-021, govuk-038, govuk-048, zenodo-006, zenodo-081, zenodo-115, zenodo-126, zenodo-174 (govuk-018 is in the section below, another session's lead).
- Next: run readiness-windows.yml on this commit for the official result (the owner dispatches it, or a session that can run workflows). Corpus work dirs in this session: /home/user/work/corpus-base-ce70748 (baseline PDFs), target/readiness/corpus (candidate), CLIs /home/user/work/base-cli and cand2-cli. No Release/Pages.

# Windows confirmation and next investigation · 2026-11-18

- Form arc `cedff9f` is committed/pushed. Windows run **44224006895** succeeded; generated report `ce70748`: counts **219/219**, boundaries **129/219**, **90 failures remain**, readiness **95.0% / 96.9% without 22 deferred**. Main also includes owner README/screenshot updates through `e2bf0ae`. Preserve them. No Release/Pages. Earlier snapshots below are historical.
- Revalidation complete: canonical CI6/6, native2621passed/0failed/1existingignored, assets152, layers20, WASM16. Log `/workspace/work/ci-windows-confirmation.log`. Owner `e2bf0ae` README-only deletion fast-forwarded after CI; tested code/assets unchanged. Fresh original-verifier results/reasons match all219 committed Linux rows.
- Continue govuk-018: its fixed nested phone row has 39 leading spaces before Fax. Nominal Verdana10 advances total154.96pt against154.8pt cell content, causing an extra line and pushing section C to page2. Reference draws Fax on the first line. Do not relax global fit tolerance from this single case; distinguish cell geometry, layout rounding and font-size quantization first. Scratch-only 38-space counterfactual removes12.153pt and restores C to p1; p2last/p4last still differ; see `docs/readiness/govuk018-phone-wrap-2026-11-18.md`.
- The environment restart erased previous RAM binaries/PDFs and relocated native caches. Current prepared CLI rebuilt from unchanged source: `/dev/shm/cedff9f-prepared-cli` (SHA256 `68fb15a87ae87bb79add51df6d9e784c11de065c7af0a5cc1d49ce2b5bc49198`, identical to the earlier validated binary; opt0, pinned craft-fonts, correctness only). Fresh original-verifier baseline is restored at `/dev/shm/cedff9f-baseline`, symlinked from `target/readiness/corpus`; see `/workspace/work/cedff9f-baseline-restored.log`. Rebuilt CLI hash matches the earlier validated binary; original verifier exits0 with only the existing3 page-count/89 boundary failures and no open/roundtrip/stability or extractor failures. All438 original input hashes reverified. Old RAM paths below are invalid. Native cache files now relocated under `/dev/shm/doc4me-native-rlibs`; retain while symlinks exist.

# Form fidelity follow-up · 2026-11-18

- Linux arc complete: modern nested-table tail marks retain caret access without growing the row; measured Verdana advances apply with Inter; PDF protects expanded metric-fitted words without intentional tracking, preserving native glyph Unicode maps. Existing krilla0.8.2 is vendored with scoped ActualText inside BT/ET; upstream licenses/notices and four CC0 profiles are retained/attributed. No new font program or proprietary asset.
- Full original verifier from a fresh work dir: open/roundtrip/stability219/219, counts216/219 unchanged, boundaries128→130/219. govuk-017 and zenodo-027 newly match; no previously passing document regresses. All438 inputs hash-verified. Exact2941-page comparison:23 endpoint gains,3 losses (Z096p8first,Z147p7/p11first), all in already-failing documents. Total per-page word-multiset distance improves534→36 and463→63, but Z147 exact multisets22→18; diagnostics are not pass criteria. Audit and all rows: docs/readiness/verdana-nested-table-2026-11-18.md and TSVs.
- Full canonical CI6/6:2621 native passed,0failed,1existingignored;assets145,layers20,WASM16. New width/table tests and PDF metadata test fail with fixes removed, pass restored. PDF tests cover native glyph-reader interoperability, tagged/untagged, BT/ET geometry, fit vs tracking, bounds/tabs/reversed maps. Log /workspace/work/ci-form-final.log. Criteria/accepted/excluded/deferred unchanged.
- Latest official Windows remains run44212279192 for df1ec6c, report2541e0c:219counts127boundaries,95.0/96.8,92failures. This confirmed manual-break ff0cf2a/df1ec6c, not the new form arc. Main advanced concurrently to1581bea with workflow-only changes; fast-forwarded intact. Commit/push form arc, then run readiness-windows.yml for Windows confirmation. No Release/Pages.
- Frozen final prepared CLI: /dev/shm/form-final-cli SHA25668fb15a87ae87bb79add51df6d9e784c11de065c7af0a5cc1d49ce2b5bc49198. Same pinned prepared fonts/profile0 as paired baseline; correctness only. df1ec6c workdir preserved /dev/shm/form-baseline-df1ec6c; final full workdir /dev/shm/form-full-verifier is symlinked at target/readiness/corpus. Final fast PDFs/results /dev/shm/form-final-results. Original1d48baseline remainsRAM via/workspace/work/corpus-baseline-1d48e99 symlink. Prepared build cache removed AFTER freezing/testing to recover RAM. Native third-party rlibs/rmeta moved atomically into /dev/shm/doc4me-native-rlibs and symlinked from target/debug; all links valid. Keep that RAM data while keeping the native cache. Source env script for Cargo (opt1,debug0,incremental0,serial). Corpus/fullCI logs and probe scripts stay/workspace/work. Rejected probe samples/TSVs retained/workspace/work/form-probe-evidence; most duplicate PDFs cleaned. Git CLI push works; GitHub connector hung on 1MB changelog.

# Manual-break height follow-up · 2026-11-18

- Scope remains corpus-breaks; milestone still incomplete. A manual break after text now ignores its own font metrics for the preceding line. Blank lines retain their own break size. Body/table, compatibility 14/15 and typing-command regressions pass; removing the production fix makes the layout regression fail.
- govuk-024 page 1 first 8 pt baseline improves 43.852→28.504 pt (Word 28.380); the 24 pt heading is still 53.051 vs Word 56.580. Page 2 Annex/Rule extraction-order failure remains. No invented Arial Black metrics or PDF-metadata patch shipped.
- Full local corpus with pinned prepared fonts: 438 source hashes verified; open/roundtrip/stability 219/219, counts 216/219, boundaries 128/219. Counts and document statuses unchanged. Exact Rust endpoint comparison on 2,941 corresponding pages finds one first-word gain (zenodo-044 p10), zero losses. Word-box geometry changes on 12 pages in seven documents. See docs/readiness/manual-break-2026-11-18.md and both TSVs.
- Read-only hhea/USE_TYPO PDF-descriptor probe gained three documents but regressed govuk-072 p18; font-size quantization did not repair it. Rejected. Criteria, accepted/excluded/deferred unchanged. Official Windows remains run 44078179776, source 019b0ff, boundaries127/219 (92 failures); local results do not replace it.
- Prepared candidate CLI: /workspace/work/linebreak-candidate-cli, SHA256 57159d9e1e0f66a4551eb07d32a41644ea91f3fabc0e62c3a7ee56ec8e70f6b2. Baseline PDFs moved intact to /workspace/work/corpus-baseline-1d48e99; candidate full results are target/readiness/corpus. Source /workspace/work/doc4me-env.sh for Cargo. Full CI uses opt-level1, serial tests, disabled incremental; corpus CLI uses matching opt-level0 for correctness only.
- Final resumed `cargo xtask ci` passed all 6/6 gates: 2,615 native passed, zero failed, one existing ignored; assets141, layers20, WASM16. Log: /workspace/work/ci-linebreak-resumed.log. Initial restart-interrupted run discarded. No Release or Pages deployment. Baseline PDFs currently live in /dev/shm/corpus-baseline-1d48e99 with a symlink at their former path; candidate PDFs remain in target/readiness/corpus. Duplicate probe PDFs and the read-only upstream clone were removed to free build space; audit logs retained.

# Corpus-breaks verifier and extraction-order follow-up · 2026-11-17

- Current official baseline is newer than the snapshots below: Windows run **44078179776**, source `019b0ff`, report `1559d7a`: **219/219 page counts, 127/219 boundaries**, **95.0% overall / 96.8% without deferred**. Still **92 boundary failures**; do not use the older 120/219 Linux result as the latest Windows status.
- Fixed a fail-open path in `xtask/src/readiness.rs`: failed `pdftotext` calls could become empty pages and falsely match. Require successful process exit and valid explicitly requested UTF-8 from both Poppler tools; require positive page count. Preserve page-count success when extraction fails, and identify the failed PDF side/page. Three regressions cover empty/partial failed output, launch errors, invalid UTF-8, invalid page counts, failed sides and valid blank pages. No change to scoring rules, corpus/accepted/excluded/deferred rows, renderer or fonts. No claim that earlier official scores were affected.
- Rechecked the five same-page-text cases with renderer `aba659a`, pinned prepared craft-fonts and ten verified input hashes: govuk-024, zenodo-020/030/050/070, **52 pages**, all page word multisets identical without NFKC. Six endpoint-order mismatches remain. All five still fail; no speculative layout/export patch. See `docs/readiness/extraction-order-2026-11-17.md` and TSV for endpoints and `-bbox-layout` leads. Next: compare block grouping/geometry in these cases; matching a word multiset is not a new pass criterion.
- Clean final Linux run: all **438 source hashes verified**, frozen prepared-font renderer from `aba659a` (binary SHA-256 `24aee41ff665564abea3e9b76c6ca38f1252261f406a8b665332633f00d63151`), final verifier, fresh result directory, `readiness --corpus --dry-run --diagnostics --jobs 2 --cli <frozen CLI>`. Open/roundtrip/stability **219/219**, page counts **216/219**, boundaries **128/219**; no missing files, extractor errors or work-folder errors. All 219 rows: `docs/readiness/verifier-corpus-2026-11-17.tsv`. Linux count failures: govuk-068 (16/15), zenodo-019 (10/11), zenodo-114 (72/71). This is a local baseline, not a renderer improvement or a replacement for the official Windows result. Reports were not rewritten. Initial HTTP 429/download gaps were resolved; runs interrupted by ENOSPC were discarded.
- Real Poppler smoke check of the final comparison functions: a generated blank page matches itself; missing PDFs and an out-of-range page extraction fail; all five audited documents retain their original strict boundary failures. Final `cargo xtask ci` passes **6/6**: **2,613 native passed, one existing ignored; assets 141, layers 20, WASM 16**. Keep the project's opt-level 1 for performance tests; this run used `CARGO_INCREMENTAL=0`, debug info 0 and `RUST_TEST_THREADS=1` on the constrained machine. No timing threshold or ignored-test list was changed.
- Workspace CI exposed a stale engine timestamp test: `Comment::date` deliberately uses Word's local wall-clock-with-Z representation, whereas `date_utc` is UTC. The old equality assertion failed under UTC+7. Updated the test to validate the two creation-time windows with the observed local offsets, including clock-read/second and daylight-saving boundaries. It passes under separate `TZ=Asia/Bangkok`, `TZ=UTC` and `TZ=America/New_York` processes; no date-writing/rendering code changed.
- Scope remains corpus boundaries. No Release or Pages deployment; commit/push to main only. Shared `storytold/craftrules/AGENTS.md` returned 404 through both Git and the GitHub connector; local AGENTS.md and CLAUDE.md were followed.

# Corpus-breaks initial-header follow-up · 2026-11-17

- Scope remains corpus-breaks only; still incomplete. AutoFit already pushed as e4277bf (Linux 119/219 boundaries). The new header/body prefix fix makes govuk-052 match all nine pages and boundaries: 120/219, still 99 failures. All 219 page counts unchanged (213/219); no previously passing document/page is newly failing. Endpoint words within existing failures change in zenodo-108 and zenodo-173; z173 page15 loses a formerly matching first word, documented in the audit.
- Shared header-prefix measurement also keeps captions with the table; no-body/oversized cases retain old paths. CI 6/6 (2,600 native passed, one existing ignored; layout 358, assets141, layers20, wasm16). See docs/readiness/table-header-prefix-2026-11-17.md and TSV. Windows confirmation pending; do not publish Release or Pages.
- Read-only bag-of-words triage on the 94 same-page-count failures found five with identical NFKC word multisets on every page (govuk-024, zenodo-020/030/050/070). They remain failing under the original first/last-word criterion. govuk-066's footer date is a DATE field, not evidence by itself of different source editions.

# Corpus-breaks continuation · 2026-11-17

- Scope from owner: complete only corpus-breaks/page boundaries. Still incomplete.
- Baseline `ae83b65` / engine `f352d1a`: verified 438 corpus hashes, pinned prepared fonts; Linux 212/219 page counts and 118/219 boundaries. At that baseline Windows was 214/219 and 118/219.
- Implemented floating text-box `a:spAutoFit` import, round-trip and measurement; govuk-002 page 24 now includes its final line and every page boundary matches. Full first corpus comparison 119/219 boundaries, no page-count change, no other row changes. See docs/readiness/textbox-autofit-2026-11-17.md and comparison TSV. Fixed and linked boxes unchanged; inline resize and VML-only AutoFit remain unsupported.
- Focused tests pass (layout 352, DOCX unit 98, text-box integration 5); final release corpus verification confirms open/round-trip/stability 219/219 each, counts 212/219, boundaries 119/219. Full CI passes all 6 steps (2,594 native tests, one existing ignored; assets 141, layers 20 crates, WASM 16 crates). Final fmt/clippy passes after the linked-chain guard. No linked text boxes occur in the corpus.
- Integrated main through `5222566`, preserving both concurrent table fixes. Combined CI passes 6/6 (native 2,598, existing ignored 1; layout 356). Full final pagination run: 213/219 pages and 119/219 boundaries, no conversion errors. Only govuk-067 gains a page-count match from the later main fix; AutoFit still adds only govuk-002 boundaries. Combined comparison TSV is linked in the audit. Still 100 failing corpus-breaks documents.
- Next case investigated: govuk-024 page 2 has complete text but Rule 15.18 / Annex A reading order differs. Arial Black substitutes Carlito Bold. Word PDF Arial-Black descriptor ascent1083/descent307 per1000 vs Carlito750/250; these are PDF descriptor metrics, not enough evidence for Word line spacing. No font/position patch applied.
- Do not publish a Release or deploy Pages. Keep criteria/accepted/excluded/deferred unchanged.

> **Standing instruction from the owner (2026-11-16):** after finishing work, do **not** publish a GitHub Release or redeploy the GitHub Pages Web App; the owner does both. Commit/push to `main` as CLAUDE.md requires, and stop there.

# Resumed 2026-11-16: the not-yet-analysed group first (owner)

- Owner order: finish the not-yet-analysed group (govuk-053, zenodo-022, -039, -049, -053, -069, -081, -123, -125, -173), then the large/unknown group.
- Done: zenodo-125 (continuous section marks take no room; spacing collapses at a continuous section's top).
- Done (Linux 203→207 pages, 104→105 boundaries, no regressions; `docs/readiness/top-space-comparison-2026-11-16.tsv`): space before is dropped at the top of a page opened by a page break or a section break **in every compatibility mode** (the old "mode 15 keeps it, govuk-046" note was wrong: Word's govuk-046 page 5 heading also sits at the margin) → **zenodo-022, -069, -081, -123 match**, zenodo-102 boundaries match; a super/subscript paragraph mark keeps the full line height (zenodo-022's diagram of empty superscript paragraphs). zenodo-081's "page 7 lower by 10.2 pt" was this rule, not the header.
- Readiness Windows run 43902313637 (source `42b3c46`) failed at the README step: `tools/readiness_readme.py` looked for "คลังเอกสาร 220 …" with the new count 219; fixed (count-agnostic patterns). Re-dispatch the workflow to get official numbers.
- Done (Linux 207→208 pages, 105→110 boundaries, no regressions; `docs/readiness/space-cambria-comparison-2026-11-16.tsv`): justified spaces shrink up to 22% but not below 1.6 pt (`space_shrink_room`; data: 21.95% fits at 11 pt TNR zenodo-039, 23.0% refused at 12 pt Arial govuk-072, 20.6%/21.4% refused at 8 pt zenodo-165) → **zenodo-039 matches**; Cambria drawn with Caladea takes Word's Cambria widths (10 PDFs agree) and Caladea's `tnum` digits; keep-with-next counts a short widow-controlled next paragraph whole (zenodo-090); tall page-anchored floating tables start at their own offset (zenodo-053).
- Official Windows run 43953675813 (source `5965b5c`): pages 209/219, boundaries 114/219, 94.2% / 96.1%.
- Official Windows run 43915137821 (source `11deeb1`): pages 208/219, boundaries 106/219, 93.9% / 95.8%. zenodo-123 matches on Linux but not on Windows (only Arial → Arimo on both; suspect ●/■/○ bullet fallback to Windows system fonts; unverified).
- Done: Calibri Light drawn with Carlito takes Word's Calibri Light widths, bold included (synthetic bold keeps Light widths) → zenodo-049 52→45 (Word 43); its line breaks now match, the rest is the footer lead (Word's last line ends 0.5 pt above the footer top).
- Done: a keep-with-next run that already started on the page moves only if the whole run fits a page → **zenodo-128 matches** (large group).
- Done: the recto filler page also applies with mirror margins and to one-column Next Column breaks → **zenodo-114 matches** (large group).
- zenodo-157 now 7 pages (2026-11-17): the justify overhang rule (MAX_WORD_OVERHANG 0.35, MAX_SPACE_SHRINK 0.25) is applied; it no longer breaks zenodo-039/-165 once list labels add only their unscaled ascent. Earlier notes: zenodo-157 (6 vs 7): space before kept on a split row's continuation and picture lines in cells get the multiple-spacing share (both committed); page 5 now within ~1 pt of Word. Remaining cause: justified line breaking. Word takes an overflowing word in by shrinking spaces only if it sticks out by at most ~0.35 of (word + space before) and the shrink stays within ~25% (fit over 671 lines of zenodo-003/-039/-157/-165; cap-only rule: 141 disagreements, this rule: 10 without the kerned zenodo-165). Implemented as `overhang_fits` + MAX_WORD_OVERHANG 0.35, MAX_SPACE_SHRINK 0.25, MIN_SPACE 0 (patch kept out of the repo): line ends matching Word 69,787 -> 71,450 of 75,526, boundaries 117 -> 121, zenodo-157 7 pages, but zenodo-039/-053/-165 gain a page from 1-2.5 pt vertical drift (z039: second empty paragraph misses page 5 by 1.1 pt; z165: Table 1 row 'Maximum' misses page 6 by ~2.5 pt). Lead for that drift: Word seems to round multiple-spacing line heights to 1/600 in (0.12 pt): z165 TNR 8 pt at 1.15 steps 10.56 = 88 units (ours 10.579), z039 TNR 11 pt at 1.15 steps mostly 14.52 = 121 units (ours 14.546). Verify that before applying the overhang rule.
- Line-height rounding experiment (2026-11-17, not applied): Word's line pitches sit on a 0.12 pt (1/600 in) grid. In zenodo-093 (TNR 12, 1.5), govuk-053 (Arial 12, 1.15) and zenodo-157 (TNR 11 single) two pitches alternate (20.64/20.76, 15.84/15.96, 12.60/12.72) and their mean equals the computed height, i.e. Word rounds positions, not heights. Flooring every multiple-spacing height to 0.12 pt: pages 214 -> 209; flooring only multiples != 1.0: pages 214 (govuk-053 matches at 49, zenodo-093 33 -> 32), with the overhang rule pages 214 (+govuk-053, +zenodo-157, -zenodo-093, -zenodo-165) and boundaries 117 -> 120. zenodo-039 (docDefault 259 auto, Calibri characters among TNR text) and zenodo-165 (TNR 8, 1.15, always 10.56) don't follow the mean-exact pattern; their real rule is still unknown. Current state with only the overhang rule: pages 213 (+zenodo-157, -zenodo-039, -zenodo-165), boundaries 121, line ends 69,809 -> 71,472.
- zenodo-053 (Windows 59 vs 58): Linux matched only by two errors cancelling (pages 31-57 one page off). Fixed: cells grow for wrapping picture floats, too-tall page-positioned floating tables start at tblpY on a fresh page, automatic tables whose minimum content cannot fit keep the file grid. Now 9 pages start differently from Word (was 31); confirm on Windows. Remaining small differences: pages 6, 7, 10, 14, 20, 26, 30, 37, 48 (row split points).
- Done: Tahoma's own 1.207 em line with Arimo → **govuk-038 matches**, govuk-061 boundaries. Tahoma widths from 7 PDFs regress govuk-044 (10→11) again and don't help govuk-067 (15 vs 16, its lines still wrap differently); not applied. Note: the Windows readiness also substitutes open fonts (Tahoma→Arimo, Cambria→Caladea, Calibri Light→Carlito), so these font fixes count there.
- govuk-067 now 16 pages (2026-11-17): a too-tall floating table clears a floating table placed within 1 pt below its start. Boundaries still differ from page 7 (one page behind until page 15).
- Done: Tahoma regular widths (not bold) with Arimo; govuk-067 pages 1–4 now match. govuk-067 lead: page 5's in-flow table B overlaps floating table A placed 1 twip below B's start (`below` only counts floats at or above y); making floats inside B's first row count fixes page 5 but gives 17 pages (Word 16) because later floating tables with page breaks between diverge; not applied.
- zenodo-123 Windows-only: the diagnostic step showed Windows ends page 2 one line early; its emoji (✅ 🚫…) fall back to Segoe UI Symbol there, whose taller line Word doesn't use (Word steps those lines like plain Arial). Fixed: emoji/dingbats drawn by a fallback face keep the requested font's line. Confirm on the next Windows run.
- zenodo-049 44 vs 43: the footer 'lead' was really a 12 pt page-break run making the last line taller (fixed: break clusters after text don't count for line height). Next: page 20's floating 'INDICE' box sits ~18 pt lower in ours (216 vs 198.6), so the wrapped lines beside it differ.
- zenodo-157: picture 3 is anchored at the END of 'La capacidad…' (offset 9 pt from the paragraph); Word keeps 3 full-width lines on page 2 and puts the picture at page 3's top with the rest beside it. The rule for how many lines stay is not derived yet.
- govuk-053 now 49 pages (2026-11-17): list labels taller than the text add only their unscaled ascent excess (the body there has 1.15 lists). Earlier analysis kept below. govuk-053 (50 vs 49), more data (2026-11-17): Word body text reaches yMax 754.8 typically (max 759.3) vs ours 749.0 (751.5); Word page number glyph yMax 786.7 vs ours 780.8. Footer = framed PAGE paragraph (wrap around, vAnchor text, y=1 twip) + two empty Footer paragraphs (Arial 12, 12 pt before, single). Dropping the first empty paragraph's space before (39.6 pt footer) fits the body evidence but puts the page number ~6.4 pt too high; no single rule fits both yet. Earlier note: the footer (framed PAGE paragraph + two empty Footer paragraphs, each 12 pt before from Normal + 13.8 pt line) measures 51.6 pt in ours, so the body ends at 754.95 (A4 margin 769.95). From Word's page-number glyph (775.56 vs ours 767.39) Word's footer is ~43.4 pt; no spacing rule tried reproduces it exactly (first-paragraph space before dropped = 39.6). Not changed.
- Done: AutoFit ignores turned (vertical) cell text when widening a column → **zenodo-049 and zenodo-053 match** (the 'INDICE' btLr column).
- zenodo-173 now 25 pages (2026-11-17): split-row rules applied (bottom margin when a split is possible, widow/orphan in cells, per-cell continuation tops, no space before after an orphan move, 0.5 pt top room) together with the zero-width-mark line-break fix that corrected zenodo-103 (which the rules would otherwise have cost a page). Linux page-count failures left: govuk-026 (accepted), zenodo-045 (excluded), zenodo-019 (Linux only). Earlier: zenodo-173 24 pages (2026-11-17): a row's largest top/bottom cell margins frame its tallest content (rows were 8 pt short). Remaining: page 6's row 'projet_titre_fr' that Word moves whole. Reserving the row's bottom margin when a split is possible (split_row(room) is Some, then split_row(room - margin)) gives zenodo-173 25 pages but zenodo-123 39 (38); adding widow/orphan control inside split rows matches zenodo-072's 5/2 split and zenodo-123 but costs zenodo-022/-072/-103 a page; per-cell continuation tops lose zenodo-157. Patches in the session scratchpad, not applied.
- zenodo-173 split-row rules (2026-11-17, not applied; patch kept in the session scratchpad as split-row-rules.patch, env knobs D4M_BM/WC/PC/OB/CT): together they give zenodo-173 25 pages, zenodo-022/-072 kept, govuk-072 boundaries ok, zenodo-053/-123/-072 boundaries later, but zenodo-103 30 (29). Evidence for each: (1) a split reserves the row's largest bottom cell margin, only when the row would split without it (zenodo-173, zenodo-123); (2) widow/orphan control inside split rows (zenodo-072 splits a 7-line paragraph 5/2, zenodo-123 moves a 3-line row whole; Word applies it although none of these files has w:widowControl, matching Doc4Me's default); (3) each cell's continuation starts at the continuation top (zenodo-072 page 8), the continuation row then as tall as its lowest content, only in rows with no moved pictures (zenodo-157); (4) a paragraph moved by the orphan rule keeps no space before at the top; (5) the continuation keeps 0.5 pt, not 2 pt, of top room (zenodo-022 page 16; neutral alone). zenodo-103's base passes by luck (its rows are off by up to 35 pt from page 4); with the rules its page 7 matches Word to 1-2 pt but row [41] splits at the page 7/8 boundary where Word keeps it whole. Next: find why row [41] doesn't fit whole.
- zenodo-173 (2026-11-17): page 6's moved row is 'projet_titre_fr' (example cell 4 lines, definition 2). On pages where Word does split a row its last glyph ends <= 514.3 (bottom 524.06); here Word's second line would end ~523.4 and the cells have 4 pt bottom margins (tcMar 80). Reserving the row's bottom cell margin in split_row (like the border) did not change zenodo-173 (still 23) and broke zenodo-165's boundaries (ok -> p15); reverted.
- zenodo-173 page 6 geometry (2026-11-17): Word's moved row 'projet_titre_fr' measured at the top of page 7 (top border 95.28, example lines from +6.74 at 12.24 pt pitch, row 57.4 pt) would end on page 6 at about 521.0 (example, 2 of 4 lines) and 523.6 + 0.5 border (definition, 2 lines with 4 pt tcMar) against a bottom of 524.1 (11900-twip page, 1418 margin): within ~0.02-0.4 pt, so the deciding rule can't be read from this row. Split rows elsewhere in the document end at 511.4-520.3.
- zenodo-173: Word does split rows in this document (pages 10–12, 14, 17, 20, 21 start with a continuation), yet moves page 6's next row whole though it starts ~3 pt *lower* in ours and still fits: not a room/border/min-height rule. Word's page 9 ends at y 311 with ~200 pt left; no merged cells, no keeps, no cantSplit. Unresolved.
- zenodo-053 (59 vs 58): page 9 now starts right; Word keeps a long URL whole below a floating picture in a cell, ours breaks it after "?" beside the picture; more small differences on other pages.
- Done: a row doesn't split into less room than its at-least height (zenodo-173 page 5; govuk-056, zenodo-009, zenodo-161 boundaries now match). zenodo-173 still 23 vs 25: page 6's next row (min 11.7 pt, cells without widowControl) is moved whole by Word with ~36 pt left though two lines fit; rule unknown.
- Lead, not fixed (row splits): zenodo-123 (now matches via the top-space rule) and zenodo-173 (closer) when a row split never leaves one line of a multi-line cell paragraph alone (widow-safe cut), and zenodo-009/-091/-161 boundaries match, but zenodo-072 (9→10) and zenodo-069 regress; none of the four documents has `w:widowControl` (ECMA-376: absent means off, while Doc4Me defaults widow control on), so the real rule for zenodo-173's row (trHeight 589 atLeast, moved whole by Word) is still unknown. Reverted.
- Lead, not fixed: zenodo-081 page 7 (first page of a landscape section whose two-line header reaches below the 56.7 pt top margin) starts 10.2 pt lower than Word (heading at 106.5 vs 96.3), so the last "1 2 3 4" row of page 7 moves on and adds a page.
- Not analysed yet: none of zenodo-022/069 (fixed); zenodo-039 see above.
- Lead, not fixed: zenodo-049 and govuk-053 pass on Linux if multiple-spacing leading may run past the top of a footer that pushes the body up, but WordCraft 627ded32's 38 Word probes say a footer above the margin needs whole line boxes (test `multiple_spacing_can_put_leading_in_the_margin_but_not_in_a_tall_footer`), and zenodo-109 regressed; reverted. zenodo-049's last line ends 0.5 pt above the footer top in Word: look at where Word puts the 1.079 multiple's extra leading.

# PAUSED · 2026-11-16 — page-count work on hold by the owner

- Owner decisions: **zenodo-045 excluded**, **govuk-026 accepted** (`check` = `accepted`: its pagination counts as passing; `xtask/src/readiness.rs` `record_document`). The next Windows readiness run will count 219 documents with govuk-026 passing (last run 43886112079 counted 203/220 before this change).
- **On hold (owner, 2026-11-16): the remaining page-count failures**, both the large/unknown ones (zenodo-157 cell row re-layout for a picture anchored to a cell's last line; zenodo-128 empty Heading 1 line height; zenodo-114 blank page between vertically centred sections; govuk-038/govuk-067 Tahoma, not reproducible on Linux) and the ones not yet analysed (govuk-053, zenodo-022, -039, -049, -053, -069, -081, -123, -125, -173). Do not resume them unless the owner asks.
- After this the owner asked for one GitHub Pages Web App deployment from the latest `main`, then to stop. (This overrides the earlier "don't deploy Pages" instruction for this one request only.)

# Page-count work · 2026-11-16 (user: continue until all 220 match)

- Linux prepared-font comparison: CLI built with `CRAFT_FONTS_DIR=/home/user/work/craft-fonts CRAFT_FONTS_REQUIRED=1 CARGO_TARGET_DIR=/home/user/work/target-prepared cargo build --release -p doc4me-cli`; `JOBS=8 python3 /home/user/work/scripts/compare.py <repo> <base-cli> <cand-cli> <outdir> <tsv>`; helpers `drift.py`, `lines.py`, `imgpos.py` (image placements from PDF content streams) in `/home/user/work/scripts`. Latest candidate CLI `/home/user/work/mark-cli` (= `main` layout).
- Fixed and pushed (each with test, CI 6/6, 220-doc comparison, no regressions): floats kept inside their table cell (`e0aa7c1`, zenodo-157 pages 3–4 now match); list's first item keeps HTML auto space before (`b111159`, **zenodo-083 now matches**); numId 0 drops the style's indents (`e37c9c1`, zenodo-128 pictures at the margin); a bookmark before an oversized picture adds no empty line (this commit; zenodo-045 22→21, zenodo-128 23→22). Linux: pages 199/220, boundaries 104/220.
- Later fixes: no HTML auto spacing at a cell's first/last paragraph (govuk-018, zenodo-003, zenodo-021 match), Mac-charset (4D) fonts drawn with altName (zenodo-022/069 one page closer) — `eaff5f5`; hideMark cells keep the space before the hidden mark (govuk-068 regression on Windows run 43880197807 fixed). Official Windows run 43880197807 (source `eaff5f5`): **pages 202/220**, boundaries 104/220, 93.5% / 95.4%. Then run 43886112079 (source `61dfe51`): **pages 203/220**, boundaries 104/220, **93.6% / 95.5%**; govuk-068 passes again. 17 page-count failures left: govuk-026, govuk-038, govuk-053, govuk-067, zenodo-022, -039, -045, -049, -053, -069, -081, -114, -123, -125, -128, -157, -173.
- Reference mismatches for the owner (not excluded): govuk-026, zenodo-045 — `docs/readiness/reference-mismatch-2026-11-16.md`.
- Open leads: zenodo-157 (a picture anchored to a cell paragraph's last line runs past the page bottom; Word moves that line, with widow control, to the next page and puts the picture at the new page's top: needs row-splitting re-layout); zenodo-128 (empty Heading 1 paragraphs: Word's line ≈25.1 pt for 20 pt Aptos Display at 1.1625 auto spacing, ours differs in the prepared build so the second picture of page 3 moves on); govuk-067 (Tahoma: Linux draws Arimo, 1.149 em vs Tahoma 1.207 em; Windows also fails 15/16, not analysed there); zenodo-019 fails on Linux only.

# Status · 2026-11-16 (builds named by commit; pasted lists in Thai digits)

- **Owner instruction (2026-11-17, supersedes the About part below):** every build workflow takes an optional `version` input, baked in as `DOC4ME_VERSION`; `build_info::version()`/`describe()` show it in the About dialog (`Version {version} ({build})`, restored to all 90 catalogs from `176ca2a^`), `--version` (`1.2.0 (abcdef12)`), MCP `serverInfo.version`, user agent, Windows VERSIONINFO and the installers' numeric fields. A build without a version keeps the commit-only behaviour below. `.github/workflows/release.yml` (manual) validates the version, resolves `main`/commit to one SHA, calls the chosen build workflows (`workflow_call`), checks every artifact's `SOURCE_COMMIT.txt`/`VERSION.txt`, writes `SHA256SUMS.txt`, and creates tag `v<version>` + Release with `tools/release_notes.py --unreleased-as`. The owner's standing instruction still holds: don't run it (or publish Releases/Pages) unless asked. Source commits older than this change ignore `DOC4ME_VERSION` in the app (About/`--version` show the commit only), though their packages' numeric version fields still take it.
- **Owner instruction (2026-11-17):** download file names follow a choice: the build workflows' `file-name` input (`commit` default, `version` needs a version) and release.yml's `file-name` (`version` default, `commit`) set `DOC4ME_FILE_NAME`; `packaging/env.sh` exports `LABEL` (commit or version, validated as X.Y.Z[-pre]) and `package.ps1` `$Label`, used in every package, archive and in-archive folder name. release.yml has no `prerelease` field any more (10-input limit): a version with a `-pre` part is published as a pre-release. Source commits older than this change ignore the choice (commit names).
- **Owner instruction (2026-11-17, supersedes the About parts above):** the About dialog follows the file-name choice, also for builds of older commits: `build_info::about_line()` gives `Version {version} ({build})` with `DOC4ME_FILE_NAME=version` and a version, else `Commit {commit}` (new string, 90 catalogs), or `development build` without a commit. The Pages workflow sets `DOC4ME_FILE_NAME` to `version` when a version is typed, else `commit`. Every build workflow sparse-checks out `tools/about_label.py` + `crates/ui-egui/src/i18n/*.tsv` from `github.workflow_sha` and runs it on the selected source before building: sources without `pub fn about_line()` get the lines between the About heading and its blurb replaced by self-contained code reading the same variables, plus the three strings appended to their catalogs; it fails the run if the About code isn't recognised and preserves CRLF. Older sources' `--version`/MCP and file names are unchanged.
- **Owner instruction (2026-11-17, supersedes the About/file-name parts above):** build, Pages and Release workflows let the owner show or hide the version and the commit separately: `about` input → `DOC4ME_ABOUT` (`build_info::about_line`: `version-commit` → `Version {version} ({build})`, `version` → `Version {version}` (new string, 90 catalogs, derived from the ({build}) row), `commit` (default) → `Commit {commit}`, `none` → nothing); `file-name` input → `DOC4ME_FILE_NAME` (`commit`, `version`, `version-commit` = `<version>-<commit>`, `none` = no label: `doc4me-windows-x64.msi`; scripts write `doc4me${LABEL:+-$LABEL}`, package.ps1 `$LabelPart`). Release defaults: file-name `version`, about `version-commit`; its three Windows booleans became one `windows` choice (`x64` default, combos, `none`) gated by `contains()` and validated in `prepare` (10-input limit). `tools/about_label.py` now detects native support by `option_env!("DOC4ME_ABOUT")` and also replaces the 71b1731-era `about_line()` match. Sources whose env.sh predates `version-commit)` get version/commit with a warning; sources predating DOC4ME_FILE_NAME keep their own names (warning).
- **Owner instruction:** content, images and workflows no longer refer to version numbers; refer to the commit in use. The About dialog shows **no version and no commit** (owner's later instruction; its `Commit {commit}`/`development build` strings removed from all 90 catalogs); `--version`, MCP `serverInfo.version`, user agent and Windows VERSIONINFO strings show the short commit (`crates/engine/src/build_info.rs`, `dev` without `DOC4ME_BUILD_SHA`). Package files are `doc4me-<commit>-…`; numeric installer fields (MSI, plist, DEB/RPM, AppStream) still use Cargo's version because the formats require numbers. CHANGELOG headings for new releases are `## [<commit>] - date`; released headings stay. `tools/release_notes.py` takes a commit. Historical revision-history rows and the older handoff snapshots below keep their version numbers (history).
- README.md/README.en.md name **no version number and no commit** (owner's instruction); `tools/readiness_readme.py` no longer writes the source commit there.
- **GitHub has no Releases and no tags now** (`list_releases`/`list_tags` empty on 2026-11-16; the owner removed them). Local clones may still hold stale `v1.1.3`/`v1.1.4` tags: never push them.
- AutoFormat: lists pasted from elsewhere (Paste Special HTML/RTF) take Thai digits while the switch is on, keep their format when off (`f101a79`).

# Status · 2026-11-16 (after AutoFormat new numbers)

- AutoFormat now also makes new lists and inserted page numbers in Thai digits (`952f3a0`, user chose “all Thai”); CI 6/6 (2,566/1 ignored), language audit incl. ignored 65/65, cargo audit exit 0.
- Official Windows: run 43804632296 (`093eaba`, report `15584f5`): pages 199/220, boundaries 104/220, 93.4% / 95.3%; live summaries refreshed.
- **The owner deleted `.github/workflows/windows-x64-installer.yml` in `247aa7a` and, when asked, chose not to release now.** Do not restore it or publish a release unless newly requested. Published v1.1.4 remains at `e5d82d6` (refresh #2); later main commits (Garamond, AutoFormat list/page-number changes) are not in it.
- GitHub Pages redeploy dispatched from `952f3a0` (run 43834113425). An earlier dispatch with a mistyped SHA (run 43834072758) was cancelled.
- Page-count leads unchanged (govuk-053, zenodo-125, zenodo-003, zenodo-069/022 and 15 unanalysed rows).

---

# Status after v1.1.4 refresh #2 · 2026-11-16

- **Release v1.1.4 refreshed** by run 43802874365 (update_release, commit `e5d82d6`): same release ID 445039704; the workflow moved tag v1.1.4 (lightweight now) to `e5d82d6` because this session's git proxy refuses tag pushes (HTTP 403). Assets verified: MSI sha256 `39cd0066…5d41`, ZIP `bd47c746…58ad` match the API digests; `doc4me.exe` embeds build SHA `e5d82d6…`; CLI reports 1.1.4; notes 121,381 bytes, six versions newest-first, include “รอบที่ 2”.
- **GitHub Pages** redeployed by run 43802877585 from `e5d82d6`; https://veevie55.github.io/doc4me/SOURCE_COMMIT.txt = `e5d82d6…`.
- **After the release (main only, not in the release):** Garamond widths/line `093eaba` (Linux 197→198), AutoFormat “1. ”→Thai numbered list `2e9a1e8`. Windows readiness run 43804632296 on `093eaba` (Overpass + Garamond) was still running at handoff; its workflow now also rewrites README numbers (`tools/readiness_readme.py`). Refresh live summaries/audits (Overpass, Garamond) from its report.
- **Open leads:** govuk-053 (Word lets the last body lines reach into a tall footer whose top is set by framed page number + Footer paragraphs with Normal's 12 pt space before; conflicts with `multiple_spacing_can_put_leading_in_the_margin_but_not_in_a_tall_footer` — needs evidence before changing), zenodo-125 (floating figure placement), zenodo-003 (20% justification cap edge), zenodo-069/022 (Word drew Calibri for Overpass).

---

# v1.1.4 refresh #2 + AutoFormat + page counts · 2026-11-16

User authorized (this session): refresh existing release v1.1.4 and redeploy the GitHub Pages Web App from latest code; add AutoFormat (Thai digits) switch beside AutoSave; keep README readiness numbers in sync; continue page counts.

Done: AutoFormat (`file.autoFormat`, `Prefs::auto_thai_digits`, title-bar switch + Alt Z F; Thai label via i18n context `titlebar`; tooltip reuses "Convert to Thai Digits"). Page-count arcs pushed: Cambria Math line (Windows 193), tab/box spacing (Windows 195), Symbol/Wingdings bullets (Windows 197, run 43796305956, report `3fca7cd`, 93.3%/95.2%), Overpass line (Linux 196→197, not yet Windows-measured). `tools/readiness_readme.py` + workflow step keep README.md/README.en.md numbers equal to docs/readiness.md. Gates on `3668e3c`: CI 6/6 (2,564/1 ignored), language audit incl. ignored 65/65, cargo audit exit 0 (5 unmaintained warnings).

Release procedure: CHANGELOG Unreleased moved under `[1.1.4]` (“รอบที่ 2”); push; move tag v1.1.4 to the release commit; dispatch `windows-x64-installer.yml` on main with update_release=true and commit=<full SHA>; dispatch `build-web-pages.yml` with ref=<same SHA>; verify assets/notes/Pages. Then dispatch readiness Windows to measure Overpass.

Open page-count leads: zenodo-003 (justified line needs 20.7% space shrink vs 20% cap; Arimo widths +0.36 pt; Windows uses real Arial — unverified); zenodo-069/022 (Word drew Calibri for Overpass via altName; author lacked the font); govuk-038 now 45 vs 46 on Linux. Working dir `/home/user/work` (CLIs: symbol-cli=df4f160, overpass-cli=3668e3c code; scripts compare/drift/lines/words.py). Disk is tight: delete `target/debug/deps` executables before full CI.

---

# Page-count work · tab and box spacing candidate · 2026-11-16

Pushed after Cambria Math line (`6d4976c`, Windows run 43792165925 dispatched on it): a tab no longer raises a line holding text or an equation; `frame_box` (table cells, text boxes, frames) separates paragraphs by the larger of space after/before unless `add_para_spacing`, as the body already did. See [audit](docs/readiness/tab-box-spacing-2026-11-16.md). Linux vs `6d4976c`: pages **192→194/220** (govuk-068 16→15, zenodo-091 12→11), boundaries stay 102; govuk-046 matches further, zenodo-069 34→33 (Word 24), **zenodo-022 27→28 (Word 20)**; no previously passing row regresses. Full CI 6/6 in one run (2,560 pass / 1 existing ignored).

Next: refresh docs with Windows run 43792165925 when it completes, dispatch Windows on this commit, continue remaining page-count cases (zenodo-091 boundary now page 4; zenodo-156/115/125 small docs).

---

# Page-count work · Cambria Math line candidate · 2026-11-16

Current request: continue page counts; commit/push main without asking. Official Windows baseline remains run 43785505707 (engine `55d0c37`, report `7e50321`): 192/220 page counts, 99/220 boundaries, 93.0% / 94.9% without 22 deferred.

This arc: letters Caladea lacks (Greek) in Cambria/Cambria Math runs now keep Cambria's 1.172 em line instead of the fallback face's (Sarabun 1.853 em); installed Cambria Math uses Cambria's line too, matching Word's PDF of zenodo-082. See [audit](docs/readiness/cambria-line-2026-11-16.md). Linux prepared-font comparison against baseline `7e50321`: pages **191→192/220** (only zenodo-082 6→5, Word 5), boundaries **99→102/220** (zenodo-032/073/082 newly pass; zenodo-056/057 later first failure); no regressions; 440 hashes verified. CI: fmt/clippy/test (2,558 pass / 1 existing ignored)/assets 141/layers 20 pass; wasm passed 16/16 on rerun after installing the wasm32 target.

Working inputs (this container only): `/home/user/work/craft-fonts` (pinned `c1db6d1`, prepared), corpus in `corpus/readiness`, baseline CLI `/home/user/work/baseline-cli` (`7e50321`), candidate `/home/user/work/target-prepared/release/doc4me-cli`, scripts `/home/user/work/scripts/{compare,drift,lines}.py`, logs `/home/user/work/cambria-{ci,wasm,comparison}.log`.

Next: after push, dispatch Windows readiness to confirm; then continue remaining page-count failures (start with small docs: zenodo-156, zenodo-115, zenodo-091, zenodo-125, zenodo-067 using `drift.py`).

---

# Page-count work · Windows confirmed 192/220 · 2026-11-16

Current request: continue page counts only. Windows [run 43785505707](https://github.com/veevie55/doc4me/actions/runs/43785505707) measured engine `55d0c37` (restart-recto fix), report `7e50321`: **192/220 page counts** (only govuk-036 newly matches), **99/220 boundaries**, **93.0% / 94.9% without 22 deferred**; 28 page-count and 121 boundary failures. Row-by-row diff of report failure lists `b6e1076`→`7e50321`: only govuk-036 leaves corpus-pages; no newly failing row. EN/TH reports, blockers and TSV were regenerated by the workflow; all live summaries now cite this run. **Do not republish v1.1.4.**

Environment note: `/workspace` from earlier sessions no longer exists; earlier `/workspace/work/*` logs are gone. This session rebuilt inputs under `/home/user/work` (craft-fonts pinned `c1db6d1`, prepared with `tools/prepare_corpus_fonts.py`; corpus via `cargo xtask readiness --download`).

Next: continue the remaining 28 page-count failures with actual font/layout evidence (govuk-018 Verdana→Inter lead in the section below remains read-only).

---

# Active page-count work · 2026-11-16

Current user request is to continue page counts. The v1.1.4 refresh is already complete: source/tag `c6945389f50fe548f778a85b578cb842dca4a665`, installer run 43778427578, same release ID 445039704, both new MSI/ZIP downloaded and verified, six cumulative version headings and pinned OFL notices checked. **Do not republish or move that tag without a new request.** Current work belongs in Unreleased.

Accepted local candidate: for Next Page + restart 1 + odd/even headers + preceding odd displayed number, insert a blank parity page before the real page. It belongs to the previous section and suppresses headers/footers; actual First Page stories and note indices remain intact. Other restart values, explicit Odd/Even Page behavior, Continuous/Next Column and web behavior are unchanged. See [audit](docs/readiness/restart-recto-2026-11-16.md).

Linux prepared-font page-count comparison **190→191/220** against c694538: only govuk-036 39→40 (reference40), other 219 unchanged, 440 corpus hashes verified. Blank page 2 inspected; Contents physical page 3 numbered1. Normalized page endpoints **37/40**; pages8/19/37 still differ, so boundary failure remains. No criteria/source/reference/exclusion/deferred changes.

Two new regressions cover the parity matrix, First Page headers/footers and note caret pages0/2. Initial prepared restart filter 4/4 passed; full CI with the added note assertions passed both new tests but stopped at unchanged proof_text timing ratio32.0 (<32 required). Isolated same-source recheck passed28.0; no threshold change. Final quiet full-CI rerun **passed 6/6**, native **2,557 pass / 1 existing ignored**, assets141, layers20, wasm16, without source/threshold changes. Final log `/workspace/work/restart-recto-ci-final.log`. Logs `/workspace/work/restart-recto-{tests-final,cli,comparison,ci,proof-recheck}.log`; baseline CLI `/workspace/work/page-count-baseline-2026-11-16-cli` is c694538. Candidate prepared CLI `/workspace/work/target-corpus-font-metrics/debug/doc4me-cli`.

Next: commit/push main, measure Windows readiness, refresh EN/TH report+blockers+TSV and all live summaries. Last official Windows baseline remains 43776538181, engine e00daf1, report b6e1076: pages191/220, boundaries99/220, 93.0% overall /94.9% excluding22deferred; 29 page-count failures. Continue remaining cases after confirming the new source. Further read-only lead: govuk-018 first page misses C.REGULATOR heading; source uses Verdana→Inter, not DejaVu Sans. Do not infer a fix from empty minimum-row splitting: current split_row deliberately moves rows that are taller than their text as a whole; a global change requires evidence and full corpus checks.

Earlier handoffs below are historical snapshots.

---

# v1.1.4 refresh after additional page-count match · 2026-11-16

The user's latest request re-authorizes refreshing existing release v1.1.4 from the latest code after improving page counts. It supersedes the earlier cancellation preserved below. Preserve the release URL/ID, replace Windows x64 MSI/portable ZIP, move the existing tag to the exact tested source, and retain six cumulative version sections newest-to-oldest through 1.0.0. Freeze main during publication; do not deploy Pages.

Windows [run 43776538181](https://github.com/veevie55/doc4me/actions/runs/43776538181) confirms engine `e00daf1`, report `b6e1076`: **191/220 page counts** (one added: zenodo-136 26→25), **99/220 boundaries**, **93.0% overall / 94.9% excluding 22 deferred**; open/round-trip/stability 220/220. No newly failing row in the five corpus sets. Current EN/TH blockers and TSV agree; 29 page-count and 121 boundary failures remain. No criterion, source/reference, exclusion or deferred change. See [blank-panel audit](docs/readiness/page-float-2026-11-15.md).

Release gates passed: full CI **6/6**, native **2,555 pass / 1 existing ignored**, prepared floating-table tests **6/6**, language audit including ignored **65/65**, cargo-audit **exit 0** with five unchanged unmaintained warnings (none suppressed). Final code stats **238,639 Rust lines / 2,564 test attributes**. Linux comparison **189→190/220**, only zenodo-136 changes; other 219 counts unchanged, all 440 source/reference hashes verified. Cover text returns to page 1; blue panel height and rule still differ from reference, so no pixel/full-boundary equivalence claim.

The release preparation includes current scores, cumulative notes and corrected external-font NOTICE/ATTRIBUTION. Rust engine/app code remains identical to measured engine `e00daf1`; later changes are release documentation only. For publication, verify exact source/tag/build SHA and asset digests, executable/version checks and pinned OFL notices. Existing release is [v1.1.4](https://github.com/veevie55/doc4me/releases/tag/v1.1.4); the release workflow and GitHub metadata provide the current publication result. Verification script: `/workspace/work/verify_release_refresh.py`. Logs: `/workspace/work/page-float-ci-final-guard.log`, `/workspace/work/page-float-release-prep-ci.log`, `/workspace/work/page-float-release-language.log`, `/workspace/work/page-float-release-audit.log`, `/workspace/work/page-float-comparison-final.log`; measured result JSON `/workspace/work/page-float-windows-result.json`.

Earlier handoffs below are historical snapshots.

---

# Publication cancelled by user · 2026-11-15

The user explicitly cancelled publishing v1.1.4. **Do not move the version tag, replace release assets, or run a publishing workflow unless newly requested.** No installer/release workflow was dispatched for this refresh; tag v1.1.4 remains `9f8d26f01ed33869fa28985f30137dacf6a15a84`, and existing release assets were not changed. Changelog pending changes remain Unreleased; the previous released sections are preserved.

Continue the remaining page-count work under the existing scope. Latest official Windows run **43727351543**, source `f01f7b6`, report `4d70b3c`: **93.0% overall / 94.9% without 22 deferred**, pages **190/220**, boundaries **99/220**; open/round-trip/stability **220/220**. Remaining failures: 30 page-count / 121 boundaries; EN/TH blockers and TSV agree. No criteria/reference/exclusion/deferred changes.

Language audit **65/65 pass** and dependency audit **exit 0**, five existing unmaintained warnings (none suppressed), logged in `/workspace/work/release-refresh-{language,audit}.log`. CI was interrupted by an environment reset; the in-progress fresh full run is `/workspace/work/release-refresh-ci-restarted.log` (do not infer completion from the interrupted log). RAM cache was recreated; app/CLI cache directories spill to `/workspace/deps/cargo-spill`.

Read-only investigation while CI rebuilds: `/workspace/work/remaining-page-inventory.json` covers all 30 official page-count failures using the immutable corpus and existing prepared CLI. Linux zenodo-114 has 71 pages versus Windows 70. `govuk-036` differs by a blank second reference page: check even/odd headers with section page-number restart 1. `zenodo-136` starts with a nearly page-high floating background table; source row height 16836 twips exceeds body height, causing `place_table` to choose the in-flow branch. In temporary DOCX copies only, lowering height to 14000 or 12000 twips restores cover text to page 1 and yields 25 instead of 26 pages. No corpus input was edited; a general positioned-table fix still needs implementation and full corpus regression checks. `/workspace/work/remaining-font-line-audit.json` stores numeric original font line metrics; `/workspace/work/windows-font-resolution-43727351543.json` stores sanitized actual Windows substitution diagnostics. Treat DOCX altName diagnostics separately from direct font resolver tests.

Earlier entries below are historical; their release authorizations are superseded by this cancellation.

---

# Current page-count handoff · 2026-11-15

Scope remains the remaining page-count cases only. Read [current font/section-marker audit](docs/readiness/latin-metrics-2026-11-15.md) and its 220-row comparison. Linux prepared-font page-count matches rise **187→189/220** versus source `6d257ac`; only zenodo-095 (9→10, reference 10) and zenodo-149 (40→28, reference 28) change. No previously passing page-count row regresses. This is not a new official Windows readiness score. No corpus/reference, exclusion, deferred row or threshold changes.

Implemented: Roboto condensed for narrow Arial, Montserrat for Gotham with matching weights, original OFL Source Sans Pro direct faces, Overpass Bold 700, measured selected font advances/line metrics, Tahoma bold alias; keep flags on empty section-end markers no longer create a blank page. Tahoma width trial withdrawn because govuk-044 regressed. Nyala U+0020 omitted from fit because source mappings disagree. No proprietary outlines/programs redistributed.

Validation: `cargo xtask ci` **6/6**, native **2,554 pass / 1 existing ignored**; prepared-font tests **83/83**; full 220-document page-count comparison; actionlint. Logs `/workspace/work/font-metrics-ci-final.log`, `/workspace/work/prepared-font-metrics-tests.log`, `/workspace/work/prepared-font-metrics-cli.log`, `/workspace/work/font-metrics-final-comparison.json`. Standard CI uses bundled fonts; separate corpus CLI uses pinned prepared OFL input. Code stats 238,588 Rust lines / 2,563 test attributes (not harness test counts).

ALSA development package installed rootlessly in `/workspace/deps/alsa`, pkg-config 1.2.14 and alsa-sys pass. Source `/workspace/deps/env.sh` for PATH/pkg-config/CMake/bindgen settings. Debug/incremental disabled to fit disk. `target/debug/build` points into `/dev/shm/doc4me-build-latin-metrics` (ephemeral cache); prepared build at `/workspace/work/target-corpus-font-metrics`. Full CI uses `RUST_TEST_THREADS=1` and `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/workspace/work/cargo-test-cpu0.sh` to stabilize timing, without relaxing limits.

Next: push accepted arc, dispatch official Windows readiness, compare reports, refresh all current readiness summaries and bilingual blockers from the measured report. Workflow now diagnoses fonts for every current page-count failure, plus zenodo-095/149/114. Last completed official report remains **43701689470**, source `fd465e8`, report `2a5a0b0`: **92.9% overall / 94.8% excluding 22 deferred**, pages **188/220**, boundaries **99/220**. Goal 100% is not reached. Continue remaining cases using actual font/layout evidence; no release or Pages deployment is part of this arc.

Earlier entries below are historical snapshots.

---

# Page-count follow-up · 2026-11-15

Current official Windows report is run 43701689470, source `fd465e8`, report `2a5a0b0`: 92.9% overall / 94.8% without 22 deferred; pages 188/220, boundaries 99/220. Every future measurement must refresh current bilingual Readiness blockers and TSV with the report (`tools/readiness_blockers.py`).

The new task is restricted to the 32 page-count failures: use openly licensed originals directly; improve open substitutes for commercial originals; replace proven mismatched reference documents, excluding only proven mismatches with no matching source available and recording the reason. See [font/reference audit](docs/readiness/page-count-audit-2026-11-15.md) and its 32-row TSV. Roboto was incorrectly forced to Arimo; the resolver/build input now uses its OFL original. Other commercial substitutions still require metric validation; no new exclusion is justified by this audit. `govuk-026` publisher links the existing pair and its normalized DOCX body-word overlap with the PDF is 1.000. No new official page-count passes are claimed. The release CLI probed all 32 rows on Linux; zenodo-136 uses Roboto directly but remains 26 pages versus 25. Font tests 76/76, font clippy, fmt, assets and layers pass; full CI stops in alsa-sys because the machine lacks ALSA development metadata. Logs: `/workspace/work/page-count-*.log`.

Previous snapshots below are historical.

---

# Current handoff · 2026-11-15

CI `cargo xtask ci` passes 6/6 on the current change set (fmt, clippy, workspace tests, assets, dependency layers, wasm). The readiness report generator now lists every non-passing row rather than the first 40; added bilingual blocker assessments for all 144 current layout failures and a Thai companion report generated from the official Windows summary. The explanations record measured differences and whether cause is verified; criteria, TSV rows, weights, 220-document corpus, 22 deferred and 24 excluded are unchanged.

Latest official Windows measurement: [run 43619305678](https://github.com/veevie55/doc4me/actions/runs/43619305678), source `4ecc270`, report `162704b`: **93.2% overall / 95.1% without deferred**, pages **188/220**, boundaries **108/220**. The remaining non-deferred failures are 32 page-count and 112 page-boundary checks; exact-match readiness is still below 100%. The Windows measurement completed successfully in run 43619305678; Actions committed the English and Thai reports as `162704b`.

Release v1.1.4 is the latest release; its tag/assets were refreshed from current `main` and cumulative notes are ordered v1.1.4 → v1.1.3 → v1.1.2 → v1.1.1 → v1.1.0 → v1.0.0. The release workflow has an explicit update-existing-release path that checks the tag commit, replaces MSI/ZIP assets and cumulative notes, and preserves the release identity. The previous 1.1.1, 1.1.2 and 1.1.3 releases remain restored.

The documentation and all 13 UI screenshots were refreshed against the shipped v1.1.4 source on 2026-11-15; the historical handoff snapshots below remain unchanged.

---

## Previous handoff snapshot · before the bilingual blocker update

Historical status below is retained for context; the current status and next actions are in the preface above.



CI `cargo xtask ci` ผ่าน 6/6 ที่ `8797e66`; การเร่ง `proof_text` ทำให้ timing probe ผ่านที่ 29.0× (เกณฑ์ <32×). Commit นี้ถูก push ขึ้น `main`; การวัด Windows หลัง push ใช้ run [43609090751](https://github.com/veevie55/doc4me/actions/runs/43609090751) บน source `8797e66` และสร้างรายงาน `ddcf7f0` โดย readiness **93.2% รวม / 95.1% เมื่อตัด deferred ออก**, pages **188/220**, boundaries **108/220** — ไม่ดีขึ้นจากรอบก่อน เป้าหมาย 100% ยังไม่สำเร็จ; ปัญหาที่ไม่ deferred ที่เหลืออยู่ใน pagination (32 page-count และ 112 page-boundary failures) โดยคงเกณฑ์ 220 เอกสาร / 22 deferred / 24 excluded

สถานะ Release: v1.1.4 ยังคงเป็น Latest และ notes ผ่าน API verification มีหกหัวข้อเรียง 1.1.4 → 1.1.3 → 1.1.2 → 1.1.1 → 1.1.0 → 1.0.0. คืน Release v1.1.1, v1.1.2 และ v1.1.3 พร้อม MSI/ZIP ของ Windows x64 แล้ว; notes เรียงรุ่นจากใหม่ไปเก่า: v1.1.1 มี 1.1.1/1.1.0/1.0.0, v1.1.2 มี 1.1.2/1.1.1/1.1.0/1.0.0, v1.1.3 มี 1.1.3/1.1.2/1.1.1/1.1.0/1.0.0. Release workflow รุ่นปัจจุบันใช้ `tools/release_notes.py` สร้าง notes สะสมทุกครั้ง; `AGENTS.md` ระบุกฎเดียวกัน และเมื่อเกินขนาด GitHub จะสรุปแต่ละหัวข้อพร้อมลิงก์รายละเอียดเต็ม

ส่วนที่ยังค้าง: ตรวจและแก้สาเหตุ pagination ที่เป็นบั๊กทั่วไปพร้อมวัดบน Windows อีกครั้ง เป้าหมายยังไม่ถึง 100% โดยไม่รวม deferred; ห้ามเพิ่ม exclusion หรือเปลี่ยนเกณฑ์โดยไม่มีหลักฐาน และอย่าอ้างว่าแก้ครบจนรายงาน readiness ยืนยัน

---

# Release 1.1.4 · prepared and verified source · 2026-11-14

ผู้ใช้สั่งให้ออก 1.1.4 จากโค้ดล่าสุดหลัง push main แล้ว และทำ readiness ต่อจน 100% ไม่นับ deferred จึงอนุญาต tag/Release 1.1.4 แล้ว ไม่ต้องถามซ้ำ ต้องตรวจ full CI, language audit รวม ignored และ cargo audit ก่อน release workflow ห้ามเปลี่ยน main ระหว่าง release run

ตรวจ release gates แล้ว: full CI 6/6, native 2,543 ผ่าน / ignored 1 เดิม, assets 141, wasm 16; language audit รวม ignored ผ่าน 65/65; cargo-audit 0.23.0 exit 0 หลังอัปเดต rustix 1.1.6 และ async-executor 1.14.1 ยังมี 5 unmaintained warnings ที่ไม่มี fixed upgrade ไม่มีการปิด advisory CLI แสดง 1.1.4 ขั้นถัดไป push commit นี้ แล้ว dispatch Windows x64 บน main ด้วย release=true (ไม่ใส่ commit input ซึ่งใช้สำหรับรุ่นเก่า) ตรวจ head SHA/tag/Release assets จริงก่อนอ้างว่าเผยแพร่แล้ว logs: `/workspace/work/{ci-1.1.4-patched,lang-audit-1.1.4,audit-1.1.4-patched}.log`

สองจุดที่ค้างปิดแล้วใน main99c4b5f: popup layering และหัวตารางซ้ำใน distributeRows; CI ผ่าน 6/6, probe 101 → 51 pt เป้าหมาย readiness 100% ยังไม่สำเร็จ ผลล่าสุดและ prerequisites ด้านล่างยังใช้ได้

---

# Handoff · table/clipboard + Windows verification · 2026-11-14

**Windows verification · 2026-11-14:** วัดซอร์สงานตาราง `97e1d6f` แล้วใน [run 43541894863](https://github.com/veevie55/doc4me/actions/runs/43541894863) / report `7a38383`: **95.1% ไม่นับ deferred**; pages **188/220**, boundaries **108/220**, open/round-trip/stability 220/220 ผลห้าชุดตรวจ ตรง baseline ทุกแถวและเหตุผล ตรวจคำสั่งตารางครบ 53 คำสั่ง พร้อม rich clipboard; แก้ชั้นแถบจัดรูปแบบให้เมนู popup อยู่ด้านบน ตรวจภาพ UI/PDF และ CI 6/6, native 2,543 อีกครั้ง เป้าหมาย 100% ยังไม่สำเร็จ คงเกณฑ์ 220 / 22 deferred / 24 excluded ไม่มี tag/Release

- สองจุดแก้ที่ค้างเสร็จแล้ว: เมนู popup อยู่เหนือ mini toolbar และกระจายแถวไม่นับหัวตารางซ้ำตามจำนวนหน้า; probe 50 pt ที่เคยเป็น 101 pt เหลือ 51 pt พร้อม regression และ full CI ผ่าน
- Code arc pushed at `97e1d6f`; Windows measured this engine source. The subsequent popup layer and repeated-header distribution fixes affect UI/edit commands only; full CI passed again in `/workspace/work/ci-table-resume-final.log`. No initial-document pagination/layout change follows the measured source.
- เกณฑ์ที่ยังไม่ผ่าน: page counts 32/220 และ page boundaries 112/220 ต้องทำ pagination ต่อ พร้อม Word/font/reference prerequisites เดิมด้านล่าง ไม่อ้างว่า command coverage ทำให้ readiness 100%
- หลักฐาน: `docs/readiness/table-clipboard-audit-2026-11-13.md`; logs/UI/PDF ใน `/workspace/work/`; registry 730 commands / ribbon 504/504 / tests 2,552 attributes; ไม่ออก Release

---

# Handoff · table/clipboard audit · 2026-11-14

**Table/clipboard audit · 2026-11-14:** ตรวจคำสั่งตารางครบ 53 คำสั่ง เพิ่มการเลือกคอลัมน์/เซลล์ว่าง เส้นขอบ 8 แบบและปากกาเส้นขอบ แก้หัวตารางหลายแถว การรวม/แยก/ขนาด/ช่วงเลือก และ rich clipboard ข้ามเอกสาร รวมรูป กล่องข้อความ กลุ่มวัตถุ แผนภูมิ SmartArt และ OLE พร้อมทรัพยากรที่อ้างอิง ดู [audit](docs/readiness/table-clipboard-audit-2026-11-13.md) ผล CI ผ่าน 6/6; native 2,543, ignored 1 เดิม ไม่มี tag/Release คะแนน Windows ล่าสุดของ body source `6127a73` / report `527a832` / run 43442685451 ยัง **95.1% ไม่นับ deferred**; รอวัดซอร์สงานตาราง ไม่เปลี่ยนเกณฑ์ 220 / 22 deferred / 24 excluded

- Source work is ready for Windows measurement after main push. Logs: `/workspace/work/ci-table-audit-eighth.log`; UI `/workspace/work/table-ui-column.png`; command matrix in the audit.
- เป้าหมาย 100% ยังไม่สำเร็จ ต้องตรวจ pagination และ prerequisites เดิมด้านล่างต่อ ไม่ปรับ ruler หรืออ้าง static command coverage ว่าเป็น readiness 100%

---

# Handoff: งาน pagination หลัง 1.1.3 · 2026-11-13 · ห้ามออก Release จนกว่าผู้ใช้สั่ง

## สถานะล่าสุด: body contextual spacing · 2026-11-13

**ขอบเขตเพิ่มจากผู้ใช้:** ตรวจคำสั่งตารางทั้งหมด โดยเน้นเพิ่ม/ลบ/เปลี่ยนเส้นขอบ เลือกทั้งแถว/คอลัมน์ ซ้ำหัวตาราง แยกตาราง และคัดลอกวัตถุที่ไม่ใช่ข้อความ (กล่องข้อความ รูปภาพ รูปร่าง ตาราง ฯลฯ) ให้ตรวจการทำงานจริง เพิ่ม regression และแก้บั๊กที่พบ งานนี้กำลังเริ่มและยังไม่ประกาศว่าผ่านครบ

- เป้าหมาย **100% ไม่นับ deferred ยังไม่สำเร็จ**; คะแนนทางการยัง **95.1%** จาก Windows source `92025e8` / report `c098032` / run 43333680851 (188/220 pages, 108/220 boundaries)
- รอบนี้แก้เฉพาะ body ตาม ECMA-376 §17.3.1.9: after 10 / before 12 และ previous contextual เหลือช่องว่าง 2 pt แทน 12 pt คง additive option และกฎตาราง/หัวกระดาษเดิม ดู [audit](docs/readiness/contextual-spacing-2026-11-13.md)
- เพิ่ม 3 regressions; CI 6/6, native 2,508, ignored 1 เดิม, layout 322, assets 141, wasm 16; stats 2,517 test attributes / 234,851 Rust lines / CLI 729
- Linux ครบ 220 ฉบับตรง baseline ทุกแถวในห้าชุดตรวจ: 189/220 pages, 103/220 boundaries, open/round-trip/stability 220/220; logs `ci-contextual-body.log`, `corpus-contextual-body.log`, `contextual-body-tests.log` อยู่ `/workspace/work/` CLI ที่ตรึงซอร์สนี้คือ `contextual-body-cli`
- ถอน trial ที่เปลี่ยน box spacing ด้วย หลัง govuk-020/zenodo-067 ทำ page-count passes เหลือ 187 และ boundaries 102; patch/log เก็บใน workspace ไม่เพิ่มข้อยกเว้นหรือเปลี่ยน TSV
- **ขั้นถัดไป:** push main แล้ววัด Windows; ตรวจ run/source/report จริงและเทียบห้าชุดก่อนอ้างคะแนนใหม่ จากนั้นทำ pagination ต่อ พร้อม Word/font/reference prerequisites ด้านล่าง ไม่ออก tag/Release

## ผล Windows ก่อนชุด contextual spacing · 2026-11-13

- **ยังไม่ครบ 100%**; คง 220 เอกสาร / 22 deferred / 24 excluded เวอร์ชัน 1.1.3 โค้ดใหม่อยู่ Unreleased ไม่มี tag/Release
- ผลทางการ [43333680851](https://github.com/veevie55/doc4me/actions/runs/43333680851), source `92025e8`, report `c098032`: **93.2% overall / 95.1% ไม่นับ deferred**, **188/220 page counts / 108/220 boundaries** จาก 184/102 ที่ source `da21a99` ทั้งห้าชุดตรวจไม่มีเอกสารที่เคยผ่านกลับไม่ผ่าน; open/round-trip/stability 220/220 ทุก dimension อื่นผ่าน 100% เมื่อไม่นับ deferred
- Source `92025e8` แก้ margin leading โดยคงพื้นที่ footnote/footer, font leading สำหรับรูปภาพนอกเซลล์/กริดและ whitespace, ภาพหัวกระดาษ first/even/default/linked และ full-width continuation พร้อม overflow fallback, Keep with next กับย่อหน้าว่าง ดู [margin audit](docs/readiness/margin-leading-2026-11-12.md) ยังไม่พอร์ต partial-width header reflow ในย่อหน้าต่อหรือกฎรูปภาพในเซลล์
- Linux แยกต่างหาก: 189/220 page counts และ 103/220 boundaries; Windows ผ่านเพิ่มจำนวนหน้าที่ zenodo-004/011/070/168 และขอบเขตที่ govuk-019, zenodo-042/058/065/093/162 ฝั่ง Windows ยังเหลือ **32 page-count / 112 boundary failures**
- CI เต็มตรวจอีกครั้งหลังถอนการทดลอง **6/6**, native **2,505 ผ่าน**, ignored 1 เดิม, layout 319, assets 141, wasm 16; stats 2,514 test attributes / 234,769 Rust lines / CLI 729 ไม่ลด threshold หรือเพิ่ม ignored Logs `/workspace/work/ci-leading-blank.log`, `ci-windows-92025-docs.log`, `corpus-leading-blank.log`; Windows failure TSVs อยู่ `/workspace/work/windows-{da21,92025}-failures.tsv`
- ทดลอง WordCraft `88d936c3` จาก Word for Mac สี่แบบแล้วถอนทั้งหมด: กฎเต็มและแบบจำกัดนอกเซลล์ทำ zenodo-165 เป็น 25 แทน 24 หน้า; cap 25% อย่างเดียวทั้งสองขอบเขตทำ page 15 ลงท้าย `000` แทน `sig` แม้บางชุดได้ 190 page counts / 108 boundaries ก็ไม่เก็บโค้ดหรือเทสต์ทดลอง ดู [justification review](docs/readiness/justification-review-2026-11-13.md) แอปยังใช้ 20% และเกณฑ์เดิม
- zenodo-114 ยังไม่ผ่าน boundaries: Word มี blank physical page 8 / quote page 9 แต่เราวาง quote page 8 โดยตำแหน่งภายในหน้าใกล้เคียงกัน มี `mirrorMargins` / `nextColumn` (single-column) / restart 1 ต้องแยก Word probe กฎ parity; Linux ใช้ Noto Naskh Arabic ทำบรรทัดชื่ออาหรับสูงกว่าจน `man` ไปอีกหน้า ชดเชย blank ที่หายจึงได้ 71 โดยบังเอิญ Windows จริงได้ 70 เทียบ Word 71 ห้ามถือว่า page count ตรงแปลว่า layout ครบ
- zenodo-023 อ้าง default header2/footer2 ที่ว่างจริง ไม่มี `evenAndOddHeaders`/`titlePg` มาชดเชย แต่ PDF page 52 มีหัวกระดาษและเลข 1 ต้อง Word export จาก SHA เดิม; zenodo-149 ไม่มี embedded fonts แต่ reference มี Arial Nova Cond Light/Narrow ที่ runner ไม่มี คำถามขอ URL/พาธ Word export และ fonts ยังไม่มีคำตอบ Integration list self-hosted runners ได้ 403 จึงยังไม่ทราบว่ามี Word runner หรือไม่ และยังไม่ได้รัน Word probes ไม่ใส่ proprietary assets หรือ subset จาก reference เข้าเครื่องวัด
- **งานถัดไป:** ทำ pagination ต่อโดยคงเกณฑ์เดิม; Word for Windows probes เรื่อง justification และ mirrored number restart (starts 0/1/2, nextPage/nextColumn/odd/even, first-page stories/SECTIONPAGES) และยืนยัน input/reference กับ fonts เป้าหมาย 100% ยังเปิดอยู่ ผล Windows ล่าสุดยืนยันโค้ดทั้งหมดที่เก็บไว้แล้ว

## บันทึกก่อนรอบนี้

## สถานะรอบก่อน: พอร์ตกฎ break จาก upstream · 2026-11-12

- **ยังไม่ถึงเป้าหมาย 100% ไม่นับ deferred**; คงเกณฑ์เดิม 220 เอกสาร / 22 deferred / 24 excluded ไม่มี tag หรือ Release เวอร์ชัน 1.1.3 โค้ดใหม่อยู่ Unreleased
- ผลทางการล่าสุด: Windows [43245507907](https://github.com/veevie55/doc4me/actions/runs/43245507907), source `e9b354b`, report `e2e212f`: **92.9% overall / 94.8% ไม่นับ deferred**, pages **184/220**, boundaries **102/220**; ทุก dimension นอกจาก pagination ผ่าน 100% เมื่อไม่นับ deferred ต้องตรวจซอร์สที่รายงานวัดจริง
- โค้ดที่ push แล้วในรอบนี้: `c030b15` z-order/Arrange/Continuous section govuk-037, `a466dff` Unicode sans fallback, `2338a45` contextual spacing ก่อนตาราง govuk-034, `e9b354b` Tab ในบรรทัดโลโก้ zenodo-156; ทั้งสี่ arc ผ่าน CI 6/6 ก่อน commit และไม่พบ corpus regressions
- Windows run 43242054819 source `2338a45` ไม่ได้วัดเพราะไม่ได้ hosted runner (job cancelled ก่อน steps); run 43245507907 source `e9b354b` สำเร็จแล้ว รายงาน `e2e212f` ยืนยัน govuk-034 และ zenodo-156 ผ่านเพิ่มโดยไม่ถดถอย ไม่ใช้ผล Linux แทน Windows
- รอบนี้พอร์ต WordCraft 5c0a62a7 และ 7830333c: จุดติดวงเล็บเปิดไม่แบ่งกลางคำ และ pageBreakBefore ที่ต้น Continuous section กลางหน้าใช้ต้นหน้าจริง; เพิ่ม regression 2 ตัว (มี/ไม่มีช่องว่าง, Continuous/NextPage/หน้าแรก); CLI ยัง 729 คำสั่ง ไม่มีโมเดล/format/asset ใหม่
- ถอนการทดลอง 76345539 (superscript ปัดครึ่ง pt) หลัง zenodo-093 ถดถอย 97 → 96 boundary passes; เอกสารมี section เดียว/คอลัมน์เดียว/ไม่มี `.(`/superscript 43 แห่ง หลังถอนเทียบครบ 220 ใหม่ได้ **pages 184/220 / boundaries 97/220** เท่า source `e9b354b` ทุกแถวทั้งห้าชุดตรวจ ไม่ซ่อน failure หรือเพิ่ม ignored
- `cargo xtask ci` ของซอร์สสุดท้ายผ่าน **6/6**, native **2,499 passed**, ignored 1 เดิม, layout 313, assets 141, wasm 16 crates; ใช้ `RUST_TEST_THREADS=1 taskset -c 0` เพราะ benchmark proof_text เดิมแกว่งใกล้เกณฑ์ 32 (ตรึง CPU ผ่าน 30.4 / 30.4 / 28.8); ไม่เปลี่ยน assertion หรือปิดเทสต์, log ล้มรอบแรกคงไว้ `ci-upstream-refined-parallel-failed.log`; log ผ่าน `/workspace/work/ci-upstream-refined.log`; stats **2,508 test attributes / 234,344 Rust lines** ผล corpus `/workspace/work/corpus-upstream-refined.log`
- ต้นทาง [storytold/wordcraft](https://github.com/storytold/wordcraft) public เข้าถึงได้แล้ว ล่าสุด `07f97737`; ดู [upstream review](docs/readiness/upstream-layout-review-2026-11-12.md) ระบุพอร์ต 2 กฎ/ทดลองที่ถอน/compatibility prerequisites ที่ยังเหลือ ไม่อ้าง replay ทุก commit; craftrules ค้นไม่พบ และ native-speaker review ยังต้องมีผู้ตรวจจริง
- [Reference audit](docs/readiness/reference-audit-2026-11-12.md#zenodo-023-source-and-reference-content): zenodo-023 หน้า 52 PDF มีหัวกระดาษ/เลข 1 แต่ DOCX อ้าง header2/footer2 ที่ว่างโดยตรง; ขอ URL/พาธ Word export จาก DOCX SHA เดิมแล้ว ยังไม่มีคำตอบ zenodo-149 ไม่มี embedded font ใน DOCX แต่ PDF ใช้ Arial Nova Cond Light/Narrow ที่ runner ไม่มี คำถาม optional เรื่อง Windows/fontsยังไม่มีคำตอบ ห้ามใช้ subset จาก reference ป้อนเครื่องวัด/เพิ่ม proprietary assets/ผ่อนเกณฑ์เพื่อให้ 100%
- ขั้นถัดไป push กฎ break แล้ววัด Windows จาก mainอีกครั้ง ตรวจ run/source/report และ regressions จริงก่อนปรับคะแนน ยังมี 36 Linux page-count failures และ 123 boundary failures (รวม source/reference contradiction) ต้องทำ pagination ต่อ

### บันทึกรอบก่อน Unicode fix (รักษาเป็นประวัติ)

## งาน Codex ต่อจาก `e65459b` · 2026-11-12 · เป้าหมาย 100% ไม่นับ deferred ยังไม่สำเร็จ

คำสั่งผู้ใช้: อ่าน handoff แล้วทำงานค้างต่อให้เสร็จและ readiness 100% โดยไม่รวม deferred; คงเกณฑ์เดิมและ 22 deferred/24 excluded ไว้ ห้ามออก Release จนกว่าผู้ใช้สั่ง

### ทำแล้วในโค้ดรอบนี้

- เก็บ `wp:anchor/@relativeHeight` เป็น `Float.z_order: u32` ตลอด DOCX/JSON; ค่า XML ผิดรูปแบบ/ล้นช่วงใช้ 0, JSON เก่า default 0; writer ไม่สร้าง z ใหม่ตามลำดับ XML อีก
- แยก floating drawing layers ระหว่างจัดหน้าแล้วเรียงแบบ stable ตามชั้นหน้า/หลังข้อความและ z; รูปร่าง กล่องข้อความและ hit area อยู่ด้วยกัน จัดลำดับข้ามย่อหน้าได้; ภายในเซลล์ตาราง/หัวท้าย/กล่องมีกรอบวาดของตัวเอง ไม่เพิ่ม variant สาธารณะของ `Placed`
- Bring Forward/Send Backward ย้ายหนึ่งตำแหน่งในหน้า/story/กรอบเดียวกัน โดยไม่เปลี่ยน wrapping/พิกัด; จัดการค่า z ซ้ำ/เต็มช่วง, ไม่เปลี่ยนวัตถุบนหน้าอื่นหรือในอีกเซลล์, อยู่บนสุด/ล่างสุดแล้วเป็น no-op; undo/redo และ DOCX reopen มีเทสต์
- `govuk-037`: ตัวแบ่งหน้าล้วนก่อนย่อหน้าว่างปิด section ซึ่ง section ถัดไปเป็น Continuous ต้องวางบรรทัดให้พอก่อนแบ่งหน้า จึงรักษาหน้าว่างหน้า 2; ตรวจตามโครงสร้าง ไม่ใช้ ID เอกสาร/`lastRenderedPageBreak`/ขนาดฟอนต์ตายตัว กรณีทั่วไปและ NextPage คงกฎเดิม
- เพิ่ม regression 9 ตัว: JSON 1, DOCX UInt/round-trip 1, Arrange 3, layout 4; สถิติซอร์ส `cargo xtask stats`: 2,502 test attributes, 234,185 บรรทัด Rust; CLI ยัง 729 คำสั่ง
- `cargo xtask ci` ผ่านครบ 6 ขั้น (fmt, clippy -D warnings, native test 2,493 ผ่าน รวม layout 308, assets 141 ไฟล์, layers และ wasm 16 crates); ผล CI บนซอร์สสุดท้ายอยู่ใน `/workspace/work/ci-final.log`

### หลักฐานการเทียบจริง

- ดาวน์โหลดและตรวจ SHA-256 ครบ 440 ไฟล์ (220 DOCX + 220 PDF); เทียบ CLI จาก `e65459b` กับ working tree นี้บน Linux เครื่อง/โปรไฟล์/ฟอนต์เดียวกัน ผ่าน `readiness --corpus --dry-run --jobs 2 --diagnostics --cli ...`
- เปิด/round-trip/เสถียร 220/220 ทั้งสองรอบ; จำนวนหน้า **181 → 182/220** (`govuk-037`: 78 → 79); ขอบเขตหน้า **95/220 คงเดิม**; ไม่มีแถวใดในห้าชุดตรวจถดถอย
- `zenodo-097` หน้า 60: ภาพ Poppler สูงสุด 900 px หลังแก้มี SHA-256 `e4fd683b2b494e19dcfffc182e2508c69113890f01d231b12ab07bac5a200774` ตรงกับภาพ PDF อ้างอิง; ก่อนแก้เป็น `11a7e36d422dd6dd75e2b8073e66bf47d3fc05fa5b52430202f3e872dce03928` (พื้นขาวบังแถบเขียว) ไม่ commit ภาพ/PDF อ้างอิง
- Windows run [43180980471](https://github.com/veevie55/doc4me/actions/runs/43180980471), source `e65459b`, attempt 1: ไม่ได้เริ่มวัด; job `measure` จบ cancelled และ run failure หลัง 15 นาที: **“The job was not acquired by Runner of type hosted even after multiple attempts”** ไม่มีรายงานใหม่
- คะแนนทางการยังเป็น **92.7% / 94.5% ไม่นับ 22 deferred** จาก Windows run 43051188273 (`bd2d2d6`); `docs/readiness.md` ไม่ถูกเขียนทับด้วยผล Linux ไม่มี tag/Release หรือ platform build อื่นในรอบนี้

### งานที่ยังต้องต่อ

1. หลัง push โค้ดรอบนี้ รัน Windows readiness บน `main` อีกครั้งและตรวจผลจริง; workflow เป็น manual-only อยู่แล้ว รักษาเกณฑ์/TSV/excluded/deferred เดิม
2. Pagination ยังไม่ครบ: Linux ยังไม่ผ่านจำนวนหน้า 38/220 และขอบเขตหน้า 125/220; `govuk-037` ยังต่างที่ TOC หน้า 4 ลงท้าย 62 แทน 63 และหน้าหลัง ๆ แม้จำนวนหน้าตรงแล้ว
3. ห้ามแก้ด้วยการเรียก `references.updateFields` ทั้งชุดก่อน export โดยไม่ตรวจ: ทดลองเฉพาะไฟล์ชั่วคราวของ `govuk-037` แล้ว TOC ถูกสร้างใหม่จนรวมเป็น 80 หน้า (อ้างอิง 79)
4. คงงาน govuk-034, govuk-026 (ยัง 4 vs 7 หน้า), zenodo-114, zenodo-023, embedded-font line metrics และ zenodo-149/ฟอนต์ Arial Nova Cond Light ตามลำดับเดิม; พบเพิ่มว่า heuristic `family.contains("code")` จัด `Arial Unicode MS` เป็น monospace เพราะคำว่า Unicode มี code อยู่ ต้องตรวจแยกพร้อมเทสต์ฟอนต์และเทียบคลัง/Windows
5. WordCraft/craftrules ยังเข้าไม่ได้: connector ตอบ 404 สำหรับ `veevie55/wordcraft` และ `storytold/craftrules`; ไม่อ้างว่า replay แล้ว; การทบทวนภาษาโดยเจ้าของภาษายังไม่เสร็จ ส่วน 22 deferred ไม่อยู่ในเป้ารอบนี้

### ข้อควรระวังเมื่อทำซ้ำ

- เคยทดลองกฎ “ย่อหน้าที่มีเพียง page break ต้องเผื่อบรรทัดเสมอ” และการไม่แบ่งแถว Exact; ชุดทดลองรวมถดถอย govuk-002/046, zenodo-048/141 (178/220 pages, 94/220 boundaries) จึงถอนกฎกว้างและการเปลี่ยน Exact ทั้งหมด แล้วเทียบครบใหม่จนไม่ถดถอย ห้ามนำผลชุดทดลองมาเป็นผลสุดท้าย
- ใช้ target แยกระหว่าง baseline worktree กับต้นงานจริง อย่าใช้แคช Cargo ร่วมกัน: เคยอ้าง model เก่าหลังสลับ worktree แก้โดยบังคับ rebuild และเก็บ CLI เป็นไฟล์แยกก่อนเทียบ
- เครื่องนี้เริ่มไม่มี Rust; ติดตั้ง 1.99.1/rustfmt/clippy/wasm target ใน `/workspace/work`, CMake จาก pip และ ALSA dev จาก Debian เฉพาะ workspace ต้องให้ `libasound.so -> libasound.so.2.0.0` ชี้ถึงไลบรารีจริง; ไม่เปลี่ยน dependencies โครงการ ปิด debuginfo/incremental เพราะดิสก์ 8 GB; ลบเฉพาะ PDF/ผลแปลงและ test executables เก่าภายใน target เพื่อคืนพื้นที่

## บันทึกรอบก่อนหน้า

ผู้ใช้สั่งรอบนี้: อ่าน handoff และทำงานที่ค้างต่อใน `veevie55/doc4me`; ทำต่อด้าน pagination ตามลำดับงานล่าสุด ไม่เปลี่ยนเกณฑ์หรือรายการ excluded/deferred

## คำสั่งและสถานะล่าสุดของรอบนี้

- ผู้ใช้สั่งสร้าง workflow build แยก Windows ARM64, macOS, Linux, BSD และ Web App เลือกโค้ดล่าสุดหรือ commit ได้ ห้าม workflow เริ่มเองถ้าไม่ได้สั่ง และแก้ README ไทย/อังกฤษพร้อมภาพตาม 1.1.3
- จากนั้นผู้ใช้สั่ง **เมื่อเสร็จ ให้ทำงานค้างต่อจน readiness 100% โดยไม่รวม deferred**; เป้าหมายยังไม่ถึง ต้องทำต่อหลังบันทึกงาน workflow/README ห้ามเพิ่ม excluded/deferred หรือเปลี่ยนเครื่องวัดเพื่อให้ได้คะแนน
- สร้าง `build-{windows-arm64,macos,linux,bsd,web}.yml` แยกกัน รับ `ref` (default main), macOS เลือก arch ได้; BSD คือ FreeBSD 14.5 x86_64 ไม่ได้อ้างว่าครบทุก BSD; Artifacts 14 วัน + SOURCE_COMMIT.txt
- Definitions ปัจจุบันครบ 8 ไฟล์มีแต่ `workflow_dispatch` และทุก job มี manual event guard; ลบ combined ARM64/macOS เก่าและลบ tag trigger ของ x64; release ของ x64 ต้องติ๊กเอง ไม่ได้สั่ง build ทั้งห้า ไม่สร้าง tag/Release หรือ deploy Web App
- `actionlint` 1.7.12 และการตรวจ manual-only/ref/permissions/shell syntax ผ่าน; nFPM 2.49.0 สร้าง DEB/RPM metadata จาก fixtures ผ่าน เพิ่ม ALSA/libstdc++ dependency ตาม `ldd` ของ desktop binary
- README สองภาษาอัปเดตจริง (ลิงก์ Releases เดิมยังชี้ upstream ตามคำสั่งถาวร), ภาพเดิม 6 ภาพสร้างใหม่ + About 2 ภาษา 2 ภาพ; สคริปต์ใน docs/images/shots, ดูครบทุกภาพแล้ว และ assets 141 ไฟล์มี attribution ครบ
- ภาพสร้างได้โดยใช้ `VK_ICD_FILENAMES=/usr/lib/chromium/vk_swiftshader_icd.json`, `WGPU_BACKEND=vulkan`, `XDG_RUNTIME_DIR=/workspace/work/xdg` กับ `target/debug/examples/ui_shot`; ไม่ต้องลง Mesa เพิ่มในเครื่องนี้ ดิสก์ 8 GB เคยเต็มจน sandbox เริ่มไม่ได้ จึงลบเฉพาะ test executables และ PDF ชั่วคราวใน target (auto-review อนุมัติ cleanup) ไม่มีข้อมูลผู้ใช้หาย
- คะแนนทางการยังเป็น Windows run 43051188273 attempt 1: 92.7% / 94.5% ไม่นับ 22 deferred, pages 181/220, boundaries 99/220; ผล Linux รอบนี้ไม่แทนคะแนนทางการ

## รอบ 2026-11-12: วัตถุอิงย่อหน้าหลังตัวแบ่งหน้า/คอลัมน์

- แก้ `place_objects` และฐานตำแหน่งส่วนต่อใน `crates/layout/src/lib.rs`: วัตถุ `InFrontOfText`/`BehindText` ที่อิงแนวตั้งกับ `Paragraph` และจุดยึดอยู่หลังตัวแบ่งหน้า/คอลัมน์ที่มองเห็น ให้วางในหน้า/คอลัมน์แรกของส่วนหลังตัวแบ่ง ใช้จุดเริ่มส่วนต่อเป็นฐาน และวางครั้งเดียว แม้จุดยึดจริงไหลไปหน้าหลัง; ฟิลด์ `PAGE` ในกล่องข้อความคำนวณจากเลขหน้าปลายทาง
- จำกัดการเปลี่ยนพฤติกรรมไว้ที่เนื้อหาในมุมมองแบ่งหน้า; วัตถุอิงหน้ากระดาษ (เทสต์ปกเดิม), วัตถุที่ข้อความต้องหลบ, ตาราง, หัว/ท้ายกระดาษ, กล่องข้อความ, เว็บ และเค้าร่างคงพฤติกรรมเดิม ไม่แก้โมเดลหรือรูปแบบบันทึก
- เพิ่ม regression 6 ตัว: `paragraph_relative_art_follows_explicit_page_breaks` (หน้า/หลังข้อความ, หลายตัวแบ่ง), `paragraph_relative_art_follows_an_explicit_column_break`, `paragraph_relative_art_stays_on_its_segments_first_page_when_text_wraps`, `a_hidden_page_break_does_not_move_paragraph_relative_art`, `a_text_box_after_a_page_break_uses_its_destination_page_number` (เริ่มเลขหน้าจาก 0/7); `deferred_art_uses_the_column_origin_with_mirrored_indents` (พิกัดจากคอลัมน์จริง); ชุด layout ผ่าน 304 ตัวใน `cargo xtask ci`
- ดาวน์โหลดคลังครบ 440 ไฟล์ (220 DOCX + 220 PDF) และตรวจ SHA-256 ตรง manifest ทั้งหมด; สแกน OOXML ของย่อหน้าเนื้อหา พบกรณีที่เข้าข่าย 2 เอกสาร: `govuk-001` ย่อหน้า 4 และ `zenodo-097` ย่อหน้า 448
- ดู PDF ที่สร้างจริงเทียบ PDF อ้างอิง: `govuk-001` กรอบและชื่อ “Document explainer” อยู่หน้า 2 แล้ว; `zenodo-097` ย้ายรูปร่างไปหน้า 60 แล้ว แต่ยังมีปัญหาเดิมเรื่องลำดับซ้อน (`wp:relativeHeight`): รูปสี่เหลี่ยมสีขาวบังสีเขียว เพราะโมเดล `Float` ยังไม่เก็บลำดับซ้อน ต้องแก้แยกพร้อมทดสอบการอ่าน/เขียนและคำสั่งจัดลำดับวัตถุ
  - หลักฐาน `zenodo-097`: สีเขียวมี `relativeHeight="251660288"` สูงกว่าสีขาว `251659264`, ทั้งสอง `behindDoc="0"` แต่ XML เรียงสีเขียวก่อนสีขาว; `anchor_float` ไม่อ่านค่านี้และ writer สร้างใหม่ด้วย `self.z`; `arrange.bringForward`/`arrange.sendBackward` ปัจจุบันเปลี่ยนเฉพาะ wrap ต้องออกแบบการเก็บ/จัดชั้นให้สอดคล้องกันก่อนแก้
- `cargo xtask ci` ผ่านครบ 6 ขั้น (fmt, clippy, test, assets, layers, wasm), native ผ่าน 2,484 ตัว รวม layout 304 ตัว; เทียบ Linux แล้ว: เปิด/round-trip/เสถียร 220/220 ทั้งสองชุด, จำนวนหน้า 181/220 เท่าเดิม, ขอบเขตหน้า 94 → 95/220 (`govuk-001` ผ่านเพิ่ม), ไม่ถดถอย; ยังรอวัด Windows ห้ามใช้ผล Linux แทนคะแนนทางการ
- สภาพแวดล้อมรอบนี้เริ่มว่าง ไม่มี Rust: ติดตั้ง Rust 1.99.1 + rustfmt/clippy/wasm target ใน `/workspace/work`; CMake/libclang และ ALSA dev เป็น dependency เฉพาะเครื่องทดสอบ ไม่แก้ dependency ของโครงการ มีดิสก์ 8 GB จึงปิด debuginfo/incremental สำหรับการตรวจ
- `storytold/craftrules` ตอบ 404 ทั้ง connector และ HTTPS; `veevie55/wordcraft` ตอบ 404 ผ่าน connector จึงยัง replay upstream ไม่ได้; ไม่ได้ข้ามการตรวจโค้ดใน Doc4Me

### งานถัดไป

1. ตรวจผล Windows ของโค้ดรอบนี้และบันทึกคะแนนสองค่า โดยไม่ออก Release; คะแนนทางการเดิมยังเป็น 92.7% / 94.5% ไม่นับ 22 deferred (181/220 pages, 99/220 boundaries), run 43051188273 attempt 1
2. ลำดับซ้อนของรูปลอย `wp:relativeHeight` ใน `zenodo-097` (มีหลักฐาน PDF ด้านบน); กรณีวัตถุที่ข้อความต้องหลบหลังตัวแบ่ง ต้องจัดบรรทัดส่วนต่อใหม่ก่อนขยายกฎ
3. Pagination อื่นตามรายการเดิม: govuk-034, govuk-026, govuk-037, zenodo-114, zenodo-023; ฟอนต์ใน `.docx` ต้องตรวจ line metrics ก่อนใช้; zenodo-149 ขาดตัวเทียบฟอนต์ตรงขนาด
4. replay WordCraft เมื่อเข้าถึงต้นทางได้, ทบทวนภาษาโดยเจ้าของภาษา และหลักฐานอุปกรณ์/บริการของ 22 deferred

## ประวัติเมื่อจบการออก 1.1.3 (2026-11-11)

### ภาพรวมเดิม: Doc4Me 1.1.3 (ออกใหม่ 2026-11-11 จาก `552a0b6`) · readiness ล่าสุด **92.7% / 94.5% without 22 deferred** (Windows run [43051188273](https://github.com/veevie55/doc4me/actions/runs/43051188273), source `bd2d2d6`, คลัง 220 เอกสาร) · ห้ามออก Release ถัดไปจนกว่าผู้ใช้จะสั่ง

อัปเดต: 2026-11-11 · fork `veevie55/doc4me` · branch `main`; คำสั่งล่าสุดของผู้ใช้: "ตั้ง zenodo-104 และ govuk-003 (PDF อ้างอิงไม่ตรงกับไฟล์ .docx) เป็น excluded ตรวจ readiness อีกครั้ง แก้ไขเนื้อหาทุกไฟล์ตามโค้ดล่าสุด ออกเวอร์ชัน 1.1.3 ใหม่จากโค้ดล่าสุด บันทึก handoff และยุติการทำงานทั้งหมด" — ทำครบแล้ว และยุติงานทั้งหมด

## ยุติงานตามคำสั่งผู้ใช้ (2026-11-11, รอบสุดท้าย): สถานะสำหรับ session ถัดไป

ไม่มีงานรันค้าง, working tree สะอาด, ทุกอย่าง push เข้า `main` แล้ว

### สิ่งที่ทำในรอบนี้
- `718d22a`: ตั้ง zenodo-104 และ govuk-003 ใน `docs/readiness/corpus.tsv` เป็น `excluded` พร้อมเหตุผล (PDF อ้างอิงไม่ได้สร้างจาก .docx ในคลัง) คลังนับ 220 เอกสาร (excluded 24 แถว)
- `ff503b1` และ `bd2d2d6`: ขั้นวินิจฉัยฟอนต์ใน `.github/workflows/readiness-windows.yml` เอา govuk-003 ออก ข้ามเอกสารที่ไม่ได้ดาวน์โหลด และแก้ `"$id:"` เป็น `"${id}:"` (PowerShell อ่าน `$id:` เป็นตัวแปรมี scope จน parse ล้ม; run 43043846909 และ 43047705006 ล้มที่ขั้นนี้จึงไม่ได้ commit รายงาน)
- Windows run 43051188273 (source `bd2d2d6`, report commit `5292abd`): **92.7% / 94.5%**, pages **181/220**, boundaries **99/220**, เปิด/round-trip/เสถียร 220/220, มิติอื่นไม่นับ deferred 100% (ก่อนตั้ง excluded: run 43011248444 ได้ 182/222 และ 99/222; govuk-003 เคยผ่านจำนวนหน้า)
- `552a0b6`: แก้เอกสารทุกไฟล์ให้ตรงโค้ด (ROADMAP, docs/roadmap, docs/*-parity, docs/gaps, docs/readiness/README, deferred-plan, README/README.en เพิ่มฟอนต์ที่ฝัง; สถิติ 729 คำสั่ง, 2,487 `#[test]`, 233,735 บรรทัด Rust) และย้าย `[Unreleased]` เข้า `## [1.1.3] - 2026-11-11`; `cargo xtask ci` ผ่านครบ 6 ขั้น
- ผู้ใช้ลบ Release/tag v1.1.3 เดิม (จาก `7422790`) เอง แล้ว workflow run [43055170446](https://github.com/veevie55/doc4me/actions/runs/43055170446) สร้าง tag `v1.1.3` ที่ `552a0b6` และ [Release v1.1.3](https://github.com/veevie55/doc4me/releases/tag/v1.1.3) มี `doc4me-1.1.3-windows-x64.msi` (50,376,704 ไบต์) และ `doc4me-1.1.3-windows-x64-portable.zip` (69,668,496 ไบต์) ยังไม่ลงลายเซ็นดิจิทัล
- v1.1.3 นี้รวม: ฟอนต์ธีม `majorBidi`/`majorEastAsia`, Helvetica/Times → Arial/Times New Roman, ฝัง Carlito/Caladea/Arimo/Tinos/Cousine (OFL, เดสก์ท็อป) ต่อท้ายฟอนต์ระบบ, โหมด 14/15 รอบ page break, เลขหน้าเริ่มที่ 0, section ต่อเนื่องใต้วัตถุลอย, ตัดบรรทัดหลังยัติภังค์ระหว่างตัวเลข, grid ในตาราง, Font › Advanced › Spacing ▲▼ ทีละ 0.1 pt, About ระบุผู้พัฒนา, `cargo xtask readiness --diagnostics` แถว `fonts`

### หมายเหตุ
- session นี้ลบ Release/tag ผ่าน API ไม่ได้ (403) ถ้าต้องออกรุ่นซ้ำ ผู้ใช้ต้องลบเองก่อน แล้วสั่ง `windows-x64-installer.yml` ด้วย `{"release": true}` บน `main` (workflow ปฏิเสธถ้า tag มีอยู่แล้ว) และห้าม push ระหว่าง workflow รัน
- ห้ามแก้ลิงก์ Releases ใน README (ชี้ `chatraweechai/doc4me` ตามที่ผู้ใช้ต้องการ)

### งานที่เหลือ (เมื่อผู้ใช้สั่งต่อ)
1. Pagination มิติเดียวที่ยังไม่ 100% เมื่อไม่นับ deferred (181/220 และ 99/220); เป้า 100% ไปไม่ถึงด้วยการแก้โค้ดอย่างเดียว เพราะบางเอกสารขึ้นกับฟอนต์ที่ไม่มีตัวเทียบขนาดเปิด (zenodo-149: Arial Nova Cond Light/Arial Narrow)
2. งานที่รู้สาเหตุแต่ยังไม่แก้:
   - zenodo-067 ผ่านก่อนหน้าเพราะ Carlito กว้างใกล้ Overpass ลำดับฟอนต์ปัจจุบันตรงกับ Word จึงไม่ย้อนกลับ
   - govuk-001, zenodo-097: วัตถุลอยที่ anchor อยู่หลัง page break ในย่อหน้าเดียวกันต้องไปหน้าถัดไป (`anchor_floats`)
   - ฟอนต์ที่ฝังใน .docx (`.odttf`): ถอดรหัสถูก (XOR 32 ไบต์แรกด้วยคีย์จาก `w:fontKey`, ลงทะเบียนด้วย `w:name`) แต่ทำให้ zenodo-102/129 แย่ลง ต้องสืบเมตริกบรรทัดก่อน (โค้ดทดลองไม่ได้ commit)
   - govuk-034 เกิน 1 หน้า; govuk-026 Word ยกตารางย่อยไปหน้าใหม่; govuk-037/zenodo-114 หน้าว่างของ Word ที่ยังอธิบายไม่ได้; zenodo-023 หัว/ท้ายกระดาษหน้าสุดท้าย
3. replay commit WordCraft ช่วงที่เหลือตามบันทึกสำรวจ 2026-11-09 ด้านล่าง
4. ให้เจ้าของภาษาตรวจทาน 26 แค็ตตาล็อกฉบับร่าง AI; shaping/RTL ใน UI
5. deferred 22 แถวต้องมีหลักฐานจากอุปกรณ์/บริการจริง

### วิธีทำงานที่ใช้ได้ผล
- วัดบน Linux: `cargo xtask readiness --corpus --dry-run --jobs 4 --diagnostics --cli target-port/release/doc4me-cli` (ต้องใส่ `--cli`) แล้วเทียบรายเอกสารก่อน/หลัง; เรื่องฟอนต์ต้องยืนยันบน Windows (`readiness-windows.yml` เป็นผลทางการ และ commit `docs/readiness.md` เอง)
- อ่านรายการเอกสารที่ไม่ผ่านและแถว `fonts` จาก log ของ job บน Windows (`get_job_logs`)
- เทสต์วัดเวลาที่ล้มเป็นครั้งคราวตอนรันขนาน: `a_long_unbreakable_word_lays_out_in_linear_time`, `conversion_work_scales_with_reference_and_formatting_runs`

## สถานะก่อนหน้า (ข้อมูลเก่า; ตัวเลขล่าสุดอยู่ด้านบน)

### สถานะเมื่อออก v1.1.3 จาก `7422790` (2026-11-11, ถูกแทนแล้ว)

### Readiness
- **92.2% รวม / 94.1% เมื่อไม่นับ 22 แถว deferred** จาก Windows run [42961322752](https://github.com/veevie55/doc4me/actions/runs/42961322752) (วัด source `9abf2d6` ซึ่งรวมงาน port ล่าสุดทั้งหมด, 222 เอกสาร, report commit `1df7aae`); เทียบ run 42836913553 (source `1352238`) แล้วดีขึ้นเฉพาะจำนวนหน้า 174 → 175 (zenodo-103 ตรง Word หลังหยุดปัดข้อความในตารางเข้า document grid เมื่อไม่มี `w:adjustLineHeightInTable`) ที่เหลือเท่าเดิม ไม่มีมิติใดถดถอย
- มิติ: features 97.4% (ribbon 503/503, dialogs 216/228, 12 deferred), file formats 100% (222/222), pagination 60.8% (pages 175/222, boundaries 95/222), UI/UX 98.6% (276/280, 4 deferred), stability 100%, performance 100%, localization 100% (95/95), platforms 100% (5/5), ecosystem 83.3% (1 deferred), hardware 37.5% (5 deferred)
- ส่วนที่ไม่ deferred และยังไม่ 100% มีเพียง pagination (ห่าง 5.9 จุดของคะแนนรวม); deferred 22 แถวยังนับไม่ผ่านจนมีหลักฐานจากอุปกรณ์/บริการจริง

### ภาษา
- 91 ตัวเลือกใน `LANGUAGES`: English source + 90 แค็ตตาล็อก (en-gb + 89 ภาษา) ทุกแค็ตตาล็อกมี 3,034 คีย์; ครอบคลุม 95/95 แถวใน `docs/readiness/languages.tsv`
- 26 แค็ตตาล็อกเป็นฉบับร่าง AI ที่ต้องให้เจ้าของภาษาตรวจทาน (ig, tn, wo, yo และ 22 ภาษาที่เพิ่ม 2026-11: as, bn, gu, hi, kn, kok, ml, mr, ne, or, pa, si, ta, te, he, ar, fa, ur, ps, ko, hy, ka); 19 ภาษา RTL/อินดิกอยู่ใน `IMPERFECT_RENDERING` แสดงคำเตือน OK/Cancel ก่อนใช้ และ Automatic ไม่เลือกให้

### ความปลอดภัย (port จาก WordCraft)
- รอบ 3 ชุด 1–25 อยู่ใน 1.1.3 (ดู CHANGELOG `[1.1.3]` › ความปลอดภัย); ตรวจ `cargo clippy --workspace --all-targets --exclude doc4me-web -- -D warnings` ผ่านหลังชุด 25
- port WordCraft: ครบแล้วทุกส่วนที่ใช้กับ Doc4Me (รวมใน 1.1.3 ที่ออกใหม่ 2026-11-11): `b2734c28` (undo ใช้ style sheet/list ร่วมกัน), `1350f62b` (AccessKit focus guard), `ef35d79c` (control channel/MCP ปฏิเสธคำสั่งที่ทิ้งงานที่ยังไม่บันทึก; ไม่พอร์ตส่วนหลายเอกสาร) และเพดานตารางซ้อนของ `560565cd`; ส่วน ODT ของ `560565cd` ไม่ต้องพอร์ต (ตัวอ่าน ODT ของ Doc4Me มีงบ `text:s` ของเนื้อหาอยู่แล้ว และไม่อ่านหัว/ท้ายกระดาษ) — commit ที่เหลือของ WordCraft ตามบันทึกสำรวจ 2026-11-09 ด้านล่างยังไม่ได้ replay
- แก้แล้ว (รวมใน 1.1.3 ที่ออกใหม่ 2026-11-11): จำกัดความลึกตารางซ้อนที่ 18 ชั้นเมื่อแทรก/วาง (`MAX_TABLE_DEPTH` ใน `crates/doc/src/table.rs`)
- วิธี port: 3-way merge รายไฟล์ด้วย `git merge-file` (ours = doc4me, base/theirs = WordCraft ก่อน/หลัง commit หลัง sed wordcraft→doc4me) ใน worktree แยก แล้วแก้ conflict โดยถือฝั่ง Doc4Me และเติมเทสต์; ถ้าโครงสร้างต่างมากให้เขียนใหม่ตามเจตนาของ commit

### ความคืบหน้า pagination หลังออก 1.1.3 (2026-11-11, ยังไม่ออกรุ่น; เป้า 100% ไม่นับ deferred แล้วจึงอัปเดตเอกสารทั้งหมดและออก 1.1.3 ใหม่ตามที่ผู้ใช้สั่ง)
- ผล Windows ล่าสุด: run 43001098341 (source `b859862`) **92.6% / 94.5%**, pages **183/222**, boundaries **97/222** (จาก 175/95 ก่อนรอบนี้)
- แก้แล้ว: ฟอนต์ธีม `majorBidi`/`majorEastAsia` (ใช้ `a:cs`/`a:ea` และฟอนต์รายสคริปต์ตาม `w:themeFontLang`); Helvetica/Arial Nova → Arial และ Times → Times New Roman บน Windows; โหมด 14 ไม่ส่ง paragraph mark หลัง page break ไปหน้าใหม่; โหมด < 15 ตัด space-before ของย่อหน้าแรกหลัง page break; ตารางไม่ snap document grid ถ้าไม่มี `w:adjustLineHeightInTable`
- ฟอนต์: ฝัง Carlito/Caladea/Arimo/Tinos/Cousine (OFL, static) ในรุ่นเดสก์ท็อป; variable font ของ Gelasio/Overpass/Open Sans/Roboto/Archivo/League Spartan ทำให้คลังแย่ลง จึงไม่ใส่; ฟอนต์ที่ฝังในเอกสาร (`word/fonts/*.odttf`) ยังไม่รองรับ (7 เอกสาร, ที่ได้ผลบน Windows จริงคือ zenodo-129)
- `cargo xtask readiness --diagnostics` พิมพ์แถว `fonts<TAB>doc<TAB>ขอ=แทน` ของเอกสารที่ไม่ผ่าน อ่านจาก log ของ job บน Windows (`get_job_logs`)
- วิธีวัดบน Linux ต้องใส่ `--cli target-port/release/doc4me-cli` (ไม่ใส่จะใช้ `target/release` เก่า); เทียบก่อน/หลังรายเอกสารทุกครั้ง ผลเอกสารที่ไม่ผ่านบน Linux 34/45 ตรงกับ Windows
- รอบต่อมา (2026-11-11): แก้เลขหน้าเริ่มที่ 0 (`w:pgNumType w:start="0"`), section ต่อเนื่องเริ่มใต้วัตถุลอยที่ข้อความต้องหลบ, ตัดบรรทัดหลังยัติภังค์ระหว่างตัวเลข, ลำดับฟอนต์ทั่วไปกลับไปใช้ฟอนต์ระบบก่อนฟอนต์ที่ฝัง; Windows run 43011248444 (source `426ecad`): **92.6% / 94.5%**, pages 182, boundaries 99 (zenodo-067 กลับไปไม่ผ่านเพราะเดิมผ่านโดยบังเอิญจาก Carlito ที่กว้างใกล้ Overpass ซึ่ง Word ใช้จริง; ลำดับใหม่ตรงกับที่ Word แทนฟอนต์ที่ไม่มี)
- ทดลองแล้วไม่ใส่: ฟอนต์ที่ฝังใน .docx (ถอดรหัส `.odttf` ด้วย `w:fontKey` ตาม §17.8.1 ได้ถูกต้อง, ลงทะเบียนด้วยชื่อ `w:name` เพราะ Word เปลี่ยนชื่อ subset เป็น `___WRD_EMBED_SUB_NN`) ทำให้ zenodo-102 และ zenodo-129 แย่ลงบน Linux (ระยะบรรทัดไม่สม่ำเสมอ น่าจะผสมฟอนต์ subset กับฟอนต์สำรองในบรรทัดเดียว) ต้องสืบเรื่องเมตริกก่อนใช้
- ข้อค้นพบที่ยังไม่แก้: govuk-001 และ zenodo-097 มีวัตถุลอยที่ anchor อยู่หลัง page break ในย่อหน้าเดียวกัน (Word วางในหน้าถัดไป, Doc4Me วางที่หัวย่อหน้า); govuk-026 Word ยกตารางย่อยไปหน้าใหม่ทั้งที่ใส่ได้ (ยังไม่พบเหตุใน XML); govuk-037 Word มีหน้าว่างหลัง page break + section continuous (ยังไม่พบเหตุ); govuk-034 เหลือเกิน 1 หน้า; zenodo-114 Word มีหน้าว่างก่อน section ที่เป็น nextColumn
- ต้องให้เจ้าของตัดสิน: zenodo-104 PDF อ้างอิงมีหน้าที่ไม่มีใน .docx (ปก, หน้าว่าง, ปกหลัง 4 หน้า) จัดหน้าอย่างไรก็ไม่ตรง; zenodo-149 Word ใช้ Arial Nova Cond Light/Arial Narrow ที่ไม่มีบน Windows runner และไม่มีตัวเทียบขนาดเปิด

### งานถัดไป
1. Pagination (มิติเดียวที่ยังไม่ 100% เมื่อไม่นับ deferred; ต้องผ่านจำนวนหน้า 222/222 และตำแหน่งตัดหน้า 222/222 ตอนนี้ 175 และ 95)
   - วิธีวิเคราะห์บน Linux (ใช้ได้จริง: เอกสารที่ไม่ผ่านบน Linux 34 จาก 45 ตรงกับ Windows เป๊ะ): `cargo xtask readiness --download` แล้ว `cargo xtask readiness --corpus --diagnostics --dry-run --jobs 4 --cli <target/release/doc4me-cli ที่เพิ่ง build>` (ถ้าไม่ใส่ `--cli` จะใช้ `target/release` เดิมซึ่งอาจเก่า); เทียบแต่ละหน้าด้วย `pdftotext -bbox` และ `doc4me-cli convert`
   - แก้แล้ว: zenodo-103 (ตารางในเอกสารที่มี `w:docGrid type=lines` แต่ไม่มี `w:adjustLineHeightInTable` ไม่ปัดเข้า grid)
   - พบแล้วว่าแก้ที่ layout ไม่ได้: zenodo-149 (41 เทียบ 28 หน้า) เพราะ PDF อ้างอิงของ Word ใช้ฟอนต์แทน Calibri ที่แคบกว่า Carlito ราว 15% (Calibri Light/Arial Nova Cond ฝังอยู่ในไฟล์) ตัวเลข 100% ของมิตินี้จึงอาจไปไม่ถึงโดยไม่มีฟอนต์ตรงกัน
   - ตัวถัดไปที่น่าตรวจ: zenodo-114 (81 เทียบ 71), zenodo-049 (52 เทียบ 43 บน Linux, 46 บน Windows), zenodo-104 (25 เทียบ 29 น้อยกว่า Word), govuk-026 (4 เทียบ 7)
2. replay commit WordCraft ช่วงที่เหลือตามลำดับ dependency (ดูบันทึกสำรวจ 2026-11-09 ด้านล่าง; รายการที่ค้างเฉพาะเจาะจงของรอบนี้ทำครบแล้ว)
3. ให้เจ้าของภาษาตรวจทาน 26 แค็ตตาล็อกฉบับร่าง AI; รองรับ shaping/RTL ใน UI

## บันทึกก่อนออก 1.1.3 (ข้อมูลเก่า; ตัวเลขล่าสุดอยู่ด้านบน)

### ผลการสำรวจ port WordCraft และเกณฑ์ใหม่ (2026-11-09)

- doc4me แยกจาก WordCraft ที่ `00ed5051`; ตั้งแต่นั้นถึง `1225f561` มี 1,596 commit (1,480 แตะโค้ด). ชุด 56 commit ที่ค้าง (เริ่มที่ `1225f561`) ต้องการ dependency เช่น reference-style links (`62533d21`, #3238) ที่ `79e3610c` เรียกใช้ และยังไม่มีใน doc4me. งานนี้จึงต้อง replay ช่วงกว้างตามลำดับ dependency ไม่ใช่แค่ 56 commit; ยังไม่เริ่มแก้โค้ดชุดนี้ (ทิ้งการแก้ครึ่งทางแล้ว).
- เกณฑ์ที่เจ้าของยืนยัน: ใช้ readiness **โดยไม่นับ 22 แถว deferred** เป็นเกณฑ์แทน 100% รวม. ยังรายงานตัวเลขรวมและตัวเลขไม่นับ deferred แยกกันเหมือนเดิม. Deferred ยังนับเป็นไม่ผ่านจนกว่าจะมีหลักฐานจากอุปกรณ์หรือบริการจริง ไม่มีหลักฐานดังกล่าวใน `veevie55/doc4me` หรือ `storytold/wordcraft`.
- ลำดับงานถัดไปที่เจ้าของสั่ง: (1) port WordCraft ตาม dependency แล้ว (2) ทำส่วนที่ไม่ใช่ deferred (ribbon Read Mode 2 รายการ, roundtrip govuk-047/zenodo-018, corpus-pages/breaks, ภาษา) (3) วัดบน Windows แล้วรายงานตัวเลขสองค่า.

### ผลวัด readiness ล่าสุด (2026-11-10)

- **92.2% รวม / 94.1% เมื่อไม่นับ 22 แถว deferred** จาก Windows run [42823797611](https://github.com/veevie55/doc4me/actions/runs/42823797611) (วัด source `89bb1a8`, 222 เอกสาร, report commit `9e93e8d`); ก่อนหน้า 91.3% / 93.2%
- มิติ: features 97.4% (ribbon 503/503, dialogs 216/228, 12 deferred), file formats 100% (222/222), pagination 60.6% (pages 174/222, breaks 95/222), UI/UX 98.6% (276/280, 4 deferred), stability 100%, performance 100%, **localization 100% (95/95; เดิม 72/95)**, platforms 100%, ecosystem 83.3% (1 deferred), hardware 37.5% (5 deferred)
- ภาษาครบ 95/95 แล้ว: แค็ตตาล็อกฉบับร่าง AI ครบ 3,034 คีย์ ลงทะเบียน 91 ภาษาใน `LANGUAGES`; 19 ภาษา RTL/อินดิกอยู่ใน `IMPERFECT_RENDERING` (แสดงคำเตือนก่อนใช้) ยังต้องให้เจ้าของภาษาตรวจทาน
- พบระหว่าง port `560565cd`: Doc4Me **ไม่มีขีดจำกัดความลึกตารางซ้อน** สำหรับ `insert.table`/วาง (WordCraft จำกัด `MAX_TABLE_DEPTH` = 18 เพื่อไม่ให้ stack overflow ตอนบันทึก/คัดลอก และให้ `.json` อ่านกลับได้ เพราะ serde_json อ่านซ้อนได้ 127 ชั้น); DOCX reader จำกัดที่ 24 อยู่แล้ว (`crates/docx/src/read/story.rs`) — ควรเพิ่มเพดานใน `crates/doc` และให้ insert/paste หยุดที่เพดาน พร้อมเทสต์ `tables_nest_only_as_deep_as_a_json_save_reads`
- Windows run [42836913553](https://github.com/veevie55/doc4me/actions/runs/42836913553) สั่งวัดหลังชุด 23 (`1352238`) เพื่อยืนยันว่าไม่ถดถอย; workflow จะ commit `docs/readiness.md` เอง ให้ดูผลก่อนสรุปตัวเลขใหม่
- ส่วนที่ไม่ deferred และยังไม่ 100% เหลือเพียง pagination (ห่าง ~5.9 จุดของคะแนนรวม)
- Port ความปลอดภัย WordCraft รอบ 3 ต่อแล้วชุด 9–22 (ดู CHANGELOG › ความปลอดภัย). ยังเหลือ: `560565cd` ส่วนที่เหลือ (ขีดจำกัดตาราง ODT, ความลึกตารางเทียบ JSON save), `b2734c28` (undo ใช้ style sheet/list ร่วมกัน: 1.2 GB → 21 MB, conflict มาก), `1350f62b` (AccessKit focus guard), `ef35d79c` (control channel ปฏิเสธคำสั่งที่ทิ้งงานที่ยังไม่บันทึก). วิธี: 3-way merge รายไฟล์ด้วย `git merge-file` (ours = doc4me, base/theirs = WordCraft ก่อน/หลัง commit หลัง sed wordcraft→doc4me) แล้วแก้ conflict โดยถือฝั่ง Doc4Me และเติมเทสต์; ถ้าโครงสร้างต่างมาก ให้ implement เจตนาของ commit ใหม่บนโค้ด Doc4Me. ข้ามแล้วเพราะ Doc4Me ไม่มีฟีเจอร์หรือมีอยู่แล้ว: `1225f561 4963e30e c7148d6b bbe1dad1 e541a6f6 b246ea16 68d45318 4c5930da 639964b0 6dc062d1 b016ecc2 9b91b64d cbe6a9d5 48545479 729cd0a5` (`b3da7a4d` นำหลักการไปใช้กับ Print Preview แทน)

### สรุป readiness (บันทึก 2026-11-09 ก่อนทำงานค้างต่อ; ยังไม่ได้วัดใหม่)

- **91.3% รวม / 93.2% เมื่อไม่นับ 22 แถว deferred** — ผลอย่างเป็นทางการล่าสุดจาก Windows run [42334133340](https://github.com/veevie55/doc4me/actions/runs/42334133340) (วัด source `1868cb4`, 222 เอกสาร, 2026-11-07) ตาม `docs/readiness.md`; คำนวณถ่วงน้ำหนักซ้ำจากตารางได้ 91.27% และ 93.15% ตรงกับรายงาน
- ห่าง 100% อยู่ 8.7 จุด: pagination 5.9 จุด (page counts 174/222, boundaries 96/222 → 60.8%), deferred 22 แถว 1.9 จุด (dialogs 12, shortcuts IME/Alt 4, Mendeley 1, hardware 5), localization 1.0 จุด (72/95 → 75.8%); มิติอื่น (file formats, stability, performance, platforms, ribbon 500/500) อยู่ที่ 100%
- ตัวเลขนี้**ยังไม่สะท้อน** commit หลัง `1868cb4`: port จาก WordCraft (141 + 25 fixes) และงานความปลอดภัยรอบ 3 ชุด 1–8 ซึ่งชุด 8 ยังไม่ผ่าน clippy/test/wasm ครบ และ owner เขียนทับ `main` (`f154ab5`, history ใน clone ตื้นจึงหา `1868cb4` ไม่พบ) — ต้องรัน `readiness-windows.yml` ใหม่หลังตรวจชุด 8 ผ่าน จึงจะยืนยันได้ว่าคะแนนไม่ถดถอย

### งานค้าง: port งานความปลอดภัยจาก WordCraft (บันทึก 2026-11-09, หยุดเพราะถึงลิมิต)

- ทำแล้ว: รอบที่ 3 ชุดที่ 1–8 อยู่บน `main` (ดู CHANGELOG `[Unreleased]` › ความปลอดภัย). ชุดที่ 8 ตรวจครบแล้ว 2026-11-09 (clippy, test, wasm ผ่าน; ดู CHANGELOG) — ขั้นตอนตรวจเดิมสำหรับรอบถัดไป: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --exclude doc4me-web --no-fail-fast`, `cargo xtask wasm` (env: `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, ดิสก์มีจำกัด).
- วิธี port ที่ใช้: ต่อ commit ของ WordCraft (`veevie55/wordcraft`) ทำ 3-way merge รายไฟล์ (base = ไฟล์ก่อน commit, ours = doc4me, theirs = หลัง commit) หลัง sed เปลี่ยนชื่อ wordcraft→doc4me/WordCraft→Doc4Me; conflict ให้ใช้ฝั่ง doc4me แล้วเติมเทสต์ใหม่ของ commit นั้นเข้า `mod tests` เพื่อพิสูจน์พฤติกรรม; build+test crate ที่แก้; ถ้าใช้ฟีเจอร์ที่ doc4me ไม่มี (clip เซลล์, dash ของ Draw::Path, proofing รัสเซีย/ตุรกี, Word 2003 XML writer, unmodelled parts store, ตัวคั่น "other" ใน Convert to Text) ให้ข้าม; doc4me ใช้ zip 9. สคริปต์อยู่ใน scratchpad ที่หายเมื่อ session จบ.
- ค้างที่ `1225f561` (render): build ไม่ผ่านเพราะขาด `MAX_GLYPH_PIXELS` และ `OVERSIZED_GLYPHS_NOTED` — หา commit ที่นิยามใน WordCraft (`git log -S MAX_GLYPH_PIXELS`) แล้ว port ก่อน หรือข้าม.
- ที่เหลือ 56 commit ตามลำดับ: `1225f561 42c254c8 89a64d1e f6398e29 79e3610c 41de4739 582f4118 4963e30e 8efba59b b578be04 0f9b9264 1ae38ec2 32e4900b 3cffe22c 6c632648 b246ea16 bcce02ab c7148d6b ce7e4ea2 68d45318 bbe1dad1 e541a6f6 57e9e0f2 2337ff75 12afc754 4c5930da 60a64ace 639964b0 722afa42 364688a8 6dc062d1 c14e18cf 01a75b99 3a1c91f2 acfc3d50 f2352086 26187afa bec112a4 932a2e89 9b91b64d 48545479 f2debdec a4b1682b 028d413a e00ff04d 1350f62b 560565cd b2734c28 cbe6a9d5 b3da7a4d 96286f0e ef35d79c 729cd0a5 e7668ae5 fa59ed0e b016ecc2`.
- ข้ามแล้ว (doc4me ไม่มีฟีเจอร์ หรือดีกว่าอยู่แล้ว): รอบ 3 `5eae7b4c 3a49688c 02c7a926 59293747 2cff091c 34285531 ddb67eaa 0aae9d05 2a647661`; รอบ 2 `bf70f48b 7aea5f8c 1c8c352c 8d16b266 6e8a420d 66ff2afc 51e364fe 7ee101a5 6142567b`.
- หลัง port ครบ: อัปเดต ROADMAP.md, handoff.md, docs/*parity*, docs/readiness* และไฟล์ที่มี % ให้ตรงกับโค้ดจริง แล้ว commit/push `main` พร้อม CHANGELOG.

- Windows remeasurement requested after code commit `1868cb4`: run [42334133340](https://github.com/veevie55/doc4me/actions/runs/42334133340) completed successfully on all 222 documents; report commit `ed787f9` confirms the same scores and records exact page-boundary word pairs.

- งานพร้อมกันรอบ 2026-11-07: แก้ IME commit ลง text box ที่เลือกโดยรักษา body anchor และ file-filter labels ที่แปล glob เสีย 13 รายการ พร้อม parser validation. ผล Windows เดิมยังเป็น 91.3% / 93.2%; Windows run 42334133340 วัด patch นี้แล้ว โดยคะแนนเท่าเดิม.
- ผู้ใช้สั่งดำเนินการ deferred แล้ว: [แผนทำให้ผ่านทั้ง 22 แถว](docs/readiness/deferred-plan.md) ระบุงาน implementation และหลักฐานที่ต้องใช้. Native printing, pen-pressure pipeline และ scanner/camera acquisition ยังขาดโค้ดด้วย; OLE/ActiveX/Outlook/legacy Mac ต้องมี native server ส่วน Mendeley ต้องมี client integration กับ OAuth/test library. ไม่มีการเปลี่ยนสถานะเป็น pass จากคำสั่งหรือ synthetic test เพียงอย่างเดียว.

- ผล Windows readiness อย่างเป็นทางการล่าสุด: run [42334133340](https://github.com/veevie55/doc4me/actions/runs/42334133340), วัด source `1868cb4`, report commit `ed787f9`: **91.3% รวม**, **93.2% เมื่อตัด 22 deferred**. มิติ: features 97.4% (ribbon 500/500; dialogs 216/228, 12 deferred และ 7 excluded), file formats 100% (เปิดและ round-trip 222/222), pagination 60.8% (174/222 page counts, 96/222 boundaries), UI/UX 98.6% (shortcuts 276/280, 4 deferred), stability 100% (222/222 และ 5/5), performance 100% (6/6), localization 75.8% (72/95), platforms 100% (5/5), ecosystem 83.3% (5/6, 1 deferred), hardware 37.5% (3/8, 5 deferred). Dialog และ shortcut ทุกแถวที่ไม่ deferred ผ่าน.
- Language readiness checks catalog key presence against the union of unique TSV keys; it does not measure translation quality. The official report is 72/95 (75.8%); 69 choices include English plus 68 TSV catalogs, each with 3,031 parsed keys. Six untracked drafts (ar, bn, fa, he, ps, ur) also have 3,031 keys and no blanks but are unsafe to register: bn mistranslates “Save” and “Bold”; RTL drafts break shortcuts/file extensions or insert Tab into “Table”. The 23 missing culture rows comprise these six and 17 without drafts; RTL UI support and human review are still required.
- Pagination: the latest official Windows run has 174/222 page counts and 96/222 boundaries, unchanged from the previous official report. govuk-005 matches 36 pages but fails the page-12 boundary check; file.fontSubstitution is empty there; the new diagnostic records Doc4Me first="do", Word first="yes", and both last="account". govuk-067 is 15 pages versus Word 16; its DOCX alternate declares Calibri while the reference embeds Tahoma, so overriding it would be speculative. The following-section marker trial was reverted: on Linux, affected15 page counts improved10→11 but boundaries fell5→4; zenodo-010 body moved another12.65pt away from Word. After rollback, the rebuilt CLI reproduced the baseline exactly on all15 documents (10 page-count passes/5 boundary passes). Investigate column balancing before changing marker height. The xtask diagnostic logged exact first/last-word pairs in run 42334133340. Linux corpus results 164/222 and 82/222 remain exploratory.
- UI/hardware: เพิ่ม regression tests ให้ OS-provided printable text ถูกแทรกแม้ Alt ค้าง และ IME preedit ไม่ commit ก่อน `ImeEvent::Commit`; เพิ่มการเทียบขนาดจอแบบ scale-aware ตอนกู้ตำแหน่งหน้าต่าง พร้อม tests. สิ่งเหล่านี้ยังไม่เปลี่ยนสถานะ physical IME/multi-monitor checks ซึ่งต้องวัดบน Windows จริง.
- Windows ARM64: run [42213688289](https://github.com/veevie55/doc4me/actions/runs/42213688289) ผ่าน `/EHsc` แต่ล้มตอน link ด้วย unresolved `__imp_*` CRT 23 รายการจาก whisper/ggml. หลังตั้ง CMake `CMP0091=NEW` และ `CMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded` ให้ใช้ static CRT ตรง Rust `+crt-static`, run [42262956358](https://github.com/veevie55/doc4me/actions/runs/42262956358) build และ verify MSI กับ portable ZIP สำเร็จ; `manual.tsv` ปัจจุบันจึงบันทึก `plat-002` เป็น `pass` (SHA-256 อยู่ในแถว). แพ็กเกจ unsigned และไม่ได้ upload หรือ publish.
- Audit deferred: ปัจจุบันมี 22 แถว — 12 dialogs เป็นบริการ/ส่วนประกอบ legacy ของ Windows, Office, OLE, ActiveX, Outlook/Exchange หรือ Mac; 4 shortcuts เป็นการพิมพ์รหัส Alt และการสลับ IME ที่ระบบปฏิบัติการเป็นเจ้าของ; อีก 6 manual checks คือ Mendeley Cite ซึ่งต้องใช้ application registration/OAuth/account/test library และ printer, pen, microphone, scanner กับหลายจอที่รออุปกรณ์จริงของเจ้าของ. `hw-003` มีโค้ดแก้ DPI scale comparison และ tests แต่ยังไม่ใช่ผลทดสอบหลายจอจริง.
- Localization drafts: `ig`, `tn`, `wo` และ `yo` ลงทะเบียนใน i18n tree และนับใน catalog-key coverage แล้ว แต่เป็น AI-generated และยังต้อง native-speaker review. Readiness ไม่วัดคุณภาพคำแปล.
- Docs metric audit: current working-tree counts are 727 registered command specs (current rebuilt CLI verified), 2,071 standalone test attributes, 216,629 Rust lines across 374 files, 68 catalogs with 3,031 unique keys each, and a shortcut registry of 187 strings / 215 alternatives. Readiness and parity docs were checked against their source sets. The current rebuilt CLI reports parity 500/500; its markdown output matches docs/parity-checklist.md byte-for-byte. An earlier linker Bus error caused by a full overlay was resolved by clearing generated cache before the successful rebuild. cargo test -p xtask passed 32/32, cargo fmt --all --check passed, and cargo xtask layers passed for all 20 packages.
- UI script fallback: ฝัง Noto regular/bold faces สำหรับ 14 สคริปต์และ Hangul subset (รวม 8,050,250 bytes, OFL) ต่อท้าย proportional, medium, semibold และ monospace. Font-coverage regression ตรวจ fallback charmap owner; Arabic/Hebrew glyphs มีฟอนต์แต่ RTL/bidi ยังไม่รองรับใน epaint. `cargo xtask assets` ผ่าน 112/112.
- Full CI on the current working tree passed all six steps: fmt, clippy, workspace tests, assets, dependency layers and wasm. UI tests passed 441/441, xtask tests 32/32, and assets 112/112. The clean cloud runtime lacked ALSA development files and CMake, and libclang could not find GCC's `stdbool.h`; temporary files under `/tmp` plus `PKG_CONFIG_SYSROOT_DIR`, `PATH`, and `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/14/include` enabled the run without changing repository or system configuration. The new local diagnostic logs exact page-boundary word pairs. The latest official Windows report remains run 42334133340; the diagnostic ran successfully on Windows and the official scores are unchanged. This handoff and status-file audit are included in the requested main-branch commit; no release, package upload or environment publication was made.

## สถานะเดิมและบันทึกการทำงาน

### ความคืบหน้าสู่ 100% ไม่นับ deferred (ข้อมูลก่อนรอบที่ 23; อ่านตัวเลขล่าสุดด้านบน)

- **เจ้าของเขียนทับ `main` ของ fork อีกครั้ง (2026-11-04 02:02–02:37 UTC)**: `main` ใหม่ = `d084ccc` (เนื้อไฟล์เท่า `c9ff26b` เดิม, tag `v1.1.3` ชี้ที่นี่) + 6 commit "Release 1.1.3"/"Initial commit" ที่แก้แค่ `Cargo.toml` (1.1.3) และ `Cargo.lock`; commit 65 รายการหลัง `c9ff26b` (dialogs รอบที่ 21, 40 ภาษา, ตัดบรรทัด LB15, แก้ readiness xtask ฯลฯ) หายจาก `main` จึง rebase ต่อบนงานของเจ้าของแบบ fast-forward (ไม่ force; ต้นไม้ต่างจากก่อน rebase แค่เลขเวอร์ชัน) และแก้ `Cargo.lock` ให้ตรง 1.1.3 (`3030a22`); hash ของ commit รอบนี้จึงเปลี่ยนทั้งหมด ถ้าเจ้าของตั้งใจตัดงานหลัง `c9ff26b` ทิ้ง ต้องตัดสินใหม่
- ลองแก้ "ด้านที่ `w:tblCellMar`/`w:tcMar` ไม่ระบุให้สืบทอดจาก style" แล้ว**ย้อนออก**: zenodo-154 แสดงว่า Word ใช้ 0 กับด้านที่ `w:tblCellMar` ไม่ระบุ (บรรทัด "LEVEL 1 USE means that" ตรงเมื่อใช้ 0) และไม่แก้ govuk-006; สาเหตุจริงของ govuk-006 (หน้า 5 มีแค่ย่อหน้า "© Crown copyright" ที่ Word ใส่ท้ายหน้า 4) ยังไม่พบ — แถวสูงของเราเกิน Word ~16 pt ที่หน้า 4
- Windows run 41673874009 (`4556010`, 16 ภาษาใหม่): **88.4%**, **90.9%** เมื่อไม่นับ deferred, ภาษา 30 → 46/95 (มิติอื่นเท่าเดิม); หลัง `a128314` มี 62/95 (ha ky tt uz lb gd mi rm เพิ่ม)
- `24dcde5`: ตัดบรรทัดหลัง `” ` ก่อน `(` (Unicode 15.1 ตัด LB15 แต่ `unicode-linebreak` 0.1.5 ยังใช้) แก้ zenodo-025 (Linux: pages 172 → 173, breaks 91 → 92 ไม่มีเอกสารแย่ลง) และเพิ่มอีก 8 ภาษา (kk mk ms mt nn pt-pt cy sw → 54/95) สั่ง Windows run ถัดไปแล้ว
- กำลังแปล (subagent): ha ky tt uz lb gd mi rm; ที่เหลือซึ่ง Inter วาดได้: ig xh zu rw nso tn wo yo sr-cyrl-ba (ekavian/ijekavian?) ca-valencia; ภาษาที่ต้องฟอนต์/shaping ใหม่ (ar he fa ur ps hi mr ne kok as bn-BD bn-IN gu pa or ta te kn ml si hy ka ko) ยังไม่เริ่ม — ต้องให้ UI วาดได้ก่อน
- เครื่องมือแปลอยู่ใน scratchpad ของ session (หายเมื่อ session จบ): `BRIEF.md`, `check_catalog.py` (ตรวจคีย์ตรง `en-gb.tsv`, placeholder, "…", glyph ใน Inter), `register.py` (เพิ่มใน `LANGUAGES`, `WHOLE_CATALOGS`, `languages.tsv` และแก้ความยาว array)
- วิธีวัด pagination บน Linux: แก้ `take(40)` ใน `xtask/src/readiness.rs` เป็นค่าใหญ่ชั่วคราว (อย่า commit) แล้ว `cargo xtask readiness --corpus --jobs 8 --cli <cli ที่ build ใน CARGO_TARGET_DIR=target-fix>` แล้ว `git checkout docs/readiness.md`; baseline บน Linux ที่ `f65cff1`: pages 172, breaks 91

### Readiness ล่าสุด (บันทึก 2026-11-03 รอบที่ 22, ยืนยันด้วย run 41608625168): **87.7%**, **90.3%** เมื่อไม่นับ 26 แถว deferred

- **วัดซ้ำ 2026-11-03 (run 41608625168, `windows-latest`, วัด `main` ที่ `3418736`, commit ผล `51dab2f`, job 09:08–09:25 UTC)**: ได้ **87.7%** / 90.3% เท่าเดิมทุกมิติ (ตารางด้านล่างยังถูกต้อง); หัวรายงานเป็น `3418736` **ไม่มี `+changes` แล้ว** ยืนยันการแก้ใน `83b1e0c`; ดาวน์โหลดคลังครบในรอบนี้ (zenodo กลับมาตอบปกติ)
- ที่มาเดิม: `docs/readiness.md` จาก Windows run 41596836985 (`windows-latest`, วัด `main` ที่ `24ad50b`, commit ผล `bc99ca1`; คืนไฟล์นี้ใน `83b1e0c` หลัง run 41599589867 commit ผลที่ไม่ครบ 60.8% ทับ) ห่างเป้าหมาย 100% อยู่ 12.3 จุด
- รายมิติ (คะแนน / ไม่นับ deferred / ชุดตรวจ):

| มิติ | น้ำหนัก | คะแนน | ไม่นับ deferred | ชุดตรวจ |
|---|---|---|---|---|
| Features | 0.30 | 94.7% | 97.2% | ribbon 499/499; dialogs 204/228 (deferred 12) |
| File formats | 0.20 | 100.0% | 100.0% | corpus-open 222/222; corpus-roundtrip 222/222 |
| Layout and pagination | 0.15 | 60.1% | 60.1% | corpus-pages 171/222; corpus-breaks 96/222 |
| UI/UX | 0.10 | 95.7% | 98.2% | shortcuts 268/280 (deferred 7) |
| Stability | 0.10 | 100.0% | 100.0% | corpus-stability 222/222; stability 5/5 |
| Performance | 0.05 | 100.0% | 100.0% | performance 6/6 |
| Localization | 0.04 | 31.6% | 31.6% | languages 30/95 |
| Platforms | 0.03 | 80.0% | 100.0% | platforms 4/5 (deferred 1) |
| Ecosystem | 0.02 | 83.3% | 100.0% | ecosystem 5/6 (deferred 1) |
| Hardware | 0.01 | 37.5% | 100.0% | hardware 3/8 (deferred 5) |

- ที่ยังไม่ผ่านและไม่ deferred: dialogs 12 แถว (รอเจ้าของตัดสินว่าจะ defer หรือไม่ ดู "ข้อค้นพบที่ต้องให้เจ้าของ repo ตัดสิน"), shortcuts 5 รายการ (`ks-165`, `ks-177`, `ks-203`, `ks-219`, `ks-329`), corpus-pages 51 และ corpus-breaks 126 เอกสาร, ภาษา 65 ภาษา
- แก้ข้อมูล: deferred 26 แถวแบ่งเป็น dialogs 12, shortcuts **7**, ecosystem 1, hardware 5, platforms 1 (บรรทัดเดิมด้านล่างที่เขียน shortcuts 6 รวมได้ 25 ไม่ตรง)
- ป้าย `24ad50b+changes` ของรายงานก่อนหน้ามาจากขั้น `--download` ที่เขียน `docs/readiness.md` ก่อนขั้นวัด ไม่ได้แปลว่าวัดจากโค้ดที่ไม่ตรง commit (แก้ใน `83b1e0c` และยืนยันแล้วด้วย run 41608625168)

ผู้ใช้สั่ง (2026-11-02): "อ่าน handoff.md แล้วดำเนินการต่อ" (งานที่ค้างคือการออก v1.0.0, v1.1.0, v1.1.1 แล้วจึง v1.1.2 ของรอบที่ 19) ระหว่างทำ ผู้ใช้เลือกให้ออก Release ใน fork และเปิด Actions ของ fork เอง; หลังออกรุ่นผู้ใช้สั่งไม่เปลี่ยนลิงก์ Releases ใน README แล้วสั่ง "ทำงานต่อจากงานต่อไปใน handoff"

### ประวัติผลวัด Windows รอบที่ 21–22 (2026-11-03; เริ่มที่ **86.7%** จาก run 41565289539 ที่ `8b685b2` ยืนยันตัวเลขของ run 41559739417; ล่าสุดอยู่ในหัวข้อด้านบน)

- run 41596836985 (เช็กเอาต์ `main` ตอนเริ่ม job คือ `24ad50b` ซึ่งรวม `f5f32df` Web Options `dlg-232` และ `4c69617` Hangul Hanja `dlg-194`; `head_sha` ของ run เป็น `3a0372e` แต่ workflow ใช้ `ref: main` จึงวัด `main` ณ เวลาที่ job เริ่ม ไม่ใช่ commit ที่สั่ง): **87.7%**, **90.3%** เมื่อไม่นับ deferred, ฟีเจอร์ 94.7%, dialogs 202 → 204/228; คลังเท่าเดิม (corpus-pages 171, corpus-breaks 96, open/round-trip 222/222); commit ผล `bc99ca1`
- รอบที่ 22 (2026-11-03, session ต่อจาก handoff): `f05eeb3` (1) xtask พิมพ์รายการ `git status --porcelain` ลง stderr เมื่อหัวรายงานเป็น `+changes` เพื่อหาสาเหตุที่ทุกรายงานจาก Windows runner เป็น `+changes` (ตรวจในโค้ดแล้วไม่พบตัวเขียนไฟล์ที่ git ติดตาม: xtask เขียนแค่ `docs/readiness.md` หลังอ่านสถานะ, build script เขียนลง `OUT_DIR`, ไม่มี symlink/CRLF ใน index/ชื่อไฟล์ที่ Windows ใช้ไม่ได้, `cargo metadata --locked` ผ่าน; log ของ runner มีแค่คำเตือน LF→CRLF ของ `docs/readiness.md` ตอน commit) (2) `cargo fmt --all` ของโค้ดรอบก่อนที่ทำให้ `cargo fmt --all --check` ไม่ผ่าน (CI ไม่ได้ตรวจ fmt); สั่ง run 41599589867 ต่อคิวไว้ (วัด `main` หลัง `f05eeb3`) ผล run 41599589867: **60.8% ไม่ใช่ผลจริง** — zenodo.org ตอบ 503 ไป 344 ไฟล์แต่ `--download` คืนสำเร็จ จึงวัดแค่ 50 เอกสาร และ bot commit `77fd3b8` ทับ; log ยืนยันว่า `+changes` มาจาก `docs/readiness.md` ที่ขั้น `--download` เขียนไว้ก่อน; แก้ในรอบนี้ (ดู `CHANGELOG.md`): `--download` ล้มเมื่อไม่ครบ (workflow จึงลองซ้ำ/ล้มก่อน commit), สั่งเดี่ยวไม่เขียนรายงาน, การตรวจ `+changes` ไม่นับรายงานเอง, และคืน `docs/readiness.md` เป็นผลของ `bc99ca1` (87.7%); ตัวเลขปัจจุบันจึงยังเป็น 87.7%; รอบถัดไปที่สั่ง workflow ควรได้หัวรายงานไม่มี `+changes`; ยังไม่ได้แก้ `.github/workflows/readiness-windows.yml` ให้ปฏิเสธการ commit รายงานที่มี "Not final" (เป็นชั้นป้องกันเพิ่ม ไม่จำเป็นหลังแก้ `--download`)
- run 41594385545 (`d37e836`, รวม `bc4fe56` Edit Links to Files `dlg-021`): **87.6%**, 90.1% เมื่อไม่นับ deferred, ฟีเจอร์ 94.3%, dialogs 201 → 202/228; คลังเท่าเดิม (corpus-pages 171, corpus-breaks 96, open/round-trip 222/222); รอวัดใน run 41596836985: `f5f32df` (Web Options, `dlg-232`, เปลี่ยนการเขียน .docx: เพิ่ม `webSettings.xml`) และ `4c69617` (Hangul Hanja Conversion, `dlg-194`) (บน Linux dialogs 204/228)
- run 41592240746 (`40b5402`, รวม `dd16ed6` Copy File/Find Files `dlg-007`/`dlg-032`/`dlg-152` และ `0a41629` Letter Wizard `dlg-115`): **87.5%**, **90.1%** เมื่อไม่นับ deferred, ฟีเจอร์ 94.1%, dialogs 197 → 201/228; คลังเท่าเดิม (corpus-pages 171, corpus-breaks 96, open/round-trip 222/222)
- run 41590861975 (`bdfba2d`, รวม `7da7541` Show Repairs `dlg-153`): **87.3%**, 89.8% เมื่อไม่นับ deferred, ฟีเจอร์ 93.2%, dialogs 196 → 197/228; คลังเท่าเดิม (corpus-pages 171, corpus-breaks 96)
- run 41589001124 (`d40ff9b`, รวม `ddb1c52` File locations `dlg-209`): **87.2%**, 89.7% เมื่อไม่นับ deferred, ฟีเจอร์ 93.0%, dialogs 195 → 196/228; คลังเท่าเดิม (corpus-pages 171, corpus-breaks 96)
- run 41586915428 (`895d0cc`, รวม `8b92537` Bidirectional text `dlg-205` และ `5d6add8` Manage Fields `dlg-200`): **87.1%**, 89.6% เมื่อไม่นับ deferred, ฟีเจอร์ 92.8%, dialogs 193 → 195/228; คลังเท่าเดิม (corpus-pages 171, corpus-breaks 96)
- run 41585979571 (`5914683`, รวม `1ff4c31` Japanese Consistency Checker, `dlg-003`): **87.0%**, 89.5% เมื่อไม่นับ deferred, ฟีเจอร์ 92.3%, dialogs 192 → 193/228; คลังเท่าเดิม (corpus-pages 171, corpus-breaks 96, round-trip 222/222)
- run 41584013643 (`b3bf68f`, Japanese Find: Sounds like (Japanese) และ Find Options (Japanese), `dlg-210`): **86.9%**, 89.4% เมื่อไม่นับ deferred, ฟีเจอร์ 92.1%, dialogs 191 → 192/228; คลังเท่าเดิม (corpus-pages 171, corpus-breaks 96, round-trip 222/222)
- run 41582181218 (`0ebfb7d`, Options › Typography, `dlg-218`): **86.9%**, 89.4% เมื่อไม่นับ deferred, ฟีเจอร์ 91.9%, dialogs 190 → 191/228; คลังเท่าเดิม
- run 41579621418 (`293f915`, Fit Text และ Two Lines in One): **86.8%**, 89.3% เมื่อไม่นับ deferred, dialogs 188 → 190/228; คลังเท่าเดิม (corpus-pages 171, corpus-breaks 96, round-trip 222/222); `0ebfb7d` (Options › Typography, `dlg-218`) push แล้ว รอผลวัด Windows run ถัดไป (บน Linux dialogs 191/228)
- งานรอบนี้ (Opus): `e184943` freeform ในกลุ่มรูปร่าง, `293f915` Asian Layout (Fit Text `dlg-047`, Two Lines in One `dlg-229`; Horizontal in Vertical `dlg-083` เก็บค่าได้แต่ยังไม่วาด เพราะ layout ไม่รู้ทิศทางข้อความ), `0ebfb7d` Typography (`dlg-218`) ทั้งหมดมีเทสต์และคำแปล 28 ภาษา รายละเอียดใน `CHANGELOG.md`
- หมายเหตุ build ใน session: ดิสก์ของ session เต็มได้จาก `target/debug/incremental` และไฟล์ทดสอบซ้ำหลายรุ่น ให้ใช้ `CARGO_INCREMENTAL=0` และลบ `target/debug/incremental` เมื่อเต็ม
- run 41569447798 (`e184943`, หลังแก้ freeform ในกลุ่มรูปร่าง) วัดได้ 86.7% เท่าเดิม (corpus-pages 171/222, corpus-breaks 96/222) เพราะการแก้เปลี่ยนการวาด ไม่ใช่การจัดหน้า
- รายการ deferred ทั้ง 26 แถวพร้อมเหตุผลอยู่ใน `docs/readiness.md` หัวข้อ `## Deferred rows (26)` (dialogs 12, shortcuts 6, ecosystem 1, hardware 5, platforms 1)
- run 41559739417 ล้มครั้งแรกที่ขั้น poppler (feed ของ Chocolatey ตอบ 406) แก้ด้วยการลองซ้ำ `95e46b1` แล้วผ่าน

- ที่มา: workflow `.github/workflows/readiness-windows.yml` (สั่งเองจากแท็บ Actions ด้วย workflow_dispatch) รัน `cargo xtask readiness --download` (ลองซ้ำได้ 3 ครั้ง) แล้ว `--corpus --jobs 4`; run สำเร็จ ใช้เวลาราว 22 นาที; commit ผล `cab15d4` บน `main` (`docs/readiness.md`)
- คะแนน: 86.7% รวม, 89.2% เมื่อไม่นับ 26 deferred (เดิม 86.0% และ 88.5%); ฟีเจอร์ 91.2% (dialogs 188/228), การจัดหน้า 60.1% (corpus-pages 171/222, corpus-breaks 96/222), UI/UX 95.7%, ภาษา 31.6% (30/95)
- ข้อสังเกต: หัวรายงานเขียน `at commit b6d3056+changes` ซึ่งหมายความว่า xtask เห็นต้นไม้งานไม่สะอาดตอนวัด (น่าจะเป็นไฟล์ที่ดาวน์โหลด/สร้างใน runner) ยังไม่ได้ตรวจว่าเกิดจากอะไร จึงควรดูก่อนถือว่าตัวเลขสะอาด
- ROADMAP.md และ docs/roadmap.md อัปเดตตามตัวเลขนี้แล้ว

### รอบที่ 21 (2026-11-03, Haiku): govuk-005 ตรวจแล้ว ยังไม่แก้ส่วนที่เหลือ

- ผลของ `a351c88` (ฟิลด์ตัดบรรทัดได้) ทำให้ govuk-005 จาก 40 หน้าเหลือ 37 หน้า (Word 36) จากการวัดก่อน/หลังบน Linux; หน้า 1–12 ของผลเราตรงกับ PDF อ้างอิงทั้งลำดับหัวข้อและหน้า
- ส่วนที่เหลือ: หน้า 12 ของผลเราไม่ตรงกับอ้างอิงในส่วน 11B (Section 11B "Monetary assets abroad") ข้อความ "Provide details here" กับ "Yes ☐ No ☐" ซ้อนทับกันบางแถว และมีบรรทัด "Provide details here" ตกค้างสามบรรทัดขึ้นหน้า 13 ที่อ้างอิงไม่มี
- สาเหตุที่ตรวจแล้ว: ย่อหน้าเหล่านี้เป็นข้อความธรรมดา ไม่มี FORMTEXT/ffData/sdt ในไฟล์ จึงไม่เกี่ยวกับการแก้ฟิลด์; ตารางของส่วน 11B ตามด้วยย่อหน้าหัวข้อที่มี `wp:anchor` เป็น `wpg:wgp` (กลุ่มรูปร่าง, `wrapNone`, `positionV` ติดย่อหน้า) ซึ่งวาดเส้นกรอบทับตาราง; การซ้อนทับน่าจะมาจากการวางกลุ่มรูปร่างนี้ ยังไม่ได้พิสูจน์
- ข้อค้นพบเพิ่มเติม (รอบที่ 21, หลังวัด Windows): ใน govuk-005 ข้อความ "Provide details here" ของส่วน 11B เป็น **กล่องข้อความลอย** (`wps` ใน `wp:anchor`, `wrapTopAndBottom`, `positionV` ติดย่อหน้า) สามกล่องเรียงกันที่ x≈4003675 EMU ห่างกันประมาณ 548k EMU สูงกล่องละ 346075 EMU; กล่องแบบนี้ควรดันบรรทัดลงตามที่ Word ทำ แต่ผลของเรายังซ้อนทับ ทำให้หน้า 12 แน่นกว่า Word จนมีบรรทัดตกไปหน้า 13 งานต่อไปคือเปรียบเทียบ y ของกล่องเหล่านี้กับ PDF อ้างอิง (`pdftotext -bbox`) แล้วดูว่า `wrapTopAndBottom` ของ anchor ที่ติดย่อหน้าในเซลล์ตารางถูกคิดความสูงถูกหรือไม่
- ตำแหน่งที่วัดด้วย `pdftotext -bbox` (รอบที่ 21, ผลของเราบน Linux เทียบ PDF อ้างอิง, หน่วย pt): หน้า 1 ของเราเริ่มส่วน 11B ที่ y≈491 ส่วนอ้างอิง y≈477 (ต่าง 14 pt ตั้งแต่ก่อนหน้านั้น); แถว "Do you have a PayPal account" ของเราอยู่ y≈591 อ้างอิง y≈561 และ "Yes ☐ No ☐" ของเราอยู่ที่ x≈322/389 (คอลัมน์ Provide) ขณะที่อ้างอิงอยู่ x≈192/258 (คอลัมน์ Yes/No) จึงซ้อนกับ "Provide details here" ที่ x≈319; กลุ่มรูปร่าง `wpg` ที่ตำแหน่ง x≈192.6 pt (`positionH` 2446020 EMU) ตรงกับคอลัมน์ Yes ของอ้างอิงพอดี ควรตรวจว่า `wpg` นี้วาดหรือบังคับให้เซลล์ของแถวนี้ย้ายคอลัมน์หรือไม่ ก่อนแก้ `wrapTopAndBottom`
- แก้แล้วใน `e184943`: สมาชิกกลุ่มรูปร่าง (`wpg`) ที่มี `a:custGeom` ถูกอ่านเป็น freeform (เดิมวาดเป็นกรอบสี่เหลี่ยมเต็ม bounding box สูงเกือบทั้งหน้า พร้อมเส้นแนวตั้งพาดทับตาราง ซึ่งเป็นสาเหตุของการซ้อนทับใน govuk-005 หน้า 12); จำนวนหน้าไม่เปลี่ยน (37, Word 36) เพราะเป็นการวาด ไม่ใช่การจัดหน้า
- ส่วนต่างการจัดหน้าที่เหลือของ govuk-005 (วัดจาก `pdftotext -bbox` หน้า 12): ป้ายซ้ายตรงกับอ้างอิงถึง "Investments?" (y≈208.5) แล้วเลื่อนลง +4.4 pt ที่ตาราง Currency/Value, +1 pt ในตาราง และ +8.4 pt ระหว่างตารางกับป้าย "You must provide" (รวม +13.4 pt) +8.4 pt ใกล้เคียงหนึ่งบรรทัดว่างของฟอนต์ 7 pt (8.05 pt) พอดี: หลังตารางมีย่อหน้าว่างสองอัน (อันที่สองถือ `sectPr` แบบ `continuous` ฟอนต์ 7 pt) แล้วจึงเป็นป้าย ขณะที่ช่องว่างในอ้างอิงมีหนึ่งบรรทัดเท่านั้น; ก่อนตารางมีย่อหน้าว่างสองอัน (10 pt) และย่อหน้าถือ `sectPr` แบบสามคอลัมน์หนึ่งอัน สมมติฐานที่ยังไม่พิสูจน์: Word ไม่ให้ความสูงแก่ย่อหน้าว่างที่ถือ `sectPr` แบบ `continuous` (หรือการ balance คอลัมน์ของ section ต่อเนื่องทำให้ต่าง) ต้องหาตัวอย่างเทียบในคลังก่อนแก้ แล้ววัดว่าไม่มีไฟล์แย่ลง
- ยังไม่มี commit สำหรับการจัดหน้าส่วนนี้ (ไม่ได้เดาแก้โดยไม่มีหลักฐาน); งานต่อไปคือยืนยันว่า wpg anchor ทำให้ย่อหน้าแถวหลังตารางขยับ/ซ้อน แล้วแก้ที่ layout ของ anchor นั้น และวัด govuk-005 ซ้ำ

### เจ้าของเขียนทับ `main` ของ fork ระหว่างรอบ (พบ 13:02 UTC)

- `main` ของ fork ถูก force-update จาก `daa522a` เป็น `4554ea9` หลัง 11:48 UTC: `4554ea9` คือ `3e00170` ที่ `chatraweechai` (committer, 11:29 UTC) นำไปไว้ใน repo ต้นทาง โดยเนื้อไฟล์เหมือน `3e00170` ทุกไฟล์ และต้นทางมี tag `v1.1.2` → `4554ea9` (`main` ของต้นทางก็อยู่ที่ `4554ea9`) ส่วน tag `v1.1.2` ของ fork ยังชี้ `3e00170`
- commit ที่หายจาก `main` ของ fork (`473e0da`, `be45dad`, `daa522a`) และงานใหม่ ถูก rebase ต่อบน `4554ea9` โดยไม่มีไฟล์ชน แล้ว push แบบ fast-forward (ไม่ได้ force) จึงได้ hash ใหม่: `473e0da` → `8d21a41`, `be45dad` → `effd83f`, `daa522a` → `6d9bf34`; ถ้าเจ้าของตั้งใจให้ `main` ของ fork ตามต้นทางเท่านั้น ต้องตัดสินว่าจะเอา commit เหล่านี้เข้าต้นทางหรือไม่

### ผลการออกรุ่น (ตรวจด้วย GitHub API และหน้า Releases)

| รุ่น | commit ที่ build (tag) | run | ไฟล์แนบ | ป้าย |
|---|---|---|---|---|
| v1.0.0 | `7946cbb` (annotated tag ของเจ้าของ) | 41430786475 | `doc4me-1.0.0-windows-x64.msi` 30,146,560 B, `-portable.zip` 40,242,969 B | — |
| v1.1.0 | `51a9658` (annotated tag ของเจ้าของ) | 41430819045 | msi 33,005,568 B, zip 43,571,345 B | — |
| v1.1.1 | `be5893e` (annotated tag ของเจ้าของ) | 41430823465 | msi 32,997,376 B, zip 43,584,023 B | — |
| v1.1.2 | `3e00170` (HEAD ของ `main` ตอนสั่ง; job `release` สร้าง tag เอง; โค้ดแอปเท่ากับ `861e994`) | 41433220035 | msi 36.9 MB, zip 50 MB | Latest |

- ทุกไฟล์ยังไม่ลงนาม (ไม่มี secret ลงนาม) บันทึกของ Release มาจากหัวข้อของรุ่นนั้นใน `CHANGELOG.md` ของ `main` (v1.1.2 ย่อเหลือบรรทัดแรกของแต่ละรายการพร้อมลิงก์ไปหัวข้อเต็มใน fork เพราะยาวเกิน 125,000 ตัวอักษร)
- วิธี: รุ่นเก่าใช้ทางสำรองของ workflow (`run_workflow` บน `main` inputs `release: true`, `commit: <sha>`) สามรุ่นพร้อมกัน job `build` ราว 16 นาทีต่อรุ่น (fork ยังไม่มี cache) job `release` เห็น tag ที่ชี้ commit เดียวกันจึงข้ามการสร้าง tag แล้ว `gh release create --verify-tag --latest=false` สำเร็จทั้งสาม (ข้อจำกัด workflow scope ไม่ติดเพราะ commit มี tag อยู่แล้ว); จากนั้นจึงสั่ง v1.1.2 (`release: true`) ซึ่ง build แบบปกติ (`-BuildOnly` แล้ว `-SkipBuild`) และสร้าง Release พร้อม tag ที่ HEAD
- ก่อนมี v1.1.2 `get_latest_release` ตอบ v1.1.1 แม้สร้างด้วย `--latest=false` (API เลือกตาม `created_at` เมื่อไม่มีรุ่นใดถูกตั้ง Latest); หลังออก v1.1.2 หน้า Releases แสดงป้าย Latest ที่ v1.1.2 รุ่นเดียว
- `chatraweechai/doc4me` ยังไม่มี Release (มี tag `v1.1.2` → `4554ea9` แล้ว ดูหัวข้อด้านบน) ส่วน `main` ของ fork นำหน้าต้นทางด้วยงานรอบนี้ (ต้นทางอ่านได้แต่ push ไม่ได้จาก session นี้)

### สิ่งที่พบตอนเริ่มรอบ (ตรวจ 2026-11-02 ราว 10:00–10:30 UTC)

- session นี้เชื่อมกับ `veevie55/doc4me` ซึ่งเป็น fork (public) ของ `chatraweechai/doc4me`; บัญชี GitHub ของ session คือ `veevie55` (สร้าง 2026-11-02 09:44 UTC) จึงไม่มีสิทธิ์ API กับ repo ต้นทาง อ่านได้เฉพาะ git (`git ls-remote`) และหน้าเว็บสาธารณะ
- เจ้าของ repo push tag ของรุ่นเก่าแล้ว (annotated, tagger `envburiram`, 09:27–09:28 UTC) ชี้ commit ถูกต้องทั้งสาม: `v1.0.0` (`f6055dd`) → `7946cbb57a0f4618711e765a844f46194150ec1f`, `v1.1.0` (`4cfa336`) → `51a9658096d7f5f8b2ea6ed00acd49b5db463a47`, `v1.1.1` (`091a436`) → `be5893e7ab2fe40285205e976ad366bd1d27d50d` (ตรวจด้วย `git ls-remote` ทั้งต้นทางและ fork และ `get_tag`; fork ได้ tag ชุดเดียวกันมาตอน fork)
- **สาเหตุที่ Actions ไม่จ่าย runner (ปัญหาของรอบที่ 19)**: run ของ tag ใน repo ต้นทาง #154 (41420985373, `v1.0.0`), #155 (41421019066, `v1.1.0`), #156 (41421048403, `v1.1.1`) ล้มภายใน 4–11 วินาที และ annotation ของ job `build` ทั้งสาม run (อ่านจากหน้า run สาธารณะด้วย WebFetch) คือ "The job was not started because your account is locked due to a billing issue." คือบัญชี `chatraweechai` ถูกล็อกเพราะปัญหาการชำระเงิน ไม่ใช่ไฟล์ workflow หรือสิทธิ์ของ action; push build #153 (41413504791) ของ `main` มีข้อความเดียวกัน (#151–#152 ล้มภายในไม่กี่วินาทีเหมือนกัน แต่ไม่ได้เปิดอ่าน)
- fork ตอนเริ่ม: ไม่มี Release (Release ไม่ติดมากับการ fork), `list_workflows` ได้ 0 และ `run_workflow` ได้ HTTP 404 เพราะ GitHub ปิด workflow ของ fork ไว้จนกว่าเจ้าของจะกดเปิดในแท็บ Actions (เจ้าของเปิดแล้วราว 10:40 UTC)

### งานรอบนี้ (push ไป `main` แล้ว รายละเอียดและผลตรวจอยู่ใน `CHANGELOG.md` `[Unreleased]`)

- `3e00170` (บน `main` ตอนนี้คือ `4554ea9`) handoff: สิ่งที่พบตอนเริ่มรอบ (ก่อนออกรุ่น)
- `8d21a41` (เดิม `473e0da`) แก้หน้าเปล่าหลังตัวแบ่งหน้า (`crates/layout/src/lib.rs`, เทสต์ `a_page_break_before_an_empty_section_break_paragraph_leaves_no_blank_page`): ย่อหน้าที่จบด้วยตัวแบ่งหน้าแล้วตามด้วยย่อหน้าว่างที่ปิด section ซึ่ง section ถัดไปขึ้นหน้าใหม่ ไม่สร้างหน้าเปล่าแล้ว (ข้อ "zenodo-160" ในงานต่อไปของรอบที่ 18) ทำและ push หลัง v1.1.2 ออกแล้ว Release v1.1.2 จึงไม่มีโค้ดนี้
- วัดบน Linux ด้วย `cargo xtask readiness --corpus --jobs 4` เทียบ CLI ก่อน/หลังแก้ (ให้รายงานแสดงไฟล์ไม่ผ่านครบชั่วคราว): จำนวนหน้าตรง 166 → 168, ต้น/ท้ายหน้าตรง 89 → 90, ไม่มีไฟล์ที่ผลแย่ลง (zenodo-169 ตรงทั้งสองแบบ, zenodo-160 จำนวนหน้าตรง, zenodo-053 64 → 61 จาก Word 58, zenodo-136 27 → 26 จาก Word 25); govuk-037 มีย่อหน้าแบบเดียวกันแต่ section ถัดไปเป็นแบบต่อเนื่องจึงไม่เปลี่ยน; คลังทั้ง 222 ไฟล์ดาวน์โหลดได้ครบ (zenodo-078.pdf และ zenodo-161.docx ล้มครั้งแรก ดาวน์โหลดซ้ำแล้ว SHA-256 ตรง); ตรวจก่อน push: `cargo test -p doc4me-layout` 240, `-p doc4me-engine` 451 + 13 + 8 ผ่าน, `cargo fmt --all --check` และ `cargo clippy --workspace --all-targets -- -D warnings` ผ่าน (บน Linux ต้อง `apt-get install libasound2-dev` ก่อน ไม่งั้น build script ของ `alsa-sys` ล้ม)
- push ของ `473e0da` เป็น push build แรกของ `main` ใน fork (ไม่ออก Release): run 41437764535 ผ่าน (build และทำ MSI/zip บน Windows ราว 12 นาที ใช้ cache ที่ run ของ v1.1.2 บันทึกไว้)
- `effd83f` (เดิม `be45dad`) และ `6d9bf34` (เดิม `daa522a`): handoff, `ROADMAP.md`, `docs/roadmap.md` หลังออกรุ่น และการตัดสินใจเรื่องลิงก์ README

### งานต่อด้าน pagination (สั่ง "ทำงานต่อจากงานต่อไปใน handoff"; วัดบน Linux เครื่องเดียวกัน แสดงไฟล์ไม่ผ่านครบชั่วคราว)

- `zenodo-023` (งานต่อไปของรอบที่ 18): ไม่ใช่บั๊ก — DOCX สร้างเมื่อ 2026-07-22 ด้วยตัวแปลงไฟล์ (core.xml วันสร้างเท่าวันแก้ไข, AppVersion 12.0000, ไม่มีจำนวนหน้า) ส่วน PDF ที่ Word สร้างลงวันที่ 2022-06-14 จึงเป็นคนละรุ่น; บนหน้า 52 ของ PDF ข้อความ header1 และเลข "1" ถูกรูปสีเขียวทั้งหน้าทับไว้ (pdftotext ยังอ่านได้) แต่ section สุดท้ายของ DOCX อ้าง header2/footer2 ที่ว่าง; ไม่ได้แก้ manifest หรือ comparator
- `bddb167` ตาราง: `w:trPr/w:hidden` และ `w:hideMark` ไม่หายตอนบันทึกแล้ว (เดิมทิ้งไป ไฟล์ที่บันทึกด้วย Doc4Me จึงแสดงแถวที่ซ่อนใน Word) แถวที่เครื่องหมายท้ายแถวและเนื้อหาทั้งหมดซ่อนไม่กินที่ และย่อหน้าว่างท้ายเซลล์ `hideMark` ไม่นับในความสูงแถว: govuk-066 9 → 6 หน้าตรงทั้งสองแบบ, govuk-068 17 → 16 (Word 15); จำนวนหน้าตรง 168 → 169, ต้น/ท้ายหน้าตรง 90 → 91 (`hideMark` ใช้ใน 36 ไฟล์ แถวซ่อนมีแค่ govuk-066)
- `3593d05` ตัวแบ่งหน้าท้ายย่อหน้าที่ปิด section เองไม่สร้างหน้าเปล่า และบรรทัดที่มีแค่ตัวแบ่งหน้ากับรูปลอยหรือ marker คงอยู่ท้ายหน้า: zenodo-029 18 → 17 และ zenodo-140 16 → 15 (เท่า Word แต่ยังต่างที่หน้า 2), zenodo-149 42 → 41, zenodo-053 61 → 60; จำนวนหน้าตรง 169 → 171, ต้น/ท้ายหน้าคงที่ 91
- รวมรอบนี้บน Linux: จำนวนหน้าตรง 166 → 171, ต้น/ท้ายหน้าตรง 89 → 91, ไม่มีไฟล์ที่ผลแย่ลง; ตรวจก่อน push ทุกครั้ง: เทสต์ layout (ล่าสุด 245), docx 75 + 49, docbin 65, doc 118, engine 451 + 13 + 8, ui-egui `--lib` 396, `cargo fmt --all --check`, `cargo clippy` (workspace สำหรับ `bddb167`, crate layout สำหรับ `3593d05`)
- `a351c88` ผลลัพธ์ของฟิลด์ (FORMTEXT, FILLIN) ตัดบรรทัดตามช่องว่าง: govuk-024 3 → 2 หน้า (Word 2) แก้แล้ว (ต้น/ท้ายหน้า 2 ยังต่าง), จำนวนหน้าตรง 171 → 172 บน Linux; zenodo-115 ได้ 10 หน้า (Word 9) ทั้งก่อนและหลังแก้ (ไม่เกี่ยวกับงานนี้ ยังไม่ได้สอบสวน); `cargo xtask readiness --corpus` เขียนทับ `docs/readiness.md` ต้อง `git checkout docs/readiness.md` ก่อน commit (ไฟล์ที่ commit คือผลวัดบน Windows)
- (แก้แล้วใน `a351c88`; ข้อความต่อไปนี้เป็นบันทึกก่อนแก้) พบแต่ยังไม่แก้: **govuk-024** (3 หน้า Word 2 ทั้งบน Windows และ Linux) ฟิลด์ฟอร์ม FORMTEXT ที่ผลลัพธ์ยาวถูกจัดเป็น cluster เดียว (`shape_atomic` ใน `crates/layout/src/para.rs`) ตัดบรรทัดไม่ได้ ความกว้างขั้นต่ำของเซลล์จึงเป็นทั้งประโยค (499 และ 1,306 pt) autofit ขยายคอลัมน์ทั้งหมดจนรวม 1,385 pt แล้วย่อทั้งตารางเหลือ 538 pt คอลัมน์หมายเหตุซ้ายจึงแคบจนตัดทีละพยางค์ ตารางสูงจนล้นหน้า; ใน Word ผลลัพธ์ของ FORMTEXT ตัดบรรทัดได้แบบข้อความปกติ ทางแก้คือแตก cluster ของฟิลด์ฟอร์มเป็นคำที่ตัดบรรทัดได้ (ต้องดูการวาง caret การเลือก และข้อความที่ส่งออก PDF) ในคลังมีไฟล์นี้ไฟล์เดียว; **zenodo-149** ยังเกิน 13 หน้า หน้าปกล้นเพราะข้อความในกล่องสัญญาอนุญาตตัดบรรทัดมากกว่า Word (ความกว้างของฟอนต์บน Linux ต้องดูบน Windows); **zenodo-156** หน้า 1 ของ Word จุคำมากกว่าเกือบสองเท่า (น่าจะเป็นฟอนต์)

### ข้อค้นพบที่ต้องให้เจ้าของ repo ตัดสิน

- **บ้านของโปรเจกต์ (ตัดสินแล้วสำหรับ README)**: ผู้ใช้สั่ง (2026-11-02) ไม่เปลี่ยนลิงก์ Releases ใน README จึงคงชี้ `chatraweechai/doc4me` แม้ Release อยู่ที่ https://github.com/veevie55/doc4me/releases (บันทึกไว้ในการตัดสินใจถาวรด้านล่าง); ลิงก์อื่นที่ชี้ `chatraweechai/doc4me` (หน้า About, `repository` ใน `Cargo.toml`, URL แค็ตตาล็อก Add-in ใน `crates/engine/src/cmd/addins.rs`, `websiteUrl` ของ MCP และลิงก์ท้าย `CHANGELOG.md`) ก็ยังไม่ได้เปลี่ยน (app id `io.github.chatraweechai.doc4me` ไม่ควรเปลี่ยน เพราะเป็นชื่อไฟล์ .desktop ไอคอน และ Flatpak ที่ติดตั้งไว้แล้ว; แค็ตตาล็อก Add-in ใน `raw.githubusercontent.com/chatraweechai/...` ยังเปิดได้เพราะ repo ต้นทางยัง public)
- ถ้าจะให้ต้นทางมี Release ด้วย: แก้การชำระเงินของ `chatraweechai` แล้ว re-run #154–#156 (ใช้ commit และ workflow เดิมของ tag) แล้วออก v1.1.2 ที่นั่น (ต้นทางมี tag `v1.1.2` → `4554ea9` แล้ว ซึ่งเนื้อไฟล์เหมือน `3e00170` ที่ Release v1.1.2 ของ fork build; การ push tag นั้นสร้าง run #158 และ push `main` สร้าง run #157 ซึ่งล้มแบบ startup failure ทั้งคู่เพราะบัญชียังถูกล็อก เมื่อแก้การชำระเงินแล้ว re-run #158 ได้เลย)
- `zenodo-023`: PDF อ้างอิง (Word, 2022-06-14) กับ DOCX (ตัวแปลงไฟล์, 2026-07-22) เป็นคนละรุ่น หน้าสุดท้ายจึงต่างเฉพาะข้อความหัว/ท้ายกระดาษที่ถูกรูปทับ (ดูงานต่อด้าน pagination)
- `zenodo-160` หน้า 8: หน้า PDF อ้างอิงมีข้อความ comment "Commented [SP1]: I updated this table …" ในแถบ markup แต่ไฟล์ .docx ไม่มี `word/comments.xml` แล้ว (คนละรุ่นในส่วนของ comment) ตัวตรวจต้น/ท้ายหน้าจึงไม่ผ่านทั้งที่เนื้อหาในหน้าตรงกัน (ตารางจบที่ "66 60 60 66 62 62" ทั้งสองฝั่ง); ไม่ได้แก้ manifest หรือ comparator ตามกติกา
- ข้อค้นพบของรอบที่ 18 (`govuk-026`, `govuk-018`, `zenodo-104`) ยังรอการตัดสินใจ

- **Dialog ที่ยังไม่ผ่าน 12 แถว (ไม่นับ 12 deferred) — ขอให้เจ้าของตัดสินว่าจะ defer หรือไม่** (2026-11-03): เป็นฟีเจอร์ที่ Word รุ่นปัจจุบัน (Microsoft 365) ตัดออกไปแล้ว — `dlg-081`/`dlg-082` WordPerfect Help, `dlg-151` Schema Library / `dlg-234` XML Element Attributes / `dlg-235` XML Options (custom XML markup ถูกตัดตั้งแต่ Word 2010; Doc4Me อ่านเนื้อหาแต่ทิ้ง markup แบบเดียวกัน), `dlg-215` Smart Tag Options (smart tags ถูกตัด; Doc4Me ทิ้ง markup เช่นกัน), `dlg-079` Frames Page properties และ `dlg-010` CSS Links (หน้าเว็บแบบ frames/HTML รุ่นเก่า), `dlg-052` Format Callout (callout ของ Word 97); ที่ยังมีใน Word แต่ต้องงานใหญ่หรือเครื่องของเจ้าของ — `dlg-083` Horizontal in Vertical (ต้องจัดวางที่รู้ทิศทางข้อความ), `dlg-108` Insert Subdocument (master document), `dlg-041` Print Setup (เลือกเครื่องพิมพ์ของระบบ; Doc4Me ส่ง PDF ให้โปรแกรมดู PDF พิมพ์ จึงต้องรายการเครื่องพิมพ์ของแต่ละระบบ)
- **Shortcut ที่ยังไม่ผ่าน**: `ks-203` (Alt+Shift+F12, Space: ตัวเลือก citation) และ `ks-329` (Alt+Shift+F12: ปุ่ม Table of Contents) ต้องมีปุ่มแท็บของ content control ของ TOC/citation ซึ่ง Doc4Me ยังไม่มี — ไม่ผูกปุ่มกับคำสั่งอื่นแทนเพื่อไม่ให้ผ่านแบบไม่จริง; `ks-219` (Ctrl+Shift+F7 อัปเดตไฟล์ต้นทางของลิงก์) ต้องประเมิน INCLUDETEXT ก่อน; `ks-177` (F9 รีเฟรชหน้าเว็บ) ไม่มีมุมมองเว็บสด; `ks-165` รอเจ้าของตรวจบน Word for Windows

### งานต่อไป (ยังไม่ถึงเป้าหมาย 100%)

- ทำต่อจาก "งานต่อไป" ของรอบที่ 18 ด้านล่าง (ข้อ zenodo-160 เสร็จแล้ว, zenodo-023 เป็นคนละรุ่น): วัดซ้ำบน Windows แล้วสร้าง `docs/readiness.md` ใหม่, pagination (`govuk-005`, govuk-024 ฟิลด์ฟอร์มที่ตัดบรรทัดได้, zenodo-029/zenodo-140 หน้า 2), dialogs, localization
- ห้ามออกเวอร์ชันถัดจาก 1.1.2 จนกว่าผู้ใช้สั่ง; release workflow ใน fork: push ไป `main` ไม่ build installer แล้ว (ผู้ใช้สั่ง 2026-11-03) สั่งจากแท็บ Actions หรือ push tag `v*` เท่านั้น

## สถานะรอบที่ 19 (2026-11-02: ออก v1.0.0, v1.1.0, v1.1.1 และ v1.1.2; ประวัติ เจ้าของ push tag แล้วและพบสาเหตุในรอบที่ 20)

ผู้ใช้สั่ง (2026-11-02): "ออกเวอร์ชันเก่าทั้ง v1.0.0 v1.1.0 v1.1.1 และเวอร์ชันใหม่จากโค้ดล่าสุด v1.1.2" ระหว่างทำสั่งเพิ่มให้ตรวจ tag/draft release อีกครั้ง แล้วให้ตรวจสคริปต์ build ของแต่ละรุ่นเพื่อแก้ workflow ให้ออกรุ่นเก่าจาก commit เดิมได้

### สถานะ: รอเจ้าของ repo แก้ปัญหา Actions ไม่จ่าย runner และ push tag รุ่นเก่า แล้วจึงออก v1.1.2 (ผู้ใช้เลือก 2026-11-02 ให้ออก v1.1.2 หลังรุ่นเก่า เพื่อให้ v1.1.2 เป็น Latest)

- **ปัญหาใหม่ (พบ 2026-11-02 08:25 UTC): GitHub Actions ไม่จ่าย runner ให้ job ใดเลย** เริ่มระหว่าง 03:47 ถึง 07:51 UTC (job ล่าสุดที่ได้ runner คือ attempt 9 ของ run 41379963417 เวลา 03:37–03:47 UTC ซึ่งถูกยกเลิก)
  - job `build` (windows) ของ push build `8f7d2e2` (run 41409013327, 07:51 UTC) และ `ee4e1ee` (run 41409192856, 07:53 UTC และ attempt 2 ที่ session re-run 08:25 UTC) ล้มภายใน 2 วินาที: `runner_id` 0, ไม่มี step, log ได้ HTTP 404
  - job `release` (ubuntu) ของ run 41379967784 ที่ session re-run (attempt 3, 08:28 UTC) ล้มแบบเดียวกัน จึงไม่ได้เป็นเฉพาะ Windows
  - ไม่ได้เกิดจากไฟล์ workflow: ส่วนระดับ job (`runs-on`, `timeout-minutes`, `outputs`) ไม่เปลี่ยนตั้งแต่ run 41379931795 (`861e994`) ที่ผ่าน และ job ubuntu ข้างบนใช้ workflow ของ `861e994` นั้นเอง; repo เป็น public (นาทีของ runner มาตรฐานฟรี); githubstatus.com ทุกระบบ Operational ไม่มี incident
  - ข้อความสาเหตุอยู่ใน annotation ของ run ซึ่งเครื่องมือของ session อ่านไม่ได้ (`get_check_run` ได้ output ว่าง)
  - **[เจ้าของ repo]** เปิด https://github.com/chatraweechai/doc4me/actions/runs/41409192856 อ่านข้อความใต้ "Annotations" แล้วแก้ตามนั้น สาเหตุที่ให้อาการนี้: Settings › Actions › General › Actions permissions ไม่อนุญาต action ที่ใช้ (`actions/checkout`, `actions/upload-artifact`, `actions/download-artifact`, `dtolnay/rust-toolchain`, `Swatinem/rust-cache`) หรือบัญชีถูกล็อกเพราะการชำระเงิน (Settings › Billing and licensing)
  - push tag ได้ก่อนแก้ แต่ run ของ tag จะล้มแบบเดียวกัน หลังแก้แล้ว re-run run นั้นได้เลย ไม่ต้อง push tag ซ้ำ (re-run ใช้ commit และ workflow เดิมของ tag)

- push ไป `main` แล้ว:
  - `861e994` ออกเวอร์ชัน 1.1.2 (`cargo xtask version set 1.1.2`, ย้าย `[Unreleased]` เป็น `## [1.1.2] - 2026-11-02`) และ workflow รับ input `commit` เพื่อ build/ออก Release ของ commit เก่าบน `main` (ทางสำรอง)
  - `8f7d2e2` แก้ workflow หลังรันจริง: tag ของ commit ที่ไฟล์ workflow ต่างจาก `main` ต้อง push จาก clone ก่อน job `build` จึงหยุดตั้งแต่ก่อน build พร้อมบอกคำสั่ง
- รันจริงบน `861e994`: run 41379963417 (1.0.0), 41379965614 (1.1.0), 41379967784 (1.1.1) ตรวจ input/ancestry และ build MSI/zip ผ่านทั้งหมด แต่ "Tag the earlier commit" ได้ `Resource not accessible by integration (HTTP 403)` ทั้งสามรุ่น จึงยังไม่มี Release; push build ของ `main` (run 41379931795) ผ่านด้วยทางปกติ
  - เจ้าของ repo re-run เองหลายครั้ง (attempt 2 ของ 41379965614 ล้มที่ "Choose the tag" เพราะตอนนั้นมี tag `v1.1.0` ที่ `861e994` ซึ่งเกิดจากการ Publish draft แล้วลบภายหลัง; 41379963417 ถึง attempt 9 แล้วถูกยกเลิก)
  - session นี้ลอง push tag `v1.1.1` เองอีกครั้ง ได้ HTTP 403 จาก git proxy เหมือนเดิม (ลบ tag ในเครื่องแล้ว ไม่ลองซ้ำ)
- **[เจ้าของ repo]** รันใน clone ของตัวเอง (ห้าม Publish draft 4 รายการ เพราะ target เป็น `main` จะได้ tag ผิด commit; ลบทิ้งได้):
  - `git fetch origin`
  - `git tag -a v1.0.0 7946cbb57a0f4618711e765a844f46194150ec1f -m "Doc4Me 1.0.0"`
  - `git tag -a v1.1.0 51a9658096d7f5f8b2ea6ed00acd49b5db463a47 -m "Doc4Me 1.1.0"`
  - `git tag -a v1.1.1 be5893e7ab2fe40285205e976ad366bd1d27d50d -m "Doc4Me 1.1.1"`
  - `git push origin v1.0.0`, `git push origin v1.1.0`, `git push origin v1.1.1` (แยกทีละตัว) แต่ละ tag รัน workflow ของ commit นั้นเองซึ่ง build และออก Release พร้อมหัวข้อ CHANGELOG ของรุ่นนั้น
- งานของ session หลังเจ้าของ repo แก้ปัญหา Actions และ push tag:
  0. re-run push build ล่าสุดของ `main` ให้ผ่านก่อน เพื่อยืนยันว่า Actions จ่าย runner แล้ว และ re-run run ของ tag ที่ล้มเพราะปัญหานั้น
  1. รอ run ของ tag ทั้งสามจบ ตรวจว่า Release มี `doc4me-<เวอร์ชัน>-windows-x64.msi` และ `-portable.zip` และ tag ชี้ commit ข้างบน
  2. ถ้า run ของ tag ใดออก Release ไม่ได้ (เช่น workflow ของ `be5893e` ส่ง `--target` ทั้งที่มี tag แล้ว) ให้ใช้ทางสำรอง: re-run failed jobs ของ run 41379967784 (หรือ run ใหม่ `release: true`, `commit: <sha>`) ซึ่งจะเจอ tag ที่ commit เดียวกันแล้วออก Release แบบไม่ย้ายป้าย Latest
  3. แล้วจึงออก v1.1.2: `actions_run_trigger` workflow `windows-x64-installer.yml` ref `main` input `release: true` (ไม่ใส่ `commit`) บันทึกของ Release จะย่อเพราะหัวข้อ `[1.1.2]` ยาวเกิน 125,000 ตัวอักษร; ห้าม push ไป `main` ระหว่างที่ job `release` รัน
  4. ตรวจว่า v1.1.2 เป็น Latest แล้วอัปเดต handoff, `ROADMAP.md` แถว Releases และ `docs/roadmap.md` M14

### สิ่งที่พบก่อนออก (ตรวจ 2026-11-02 ด้วย GitHub API)

- ไม่มี tag เลย (`list_tags` ว่าง, `git ls-remote --tags` ว่าง) และไม่มี Release ที่เผยแพร่: Release v1.1.0 (tag ที่ `51a9658`, ออก 2026-10-21 run #31) และ v1.1.1 (`be5893e`, ออก 2026-10-22 run 39788628618 แล้วออกซ้ำที่ `4f1d407` โดย run 40634619053 เมื่อ 2026-10-28) ไม่มีแล้วทั้ง Release และ tag เมื่อตรวจ (ไม่ทราบว่าใครลบเมื่อไร); v1.0.0 (`7946cbb`) ยังไม่เคยมี tag
- draft ที่เจ้าของ repo สร้างไว้ 4 รายการ (ยังไม่มี tag, body ว่าง): `v1.0.0` "Doc4Me v1.0.0", `v1.1.0` "Doc4Me v1.1.0", `v.1.1.1` "Doc4Me v1.1.1" (ชื่อ tag มีจุดเกินหลัง v) และ `v1.1.2` "Doc4Me v1.1.2"; draft ไม่ขวาง `gh release create` (gh ตรวจเฉพาะ Release ที่เผยแพร่แล้วด้วย HEAD `releases/tags/<tag>`) session จึงไม่ได้แตะ draft เหล่านี้
- commit ของรุ่นเก่า (ยืนยันด้วย `git merge-base --is-ancestor` ว่าอยู่บน `main` ทั้งหมด และ `Cargo.toml` ตรงเวอร์ชัน): 1.0.0 = `7946cbb57a0f4618711e765a844f46194150ec1f` (handoff ข้อ 5.1 เดิม), 1.1.0 = `51a9658096d7f5f8b2ea6ed00acd49b5db463a47`, 1.1.1 = `be5893e7ab2fe40285205e976ad366bd1d27d50d`
- สคริปต์ build ของทั้งสามรุ่น: workflow รัน `package.ps1 -Arch x64` ขั้นเดียวไม่มี secret (ยังไม่มี `-BuildOnly`), `DOC4ME_BUILD_SHA` มาจาก `git rev-parse HEAD` จึงตรงกับ commit ที่ checkout, `cargo build --release --locked`, ไม่มี `rust-toolchain.toml` (ใช้ Rust stable ปัจจุบัน) และไม่มี workflow รุ่นใดตั้ง `CRAFT_FONTS_DIR`
- ข้อจำกัดของ GitHub: เอกสาร REST "Create a release" ระบุว่า `GITHUB_TOKEN` ให้ Release สร้าง tag ที่ commit ซึ่งไฟล์ workflow ต่างจาก default branch ไม่ได้ (commit ของรุ่นเก่าต่างทั้งหมด) `861e994` จึงให้ workflow สร้าง tag ด้วย git refs API ก่อนแล้วค่อยออก Release ของ tag นั้น แต่ API นั้นก็ตอบ 403 (ข้างบน) tag ของรุ่นเก่าจึงต้อง push จาก clone

## สถานะรอบที่ 18 (2026-11-02)

ผู้ใช้สั่ง (2026-11-02): อ่าน handoff.md และ ROADMAP.md แล้วทำส่วนที่ค้างต่อให้เสร็จ commit/push `main` ก่อนแล้วค่อยตรวจทีหลัง ตรวจเสร็จ commit/push ที่แก้ แล้วบันทึก handoff.md และ ROADMAP.md ตามสถานะปัจจุบัน

### งานรอบนี้ (push ไป `main` แล้ว รายละเอียดและผลตรวจอยู่ใน `CHANGELOG.md` `[Unreleased]`)

- `be82b92` เก็บ `w:wBefore`/`w:wAfter` ของแถวที่เว้นคอลัมน์ต้น/ท้าย (ต่อจาก `ee9c49f` ของรอบที่ 17 ซึ่งรองรับ `w:gridBefore`/`w:gridAfter` แล้ว; session นี้เคยทำซ้ำซ้อนไว้ในเครื่องจึงทิ้งของตัวเองและเติมเฉพาะส่วนที่ขาด)
- `7c5899a` ระยะขอบภายในกล่องข้อความตามไฟล์ (`wps:bodyPr` `lIns`/`tIns`/`rIns`/`bIns`) และจุดยึดของวัตถุลอยไม่ทำให้บรรทัดสูงขึ้น (govuk-005: ท้ายกระดาษ "Page 5 of 37 ASF1_ V4.0" ครบแล้ว และไม่มีหน้า 6 เปล่า)
- `b7d4321` ย่อหน้าว่างที่มีแต่ตัวแบ่ง section ท้ายหน้า (section ถัดไปขึ้นหน้าใหม่) ไม่สร้างหน้าเปล่า (zenodo-135 เคยเกิน 4 หน้า)
- `9cd6bff` Review › Chinese Conversion: จีนตัวเต็ม ↔ ตัวย่อด้วยตาราง OpenCC (Apache-2.0, `assets/opencc`, มีแถวใน `ATTRIBUTION.md`) ผูก `dlg-174`; Convert to Thai/Arabic Digits ใช้ `map_chars` ตัวเดียวกัน
- ตรวจหลัง push (กฎผู้ใช้: push ก่อนแล้วตรวจ): `cargo fmt --all --check`, `cargo test -p` doc 118, docbin 65, docx 224, layout 239, engine 451+13+8, ui-egui `--lib` 396, i18n `--include-ignored` 62 ผ่าน, `cargo clippy --workspace --all-targets -- -D warnings` ผ่าน, `cargo xtask assets` ผ่าน (79 ไฟล์); อ่าน diff ซ้ำแล้วไม่พบบั๊กที่ต้องแก้ จึงไม่มีคอมมิตแก้หลังตรวจ

### ผลวัดรอบนี้ (ยังไม่ได้สร้าง `docs/readiness.md` ใหม่)

- Dialogs: 188/228 (เพิ่ม `dlg-174`); ชุดนี้ตรวจจาก registry จึงไม่ขึ้นกับแพลตฟอร์ม
- Pagination วัดบน Linux ด้วยสคริปต์ที่ใช้เกณฑ์เดียวกับ `cargo xtask readiness --corpus` (scratchpad ของ session) เทียบก่อน/หลังงานรอบนี้บนเครื่องเดียวกัน: จำนวนหน้าตรง 160 → 166, คำต้น/ท้ายทุกหน้าตรง 82 → 89, ไม่มีไฟล์ที่ผลแย่ลง (ดีขึ้น: govuk-057, zenodo-054, zenodo-127, zenodo-165 จากระยะขอบกล่องข้อความ/จุดยึด; zenodo-072, zenodo-102, zenodo-135, zenodo-151, zenodo-154 จากกฎย่อหน้า section); ตัวเลขสัมบูรณ์บน Linux ต่างจาก Windows (รอบ 17: 162/87) เพราะ Linux ไม่มีฟอนต์ Windows หลายตัว (Tahoma, Segoe UI, MS Gothic ฯลฯ) ต้องวัดซ้ำบน Windows: `cargo xtask readiness --corpus --jobs 4 --cli target\\release\\doc4me-cli.exe`
- ภาษา 30/95 ไม่เปลี่ยน

### ข้อค้นพบที่ต้องให้เจ้าของ repo ตัดสิน (ไม่ได้แก้ manifest หรือ comparator ตามกติกา)

- `govuk-026`: DOCX บันทึกเมื่อ 2025-01-30 และ `docProps/app.xml` นับ 4 หน้า (เท่ากับ Doc4Me) แต่ PDF อ้างอิงสร้างปี 2012 ด้วย Acrobat PDFMaker 9.1 มี 7 หน้า (แต่ละตารางขึ้นหน้าใหม่) จึงน่าจะเป็นคนละรุ่น; เกณฑ์ `excluded` ใน `docs/readiness/README.md` เขียนไว้ตามชื่อไฟล์ จึงไม่ได้ตัดออกเอง
- แถวที่หน้าของ Doc4Me เท่ากับ `app.xml` แต่ไม่เท่า PDF (อาจคนละรุ่นเหมือนกัน แต่ `app.xml` อาจค้างจึงต้องตรวจก่อน): `govuk-018` (7/7 vs PDF 6), `zenodo-104` (25/25 vs PDF 29)

### งานต่อไป (ยังไม่ถึงเป้าหมาย 100%)

1. **วัดซ้ำบน Windows** แล้วสร้าง `docs/readiness.md` และตัวเลขใน `ROADMAP.md` ใหม่จากผลนั้น
2. **Pagination**: `govuk-005` บน Linux เหลือ 37 หน้า (Word 36) ส่วนที่ยังคลาด: ความสูงบรรทัดที่มี ☐ (U+2610, Word ใช้ MS Gothic/Segoe UI Symbol ซึ่ง Linux ไม่มี) ใน section แบบหลายคอลัมน์ต่อเนื่อง และการจัดคอลัมน์ก่อนตัวแบ่งต่อเนื่อง (ตารางถัดไปเลื่อนลง ~13pt) ให้ดูบน Windows ก่อน; รูปแบบที่พบแต่ยังไม่แก้: `zenodo-160` (ตัวแบ่งหน้าตามด้วยย่อหน้าว่างที่มีตัวแบ่ง section และ section ถัดไปขึ้นหน้าใหม่: Word ไม่มีหน้าเปล่า แต่ Doc4Me มี; ต้องทิ้งหน้านั้นพร้อมจัดการเลขหน้าและขนาดหน้า), `zenodo-023` (หน้าสุดท้ายของ Word แสดงหัวกระดาษของ header1 และเลข "1" ทั้งที่ section สุดท้ายอ้าง header2/footer2 ที่ว่าง)
3. **Dialogs**: เหลือ 28 แถวที่ไม่ deferred; ที่ทำได้จริงต่อไป: `dlg-047` Fit Text (`w:fitText` ต้องจัดวาง), `dlg-021` Edit Links, `dlg-218` East Asian typography (kinsoku), `dlg-229` Two Lines in One, `dlg-083` Horizontal in Vertical; `dlg-209` File Locations มีผลจริงได้เฉพาะโฟลเดอร์ Documents เพราะ AutoSave เขียนไฟล์ตรงและไม่มีโฟลเดอร์ template ของผู้ใช้
4. **Localization** 30/95: ต้องแปลจริงราว 2,900 ข้อความต่อภาษา ห้ามนับ fallback ภาษาอังกฤษ
5. คง 26 แถว deferred ไว้จนกว่าจะมีหลักฐานหรือคำสั่งจากเจ้าของ

## ผลวัดรอบที่ 17 (2026-11-01, Windows)

### Readiness ล่าสุด: 86.0% (วัดครบ 222 เอกสาร, 2026-11-01)

| มิติ | น้ำหนัก | คะแนน | ชุดตรวจ |
|---|---:|---:|---|
| ฟีเจอร์ | 0.30 | 91.0% | ribbon 499/499; dialogs 187/228 (12 deferred; dlg-040 excluded: Macintosh-only) |
| รูปแบบไฟล์ | 0.20 | 100.0% | เปิด 222/222; round-trip 222/222 |
| การจัดหน้า | 0.15 | 56.1% | จำนวนหน้าตรง 162/222; คำต้น/ท้ายตรง 87/222 |
| UI/UX | 0.10 | 95.7% | shortcuts 268/280 (7 deferred) |
| ความเสถียร | 0.10 | 100.0% | corpus 222/222; hand checks 5/5 |
| ประสิทธิภาพ | 0.05 | 100.0% | 6/6 |
| ภาษา | 0.04 | 31.6% | 30/95 |
| แพลตฟอร์ม | 0.03 | 80.0% | 4/5 (Windows arm64 deferred) |
| ระบบเสริม | 0.02 | 83.3% | 5/6 (1 deferred; Syncthing eco-003 ผ่าน) |
| ฮาร์ดแวร์ | 0.01 | 37.5% | 3/8 (5 owner-deferred) |

คะแนนเมื่อไม่นับ 26 แถว deferred = 88.5%; ยังไม่ถึง 100% เพราะยังมีความคลาดเคลื่อนด้าน pagination, dialog และภาษา รวมถึงการตรวจที่ต้องใช้ฮาร์ดแวร์/แพลตฟอร์มของเจ้าของ repo; ไม่เปลี่ยนสถานะ deferred โดยไม่มีหลักฐานหรือคำสั่งตรงตามกติกา. `dlg-040` ถูกตัดออกจาก check set เพราะเป็น Macintosh-only ตามเอกสารอ้างอิง ไม่ใช่การทำเครื่องหมายผ่าน. ไฟล์ผล generated อยู่ใน `docs/readiness.md`.

รอบนี้เชื่อม `dlg-144`, `dlg-189` และ `dlg-190` (New Toolbar, Customize Menu Bar และ Customize Menus แบบคลาสสิก) กับ `tools.customizeRibbon`; Word 365 ใช้ ribbon แทน menu bar และ Doc4Me มี custom tabs, groups และ command placement. UI regression `customize_ribbon_opens_and_the_layout_persists` ผ่าน. Dialog checks เพิ่ม 184→187/228; readiness 86.0% (88.5% ไม่มี deferred), corpus ยัง 222/222 เปิดและ round-trip, pagination 162/222 page counts และ 87/222 page boundaries. ตรวจ: `cargo test -p doc4me-ui-egui --lib customize_ribbon_opens_and_the_layout_persists` และ `cargo xtask readiness --corpus --jobs 4 --cli target\\release\\doc4me-cli.exe` ผ่าน.

### งานต่อไป (ยังไม่ถึงเป้าหมาย 100%)

1. **Pagination ก่อน**: 162/222 page counts และ 87/222 page-boundary checks ผ่าน; ใน readiness ปัจจุบันยังตก 60 page counts และ 135 page-boundary checks. เริ่มตรวจ `govuk-005` (Doc4Me 40 หน้า, Word reference 36) และแยกสาเหตุระดับเอกสาร/ตาราง/ฟอนต์ก่อนเปลี่ยนกฎ layout; อย่าปรับ comparator หรือรับผลเมื่อ DOCX กับ reference เป็นคนละรุ่น.
2. **Dialogs**: ผ่าน 187/228; 41 แถวไม่ผ่านรวม 12 deferred (จึงมี 29 แถวที่ยังต้องตรวจ/ทำจริง). เติม mapping เฉพาะเมื่อมี UI/command ที่ทำงานเทียบเท่าจริงและมี regression test.
3. **Localization**: ผ่าน 30/95; อีก 65 ภาษาไม่มี catalog ที่ผ่านเกณฑ์. ต้องเพิ่มคำแปลจริงและตรวจ coverage; ห้ามนับ English fallback หรือ mapping ที่ไม่มี catalog เป็นผ่าน.
4. รัน `cargo xtask readiness --corpus --jobs 4 --cli target\\release\\doc4me-cli.exe` หลังการแก้แต่ละช่วง, ปรับ readiness/gap/parity documents และ `CHANGELOG.md` ตามผลจริง, แล้ว commit/push `main`; คง 26 owner-deferred checks ไว้ตามสถานะจนกว่าจะมีหลักฐานหรือคำสั่งจากเจ้าของ.
5. งานรอบนี้จบตามคำขอให้หยุดหลังบันทึกสถานะ; readiness ยังไม่ใช่ 100% (86.0% รวม, 88.5% เมื่อตัด 26 deferred) และยังไม่มีการอ้างว่าทำเป้าหมายสำเร็จ.

รอบนี้เชื่อม `dlg-177` (`wdDialogToolsAdvancedSettings`) กับ `file.options` โดยมี `pane=advanced`; File › Options มี Advanced pane จริงและ UI regression `options_panes_open_and_change_settings` ตรวจการเปิด pane และเปลี่ยนค่าได้. จำนวน dialog ผ่านเพิ่ม 183→184/228; readiness 85.8% (88.3% ไม่มี deferred), corpus ยัง 222/222 เปิดและ round-trip, pagination 162/222 page counts และ 87/222 page boundaries. ตรวจ: `cargo test -p doc4me-ui-egui --lib options_panes_open_and_change_settings` และ `cargo xtask readiness --corpus --jobs 4 --cli target\\release\\doc4me-cli.exe` ผ่าน.

รอบนี้ยืนยัน `eco-003` ด้วย Syncthing 2.1.7 สอง peer บน loopback-only: Doc4Me สร้าง DOCX ในโฟลเดอร์ A, sync ไป B, เปิดและบันทึกทับไฟล์เดิมจาก B แล้วตรวจว่าไฟล์ที่เปิดซ้ำจากทั้งสอง peer มี edits จากทั้งคู่. ปิด global/local discovery, relay, NAT และ crash reporting ระหว่างทดสอบ; ล้าง whitespace ใน `encryptionPassword` ของ template เพื่อให้ cluster-config มี device info ครบ. บันทึกแถวเป็น `pass`; Nextcloud Desktop ไม่ได้ทดสอบแยก เพราะเกณฑ์ระบุ Syncthing หรือ Nextcloud อย่างใดอย่างหนึ่ง. readiness ใหม่ 85.7% (88.2% ไม่มี deferred), ecosystem 5/6.

รอบนี้แก้ `crates/layout/src/lib.rs`/`table.rs` ให้ข้ามชิ้นแรกที่เกิดจากการแยกแถวตารางเฉพาะเมื่อทุก cluster เป็น page-break marker; เดิมตรวจ `Line::vis` ซึ่งว่างสำหรับบรรทัด LTR ทำให้เสี่ยงทิ้งข้อความจริง. เพิ่ม/คง regression สำหรับ `a_leading_page_break_in_a_table_row_does_not_leave_a_blank_page` และ `table_rows_keep_with_next_and_page_breaks_in_cells`; ทดสอบ layout 235 ผ่านเมื่อข้าม 3 test ที่ขึ้นกับฟอนต์ไทยใน Windows profile, suite เต็ม 235 ผ่านแต่ 3 fail เพราะ TH Sarabun New/PSK ติดตั้งอยู่และทำให้ test fallback ล้ม. `cargo clippy` ของ 6 crates, `cargo fmt --all -- --check`, release build และ `cargo xtask readiness --corpus --jobs 4 --cli target\\release\\doc4me-cli.exe` ผ่าน; corpus ครบ 222 ไฟล์ readiness 85.4%, page counts 162/222, page breaks 87/222. `govuk-003` มีจำนวนหน้าเท่ากับ reference (5) แต่ page-boundary check ยังไม่ผ่าน จึงไม่เปลี่ยนสถานะ check. `cargo xtask ci` ยังติด workspace Clippy จาก unused import ใน `apps/doc4me/src/screen_clip.rs` และ `chunks_exact_to_as_chunks` ใน `apps/doc4me/src/dictate.rs` ซึ่งอยู่นอก diff นี้; dialogs 183/228, localization 30/95 และ ecosystem 4/6 (หนึ่งรายการยังไม่ทดสอบและหนึ่ง deferred) ยังคงเป็นช่องว่าง.

รอบนี้เพิ่ม `w:gridBefore`/`w:gridAfter` ใน row model, DOCX reader/writer, table layout, cell/column helpers และ flow conversion เพื่อไม่เลื่อน cell ที่ขาด grid columns ต้น/ท้ายแถวไปยังคอลัมน์ผิด; govuk-009 ผ่าน page-count และ page-break check หลังแก้. วัด corpus ครบ 222 เอกสาร: readiness 85.4% (87.8% เมื่อตัด deferred), pagination 161/222 และ 87/222. ผ่าน `cargo fmt --all -- --check`, model (118 tests), DOCX (75 unit + 49 round-trip), docbin, formats (70), layout regression, engine table tests (36) และ Clippy ของ 6 crates. `cargo xtask readiness --download --corpus --jobs 4 --cli target/release/doc4me-cli.exe` ผ่าน; `cargo xtask ci` หยุดที่ workspace Clippy จาก unused import ใน `apps/doc4me/src/screen_clip.rs` และ lint `chunks_exact_to_as_chunks` ใน `apps/doc4me/src/dictate.rs` ซึ่งอยู่นอก diff นี้ จึงไม่ถึง gates ที่เหลือ. ยังไม่ถึงเป้าหมาย 100%, โดย dialogs และ localization ยังเป็นช่องว่างหลัก และ Nextcloud/Syncthing ยังไม่มีหลักฐานผ่าน.

รอบนี้แก้ `crates/layout/src/lib.rs` ให้ `w:framePr` marker ของ Word ที่มีค่า auto/zero-size ตามรูปแบบที่พบใน corpus อยู่ใน page flow แทนการรวมเป็น floating frame เดียว; เพิ่ม regression test ใน `crates/layout/src/tests.rs`. `zenodo-030` กลับมาแสดงเนื้อหาที่เคยถูกตัดและจำนวนหน้าเพิ่มจาก 16 เป็น 34 (Word 32) แต่ยังไม่ผ่าน exact page-count; aggregate pagination คงเดิมที่ 160/222 และ 86/222. `cargo xtask readiness --corpus --jobs 4 --cli target/release/doc4me-cli.exe` วัดครบ 222 เอกสารและ readiness เป็น 85.3%. Layout suite แบบข้าม 3 test ผ่าน 233 tests; suite เต็มมี 3 failures เพราะ test เหล่านั้นคาดว่า TH Sarabun New/PSK จะไม่ติดตั้งเพื่อทดสอบ fallback แต่ Windows profile นี้มี TH Sarabun New ติดตั้งอยู่; fmt และ Clippy ของ layout ผ่าน. `cargo xtask ci` หยุดที่ workspace Clippy เพราะข้อผิดพลาดเดิมใน `apps/doc4me/src/screen_clip.rs` และ `apps/doc4me/src/dictate.rs` (ไม่อยู่ใน diff นี้) จึงยังไม่ถึง test/assets/layers/wasm gates.

ทดสอบ ecosystem `eco-003` ด้วย Syncthing 2.1.7 แบบสอง peer บน loopback แล้ว แต่ peer ปฏิเสธ cluster configuration และไม่ได้ยืนยันว่า Doc4Me บันทึก DOCX ทับตำแหน่งแล้ว sync/reopen ได้ จึงคงสถานะ `untested` และไม่นับผ่าน; Nextcloud Desktop ยังไม่ได้ตรวจ.

รอบนี้แก้การจัดประเภทภาพใน `crates/docx/src/read/story.rs`: เมื่อ `wps:wsp` มี `w:txbxContent` ภาพที่อยู่ข้างในเป็นเนื้อหาของ story ไม่ใช่รูปภาพแทนกล่องข้อความ; เพิ่ม regression test ที่มีภาพ PNG จริงและตรวจ round-trip. corpus ผ่าน 222/222 (รวม zenodo-153); คะแนน readiness รวมและคะแนนเมื่อตัด deferred ยังเท่าเดิมหลังปัดเศษ. ตรวจ `cargo fmt --all`, `cargo test -p doc4me-docx`, `cargo clippy -p doc4me-docx --all-targets -- -D warnings`, release build และ readiness corpus ครบ. เป้าหมาย 100% ยังไม่สำเร็จ; งานหลักที่เหลือคือ pagination, dialogs และภาษา.

รอบนี้เพิ่ม `Alt+Shift+F10` ให้ปิด KeyTips (ถ้าเปิดอยู่) และเปิดเมนูคำสั่งของเอกสารตรง caret; Escape ปิดกลับได้; UI regression test ผ่าน, `ks-319` เปลี่ยนเป็นผ่านตามหลักฐาน; shortcuts 268/280 และ readiness รวมยัง 85.2% (87.6% เมื่อตัด deferred). ตรวจ `cargo fmt --all -- --check`, UI suite แบบข้าม network integration test (394 ผ่าน), `cargo clippy -p doc4me-ui-egui --all-targets -- -D warnings` และวัด corpus readiness ครบ. UI suite เต็มมี Zotero integration test ล้มเหลวเพราะ remote host ปิดการเชื่อมต่อ; งาน 100% ยังไม่สำเร็จ.

รอบนี้แก้ `Alt+Shift+Plus` ให้สลับขยาย/ยุบหัวข้อใน Outline ตามสถานะ และเลิกดัก `Alt+Shift+Minus` เพื่อให้แทรก nonbreaking hyphen ได้ตามปกติ; regression test ตรวจ mapping และทั้งสองสถานะ; ปรับรายการ `ks-257`/`ks-258`. ตรวจ `cargo fmt --all -- --check` ผ่าน, `cargo test -p doc4me-engine --lib outline_show_level_filters_paragraphs` (1 ผ่าน), `cargo test -p doc4me-ui-egui` (395 ผ่าน) และ readiness corpus ครบ (85.2%). `cargo xtask ci` ยังไม่ผ่าน: Clippy พบ unused import ใน `apps/doc4me/src/screen_clip.rs` และ lint `chunks_exact_to_as_chunks` ใน `apps/doc4me/src/dictate.rs` ซึ่งไม่ได้อยู่ใน diff นี้; `cargo test --workspace` มี test failures ใน control server/XDG app dirs และ font-dependent tests บนเครื่องนี้. งานใหม่อยู่ใต้ `[Unreleased]`; ยังไม่ออกเวอร์ชัน.

### การตัดสินใจถาวรของผู้ใช้ (ทำตามเสมอ ไม่ต้องถามหรือแจ้งอีก)

- เป้าหมาย (สั่ง 2026-11-03): ทำงานต่อจน readiness **100% เมื่อไม่นับ 26 แถว deferred** แล้วรายงานผล; ลำดับ: **pagination ก่อน** แล้วจึง localization และ dialogs/shortcuts; ไม่เพิ่มแถว deferred (เกณฑ์คงที่ 26 แถว)
- Localization (สั่ง 2026-11-03): **ใช้ subagent แปลหลายภาษาพร้อมกันได้**; ระบุในหัวไฟล์และ `CHANGELOG.md` ว่าเป็นคำแปลจาก AI ที่ยังไม่ได้ให้เจ้าของภาษาตรวจ; คง clean room (ไม่ดูคำแปลของ Microsoft); ภาษาที่ต้องใช้ฟอนต์/การจัดรูปอักษรนอกเหนือ Inter (อารบิก ฮีบรู อินเดีย อาร์เมเนีย จอร์เจีย สิงหล เกาหลี ฯลฯ) ต้องให้ UI วาดได้จริงก่อนนับว่าผ่าน
- Dialog 8 แถวที่ Word รุ่นปัจจุบันตัดออกแล้ว (`dlg-081`/`dlg-082` WordPerfect Help, `dlg-151` Schema Library, `dlg-234` XML Element Attributes, `dlg-235` XML Options, `dlg-215` Smart Tag Options, `dlg-079` Frames Page, `dlg-010` CSS Links, `dlg-052` Format Callout) **ให้สร้างให้ใช้งานได้จริง** (มีผลจริงต่อเอกสาร ไม่ใช่ dialog เปล่า) ไม่ defer (สั่ง 2026-11-03)
- แจ้งสถานะความคืบหน้า readiness ให้ผู้ใช้เป็นระยะ (สั่ง 2026-11-03): หลังผลวัด Windows แต่ละรอบ ให้รายงานตัวเลข (รวม, ไม่นับ deferred, dialogs, การจัดหน้า) และสิ่งที่เปลี่ยน
- วัด readiness บน Windows ทุกครั้งหลังเปลี่ยนโค้ด ด้วย workflow `readiness-windows.yml` (สั่ง 2026-11-03) แล้วบันทึกผลใน handoff และ ROADMAP พร้อม commit/push `main`
- ไม่ต้องสั่งรัน workflow Windows x64 installer ทุกครั้งที่ commit push `main` เว้นแต่จะสั่งให้ออกเวอร์ชันใหม่ (สั่ง 2026-11-03; trigger ของ `main` ถูกเอาออกแล้ว)

- ตอบและแสดงข้อความเป็นภาษาไทยเท่านั้น (ชื่อคำสั่ง ชื่อไฟล์ และโค้ดคงเดิมได้)
- ห้ามออกเวอร์ชันใหม่จนกว่าผู้ใช้สั่ง (งานใหม่อยู่ใต้ `[Unreleased]`, ห้ามสั่ง workflow พร้อม `release: true`); 2026-11-02 ผู้ใช้สั่งออก v1.0.0, v1.1.0, v1.1.1 จาก commit เดิม และ v1.1.2 จากโค้ดล่าสุด (`Cargo.toml` 1.1.2) โดยออก v1.1.2 หลังรุ่นเก่า (ออกครบแล้วในรอบที่ 20 ที่ fork `veevie55/doc4me` ตามที่ผู้ใช้เลือก) รุ่นถัดจาก 1.1.2 ยังห้ามออกจนกว่าผู้ใช้สั่ง
- release workflow ไม่ตั้ง `CRAFT_FONTS_DIR` ข้ามฟอนต์ CJK ของ craft-fonts
- parity ที่ต้องใช้บริการเสียเงินให้ตัดทิ้ง
- ไม่เพิ่ม job ทดสอบบน Linux ใน CI เว้นแต่ผู้ใช้สั่ง
- ข้ามงาน build ของ Windows arm64, Linux และ macOS จนกว่าผู้ใช้จะสั่ง (สั่งเมื่อ 2026-10-26: ไม่สั่งรัน workflow "Windows arm64 and macOS builds" ไม่แก้ปัญหา build ของแพลตฟอร์มเหล่านี้ แถว plat-002 คงสถานะไว้ตามผลล่าสุด)
- ยังติดตาม WordCraft ต้นทางต่อ (เอกสารที่อ้าง PR/issue ของ WordCraft คงไว้)
- ระบบเสริมที่เป็นของ Microsoft (VBA, Office add-ins/Office Store, OneDrive/SharePoint, co-authoring ของ Microsoft) อยู่นอกเป้าหมาย ใช้ open-source แทน ถ้าหาตัวแทนไม่ได้ให้ตัดเงียบ ๆ
- ไม่มีผู้ให้บริการ AI ในโปรแกรม
- TH SarabunIT๙ ไม่มีค่าวัด: เครื่องที่ไม่มีฟอนต์นี้แสดงด้วย Sarabun และแจ้งผู้ใช้ตอนเปิดไฟล์; ไม่ต้องให้เจ้าของภาษาตรวจ UI ไทย
- `.gitignore` คง `/target` ฯลฯ ไว้ (ผู้ใช้เลือกลบแค่บรรทัด `/.claude/`)
- ไม่เปลี่ยนลิงก์ Releases ใน `README.md`/`README.en.md`: คงชี้ `https://github.com/chatraweechai/doc4me/releases` แม้ Release ของ v1.0.0–v1.1.2 อยู่ที่ fork `veevie55/doc4me` (ผู้ใช้สั่ง 2026-11-02 รอบที่ 20)
- ลำดับงาน (ผู้ใช้สั่งซ้ำ 2026-11-02): commit/push `main` ก่อน แล้วตรวจทีหลัง; ตรวจเสร็จ commit/push ที่แก้ แล้วบันทึก `handoff.md` และ `ROADMAP.md` ตามสถานะปัจจุบัน

### ประวัติถึงรอบที่ 8 (2026-10-25; รายละเอียดและผลตรวจอยู่ใน `CHANGELOG.md` `[Unreleased]`)

- `f158368` ลบบรรทัด `/.claude/` ใน `.gitignore`, ลบ `package-lock.json`, บันทึกคอมมิต `9954ae0` และ `fc7bab5` ใน CHANGELOG
- `adb9db9` TH SarabunIT๙: ใช้ Sarabun ขนาดของ Sarabun เอง และหน้าต่าง "ไม่มีฟอนต์นี้ในเครื่อง" ตอนเปิดไฟล์ (`cmd::file::unmatched_fonts`, `font_notice.rs`) — CI run 40189539051 ผ่าน
- `83e1611` ระบบเสริมของ Microsoft และ AI ในโปรแกรมออกจากเป้าหมาย (gap #22, #25) ตัวทดแทน: แมโครของ Doc4Me, แค็ตตาล็อก Add-in ของ Doc4Me, โฟลเดอร์ที่ Nextcloud Desktop/Syncthing ซิงก์
- `062d409` ฝัง Intos และ Intos Display (OFL, github.com/muglug/intos) แทน Aptos/Aptos Display — วัดเทียบไฟล์ Aptos จริงแล้วตรงทุกความกว้าง (ตัดบรรทัดตรง 387/387) ตัวอื่นที่ใกล้ที่สุดคือ Lato 35.7%; แผนความพร้อม 80% ใน `docs/roadmap.md`; handoff นี้ — CI run 40191882219 ผ่าน (build, MSI และ zip)
- ขั้น A เสร็จ (คอมมิต `cargo xtask readiness` ชุดล่าสุดบน `main`): เครื่องวัด `xtask/src/readiness.rs`, ชุดตรวจใน `docs/readiness/` (กติกาใน `README.md`), คลังเอกสารจริง 244 แถวนับ 222 (GOV.UK 72 OGL v3 + Zenodo 172 CC; 22 แถว GOV.UK เป็นคนละฉบับจึง `excluded`) ผลอยู่ที่ `docs/readiness.md` — CI run 40195346897 ของ `52d83ba` ผ่าน
- ขั้น B เสร็จ: `ROADMAP.md` และเอกสาร parity ใช้ค่าวัด 66.1% แทนค่าประมาณ ~60% (รายละเอียดใน CHANGELOG `[Unreleased]` หมวดเปลี่ยนแปลง)
- หมายเหตุ: รายละเอียดขั้น C และคะแนน 73.4% ในบรรทัดถัดไปเป็นประวัติ ณ 2026-10-25 ไม่ใช่สถานะล่าสุด; ผลปัจจุบัน 86.0% อยู่ในส่วนสถานะปัจจุบันด้านบน
- ขั้น C (เป้า 100% — ผู้ใช้ปรับจาก 95% เมื่อ 2026-10-27) กำลังทำ — ค่าวัดล่าสุด **73.4%** (`530f4cc`, ตารางด้านล่าง): `e1678b9` ระยะย่อหน้าแบบค่าที่มากกว่า + ระยะบรรทัด Aptos 1.2207 em, `eca31e8` ตัวแบ่งหน้าท้ายย่อหน้าและ Ctrl+Enter แบบ Word, `530f4cc` บรรทัดที่มีแต่รูปไม่บวก descent (จำนวนหน้าตรง 127 → 153, ต้น-ท้ายทุกหน้า 46 → 63), `1e00303` คีย์ลัดตามรายการของ Microsoft (ผ่าน 145 → 218/280), `10e228f` ข้อความหน้าจอทุก 18 ภาษาครอบคลุม 100% (ภาษาที่ผ่าน 6 → 19 จาก 95), แพ็กเกจ Linux และรุ่นเว็บผ่าน (plat-004, plat-005), `cafd5bf` `33146e8` `00e984e` กล่องโต้ตอบแบบ Word เพิ่ม 23 แถว (127 → 150/229: Break, Mark TOC Entry, Accept/Reject Changes, Highlight Changes, List Commands, Modify Style › Format ×5, Drawing Grid, Compatibility Options, Font Substitution, Insert Rows/Cells, Number, Placeholder และแถวที่มีกล่องอยู่แล้ว); เครื่องมือเทียบตำแหน่งบรรทัดกับ PDF ของ Word อยู่ใน scratchpad ของ session (ydrift.py ใช้ `pdftotext -bbox-layout`) ไม่ได้อยู่ใน repo
- ขั้น C ที่พบแต่ยังไม่ได้แก้ (การจัดหน้า): ฟอนต์ที่ไม่มีในเครื่อง Word เลือกตัวแทนจาก `w:altName` และ `w:family` (roman/swiss/modern) ใน `fontTable.xml` ซึ่ง Doc4Me ยังไม่อ่าน (เช่น govuk-010 Bembo → Times New Roman แต่เราใช้ฟอนต์ sans); เอกสารหลายไฟล์คลาดตั้งแต่ต้นหน้าเพราะความสูงของหัวกระดาษ (govuk-018 +21.7 pt, govuk-024 +15.4 pt, zenodo-025 +5.5 pt)

### ผลวัดก่อนหน้า (2026-10-25, คอมมิต `530f4cc`): 73.4%

| มิติ | น้ำหนัก | คะแนน | ชุดตรวจ |
|---|---|---|---|
| ฟีเจอร์ | 0.30 | 82.8% | ribbon 499/499, กล่องโต้ตอบ 150/229 |
| รูปแบบไฟล์ | 0.20 | 99.8% | เปิด 222/222, round-trip 221/222 |
| การจัดหน้า | 0.15 | 48.6% | จำนวนหน้า 153/222, คำต้น-ท้ายทุกหน้า 63/222 |
| UI/UX | 0.10 | 77.9% | คีย์ลัด 218/280 (ตรวจมือยังไม่ได้รัน 38) |
| ความเสถียร | 0.10 | 60.0% | ไม่มี crash/ค้าง 222/222, ตรวจมือ 1/5 |
| ประสิทธิภาพ | 0.05 | 66.7% | 4/6 |
| ภาษา | 0.04 | 20.0% | 19/95 |
| แพลตฟอร์ม | 0.03 | 60.0% | 3/5 (Windows x64, Linux x64, เว็บ) |
| ระบบเสริม | 0.02 | 66.7% | 4/6 |
| ฮาร์ดแวร์ | 0.01 | 25.0% | 2/8 |

ยังขาด 21.6 จุดจึงถึง 95%: การจัดหน้าให้ได้อีกมากที่สุด (7.7 จุด), กล่องโต้ตอบ 79 แถว (5.2 จุด; หลายแถวเป็นกล่องเก่าของ Word 97–2003), ภาษา 76 ภาษา (3.2 จุด; ต้องแปลภาษาใหม่ทั้งชุดราว 2,600 ข้อความต่อภาษา และ egui ยังไม่รองรับข้อความขวาไปซ้าย), แถวตรวจมือของความเสถียร ประสิทธิภาพ แพลตฟอร์ม ฮาร์ดแวร์ ซึ่งส่วนใหญ่ต้องใช้เครื่อง Windows/macOS ของเจ้าของ repo

### ผลวัดครั้งแรก (2026-10-25, คลัง 222 เอกสาร ตัวเลขสุดท้าย): 66.1%

| มิติ | น้ำหนัก | คะแนน | ชุดตรวจ | ได้เพิ่มสูงสุด (จุด) |
|---|---|---|---|---|
| ฟีเจอร์ | 0.30 | 77.7% | ribbon 499/499, กล่องโต้ตอบ 127/229 | 6.7 |
| รูปแบบไฟล์ | 0.20 | 99.8% | เปิด 222/222, round-trip 221/222 (zenodo-153 กล่องข้อความหาย 5 → 4) | 0.05 |
| การจัดหน้า | 0.15 | 39.0% | จำนวนหน้า 127/222, คำต้น-ท้ายทุกหน้า 46/222 | 9.2 |
| UI/UX | 0.10 | 51.8% | คีย์ลัด 145/280 (ตรวจมือยังไม่ได้รัน 42) | 4.8 |
| ความเสถียร | 0.10 | 60.0% | ไม่มี crash/ค้าง 222/222, ตรวจมือ 1/5 | 4.0 |
| ประสิทธิภาพ | 0.05 | 66.7% | 4/6 (การพิมพ์และเลื่อนตรวจมือ 2) | 1.7 |
| ภาษา | 0.04 | 6.3% | 6/95 (ภาษาอื่นแปลไว้ราว 84%) | 3.7 |
| แพลตฟอร์ม | 0.03 | 20.0% | 1/5 | 2.4 |
| ระบบเสริม | 0.02 | 66.7% | 4/6 | 0.7 |
| ฮาร์ดแวร์ | 0.01 | 25.0% | 2/8 | 0.75 |

ต้องได้เพิ่มอีก 28.9 จุดจึงถึงเป้า 95% (ผู้ใช้เปลี่ยนเป้าจาก 80% เป็น 95% เมื่อ 2026-10-25) วิธีรัน: ติดตั้ง poppler-utils, `cargo build --release -p doc4me-cli` แล้ว `cargo xtask readiness --cli target/release/doc4me-cli --download --corpus` (ราว 3 นาที ไฟล์ลง `corpus/readiness/` ที่ไม่เข้า git)

### งานที่ต้องทำต่อ: ความพร้อมใช้งานจริง 100% แบบวัดได้ (ไม่ใช่ค่าประมาณ; เป้าเดิม 95%)

ตัวเลขใน `ROADMAP.md` เดิมเป็นค่าประมาณ ~60% (ผลรวมถ่วงน้ำหนักของค่าที่ประมาณรายมิติ) ตอนนี้เป็นค่าวัด 66.1% แล้ว (ขั้น A และ B เสร็จ) งานต่อไปคือขั้น C แก้ตามผลจนได้ 95% ฉบับภาษาอังกฤษอยู่ที่ `docs/roadmap.md` หัวข้อ "Path to 95% measured readiness" ทุกข้อต้องบันทึก `[Unreleased]` และ commit/push ไป `main`

**ขั้น A สร้างเครื่องวัด (เสร็จแล้ว 2026-10-25 ผลอยู่ด้านบน รายการเดิมคงไว้เป็นข้อกำหนด)**

1. `cargo xtask readiness` (คำสั่งใหม่ใน `xtask/src/main.rs`): อ่านผลของชุดตรวจทุกมิติ คำนวณ Σ น้ำหนัก × อัตราผ่าน ด้วยน้ำหนักเดิม (ฟีเจอร์ 0.30, รูปแบบไฟล์ 0.20, การจัดหน้า 0.15, UI/UX 0.10, ความเสถียร 0.10, ประสิทธิภาพ 0.05, ภาษา 0.04, แพลตฟอร์ม 0.03, ระบบเสริม 0.02, ฮาร์ดแวร์ 0.01) เขียนผลลง `docs/readiness.md` แล้วให้ `ROADMAP.md` ใช้ตัวเลขนี้แทนค่าประมาณ รายการตรวจเพิ่มได้แต่ห้ามลบ
2. คลัง DOCX จริงอย่างน้อย 200 ไฟล์ที่ใช้ซ้ำได้ (เอกสารราชการที่เป็นสาธารณสมบัติ หรือสัญญาอนุญาตเปิด) แต่ละไฟล์มี PDF อ้างอิงที่ Word สร้าง (ดู Producer ของ PDF ว่าเป็น "Microsoft Word") จึงไม่ต้องมี Word ในเครื่อง เก็บไฟล์ใน `/corpus` (gitignored) และ manifest (URL, สัญญาอนุญาต, SHA-256) ใน repo ต่อยอดจาก `cargo xtask corpus`
3. ตรวจรูปแบบไฟล์ต่อไฟล์: เปิดได้ไม่ error, บันทึกแล้วเปิดใหม่ ข้อความ สไตล์ ตาราง รูป และ part ที่ไม่รู้จักยังอยู่
4. ตรวจการแบ่งหน้าเทียบ PDF อ้างอิง: จำนวนหน้า และคำแรก/คำสุดท้ายของทุกหน้า (ใช้ตัวอ่าน PDF ของ repo หรือ `pdftotext`)
5. ตรวจความเสถียร: เปิด จัดหน้า บันทึก ทุกไฟล์ในคลังภายใต้ timeout ต้องไม่มี panic หรือค้าง, soak test ของ autosave, fuzz `hostile_params_never_panic` ที่มีอยู่
6. รายการตรวจฟีเจอร์และ UI: ตารางคงที่ของ Word for Windows (ฟีเจอร์ กล่องโต้ตอบ ตัวเลือกในกล่องหลัก และรายการคีย์ลัดเริ่มต้นที่ Microsoft เผยแพร่) แต่ละแถวผูกกับเทสต์ที่รันได้ หรือการตรวจด้วยมือบน Windows (เจ้าของ repo)
7. เกณฑ์คงที่ของประสิทธิภาพ (เปิดเอกสาร 500 หน้า, latency การพิมพ์, การเลื่อน), ภาษา (ความครบของ catalog ต่อภาษาเทียบ 30 ภาษาของ Word), แพลตฟอร์ม (build และติดตั้งได้ต่อแพลตฟอร์ม), ฮาร์ดแวร์และระบบเสริมแบบผ่าน/ไม่ผ่าน

**ขั้น B วัดครั้งแรก (เสร็จแล้ว 2026-10-25)** 66.1% แทนค่าประมาณ ~60% ใน `ROADMAP.md` (บรรทัด Stage, ตารางตัวเลขหลัก, ตารางกลุ่มผู้ใช้, คอลัมน์ Measured ในตารางรายมิติ, Why not beta, Upcoming ข้อ 0, 2, 5), `docs/target-app-parity.md`, `docs/gaps.md`, `docs/file-format-parity.md` และ `docs/roadmap.md`; ตัวเลข Mainstream ~55% และ Essentials ~63% ยังเป็นค่าประมาณเพราะเครื่องวัดไม่ได้วัดสองกลุ่มนี้

**ขั้น C แก้ตามผลวัด** เรียงจากน้ำหนักที่ได้คืนต่อชั่วโมงมากที่สุด จนกว่า `cargo xtask readiness` บนคอมมิตล่าสุดได้ ≥ 95.0% (คอลัมน์ "ได้เพิ่มสูงสุด" ในตารางด้านบนบอกว่าแต่ละมิติให้ได้อีกเท่าไร: การจัดหน้า กล่องโต้ตอบ คีย์ลัด และแถวตรวจมือของความเสถียรให้มากที่สุด) — แถวที่ผ่านได้บนเครื่องของเจ้าของ repo เท่านั้น (รายงานจากผู้ใช้ Windows ของความเสถียร stab-002/003/004, การพิมพ์และเลื่อนบน Windows perf-005/006, เครื่องพิมพ์ ปากกา ไมโครโฟน สแกนเนอร์ จอสัมผัส หลายจอ, Mendeley) ทำให้ session ขึ้นได้สูงสุดราว 94% จึงต้องให้เจ้าของ repo ตรวจด้วยมือด้วยจึงถึง 95%

1. gap #1 ไฟล์ DOCX จริงที่เปิดไม่ได้หรือ round-trip เสีย
2. gap #4 การแบ่งหน้า: compatibility options, ตารางและรูปลอย, เทียบกับ PDF อ้างอิง (Aptos ใช้ Intos ที่ความกว้างตรงแล้ว)
3. gap #3 crash/ค้าง/บันทึกไม่ได้บน Windows (#170, #77, #59, #31, #218)
4. gap #2 Word ink (`w14:contentPart`) round-trip
5. gap #6 การพิมพ์แบบ native บน Windows (เลือกเครื่องพิมพ์ พิมพ์สองหน้า)
6. gap #7 กล่องโต้ตอบและตัวเลือกที่ขาด และ gap #24 คีย์ลัดที่ขาด ตามรายการข้อ A6 ที่ไม่ผ่าน
7. gap #13 แก้ไขแผนภูมิและ SmartArt ที่มาจาก Word
8. รัน alpha gate 6 workflow ใหม่บน Windows (เจ้าของ repo)

### งานของเจ้าของ repo (session ทำไม่ได้)

- ใบรับรองลงนามโค้ดหรือ Azure Trusted Signing แล้วใส่ secret (workflow พร้อมแล้ว) (ข้อ 5.2)

## session 2026-10-25 รอบที่ 7: งานค้างใน handoff ข้อ 2–4

- **ผู้ใช้สั่ง (2026-10-25): "ต่อไปนี้ให้ตอบและแสดงข้อความเป็นภาษาไทยเท่านั้น"** ใช้กับทุกข้อความที่ผู้ใช้เห็น รวมถึงข้อความแจ้งความคืบหน้าระหว่างทำงาน (ชื่อคำสั่ง ชื่อไฟล์ และโค้ดคงเดิมได้)
- ผู้ใช้สั่ง "ทำต่อ" (ทำงานค้างที่ session ทำได้ ข้ามงานของเจ้าของ repo)
- ทำแล้ว: เอกสารตรงกับโค้ด (`b303ed9`: ตัวเลข 625 คำสั่ง · 1,816 เทสต์ · ~188,000 บรรทัด, ROADMAP Upcoming, gap #29, README สองภาษา); `cargo audit` ไม่มีช่องโหว่ (คำเตือน unmaintained 5 รายการ)
- `cargo xtask ci` ทั้ง workspace (หลัง `cargo clean`, `CARGO_INCREMENTAL=0`): fmt, clippy, test (ผ่าน 1,809 ล้ม 0 ignored 0) ผ่าน แต่ล้มที่ `assets` เพราะสคริปต์ `docs/images/shots/*.jsonl` 11 ไฟล์ไม่มีแถวใน `ATTRIBUTION.md` (ค้างมาตั้งแต่ 2026-10-21) แก้แล้วใน `c6a91fe`; `assets`, `layers`, `wasm` ผ่าน — CI บน GitHub ไม่รันขั้นเหล่านี้ ควรรัน `cargo xtask ci` เองก่อนออกรุ่น
- ถ่ายภาพใน `docs/images` ใหม่ทั้ง 11 ภาพ: เปลี่ยนจริงแค่ `review.png` (ปุ่ม Translate) ที่เหลือต่างแค่เคอร์เซอร์/วันที่จึงคืนค่าเดิม
- ยังเหลือเฉพาะงานของเจ้าของ repo (ข้อ 5.1–5.3: แท็ก v1.0.0, secret ลงนาม, ruleset, ลบสาขาเก่า, PR Dependabot #1–#4) และการทดสอบด้วยมือบนเครื่องจริง (Dictate กับไมโครโฟน, Screen Clipping บน Windows, เปิดไฟล์ที่มีวิดีโอออนไลน์ใน Word, วัด parity/alpha gate ใหม่บน Windows)

## session 2026-10-25 รอบที่ 6: parity ที่เหลือ 8 รายการด้วยของฟรี/โอเพนซอร์ส

- ผู้ใช้สั่ง: "ทำส่วน parity ที่เหลือ 8 รายการ โดยเลือกใช้รายการ open-source หรือฟรีเป็นอย่างแรก หากจำเป็นต้องใช้แบบมีค่าใช้จ่ายให้ตัด parity นั้นทิ้งไป และไม่ต้องแจ้งอีก" และ "ไม่ต้องให้ release workflow ตั้ง CRAFT_FONTS_DIR ต่อไปนี้ข้ามฟอนต์ CJK ของ craft-fonts ไม่ต้องแจ้งอีก"
- **การตัดสินใจถาวรของเจ้าของ repo (ไม่ต้องถามหรือแจ้งอีก)**: (1) release workflow ไม่ตั้ง `CRAFT_FONTS_DIR` ข้ามฟอนต์ CJK ของ craft-fonts; (2) parity ที่ต้องใช้บริการเสียเงินให้ตัดออกจาก catalog โดยไม่ต้องแจ้ง
- ทำแล้ว (รายละเอียดและผลตรวจใน `CHANGELOG.md` `[Unreleased]`):
  - บริการเรียกเว็บ `Services::fetch` (desktop: `curl` ของระบบ https เท่านั้น; เว็บ: `fetch` ของเบราว์เซอร์) — `caa9274`
  - Online Pictures ผ่าน Openverse (ฟรี CC) พร้อมบรรทัดเครดิต — `caa9274`
  - Translate ผ่าน MyMemory (ฟรี) หรือ LibreTranslate (โอเพนซอร์ส) — `98b4cce`
  - Get Add-ins: แค็ตตาล็อก `addins/catalog.json` ใน repo นี้ + 3 ชุดแมโคร ตรวจ SHA-256 และคำสั่งที่อนุญาต — `d9b7edd` (ตรวจบน GitHub จริงแล้ว hash ตรง)
  - Ink Equation + Ink to Math ด้วยตัวจดจำ $P ของเราเอง — `af604a3`
  - Dictate ด้วย whisper.cpp บนเครื่อง (โมเดลจาก Hugging Face ตรวจ SHA-256) — `8aa0dea`; ทดสอบจริงด้วยเสียงสังเคราะห์ espeak-ng ผ่าน; CI Windows run 40178908529 ล้มเพราะ bindings ที่มากับ crate เป็นของ Linux → แก้โดยให้ bindgen สร้างบนแต่ละแพลตฟอร์ม (`6f4ded8`) แล้ว [run 40179588066](https://github.com/chatraweechai/doc4me/actions/runs/40179588066) ผ่าน (build + MSI + zip; release ข้าม)
  - Block Authors ตัดออก (ต้องใช้ SharePoint ของ Microsoft 365 แบบเสียเงิน)
- สภาพแวดล้อม: ติดตั้ง `libasound2-dev` (cpal) และ `espeak-ng` (ทดสอบ Dictate) ใน container; build ใช้ `CARGO_INCREMENTAL=0` และ `-p` ทีละ package เพราะดิสก์จำกัด (เคย `cargo clean` ได้คืน 26 GB)
- ทดสอบกับบริการจริง: `DOC4ME_NET_TESTS=1 cargo test -p doc4me fetch` (Openverse, httpbin), `DOC4ME_WHISPER_TEST=<wav 16-bit mono> cargo test -p doc4me real_model` (ดาวน์โหลดโมเดล 60 MB)

## session 2026-10-24: ปุ่มแปลงเลขไทย/อารบิก, JPG/JPEG และงานค้างจาก handoff

- ผู้ใช้สั่ง (ตามลำดับ)
  - "ดำเนินการต่อจากที่ค้างให้เสร็จ และเพิ่มปุ่มฟังก์ชันในริบบอนแปลงเลขอารบิกเป็นเลขไทย และแปลงเลขไทยเป็นเลขอารบิก"
  - "เมื่อทำเสร็จทั้งหมดรวมทั้งงานที่ค้าง ห้ามออกเวอร์ชันใหม่ ให้เพิ่มฟังก์ชันการ save as และ export file เป็นนามสกุลไฟล์ jpg/jpeg ด้วย"
  - กฎของผู้ใช้: push ไป `main` เท่านั้น, commit/push ก่อนแล้วตรวจบั๊กทีหลังแล้ว push อีกครั้ง, บันทึก handoff ทุกครั้ง
- **ห้ามออกเวอร์ชันใหม่** (ผู้ใช้สั่ง 2026-10-24): `Cargo.toml` ยังเป็น 1.1.1 และทุกอย่างอยู่ใต้ `[Unreleased]` อย่าสั่ง workflow พร้อม `release: true` จนกว่าผู้ใช้จะสั่ง
- ทำแล้ว (รายละเอียดและผลตรวจอยู่ใน `CHANGELOG.md` `[Unreleased]`)
  - ปุ่ม "แปลงเป็นเลขไทย" / "แปลงเป็นเลขอารบิก" บนริบบอน หน้าหลัก › ฟอนต์ (`format.thaiDigits`, `format.arabicDigits`)
  - Save As และ File › Export เป็น JPG/JPEG (`file.exportJpeg`, `SAVE_EXTS`, CLI, MCP, เว็บ)
  - ข้อ 1 (เวลา UTC): `now_local` ใช้กับวันที่และเวลาที่แสดงผู้ใช้ เพิ่ม dependency `chrono` (desktop เท่านั้น) วันที่ที่เก็บในไฟล์ยังเป็น UTC
  - ข้อ 2 และ 5.6: เอกสารที่ล้าสมัยแก้ตามที่ตรวจกับโค้ดแล้ว, Target เป็น Word for Windows (ไม่มีเลขรุ่น), `cargo xtask parity` ได้ 485/500 (97.0%)
  - ข้อ 3: xtask USAGE มี `version`, PngSuite ตรวจ SHA-256, `.github/dependabot.yml`; ข้อ 4: เลขเวอร์ชันใน `packaging/` และ `apps/` ไม่มีที่เขียนตายตัว (อ่านจาก `Cargo.toml`)
  - ข้อ 5.2 ส่วนของ session: workflow แยก build (ไม่มี secret) กับขั้นแพ็ก/ลงนาม (ได้ secret) และ `docs/releasing.md`
  - ข้อ 5.4: เจ้าของ repo ลบ `.claude/settings.json` แล้วใน `e5214c4` บันทึกใน `[Unreleased]` › ความปลอดภัย
- **[เจ้าของ repo]** ต้องทำเอง
  - ข้อ 5.1: tag และ Release `v1.0.0` (คำสั่งอยู่ในข้อ 5.1 ข้างล่าง ยังไม่มี tag นี้ ตรวจ 2026-10-24)
  - ข้อ 5.2: ใส่ secret ลงนามที่ Settings › Secrets and variables › Actions (ชื่ออยู่ใน `docs/releasing.md` ข้อ 2)
  - ข้อ 5.3: ruleset ป้องกัน `main`
  - ข้อ 5.5: ลบ branch `claude/optimistic-fermat-z9uknk` ที่หน้า Branches (merge เข้า `main` แล้ว ตรวจด้วย `merge-base --is-ancestor`; `git push origin --delete` ถูก git proxy ตัดการเชื่อมต่อสองครั้ง) และ branch `claude/affectionate-ptolemy-92v2ut` ของ session นี้ถ้ามีบน remote (session ไม่ได้ push branch นี้)
  - PR ของ Dependabot ([#1](https://github.com/chatraweechai/doc4me/pull/1) rust-cache 2.9.2 → 2.9.3, [#2](https://github.com/chatraweechai/doc4me/pull/2) checkout 4.4.0 → 7.0.1, [#3](https://github.com/chatraweechai/doc4me/pull/3) download-artifact 4.3.0 → 8.0.2, [#4](https://github.com/chatraweechai/doc4me/pull/4) upload-artifact 4.6.2 → 7.0.2): session ไม่ได้ merge เพราะเป็นการขึ้นเวอร์ชันหลักที่แก้ไฟล์ workflow และ PR ไม่รัน build (ไม่มี trigger `pull_request`) ให้อ่าน release notes ใน PR ก่อน merge โดยเฉพาะ download-artifact ที่ job `release` ใช้ แล้วสั่ง run บน `main` (ไม่ติ๊ก `release`) หลัง merge ห้าม merge ระหว่างที่ job `release` รันอยู่
- รอบที่ 3 (ผู้ใช้สั่ง: แก้วันที่ในบานหน้าต่างข้อคิดเห็น, ให้ปุ่มแปลงเลขเปลี่ยนเลขหน้าและเลขลำดับรายการ, เปลี่ยนทางลัด ⌘ ในเอกสารเป็นแบบ Windows) ทำแล้ว ดู `[Unreleased]`
  - `w:date` ของข้อคิดเห็นและ revision ใหม่เป็นเวลาท้องถิ่นแบบ Word และอ่าน `w16cex:dateUtc` (ยังไม่เขียนกลับ และยังไม่อ่าน `w16du:dateUtc` ของ revision)
  - ปุ่มแปลงเลขเปลี่ยนรูปแบบ decimal ↔ thaiNumbers ของรายการ เลขหน้า เลขเชิงอรรถ และเลขอ้างอิงท้ายเรื่อง
  - ทางลัดในเอกสารใช้ Ctrl (ไม่ใช่ ⊞ เพราะ Word for Windows ใช้ Ctrl) และคำแนะนำ "Paste"/"Go To" ในโปรแกรมแสดงทางลัดตามแพลตฟอร์ม
  - ตรวจแล้ว: เทสต์ doc, docx, engine (382 + 13 + 8), cli, mcp, layout (224), pdf ผ่านทั้งหมด, ui-egui 324 (`--include-ignored`), clippy ผ่าน; CI ของ `637bd89` (run 40106198767) ผ่าน job `release` ข้ามตามปกติ
  - **ผู้ใช้สั่งซ้ำ 2026-10-24: ห้ามออกเวอร์ชันใหม่จนกว่าจะสั่ง**
- รอบที่ 4 (ผู้ใช้สั่ง "แก้ข้อจำกัดที่เหลือทั้งหมด") ทำแล้ว ดู `[Unreleased]`
  - เขียน `commentsIds.xml` / `commentsExtensible.xml` (เวลา UTC ของข้อคิดเห็น) และอ่าน-เขียน `w16du:dateUtc` ของ tracked change (`Revision.date_utc`); ค่าของ namespace, relationship และ content type ตรวจกับ MS-DOCX และเอกสารของ Paper Instruments แต่ยังไม่ได้เทียบกับไฟล์ที่ Word บันทึกเอง **ควรเปิดไฟล์ที่ Doc4Me บันทึก (มีข้อคิดเห็นและ tracked change) ใน Word for Windows เพื่อยืนยันว่าไม่มีคำเตือนไฟล์เสีย**
  - ตรวจแล้ว: clippy `-D warnings` `--all-targets` (doc, docx, layout, engine, ui-egui, pdf) ผ่าน; เทสต์ engine 383 + 13 + 8, ui-egui 324 (`--include-ignored`), pdf, mcp, cli, docx และ layout ผ่านทั้งหมด; CI run 40108607965 ผ่าน job `release` ข้ามตามปกติ
  - ปุ่มแปลงเลขสลับ `\* Arabic` ↔ `\* ThaiArabic` ในฟิลด์ PAGE/NUMPAGES/SECTIONPAGES/SECTION/SEQ และตัวจัดวางรู้จัก `ThaiArabic`, `ThaiCardText`, `ThaiLetter`
  - เอกสาร parity: เป้าหมาย Word for Windows อธิบายใน `docs/target-app-parity.md`, แถวเครื่องพิมพ์และสแกนเนอร์ใน `docs/hardware-parity.md`, แถว Platforms ตรงกับ Release จริง
- รอบที่ 5 (2026-10-25, ผู้ใช้สั่ง "ตรวจสอบ parity ที่เหลือ ทำให้เสร็จทั้งหมด และดำเนินการส่วนที่ค้างใน handoff ยกเว้นที่ผู้ใช้ต้องทำเอง")
  - parity ที่ขาด 15 รายการ แบ่งเป็น ทำได้เอง: View › Window (เสร็จแล้ว), Screenshot, Online Video, 3D Models; ต้องให้เจ้าของ repo ตัดสินใจหรือใช้บริการภายนอก (ข้าม): Dictate (โมเดลเสียง, gap #28), Online Pictures (Bing), Get Add-ins (Office Store, gap #22), Translate (บริการแปล), Ink Equation และ Ink to Math (โมเดลจำลายมือ), Block Authors (co-authoring ของ SharePoint, gap #22)
  - View › Window: `view.newWindow` และ `view.split` เดิมเป็น stub ตอนนี้ทำงานจริง และเพิ่ม `view.arrangeAll`, `view.sideBySide`, `view.syncScroll`, `view.switchWindows` (`crates/ui-egui/src/windows.rs`) parity 489/500
  - `cargo audit`: อัปเดต zip 3 → 9.1 (ช่องโหว่ 3 รายการใน `extract` ซึ่งโค้ดไม่เรียก), ctutils, tokio, clipboard-win; เหลือคำเตือน unmaintained 5 รายการ
  - ดิสก์ของ container เคยเต็ม (linker ล้มด้วย Bus error) แก้โดยลบไฟล์ทดสอบขนาดใหญ่ใน `target/debug/deps` และ `target/debug/examples`
  - Insert › Screenshot › Screen Clipping (`a47ba63`, parity 490): engine `insert.screenshot` (pure) → UI `screen_clip.rs` → host `apps/doc4me/src/screen_clip.rs` (Windows `ms-screenclip:` + `clipboard_win::raw::seq_num`, macOS `screencapture -i -x`, Linux XDG Screenshot portal แล้ว gnome-screenshot/spectacle/xfce4-screenshooter/flameshot) ตรวจ `cargo clippy -p doc4me --target x86_64-pc-windows-msvc` และ `x86_64-apple-darwin` ผ่าน; CI Windows ผ่าน
  - Insert › Online Video (`4cb88d6`, parity 491): `WebVideo` (`crates/doc/src/video.rs`) ในฟิลด์ใหม่ `InlineObject::Image.video`, .docx `wp15:webVideoPr` ที่ ext uri `{C809E66F-F1BF-436E-B5F7-EEA9579F0CBA}` (ยืนยันจากตัวอย่าง OOXML จริง ไม่ใช่เดา), กล่อง Insert Video, ดับเบิลคลิก/`picture.playVideo` เปิดเบราว์เซอร์, โปสเตอร์วาดเอง
  - Insert › 3D Models (`4cb88d6` + `46ef579`, parity 492): `crates/engine/src/model3d.rs` อ่าน .glb/.stl/.obj วาดเป็นรูปด้วย z-buffer; หลัง push ตรวจพบและแก้ node glTF ที่วนอ้างตัวเอง + จำกัดงานอ่านรวม (`MAX_WORK`)
  - parity ที่เหลือ 8 รายการ (ข้ามตามคำสั่ง เพราะต้องใช้บริการภายนอกหรือโมเดล ML ที่เจ้าของ repo ต้องตัดสินใจ): Dictate, Online Pictures, Get Add-ins, Ink Equation (อยู่ทั้ง Insert และ Draw จึงนับ 2), Ink to Math, Translate, Block Authors (co-authoring ของ SharePoint) ตาม `docs/parity-checklist.md`
  - งานค้างใน handoff ที่ทำแล้ว: wasm check (`rustup target add wasm32-unknown-unknown`; `cargo clippy -p doc4me-web --target wasm32-unknown-unknown -- -D warnings` ผ่านหลังแก้ lint 3 จุด), สาเหตุ audit ภาษาที่ติด `#[ignore]` (ข้อความผิดพลาด Read Aloud ไม่แปลและมาถึงแบบ async; แก้แล้วเอา `#[ignore]` ออก ทดสอบ 24/24 ภายใต้โหลด), เอกสารล้าสมัย (file-format-parity การเข้ารหัส, roadmap M1/M2/M3/M8/M11/M12 + 618 คำสั่ง, package.ps1 อ้าง windows-arm64.yml, env.sh/ATTRIBUTION อ้างว่า release ฝัง craft-fonts — ตรวจไฟล์ workflow ที่ tag v1.1.1 แล้วไม่มี, docs/mcp.md "over 270")
  - ดิสก์ของ container เต็มซ้ำ 2 ครั้ง: `cargo build --workspace --all-targets` สร้าง dependency ชุดที่สอง (feature ต่างกัน) และ `target/debug/incremental` ใหญ่ ~11 GB — ใช้ `CARGO_INCREMENTAL=0` และ build/test ทีละ package (`-p`) แทน
- ยังไม่ได้ทำ / ข้อสังเกต
  - ภาพ `hero`, `thai`, `dark`, `dark-th`, `font`, `font-th` ถ่ายใหม่แล้ว ภาพอื่นใน `docs/images` (backstage, design, layout, references, review) ไม่เห็นกลุ่มฟอนต์ของหน้าหลักจึงไม่ได้ถ่ายใหม่
  - ตัวเลข parity และ alpha gate ยังเป็นค่าที่วัดกับ Word for Mac ต้องวัดใหม่บนเครื่อง Windows ที่มี Word (session บนคลาวด์ทำไม่ได้)
  - ข้อ 3 ที่เหลือ: job ทดสอบบน Linux ใน CI รอเจ้าของ repo ตัดสินใจ (`cargo audit` รันแล้วในรอบ 5; audit ภาษาที่ติด `#[ignore]` หาสาเหตุและแก้แล้ว)
  - **[เจ้าของ repo]** เพิ่มเติมจากรอบ 5: เปิดไฟล์ .docx ที่ Doc4Me บันทึกซึ่งมีวิดีโอออนไลน์ใน Word for Windows เพื่อยืนยันว่า Word เล่นได้; ทดสอบ Screen Clipping และ Dictate (ไมโครโฟน) บน Windows/macOS/เดสก์ท็อป Linux ด้วยมือ
  - ข้อ 5.7 ทั้งหมดยังรอเจ้าของ repo
- CI ของ `main`: run ของ `21bb34a` (40100455417) และ `b65c320` (40100747357) ถูกยกเลิกโดย concurrency เพราะมี push ใหม่ (ไม่ใช่การล้ม); [run 40101348687](https://github.com/chatraweechai/doc4me/actions/runs/40101348687) ของ `0cdf2bb` (คอมมิตโค้ดล่าสุด) **ผ่าน** (ตรวจ 14:26 UTC): job `build` สำเร็จทุกขั้น รวมขั้นใหม่ "Build doc4me + doc4me-cli" (14:11:45–14:22:23) และ "Sign and package MSI + portable zip" (14:22:23–14:23:05) ส่วน job `release` ถูกข้ามตามปกติของ push; คอมมิตหลังจากนั้นแก้แค่เอกสารและภาพ จึงใส่ `[skip ci]` run นี้จึงเป็น build ล่าสุดของโค้ด
- ผลตรวจในเครื่องรอบสุดท้าย: engine 378 + 13 + 8, cli/mcp/render ผ่าน, ui-egui 324 (`--include-ignored`), clippy `-D warnings` `--all-targets` ของ engine, ui-egui, cli, mcp, render, xtask ผ่าน, `cargo fmt --all --check` ผ่าน, `cargo xtask parity` 485/500; ภาพหน้าจอยืนยันปุ่มแปลงเลขและผลการแปลง
- ยังไม่ได้รัน `cargo xtask ci` ทั้ง workspace (wasm target ไม่ได้ติดตั้ง และดิสก์จำกัด) ส่วน wasm ของ `local_offset_secs` ตรวจแค่ว่า `js_sys::Date::new_0` และ `get_timezone_offset` มีใน js-sys 0.3.106

## งานที่ต้องทำต่อใน session ใหม่ (รวบรวม 2026-10-23)

**ประวัติ (2026-10-25 รอบที่ 8)**: ข้อ 1–4 และ 5.4–5.7 ปิดแล้ว ส่วน 5.1–5.3 เหลือเฉพาะงานของเจ้าของ repo ซึ่งสรุปไว้ในหัวข้อสถานะปัจจุบัน

ที่มา: workflow สำรวจงานค้าง (agent อ่านอย่างเดียว 2026-10-22 15:15 UTC) ซึ่งขั้นตรวจซ้ำล้มเพราะชนขีดจำกัดของ session

- ข้อที่มี **[ตรวจแล้ว]** ตรวจกับโค้ดเมื่อ 2026-10-23 04:10–04:15 UTC ข้ออื่นมาจาก finder ตัวเดียว ต้องตรวจซ้ำก่อนลงมือ
- ลำดับที่แนะนำ (ผู้ใช้ปรับได้):
  1. แจ้งงานที่ **[เจ้าของ repo]** ต้องทำเองในข้อ 5.1–5.3 ตั้งแต่ต้น session
  2. ทำข้อ 5.4 และ 5.5 (งานเล็ก)
  3. ทำข้อ 1 (บั๊ก UTC)
  4. ทำข้อ 2 รวมกับข้อ 5.6 (เอกสาร)
  5. ทำส่วนของ session ในข้อ 5.2 แล้วต่อด้วยข้อ 3
- งานโค้ดหรือเอกสารทุกชิ้นต้องบันทึกใน `CHANGELOG.md` `[Unreleased]` ในคอมมิตเดียวกัน (CLAUDE.md ข้อ 3) แล้ว push ไป `main`
- ห้าม push ไป `main` ระหว่างที่ job `release` ของ workflow รันอยู่ โดยเฉพาะการแก้ `.github/workflows`
- คอมมิตที่แก้แค่ `handoff.md` ใส่ `[skip ci]` ได้ run ล่าสุดของ `main` จะได้ไม่ใช่ run ที่ยังไม่จบ

### 1. บั๊ก: วันที่และเวลาที่แสดงผู้ใช้เป็น UTC ไม่ใช่เวลาของเครื่อง [ตรวจแล้ว]

- `iso_from_unix_secs` และ `now_iso` (`crates/engine/src/cmd/mod.rs:378-402`) สร้างเวลา UTC ลงท้าย `Z` และไม่มีโค้ดส่วนไหนอ่าน time zone ของเครื่อง
- ผลที่ผู้ใช้ไทย (UTC+7) เห็น:
  - คำทักทายบนหน้า Home ของ backstage (`crates/ui-egui/src/backstage.rs:194`) ขึ้น "สวัสดีตอนเช้า" ไปถึง 18:59 น.
  - Insert › Date and Time (`crates/engine/src/cmd/insert.rs:504` `format_date`) ได้วันของเมื่อวานช่วง 00:00–06:59 น. และเวลาช้าไป 7 ชั่วโมง
  - "วันนี้" ในตัวเลือกวันที่ของ content control (`crates/ui-egui/src/content_controls.rs:150` `today`) ก็ใช้ UTC
- ผู้เรียก `now_iso()` อื่นที่ต้องแยกว่าเป็นเวลาที่แสดงผู้ใช้หรือ metadata: `cmd/review.rs`, `cmd/fmt_revisions.rs`, `cmd/moves.rs` (วันที่ของ revision และ comment), `cmd/tools.rs:349` (versions), `cmd/tools.rs:925`, `session.rs:721` (`core.modified`)
- แนวทาง:
  - เพิ่มฟังก์ชันหา offset ของเครื่อง (desktop ใช้ API ของระบบ, wasm ใช้ `Date.getTimezoneOffset`) และใช้กับค่าที่แสดงผู้ใช้
  - metadata ให้ดูก่อนว่า Word เขียน `w:date` แบบไหนแล้วค่อยตัดสินว่าจะเปลี่ยนหรือไม่
  - เพิ่มเทสต์ และบันทึกใน `[Unreleased]` › แก้ไข
- แก้แล้วต้องแก้หมายเหตุเรื่องคำทักทายใน `backstage.png` ในหัวข้อถ่ายภาพหน้าจอตามด้วย

### 2. เอกสารที่ล้าสมัยหลังออก 1.1.1 (แก้แค่เอกสาร ทำในคอมมิตเดียวได้)

**สถานะ 2026-10-25 รอบที่ 7: ปิดแล้ว** ตรวจทุกข้อกับโค้ดแล้ว ส่วนใหญ่แก้ในรอบ 5 (2026-10-24) ที่เหลือแก้ใน `b303ed9`: ตัวเลข 625 คำสั่ง · 1,816 เทสต์ · ~188,000 บรรทัด, ROADMAP Upcoming ข้อ 1 (PR ที่เปิดอยู่มีแค่ Dependabot #1–#4), gap #29, README ทั้งสองภาษา รายการข้างล่างเก็บไว้เป็นประวัติ

- `ROADMAP.md` [ตรวจแล้วที่ :25]
  - :25 แถว Releases ยังเป็น v1.1.0, :5 ยังเป็น "Last updated 2026-10-21" และบันทึกความคืบหน้า (:137) ยังไม่มีแถว 2026-10-22
  - ต้องเพิ่มของ 1.1.1: ปุ่มจัดกระจายและจัดกระจายแบบไทย, ระยะห่างตัวอักขระ 0–1584 pt และการออก Release จากแท็บ Actions (`AGENTS.md:65` กำหนดให้อัปเดต gap, เอกสาร parity และบันทึกความคืบหน้า)
  - :24 ตัวเลขเก่า (433 คำสั่ง, 855 เทสต์, ~88,700 บรรทัด) ส่วน finder นับได้ 607 คำสั่ง, 1,754 เทสต์ และราว 180,900 บรรทัด
  - :17 ตัวเลข parity 484/499 ให้แก้ตามข้อ `parity-checklist.md` ข้างล่าง
  - formatter จะจัดตารางใหม่ทั้งไฟล์ ให้แก้ด้วย `sed` หรือสคริปต์ แล้วตรวจ `git diff --stat` (ดูข้อควรระวังของ session บนคลาวด์)
- `docs/releasing.md` [ตรวจแล้วที่ :7 และ :19]
  - ตัวอย่าง `cargo xtask version set 1.0.1` และ `git tag v1.0.0` ล้าสมัย ให้ใช้ x.y.z และ `git tag -a`
  - เพิ่มขั้นแก้ลิงก์ `[Unreleased]` เป็น `compare/vx.y.z...HEAD` และขั้นรัน audit ที่ติด `#[ignore]`
  - :14-15 บอกว่า `sign.ps1` ลงนามเมื่อมี secret ซึ่งไม่จริงใน CI (ดูข้อ 3)
- `docs/parity-checklist.md`
  - ตอน 1.1.1 แก้ตัวเลขด้วยมือเป็น 484/499 แต่ catalog มี 500 รายการ (รวม `File|Options|Numerals|view.numeral` ที่ `crates/engine/src/catalog.rs:22`) จึงควรได้ราว 485/500 และ File 22/22
  - ให้รัน `cargo xtask parity` (ต้อง build) แล้วแก้ `ROADMAP.md:17` ตาม
- เอกสารที่บอกว่าตัวตรวจภาษามีแค่อังกฤษกับโปแลนด์ ทั้งที่ภาษาไทยออกแล้วใน 1.1.0: `docs/gaps.md:23` (#8), `docs/target-app-parity.md:69,198`, `docs/architecture.md:20`, `docs/localization-parity.md:32,73`
- จำนวนภาษา UI และขนาดแค็ตตาล็อก: จริงมี 19 ภาษารวมไทย (`crates/ui-egui/src/i18n/mod.rs`) แต่เอกสารยังเขียนผิด
  - `docs/localization-parity.md:22` บอก 18 ภาษาและไม่มีไทย, :61 ไม่นับไทย, :76 บอกว่าไทยมี 2,433 รายการ (จริง 2,456)
  - `docs/target-app-parity.md:198` บอก 13 ภาษา, `docs/roadmap.md:63` บอก 15 ภาษา
  - `docs/gaps.md:26` (#11) ยังนับ fr ว่าขาดทั้งที่มี `fr.tsv` แล้ว และตัวเลขของ tr กับ hu ใน `ROADMAP.md:114-116` เก่า
- `docs/gaps.md` ขัดกับโค้ด
  - #9 (:24) ไฟล์เข้ารหัส: มี `crates/docx/src/crypt.rs` แล้ว เหลือแค่ .doc แบบ RC4/XOR
  - #15 (:30) แหล่งอ้างอิง: อ่านและเขียน `b:Sources` ได้แล้ว
  - #17 (:32) `para.asianTypography` อยู่ใน catalog แล้ว
  - #10 (:25) group มีแล้ว และตัวเลข Draw กับ Shape Format ต้องแก้
  - #2 (:17) รองรับ wpc แล้ว เหลือแค่ Word ink (`w14:contentPart`)
  - :54-56 ยังบอกว่ามี PR เปิดอยู่
  - `docs/file-format-parity.md:36` (เข้ารหัส ❌) ขัดกับ :127-128 ของไฟล์เดียวกัน
- เอกสารที่ยังนับว่าฟีเจอร์ที่มีแล้วยังขาด: `ROADMAP.md:44-47` (Why not beta) และ :128-131 (Upcoming), `docs/roadmap.md:21-24` (Current focus), milestone M1–M12, alpha gate แถว 35/36/38 และ `docs/target-app-parity.md:111`
  - ฟีเจอร์เหล่านั้นคือ: เก็บ chart/SmartArt/OLE ไว้เขียนกลับ, footnote ต่อหน้า, column balancing, content control, move tracking, แหล่งข้อมูล Excel, grouping และการเข้ารหัส
- `docs/target-app-parity.md:60` บอกว่า kerning/ligature ในกล่อง Font ยังขาด (ออกแล้วใน 1.1.0) ที่ขาดจริงเหลือแค่ stylistic set, number form และ number spacing
- ข้อความที่ติดมาจาก WordCraft
  - `ROADMAP.md:42-44,64` บอกว่ามี build ที่ลงนามแล้วทุกแพลตฟอร์มและเว็บ
  - `docs/roadmap.md:62` M14 บอกว่าออก release v0.1.0–v0.4.0
  - `docs/architecture.md:59-60` บอกว่ามี CI บน PR และ branch `release`
  - `packaging/windows/package.ps1:161-162` อ้างถึง `windows-arm64.yml` ที่ไม่มีอยู่จริง
  - `packaging/env.sh:57` และคอมเมนต์เรื่อง CI ใน `packaging/linux/` และ `packaging/macos/`
- เวอร์ชันและวันที่ที่ล้าสมัย
  - `docs/roadmap.md:3,34` (422 คำสั่ง) และ `docs/gaps.md:3`
  - `docs/target-app-parity.md:3,12` (v0.4.0 และคอมมิต `a99dddc` ที่ไม่มีใน repo นี้)
  - `docs/localization-hu.md:3` และ `docs/localization-tr.md:3` เขียน "Target: Doc4Me 0.5.0" ซึ่งเป็นรุ่นที่ไม่เคยมี
- `docs/gaps.md` ไม่มีแถวของภาษาไทย (ไวยากรณ์, การตัดบรรทัดเทียบกับ Word, ligature ทางเลือก, ค่า IT๙ ที่อนุมาน) และ `docs/localization-parity.md:83` เขียนเกินจริงว่า IT๙ ตัดบรรทัดเหมือนฟอนต์จริง
- `README.th.md:13` หัวข้อ "ภาษาไทยใน Doc4Me 1.1.0" ยังไม่พูดถึงปุ่มจัดกระจายแบบไทยของ 1.1.1

### 3. CI การตรวจสอบ และ dependency

**สถานะ 2026-10-25 รอบที่ 7**: xtask USAGE และ checksum ของ PngSuite แก้แล้ว (รอบ 5), `.github/dependabot.yml` มีแล้ว (PR #1–#4 รอเจ้าของ repo), `cargo audit` รันซ้ำหลังเพิ่ม whisper-rs/cpal: ไม่มีช่องโหว่ เหลือคำเตือน unmaintained 5 รายการ (`rustybuzz`, `ttf-parser` ผ่าน krilla; `zbus_names`, `zvariant`, `zvariant_derive` ผ่าน zbus/accesskit บน Linux), ผล `cargo xtask ci` ทั้ง workspace ดูหัวข้อรอบที่ 7 ด้านบน; job CI บน ubuntu และการลงนามยังรอเจ้าของ repo

- CI บน GitHub มีแค่ `windows-x64-installer.yml` [ตรวจแล้ว]
  - workflow นี้ build `doc4me` กับ `doc4me-cli` แบบ release, แพ็ก MSI และ zip แล้วรัน `doc4me-cli --version` เท่านั้น
  - ไม่รัน fmt, clippy, เทสต์, assets, layers หรือ wasm และไม่มี trigger `pull_request` "CI ผ่าน" จึงหมายถึงแค่ build installer ผ่าน
  - ข้อเสนอ: เพิ่ม job บน ubuntu-latest รัน `cargo xtask ci` (หรือ fmt + clippy + `cargo test --workspace`) ซึ่งต้องใช้ดิสก์ราว 25 GB และควรตั้ง `CARGO_INCREMENTAL=0` ให้เจ้าของ repo ตัดสินใจก่อน เพราะเพิ่มนาที Actions
- `cargo xtask ci` ทั้ง workspace ไม่ได้รันมาตั้งแต่ 1.0.0 (1.1.0 รันทีละขั้น 1.1.1 รันแค่ engine กับ ui-egui) และยังไม่ได้รัน `cargo xtask parity` กับ 1.1.1 เพราะดิสก์ของ container บนคลาวด์ไม่พอ
- audit `untranslated_interface_text` ติด `#[ignore]` (`crates/ui-egui/src/i18n/audit.rs:166`) [ตรวจแล้ว] ยังไม่รู้ว่าทำไมเคยล้ม 1 ใน ~46 รอบ
  - finder นับ `#[test]` ใน ui-egui ได้ 324 ตัว แต่ CHANGELOG รายงาน 322 ผ่าน และ 1 ignored ต่างกัน 1 ตัว ยังไม่ได้ตรวจ
- action ที่ pin SHA ไว้ยังเป็น Node 20 (GitHub บังคับรันบน Node 24 และเตือนใน log)
  - ได้แก่ `actions/checkout@11d5960…` (v4.4.0), `actions/upload-artifact@ea165f8…` (v4.6.2) และ `actions/download-artifact@d3f86a1…` (v4.3.0)
  - ให้อัปเป็นรุ่นที่ใช้ Node 24 ตรวจ SHA กับ tag ของ action และเพิ่ม `.github/dependabot.yml`
  - แก้ workflow ได้เฉพาะตอนที่ไม่มี release รันอยู่
- `cargo audit` รันครั้งล่าสุดตอน 1.0.0 ส่วน `rustybuzz 0.20.1` และ `ttf-parser 0.25.1` (มาผ่าน krilla) ยังเป็น crate ที่ไม่มีคนดูแลใน `Cargo.lock` ให้รันก่อนออกรุ่นถัดไป หรือเพิ่ม cargo-deny ใน CI
- ลงนามโค้ด [ตรวจแล้ว]: workflow ไม่มี `secrets.*` เลย (คำว่า secrets มีแค่ในคอมเมนต์ที่ :62)
  - `sign.ps1` จึงไม่เห็น `WINDOWS_CERTIFICATE` หรือ `AZURE_*` ใน CI แม้เจ้าของ repo จะใส่ secret ไว้แล้ว
  - ผู้ใช้ตัดสินใจแล้วว่าจะลงนาม ดูขั้นตอนในข้อ 5.2
- `xtask/src/main.rs`: ข้อความ USAGE (:16-30) ไม่มีคำสั่ง `version` และ `corpus --download` โหลด PngSuite ผ่าน `http://` โดยไม่ตรวจ checksum (:202, :226-231)

### 4. ส่วนที่ยังไม่ได้สำรวจ (agent ล้มก่อนทำ)

**สถานะ 2026-10-25 รอบที่ 7**: เลขเวอร์ชันใน `packaging/` และ `apps/` มาจาก workspace ทั้งหมด (ไม่มีเลขตายตัว); ข้อความ "isn't available/supported" ที่ผู้ใช้เจอได้เป็นข้อจำกัดที่ตั้งใจ (เช่นเว็บ, รหัสผ่านชนิดที่ไม่รองรับ) ไม่ใช่ stub; คีย์แปลที่ขาดมีเทสต์ i18n คุม; ภาพใน `docs/images` ถ่ายใหม่ในรอบที่ 7

- โค้ดและเทสต์
  - stub, `unimplemented!` หรือ `todo!` ที่ผู้ใช้เรียกถึงได้
  - คำสั่งใน catalog ที่ไม่มี handler หรือ UI
  - คีย์แปลที่ขาดใน `crates/ui-egui/src/i18n/*.tsv` ทั้ง 18 ไฟล์
  - เลขเวอร์ชันใน `packaging/` และ `apps/` ตรงกับ 1.1.1 หรือไม่
- ตรวจความครบ: ข้อความใน `README.md` และ `README.th.md`, `AGENTS.md`, ความใหม่ของภาพใน `docs/images`, `apps/doc4me-web`, `docs/mcp.md`, `ATTRIBUTION.md` และ `NOTICE`, `contributors/`
- ผล grep ตอนเริ่ม session: ไม่พบ `TODO`, `FIXME`, `todo!(` หรือ `unimplemented!(` ใน `crates`, `apps` และ `xtask` และพบ `#[ignore]` จุดเดียวคือ audit ในข้อ 3

### 5. งานที่ผู้ใช้ตัดสินใจแล้ว ให้ลงมือได้เลย (สั่ง 2026-10-23)

ผู้ใช้สั่งว่า "tag / release v1.0.0, ใบรับรองลงนามโค้ด, การตั้งค่าป้องกัน main, สิทธิ์ push tag ใน .claude/settings.json, การลบ branch เก่า, จะเทียบกับ Word for Windows ด้วยเลยไม่ต้องรอการตัดสินใจ"

- บางขั้น session บนคลาวด์ทำเองไม่ได้ เพราะ push tag ไม่ได้ (git proxy ตอบ 403) และไม่มีสิทธิ์ admin ของ repo
- ขั้นเหล่านั้นติดป้าย **[เจ้าของ repo]** ให้แจ้งผู้ใช้ตั้งแต่ต้น session พร้อมคำสั่งหรือเมนูที่ต้องกด

#### 5.1 tag และ Release `v1.0.0` [ตรวจแล้ว]

- ใช้คอมมิต `7946cbb57a0f4618711e765a844f46194150ec1f` (2026-10-15, [run 38742771852](https://github.com/chatraweechai/doc4me/actions/runs/38742771852) ผ่าน)
  - `Cargo.toml` ของคอมมิตนี้เป็น 1.0.0 และ CHANGELOG ของคอมมิตนี้มีหัวข้อ `## [1.0.0] - 2026-10-15`
  - โค้ดเหมือน `ca0e607` ([run 38734047702](https://github.com/chatraweechai/doc4me/actions/runs/38734047702)) ซึ่ง `[1.0.0]` › ตรวจสอบ อ้างว่าเป็น build ของ 1.0.0
  - `git diff --stat ca0e607 7946cbb` ต่างกันแค่ workflow, `CHANGELOG.md`, `CLAUDE.md`, `AGENTS.md` และ `docs/releasing.md`
  - คอมมิตถัดไปคือ `1d05c27` (run 8, 2026-10-19) เริ่มมีฟีเจอร์ของ 1.1.0 จึงใช้ไม่ได้
  - ทั้งสองคอมมิตไม่อยู่ในประวัติของ `main` เพราะประวัติปัจจุบันเริ่มที่ `a4ae748` (2026-10-20) แต่ยังดึงจาก GitHub ด้วย SHA เต็มได้ (ลองแล้ว 2026-10-23)
- **[เจ้าของ repo]** รันบนเครื่องตัวเอง (workflow_dispatch `release: true` ติด tag ได้แค่ HEAD ของ `main` ตามเวอร์ชันใน `Cargo.toml` จึงใช้กับ 1.0.0 ไม่ได้):
  - `git fetch origin 7946cbb57a0f4618711e765a844f46194150ec1f`
  - `git tag -a v1.0.0 7946cbb57a0f4618711e765a844f46194150ec1f -m "Doc4Me 1.0.0"`
  - `git push origin v1.0.0`
  - ถ้า push ถูกปฏิเสธเรื่อง `workflow` scope ให้ใช้ token ที่มีสิทธิ์นี้ เพราะไฟล์ workflow ของคอมมิตนั้นต่างจาก `main`
- การ push tag จะรัน workflow ของคอมมิตนั้น (มี trigger `tags: ["v*"]` และ job `release`): build ใหม่จากซอร์ส 1.0.0 แล้วสร้าง Release โดยใช้หัวข้อ `[1.0.0]` เป็นบันทึก
  - job `release` อาจได้ 403 แบบรอบแรกของ 1.1.1 เพราะไฟล์ workflow ของคอมมิตต่างจาก default branch ยังไม่รู้ว่าเงื่อนไขนี้ใช้กับ tag ที่มีอยู่แล้วด้วยหรือไม่
  - ถ้า job ใดล้ม **[เจ้าของ repo]** ให้สร้าง Release เองที่ Releases › Draft a new release
    - เลือก tag `v1.0.0` และใช้หัวข้อ `[1.0.0]` ของ `7946cbb` เป็นบันทึก
    - แนบ MSI และ zip จาก artifact `doc4me-windows-x64` ของ run 38742771852 (69,888,334 ไบต์ หมดอายุ 2027-01-13)
    - ไม่ต้องติ๊ก "Set as the latest release"
- หลังออก:
  - ตรวจว่าป้าย Latest ยังอยู่ที่ v1.1.1 ถ้าย้ายไป v1.0.0 ให้แก้ Release v1.1.1 กลับเป็น latest
    - เหตุผล: `gh release create` ของคอมมิตนั้นไม่ได้ใส่ `--latest=false` และค่าเริ่มต้นของ GitHub อาจตั้ง Release ที่เผยแพร่ใหม่เป็น latest (ยังไม่ได้ตรวจ)
  - ตรวจไฟล์เหมือนตอน 1.1.1: sha256 ตรงกับ digest และ `doc4me.exe` กับ MSI เป็น 1.0.0
  - session บันทึกใน `CHANGELOG.md` `[Unreleased]` › แก้ไข ว่าลิงก์ `[1.0.0]` ใช้ได้แล้ว ห้ามแก้หัวข้อ `[1.0.0]` และ `[1.1.0]`
  - ลิงก์ `[1.1.0]` ให้ชี้หน้า Release ต่อไป เพราะ `v1.0.0` กับ `v1.1.0` อยู่คนละประวัติ หน้าเปรียบเทียบจึงใช้ไม่ได้

#### 5.2 ลงนามโค้ด

- ส่วนของ session (ทำได้เลย แต่ห้ามทำขณะ job `release` รันอยู่):
  - เพิ่ม `env:` ใน step "Build MSI + portable zip" ของ `.github/workflows/windows-x64-installer.yml` ให้รับค่าจาก `secrets.*`
    - ชื่อตาม `packaging/windows/sign.ps1:7-10`: `WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD`, `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_SIGNING_ENDPOINT`, `AZURE_SIGNING_ACCOUNT`, `AZURE_CERT_PROFILE`
  - secret ที่ยังไม่ได้ใส่จะได้ค่าว่าง `sign.ps1:48` ใช้ `[bool]$env:WINDOWS_CERTIFICATE` จึงยังข้ามการลงนามพร้อมคำเตือนเหมือนเดิม [ตรวจแล้ว] (ให้ตรวจการเช็ก `AZURE_*` ที่ :49-53 ด้วย)
  - แก้ `docs/releasing.md:14-15` ให้บอกชื่อ secret และวิธีตั้ง แล้วบันทึก `[Unreleased]`
- **[เจ้าของ repo]** สมัครบริการลงนาม แล้วใส่ secret ที่ Settings › Secrets and variables › Actions
  - ตามข้อกำหนดของ CA/Browser Forum ใบรับรอง code signing ที่ออกตั้งแต่ 2023-06-01 ต้องเก็บกุญแจในฮาร์ดแวร์ (token หรือ HSM) ส่วนใหญ่จึงส่งออกเป็น .pfx มาใส่ `WINDOWS_CERTIFICATE` ไม่ได้
  - ให้ถามผู้ขายว่ามีการลงนามบนคลาวด์ที่ใช้ใน CI ได้หรือไม่ หรือใช้ Azure Trusted Signing ซึ่ง `sign.ps1` รองรับอยู่แล้ว
  - ตรวจเงื่อนไขประเทศและประเภทผู้สมัครของ Azure Trusted Signing ก่อนสมัคร (session นี้ยังไม่ได้ตรวจ)
- หลังใส่ secret ให้สั่ง run บน `main` (ไม่ต้องติ๊ก `release`) แล้วตรวจว่า log ไม่มีคำเตือน "leaving unsigned" และ `doc4me.exe`, `doc4me-cli.exe` กับ MSI มีลายเซ็น

#### 5.3 ป้องกัน `main`

- **[เจ้าของ repo]** ตั้งที่ Settings › Rules › Rulesets › New ruleset › New branch ruleset (session ไม่มีสิทธิ์ admin และไม่มีเครื่องมือตั้ง ruleset)
  - Enforcement status: Active และ Target branches: Include default branch
  - เปิด "Restrict deletions" และ "Block force pushes"
  - อย่าเปิด "Require a pull request before merging" เพราะ session push ตรงไป `main` ตาม `CLAUDE.md`
  - อย่าเปิด "Require status checks to pass" จนกว่าจะมี job ทดสอบ (ข้อ 3) และอย่าเปิด "Restrict updates"
- ถ้าจะกัน tag ด้วย ให้สร้าง tag ruleset สำหรับ `v*` เปิด "Restrict updates" และ "Restrict deletions"
  - อย่าเปิด "Restrict creations" เพราะ job `release` ใช้ `GITHUB_TOKEN` สร้าง tag ของรุ่นใหม่
- immutable releases ยังไม่ต้องเปิด เพราะจะแก้บันทึกหรือไฟล์ของ Release หลังเผยแพร่ไม่ได้ (เคยต้องเพิ่มหมายเหตุท้าย v1.1.1) ถ้าจะเปิดให้รอออก v1.0.0 เสร็จก่อน
- หลังตั้ง session ตรวจแค่ว่า push ปกติไป `main` ยังผ่าน (ห้ามทดสอบด้วย force-push) แล้วบันทึกใน handoff

#### 5.4 ถอนสิทธิ์ push tag ใน `.claude/settings.json` [ตรวจแล้ว]

- ไฟล์นี้มีแค่ `permissions.allow` สองรายการ คือ `Bash(git tag -a v*)` และ `Bash(git push origin v*)`
- session ลบสองรายการนี้ ถ้าไฟล์ไม่เหลือค่าอื่นให้ลบทั้งไฟล์ แล้วบันทึก `[Unreleased]` › ความปลอดภัย
- เหตุผล:
  - ตอนนี้ออกรุ่นผ่าน workflow_dispatch และ `v1.0.0` เจ้าของ repo push เอง session บนคลาวด์ก็ push tag ไม่ได้อยู่แล้ว
  - การอนุญาตแบบไม่ถามจึงไม่ได้ใช้ แต่เปิดทางให้ push tag สาธารณะ (ซึ่งสร้าง Release) โดยไม่มีการยืนยัน
- `.gitignore:9` (`/.claude/`) ไม่กระทบการแก้ไฟล์นี้ เพราะไฟล์ถูก track อยู่แล้ว

#### 5.5 ลบ branch เก่า [ตรวจแล้ว]

- ผล `git ls-remote --heads origin` เมื่อ 2026-10-23: มี `main` และ `claude/optimistic-fermat-z9uknk` (`be5893e` คือคอมมิตของ tag `v1.1.1` merge เข้า `main` แล้ว) ส่วน `claude/modest-maxwell-3q1mya` ไม่มีบน remote แล้ว
- session ตรวจทุก branch ด้วย `git merge-base --is-ancestor origin/<branch> origin/main` ก่อน แล้วรัน `git push origin --delete <branch>`
  - ถ้า proxy ตอบ 403 **[เจ้าของ repo]** ให้ลบที่หน้า Branches ของ GitHub
- ห้ามลบ branch ที่ยัง merge ไม่หมด หรือ branch ของ session ที่ยังทำงานอยู่ และ session ใหม่อาจสร้าง branch `claude/*` เพิ่ม จึงต้องตรวจใหม่ทุกครั้ง

#### 5.6 ใช้ Word for Windows เป็นเป้าหมาย parity

- เปลี่ยนหัวเอกสาร "Target: Microsoft Word (Microsoft 365) for Mac 16.113.4" ใน 11 ไฟล์ [ตรวจแล้ว]
  - ไฟล์: `ROADMAP.md`, `docs/architecture.md`, `docs/file-format-parity.md`, `docs/gaps.md`, `docs/hardware-parity.md`, `docs/layout-parity.md`, `docs/localization-parity.md`, `docs/roadmap.md`, `docs/target-app-parity.md`, `docs/typing-parity.md`, `docs/ui-parity.md`
  - ถามผู้ใช้ว่าเทียบกับ Word for Windows รุ่นและ build ไหน (ดูได้ที่ File › Account › About Word) ถ้ายังไม่รู้ให้เขียน "Microsoft Word (Microsoft 365) for Windows" โดยไม่ใส่เลขรุ่น
- ตรวจแถวที่อิงพฤติกรรมของ Mac ใหม่:
  - ทางลัด Cmd เทียบกับ Ctrl (`docs/typing-parity.md`)
  - เมนูของ Mac เทียบกับ ribbon และ backstage ของ Windows (`docs/ui-parity.md`, `docs/target-app-parity.md`)
  - `docs/hardware-parity.md`
  - ระยะบรรทัดทำตาม Windows อยู่แล้ว (`docs/layout-parity.md:24`)
- alpha gate ใน `docs/roadmap.md:28-29` ระบุ "main platform (macOS desktop)" ให้เปลี่ยนเป็น Windows
  - การรัน alpha gate ใหม่ต้องใช้เครื่อง Windows ซึ่ง session บนคลาวด์ทำไม่ได้ ให้ระบุว่ายังไม่ได้วัดใหม่
  - ตัวเลข readiness (~60%/55%/63%) ยังเป็นค่าที่วัดใน WordCraft
- ทำพร้อมข้อ 2 ในคอมมิตเดียวได้ และบันทึก `[Unreleased]` › เปลี่ยนแปลง

#### 5.7 ยังรอเจ้าของ repo ตัดสินใจ

**สถานะ 2026-10-25 รอบที่ 8: ผู้ใช้ตัดสินแล้วทุกข้อ** (ดู "การตัดสินใจถาวร" ในหัวข้อสถานะปัจจุบัน) ฟอนต์แทน Aptos คือ Intos ที่ฝังแล้ว

- `.gitignore:9` มี `/.claude/` ไฟล์ใหม่ใต้ `.claude/` จึงถูกละเว้น, `package-lock.json` เป็นไฟล์ว่าง (`fc7bab5`) และคอมมิต `9954ae0` กับ `fc7bab5` ไม่มีรายการใน CHANGELOG
- ฟอนต์แทน Aptos (gap #4 ซึ่งขวาง beta), ecosystem (#22), ผู้ให้บริการ AI (#25) และโมเดลเสียง (#28)
- จะเพิ่ม job ทดสอบบน Linux ใน CI หรือไม่ (ข้อ 3)
- ข้อที่ผู้ใช้สั่งให้ข้ามเมื่อ 2026-10-21: ให้นักแปลเจ้าของภาษาตรวจ UI ภาษาไทย และวัดค่า TH SarabunIT๙ จากไฟล์ฟอนต์จริง

### 6. ก่อนออกรุ่นถัดไป

- ตอนนี้ `[Unreleased]` มีแต่เรื่องเอกสาร ยังไม่มีเหตุให้ออกรุ่น
- รายการ `[Unreleased]` › แก้ไข (ข้อความเรื่อง push tag ที่ล้าสมัยใน `[1.1.1]`) ต้องย้ายไปอยู่ในหัวข้อของรุ่นถัดไปด้วย เพราะหมายเหตุบนหน้า Release v1.1.1 ชี้ไปที่ "หมวด แก้ไข ของรุ่นถัดไป"
- ขั้นตอน:
  1. `cargo xtask version set x.y.z` แล้วย้าย `[Unreleased]` เป็น `## [x.y.z] - YYYY-MM-DD`
  2. เพิ่มลิงก์ `[x.y.z]: …/compare/v1.1.1...vx.y.z` และแก้ `[Unreleased]: …/compare/vx.y.z...HEAD`
  3. รัน `cargo test -p doc4me-ui-egui i18n -- --include-ignored` และ `cargo audit`
  4. push ไป `main` แล้วสั่ง workflow บน `main` พร้อม `release: true`
  5. ห้าม push จนกว่า job `release` จะจบ

### 7. Backlog ระยะยาว (สรุป ไม่ใช่งานค้าง)

- งานจาก `docs/gaps.md`
  - คลังไฟล์ .docx จริงสำหรับทดสอบ (#1) และการเก็บ Word ink (`w14:contentPart`)
  - ความเสถียรและ autosave (#3)
  - ความตรงของการแบ่งหน้า และฟอนต์แทน Aptos (#4)
  - การพิมพ์แบบ native แทนการเปิด PDF (#6) และความลึกของกล่องโต้ตอบ (#7)
  - ตัวตรวจภาษาเพิ่ม (#8) และการแก้ไขแผนภูมิของ Word (#13)
  - การเขียน .doc และการเปิดไฟล์ Word 6/95 (#14)
  - ภาษา UI hi/ar/id/ko/vi/pt-PT และ RTL (#11)
- ข้อจำกัดภาษาไทยที่ยังเป็นจริง
  - ยังไม่ตรวจไวยากรณ์ไทย
  - การตัดคำ (ICU4X) อาจต่างจาก Word
  - ไม่จำลอง ligature ทางเลือกของ TH Sarabun (fi, ฤๅ)
  - ความกว้าง glyph ของ Sarabun ที่ปรับขนาดต่างจากต้นฉบับเล็กน้อย
- คำสั่งใน catalog ที่ยังไม่มี 14 รายการ: Dictate, Online Pictures, 3D Models, Screenshot, Get Add-ins, Online Video, Ink Equation, Ink to Math, Translate, Block Authors และคำสั่ง View › Window อีก 4 คำสั่ง
- แพ็กเกจของแพลตฟอร์มอื่นนอกจาก Windows x64

## สถานะ: ออก 1.1.1 แล้ว

- [Doc4Me v1.1.1](https://github.com/chatraweechai/doc4me/releases/tag/v1.1.1) ออก 2026-10-22 13:51 UTC จาก [run 39788628618](https://github.com/chatraweechai/doc4me/actions/runs/39788628618) (แท็บ Actions, `release: true`)
  - tag `v1.1.1` ชี้ `be5893e` มี `doc4me-1.1.1-windows-x64.msi` (32,993,280 ไบต์) และ `doc4me-1.1.1-windows-x64-portable.zip` (43,584,012 ไบต์)
  - ตรวจหลังออกแล้ว: sha256 ตรงกับ digest, `doc4me.exe` และ MSI เป็น 1.1.1, `UpgradeCode` เดียวกับ 1.1.0, บันทึกบนหน้า Release ตรงกับ `[1.1.1]` ทุกไบต์
- งานใหม่ทั้งหมดลง `## [Unreleased]` ห้ามแก้หัวข้อ `[1.1.1]`
  - ข้อความ "เจ้าของ repo ต้อง push tag เอง" ใน `[1.1.1]` (และบนหน้า Release) ล้าสมัยแล้ว บันทึกแก้ไว้ใน `[Unreleased]` › แก้ไข
  - หมายเหตุท้ายบันทึกบนหน้า Release v1.1.1 เจ้าของ repo เพิ่มแล้ว (14:32 UTC) และบรรทัด "ผู้ใช้:" ของ `[Unreleased]` › แก้ไข ใน CHANGELOG แก้ตามแล้ว ไม่มีงานค้าง
- ออกรุ่นถัดไป (`docs/releasing.md` ข้อ 3–4):
  - `cargo xtask version set x.y.z` แล้วย้าย `[Unreleased]` เป็น `## [x.y.z] - YYYY-MM-DD` และเพิ่มลิงก์เปรียบเทียบท้าย `CHANGELOG.md` แล้ว push ไป `main`
  - สั่ง workflow `windows-x64-installer.yml` บน `main` พร้อม `release: true`
  - ห้าม push ไป `main` จนกว่า job `release` จะจบ โดยเฉพาะการแก้ `.github/workflows` (เป็นเหตุให้รอบแรกของ 1.1.1 ล้มด้วย 403)

### session 2026-10-22 15:15 – 2026-10-23 04:30 UTC: สำรวจงานค้าง (ยกเลิกกลางทาง)

- ผู้ใช้สั่งตามลำดับ: "ตรวจสอบงานค้างและงานเบื้องหลังทั้งหมด", "ยกเลิกงานทั้งหมด บันทึก handoff สำหรับเริ่ม session ใหม่" และ "บันทึก handoff สิ่งที่ต้องทำต่อใน session ใหม่ทั้งหมด"
- งานเบื้องหลัง (ตรวจ 2026-10-22 15:13 UTC): ไม่มี scheduled task, task list, PR หรือ issue ที่เปิด และไม่มี run ค้าง
  - branch `claude/*` บน remote ถูก merge เข้า `main` หมดแล้ว
  - session อื่นในบัญชี "รวมโค้ด doc-scanner-pdf" เป็นคนละ repo ตามสรุปของ session นั้น รอเจ้าของใส่ secret `SITE_DEPLOY_KEY` และ publish เว็บเอง
- สั่ง workflow ให้ agent อ่านอย่างเดียว 4 ด้าน (handoff/CHANGELOG, roadmap/gaps, โค้ด/เทสต์, CI/infra) แต่ชนขีดจำกัดของ session
  - finder ด้านโค้ด/เทสต์, ตัวตรวจซ้ำทั้ง 4 ตัว และตัวตรวจความครบล้มหมด
  - ผลที่ได้ย้ายไปไว้ในหัวข้อ "งานที่ต้องทำต่อใน session ใหม่" ข้างบน และตรวจข้อสำคัญกับโค้ดแล้ว (2026-10-23)
- แก้แค่ `handoff.md` (คอมมิต `92e1929`, `30e8da4` และคอมมิตล่าสุด ใส่ `[skip ci]` ทุกคอมมิต) จึงไม่ได้แก้ `CHANGELOG.md`
  - `30e8da4` ทำหมายเหตุไว้ที่ข้อมูลเก่าในหัวข้อประวัติของ handoff
- 2026-10-23 ผู้ใช้ตัดสินใจ 6 เรื่องที่เคยรอเจ้าของ repo (v1.0.0, ลงนามโค้ด, ป้องกัน `main`, สิทธิ์ push tag, ลบ branch เก่า, เทียบกับ Word for Windows)
  - เขียนข้อ 5 ใหม่เป็นขั้นตอนที่ลงมือได้
  - ตรวจข้อเท็จจริงที่ใช้วางขั้นตอนแล้ว: คอมมิตและ artifact ของ 1.0.0 (ผ่าน GitHub API และ `git fetch` ด้วย SHA), workflow ของ `7946cbb`, `.claude/settings.json`, การอ่าน secret ของ `sign.ps1`, branch บน remote และไฟล์ที่ยังตั้งเป้าเป็น Word for Mac
  - ยังไม่ได้ลงมือทำงานใดในข้อ 5

### session 2026-10-22 14:44–14:58 UTC: ตรวจ CI ของ `main` (ยุติแล้ว)

- ผู้ใช้สั่ง "ดำเนินการต่อจากที่ค้างให้เสร็จ และตรวจว่า run ล่าสุดของ main ผ่าน" งานค้างมีข้อเดียวคือตรวจ CI ไม่มีงานโค้ด จึงไม่แก้ `CHANGELOG.md`
- [run 39797850213](https://github.com/chatraweechai/doc4me/actions/runs/39797850213) ของ `73399d7` (HEAD ของ `main` ตอนตรวจ) ผ่าน: run สรุปเป็น `success`; ทุกขั้นของ job `build` ผ่าน (Build MSI + portable zip 14:42–14:54 UTC) และ job `release` ข้ามตามปกติของ push
- run ของ `8f72a1b` (39797301324) และ `30e7503` (39797813816) ถูกยกเลิกโดย concurrency (`cancel-in-progress`) เพราะมี push ใหม่กว่า ไม่ใช่การล้ม และ `73399d7` มีไฟล์ของทั้งสองคอมมิตครบ
- คอมมิตนี้แก้แค่ `handoff.md` จึงใส่ `[skip ci]` ไม่ให้เกิด run ใหม่ run ล่าสุดของ `main` ยังเป็น 39797850213 ที่ผ่านแล้ว
- ทำงานและ push ที่ `main` ตามคำสั่งผู้ใช้ ไม่ได้ push สาขา `claude/modest-maxwell-3q1mya`

### session 2026-10-22 14:39–14:50 UTC: ปิดงานหมายเหตุบนหน้า Release v1.1.1 (ยุติแล้ว)

- ผู้ใช้สั่ง "ดำเนินการต่อจากที่ค้างให้เสร็จ หากไม่มีงานค้างให้บันทึก changelog และ handoff แล้วยุติ"
- `get_release_by_tag v1.1.1`: เจ้าของ repo ต่อหมายเหตุหลังออกรุ่นท้ายบันทึกแล้ว (`updated_at` 14:32:00Z)
  - ดึง body จาก API ตัด CR แล้ว `diff` กับหัวข้อ `[1.1.1]` ของ `be5893e` ต่างกันแค่บรรทัดว่างแรกกับหมายเหตุ เนื้อหาเดิมไม่เปลี่ยน
- เจ้าของ repo push `8f72a1b` (14:37 UTC) วางข้อความหมายเหตุต่อท้ายบรรทัด "ผู้ใช้:" ใน CHANGELOG
  - บรรทัดนั้นจึงขัดกันเอง (ยังบอกว่าหน้า Release แสดงข้อความเดิม) session นี้แยกเป็นรายการย่อยว่าหน้า Release มีหมายเหตุแล้ว พร้อมบรรทัดตรวจสอบ
- CI ของ `main`: run ของ `ffffa5f` (39794752146) ผ่าน; run ของ `8f72a1b` และ `30e7503` ยังรอคิวตอนยุติ (แก้แค่เอกสาร) ตรวจแล้วใน session ถัดมา (ดูข้างบน)
- ทำงานและ push ที่ `main` ตามคำสั่งผู้ใช้ ไม่ได้ push สาขา `claude/clever-hopper-tw74wd`

### session 2026-10-22 13:35–14:22 UTC: ออก Release v1.1.1 (ยุติแล้ว)

- ตอนยุติ: working tree สะอาด `main` ตรงกับ `origin/main` ไม่มีงานตั้งเวลาค้าง
  - run ของ push `670f22b` ผ่าน ส่วน run ของคอมมิตหลังจากนั้น (แก้แค่เอกสาร) ยังรอคิวอยู่ ให้ session ถัดไปตรวจว่า run ล่าสุดของ `main` ผ่าน

- ผู้ใช้สั่ง "ดำเนินการต่อจากที่ค้างให้เสร็จ" งานที่ค้างมีข้อเดียวคือ Release v1.1.1 (ตอนเริ่ม session ยังไม่มี tag `v1.1.1` บน GitHub)
- สาเหตุที่รอบแรก (run 39783304886) ได้ 403:
  - log ของ job `release` มีแค่ `Contents: write` และ `HTTP 403: Resource not accessible by integration` ตอน `POST /releases` (13:14:20 UTC) ส่วนเวลา push ดูจาก `gh api repos/chatraweechai/doc4me/activity`
  - ก่อนหน้านั้น `main` ถูก push `e06edac` ซึ่งแก้ `.github/workflows/windows-x64-installer.yml` (13:11:46) คอมมิต `858b9bd` ที่จะติด tag จึงมีไฟล์ workflow ต่างจาก default branch และไม่มี branch หรือ tag ชี้อยู่
  - เอกสาร REST ของ GitHub (Create a release) เขียนไว้ว่ากรณีนี้ token ต้องมีสิทธิ์แก้ workflow ซึ่ง `GITHUB_TOKEN` ได้ไม่ได้ (ลิงก์อยู่ใน CHANGELOG `[Unreleased]`)
- 13:38 UTC สั่ง workflow_dispatch `release: true` บน `main` ที่ `be5893e` เป็น run 39788628618 แล้วผ่านทั้ง job `build` และ `release`
  - ระหว่างรัน push `10a276a` (handoff อย่างเดียว) ขึ้น `main` ตอน 13:42:41 เพราะ stop hook เตือนเรื่องคอมมิตที่ยังไม่ push
  - release ยังผ่าน เพราะ `be5893e` ไม่ได้แก้ไฟล์ workflow และไฟล์ workflow ตรงกับ `main` แต่การเทียบสองรอบนี้แยกไม่ได้ว่า GitHub ใช้เงื่อนไขแบบไหน จึงให้งด push ทั้งหมดระหว่าง release
  - run ของ push `10a276a` (39789261472) ผ่าน
- บันทึกผลใน `CHANGELOG.md` `[Unreleased]` และเพิ่มข้อควรระวังใน `docs/releasing.md` (คอมมิต `670f22b`)
- ตรวจหลัง push ด้วย agent: ตรวจไฟล์ใน Release, ตรวจข้อเท็จจริงในเอกสาร และพยายามหักล้างสาเหตุของ 403 แต่ละข้อที่พบมี agent อีกตัวตรวจซ้ำ
  - ไฟล์ใน Release ถูกต้องทุกข้อ
  - เอกสารแก้ 4 เรื่องในคอมมิตถัดมา:
    - ข้อสรุปเกินหลักฐาน: เปลี่ยนไปอ้างเอกสารของ GitHub
    - เวลาที่ใช้เป็นเวลาสร้าง run ไม่ใช่เวลา push
    - CHANGELOG ขาดบรรทัดตรวจสอบและบรรทัดผลต่อผู้ใช้
    - ขั้นออกรุ่นใน handoff ขาดลิงก์เปรียบเทียบ

### ทางเลือกในการออก Release v1.1.1 (บันทึกเดิม ก่อนออกสำเร็จ)

- งานโค้ด ภาพหน้าจอ อภิธานศัพท์ และเอกสารของ 1.1.1 เสร็จและ push ไป `main` แล้ว ไม่มีงานโค้ดค้าง
- ทางที่ใช้ได้: เจ้าของ repo รันบนเครื่องตัวเอง `git fetch origin && git tag -a v1.1.1 origin/main -m "Doc4Me 1.1.1" && git push origin v1.1.1`
- ทางที่ลองแล้วไม่สำเร็จ:
  - session บนคลาวด์ push tag ไม่ได้ (git proxy ตอบ HTTP 403) แม้ `.claude/settings.json` อนุญาต `git tag -a v*` / `git push origin v*` แล้ว
  - ตัวเลือก `release` ของ workflow_dispatch (เพิ่มใน `858b9bd`, แก้ใน `e06edac`): run 39783304886 build ผ่าน แต่ `gh release create v1.1.1 --target` ล้มด้วย `HTTP 403: Resource not accessible by integration` (สร้าง tag ใหม่ด้วย `GITHUB_TOKEN` ไม่ได้)
  - ถ้าจะทำให้ทางนี้ใช้ได้ (ยังไม่ได้ตรวจ): ดู Settings › Actions › General › Workflow permissions ว่าเป็น "Read and write" หรือไม่ และ GitHub อาจไม่ให้ `GITHUB_TOKEN` สร้าง ref ที่ชี้คอมมิตซึ่งแก้ไฟล์ workflow ทางแก้คือใช้ PAT หรือ GitHub App token ที่มีสิทธิ์ `contents` + `workflows` เป็น secret
- หลังออกแล้ว: ตรวจว่า Release มี MSI และ zip และ job `release` ผ่าน งานใหม่ทั้งหมดลง `## [Unreleased]` ห้ามแก้ `[1.1.1]`

### ประวัติของ 1.1.1 (บันทึกเดิม ก่อนออกสำเร็จ: ข้อ "push tag เอง" และ "`[Unreleased]` ว่าง" ไม่จริงแล้ว)

- 2026-10-22 ผู้ใช้สั่ง: เพิ่มการจัดแนว จัดกระจายแบบไทย, ระยะห่างตัวอักขระ ปกติ/บีบ/ขยาย ขนาด 0–1584 pt และเลขเวอร์ชัน 1.1.1 ทำเสร็จและ push ไป `main` แล้ว เทสต์ ui-egui/engine, audit ภาษา, clippy และ fmt ผ่าน (ดู `[1.1.1]` หมวดการตรวจสอบ)
- `Cargo.toml` เป็น 1.1.1 และ CHANGELOG มี `## [1.1.1] - 2026-10-22` แล้ว `[Unreleased]` ว่าง
- ขั้นที่ session บนคลาวด์ทำไม่ได้ (push tag ได้ HTTP 403): `git fetch origin && git tag -a v1.1.1 origin/main -m "Doc4Me 1.1.1" && git push origin v1.1.1` workflow จะสร้าง Release เอง
- 2026-10-22 (รอบสอง): ถ่ายภาพ `hero`, `thai`, `dark`, `dark-th`, `font`, `font-th` ใหม่ และเพิ่มคำ ขยาย/บีบ/ขนาด:/ปกติ ในอภิธานศัพท์แล้ว
- push tag จาก session นี้ถูกปฏิเสธ (นับเป็นการเผยแพร่สาธารณะ) เจ้าของ repo ต้อง push `v1.1.1` เอง

## ประวัติ 1.1.0 (ออกแล้ว 2026-10-21)

- Release [Doc4Me v1.1.0](https://github.com/chatraweechai/doc4me/releases/tag/v1.1.0) ที่ `51a9658` มี MSI และ portable zip
- เจ้าของ repo เพิ่ม `package-lock.json` ว่างที่ราก (`fc7bab5`) ไม่มีผลต่อโปรแกรม

### ทำแล้วใน session 2026-10-21 (ทั้งหมดอยู่ใน `[1.1.0]`)

- ช่องตัวเลข `DragValue` ที่มี `range` ทั้ง 47 ช่องไม่บีบค่าเดิมของเอกสาร (`.clamp_existing_to_range(false)`) และเทสต์ตรวจซอร์ส `ranged_number_fields_keep_existing_values` กันช่องใหม่ที่ลืมใส่
- รางตัวเลื่อนซูมในแถบสถานะ: สี `input_border` และสูง 3 pt (`chrome.rs`) เห็นได้ทั้งสองธีม ปุ่มยังเด่นกว่าราง
- drop cap ใช้ระยะบรรทัดจริงของย่อหน้า (`layout/src/para.rs`) แทน 1.2 em ตายตัว ซึ่งทำให้ drop cap เล็กเกินไปตั้งแต่เปลี่ยนเป็นความสูงบรรทัดแบบ Word
- ภาพตัวอย่าง style set บนแท็บ Design วาดพื้นที่ 150 pt ที่สเกล 0.38 จึงเห็นครบสามย่อหน้า
- `README.th.md` ใช้คำตามอภิธานศัพท์และ `th.tsv` (12 จุด ตรวจด้วย agent สองชั้น)
- ภาพหน้าจอทุกภาพใน `docs/images` ถ่ายใหม่ด้วยโค้ดล่าสุด และสคริปต์ `ui_shot` ทุกภาพเก็บไว้ที่ `docs/images/shots/`
- ผลตรวจล่าสุด: `cargo test` ของ `doc4me-layout` (224), `doc4me-engine` (372 + 13 + 8) และ `doc4me-ui-egui` (320) ผ่าน; audit ภาษา `--include-ignored` ผ่าน 60 รายการ; clippy `-D warnings` ของ layout และ ui-egui และ `cargo fmt --all --check` ผ่าน
  - ยังไม่ได้รัน `cargo xtask ci` ทั้ง workspace เพราะดิสก์ของ container ไม่พอ (ต้องใช้ราว 25 GB); crate อื่นไม่ได้แก้ใน session นี้

### ข้อสังเกตที่ไม่ได้แก้ (ไม่ใช่บั๊กที่ยืนยันแล้ว)

- `Slider` อื่น (`read_aloud.rs`, `options.rs`, กล่อง Zoom ใน `dialogs.rs` และแถบซูมในแถบสถานะ `chrome.rs`) ยังบีบค่าแบบเดิมของ egui ค่ามาจากการตั้งค่าหรือมุมมองซึ่งอยู่ในช่วงเสมอ ถ้าวันหนึ่งผูกกับค่าจากเอกสาร ให้ใช้ `.clamping(egui::SliderClamping::Edits)`
- `backstage.png` มีคำทักทายตามเวลา ถ่ายใหม่หลังเที่ยง UTC จะได้ "Good afternoon" (ต้นเหตุคือคำทักทายใช้เวลา UTC ไม่ใช่เวลาเครื่อง ดูหัวข้องานที่ต้องทำต่อ ข้อ 1)

## ข้อที่ผู้ใช้สั่งให้ข้าม (2026-10-21)

- ข้อ 6 เดิม: ให้นักแปลเจ้าของภาษาตรวจภาษาไทยใน UI
- ข้อ 8 เดิม: วัดค่า TH SarabunIT๙ จากไฟล์ฟอนต์จริง (ตอนนี้เป็นค่าอนุมาน)

## สิ่งที่อยู่ใน 1.1.0

- **ระยะบรรทัดแบบ Word บน Windows**
  - ไฟล์: `crates/fonts/src/word.rs` (`line_metrics`, `pick_line`, `single_line`)
  - ใช้ usWin บวก external leading ทำให้ Sarabun ได้ 1.853 em
  - ฟอนต์ที่ถูกแทนที่ใช้ระยะบรรทัดของฟอนต์ที่เอกสารขอ: Cambria 1.172 em, Courier New 1.133 em ส่วนฟอนต์อื่นใช้ค่า hhea ของตัวแทน
- **TH Sarabun New / PSK / IT๙ ขนาดเท่าต้นฉบับ**
  - ไฟล์: `crates/fonts/src/thsarabun.rs` และ `word::MetricFit`
  - ใช้กับเนื้อความ แผนภูมิ SmartArt และ m:nor ในสมการ (`display.rs` `text_glyphs`, `math.rs` `run_atoms`)
- **Kerning และ ligature แบบ Word**
  - ไฟล์: `crates/layout/src/para.rs` (`run_features`)
  - อ่านและเขียนใน .docx, RTF, ODT และอ่านจาก .doc
  - กล่อง Font แยกเป็นหน้า Font และ Advanced
- **เลขลำดับไทย**: `thaiCounting`, thaiNumbers, thaiLetters ในเมนู Numbering และกล่อง Define New Multilevel List
- **ตัวตรวจตัวสะกดไทย**: `crates/proof/src/th` กับคำที่มักเขียนผิด 105 คู่ (`assets/spelling/th-misspellings.tsv`)
- **ส่วนติดต่อผู้ใช้ภาษาไทย**
  - `crates/ui-egui/src/i18n/th.tsv` มี 2,452 รายการ ครบทุกคีย์ (ตอน 1.1.0; 1.1.1 เพิ่ม 4 คีย์เป็น 2,456)
  - คู่มือรูปแบบและอภิธานศัพท์อยู่ที่ [`docs/localization-th.md`](docs/localization-th.md)
  - เทสต์ audit `untranslated_interface_text` ยังติด `#[ignore]` เพราะเคยล้มครั้งเดียวตอนเครื่องโหลดหนัก ให้รันด้วยมือก่อนออกเวอร์ชัน
    - `cargo test -p doc4me-ui-egui i18n -- --include-ignored` ผ่าน 60 รายการเมื่อ 2026-10-21

## ข้อควรระวังของ session บนคลาวด์ (hook ของ plugin)

- **commit อัตโนมัติ**: ทุกครั้งที่ใช้ Edit/Write (รวมถึง agent ย่อย และไฟล์นอก repo) hook จะรัน `git add <ไฟล์>; git commit -m "Claude checkpoint: …"`
  - ถ้ามีไฟล์ที่ stage ค้างไว้ จะถูก commit ไปด้วย ให้ stage แล้ว commit ในคำสั่ง Bash เดียวกัน
  - ก่อน push ให้รวม commit เหล่านี้: `git reset --soft origin/main` แล้ว commit ใหม่ครั้งเดียว
- **formatter**: จัดตารางในไฟล์ `.md` ใหม่ทั้งไฟล์หลัง Edit/Write (เช่น `ROADMAP.md` เคยเปลี่ยน 439 บรรทัด)
  - แก้ไฟล์ที่มีตารางผ่านสคริปต์หรือ `sed` แล้วตรวจ `git diff --stat` ทุกครั้ง
- **ข้อจำกัดของ Bash**
  - ห้ามรัน Python แบบ inline ให้เขียนสคริปต์เป็นไฟล์ใน scratchpad แล้วรันด้วย `python3 -I`
  - คำสั่งยาวได้ไม่เกิน 500 ตัวอักษร และมีตัวคั่นคำสั่งได้ไม่เกิน 10 ตัว (`|` ในข้อความ Markdown ก็นับด้วย)
  - เขียนไฟล์ใน `.git` ไม่ได้ (เช่น `.git/info/exclude`)
- **`CLAUDE.md` ถูกเติมเอง**: hook ชื่อ context-recovery ต่อบล็อก "Session Recovery" ท้าย `CLAUDE.md`
  - อย่า commit บล็อกนี้ ให้คืนไฟล์ด้วย `git checkout -- CLAUDE.md`
- **stop hook** เตือนเมื่อ working tree ไม่สะอาด
  - ไดเรกทอรี log ของ plugin (`.anchor/`, `.harnesskit/`, `.viberevert/`) อยู่ใน `.gitignore` แล้ว
- **รันเทสต์ผ่าน pipe**: ใช้ `set -o pipefail` เพื่อให้ exit code เป็นของ cargo

## ถ่ายภาพหน้าจอ

- `cargo build -p doc4me-ui-egui --example ui_shot` แล้วรัน `target/debug/examples/ui_shot script.jsonl` (รูปแบบสคริปต์ดูหัวไฟล์ `crates/ui-egui/examples/ui_shot.rs`)
- container ไม่มี GPU ต้องติดตั้ง Vulkan แบบซอฟต์แวร์ก่อน: `apt-get update && apt-get install -y mesa-vulkan-drivers`
  - ถ้าไม่ติดตั้งจะล้มด้วย `No adapter found`
- กล่องฟอนต์: `ui.dialog` (`name: font`) แล้ว `ui.click` ที่แท็บขั้นสูง (อังกฤษ 630,257 pt ไทย 625,257 pt) และ `ui.move` ไปนอกหน้าต่างเพื่อไม่ให้มีตัวชี้เมาส์ในภาพ; ตั้งค่าตัวอักษรด้วย `format.set` (`{"props": {...}}`)
- สคริปต์ของทุกภาพอยู่ที่ `docs/images/shots/<ชื่อภาพ>.jsonl` รันจากรากของ repo แล้วจะเขียนทับภาพใน `docs/images` (ถ่ายใหม่หลังเที่ยง UTC `backstage.png` จะทักทายต่างไป)
- คำสั่งที่ใช้สร้างภาพ `thai.png`: `ui.language` (`th`), `review.language` (`th-TH`, `default: true`), `styles.apply`, `text.insert` (`raw: true`), `text.newParagraph`, `view.zoomOut` และ `ui.key` Home พร้อม `cmd: true` เพื่อกลับไปต้นเอกสาร
- ธีมมืด: `ui.theme` (`value: dark`)

## วิธีตรวจก่อน commit (ตามกฎใน CLAUDE.md)

- ใช้ `cargo xtask ci` ซึ่งรัน fmt, clippy -D warnings, test, assets, layers และ wasm
- ถ้าดิสก์น้อย ให้ใช้ `CARGO_INCREMENTAL=0` และทดสอบทีละแพ็กเกจ
  - หลังแต่ละแพ็กเกจ ให้ลบไฟล์ทดสอบใน `target/debug/deps`
  - build ทั้ง workspace ต้องใช้ดิสก์ราว 25 GB
- เทสต์ Myanmar จะข้ามไปเองเมื่อเครื่องไม่มีฟอนต์ที่มีกฎ `mym2`/`mymr` เช่นเครื่องที่มีแค่ Unifont
