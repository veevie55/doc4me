//! The system clipboard: copies carried to other programs and other Doc4Me windows
//! (`clip_transfer`), and Paste reading their HTML and RTF back.
use doc4me_doc::para::{InlineObject, OBJ};
use doc4me_doc::{Block, Document, StoryRef};
use serde_json::{Value, json};

use crate::Session;
use crate::clip_transfer::{self, ExportOptions};

fn run(s: &mut Session, id: &str, v: Value) -> Value {
    s.run(id, &v).unwrap_or_else(|e| panic!("{id}: {e}"))
}

fn png(color: u8) -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    image::RgbaImage::from_pixel(3, 2, image::Rgba([color, 20, 30, 255])).write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

/// The kind of every object in a document's body (tables included), in order.
fn kinds(d: &Document) -> Vec<String> {
    let mut out = Vec::new();
    let mut todo: Vec<&Block> = d.body.iter().map(|b| &**b).rev().collect();
    while let Some(b) = todo.pop() {
        match b {
            Block::Para(p) => {
                for o in &p.objects {
                    let name = match o {
                        InlineObject::Image { .. } => "image",
                        InlineObject::Graphic { graphic, .. } if graphic.chart.is_some() => "chart",
                        InlineObject::Graphic { graphic, .. } if graphic.smart_art.is_some() => "smartArt",
                        InlineObject::Graphic { .. } => "graphic",
                        InlineObject::Shape { story: Some(_), .. } => "textShape",
                        InlineObject::Shape { .. } => "shape",
                        InlineObject::Group { .. } => "group",
                        InlineObject::Equation { .. } => "equation",
                        _ => continue,
                    };
                    out.push(name.to_string());
                }
            }
            Block::Table(t) => {
                out.push("table".into());
                todo.extend(t.rows.iter().flat_map(|r| &r.cells).flat_map(|c| &c.blocks).map(|b| &**b).rev());
            }
        }
    }
    out
}

/// Copy everything in `s`, hand it to the system clipboard (as the desktop app does), and paste
/// that clipboard's HTML into a new Doc4Me in another process (a fresh session).
fn copy_to_another_window(s: &mut Session) -> (clip_transfer::SystemClip, Session) {
    // Out of a text box the insert left the caret in: select the body.
    s.sel = crate::Selection::caret(doc4me_doc::Pos::body(0, 0));
    run(s, "select.all", json!({}));
    let r = run(s, "edit.copy", json!({}));
    assert_eq!(r["copied"], true);
    let clip = clip_transfer::export(s, ExportOptions { rtf: true, png: true }).unwrap();
    let mut other = Session::new(Document::new());
    run(&mut other, "edit.paste", json!({"text": clip.text, "html": clip.html}));
    (clip, other)
}

#[test]
fn every_kind_of_object_reaches_another_doc4me_window_editable() {
    let model = "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nv 0 0 1\nf 1 2 3 4\nf 1 2 5\n";
    let cases: Vec<(&str, Value, &str)> = vec![
        ("insert.picture", json!({"data": crate::cmd::insert::base64_encode(&png(90))}), "image"),
        ("insert.shape", json!({"kind": "star", "fill": "C00000"}), "shape"),
        ("insert.textBox", json!({"text": "กล่องข้อความ box"}), "textShape"),
        ("insert.wordArt", json!({"text": "อักษรศิลป์", "style": "gradient", "transform": "textArchUp"}), "textShape"),
        ("insert.icon", json!({"id": "heart", "color": "C00000", "size": 48}), "image"),
        ("insert.model3d", json!({"data": crate::cmd::insert::base64_encode(model.as_bytes()), "name": "part.obj"}), "image"),
        ("insert.chart", json!({"type": "column", "title": "ยอดขาย"}), "chart"),
        ("insert.smartArt", json!({"layout": "process", "items": ["หนึ่ง", "two"]}), "smartArt"),
        ("insert.equation", json!({"linear": "a^2+b^2=c^2", "edit": false}), "equation"),
        ("insert.table", json!({"rows": 2, "cols": 2}), "table"),
    ];
    for (command, params, kind) in cases {
        let mut s = Session::new(Document::new());
        run(&mut s, command, params);
        s.math = None;
        let expected = kinds(&s.doc);
        assert!(expected.iter().any(|k| k == kind), "{command}: {expected:?}");
        let (clip, other) = copy_to_another_window(&mut s);
        assert!(clip.html.contains(&format!("<!--{} v1 ", clip_transfer::MARKER)), "{command}: the lossless copy");
        assert_eq!(kinds(&other.doc), expected, "{command}: pasted in another window");
        // Pictures bring their bytes along.
        for key in s.doc.media.keys() {
            let bytes = &s.doc.media[key];
            assert!(other.doc.media.values().any(|b| b == bytes), "{command}: picture {key} lost");
        }
    }
}

