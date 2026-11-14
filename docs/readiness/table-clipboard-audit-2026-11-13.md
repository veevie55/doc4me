# ตรวจคำสั่งตารางและ rich clipboard · 2026-11-13

ขอบเขตตามคำสั่งผู้ใช้: เพิ่ม/ลบ/เปลี่ยนเส้นขอบ เลือกทั้งแถวหรือคอลัมน์ ซ้ำหัวตาราง แยกตาราง คำสั่งตารางทั้งหมด และคัดลอกวัตถุที่ไม่ใช่ข้อความข้ามเอกสารภายใน Doc4Me งานอยู่ใต้ Unreleased ของ 1.1.3

ตรวจ registry ทั้ง **53 คำสั่ง** (`table.*` รวม `insert.table`) โดยใช้ regression ใหม่ร่วมกับชุดทดสอบเดิม ไม่ใช้เพียงการพบ command id เป็นหลักฐานว่าพฤติกรรมถูกต้อง คะแนน readiness ต้องมาจาก workflow วัดจริงแยกต่างหาก

## สิ่งที่แก้

- เก็บเซลล์ที่เลือกอย่างชัดเจน รวมเซลล์ว่าง แถว คอลัมน์ และทั้งตาราง เพิ่ม `table.selectColumn` ตาม grid span; จัดรูปแบบและลบเนื้อหาเฉพาะเซลล์ที่เลือก รักษารูปตารางและ Undo/Redo
- เส้นขอบเลือกขอบนอก/ใน/บน/ล่าง/ซ้าย/ขวา/แนวนอน/แนวตั้ง มี Single, Double, Dotted, Dashed, Thick, Triple, Dash Dot, Wave พร้อมความกว้างและสี การลบเขียน nil ทั้งสองด้านของขอบร่วมเพื่อไม่ให้เส้นจาก style กลับมา และรักษาขอบอื่นไว้
- แก้ Triple กับ Dash Dot ที่เดิมวาดเหมือน Double กับ Dashed และแก้เส้นคู่/คลื่นแนวตั้ง ทั้ง raster และ PDF; ปากกาเส้นขอบใช้พิกัดหน้าและแก้เฉพาะขอบที่คลิกหรือลาก หนึ่ง stroke เป็นหนึ่ง Undo
- เพิ่ม/ลบแถวหรือคอลัมน์ตามช่วงที่เลือก ซิงก์ preferred widths กับกริด กระจายความสูงจากขนาดที่จัดหน้าจริง แยกเซลล์เป็นส่วนเท่ากันและรักษารูปแบบ รวมแนวตั้งหลายแถวและกริดที่ละเซลล์หัวแถว
- หัวตารางซ้ำได้หลายแถวติดกันตั้งแต่ต้นตาราง การ sort รักษาหัวตารางทั้งหมดและแถวที่อยู่นอกช่วงเลือก การแยกตารางตรวจแถวจริงก่อนเปลี่ยนโครงสร้างและรักษา properties/grid
- คัดลอกทั้งตารางหรือบางแถว/คอลัมน์เป็น TableBlock; เซลล์ต่อของ vertical merge ที่คัดลอกแยกออกมาได้รับเนื้อหาจากต้นกลุ่ม คัดลอกหลายวัตถุครบทุกชิ้นโดยไม่พาข้อความที่ไม่ได้เลือกมาด้วย Cut ลบครบทุกชิ้น
- Clipboard เก็บ bytes ของรูปภาพ รูปในกราฟิก styles/numbering เรื่องในกล่องข้อความ/เชิงอรรถ/อ้างอิงท้ายเรื่อง และ dependency graph ของแผนภูมิ/SmartArt/OLE; วางไปเอกสารใหม่ได้หลังปิดต้นฉบับ ชื่อหรือ id ซ้ำไม่เขียนทับของปลายทาง วัตถุใหม่แก้ไขแยกจากต้นฉบับได้
- แปลงตารางเป็นข้อความและกลับคืนโดยรักษารูปภาพ วัตถุ และ direct character formatting; ไม่ใช้ plain text มาทิ้งโครงสร้างภายในเซลล์
- ตรวจขอบเขตแถว/คอลัมน์ พารามิเตอร์ผิด เส้นทแยงที่ไม่ใช่ขอบ และ story id เต็มช่วง ไม่ใช้ panic เป็นทางทำงาน; paste ที่ทรัพยากรไม่ครบคืนข้อผิดพลาดอย่าง atomic มีเพดาน bytes/definitions/parts และคิด payload ของวัตถุในงบ clipboard history

## เมทริกซ์คำสั่ง

ชื่อทดสอบแบบย่อด้านล่างอยู่ใน [tests_tables.rs](../../crates/engine/src/tests_tables.rs), [tests.rs](../../crates/engine/src/tests.rs) หรือโมดูลที่ระบุ

