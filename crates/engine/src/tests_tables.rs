//! End-to-end table commands and rich object clipboard regressions.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use doc4me_doc::para::{Float, ShapeKind};
use doc4me_doc::props::{Align, Border, BorderStyle, Borders, HeightRule, VAlign, VMerge};
use doc4me_doc::{Block, CharProps, Document, InlineObject, Paragraph, PartKind, Path, Pos, Rgb, StoryRef, Table, para_block};
use serde_json::{Value, json};

use crate::{Selection, Session};

fn run(s: &mut Session, id: &str, v: Value) -> Value {
    s.run(id, &v).unwrap_or_else(|e| panic!("{id}: {e}"))
}
fn at(r: usize, c: usize) -> Pos {
    Pos { story: StoryRef::Body, path: Path(vec![1, r as u32, c as u32, 0]), off: 0 }
}
fn focus(s: &mut Session, r: usize, c: usize) {
    s.sel = Selection::caret(at(r, c));
    s.table_selection = None;
}
fn table(s: &Session) -> &Table {
    s.doc.table(StoryRef::Body, &Path::top(1)).unwrap()
}
fn label(r: usize, c: usize) -> String {
    format!("r{r}c{c}")
}
fn fixture(rows: usize, cols: usize) -> Session {
    let mut d = Document::new();
    let mut t = Table::new(rows, cols, 300.0);
    for (r, row) in t.rows.iter_mut().enumerate() {
        for (c, cell) in row.cells.iter_mut().enumerate() {
            cell.blocks = vec![para_block(Paragraph::with_text(&label(r, c), CharProps::default()))];
        }
    }
    d.body = vec![
        para_block(Paragraph::with_text("Before", Default::default())),
        Arc::new(Block::Table(t)),
        para_block(Paragraph::with_text("After", Default::default())),
    ];
    let mut s = Session::new(d);
    focus(&mut s, 0, 0);
    s
}
fn cell_text(t: &Table, r: usize, c: usize) -> String {
    t.rows[r].cells[c].blocks.iter().filter_map(|b| b.as_para()).map(|p| p.text.as_str()).collect::<Vec<_>>().join("\n")
}
fn roundtrip(d: &Document) -> Document {
    doc4me_docx::read(&doc4me_docx::write(d).unwrap()).unwrap()
}
fn png(color: u8) -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(2, 2, image::Rgba([color, 20, 30, 255])))
        .write_to(&mut out, image::ImageFormat::Png)
        .unwrap();
    out.into_inner()
}
fn picture(d: &mut Document, color: u8) -> InlineObject {
    InlineObject::Image {
        media: d.add_media(png(color), "png"),
        w: 24.0,
        h: 12.0,
        alt: "original picture".into(),
        float: Float::default(),
        crop: [0.0; 4],
        ole: None,
        video: None,
    }
}
fn shape(story: Option<u32>) -> InlineObject {
    InlineObject::Shape {
        kind: ShapeKind::Rectangle,
        w: 48.0,
        h: 36.0,
        fill: Some(Rgb(1, 2, 3)),
        stroke: Some(Rgb(4, 5, 6)),
        stroke_width: 1.5,
        float: Float::default(),
        story,
        freeform: None,
        effects: Default::default(),
        extra: Default::default(),
    }
}
fn pick(s: &mut Session, pos: Pos) {
    s.sel = Selection { anchor: pos.clone(), focus: Pos { off: pos.off + doc4me_doc::para::OBJ.len_utf8(), ..pos } };
    s.table_selection = None;
}

#[test]
fn table_column_and_row_selection_only_format_the_selected_cells_and_survive_undo() {
    let mut s = fixture(3, 4);
    focus(&mut s, 1, 2);
    let history = s.undo_depth();
    run(&mut s, "table.selectColumn", json!({}));
    assert_eq!(s.undo_depth(), history);
    assert_eq!(s.selected_text(), "r0c2\nr1c2\nr2c2");
    assert_eq!(s.selected_table_cells().unwrap().cells, BTreeSet::from([(0, 2), (1, 2), (2, 2)]));
    run(&mut s, "table.shading", json!({"color":"112233"}));
    for r in 0..3 {
        for c in 0..4 {
            assert_eq!(table(&s).rows[r].cells[c].props.shading, (c == 2).then_some(Rgb(0x11, 0x22, 0x33)));
        }
    }
    run(&mut s, "edit.undo", json!({}));
    assert!(s.selected_table_cells().is_some());
    run(&mut s, "edit.redo", json!({}));
    assert!(s.selected_table_cells().is_some());
    focus(&mut s, 1, 1);
    run(&mut s, "table.selectRow", json!({}));
    assert_eq!(s.selected_table_cells().unwrap().cells, BTreeSet::from([(1, 0), (1, 1), (1, 2), (1, 3)]));
    run(&mut s, "table.rowHeight", json!({"height":42,"rule":"exact"}));
    assert_eq!(table(&s).rows.iter().map(|r| r.props.height).collect::<Vec<_>>(), vec![None, Some(42.0), None]);
}

#[test]
fn table_column_selection_follows_grid_columns_across_merged_cells() {
    let mut s = fixture(3, 3);
    s.doc.table_mut(StoryRef::Body, &Path::top(1)).unwrap().merge(0, 0, 0, 1);
    focus(&mut s, 1, 1);
    run(&mut s, "table.selectColumn", json!({}));
    assert_eq!(s.selected_table_cells().unwrap().cells, BTreeSet::from([(0, 0), (1, 1), (2, 1)]));
    run(&mut s, "table.shading", json!({"color":"ABCDEF"}));
    assert_eq!(table(&s).rows[0].cells[0].props.shading, Some(Rgb(0xAB, 0xCD, 0xEF)));
    assert_eq!(table(&s).rows[0].cells[1].props.shading, None);
}

#[test]
fn copying_a_whole_table_keeps_rows_columns_properties_and_docx_structure() {
    let mut s = fixture(3, 3);
    run(&mut s, "table.altText", json!({"title":"A table","text":"caption"}));
    let expected = table(&s).clone();
    run(&mut s, "table.selectTable", json!({}));
    run(&mut s, "edit.copy", json!({}));
    assert!(matches!(s.clipboard.as_ref().unwrap().blocks.as_slice(), [Block::Table(_)]));
    run(&mut s, "file.new", json!({}));
    run(&mut s, "edit.paste", json!({}));
    assert!(matches!(&*s.doc.body[0], Block::Table(_)), "no extra blank paragraph before a pasted table");
    let got = s.doc.body.iter().find_map(|b| b.as_table()).unwrap();
    assert_eq!(got.grid, expected.grid);
    assert_eq!(got.props, expected.props);
    assert_eq!(got.rows.len(), 3);
    for r in 0..3 {
        for c in 0..3 {
            assert_eq!(cell_text(got, r, c), label(r, c));
        }
    }
    let d = roundtrip(&s.doc);
    assert_eq!(d.body.iter().filter(|b| b.as_table().is_some()).count(), 1);
    assert_eq!(d.body.iter().find_map(|b| b.as_table()).unwrap().rows.len(), 3);
}

