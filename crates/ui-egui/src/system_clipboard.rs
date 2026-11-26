//! Copy, Cut and Paste through the system clipboard with more than text.
//!
//! egui's clipboard only carries text, so pictures, shapes, text boxes, tables, charts and the
//! like reached other programs (and Doc4Me in another window) as their text, or not at all. The
//! host gives a [`SystemClipboard`]: after every Copy and Cut the UI hands it the formats
//! [`doc4me_engine::clip_transfer::export`] makes (HTML with the lossless copy, RTF, a picture,
//! text), and Paste reads the clipboard's HTML and RTF back and passes them to the engine, which
//! tells our own copy, another Doc4Me window's and another program's apart.
//!
//! Without the service (the web) Copy puts the text on the clipboard through egui, as before.

use serde_json::Value;

use doc4me_engine::clip_transfer::{ExportOptions, SystemClip};

use crate::WordApp;

/// What the host read from the system clipboard besides text and pictures.
#[derive(Clone, Debug, Default)]
pub struct RichClipboard {
    pub html: Option<String>,
    pub rtf: Option<String>,
}

/// The host's system clipboard (desktop).
pub struct SystemClipboard {
    /// The formats this system's clipboard takes besides HTML and text.
    pub formats: ExportOptions,
    /// Replace the clipboard's content with these formats at once. An error means nothing was
    /// set; the UI then puts the text there through egui.
    pub set: Box<dyn Fn(&SystemClip) -> Result<(), String>>,
    /// The clipboard's HTML and RTF, if any.
    pub get: Box<dyn Fn() -> RichClipboard>,
}

/// After Copy or Cut: hand the copy to the system clipboard (every format, or the text alone).
pub(crate) fn publish(app: &mut WordApp) {
    let formats = app.services.system_clipboard.as_ref().map(|c| c.formats);
    if let Some(formats) = formats
        && let Some(clip) = doc4me_engine::clip_transfer::export(&mut app.session, formats)
        && let Some(c) = &app.services.system_clipboard
    {
        match (c.set)(&clip) {
            Ok(()) => return,
            Err(e) => log::warn!("clipboard: couldn't offer the copy in every format ({e}); copying its text"),
        }
    }
    let text = app.session.clipboard_text.clone();
    if let Some(ctx) = &app.ctx {
        // An object alone has no text: still replace what another program left there, so a
        // later Paste doesn't bring that back instead of our copy.
        ctx.copy_text(text);
    }
}