| กลุ่ม | คำสั่งทั้งหมดในกลุ่ม | หลักฐานพฤติกรรม |
|---|---|---|
| สร้าง | `insert.table`, `table.fromText`, `table.quick` | `tables_commands`, `convert_text_to_table_refuses_to_drop_content`, `backspace_after_a_nested_table_stays_in_its_cell`, converter picture/format regression |
| เพิ่ม | `table.insertRowAbove`, `table.insertRowBelow`, `table.insertColumnLeft`, `table.insertColumnRight`, `table.insertRows`, `table.insertCells` | selected-rectangle insertion; `insert_rows_cells_grid_compatibility_and_number`; จำนวน/ตำแหน่ง/เนื้อหา/ความกว้าง/Undo |
| ลบ | `table.deleteRow`, `table.deleteColumn`, `table.deleteCells`, `table.deleteTable` | multiple-row/cell/last-cell deletion; `deleting_the_last_cell_column_never_leaves_a_table_without_rows`; ไม่เหลือตารางไร้แถว |
| รวม/แยก | `table.merge`, `table.split`, `table.splitTable` | merge/split content + vertical structure; unequal-grid split; `split_cells_leaves_the_other_rows_alone`; split table + stale indices + Undo |
| เลือก | `table.selectCell`, `table.selectRow`, `table.selectColumn`, `table.selectTable` | explicit-cell selection, merged grid columns, empty cells, copy/cut/delete/format/Undo; UI highlight |
| นำทาง | `table.firstCellInRow`, `table.lastCellInRow`, `table.firstCellInColumn`, `table.lastCellInColumn` | `table_keys_move_and_select_like_word`, `shift_alt_keys_select_table_rows_and_columns` |
| เส้นขอบ/สี | `table.borders`, `table.borderPainter`, `table.shading`, `table.viewGridlines` | selected borders/shared nil/invalid atomic; painter geometry + one Undo; raster eight distinct patterns; `table_border_presets_mask_the_sides_they_clear`, `page_and_table_gridlines_toggle_independently` |
| รูปแบบตาราง | `table.style`, `table.look`, `table.newStyle`, `table.modifyStyle`, `table.deleteStyle` | `custom_table_style_applies_modifies_and_saves`, `custom_table_style_column_and_total_regions`, `delete_table_style_falls_back_and_undoes`, built-in deletion guards |
| ขนาด | `table.autofit`, `table.distributeColumns`, `table.distributeRows`, `table.columnWidth`, `table.rowHeight` | `autofit_contents_sizes_columns_to_their_text`, cell size tests, selected distribution + actual layout height, merged-grid widths |
| จัดแนว/คุณสมบัติ | `table.cellAlign`, `table.textDirection`, `table.rtl`, `table.properties`, `table.cellOptions`, `table.cellMargins`, `table.positioning`, `table.altText` | nine alignments + cell isolation + DOCX; `table_properties_apply_in_one_step`, `table_text_direction_cycles_and_turns_the_text`, `table_and_section_direction_commands`, `cell_options_and_table_positioning`, caption copy |
| ข้อมูล | `table.repeatHeader`, `table.sort`, `table.toText`, `table.formula` | multi-row headers on later pages + DOCX; selected sort; conversion retains picture and bold; supported numeric functions, invalid formula atomic |
| เมาส์ | `table.draw`, `table.eraser`, `table.resizeDrag` | geometry/merge/omitted grids/hostile-input tests in [table_draw.rs](../../crates/engine/src/cmd/table_draw.rs) and [table_resize.rs](../../crates/engine/src/cmd/table_resize.rs) |

## หลักฐาน clipboard

ทดสอบรูปภาพ, รูปร่าง, กล่องข้อความ, ตารางเต็ม/บางส่วน/เซลล์ว่าง/vertical merge, multi-object copy/cut, group/canvas, รูปภายในกราฟิก, แผนภูมิและ SmartArt ที่ยังแก้ไขได้, original chart + workbook dependency, OLE preview + opaque payload, สมการ, Ruby, field, footnote/endnote พร้อม DOCX save/reopen, ปิดต้นฉบับ, ชื่อ/id ชนกัน, เปลี่ยนสำเนาโดยไม่เปลี่ยนต้นฉบับ และงบหน่วยความจำของ history

## การยืนยันขั้นสุดท้าย

ตรวจยืนยัน 2026-11-14: CI ผ่าน 6/6, native 2,543, ignored 1 เดิม, assets 141, wasm 16; 32 engine + 1 raster + 2 UI regressions ใหม่ ภาพ UI ตรวจด้วย ui_shot; corpus ของซอร์สตารางรอ workflow หลัง push main ไม่แก้รายงาน readiness ด้วยมือ

## ขอบเขตที่ยังไม่อ้างว่ารองรับ

- เป็น rich clipboard ภายใน Doc4Me; การโอน rich object ผ่าน system clipboard ไปทุกโปรแกรม/ทุกระบบปฏิบัติการยังไม่ได้ยืนยัน
- Word ink (`w14:contentPart`) ที่ reader ยังไม่รองรับยังเป็นช่องว่างเดิมใน [gaps](../gaps.md); OLE เก็บ payload และ preview ไม่เปิดรันโปรแกรม OLE server
- ไม่ sort กลุ่มเซลล์ที่รวมแนวตั้ง และไม่ split table ผ่าน vertical merge ที่ยังต่อจากตารางส่วนแรก; คืนเหตุผลให้ผู้ใช้แยกเซลล์ก่อน ไม่ทำโครงสร้างเสีย
- การแปลง table ที่ถูกหุ้มด้วย content control เชิงโครงสร้างต้องนำ wrapper ออกก่อน เพื่อไม่ทิ้งขอบเขต control อย่างเงียบ ๆ; nested table ยังคงเป็น table block
- การคัดลอกบางส่วนเป็น table fragment ไม่อ้างพฤติกรรมเติมกริดปลายทางแบบ Word ทุกกรณี ขอบของเซลล์ที่รวมเป็นขอบของทั้งเซลล์
- ชุดนี้ไม่ใช่การตรวจ Word UI ด้วยตนเองบน Windows และไม่ทำให้ pagination ผ่านอัตโนมัติ เป้าหมาย readiness 100% ไม่นับ deferred ยังต้องผ่านเกณฑ์เดิม 220 เอกสาร โดยคง 22 deferred / 24 excluded