#[test]
fn copying_rows_columns_and_empty_cells_keeps_table_fragments() {
    for column in [false, true] {
        let mut s = fixture(3, 3);
        focus(&mut s, 1, 1);
        run(&mut s, if column { "table.selectColumn" } else { "table.selectRow" }, json!({}));
        run(&mut s, "edit.copy", json!({}));
        let Block::Table(t) = &s.clipboard.as_ref().unwrap().blocks[0] else { panic!("table structure lost") };
        assert_eq!((t.rows.len(), t.grid.len()), if column { (3, 1) } else { (1, 3) });
        if column {
            for r in 0..3 {
                assert_eq!(cell_text(t, r, 0), label(r, 1));
            }
        } else {
            for c in 0..3 {
                assert_eq!(cell_text(t, 0, c), label(1, c));
            }
        }
    }
    let mut s = fixture(1, 1);
    s.doc.para_mut(StoryRef::Body, &at(0, 0).path).unwrap().delete(0, 4).unwrap();
    run(&mut s, "table.selectCell", json!({}));
    assert!(s.sel.is_collapsed());
    run(&mut s, "edit.copy", json!({}));
    assert!(matches!(s.clipboard.as_ref().unwrap().blocks.as_slice(), [Block::Table(_)]));
    run(&mut s, "text.delete", json!({}));
    assert!(s.doc.table(StoryRef::Body, &Path::top(1)).is_some());
}

#[test]
fn deleting_a_selected_column_clears_only_its_contents_and_retains_the_grid() {
    let mut s = fixture(3, 3);
    focus(&mut s, 1, 1);
    run(&mut s, "table.selectColumn", json!({}));
    run(&mut s, "text.delete", json!({}));
    assert_eq!((table(&s).rows.len(), table(&s).grid.len()), (3, 3));
    for r in 0..3 {
        for c in 0..3 {
            assert_eq!(cell_text(table(&s), r, c), if c == 1 { String::new() } else { label(r, c) });
        }
    }
    run(&mut s, "edit.undo", json!({}));
    assert!(s.selected_table_cells().is_some());
    for r in 0..3 {
        assert_eq!(cell_text(table(&s), r, 1), label(r, 1));
    }
}

#[test]
fn pasting_over_a_selected_table_replaces_it_without_nesting_or_losing_text() {
    let mut s = fixture(2, 2);
    run(&mut s, "table.selectTable", json!({}));
    run(&mut s, "edit.copy", json!({}));
    let before = s.doc.clone();
    run(&mut s, "edit.paste", json!({}));
    assert_eq!(s.doc.body.iter().filter(|b| b.as_table().is_some()).count(), 1);
    assert_eq!(cell_text(table(&s), 1, 1), label(1, 1));
    assert!(table(&s).rows.iter().flat_map(|r| &r.cells).flat_map(|c| &c.blocks).all(|b| b.as_table().is_none()));
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(s.doc, before);
    assert!(s.selected_table_cells().is_some());
}

#[test]
fn selected_table_borders_keep_styles_and_unrelated_cell_edges() {
    let mut s = fixture(3, 3);
    let green = Border { style: BorderStyle::Dotted, width: 2.0, color: Some(Rgb(0, 100, 0)), space: 0.0 };
    s.doc.table_mut(StoryRef::Body, &Path::top(1)).unwrap().rows[0].cells[0].props.borders = Some(Borders::all(green));
    focus(&mut s, 1, 1);
    run(&mut s, "table.selectRow", json!({}));
    run(&mut s, "table.borders", json!({"kind":"all","style":"double","width":1.5,"color":"AA3300"}));
    let b = table(&s).rows[1].cells[1].props.borders.unwrap().top.unwrap();
    assert_eq!((b.style, b.width, b.color), (BorderStyle::Double, 1.5, Some(Rgb(0xAA, 0x33, 0))));
    let untouched = table(&s).rows[0].cells[0].props.borders.unwrap();
    assert_eq!(untouched.top, Some(green));
    assert_eq!(untouched.left, Some(green));
    assert_eq!(untouched.right, Some(green));
    run(&mut s, "table.borders", json!({"kind":"none"}));
    assert_eq!(table(&s).rows[1].cells[1].props.borders.unwrap().top.unwrap().style, BorderStyle::None);
    assert_eq!(table(&s).rows[0].cells[1].props.borders.unwrap().bottom.unwrap().style, BorderStyle::None);
    let d = roundtrip(&s.doc);
    assert_eq!(d.table(StoryRef::Body, &Path::top(1)).unwrap().rows[1].cells[1].props.borders.unwrap().left.unwrap().style, BorderStyle::None);
    let before = s.doc.clone();
    let undo = s.undo_depth();
    assert!(s.run("table.borders", &json!({"style":"unknown"})).is_err());
    assert_eq!(s.doc, before);
    assert_eq!(s.undo_depth(), undo);
}

#[test]
fn multiple_header_rows_repeat_in_layout_and_sort_preserves_all_of_them() {
    let mut s = fixture(8, 1);
    let a = at(0, 0);
    let b = Pos { off: 4, ..at(1, 0) };
    s.sel = Selection { anchor: a, focus: b };
    run(&mut s, "table.repeatHeader", json!({"value":true}));
    assert_eq!(table(&s).rows.iter().map(|r| r.props.header).collect::<Vec<_>>(), vec![true, true, false, false, false, false, false, false]);
    s.doc.last_section.page_h = 180.0;
    s.doc.last_section.margin_top = 10.0;
    s.doc.last_section.margin_bottom = 10.0;
    for (i, row) in s.doc.table_mut(StoryRef::Body, &Path::top(1)).unwrap().rows.iter_mut().enumerate() {
        row.props.height = Some(if i < 2 { 24.0 } else { 50.0 });
        row.props.height_rule = HeightRule::AtLeast;
    }
    let layout = s.layout();
    assert!(layout.pages.len() > 2);
    for page in &layout.pages[1..] {
        for row in 0..2 {
            assert!(
                page.items.iter().any(|i| matches!(i,doc4me_layout::Placed::Lines{path,..} if path.0.get(..3)==Some(&[1,row,0]))),
                "header row {row} missing"
            );
        }
    }
    focus(&mut s, 4, 0);
    run(&mut s, "table.sort", json!({"column":0,"descending":true}));
    assert_eq!(cell_text(table(&s), 0, 0), label(0, 0));
    assert_eq!(cell_text(table(&s), 1, 0), label(1, 0));
    assert_eq!(cell_text(table(&s), 2, 0), label(7, 0));
    let d = roundtrip(&s.doc);
    assert!(d.table(StoryRef::Body, &Path::top(1)).unwrap().rows[..2].iter().all(|r| r.props.header));
}

#[test]
fn headers_require_a_prefix_and_split_table_is_atomic_and_undoable() {
    let mut s = fixture(4, 2);
    focus(&mut s, 2, 0);
    let before = s.doc.clone();
    assert!(s.run("table.repeatHeader", &json!({"value":true})).is_err());
    assert_eq!(s.doc, before);
    run(&mut s, "table.splitTable", json!({}));
    let tables: Vec<_> = s.doc.body.iter().filter_map(|b| b.as_table()).collect();
    assert_eq!(tables.iter().map(|t| t.rows.len()).collect::<Vec<_>>(), vec![2, 2]);
    assert_eq!(cell_text(tables[1], 0, 0), label(2, 0));
    assert_eq!(tables[0].grid, tables[1].grid);
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(s.doc, before);
    focus(&mut s, 0, 0);
    assert!(s.run("table.splitTable", &json!({})).is_err());
    s.sel = Selection::caret(Pos { story: StoryRef::Body, path: Path(vec![1, u32::MAX, u32::MAX, 0]), off: usize::MAX });
    let e = s.run("table.splitTable", &json!({})).unwrap_err().to_string();
    assert!(!e.contains("panicked"), "{e}");
}