#[test]
fn text_boxes_wordart_and_tables_keep_their_text_and_formatting_across_windows() {
    let mut s = Session::new(Document::from_text("ก่อน before"));
    run(&mut s, "select.all", json!({}));
    run(&mut s, "format.bold", json!({}));
    run(&mut s, "caret.docEnd", json!({}));
    run(&mut s, "insert.textBox", json!({"text": "ข้อความในกล่อง"}));
    let (_, other) = copy_to_another_window(&mut s);
    let body = other.doc.plain_text(StoryRef::Body);
    assert!(body.starts_with("ก่อน before"), "{body:?}");
    let p = other.doc.body[0].as_para().unwrap();
    assert_eq!(p.runs[0].props.bold, Some(true), "bold survives");
    let boxes: Vec<_> = p.objects.iter().flat_map(InlineObject::text_boxes).collect();
    assert_eq!(boxes.len(), 1);
    assert_eq!(other.doc.plain_text(StoryRef::Part(boxes[0])), "ข้อความในกล่อง");
}

#[test]
fn other_programs_get_pictures_of_objects_they_cannot_read() {
    let mut s = Session::new(Document::from_text("Symbols ✓ ß ๑๒๓ "));
    run(&mut s, "caret.docEnd", json!({}));
    run(&mut s, "insert.symbol", json!({"char": "Ω"}));
    run(&mut s, "insert.shape", json!({"kind": "ellipse", "fill": "2050C0", "width": 60, "height": 40}));
    run(&mut s, "select.all", json!({}));
    run(&mut s, "edit.copy", json!({}));
    let clip = clip_transfer::export(&mut s, ExportOptions { rtf: true, png: true }).unwrap();
    // The HTML after the lossless comment is for other programs: text and a drawn picture.
    let shared = clip.html.split("-->").last().unwrap();
    assert!(shared.contains("Symbols ✓ ß ๑๒๓ Ω"), "{shared}");
    assert!(shared.contains("<img src=\"data:image/png;base64,"), "the shape as a picture");
    assert!(clip.rtf.as_deref().unwrap().contains("\\pngblip"), "the shape as an RTF picture");
    assert!(clip.png.is_none(), "text and a shape aren't one object");
    assert!(clip.text.contains("Symbols ✓ ß ๑๒๓ Ω"));
    // Another program pasting only the HTML sees the picture, sized as the shape.
    let mut other = Session::new(Document::new());
    run(&mut other, "edit.paste", json!({"html": shared}));
    assert!(kinds(&other.doc).contains(&"image".to_string()), "{:?}", kinds(&other.doc));
}

