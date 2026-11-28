# รายงานตรวจสอบช่องว่างฟีเจอร์ Khiankrub → Doc4Me

อัปเดตเมื่อ **28 November 2026 22:25 น. (Asia/Bangkok)** · source `/workspace/kruboffice/sources/khiankrub` (`ab6d86a`) · target `veevie55/doc4me` · latest verified code push `b5818c296f4f2bba815dfac5fa277ed76ca5dffb` (`main`, commit 2026-11-28 22:25:47 ICT · push ยืนยัน 22:25:53 ICT)

## วิธีเปรียบเทียบ

รัน `wordcraft-cli commands --json` ใน source และ `doc4me-cli commands --json` ใน target แล้วเทียบ command ID แบบตรงตัว: source มี **655** IDs, target มี **779** IDs, source-only **0**, target-only **124**. นอกจากนี้ตรวจ UI และโครงสร้าง DOCX/layout สำหรับฟีเจอร์ที่พฤติกรรมไม่ได้อยู่ใน command ID เพียงอย่างเดียว.

การเทียบ command ID รอบก่อนพบ source-only **0 รายการ** แต่ command catalog ไม่ครอบคลุมพฤติกรรม UI และบริการของ host ทั้งหมด การตรวจไฟล์ระดับโมดูลรอบนี้พบช่องว่างเพิ่มเติมซึ่งยังต้องพอร์ตตามรายการสถานะด้านล่าง; จึงยังไม่นับว่าฟีเจอร์ทั้งหมดจาก source ครบแล้ว. รายการช่องว่าง command 22 รายการจากรอบก่อนพอร์ตใน `fd75ccb` และยืนยันบน `main` แล้ว.

## รายการช่องว่างที่พบและผลการพอร์ต