#[test]
fn pictures_survive_copy_new_document_paste_and_media_name_collisions() {
    let mut d = Document::from_text("left right");
    let image = picture(&mut d, 10);
    let original = png(10);
    d.para_mut(StoryRef::Body, &Path::top(0)).unwrap().insert_object(4, image, &CharProps::default()).unwrap();
    let mut s = Session::new(d);
    pick(&mut s, Pos::body(0, 4));
    run(&mut s, "edit.copy", json!({}));
    run(&mut s, "file.new", json!({}));
    let other = s.doc.add_media(png(200), "png");
    run(&mut s, "edit.paste", json!({}));
    let p = s.doc.para_at(&Pos::body(0, 0)).unwrap();
    assert_eq!(p.objects.len(), 1);
    assert_eq!(p.text, doc4me_doc::para::OBJ.to_string());
    let InlineObject::Image { media, w, h, .. } = &p.objects[0] else { panic!() };
    assert_eq!((*w, *h), (24.0, 12.0));
    assert_eq!(s.doc.media.get(media).unwrap().as_slice(), original);
    assert_eq!(s.doc.media.get(&other).unwrap().as_slice(), png(200));
    let d = roundtrip(&s.doc);
    assert!(d.media.values().any(|b| b.as_slice() == original));
}

#[test]
fn text_boxes_are_independent_after_copy_and_survive_closing_the_source() {
    let mut d = Document::new();
    let id = d.add_part(PartKind::TextBox, vec![para_block(Paragraph::with_text("กล่องต้นฉบับ", Default::default()))]);
    d.para_mut(StoryRef::Body, &Path::top(0)).unwrap().insert_object(0, shape(Some(id)), &CharProps::default()).unwrap();
    let mut s = Session::new(d);
    pick(&mut s, Pos::body(0, 0));
    run(&mut s, "edit.copy", json!({}));
    s.sel = Selection::caret(Pos::body(0, 3));
    run(&mut s, "edit.paste", json!({}));
    let ids: Vec<_> = s.doc.para_at(&Pos::body(0, 0)).unwrap().objects.iter().flat_map(InlineObject::text_boxes).collect();
    assert_eq!(ids.len(), 2);
    assert_ne!(ids[0], ids[1]);
    s.doc.para_mut(StoryRef::Part(ids[1]), &Path::top(0)).unwrap().insert_text(0, "แก้ไข", &CharProps::default()).unwrap();
    assert_eq!(s.doc.plain_text(StoryRef::Part(ids[0])), "กล่องต้นฉบับ");
    run(&mut s, "file.new", json!({}));
    let foreign = s.doc.add_part(PartKind::TextBox, vec![para_block(Paragraph::with_text("unrelated", Default::default()))]);
    run(&mut s, "edit.paste", json!({}));
    let copied = s.doc.para_at(&Pos::body(0, 0)).unwrap().objects[0].text_boxes().into_iter().next().unwrap();
    assert_ne!(copied, foreign);
    assert_eq!(s.doc.plain_text(StoryRef::Part(copied)), "กล่องต้นฉบับ");
    let d = roundtrip(&s.doc);
    assert!(
        d.parts.values().filter(|p| p.kind == PartKind::TextBox).any(|p| p.blocks.iter().any(|b| b.as_para().is_some_and(|p| p.text == "กล่องต้นฉบับ")))
    );
}

#[test]
fn multiple_objects_copy_and_cut_include_every_pick_without_intervening_text() {
    let mut d = Document::from_text("between");
    let pic = picture(&mut d, 30);
    let p = d.para_mut(StoryRef::Body, &Path::top(0)).unwrap();
    p.insert_object(0, shape(None), &CharProps::default()).unwrap();
    p.insert_object(p.len(), pic, &CharProps::default()).unwrap();
    let mut s = Session::new(d);
    pick(&mut s, Pos::body(0, 0));
    s.also_selected = vec![Pos::body(0, 10)];
    run(&mut s, "edit.copy", json!({}));
    assert_eq!(s.clipboard.as_ref().unwrap().blocks.iter().filter_map(|b| b.as_para()).map(|p| p.objects.len()).sum::<usize>(), 2);
    assert!(!s.clipboard_text.contains("between"));
    let before = s.doc.clone();
    run(&mut s, "edit.cut", json!({}));
    assert_eq!(s.doc.plain_text(StoryRef::Body), "between");
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(s.doc, before);
}

#[test]
fn converting_between_table_and_text_preserves_pictures_and_character_formatting() {
    let mut s = fixture(1, 2);
    let pic = picture(&mut s.doc, 50);
    s.doc.para_mut(StoryRef::Body, &at(0, 0).path).unwrap().insert_object(0, pic, &CharProps { bold: Some(true), ..Default::default() }).unwrap();
    run(&mut s, "table.toText", json!({"separator":"tab"}));
    assert!(
        s.doc
            .para_paths(StoryRef::Body)
            .iter()
            .any(|p| s.doc.para(StoryRef::Body, p).is_some_and(|p| p.objects.iter().any(|o| matches!(o, InlineObject::Image { .. })))),
        "conversion dropped the picture"
    );
    s.sel = Selection { anchor: Pos::body(1, 0), focus: Pos::body(1, s.doc.para_at(&Pos::body(1, 0)).unwrap().len()) };
    run(&mut s, "table.fromText", json!({"separator":"tab"}));
    assert_eq!(table(&s).grid.len(), 2);
    assert_eq!(table(&s).rows[0].cells[0].blocks[0].as_para().unwrap().objects.len(), 1);
    assert!(table(&s).rows[0].cells[0].blocks[0].as_para().unwrap().runs.iter().any(|r| r.props.bold == Some(true)));
}

fn rectangle(s: &mut Session, r0: usize, c0: usize, r1: usize, c1: usize) {
    s.table_selection = None;
    s.sel = Selection { anchor: at(r0, c0), focus: Pos { off: 4, ..at(r1, c1) } };
}

#[test]
fn insert_above_below_left_and_right_respect_the_whole_selected_rectangle() {
    for (id, below) in [("table.insertRowAbove", false), ("table.insertRowBelow", true)] {
        let mut s = fixture(3, 3);
        rectangle(&mut s, 0, 0, 1, 2);
        let before = s.doc.clone();
        let depth = s.undo_depth();
        run(&mut s, id, json!({}));
        assert_eq!(table(&s).rows.len(), 5);
        let at = if below { 2 } else { 0 };
        assert_eq!(cell_text(table(&s), at, 0), "");
        assert_eq!(cell_text(table(&s), at + 1, 0), "");
        assert_eq!(cell_text(table(&s), if below { 0 } else { 2 }, 0), "r0c0");
        assert_eq!(cell_text(table(&s), 4, 0), "r2c0");
        assert_eq!(s.undo_depth(), depth + 1);
        run(&mut s, "edit.undo", json!({}));
        assert_eq!(s.doc, before);
    }
    for (id, right) in [("table.insertColumnLeft", false), ("table.insertColumnRight", true)] {
        let mut s = fixture(2, 3);
        rectangle(&mut s, 0, 0, 1, 1);
        let before = s.doc.clone();
        run(&mut s, id, json!({}));
        let t = table(&s);
        assert_eq!(t.grid.len(), 5);
        assert!((t.grid.iter().sum::<f32>() - 300.0).abs() < 0.01);
        for r in 0..2 {
            let at = if right { 2 } else { 0 };
            assert_eq!(cell_text(t, r, at), "");
            assert_eq!(cell_text(t, r, at + 1), "");
            assert_eq!(cell_text(t, r, if right { 0 } else { 2 }), label(r, 0));
            for (c, cell) in t.rows[r].cells.iter().enumerate() {
                assert!((cell.props.width.unwrap() - t.grid[c]).abs() < 0.01);
            }
        }
        run(&mut s, "edit.undo", json!({}));
        assert_eq!(s.doc, before);
    }
}