/// Before Paste: add the clipboard's HTML and RTF to the params (unless a caller passed them).
/// True when the HTML carries a Doc4Me copy, which then comes before a picture on the clipboard.
pub(crate) fn add_rich(app: &WordApp, params: &mut Value) -> bool {
    let Some(c) = &app.services.system_clipboard else { return false };
    let Some(o) = params.as_object_mut() else { return false };
    if !o.contains_key("html") && !o.contains_key("rtf") {
        let rich = (c.get)();
        if let Some(h) = rich.html.filter(|h| !h.trim().is_empty() && h.len() <= doc4me_engine::clip_transfer::MAX_HTML_BYTES) {
            o.insert("html".into(), Value::String(h));
        }
        if let Some(r) = rich.rtf.filter(|r| !r.trim().is_empty() && r.len() <= doc4me_engine::cmd::paste::MAX_PASTE_BYTES) {
            o.insert("rtf".into(), Value::String(r));
        }
    }
    let marker = format!("<!--{} ", doc4me_engine::clip_transfer::MARKER);
    o.get("html").and_then(Value::as_str).is_some_and(|h| h.contains(&marker))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use doc4me_engine::Session;
    use serde_json::json;

    use super::*;
    use crate::Services;
    use crate::paste_picture::ClipboardPicture;

    /// A system clipboard shared by two windows (two apps) and the picture another program put
    /// beside it.
    #[derive(Clone, Default)]
    struct FakeSystem {
        clip: Rc<RefCell<Option<SystemClip>>>,
        sets: Rc<RefCell<usize>>,
    }

    impl FakeSystem {
        fn app(&self, doc: doc4me_doc::Document) -> WordApp {
            let (put, sets, got) = (self.clip.clone(), self.sets.clone(), self.clip.clone());
            let services = Services {
                system_clipboard: Some(SystemClipboard {
                    formats: ExportOptions { rtf: true, png: true },
                    set: Box::new(move |c| {
                        *put.borrow_mut() = Some(c.clone());
                        *sets.borrow_mut() += 1;
                        Ok(())
                    }),
                    get: Box::new(move || RichClipboard { html: got.borrow().as_ref().map(|c| c.html.clone()), rtf: None }),
                }),
                // Our copy's picture is on the clipboard too (Windows): Paste must not take it.
                clipboard_picture: Some(Box::new(|| Some(ClipboardPicture::Pixels { width: 2, height: 2, rgba: vec![9; 16] }))),
                ..Default::default()
            };
            WordApp::new(Session::new(doc), services)
        }
    }

    fn shape_kinds(a: &WordApp) -> Vec<&'static str> {
        a.session
            .doc
            .body
            .iter()
            .filter_map(|b| b.as_para())
            .flat_map(|p| &p.objects)
            .filter_map(|o| match o {
                doc4me_doc::InlineObject::Shape { .. } => Some("shape"),
                doc4me_doc::InlineObject::Image { .. } => Some("image"),
                _ => None,
            })
            .collect()
    }

    /// Copy and Cut (keys, ribbon and menus all go through `run`) put every format on the system
    /// clipboard; another window pastes the shape itself, not the picture beside it.
    #[test]
    fn a_shape_copied_in_one_window_pastes_as_a_shape_in_another() {
        let system = FakeSystem::default();
        let mut one = system.app(doc4me_doc::Document::new());
        one.run("insert.shape", json!({"kind": "star"})).unwrap();
        one.run("select.all", json!({})).unwrap();
        one.run("edit.copy", json!({})).unwrap();
        assert_eq!(*system.sets.borrow(), 1);
        let clip = system.clip.borrow().clone().unwrap();
        assert!(clip.html.contains(doc4me_engine::clip_transfer::MARKER));
        assert!(clip.png.is_some(), "a picture for picture programs");
        let mut two = system.app(doc4me_doc::Document::new());
        two.run("edit.paste", json!({})).unwrap();
        assert_eq!(shape_kinds(&two), vec!["shape"]);
        // In the same window too (our own copy).
        one.run("caret.docEnd", json!({})).unwrap();
        one.run("edit.paste", json!({})).unwrap();
        assert_eq!(shape_kinds(&one), vec!["shape", "shape"]);
        // Cut publishes as well; Copy with nothing selected leaves the clipboard alone.
        one.run("select.all", json!({})).unwrap();
        one.run("edit.cut", json!({})).unwrap();
        assert_eq!(*system.sets.borrow(), 2);
        one.run("edit.copy", json!({})).unwrap();
        assert_eq!(*system.sets.borrow(), 2);
        one.run("edit.paste", json!({})).unwrap();
        assert_eq!(shape_kinds(&one), vec!["shape", "shape"]);
    }

    /// Another program's HTML (no Doc4Me copy) with its text: pasted with its formatting.
    #[test]
    fn another_programs_html_pastes_formatted() {
        let services = Services {
            clipboard_text: Some(Box::new(|| Some("ตัวหนา".to_string()))),
            system_clipboard: Some(SystemClipboard {
                formats: ExportOptions::default(),
                set: Box::new(|_| Err("no clipboard".into())),
                get: Box::new(|| RichClipboard { html: Some("<p><b>ตัวหนา</b></p>".into()), rtf: None }),
            }),
            ..Default::default()
        };
        let mut a = WordApp::new(Session::new(doc4me_doc::Document::new()), services);
        a.run("edit.paste", json!({})).unwrap();
        let p = a.session.doc.body[0].as_para().unwrap();
        assert_eq!(p.text, "ตัวหนา");
        assert_eq!(p.runs[0].props.bold, Some(true));
        // A failing clipboard still copies (the text goes through egui).
        a.run("select.all", json!({})).unwrap();
        assert!(a.run("edit.copy", json!({})).is_ok());
    }
}