#[test]
fn a_single_object_is_also_a_picture_on_the_clipboard() {
    // A picture: its own PNG.
    let mut s = Session::new(Document::new());
    run(&mut s, "insert.picture", json!({"data": crate::cmd::insert::base64_encode(&png(40))}));
    run(&mut s, "select.all", json!({}));
    run(&mut s, "edit.copy", json!({}));
    let clip = clip_transfer::export(&mut s, ExportOptions { rtf: false, png: true }).unwrap();
    assert_eq!(clip.png.as_deref(), Some(png(40).as_slice()));
    assert!(clip.rtf.is_none());
    // A shape, chart and equation: drawn.
    for (command, params) in [
        ("insert.shape", json!({"kind": "rectangle", "fill": "00A000", "width": 50, "height": 30})),
        ("insert.chart", json!({"type": "pie", "title": "share"})),
        ("insert.equation", json!({"linear": "x=1/2", "edit": false})),
    ] {
        let mut s = Session::new(Document::new());
        run(&mut s, command, params);
        s.math = None;
        run(&mut s, "select.all", json!({}));
        run(&mut s, "edit.copy", json!({}));
        let only_object = s.clipboard.as_ref().unwrap().blocks.iter().all(|b| b.as_para().is_some_and(|p| p.text.chars().all(|c| c == OBJ)));
        let clip = clip_transfer::export(&mut s, ExportOptions { rtf: false, png: true }).unwrap();
        if only_object && s.clipboard.as_ref().unwrap().blocks.len() == 1 {
            let png = clip.png.unwrap_or_else(|| panic!("{command}: no picture"));
            let img = image::load_from_memory(&png).unwrap();
            assert!(img.width() > 4 && img.height() > 4, "{command}: {}×{}", img.width(), img.height());
            let bmp = clip_transfer::bmp_from_png(&png).unwrap();
            assert_eq!(&bmp[..2], b"BM");
            assert_eq!(u32::from_le_bytes(bmp[2..6].try_into().unwrap()) as usize, bmp.len());
            assert_eq!(u32::from_le_bytes(bmp[14..18].try_into().unwrap()), 40, "BITMAPINFOHEADER");
        }
    }
}

#[test]
fn pasting_our_own_copy_back_uses_the_copy_itself() {
    let mut s = Session::new(Document::from_text("one"));
    run(&mut s, "select.all", json!({}));
    run(&mut s, "edit.copy", json!({}));
    let clip = clip_transfer::export(&mut s, ExportOptions::default()).unwrap();
    assert_eq!(s.clipboard_token.as_deref(), Some(clip.token.as_str()));
    // The marked copy wins over the picture/text the system has (same session: no decoding).
    let (f, origin) = crate::cmd::paste::from_system(&mut s, &json!({"html": clip.html})).unwrap();
    assert_eq!(origin, crate::cmd::paste::Origin::Ours);
    assert_eq!(f, *s.clipboard.as_ref().unwrap());
    run(&mut s, "caret.docEnd", json!({}));
    run(&mut s, "edit.paste", json!({"html": clip.html}));
    assert_eq!(s.doc.plain_text(StoryRef::Body), "oneone");
    // A new copy forgets the old token: the old HTML is now another window's copy.
    run(&mut s, "select.all", json!({}));
    run(&mut s, "edit.copy", json!({}));
    assert_eq!(s.clipboard_token, None);
    let (_, origin) = crate::cmd::paste::from_system(&mut s, &json!({"html": clip.html})).unwrap();
    assert_eq!(origin, crate::cmd::paste::Origin::Doc4Me);
}