#[test]
fn deleting_multiple_rows_cells_or_the_last_cell_never_leaves_an_empty_table() {
    let mut s = fixture(3, 3);
    rectangle(&mut s, 0, 0, 1, 2);
    let before = s.doc.clone();
    run(&mut s, "table.deleteRow", json!({}));
    assert_eq!(table(&s).rows.len(), 1);
    assert_eq!(cell_text(table(&s), 0, 0), "r2c0");
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(s.doc, before);
    rectangle(&mut s, 0, 0, 0, 1);
    run(&mut s, "table.deleteCells", json!({}));
    assert_eq!(table(&s).rows[0].cells.len(), 1);
    assert_eq!(cell_text(table(&s), 0, 0), "r0c2");
    assert_eq!(cell_text(table(&s), 1, 0), "r1c0");
    run(&mut s, "edit.undo", json!({}));
    focus(&mut s, 0, 0);
    run(&mut s, "table.selectTable", json!({}));
    run(&mut s, "table.deleteCells", json!({}));
    assert!(s.doc.body.iter().all(|b| b.as_table().is_none()));
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(s.doc, before);
    let mut single = fixture(1, 1);
    run(&mut single, "table.deleteCells", json!({}));
    assert!(single.doc.body.iter().all(|b| b.as_table().is_none()));
}

#[test]
fn distribution_uses_selected_columns_and_natural_row_heights() {
    let mut s = fixture(3, 3);
    s.doc.table_mut(StoryRef::Body, &Path::top(1)).unwrap().grid = vec![40.0, 100.0, 160.0];
    rectangle(&mut s, 0, 0, 2, 1);
    run(&mut s, "table.distributeColumns", json!({}));
    assert_eq!(table(&s).grid, vec![70.0, 70.0, 160.0]);
    s.doc.para_mut(StoryRef::Body, &at(1, 0).path).unwrap().insert_text(0, "one\ntwo\nthree\nfour\nfive\n", &CharProps::default()).unwrap();
    rectangle(&mut s, 0, 0, 1, 2);
    run(&mut s, "table.distributeRows", json!({}));
    let t = table(&s);
    assert_eq!(t.rows[0].props.height, t.rows[1].props.height);
    assert!(t.rows[0].props.height.unwrap() > 60.0);
    assert_eq!(t.rows[2].props.height, None);
    let layout = s.layout();
    let mut heights = BTreeMap::<usize, f32>::new();
    for it in &layout.pages[0].items {
        if let doc4me_layout::Placed::Cell { table, row, rect, cell: 0, .. } = it
            && *table == Path::top(1)
        {
            heights.insert(*row, rect.h);
        }
    }
    assert!((heights[&0] - heights[&1]).abs() < 0.1, "rows still differ: {heights:?}");
    let before = s.doc.clone();
    assert!(s.run("table.autofit", &json!({"mode":"wrong"})).is_err());
    assert_eq!(s.doc, before);
}

#[test]
fn merged_cell_widths_cover_the_grid_and_copied_continuations_keep_their_content() {
    let mut s = fixture(3, 3);
    s.doc.table_mut(StoryRef::Body, &Path::top(1)).unwrap().merge(0, 1, 0, 1);
    focus(&mut s, 0, 0);
    run(&mut s, "table.selectCell", json!({}));
    run(&mut s, "table.columnWidth", json!({"width":40}));
    assert_eq!(table(&s).grid, vec![40.0, 40.0, 100.0]);
    focus(&mut s, 1, 0);
    run(&mut s, "table.selectRow", json!({}));
    run(&mut s, "edit.copy", json!({}));
    let copied = s.clipboard.as_ref().unwrap().blocks[0].as_table().unwrap();
    assert_eq!(copied.rows[0].cells[0].props.vmerge, VMerge::Restart);
    assert!(cell_text(copied, 0, 0).contains("r0c0"));
    assert_eq!(table(&s).rows[1].cells[0].props.vmerge, VMerge::Continue);
    assert!(s.run("table.splitTable", &json!({})).is_err());
    run(&mut s, "file.new", json!({}));
    run(&mut s, "edit.paste", json!({}));
    let d = roundtrip(&s.doc);
    let t = d.body.iter().find_map(|b| b.as_table()).unwrap();
    assert!(cell_text(t, 0, 0).contains("r0c0"));
}

#[test]
fn cell_alignment_and_table_margins_keep_other_cells_and_roundtrip() {
    for (value, h, v) in [
        ("topLeft", Align::Left, VAlign::Top),
        ("topCenter", Align::Center, VAlign::Top),
        ("topRight", Align::Right, VAlign::Top),
        ("centerLeft", Align::Left, VAlign::Center),
        ("center", Align::Center, VAlign::Center),
        ("centerRight", Align::Right, VAlign::Center),
        ("bottomLeft", Align::Left, VAlign::Bottom),
        ("bottomCenter", Align::Center, VAlign::Bottom),
        ("bottomRight", Align::Right, VAlign::Bottom),
    ] {
        let mut s = fixture(2, 2);
        run(&mut s, "table.selectColumn", json!({}));
        run(&mut s, "table.cellAlign", json!({"value":value}));
        for row in &table(&s).rows {
            assert_eq!(row.cells[0].props.valign, v);
            assert_eq!(row.cells[0].blocks[0].as_para().unwrap().props.align, Some(h));
            assert_eq!(row.cells[1].blocks[0].as_para().unwrap().props.align, None);
        }
        run(&mut s, "table.cellMargins", json!({"top":3,"left":4,"bottom":5,"right":6}));
        let back = roundtrip(&s.doc);
        assert_eq!(back.table(StoryRef::Body, &Path::top(1)).unwrap().props.cell_margins, Some([3.0, 4.0, 5.0, 6.0]));
    }
}

#[test]
fn table_formula_fields_compute_supported_operations_and_reject_invalid_or_overflowing_input() {
    for (formula, expected) in [
        ("=SUM(ABOVE)", "6"),
        ("=AVERAGE(ABOVE)", "3"),
        ("=COUNT(ABOVE)", "2"),
        ("=MIN(ABOVE)", "2"),
        ("=MAX(ABOVE)", "4"),
        ("=PRODUCT(ABOVE)", "8"),
        ("=SUM(LEFT)", "0"),
    ] {
        let mut s = fixture(3, 2);
        for (r, text) in [(0, "2"), (1, "4"), (2, "")] {
            s.doc.table_mut(StoryRef::Body, &Path::top(1)).unwrap().rows[r].cells[0].blocks =
                vec![para_block(Paragraph::with_text(text, Default::default()))];
        }
        focus(&mut s, 2, 0);
        assert_eq!(run(&mut s, "table.formula", json!({"formula":formula}))["result"], expected);
        let d = roundtrip(&s.doc);
        assert!(
            d.para(StoryRef::Body, &at(2, 0).path)
                .unwrap()
                .objects
                .iter()
                .any(|o| matches!(o, InlineObject::Field { instr, result, .. } if instr == formula && result == expected))
        );
        let before = s.doc.clone();
        assert!(s.run("table.formula", &json!({"formula":"=EXEC(LEFT)"})).is_err());
        assert_eq!(s.doc, before);
    }
}

