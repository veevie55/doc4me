//! Context actions for built-in citation fields and generated bibliographies.

use doc4me_doc::StoryRef;
use doc4me_doc::para::InlineObject;
use egui::Context;
use serde_json::json;

use crate::WordApp;

fn field_name(instr: &str) -> &str {
    instr.split_whitespace().next().unwrap_or("").trim_start_matches('\u{0013}')
}

fn bibliography_field(app: &WordApp, block: usize) -> bool {
    app.session.doc.body.get(block).and_then(|b| b.as_para()).is_some_and(|p| {
        p.object_offsets()
            .into_iter()
            .any(|off| matches!(p.object_at(off), Some(InlineObject::Field { instr, .. }) if field_name(instr).eq_ignore_ascii_case("BIBLIOGRAPHY")))
    })
}

/// The caret is in a simple CITATION field, a BIBLIOGRAPHY field, or one of its generated entries.
pub fn caret_in_citation_content(app: &WordApp) -> bool {
    let caret = &app.session.sel.focus;
    if caret.story != StoryRef::Body || caret.path.depth() != 0 {
        return false;
    }
    if app
        .session
        .doc
        .field_ranges(StoryRef::Body)
        .iter()
        .any(|r| r.start < *caret && *caret <= r.end && field_name(&r.instr).eq_ignore_ascii_case("BIBLIOGRAPHY"))
    {
        return true;
    }
    let block = caret.path.last();
    let Some(para) = app.session.doc.para_at(caret) else { return false };
    let width = doc4me_doc::para::OBJ.len_utf8();
    if para.object_offsets().into_iter().any(|off| {
        matches!(para.object_at(off), Some(InlineObject::Field { instr, .. })
            if field_name(instr).eq_ignore_ascii_case("BIBLIOGRAPHY")
                && caret.off <= off.saturating_add(width))
    }) {
        return true;
    }
    if para.object_offsets().into_iter().any(|off| {
        matches!(para.object_at(off), Some(InlineObject::Field { instr, .. })
            if field_name(instr).eq_ignore_ascii_case("CITATION")
                && off <= caret.off.saturating_add(width)
                && caret.off <= off.saturating_add(width))
    }) {
        return true;
    }
    // The built-in bibliography is represented by a heading field followed by ListEntry paragraphs.
    if para.props.style.as_deref() != Some("ListEntry") {
        return false;
    }
    let mut i = block;
    loop {
        if bibliography_field(app, i) {
            return true;
        }
        if i == 0
            || (i != block && !app.session.doc.body.get(i).and_then(|b| b.as_para()).is_some_and(|p| p.props.style.as_deref() == Some("ListEntry")))
        {
            return false;
        }
        i -= 1;
    }
}

/// Open the supported citation actions as a small menu near the document.
pub fn show(app: &mut WordApp, ctx: &Context) {
    if !app.ui.citation_options_open {
        return;
    }
    if !caret_in_citation_content(app) {
        app.ui.citation_options_open = false;
        return;
    }
    let mut action: Option<(&str, serde_json::Value)> = None;
    egui::Window::new(crate::tl!("Citation Style"))
        .id(egui::Id::new("citation_options_button"))
        .collapsible(false)
        .resizable(false)
        .open(&mut app.ui.citation_options_open)
        .show(ctx, |ui| {
            if ui.button(crate::tl!("Update Field")).clicked() {
                action = Some(("references.updateFields", json!({})));
            }
            ui.separator();
            for style in &doc4me_engine::cmd::citations::STYLES {
                let button = egui::Button::new(style.name).selected(style.name == app.session.bib_style);
                if ui.add(button).clicked() {
                    action = Some(("references.citationStyle", json!({"style": style.name})));
                }
            }
        });
    if let Some((id, params)) = action {
        app.ui.citation_options_open = false;
        let _ = app.run(id, params);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc4me_doc::Document;
    use doc4me_engine::Session;

    fn citation_app() -> WordApp {
        let mut app = WordApp::new(Session::new(Document::from_text("Source passage")), crate::Services::default());
        app.run("references.citation", json!({"source": {"author": "Ada Writer", "title": "A Test Book", "year": "2024"}})).unwrap();
        app
    }

    #[test]
    fn only_citation_and_bibliography_content_has_options() {
        let mut app = citation_app();
        assert!(caret_in_citation_content(&app), "caret is just after the inserted citation field");
        app.run("references.bibliography", json!({})).unwrap();
        let list_entry = app
            .session
            .doc
            .body
            .iter()
            .enumerate()
            .find_map(|(i, b)| b.as_para().filter(|p| p.props.style.as_deref() == Some("ListEntry")).map(|_| i));
        if let Some(i) = list_entry {
            app.session.sel = doc4me_engine::Selection::caret(doc4me_doc::Pos::body(i, 0));
            assert!(caret_in_citation_content(&app), "caret is inside the generated bibliography");
        }
        app.run("caret.docEnd", json!({})).unwrap();
        assert!(!caret_in_citation_content(&app), "ordinary prose doesn't show citation options");
    }

    #[test]
    fn menu_updates_the_citation_style_for_a_citation() {
        use egui_kittest::kittest::Queryable;

        let mut app = citation_app();
        assert_eq!(app.execute("ui.citationOptions", json!({})).unwrap(), json!({"open": true}));
        let mut h =
            egui_kittest::Harness::builder().with_size(egui::vec2(900.0, 700.0)).build_ui_state(|ui, app: &mut WordApp| show(app, ui.ctx()), app);
        h.run_steps(2);
        let _ = h.get_by_label("MLA");
        h.get_by_label("MLA").click();
        h.run_steps(2);
        assert_eq!(h.state().session.bib_style, "MLA");
        assert!(!h.state().ui.citation_options_open);
    }

    #[test]
    fn citation_options_use_the_two_key_sequence_without_interfering_with_toc() {
        let mut app = citation_app();
        let spaces_before = app.session.doc.plain_text(StoryRef::Body).matches(' ').count();
        let ctx = Context::default();
        let chord = egui::Modifiers { alt: true, shift: true, ..Default::default() };
        ctx.run_ui(
            egui::RawInput {
                events: vec![
                    egui::Event::ModifiersChanged(chord),
                    egui::Event::Key { key: egui::Key::F12, physical_key: None, pressed: true, repeat: false, modifiers: chord },
                ],
                ..Default::default()
            },
            |ui| crate::keys::canvas_events(&mut app, ui.ctx()),
        )
        .drop_without_applying_deltas();
        assert!(app.ui.citation_options_pending);
        assert!(!app.ui.citation_options_open);
        let no_modifiers = egui::Modifiers::NONE;
        ctx.run_ui(
            egui::RawInput {
                events: vec![
                    egui::Event::ModifiersChanged(no_modifiers),
                    egui::Event::Key { key: egui::Key::Space, physical_key: None, pressed: true, repeat: false, modifiers: no_modifiers },
                    egui::Event::Text(" ".into()),
                ],
                ..Default::default()
            },
            |ui| crate::keys::canvas_events(&mut app, ui.ctx()),
        )
        .drop_without_applying_deltas();
        assert!(!app.ui.citation_options_pending);
        assert!(app.ui.citation_options_open);
        assert_eq!(
            app.session.doc.plain_text(StoryRef::Body).matches(' ').count(),
            spaces_before,
            "Space was consumed rather than inserted into the citation"
        );
    }
}