| ประเภท | Source command ID | ความสามารถใน Khiankrub | การปรับให้เข้ากับ Doc4Me | สถานะ |
|---|---|---|---|---|
| แผนภูมิ | <code>chart.format</code> | จัดรูปแบบ chart area, plot area, title, legend, axes, gridlines และ series: fill, border, effects และ text | crate `doc4me-chart-format`, การอ่าน/เขียน DOCX และ Chart pane ของ Doc4Me | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| ไฟล์และการแชร์ | <code>file.embedFonts</code> | ฝังฟอนต์ที่เอกสารใช้ใน DOCX; รองรับ subset และ system fonts | ค่าตั้งค่าเอกสาร, subset ของไฟล์ฟอนต์, ตัวเลือกใน File › Options › Save | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| ไฟล์และการแชร์ | <code>file.fontEmbedding</code> | อ่านสถานะการฝังฟอนต์ | ส่งคืนค่าตั้งค่าฝังฟอนต์ที่กำลังใช้ | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| ความเป็นส่วนตัว | <code>file.privacy</code> | อ่านตัวเลือกการนำชื่อและวันเวลาออกเมื่อตอนบันทึก | แสดงค่าตั้งค่าและแสดงตัวเลือกใน File › Options › Save | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| ความเป็นส่วนตัว | <code>file.privacyOptions</code> | กำหนดการปกปิดผู้เขียนและวันเวลาจากข้อมูล DOCX ที่บันทึก | คงข้อมูลในเอกสารที่กำลังแก้ แล้วปกปิดในผลลัพธ์ DOCX ตามตัวเลือก | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| แบบอักษร | <code>font.loadSystem</code> | โหลดรายการฟอนต์ที่ติดตั้ง โดย browser ขอสิทธิ์จากผู้ใช้ | เชื่อมกับเมนูฟอนต์บนเว็บ; desktop ส่งสถานะรองรับจากระบบ | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| หัวท้ายกระดาษและเลขหน้า | <code>hf.hidePageNumber</code> | ซ่อนหรือแสดงเลขหน้าเฉพาะหน้า | เพิ่มรายการใน context menu เมื่อคลิกขวาบริเวณ header/footer และคงลำดับเลขหน้า | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| หัวท้ายกระดาษและเลขหน้า | <code>hf.pageNumberState</code> | อ่านสถานะเลขหน้าของหน้า: shown, hidden หรือ none | คำสั่งอ่านสถานะสำหรับ automation/UI | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| หัวท้ายกระดาษและเลขหน้า | <code>hf.showDocumentText</code> | แสดงหรือซ่อน body ขณะแก้ header/footer | คำสั่งสถานะมุมมองและตัวเลือกใน ribbon header/footer | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| หัวท้ายกระดาษและเลขหน้า | <code>hf.state</code> | อ่าน section, slot, link-to-previous, ระยะขอบ และรูปแบบเลขหน้า | ปรับผลลัพธ์ให้ใช้ชนิดข้อมูลและ section ของ Doc4Me | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| หัวท้ายกระดาษและเลขหน้า | <code>hf.turnOnLandscape</code> | หมุน header/footer บนหน้า landscape ในรายงาน portrait เพื่อให้หันเข้าด้านสัน | จัดวางบน sheet portrait และส่งต่อ pointer hit-testing ให้คลิกแก้ header/footer ได้ | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| หัวท้ายกระดาษและเลขหน้า | <code>insert.createFooter</code> | สร้างหรือเปิด footer ของหน้า/section ที่ระบุ | คำสั่ง editor สำหรับ story ของ footer ใน Doc4Me | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| หัวท้ายกระดาษและเลขหน้า | <code>insert.createHeader</code> | สร้างหรือเปิด header ของหน้า/section ที่ระบุ | คำสั่ง editor สำหรับ story ของ header ใน Doc4Me | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| หัวท้ายกระดาษและเลขหน้า | <code>insert.removeEmptyHeaders</code> | ลบ header/footer ว่างที่สร้างใหม่ | คำสั่ง cleanup ที่ทำงานกับ story/header/footer ของ Doc4Me | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| จดหมายเวียน | <code>mailings.filterRecipients</code> | กรองผู้รับ mail merge | เพิ่ม compatibility command และคงการทำงานร่วมกับ `mailings.query` | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| จดหมายเวียน | <code>mailings.sortRecipients</code> | เรียงผู้รับ mail merge | เพิ่ม compatibility command และคงการทำงานร่วมกับ `mailings.query` | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| สารบัญและรายการอ้างอิง | <code>references.list</code> | เพิ่ม ปรับแต่ง และลบสารบัญ/รายการรูป/รายการตาราง | ใช้ Lists pane สำหรับ Contents, Figures, Tables และ Thai report settings | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| สารบัญและรายการอ้างอิง | <code>references.lists</code> | อ่านรายการอ้างอิงที่มีในเอกสาร | เชื่อมข้อมูลตำแหน่งและชนิด list กับโครงสร้าง story/layout ของ Doc4Me | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| สารบัญและรายการอ้างอิง | <code>references.updateLists</code> | อัปเดตสารบัญ รายการรูป และรายการตารางทั้งหมด | สร้างรายการใหม่และรองรับ report continuation/page labels | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| รูปร่างและการจัดวางวัตถุ | <code>shape.cornerRadius</code> | กำหนดรัศมีมุม rounded rectangle และบันทึกใน DOCX | เก็บ corner value ใน `ShapeExtra`, คำนวณ layout/render/PDF และควบคุมจาก Shape Format | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| รูปร่างและการจัดวางวัตถุ | <code>shape.lock</code> | ล็อกหรือปลดล็อกการย้าย/ปรับขนาดและแก้ไขรูปร่าง | เก็บ lock ใน DOCX, ปฏิเสธคำสั่งแก้ไข object และแสดง Lock/Unlock บน Shape Format | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |
| ตาราง | <code>table.paintBorder</code> | วาดเส้นขอบตารางด้วย border painter | มีทั้ง compatibility command และ native `table.borderPainter`/`table.borders` | พอร์ตแล้ว; `fd75ccb` · push 2026-11-28 16:20 ICT |

## ช่องว่างเพิ่มเติมจากการเทียบพฤติกรรมและโมดูล

ตรวจ source `ab6d86a` เทียบ implementation จริงใน Doc4Me; สถานะ “ค้าง” หมายถึงยังไม่พบพฤติกรรมที่เทียบเท่าบน `main` ณ push ล่าสุด.