fn laid_cell(s: &mut Session, row: usize, cell: usize) -> doc4me_geom::Rect {
    s.layout()
        .pages
        .iter()
        .flat_map(|p| &p.items)
        .find_map(|it| match it {
            doc4me_layout::Placed::Cell { table, row: r, cell: c, rect, .. } if *table == Path::top(1) && *r == row && *c == cell => Some(*rect),
            _ => None,
        })
        .unwrap()
}

#[test]
fn border_painter_changes_only_hit_edges_in_one_undo_step() {
    let mut s = fixture(2, 3);
    let a = laid_cell(&mut s, 0, 0);
    let b = laid_cell(&mut s, 0, 1);
    let before = s.doc.clone();
    let depth = s.undo_depth();
    run(
        &mut s,
        "table.borderPainter",
        json!({"page":0,"x0":a.x+2.0,"y0":a.y,"x1":b.right()-2.0,"y1":a.y,"style":"triple","width":2,"color":"123456"}),
    );
    for c in 0..2 {
        let edge = table(&s).rows[0].cells[c].props.borders.unwrap().top.unwrap();
        assert_eq!(edge.style, BorderStyle::Triple);
        assert_eq!(edge.color, Some(Rgb(0x12, 0x34, 0x56)));
    }
    assert_eq!(table(&s).rows[0].cells[2].props.borders, None);
    assert_eq!(table(&s).rows[1].cells[0].props.borders, None);
    assert_eq!(s.undo_depth(), depth + 1);
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(s.doc, before);
    run(&mut s, "table.borderPainter", json!({"page":0,"x0":a.right(),"y0":a.y+a.h/2.0,"x1":a.right(),"y1":a.y+a.h/2.0,"style":"dotDash"}));
    assert_eq!(table(&s).rows[0].cells[0].props.borders.unwrap().right.unwrap().style, BorderStyle::DotDash);
    assert_eq!(table(&s).rows[0].cells[1].props.borders.unwrap().left.unwrap().style, BorderStyle::DotDash);
    let before = s.doc.clone();
    for params in [
        json!({"page":999,"x0":1,"y0":1,"x1":1,"y1":1}),
        json!({"page":0,"x0":1,"y0":1,"x1":1,"y1":1}),
        json!({"page":0,"x0":a.x,"y0":a.y,"x1":a.x+40.0,"y1":a.y+40.0}),
    ] {
        assert!(s.run("table.borderPainter", &params).is_err());
        assert_eq!(s.doc, before);
    }
    let mut rtl = fixture(2, 3);
    run(&mut rtl, "table.rtl", json!({"value":true}));
    let r = laid_cell(&mut rtl, 0, 1);
    run(
        &mut rtl,
        "table.borderPainter",
        json!({"page":0,"x0":r.right(),"y0":r.y+r.h/2.0,"x1":r.right(),"y1":r.y+r.h/2.0,"style":"dotDash","color":"FF00AA"}),
    );
    assert_eq!(table(&rtl).rows[0].cells[1].props.borders.unwrap().left.unwrap().style, BorderStyle::DotDash);
    assert_eq!(table(&rtl).rows[0].cells[0].props.borders.unwrap().right.unwrap().style, BorderStyle::DotDash);
    assert_eq!(table(&rtl).rows[0].cells[2].props.borders, None);
    assert!(rtl.layout().pages[0].items.iter().any(|it|matches!(it,doc4me_layout::Placed::Rule { x0,x1,border,.. } if (*x0-r.right()).abs()<0.01 && (*x1-r.right()).abs()<0.01 && border.color==Some(Rgb(255,0,170)))));
}

#[test]
fn groups_canvases_and_graphic_picture_resources_survive_cross_document_copy() {
    use doc4me_doc::graphic::{Graphic, GraphicItem};
    use doc4me_doc::para::{CanvasStyle, GroupChild};
    for canvas in [None, Some(CanvasStyle { fill: Some(Rgb(20, 30, 40)), ..Default::default() })] {
        let mut d = Document::new();
        let id = d.add_part(PartKind::TextBox, vec![para_block(Paragraph::with_text("nested box", Default::default()))]);
        let pic = picture(&mut d, 77);
        let InlineObject::Image { media, .. } = &pic else { panic!() };
        let graphic = InlineObject::Graphic {
            w: 30.0,
            h: 20.0,
            alt: "image in graphic".into(),
            float: Default::default(),
            graphic: Arc::new(Graphic {
                items: vec![GraphicItem::Image { rect: [0.0, 0.0, 30.0, 20.0], media: media.clone() }],
                ..Default::default()
            }),
        };
        let group = InlineObject::Group {
            w: 140.0,
            h: 100.0,
            ch_w: 140.0,
            ch_h: 100.0,
            float: Default::default(),
            canvas,
            children: vec![
                GroupChild { x: 0.0, y: 0.0, obj: pic },
                GroupChild { x: 30.0, y: 0.0, obj: shape(Some(id)) },
                GroupChild { x: 80.0, y: 0.0, obj: graphic },
            ],
        };
        d.para_mut(StoryRef::Body, &Path::top(0)).unwrap().insert_object(0, group, &Default::default()).unwrap();
        let mut s = Session::new(d);
        pick(&mut s, Pos::body(0, 0));
        run(&mut s, "edit.copy", json!({}));
        run(&mut s, "file.new", json!({}));
        let occupied = s.doc.add_media(png(99), "png");
        run(&mut s, "edit.paste", json!({}));
        let InlineObject::Group { children, canvas: got, .. } = &s.doc.para_at(&Pos::body(0, 0)).unwrap().objects[0] else { panic!() };
        assert_eq!(*got, canvas);
        assert_eq!(children.len(), 3);
        let InlineObject::Graphic { graphic, .. } = &children[2].obj else { panic!() };
        let GraphicItem::Image { media, .. } = &graphic.items[0] else { panic!() };
        assert_eq!(s.doc.media[media].as_slice(), png(77));
        assert_eq!(s.doc.media[&occupied].as_slice(), png(99));
        let box_id = children[1].obj.text_boxes()[0];
        assert_eq!(s.doc.plain_text(StoryRef::Part(box_id)), "nested box");
        let d = roundtrip(&s.doc);
        assert!(d.media.values().any(|b| b.as_slice() == png(77)));
        assert!(d.parts.values().any(|p| p.blocks.iter().any(|b| b.as_para().is_some_and(|p| p.text == "nested box"))));
    }
}

