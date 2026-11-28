# รายงานตรวจสอบช่องว่างฟีเจอร์ Khiankrub → Doc4Me

อัปเดตเมื่อ **28 November 2026 16:18:24 น. (Asia/Bangkok)** · source `/workspace/kruboffice/sources/khiankrub` (`ab6d86a`) · target `veevie55/doc4me` (`origin/main` ก่อนชุดนี้: `0768b4a`)

## วิธีเปรียบเทียบ

รัน `wordcraft-cli commands --json` ใน source และ `doc4me-cli commands --json` ใน target แล้วเทียบ command ID แบบตรงตัว: source มี **655** IDs, target มี **776** IDs, source-only **0**, target-only **121**. นอกจากนี้ตรวจ UI และโครงสร้าง DOCX/layout สำหรับฟีเจอร์ที่พฤติกรรมไม่ได้อยู่ใน command ID เพียงอย่างเดียว.

catalog ปัจจุบันไม่เหลือ command ID ของ Khiankrub ที่ขาดใน Doc4Me. ตารางต่อไปนี้เก็บรายการช่องว่าง 22 รายการจากการตรวจรอบก่อน พร้อมจุดที่ปรับให้ตรงกับสถาปัตยกรรม Doc4Me; ชุดโค้ดอยู่ใน working tree และรอ commit/push ในรอบนี้.

## รายการช่องว่างที่พบและผลการพอร์ต