| ประเภท | ความสามารถใน Khiankrub | สถานะใน Doc4Me | สถานะงาน |
|---|---|---|---|
| Canvas และการเลือก | คลิกปุ่มกลางเพื่อเลื่อนเอกสารตามระยะ pointer, หยุดด้วย Escape/ปุ่มอื่น/ล้อเลื่อน (`ui-egui/autoscroll.rs`) | มี edge auto-scroll ระหว่างลากเลือกข้อความ และ middle-click scroll ที่เคารพการยกเลิก gesture | พอร์ตแล้ว; `bd5a2c7` · push 2026-11-28 20:26 ICT |
| รูปภาพ | Crop mode บน canvas พร้อม handles, preview ส่วนที่ถูกตัด, ลากคืนพื้นที่ภาพ และจบเป็น undo step เดียว (`ui-egui/crop.rs`) | Crop overlay/handles พร้อม preview; `picture.crop` รับ crop และขนาดใหม่ใน undo step เดียว | พอร์ตแล้ว; `5eaaeec` · push 2026-11-28 20:31 ICT |
| จัดวางวัตถุ | Snap วัตถุไปยังขอบ/กึ่งกลางหน้า, margins และวัตถุอื่น พร้อมเส้น alignment guides (`ui-egui/guides.rs`) | ลาก/ปรับขนาดแล้ว snap พร้อมเส้นนำทาง; Alt ปิด snap, drawing grid มีลำดับความสำคัญ, ปิดได้ใน Options › Advanced | พอร์ตแล้ว; `e315da4` · push 2026-11-28 20:36 ICT |
| บันทึกบนเว็บ | ติดตาม save ticket ที่เสร็จภายหลัง; Save Changes รอผล picker และคงเอกสารไว้เมื่อยกเลิก/ผิดพลาด (`ui-egui/async_saves.rs`) | `AsyncSaves` รายงานผล Saved/Cancelled/Failed; คำสั่ง New/Open ที่ค้างจะทำต่อเมื่อเขียนไฟล์และปิดสำเร็จ โดยตรวจ document/revision; ถ้ายกเลิกหรือผิดพลาดจะคงเอกสารไว้เป็น unsaved และแสดงเหตุผล | พอร์ตแล้ว; `d2943cf` · push 2026-11-28 21:18 ICT |
| อัปเดตแอป | ตรวจ GitHub release อัตโนมัติรายวัน/จาก Help และแสดงรุ่นใหม่ให้เปิดหน้าดาวน์โหลด (`ui-egui/updates.rs`) | desktop มี dialog ขออนุญาตครั้งแรก, ตรวจรายวันเมื่อเปิดใช้, Help ตรวจเอง, แถบรุ่นใหม่พร้อม Download/Skip/Close; ตรวจผ่าน Trust Center และเปิดหน้า release โดยไม่ดาวน์โหลดหรือติดตั้งเอง | พอร์ตแล้ว; `f462f42` · commit 2026-11-28 21:35:45 ICT · push/ยืนยัน `origin/main` 2026-11-28 21:35:55 ICT (`f462f427708ab600cf782a30563960bfed956426`) |
| แก้ไขตัวอักษร | Character dialog แยกหน้า Font, Effects, Position, Background และ Border (`ui-egui/dialogs_character.rs`) | ขยาย Font dialog โดยคง Font/Advanced และเพิ่ม Effects, Position, Background, Border; รองรับฟอนต์/ขนาด/รูปแบบ/ภาษา Western, East Asian และ complex script, underline/สี, strike, caps/effects, ตำแหน่ง/scale/spacing, shading และ border; มี preview และ Reset/Cancel/OK; ใช้ `format.set` พร้อม `clear` เพื่อรวมการแก้ไขเป็น action เดียว | พอร์ตแล้ว; `3a88721` · commit 2026-11-28 21:58:00 ICT · push/ยืนยัน `origin/main` 2026-11-28 21:58:10 ICT (`3a88721b43f7cd02cbceaf7ae984978f0f15e9cf`) |
| สัญลักษณ์หัวข้อ | Define New Bullet dialog เลือก character, font, size, bold/italic, color และ alignment พร้อม preview (`ui-egui/dialogs_bullet.rs`) | เพิ่ม dialog ใน Home › Paragraph › Bullets; รูปแบบสร้าง private list definition และไม่กระทบรายการอื่น; การเรียก command เดิมที่ส่งแค่ `char` ยังคง toggle แบบเดิม | พอร์ตแล้ว; `5570d26` · push 2026-11-28 20:46 ICT |
| แผงจัดรูปแบบ | แผงเต็มความสูงสำหรับข้อความ/ย่อหน้า/หน้า/ดีไซน์ (`ui-egui/format_panes.rs`) | View › Panes › Formatting Pane เปิดแผงปรับขนาดได้ สลับ Text, Paragraph, Page และ Design; ทุก control ใช้คำสั่งเดิมของ Doc4Me และปิดไว้โดยค่าเริ่มต้น | พอร์ตแล้ว; `d372c42` · commit 2026-11-28 22:07:04 ICT · push/ยืนยัน `origin/main` 2026-11-28 22:07:23 ICT (`d372c4264aadda3126193210edbaabbfd033a553`) |
| แผงตารางและวัตถุ | Table pane แยก Styles/Borders/Rows and Columns; object pane สำหรับ Shape, Picture, Chart และ SmartArt (`ui-egui/table_pane.rs`, `object_pane.rs`) | View › Panes › Table Pane รวม style/look, borders/shading/pen, rows/columns, merge/split, cell size/alignment และ data; Object Pane ปรับชุดคำสั่งตาม selection และมี fill/outline/effects/size/position/text, picture adjust/crop, chart layout/data/type/size และ SmartArt layout/colors/text actions; Chart Format pane เดิมยังใช้ได้ | พอร์ตแล้ว; `b5818c2` · commit 2026-11-28 22:25:47 ICT · push/ยืนยัน `origin/main` 2026-11-28 22:25:53 ICT (`b5818c296f4f2bba815dfac5fa277ed76ca5dffb`) |
| ลำดับเรียงตามภาษา | Collation ตามภาษาสำหรับ accent และอักษรเฉพาะ (`doc/collate.rs`) | เพิ่ม comparator ภาษา; ใช้กับ paragraph/table sort, index entries/letter headings และ bibliography; รองรับลำดับอักษรเฉพาะของภาษาและ decomposed accents | พอร์ตแล้ว; `5983459` · push 2026-11-28 20:55 ICT |
| ตัวเลขตาม locale | แยกเลขทศนิยม/ตัวคั่นหลักพันและสกุลเงินของภาษาขณะ sort ตาราง (`doc/locale_number.rs`) | `table.sort` อ่านภาษาของ cell; รองรับจุด/จุลภาคทศนิยม, space/จุด/จุลภาค/apostrophe คั่นหลักพัน, สัญลักษณ์สกุลเงิน, เปอร์เซ็นต์ และเครื่องหมายลบหลายแบบ | พอร์ตแล้ว; `1eb5219` · push 2026-11-28 20:51 ICT |
| ฟอนต์สัญลักษณ์ | แปลง Symbol/Wingdings/Webdings เป็น Unicode เมื่อไม่มีฟอนต์ต้นฉบับ (`doc/symbol_fonts.rs`) | เพิ่ม mapping สำหรับ Symbol, Wingdings 1–3 และ Webdings; ใช้ใน layout/display, `Paragraph::shown_text`, story text และ export `.txt`; ไม่เปลี่ยนรหัสที่เก็บในเอกสาร | พอร์ตแล้ว; `2caed36` · push 2026-11-28 21:02 ICT |
| งานนำเสนอ/หน้าตา | ลายไทยบนแถบด้านบน, pointer แบบวาดเอง, ภาพตัวอย่าง line style และปุ่มรายงานปัญหา (`band_motifs.rs`, `pointer.rs`, `line_looks.rs`, `report.rs`) | มีแถบสี, system pointer, ตัวเลือกเส้น และไม่มี report-form integration; รายละเอียด/การตั้งค่าไม่เทียบเท่า source | ค้างหรือเทียบเท่าบางส่วน |
| ตรวจทาน UI | Hover stability และ large-table performance checks (`hover_stability.rs`, `tests_perf.rs`) | เป็น test harness ของ source ไม่ใช่ฟีเจอร์ใช้งาน; ไม่รวมในรายการพอร์ต production | ไม่นับเป็นฟีเจอร์ผู้ใช้ |