#[test]
fn editable_charts_and_smart_art_remain_editable_after_copy_and_reopen() {
    for (command, params) in
        [("insert.chart", json!({"type":"line","title":"copied chart"})), ("insert.smartArt", json!({"layout":"process","items":["first","second"]}))]
    {
        let mut s = Session::new(Document::new());
        run(&mut s, command, params);
        pick(&mut s, Pos::body(0, 0));
        let InlineObject::Graphic { graphic: expected, .. } = &s.doc.para_at(&Pos::body(0, 0)).unwrap().objects[0] else { panic!() };
        let expected = expected.clone();
        run(&mut s, "edit.copy", json!({}));
        run(&mut s, "file.new", json!({}));
        run(&mut s, "edit.paste", json!({}));
        let d = roundtrip(&s.doc);
        let InlineObject::Graphic { graphic, .. } = &d.para_at(&Pos::body(0, 0)).unwrap().objects[0] else { panic!() };
        assert_eq!(graphic.chart, expected.chart);
        assert_eq!(graphic.smart_art, expected.smart_art);
    }
}

#[test]
fn imported_chart_package_graphs_are_independent_and_case_insensitive_collisions_do_not_overwrite() {
    use doc4me_doc::graphic::{EMBEDDED_PARTS, Embedded, EmbeddedManifest, EmbeddedPart, EmbeddedRef};
    let mut original = Session::new(Document::new());
    run(&mut original, "insert.chart", json!({"type":"column","title":"original"}));
    let bytes = doc4me_docx::write(&original.doc).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&bytes)).unwrap();
    let mut chart_bytes = Vec::new();
    std::io::Read::read_to_end(&mut zip.by_name("word/charts/chart1.xml").unwrap(), &mut chart_bytes).unwrap();
    let mut d = original.doc;
    let InlineObject::Graphic { graphic, .. } = &mut d.para_mut(StoryRef::Body, &Path::top(0)).unwrap().objects[0] else { panic!() };
    let g = Arc::make_mut(graphic);
    g.chart = None; // An imported read-only chart uses its original OOXML graph.
    let root = "word/charts/chart1.xml".to_string();
    g.source = Some(Arc::new(Embedded {
        xml: r#"<a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart r:id="rChart"/></a:graphicData></a:graphic>"#.into(),
        refs:vec![EmbeddedRef { id:"rChart".into(),kind:"http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart".into(),target:root.clone(),external:false }],
    }));
    let mut manifest = EmbeddedManifest::default();
    manifest.parts.insert(
        root.clone(),
        EmbeddedPart { content_type: "application/vnd.openxmlformats-officedocument.drawingml.chart+xml".into(), rels: vec![] },
    );
    d.passthrough.insert(root.clone(), Arc::new(chart_bytes));
    let book = "word/embeddings/book1.xlsx";
    manifest.parts.get_mut(&root).unwrap().rels.push(EmbeddedRef {
        id: "rWorkbook".into(),
        kind: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/package".into(),
        target: book.into(),
        external: false,
    });
    manifest
        .parts
        .insert(book.into(), EmbeddedPart { content_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".into(), rels: vec![] });
    d.passthrough.insert(book.into(), Arc::new(b"source workbook bytes".to_vec()));
    d.passthrough.insert(EMBEDDED_PARTS.into(), Arc::new(manifest.to_bytes()));
    let root_bytes = d.passthrough[&root].clone();
    let mut s = Session::new(d);
    pick(&mut s, Pos::body(0, 0));
    run(&mut s, "edit.copy", json!({}));
    run(&mut s, "file.new", json!({}));
    s.doc.passthrough.insert(root.clone(), Arc::new(b"destination content".to_vec()));
    s.doc.passthrough.insert("word/embeddings/partpaste0.xml".into(), Arc::new(b"occupied different case".to_vec()));
    run(&mut s, "edit.paste", json!({}));
    assert_eq!(s.doc.passthrough[&root].as_slice(), b"destination content");
    assert_eq!(s.doc.passthrough["word/embeddings/partpaste0.xml"].as_slice(), b"occupied different case");
    let InlineObject::Graphic { graphic, .. } = &s.doc.para_at(&Pos::body(0, 0)).unwrap().objects[0] else { panic!() };
    let target = &graphic.source.as_ref().unwrap().refs.iter().find(|r| !r.external).unwrap().target;
    assert_ne!(target, &root);
    assert_eq!(&s.doc.passthrough[target], &root_bytes);
    let imported = EmbeddedManifest::parse(&s.doc.passthrough[EMBEDDED_PARTS]);
    let related = &imported.parts[target].rels.iter().find(|r| r.id == "rWorkbook").unwrap().target;
    assert_eq!(s.doc.passthrough[related].as_slice(), b"source workbook bytes");
    let bytes = doc4me_docx::write(&s.doc).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&bytes)).unwrap();
    let mut found = false;
    for i in 0..zip.len() {
        let mut part = zip.by_index(i).unwrap();
        if part.name().unwrap().ends_with(".xlsx") {
            let mut data = Vec::new();
            std::io::Read::read_to_end(&mut part, &mut data).unwrap();
            found |= data == b"source workbook bytes";
        }
    }
    assert!(found, "the dependency graph must reach the saved package");
    assert!(doc4me_docx::read(&bytes).is_ok());
}

#[test]
fn copied_equations_ruby_fields_and_notes_keep_their_models_and_independent_stories() {
    use doc4me_doc::para::NoteKind;
    let mut d = Document::new();
    let foot = d.add_part(PartKind::Footnote, vec![para_block(Paragraph::with_text("footnote contents", Default::default()))]);
    let end = d.add_part(PartKind::Endnote, vec![para_block(Paragraph::with_text("endnote contents", Default::default()))]);
    let p = d.para_mut(StoryRef::Body, &Path::top(0)).unwrap();
    for o in [
        InlineObject::Equation { linear: "a^2+b^2".into(), display: false, math: Default::default() },
        InlineObject::Ruby {
            base: "漢".into(),
            ruby: "かん".into(),
            align: Default::default(),
            size: 6.0,
            raise: 10.0,
            base_size: 12.0,
            lang: "ja-JP".into(),
            props: Default::default(),
        },
        InlineObject::Field { instr: "DATE".into(), result: "saved date".into(), locked: true, code: false, form: None },
        InlineObject::NoteRef { kind: NoteKind::Footnote, id: foot, custom: String::new() },
        InlineObject::NoteRef { kind: NoteKind::Endnote, id: end, custom: String::new() },
    ] {
        p.insert_object(p.len(), o, &Default::default()).unwrap();
    }
    let mut s = Session::new(d);
    s.sel = Selection { anchor: Pos::body(0, 0), focus: Pos::body(0, 15) };
    run(&mut s, "edit.copy", json!({}));
    run(&mut s, "file.new", json!({}));
    let occupied = s.doc.add_part(PartKind::Footnote, vec![para_block(Paragraph::with_text("unrelated", Default::default()))]);
    run(&mut s, "edit.paste", json!({}));
    assert_eq!(s.doc.plain_text(StoryRef::Part(occupied)), "unrelated");
    let p = s.doc.para_at(&Pos::body(0, 0)).unwrap();
    assert_eq!(p.objects.len(), 5);
    let InlineObject::NoteRef { id, .. } = p.objects[3] else { panic!() };
    assert_ne!(id, occupied);
    assert_eq!(s.doc.plain_text(StoryRef::Part(id)), "footnote contents");
    let d = roundtrip(&s.doc);
    let p = d.para_at(&Pos::body(0, 0)).unwrap();
    assert!(p.objects.iter().any(|o| matches!(o,InlineObject::Equation { linear,.. } if linear.contains("a"))));
    assert!(p.objects.iter().any(|o| matches!(o,InlineObject::Ruby { base,ruby,.. } if base=="漢" && ruby=="かん")));
    assert!(
        d.parts
            .values()
            .any(|p| p.kind == PartKind::Endnote && p.blocks.iter().any(|b| b.as_para().is_some_and(|p| p.text.contains("endnote contents"))))
    );
}