| ประเภท | Source command ID | ความสามารถใน Khiankrub | การปรับให้เข้ากับ Doc4Me | สถานะ |
|---|---|---|---|---|
| แผนภูมิ | <code>chart.format</code> | จัดรูปแบบ chart area, plot area, title, legend, axes, gridlines และ series: fill, border, effects และ text | crate `doc4me-chart-format`, การอ่าน/เขียน DOCX และ Chart pane ของ Doc4Me | พอร์ตแล้ว; รอ commit/push |
| ไฟล์และการแชร์ | <code>file.embedFonts</code> | ฝังฟอนต์ที่เอกสารใช้ใน DOCX; รองรับ subset และ system fonts | ค่าตั้งค่าเอกสาร, subset ของไฟล์ฟอนต์, ตัวเลือกใน File › Options › Save | พอร์ตแล้ว; รอ commit/push |
| ไฟล์และการแชร์ | <code>file.fontEmbedding</code> | อ่านสถานะการฝังฟอนต์ | ส่งคืนค่าตั้งค่าฝังฟอนต์ที่กำลังใช้ | พอร์ตแล้ว; รอ commit/push |
| ความเป็นส่วนตัว | <code>file.privacy</code> | อ่านตัวเลือกการนำชื่อและวันเวลาออกเมื่อตอนบันทึก | แสดงค่าตั้งค่าและแสดงตัวเลือกใน File › Options › Save | พอร์ตแล้ว; รอ commit/push |
| ความเป็นส่วนตัว | <code>file.privacyOptions</code> | กำหนดการปกปิดผู้เขียนและวันเวลาจากข้อมูล DOCX ที่บันทึก | คงข้อมูลในเอกสารที่กำลังแก้ แล้วปกปิดในผลลัพธ์ DOCX ตามตัวเลือก | พอร์ตแล้ว; รอ commit/push |
| แบบอักษร | <code>font.loadSystem</code> | โหลดรายการฟอนต์ที่ติดตั้ง โดย browser ขอสิทธิ์จากผู้ใช้ | เชื่อมกับเมนูฟอนต์บนเว็บ; desktop ส่งสถานะรองรับจากระบบ | พอร์ตแล้ว; รอ commit/push |
| หัวท้ายกระดาษและเลขหน้า | <code>hf.hidePageNumber</code> | ซ่อนหรือแสดงเลขหน้าเฉพาะหน้า | เพิ่มรายการใน context menu เมื่อคลิกขวาบริเวณ header/footer และคงลำดับเลขหน้า | พอร์ตแล้ว; รอ commit/push |
| หัวท้ายกระดาษและเลขหน้า | <code>hf.pageNumberState</code> | อ่านสถานะเลขหน้าของหน้า: shown, hidden หรือ none | คำสั่งอ่านสถานะสำหรับ automation/UI | พอร์ตแล้ว; รอ commit/push |
| หัวท้ายกระดาษและเลขหน้า | <code>hf.showDocumentText</code> | แสดงหรือซ่อน body ขณะแก้ header/footer | คำสั่งสถานะมุมมองและตัวเลือกใน ribbon header/footer | พอร์ตแล้ว; รอ commit/push |
| หัวท้ายกระดาษและเลขหน้า | <code>hf.state</code> | อ่าน section, slot, link-to-previous, ระยะขอบ และรูปแบบเลขหน้า | ปรับผลลัพธ์ให้ใช้ชนิดข้อมูลและ section ของ Doc4Me | พอร์ตแล้ว; รอ commit/push |
| หัวท้ายกระดาษและเลขหน้า | <code>hf.turnOnLandscape</code> | หมุน header/footer บนหน้า landscape ในรายงาน portrait เพื่อให้หันเข้าด้านสัน | จัดวางบน sheet portrait และส่งต่อ pointer hit-testing ให้คลิกแก้ header/footer ได้ | พอร์ตแล้ว; รอ commit/push |
| หัวท้ายกระดาษและเลขหน้า | <code>insert.createFooter</code> | สร้างหรือเปิด footer ของหน้า/section ที่ระบุ | คำสั่ง editor สำหรับ story ของ footer ใน Doc4Me | พอร์ตแล้ว; รอ commit/push |
| หัวท้ายกระดาษและเลขหน้า | <code>insert.createHeader</code> | สร้างหรือเปิด header ของหน้า/section ที่ระบุ | คำสั่ง editor สำหรับ story ของ header ใน Doc4Me | พอร์ตแล้ว; รอ commit/push |
| หัวท้ายกระดาษและเลขหน้า | <code>insert.removeEmptyHeaders</code> | ลบ header/footer ว่างที่สร้างใหม่ | คำสั่ง cleanup ที่ทำงานกับ story/header/footer ของ Doc4Me | พอร์ตแล้ว; รอ commit/push |
| จดหมายเวียน | <code>mailings.filterRecipients</code> | กรองผู้รับ mail merge | เพิ่ม compatibility command และคงการทำงานร่วมกับ `mailings.query` | พอร์ตแล้ว; รอ commit/push |
| จดหมายเวียน | <code>mailings.sortRecipients</code> | เรียงผู้รับ mail merge | เพิ่ม compatibility command และคงการทำงานร่วมกับ `mailings.query` | พอร์ตแล้ว; รอ commit/push |
| สารบัญและรายการอ้างอิง | <code>references.list</code> | เพิ่ม ปรับแต่ง และลบสารบัญ/รายการรูป/รายการตาราง | ใช้ Lists pane สำหรับ Contents, Figures, Tables และ Thai report settings | พอร์ตแล้ว; รอ commit/push |
| สารบัญและรายการอ้างอิง | <code>references.lists</code> | อ่านรายการอ้างอิงที่มีในเอกสาร | เชื่อมข้อมูลตำแหน่งและชนิด list กับโครงสร้าง story/layout ของ Doc4Me | พอร์ตแล้ว; รอ commit/push |
| สารบัญและรายการอ้างอิง | <code>references.updateLists</code> | อัปเดตสารบัญ รายการรูป และรายการตารางทั้งหมด | สร้างรายการใหม่และรองรับ report continuation/page labels | พอร์ตแล้ว; รอ commit/push |
| รูปร่างและการจัดวางวัตถุ | <code>shape.cornerRadius</code> | กำหนดรัศมีมุม rounded rectangle และบันทึกใน DOCX | เก็บ corner value ใน `ShapeExtra`, คำนวณ layout/render/PDF และควบคุมจาก Shape Format | พอร์ตแล้ว; รอ commit/push |
| รูปร่างและการจัดวางวัตถุ | <code>shape.lock</code> | ล็อกหรือปลดล็อกการย้าย/ปรับขนาดและแก้ไขรูปร่าง | เก็บ lock ใน DOCX, ปฏิเสธคำสั่งแก้ไข object และแสดง Lock/Unlock บน Shape Format | พอร์ตแล้ว; รอ commit/push |
| ตาราง | <code>table.paintBorder</code> | วาดเส้นขอบตารางด้วย border painter | มีทั้ง compatibility command และ native `table.borderPainter`/`table.borders` | พอร์ตแล้ว; รอ commit/push |

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
| `b26606c` | 2026-11-28 14:56:25 | Port search, internal links, and line endpoint tools | รวมใน push รอบนี้ |
| `pending` | pending | พอร์ต 22 ช่องว่างและ UI ที่เกี่ยวข้อง | รอ commit/push |

หลัง push ให้แทน `pending` ด้วย hash จริง เวลา ICT และผลยืนยันจาก `git ls-remote origin refs/heads/main`. เวลาทั้งหมดในรายงานใช้เขตเวลา Asia/Bangkok.

## หมายเหตุเรื่องชื่อ API

- `mailings.filterRecipients` และ `mailings.sortRecipients` มี compatibility command IDs โดย Doc4Me ยังคงตัวกรอง/เรียงข้อมูลแบบ native ผ่าน `mailings.query`.
- `table.paintBorder` รองรับเป็น ID สำหรับสคริปต์ Khiankrub และใช้ implementation border painter ของ Doc4Me.
- ความสามารถที่มี ID แล้วแต่ยังขาด UI ตรวจรวมในตารางด้วย เช่น `chart.format`, `file.embedFonts`, `font.loadSystem`, เลขหน้า และ shape lock.

## การตรวจสอบ

- Catalog source/target เปรียบเทียบสำเร็จ; ไม่มี source-only command ID.
- `cargo check --locked --offline -p doc4me-engine -p doc4me-ui-egui` ผ่าน; ไม่ได้รันชุดทดสอบ.
- รายละเอียดความสามารถและภาพประกอบอยู่ใน [`changelog.md`](../changelog.md).
