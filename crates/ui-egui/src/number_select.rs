//! Selecting and formatting list numbers, as in Word.

use doc4me_doc::{Path, Pos, StoryRef};
use doc4me_layout::DocLayout;
use egui::{Color32, Painter, Rect, pos2, vec2};
use serde_json::{Value, json};

use crate::WordApp;

/// The list and level of the paragraph at `story`/`path`, when it is numbered.
pub fn list_of(doc: &doc4me_doc::Document, story: StoryRef, path: &Path) -> Option<(u32, u8)> {
    let paragraph = doc.para_at(&Pos { story, path: path.clone(), off: 0 })?;
    let numbering = paragraph.props.numbering.or_else(|| doc.styles.resolve_para(&paragraph.props).numbering)?;
    (numbering.num != 0).then_some((numbering.num, numbering.level))
}

/// The number selected by clicking it, while the caret remains at that paragraph's start.
pub fn active(app: &WordApp) -> Option<(StoryRef, Path)> {
    let (story, path) = app.canvas.number_sel.clone()?;
    let start = Pos { story, path: path.clone(), off: 0 };
    (app.session.sel.is_collapsed() && app.session.sel.focus == start && list_of(&app.session.doc, story, &path).is_some()).then_some((story, path))
}

/// Select a paragraph's number and put the caret at the paragraph start.
pub fn select(app: &mut WordApp, story: StoryRef, path: Path) {
    app.session.sel = doc4me_engine::Selection::caret(Pos { story, path: path.clone(), off: 0 });
    app.canvas.number_sel = Some((story, path));
}

const ROUTED: [&str; 8] =
    ["format.bold", "format.italic", "format.underline", "format.font", "format.size", "format.color", "format.growFont", "format.shrinkFont"];

const FONT_SIZES: [f32; 16] = [8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0, 26.0, 28.0, 36.0, 48.0, 72.0];

/// Route font-formatting commands to the selected number's entire list level.
pub fn route(app: &mut WordApp, id: &str, params: &Value) -> Option<Result<Value, String>> {
    if !ROUTED.contains(&id) {
        return None;
    }
    let (story, path) = active(app)?;
    let (num, level) = list_of(&app.session.doc, story, &path)?;
    let definition = app.session.run("list.get", &json!({"num": num})).ok()?;
    let current = definition.get("levels")?.get(usize::from(level))?.clone();
    let flag = |key: &str| current.get(key).and_then(Value::as_bool).unwrap_or(false);
    let given = |key: &str| params.get(key).and_then(Value::as_bool);
    let text_size =
        app.session.run("format.state", &json!({})).ok().and_then(|state| state.get("size").and_then(Value::as_f64)).map(|size| size as f32);
    let size_now = || current.get("size").and_then(Value::as_f64).map(|size| size as f32).or(text_size);
    let patch = match id {
        "format.bold" => json!({"bold": given("value").unwrap_or(!flag("bold"))}),
        "format.italic" => json!({"italic": given("value").unwrap_or(!flag("italic"))}),
        "format.underline" => json!({"underline": given("value").unwrap_or(!flag("underline"))}),
        "format.font" => json!({"font": params.get("name").cloned().unwrap_or(Value::Null)}),
        "format.size" => json!({"size": params.get("size").cloned().unwrap_or(Value::Null)}),
        "format.color" => match params.get("color").and_then(Value::as_str) {
            Some("auto") | None => json!({"color": Value::Null}),
            Some(color) => json!({"color": color}),
        },
        "format.growFont" | "format.shrinkFont" => {
            let now = size_now().unwrap_or(11.0);
            let next = if id == "format.growFont" {
                FONT_SIZES.iter().copied().find(|size| *size > now + 0.01).unwrap_or((now + 10.0).min(1638.0))
            } else {
                FONT_SIZES.iter().rev().copied().find(|size| *size < now - 0.01).unwrap_or((now - 1.0).max(1.0))
            };
            json!({"size": next})
        }
        _ => return None,
    };
    let mut levels = vec![json!({}); usize::from(level) + 1];
    *levels.last_mut()? = patch;
    Some(app.run("list.define", json!({"num": num, "apply": false, "levels": levels})))
}

/// Shade every list number at the selected level, with a stronger shade on the selected one.
pub fn paint(app: &WordApp, layout: &DocLayout, page_rects: &[Rect], scale_of: impl Fn(usize) -> f32, painter: &Painter) {
    let Some((story, path)) = active(app) else { return };
    let Some(list) = list_of(&app.session.doc, story, &path) else { return };
    let (selected, others) = if crate::theme::Tokens::get(painter.ctx()).dark {
        (Color32::from_white_alpha(70), Color32::from_white_alpha(34))
    } else {
        (Color32::from_black_alpha(64), Color32::from_black_alpha(30))
    };
    for (page_index, page_rect) in page_rects.iter().enumerate() {
        if !painter.clip_rect().intersects(*page_rect) {
            continue;
        }
        let Some(page) = layout.pages.get(page_index) else { continue };
        let scale = scale_of(page_index);
        for item in &page.items {
            let doc4me_layout::Placed::Lines { story: item_story, path: item_path, l0: 0, .. } = item else { continue };
            if *item_story != story || list_of(&app.session.doc, *item_story, item_path) != Some(list) {
                continue;
            }
            let Some((label_page, rect)) = layout.label_rect(*item_story, item_path, page_index) else { continue };
            if label_page != page_index {
                continue;
            }
            let screen =
                Rect::from_min_size(pos2(page_rect.min.x + rect.x * scale, page_rect.min.y + rect.y * scale), vec2(rect.w * scale, rect.h * scale));
            painter.rect_filled(screen, 0.0, if *item_path == path { selected } else { others });
        }
    }
}
