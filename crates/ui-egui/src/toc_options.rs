//! The Table of Contents content-control button and its focused options menu.

use doc4me_doc::control::ControlKind;
use doc4me_doc::para::InlineObject;
use egui::Context;
use serde_json::json;

use crate::WordApp;

/// Whether the caret is in Word's Table of Contents gallery content control.
pub fn caret_in_toc_control(app: &WordApp) -> bool {
    let caret = &app.session.sel.focus;
    let ranges = app.session.doc.control_ranges(caret.story);
    ranges.into_iter().any(|range| {
        range.contains(caret)
            && matches!(range.control.kind, ControlKind::Gallery { list: false, ref gallery, .. } if gallery.eq_ignore_ascii_case("Table of Contents"))
    }) || (caret.story == doc4me_doc::StoryRef::Body
        && caret.path.depth() == 0
        && app.session.doc.para_at(caret).is_some_and(|p| {
            p.props.style.as_deref().is_some_and(|style| style.starts_with("TOC") && style != "TOCHeading")
                || (p.props.style.as_deref() == Some("TOCHeading")
                    && p.object_offsets().into_iter().any(|off| {
                        matches!(p.object_at(off), Some(InlineObject::Field { instr, .. }) if instr.trim_start().get(..3).is_some_and(|name| name.eq_ignore_ascii_case("TOC")))
                    }))
        }))
}

/// Show the options attached to the selected TOC content-control button.
pub fn show(app: &mut WordApp, ctx: &Context) {
    if !app.ui.toc_options_open {
        return;
    }
    if !caret_in_toc_control(app) {
        app.ui.toc_options_open = false;
        return;
    }
    let mut action = None;
    egui::Window::new(crate::tl!("Table of Contents"))
        .id(egui::Id::new("toc_options_button"))
        .collapsible(false)
        .resizable(false)
        .open(&mut app.ui.toc_options_open)
        .show(ctx, |ui| {
            if ui.button(crate::tl!("Update Table")).clicked() {
                action = Some("references.updateToc");
            }
            if ui.button(crate::tl!("Remove Table of Contents")).clicked() {
                action = Some("references.removeToc");
            }
        });
    if let Some(id) = action {
        app.ui.toc_options_open = false;
        let _ = app.run(id, json!({}));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc4me_doc::control::{ContentControl, ControlKind};
    use doc4me_doc::para::InlineObject;
    use doc4me_doc::{Document, Pos};
    use doc4me_engine::Session;

    fn app_with_toc_control() -> WordApp {
        let mut app = WordApp::new(Session::new(Document::from_text("Source heading")), crate::Services::default());
        app.run("select.range", json!({"anchor": {"block": 0, "off": 0}, "focus": {"block": 0, "off": 14}})).unwrap();
        app.run("styles.apply", json!({"style": "Heading 1"})).unwrap();
        app.run("references.toc", json!({})).unwrap();
        let mut control =
            ContentControl::new(ControlKind::Gallery { list: false, gallery: "Table of Contents".into(), category: String::new(), unique: false });
        control.title = "Table of Contents".into();
        app.session.doc.insert_object(&Pos::body(0, 0), InlineObject::ControlStart { control: Box::new(control) }, &Default::default()).unwrap();
        let end = app.session.doc.para_at(&Pos::body(0, 0)).unwrap().len();
        app.session.doc.insert_object(&Pos::body(0, end), InlineObject::ControlEnd, &Default::default()).unwrap();
        app.session.sel = doc4me_engine::Selection::caret(Pos::body(0, 4));
        app
    }

    #[test]
    fn options_open_only_inside_a_toc_gallery_control() {
        let mut app = app_with_toc_control();
        assert!(caret_in_toc_control(&app));
        assert_eq!(app.execute("ui.tocOptions", json!({})).unwrap(), json!({"open": true}));
        assert!(app.ui.toc_options_open);
        assert_eq!(app.execute("ui.tocOptions", json!({})).unwrap(), json!({"open": false}));
        let outside = app
            .session
            .doc
            .body
            .iter()
            .enumerate()
            .find_map(|(i, block)| block.as_para().filter(|p| !p.props.style.as_deref().is_some_and(|s| s.starts_with("TOC"))).map(|_| i))
            .expect("source heading paragraph");
        app.session.sel = doc4me_engine::Selection::caret(Pos::body(outside, 0));
        assert!(!caret_in_toc_control(&app));
        assert!(app.execute("ui.tocOptions", json!({})).is_err());
        assert!(!app.ui.toc_options_open);
    }

    #[test]
    fn word_shortcut_opens_options_only_in_the_toc_control() {
        let mut app = app_with_toc_control();
        let ctx = Context::default();
        let modifiers = egui::Modifiers { alt: true, shift: true, ..Default::default() };
        ctx.run_ui(
            egui::RawInput {
                events: vec![
                    egui::Event::ModifiersChanged(modifiers),
                    egui::Event::Key { key: egui::Key::F12, physical_key: None, pressed: true, repeat: false, modifiers },
                ],
                ..Default::default()
            },
            |ui| crate::keys::canvas_events(&mut app, ui.ctx()),
        )
        .drop_without_applying_deltas();
        assert!(app.ui.toc_options_open);

        let mut plain = WordApp::new(Session::new(Document::from_text("plain text")), crate::Services::default());
        let none = egui::Modifiers::NONE;
        ctx.run_ui(
            egui::RawInput {
                events: vec![
                    egui::Event::ModifiersChanged(modifiers),
                    egui::Event::Key { key: egui::Key::F12, physical_key: None, pressed: true, repeat: false, modifiers },
                    egui::Event::ModifiersChanged(none),
                ],
                ..Default::default()
            },
            |ui| crate::keys::canvas_events(&mut plain, ui.ctx()),
        )
        .drop_without_applying_deltas();
        assert!(!plain.ui.toc_options_open);
    }

    #[test]
    fn options_menu_shows_and_runs_remove_table() {
        use egui_kittest::kittest::Queryable;

        let mut app = app_with_toc_control();
        app.ui.toc_options_open = true;
        let mut h =
            egui_kittest::Harness::builder().with_size(egui::vec2(900.0, 600.0)).build_ui_state(|ui, app: &mut WordApp| show(app, ui.ctx()), app);
        h.run_steps(2);
        let _ = h.get_by_label("Update Table");
        let _ = h.get_by_label("Remove Table of Contents");
        h.get_by_label("Remove Table of Contents").click();
        h.run_steps(2);
        assert!(!h.state().ui.toc_options_open);
        assert!(!h.state().session.doc.plain_text(doc4me_doc::StoryRef::Body).contains("Contents"));
    }

    #[test]
    fn options_also_open_for_an_app_generated_toc_without_a_content_control() {
        let mut app = WordApp::new(Session::new(Document::from_text("Source heading")), crate::Services::default());
        app.run("select.range", json!({"anchor": {"block": 0, "off": 0}, "focus": {"block": 0, "off": 14}})).unwrap();
        app.run("styles.apply", json!({"style": "Heading 1"})).unwrap();
        app.run("references.toc", json!({})).unwrap();
        let entry = app
            .session
            .doc
            .body
            .iter()
            .enumerate()
            .find_map(|(i, block)| block.as_para().and_then(|p| p.props.style.as_deref()).filter(|s| s.starts_with("TOC")).map(|_| i))
            .expect("TOC paragraph");
        app.session.sel = doc4me_engine::Selection::caret(Pos::body(entry, 0));
        assert!(caret_in_toc_control(&app));
        assert_eq!(app.execute("ui.tocOptions", json!({})).unwrap(), json!({"open": true}));
    }
}