#[test]
fn another_programs_html_and_rtf_paste_with_their_formatting() {
    let mut s = Session::new(Document::new());
    let pic = crate::cmd::insert::base64_encode(&png(7));
    let html =
        format!("<p><b>หนา</b> plain</p><table><tr><td>a</td><td>b</td></tr></table><p><img src=\"data:image/png;base64,{pic}\" alt=\"x\"></p>");
    run(&mut s, "edit.paste", json!({"text": "หนา plain\na\tb", "html": html}));
    assert_eq!(s.doc.body[0].as_para().unwrap().runs[0].props.bold, Some(true));
    let k = kinds(&s.doc);
    assert!(k.contains(&"table".to_string()) && k.contains(&"image".to_string()), "{k:?}");
    // Keep Text Only for other programs: the clipboard's own text.
    let mut s = Session::new(Document::new());
    s.prefs.paste_other = crate::session::PasteMode::Text;
    run(&mut s, "edit.paste", json!({"text": "หนา plain", "html": "<p><b>หนา</b> plain</p>"}));
    assert_eq!(s.doc.plain_text(StoryRef::Body), "หนา plain");
    assert_eq!(s.doc.body[0].as_para().unwrap().runs[0].props.bold, None);
    // RTF when there is no HTML.
    let mut s = Session::new(Document::new());
    run(&mut s, "edit.paste", json!({"text": "Bold", "rtf": r"{\rtf1\ansi {\b Bold}\par}"}));
    assert_eq!(s.doc.plain_text(StoryRef::Body), "Bold");
    assert_eq!(s.doc.body[0].as_para().unwrap().runs[0].props.bold, Some(true));
    // Word's HTML names its pictures by temporary file; its RTF carries them: RTF first then.
    let mut s = Session::new(Document::new());
    let hex: String = png(5).iter().map(|b| format!("{b:02x}")).collect();
    let rtf = format!("{{\\rtf1\\ansi Pic {{\\pict\\pngblip\\picwgoal600\\pichgoal400 {hex}}}\\par}}");
    let html = r#"<p>Pic <img src="file:///C:/Users/me/AppData/Local/Temp/msohtmlclip1/01/clip_image001.png"></p>"#;
    run(&mut s, "edit.paste", json!({"text": "Pic ", "html": html, "rtf": rtf}));
    assert!(kinds(&s.doc).contains(&"image".to_string()), "{:?}", kinds(&s.doc));
    // Our own text on the clipboard still pastes our own copy, not stale HTML.
    let mut s = Session::new(Document::from_text("mine"));
    run(&mut s, "select.all", json!({}));
    run(&mut s, "edit.copy", json!({}));
    run(&mut s, "caret.docEnd", json!({}));
    run(&mut s, "edit.paste", json!({"text": "mine"}));
    assert_eq!(s.doc.plain_text(StoryRef::Body), "minemine");
}

#[test]
fn paste_special_offers_a_doc4me_copy_from_another_window_as_formatted() {
    let mut s = Session::new(Document::new());
    run(&mut s, "insert.shape", json!({"kind": "heart"}));
    run(&mut s, "select.all", json!({}));
    run(&mut s, "edit.copy", json!({}));
    let clip = clip_transfer::export(&mut s, ExportOptions::default()).unwrap();
    let mut other = Session::new(Document::new());
    let r = run(&mut other, "edit.pasteSpecial", json!({"html": clip.html}));
    assert_eq!(r["formats"][0]["id"], "formatted");
    run(&mut other, "edit.pasteSpecial", json!({"as": "formatted", "html": clip.html}));
    assert_eq!(kinds(&other.doc), vec!["shape"]);
}

#[test]
fn damaged_or_hostile_clipboard_copies_fall_back_without_harm() {
    let good = {
        let mut s = Session::new(Document::from_text("good"));
        run(&mut s, "select.all", json!({}));
        run(&mut s, "edit.copy", json!({}));
        clip_transfer::export(&mut s, ExportOptions::default()).unwrap().html
    };
    let m = clip_transfer::MARKER;
    let bad_zip = crate::cmd::insert::base64_encode(b"PK\x03\x04 not really a zip");
    let cases = [
        format!("<!--{m} v1 id=x blocks=1 docx=!!!not base64!!!--><p>fallback</p>"),
        format!("<!--{m} v1 id=x blocks=1 docx={bad_zip}--><p>fallback</p>"),
        format!("<!--{m} v2 id=x blocks=1 docx={bad_zip}--><p>fallback</p>"),
        format!("<!--{m} v1 id=x blocks=99999999999999999999 docx=AAAA--><p>fallback</p>"),
        format!("<!--{m} v1 id=x blocks=1 docx=--><p>fallback</p>"),
        format!("<!--{m} v1 id=x blocks=1 docx=AAAA<p>fallback</p>"),
        good.replace("blocks=1", "blocks=0").replace("good", "fallback"),
    ];
    for html in cases {
        let mut s = Session::new(Document::new());
        let r = s.run("edit.paste", &json!({"text": "fallback", "html": html}));
        assert!(r.is_ok(), "{html:.80}: {r:?}");
        let text = s.doc.plain_text(StoryRef::Body);
        assert!(text == "fallback" || text == "good", "{html:.80}: {text:?}");
    }
    // Too big: refused before anything is decoded.
    let mut s = Session::new(Document::new());
    let huge = "x".repeat(clip_transfer::MAX_HTML_BYTES + 1);
    assert!(s.run("edit.paste", &json!({"html": huge})).is_err());
    assert!(
        clip_transfer::payload_from_html(&format!("<!--{m} v1 id=x blocks=1 docx={}-->", "A".repeat(clip_transfer::MAX_PAYLOAD_BYTES * 2))).is_none()
    );
    assert!(clip_transfer::bmp_from_png(b"not a png").is_none());
}