#[test]
fn clipboard_history_charges_picture_and_text_box_payloads() {
    for textbox in [false, true] {
        let mut d = Document::new();
        let object = if textbox {
            let id = d.add_part(PartKind::TextBox, vec![para_block(Paragraph::with_text(&"x".repeat(5 << 20), Default::default()))]);
            shape(Some(id))
        } else {
            let mut pic = picture(&mut d, 33);
            let InlineObject::Image { media, .. } = &mut pic else { panic!() };
            d.media.insert(media.clone(), Arc::new(vec![7; 5 << 20]));
            pic
        };
        d.para_mut(StoryRef::Body, &Path::top(0)).unwrap().insert_object(0, object, &Default::default()).unwrap();
        let mut s = Session::new(d);
        pick(&mut s, Pos::body(0, 0));
        run(&mut s, "edit.copy", json!({}));
        assert!(s.clipboard.is_some(), "the live clipboard still accepts a large item");
        assert!(s.clip_history.is_empty(), "large object payload leaked into bounded history");
    }
}

#[test]
fn rich_paste_rejects_missing_resources_atomically_and_story_id_exhaustion_never_overwrites() {
    let mut source = Document::new();
    let pic = picture(&mut source, 9);
    source.para_mut(StoryRef::Body, &Path::top(0)).unwrap().insert_object(0, pic, &Default::default()).unwrap();
    let mut frag = source.body_fragment();
    frag.media.clear();
    let mut destination = Document::new();
    destination.add_media(png(99), "png");
    let before = destination.clone();
    assert!(destination.insert_fragment(&Pos::body(0, 0), &frag).is_err());
    assert_eq!(destination, before);
    let id = source.add_part(PartKind::TextBox, vec![para_block(Paragraph::with_text("copy", Default::default()))]);
    source.body = vec![para_block(Paragraph::new())];
    source.para_mut(StoryRef::Body, &Path::top(0)).unwrap().insert_object(0, shape(Some(id)), &Default::default()).unwrap();
    destination.parts.insert(
        u32::MAX,
        doc4me_doc::Part {
            kind: PartKind::TextBox,
            blocks: vec![para_block(Paragraph::with_text("keep", Default::default()))],
            body: Default::default(),
        },
    );
    destination.insert_fragment(&Pos::body(0, 0), &source.body_fragment()).unwrap();
    assert_eq!(destination.plain_text(StoryRef::Part(u32::MAX)), "keep");
    assert!(destination.parts.iter().any(|(id, p)| *id != u32::MAX && p.blocks.iter().any(|b| b.as_para().is_some_and(|p| p.text == "copy"))));
}

#[test]
fn formatting_a_selected_column_never_formats_intervening_cells_and_empty_marks_are_included() {
    let mut s = fixture(3, 3);
    s.doc.table_mut(StoryRef::Body, &Path::top(1)).unwrap().rows[1].cells[1].blocks = vec![para_block(Paragraph::new())];
    focus(&mut s, 0, 1);
    run(&mut s, "table.selectColumn", json!({}));
    run(&mut s, "format.bold", json!({}));
    run(&mut s, "para.align", json!({"value":"center"}));
    for (r, row) in table(&s).rows.iter().enumerate() {
        for (c, cell) in row.cells.iter().enumerate() {
            let p = cell.blocks[0].as_para().unwrap();
            assert_eq!(p.props.align, (c == 1).then_some(Align::Center), "{r},{c}");
            assert_eq!(p.mark.bold, (c == 1).then_some(true), "mark {r},{c}");
            for run in &p.runs {
                assert_eq!(run.props.bold, (c == 1).then_some(true), "run {r},{c}");
            }
        }
    }
    run(&mut s, "format.changeCase", json!({"mode":"upper"}));
    for r in [0, 2] {
        assert_eq!(cell_text(table(&s), r, 1), format!("R{r}C1"));
        assert_eq!(cell_text(table(&s), r, 0), format!("r{r}c0"));
        assert_eq!(cell_text(table(&s), r, 2), format!("r{r}c2"));
    }
    focus(&mut s, 0, 1);
    run(&mut s, "table.lastCellInColumn", json!({"extend":true}));
    assert_eq!(s.selected_table_cells().unwrap().cells, BTreeSet::from([(0, 1), (1, 1), (2, 1)]));
    run(&mut s, "format.italic", json!({"value":true}));
    for row in &table(&s).rows {
        for (c, cell) in row.cells.iter().enumerate() {
            assert_eq!(cell.blocks[0].as_para().unwrap().mark.italic, (c == 1).then_some(true));
        }
    }
}

#[test]
fn table_clipboard_imports_conflicting_styles_lists_and_source_defaults() {
    use doc4me_doc::numbering::ListKind;
    use doc4me_doc::styles::{Style, StyleKind, TableStyleParts};
    let mut s = fixture(1, 1);
    let source_fill = Rgb(12, 34, 56);
    Arc::make_mut(&mut s.doc.styles).default_chr.size = Some(19.0);
    Arc::make_mut(&mut s.doc.styles).upsert(Style {
        id: "Same".into(),
        name: "Same".into(),
        kind: StyleKind::Table,
        table: Some(TableStyleParts { fill: Some(source_fill), ..Default::default() }),
        ..Default::default()
    });
    let list = Arc::make_mut(&mut s.doc.numbering).add_list(ListKind::ThaiNumbers);
    let t = s.doc.table_mut(StoryRef::Body, &Path::top(1)).unwrap();
    t.props.style = Some("Same".into());
    t.rows[0].cells[0].blocks[0] = para_block(Paragraph {
        props: doc4me_doc::ParaProps { numbering: Some(doc4me_doc::props::NumRef { num: list, level: 0 }), ..Default::default() },
        ..Paragraph::with_text("ไทย", Default::default())
    });
    run(&mut s, "table.selectTable", json!({}));
    run(&mut s, "edit.copy", json!({}));
    run(&mut s, "file.new", json!({}));
    let destination_fill = Rgb(99, 88, 77);
    Arc::make_mut(&mut s.doc.styles).upsert(Style {
        id: "Same".into(),
        name: "Same".into(),
        kind: StyleKind::Table,
        table: Some(TableStyleParts { fill: Some(destination_fill), ..Default::default() }),
        ..Default::default()
    });
    let other_list = Arc::make_mut(&mut s.doc.numbering).add_list(ListKind::Numbered);
    run(&mut s, "edit.paste", json!({}));
    let t = s.doc.body[0].as_table().unwrap();
    let style = t.props.style.as_ref().unwrap();
    assert_ne!(style, "Same");
    assert_eq!(s.doc.styles.get("Same").unwrap().table.as_ref().unwrap().fill, Some(destination_fill));
    assert_eq!(s.doc.styles.get(style).unwrap().table.as_ref().unwrap().fill, Some(source_fill));
    let p = t.rows[0].cells[0].blocks[0].as_para().unwrap();
    let copied_list = p.props.numbering.unwrap().num;
    assert_ne!(copied_list, other_list);
    assert_eq!(s.doc.numbering.level(copied_list, 0).unwrap().format, doc4me_doc::section::NumFormat::ThaiNumbers);
    // Normal has an explicit size; a source default still belongs to its imported root style.
    let imported = p.props.style.as_deref().unwrap_or("Normal");
    assert!(s.doc.styles.get(imported).is_some());
    let d = roundtrip(&s.doc);
    assert_eq!(d.body[0].as_table().unwrap().props.style.as_deref(), Some(style.as_str()));
}