### ความสามารถที่มีตำแหน่งไฟล์ต่างกัน แต่พบ implementation เทียบเท่าแล้ว

- `ui-egui/printing.rs` เทียบกับ `print_page.rs` และ `engine/print.rs`: ตั้งช่วงหน้า, odd/even, copies และ collate ได้.
- `engine/cmd/table_formula.rs` เทียบกับ `engine/cmd/table.rs`: มี `table.formula` ใน Doc4Me.
- `ui-egui/dialogs_table_style.rs` เทียบกับ `ui-egui/dialogs.rs`: มี New/Modify Table Style dialog.
- การคัดลอก rich clipboard เทียบกับ `ui-egui/system_clipboard.rs` และ `apps/doc4me/src/clipboard.rs`.
- การนับคำภาษาไทยใช้ dictionary segmentation ใน `doc::dictionary_words`; ไม่ต้องพอร์ตไฟล์แยก `thai_words.rs`.
- ช่องว่างด้านภูมิภาค/หน่วย, resize ตาราง และการตั้ง/ย้าย/ลบ tab stop บน ruler รวมอยู่ใน code push `6215d03`.

## Commit และ push

| Commit | วันและเวลา commit (ICT) | งาน | push ไป `main` |
|---|---|---|---|
| `380a1b7` | 2026-11-28 12:05:51 | Add a session-only view mode | ยืนยันบน `origin/main` |
| `efee91f` | 2026-11-28 12:30:36 | Port remaining Khiankrub navigation and list features | ยืนยันบน `origin/main` |
| `ddc07f1` | 2026-11-28 13:12:14 | Port remaining Khiankrub list and page number features | ยืนยันบน `origin/main` |
| `093da12` | 2026-11-28 13:30:56 | Port full style editor from Khiankrub | ยืนยันบน `origin/main` |
| `c8e344e` | 2026-11-28 13:54:31 | Add Undo History to the Quick Access Toolbar | ยืนยันบน `origin/main` |
| `d42436e` | 2026-11-28 13:58:17 | Add native Full Screen view command | ยืนยันบน `origin/main` |
| `46962fe` | 2026-11-28 14:05:10 | Add document renaming | ยืนยันบน `origin/main` |
| `868a07c` | 2026-11-28 14:13:47 | Add drag and drop for selected text | ยืนยันบน `origin/main` |
| `2f95c82` | 2026-11-28 14:25:12 | Add table selection and mouse operations | ยืนยันบน `origin/main` |
| `a3e7f4b` | 2026-11-28 14:27:56 | Add copy by dragging selected objects | ยืนยันบน `origin/main` |
| `0768b4a` | 2026-11-28 14:33:26 | Add save as picture | ยืนยันบน `origin/main` |
| `b26606c` | 2026-11-28 14:56:25 | Port search, internal links, and line endpoint tools | ยืนยันบน `origin/main` |
| `fd75ccb` | 2026-11-28 16:20:21 ICT | พอร์ต 22 ช่องว่างและ UI ที่เกี่ยวข้อง | push 2026-11-28 16:20 ICT; `origin/main` ยืนยัน SHA `fd75ccb4bee4622a59df53d736350d4fee9dc429` |
| `6215d03` | 2026-11-28 20:19:45 ICT | พอร์ต region-aware units/number formatting, table ruler interactions และ tab-stop ruler; ปรับ DOCX/text import ให้รับ region | push 2026-11-28 20:19 ICT; `origin/main` ยืนยัน SHA `6215d03b08568a2042a5433ca60486b203e45511` |
| `bd5a2c7` | 2026-11-28 20:26:18 ICT | เพิ่ม middle-click canvas auto-scroll สำหรับ platform ที่รองรับ | push 2026-11-28 20:26 ICT; `origin/main` ยืนยัน SHA `bd5a2c7476d1aa29136baae2c01d50e4d26bed20` |
| `5eaaeec` | 2026-11-28 20:31:30 ICT | เพิ่ม Crop mode บน canvas และบันทึก crop/size พร้อมกัน | push 2026-11-28 20:31 ICT; `origin/main` ยืนยัน SHA `5eaaeec921bf336cc07d6cde5dfb1583ea2a9861` |
| `e315da4` | 2026-11-28 20:36:27 ICT | เพิ่ม alignment snapping/guides สำหรับการย้ายและปรับขนาดวัตถุ | push 2026-11-28 20:36 ICT; `origin/main` ยืนยัน SHA `e315da4d578d2abb2604f137e55bc5396404c680` |
| `5570d26` | 2026-11-28 20:46:10 ICT | เพิ่ม Define New Bullet dialog, preview และการจัดรูปแบบ bullet แบบส่วนตัว | push 2026-11-28 20:46 ICT; `origin/main` ยืนยัน SHA `5570d26b981a3d01b8218a4f5802b66540ae261f` |
| `1eb5219` | 2026-11-28 20:51:09 ICT | เรียงค่าตัวเลขในตารางด้วย number parser ตามภาษาของ cell | push 2026-11-28 20:51 ICT; `origin/main` ยืนยัน SHA `1eb5219c18d71a31e5284d8da17459b63477fe0f` |
| `5983459` | 2026-11-28 20:55:54 ICT | เพิ่ม language-aware collation ในการเรียงย่อหน้า ตาราง ดัชนี และ bibliography | push 2026-11-28 20:55 ICT; `origin/main` ยืนยัน SHA `598345932d6bc6d2cae23606eeca3d639620b383` |
| `2caed36` | 2026-11-28 21:02:35 ICT | เพิ่ม Unicode mappings สำหรับ Symbol, Wingdings 1–3 และ Webdings ในการแสดงผล/ข้อความธรรมดา | push 2026-11-28 21:02 ICT; `origin/main` ยืนยัน SHA `2caed36b6820874d75db2374356e5b0a7ffbb5ca` |
| `d2943cf` | 2026-11-28 21:18:38 ICT | เพิ่ม web async save tickets; Save Changes รอผล picker และคืน prompt พร้อมเหตุผลเมื่อบันทึกไม่สำเร็จ | push 2026-11-28 21:18:48 ICT; `origin/main` ยืนยัน SHA `d2943cf16f81785dda1fb8999bfe2707a35675ae` |
| `f462f42` | 2026-11-28 21:35:45 ICT | เพิ่ม GitHub release update checker บน desktop | push 2026-11-28 21:35:55 ICT; `origin/main` ยืนยัน SHA `f462f427708ab600cf782a30563960bfed956426` |
| `3a88721` | 2026-11-28 21:58:00 ICT | ขยาย Character dialog และ `format.set` | push 2026-11-28 21:58:10 ICT; `origin/main` ยืนยัน SHA `3a88721b43f7cd02cbceaf7ae984978f0f15e9cf` |
| `d372c42` | 2026-11-28 22:07:04 ICT | เพิ่มแผง Formatting สำหรับ Text, Paragraph, Page และ Design | push 2026-11-28 22:07:23 ICT; `origin/main` ยืนยัน SHA `d372c4264aadda3126193210edbaabbfd033a553` |
| `b5818c2` | 2026-11-28 22:25:47 ICT | เพิ่ม Table Pane และ Object Pane สำหรับการจัดรูปแบบตามบริบท | push 2026-11-28 22:25:53 ICT; `origin/main` ยืนยัน SHA `b5818c296f4f2bba815dfac5fa277ed76ca5dffb` |