#[test]
fn a_copy_ending_with_a_table_pastes_without_an_extra_paragraph() {
    let mut s = Session::new(Document::new());
    run(&mut s, "insert.table", json!({"rows": 2, "cols": 2}));
    // Select just the table (the paragraph after it isn't part of the copy).
    run(&mut s, "table.selectTable", json!({}));
    run(&mut s, "edit.copy", json!({}));
    let blocks = s.clipboard.as_ref().unwrap().blocks.len();
    let clip = clip_transfer::export(&mut s, ExportOptions::default()).unwrap();
    let payload = clip_transfer::payload_from_html(&clip.html).unwrap();
    let f = clip_transfer::fragment_from_payload(&payload).unwrap();
    assert_eq!(f.blocks.len(), blocks);
    assert!(matches!(f.blocks.last(), Some(Block::Table(_))) == matches!(s.clipboard.as_ref().unwrap().blocks.last(), Some(Block::Table(_))));
}

#[test]
fn select_all_copies_a_table_at_the_start_of_the_document_as_a_table() {
    // Copying from inside a table to text after it used to flatten the table into paragraphs.
    let mut s = Session::new(Document::new());
    run(&mut s, "insert.table", json!({"rows": 2, "cols": 2}));
    run(&mut s, "text.insert", json!({"text": "ช่อง"}));
    run(&mut s, "select.all", json!({}));
    run(&mut s, "edit.copy", json!({}));
    let blocks = &s.clipboard.as_ref().unwrap().blocks;
    assert!(matches!(blocks.first(), Some(Block::Table(t)) if t.rows.len() == 2), "{blocks:?}");
    // Pasted in the same document, after it: a second table with the text.
    run(&mut s, "caret.docEnd", json!({}));
    run(&mut s, "edit.paste", json!({}));
    assert_eq!(kinds(&s.doc), vec!["table", "table"]);
    assert_eq!(s.doc.plain_text(StoryRef::Body).matches("ช่อง").count(), 2);
    // From the middle of a table's second row to the text after it: that row onwards.
    let mut d = Document::new();
    let mut t = doc4me_doc::Table::new(3, 2, 200.0);
    for (r, row) in t.rows.iter_mut().enumerate() {
        row.cells[0].blocks = vec![std::sync::Arc::new(Block::Para(doc4me_doc::Paragraph::with_text(&format!("row{r}"), Default::default())))];
    }
    d.body =
        vec![std::sync::Arc::new(Block::Table(t)), std::sync::Arc::new(Block::Para(doc4me_doc::Paragraph::with_text("after", Default::default())))];
    let mut s = Session::new(d);
    s.sel = crate::Selection {
        anchor: doc4me_doc::Pos { story: StoryRef::Body, path: doc4me_doc::Path(vec![0, 1, 0, 0]), off: 2 },
        focus: doc4me_doc::Pos::body(1, 3),
    };
    run(&mut s, "edit.copy", json!({}));
    let blocks = &s.clipboard.as_ref().unwrap().blocks;
    let Some(Block::Table(t)) = blocks.first() else { panic!("{blocks:?}") };
    assert_eq!(t.rows.len(), 2, "rows 1 and 2");
    assert_eq!(blocks.get(1).and_then(Block::as_para).map(|p| p.text.as_str()), Some("aft"));
}