#[test]
fn ole_preview_and_original_payload_survive_copy_collision_and_docx_reopen() {
    use doc4me_doc::graphic::{EMBEDDED_PARTS, Embedded, EmbeddedManifest, EmbeddedPart, EmbeddedRef};
    let mut d = Document::new();
    let mut object = picture(&mut d, 63);
    let preview = "word/media/preview.png";
    let payload = "word/embeddings/object.bin";
    let rel = |id: &str, kind: &str, target: &str| EmbeddedRef {
        id: id.into(),
        kind: format!("http://schemas.openxmlformats.org/officeDocument/2006/relationships/{kind}"),
        target: target.into(),
        external: false,
    };
    if let InlineObject::Image { ole, .. } = &mut object {
        *ole=Some(Arc::new(Embedded {
            xml:r##"<w:object xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><v:shape id="originalObject" style="width:24pt;height:12pt"><v:imagedata r:id="rPreview"/></v:shape><o:OLEObject Type="Embed" ProgID="Package" ShapeID="originalObject" r:id="rObject"/></w:object>"##.into(),
            refs:vec![rel("rPreview","image",preview),rel("rObject","oleObject",payload)],
        }));
    }
    let manifest = EmbeddedManifest {
        parts: BTreeMap::from([
            (preview.into(), EmbeddedPart { content_type: "image/png".into(), rels: vec![] }),
            (payload.into(), EmbeddedPart { content_type: "application/vnd.openxmlformats-officedocument.oleObject".into(), rels: vec![] }),
        ]),
    };
    d.passthrough.insert(EMBEDDED_PARTS.into(), Arc::new(manifest.to_bytes()));
    d.passthrough.insert(preview.into(), Arc::new(png(63)));
    d.passthrough.insert(payload.into(), Arc::new(b"opaque original OLE payload".to_vec()));
    d.para_mut(StoryRef::Body, &Path::top(0)).unwrap().insert_object(0, object, &Default::default()).unwrap();
    let mut s = Session::new(d);
    pick(&mut s, Pos::body(0, 0));
    run(&mut s, "edit.copy", json!({}));
    run(&mut s, "file.new", json!({}));
    s.doc.add_media(png(99), "png");
    s.doc.passthrough.insert(payload.into(), Arc::new(b"destination OLE".to_vec()));
    run(&mut s, "edit.paste", json!({}));
    assert_eq!(s.doc.passthrough[payload].as_slice(), b"destination OLE");
    let d = roundtrip(&s.doc);
    assert!(d.passthrough.values().any(|b| b.as_slice() == b"opaque original OLE payload"));
    assert!(matches!(&d.para_at(&Pos::body(0, 0)).unwrap().objects[0], InlineObject::Image { ole: Some(_), .. }));
    assert!(d.media.values().any(|b| b.as_slice() == png(63)));
}

#[test]
fn merge_and_split_preserve_content_grid_and_vertical_merge_structure() {
    let mut s = fixture(3, 3);
    rectangle(&mut s, 0, 0, 1, 1);
    let before = s.doc.clone();
    run(&mut s, "table.merge", json!({}));
    assert_eq!(table(&s).rows[0].cells[0].span(), 2);
    assert_eq!(table(&s).rows[0].cells[0].props.vmerge, VMerge::Restart);
    for text in ["r0c0", "r0c1", "r1c0", "r1c1"] {
        assert!(cell_text(table(&s), 0, 0).contains(text));
    }
    let merged = s.doc.clone();
    focus(&mut s, 1, 0);
    run(&mut s, "table.split", json!({"columns":2}));
    assert_eq!(table(&s).grid, vec![100.0; 3]);
    for r in 0..2 {
        assert_eq!(table(&s).rows[r].cells.len(), 3);
        for c in 0..2 {
            assert_eq!(table(&s).rows[r].cells[c].span(), 1);
            assert_eq!(table(&s).rows[r].cells[c].props.vmerge, if r == 0 { VMerge::Restart } else { VMerge::Continue });
        }
    }
    assert!(cell_text(table(&s), 0, 0).contains("r1c1"));
    let d = roundtrip(&s.doc);
    assert_eq!(d.table(StoryRef::Body, &Path::top(1)).unwrap().rows[1].cells[1].props.vmerge, VMerge::Continue);
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(s.doc, merged);
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(s.doc, before);
}

#[test]
fn sorting_a_row_selection_keeps_other_rows_and_rejects_vertical_merges_or_bad_columns() {
    let mut s = fixture(5, 2);
    rectangle(&mut s, 1, 0, 3, 1);
    run(&mut s, "table.sort", json!({"column":0,"descending":true}));
    assert_eq!((0..5).map(|r| cell_text(table(&s), r, 0)).collect::<Vec<_>>(), vec!["r0c0", "r3c0", "r2c0", "r1c0", "r4c0"]);
    let before = s.doc.clone();
    assert!(s.run("table.sort", &json!({"column":u64::MAX})).is_err());
    assert_eq!(s.doc, before);
    focus(&mut s, 0, 0);
    rectangle(&mut s, 0, 0, 1, 0);
    run(&mut s, "table.merge", json!({}));
    let before = s.doc.clone();
    assert!(s.run("table.sort", &json!({})).is_err());
    assert_eq!(s.doc, before);
}

#[test]
fn splitting_a_cell_over_unequal_grid_columns_makes_equal_formatted_cells() {
    let mut s = fixture(2, 3);
    let t = s.doc.table_mut(StoryRef::Body, &Path::top(1)).unwrap();
    t.grid = vec![40.0, 80.0, 180.0];
    rectangle(&mut s, 0, 0, 0, 1);
    run(&mut s, "table.merge", json!({}));
    run(&mut s, "table.shading", json!({"color":"123456"}));
    run(&mut s, "table.split", json!({"columns":2}));
    let t = table(&s);
    assert_eq!(t.rows[0].cells.len(), 3);
    assert_eq!(t.rows[1].cells.len(), 3, "other rows must keep their cells");
    for c in &t.rows[0].cells[..2] {
        assert_eq!(c.props.width, Some(60.0));
        assert_eq!(c.props.shading, Some(Rgb(0x12, 0x34, 0x56)));
    }
    assert_eq!(cell_text(t, 1, 1), "r1c1");
    assert!((t.total_width() - 300.0).abs() < 0.01);
    let mut s = fixture(2, 3);
    let t = s.doc.table_mut(StoryRef::Body, &Path::top(1)).unwrap();
    t.rows[1].cells.remove(0);
    t.rows[1].props.grid_before = 1;
    let x = laid_cell(&mut s, 1, 0).x;
    run(&mut s, "table.split", json!({"columns":2}));
    assert_eq!(table(&s).rows[1].props.grid_before, 2);
    assert_eq!(cell_text(table(&s), 1, 0), "r1c1");
    assert!((laid_cell(&mut s, 1, 0).x - x).abs() < 0.01);
}