`fd75ccb` commit เวลา 2026-11-28 16:20:21 ICT, push เวลา 2026-11-28 16:20 ICT; `git ls-remote origin refs/heads/main` คืน `fd75ccb4bee4622a59df53d736350d4fee9dc429`. เวลาทั้งหมดในรายงานใช้เขตเวลา Asia/Bangkok.

## หมายเหตุเรื่องชื่อ API

- `mailings.filterRecipients` และ `mailings.sortRecipients` มี compatibility command IDs โดย Doc4Me ยังคงตัวกรอง/เรียงข้อมูลแบบ native ผ่าน `mailings.query`.
- `table.paintBorder` รองรับเป็น ID สำหรับสคริปต์ Khiankrub และใช้ implementation border painter ของ Doc4Me.
- ความสามารถที่มี ID แล้วแต่ยังขาด UI ตรวจรวมในตารางด้วย เช่น `chart.format`, `file.embedFonts`, `font.loadSystem`, เลขหน้า และ shape lock.

## การตรวจสอบ

- Catalog source/target รอบก่อนเปรียบเทียบแล้ว: ไม่มี source-only command ID; behavioral gaps เพิ่มเติมอยู่ในตารางข้างต้น.
- `cargo check --locked --offline` ผ่านสำหรับ desktop, CLI, web, engine, UI, DOCX, render และ PDF ณ `6215d03`; middle-click `bd5a2c7` ผ่าน UI/desktop/web check; Crop `5eaaeec` ผ่าน engine/UI/desktop/web check; alignment guides `e315da4` ผ่าน UI/desktop/web check; Define New Bullet `5570d26` ผ่าน engine/UI/desktop/CLI/web check; locale sort `1eb5219` และ collation `5983459` ผ่าน doc/engine/UI/desktop/CLI/web check; symbol fonts `2caed36` ผ่าน doc/formats/layout/engine/UI/desktop/CLI/web check; async save `d2943cf` ผ่าน `cargo fmt --all -- --check`, `cargo check --locked --offline -p doc4me-ui-egui` และ `cargo check --locked --offline -p doc4me-web --target wasm32-unknown-unknown`. ไม่ได้รันชุดทดสอบ.
- รายละเอียดความสามารถและภาพประกอบอยู่ใน [`changelog.md`](../changelog.md).
