//! Settings for the table of contents, list of figures and list of tables.

use egui::Ui;
use serde_json::{Map, Value, json};

use crate::WordApp;

#[derive(Default)]
pub struct ListsPane {
    pub open: bool,
    pub kind: usize,
    pending: [Map<String, Value>; 3],
}

const KINDS: [&str; 3] = ["contents", "figures", "tables"];

fn list_state(app: &mut WordApp, kind: &str) -> Value {
    app.session
        .run("references.lists", &json!({}))
        .ok()
        .and_then(|v| v.as_array().and_then(|a| a.iter().find(|x| x.get("kind").and_then(Value::as_str) == Some(kind)).cloned()))
        .unwrap_or(Value::Null)
}

fn change(app: &mut WordApp, pane: &mut ListsPane, kind: &str, info: &Value, key: &str, value: Value) {
    let Some(i) = KINDS.iter().position(|x| *x == kind) else { return };
    if info.get("inDocument").and_then(Value::as_bool) == Some(true) {
        let _ = app.run("references.list", json!({"kind": kind, key: value}));
    } else {
        pane.pending[i].insert(key.to_string(), value);
    }
}

pub fn show(app: &mut WordApp, ui: &mut Ui) {
    if !app.lists_pane.open {
        return;
    }
    let t = crate::theme::Tokens::get(ui.ctx());
    egui::Panel::left("lists_pane")
        .default_size(300.0)
        .resizable(true)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(10).stroke(egui::Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            let mut pane = std::mem::take(&mut app.lists_pane);
            crate::ribbon_keys::stop(ui, crate::ribbon_keys::Region::Pane, ui.max_rect());
            if crate::panes::header(ui, "Tables of Contents") {
                pane.open = false;
                app.lists_pane = pane;
                return;
            }
            ui.separator();
            ui.horizontal(|ui| {
                for (i, name) in ["Contents", "Figures", "Tables"].iter().enumerate() {
                    ui.selectable_value(&mut pane.kind, i, *name);
                }
            });
            let i = pane.kind.min(2);
            let kind = KINDS[i];
            let info = list_state(app, kind);
            if info.is_null() {
                ui.label("List settings are unavailable.");
                app.lists_pane = pane;
                return;
            }
            if info.get("inDocument").and_then(Value::as_bool) == Some(true) {
                let page = info.get("page").and_then(Value::as_str).unwrap_or("");
                ui.label(if page.is_empty() { "In the document".to_string() } else { format!("In the document · page {page}") });
            } else {
                ui.label("Not in the document yet");
            }

            let mut layout = info.get("layout").and_then(Value::as_str).unwrap_or("general").to_string();
            egui::ComboBox::from_label("Layout").selected_text(if layout == "report" { "Report" } else { "General" }).show_ui(ui, |ui| {
                ui.selectable_value(&mut layout, "general".into(), "General");
                ui.selectable_value(&mut layout, "report".into(), "Thai report");
            });
            if info.get("layout").and_then(Value::as_str) != Some(&layout) {
                change(app, &mut pane, kind, &info, "layout", json!(layout));
            }

            if kind == "contents" {
                let mut levels = info.get("levels").and_then(Value::as_u64).unwrap_or(3).clamp(1, 9) as u8;
                if ui.add(egui::Slider::new(&mut levels, 1..=9).text("Heading levels")).changed() {
                    change(app, &mut pane, kind, &info, "levels", json!(levels));
                }
            } else {
                let choices: &[&str] =
                    if kind == "tables" { &["Table", "ตารางที่"] } else { &["Figure", "ภาพที่", "รูปที่"] };
                let mut label = info.get("label").and_then(Value::as_str).unwrap_or(choices[0]).to_string();
                egui::ComboBox::from_label("Caption label").selected_text(&label).show_ui(ui, |ui| {
                    for choice in choices {
                        ui.selectable_value(&mut label, (*choice).to_string(), *choice);
                    }
                });
                if info.get("label").and_then(Value::as_str) != Some(&label) {
                    change(app, &mut pane, kind, &info, "label", json!(label));
                }
                let mut mode = info.get("show").and_then(Value::as_str).unwrap_or("full").to_string();
                egui::ComboBox::from_label("Each line shows")
                    .selected_text(match mode.as_str() {
                        "number" => "Number only",
                        "none" => "Title only",
                        _ => "Label, number and title",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut mode, "full".into(), "Label, number and title");
                        ui.selectable_value(&mut mode, "number".into(), "Number only");
                        ui.selectable_value(&mut mode, "none".into(), "Title only");
                    });
                if info.get("show").and_then(Value::as_str) != Some(&mode) {
                    change(app, &mut pane, kind, &info, "show", json!(mode));
                }
                if layout == "report" {
                    let mut column = info.get("columnHeading").and_then(Value::as_bool).unwrap_or(false);
                    if ui.checkbox(&mut column, "Show caption label over number column").changed() {
                        change(app, &mut pane, kind, &info, "columnHeading", json!(column));
                    }
                }
            }

            let mut leader = info.get("leader").and_then(Value::as_str).unwrap_or("dot").to_string();
            egui::ComboBox::from_label("Tab leader").selected_text(&leader).show_ui(ui, |ui| {
                for (value, label) in [("dot", "Dots"), ("hyphen", "Hyphens"), ("underscore", "Underline"), ("none", "None")] {
                    ui.selectable_value(&mut leader, value.into(), label);
                }
            });
            if info.get("leader").and_then(Value::as_str) != Some(&leader) {
                change(app, &mut pane, kind, &info, "leader", json!(leader));
            }

            ui.separator();
            ui.horizontal(|ui| {
                if info.get("inDocument").and_then(Value::as_bool) == Some(true) {
                    if ui.button("Remove").clicked() {
                        let _ = app.run("references.list", json!({"kind": kind, "remove": true}));
                    }
                } else if ui.button("Insert at Cursor").clicked() {
                    let mut params = pane.pending[i].clone();
                    params.insert("kind".into(), json!(kind));
                    let _ = app.run("references.list", Value::Object(params));
                }
                if ui.button("Update All").clicked() {
                    let _ = app.run("references.updateLists", json!({}));
                }
            });
            app.lists_pane = pane;
        });
}
