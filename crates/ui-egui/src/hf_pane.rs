//! Header & Footer controls shown while editing a header or footer.

use egui::{Stroke, Ui};
use serde_json::{Value, json};

use crate::{WordApp, options::MeasureExt as _};

const EDITING: &str = "doc4me_hf_pane_editing";
const CLOSED: &str = "doc4me_hf_pane_closed";

/// Show the pane while editing a header/footer; closing it hides it until editing ends.
pub fn wanted(app: &mut WordApp, ctx: &egui::Context) -> bool {
    let layout = app.session.layout();
    let editing = crate::canvas::editing_header_footer(app, &layout);
    let id_editing = egui::Id::new(EDITING);
    let id_closed = egui::Id::new(CLOSED);
    let (was_editing, was_closed) =
        ctx.data(|data| (data.get_temp::<bool>(id_editing).unwrap_or(false), data.get_temp::<bool>(id_closed).unwrap_or(false)));
    let closed = editing && was_editing && was_closed;
    ctx.data_mut(|data| {
        data.insert_temp(id_editing, editing);
        data.insert_temp(id_closed, closed);
    });
    editing && !closed
}

pub fn show(app: &mut WordApp, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    let tokens = crate::theme::Tokens::get(&ctx);
    egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt("hf_pane_scroll").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(tl!("Header & Footer")).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if crate::widgets::icon_button(ui, "close", tl!("Close")).clicked() {
                    ctx.data_mut(|data| data.insert_temp(egui::Id::new(CLOSED), true));
                }
            });
        });
        ui.add_space(6.0);
        let state = app.session.run("hf.state", &json!({})).unwrap_or(Value::Null);
        let is_header = state.get("header").and_then(Value::as_bool).unwrap_or(true);
        let section = state.get("section").and_then(Value::as_u64).unwrap_or(0) as usize;
        let sections = state.get("sections").and_then(Value::as_u64).unwrap_or(1) as usize;
        let page = doc4me_engine::cmd::hf::context(&mut app.session).map_or(0, |context| context.page);

        heading(ui, "Insert", &tokens);
        ui.horizontal(|ui| {
            menu(ui, app, "Header", |ui, app| {
                item(ui, app, "Blank", "insert.header", json!({"preset": "blank"}));
                item(ui, app, "Blank (Three Columns)", "insert.header", json!({"preset": "blankThree"}));
                item(ui, app, "Document Title", "insert.header", json!({"preset": "title"}));
                ui.separator();
                item(ui, app, "Edit Header", "insert.editHeader", json!({"page": page}));
            });
            menu(ui, app, "Footer", |ui, app| {
                item(ui, app, "Blank", "insert.footer", json!({"preset": "blank"}));
                item(ui, app, "Blank (Three Columns)", "insert.footer", json!({"preset": "blankThree"}));
                item(ui, app, "Page Number", "insert.footer", json!({"preset": "pageNumber"}));
                ui.separator();
                item(ui, app, "Edit Footer", "insert.editFooter", json!({"page": page}));
            });
            menu(ui, app, "Page Number", |ui, app| {
                item(ui, app, "Top of Page", "insert.pageNumber", json!({"position": "top", "align": "right"}));
                item(ui, app, "Bottom of Page", "insert.pageNumber", json!({"position": "bottom", "align": "center"}));
                item(ui, app, "Page X of Y", "insert.pageNumber", json!({"position": "bottom", "format": "x of y"}));
                item(ui, app, "Current Position", "insert.pageNumber", json!({"position": "current"}));
            });
        });
        ui.horizontal(|ui| {
            if ui.button(tl!("Date & Time")).clicked() {
                let _ = app.run("insert.dateTime", json!({}));
            }
            if ui.button(tl!("Field…")).clicked() {
                let _ = app.run("ui.dialog", json!({"name": "field"}));
            }
            if ui.button(tl!("Picture from File")).clicked() {
                let _ = app.run("insert.picture", json!({}));
            }
        });

        heading(ui, "Navigation", &tokens);
        ui.horizontal(|ui| {
            if ui.add_enabled(!is_header, egui::Button::new(tl!("Go to Header"))).clicked() {
                let _ = app.run("insert.editHeader", json!({"page": page}));
            }
            if ui.add_enabled(is_header, egui::Button::new(tl!("Go to Footer"))).clicked() {
                let _ = app.run("insert.editFooter", json!({"page": page}));
            }
        });
        ui.horizontal(|ui| {
            if ui.add_enabled(section > 0, egui::Button::new(tl!("Previous Section"))).clicked() {
                let _ = app.run("hf.previous", json!({}));
            }
            if ui.add_enabled(section + 1 < sections, egui::Button::new(tl!("Next Section"))).clicked() {
                let _ = app.run("hf.next", json!({}));
            }
        });
        let linked = state.get("linked").and_then(Value::as_bool).unwrap_or(false);
        if ui.add_enabled(section > 0, egui::Button::new(tl!("Link to Previous")).selected(linked)).clicked() {
            let _ = app.run("layout.linkToPrevious", json!({}));
        }

        heading(ui, "Options", &tokens);
        checkbox_command(ui, app, &state, "differentFirst", "Different First Page", "layout.differentFirstPage");
        checkbox_command(ui, app, &state, "differentOddEven", "Different Odd & Even Pages", "layout.differentOddEven");
        checkbox_command(ui, app, &state, "turnOnLandscape", "Turn on Landscape Pages", "hf.turnOnLandscape");
        checkbox_command(ui, app, &state, "showDocumentText", "Show Document Text", "hf.showDocumentText");

        heading(ui, "Position", &tokens);
        for (key, label) in [("headerFromTop", "Header from Top"), ("footerFromBottom", "Footer from Bottom")] {
            let mut value = state.get(key).and_then(Value::as_f64).unwrap_or(36.0) as f32;
            ui.horizontal(|ui| {
                ui.label(tl!(label));
                let unit = crate::options::unit();
                if ui.add(egui::DragValue::new(&mut value).range(0.0..=300.0 / unit.pt_per_unit()).speed(0.5).measure(unit).max_decimals(2)).changed()
                {
                    let _ = app.run("hf.position", json!({(key): value}));
                }
            });
        }

        heading(ui, "Page Number Format", &tokens);
        let mut format = state.get("pageNumberFormat").and_then(Value::as_str).unwrap_or("decimal").to_string();
        egui::ComboBox::from_id_salt("hf_page_number_format").selected_text(tl!(format.as_str())).show_ui(ui, |ui| {
            for (value, label) in [
                ("decimal", "Decimal"),
                ("lowerRoman", "Lowercase Roman"),
                ("upperRoman", "Uppercase Roman"),
                ("lowerLetter", "Lowercase Letter"),
                ("upperLetter", "Uppercase Letter"),
                ("thaiNumbers", "Thai Numbers"),
                ("thaiLetters", "Thai Letters"),
            ] {
                ui.selectable_value(&mut format, value.to_string(), tl!(label));
            }
        });
        if state.get("pageNumberFormat").and_then(Value::as_str) != Some(&format) {
            let _ = app.run("layout.pageNumberFormat", json!({"format": format}));
        }
        let mut continues = state.get("pageNumberStart").is_none_or(Value::is_null);
        if ui.checkbox(&mut continues, tl!("Continue from Previous Section")).changed() {
            let params = if continues { json!({"continue": true}) } else { json!({"start": 1}) };
            let _ = app.run("layout.pageNumberFormat", params);
        }
        if !continues {
            let mut start = state.get("pageNumberStart").and_then(Value::as_u64).unwrap_or(1).min(100_000) as u32;
            if ui.add(egui::DragValue::new(&mut start).range(0..=100_000).prefix(tl!("Start at "))).changed() {
                let _ = app.run("layout.pageNumberFormat", json!({"start": start}));
            }
        }

        ui.separator();
        if ui.button(tl!("Close Header and Footer")).clicked() {
            let _ = app.run("insert.closeHeader", json!({}));
        }
    });
}

fn heading(ui: &mut Ui, title: &str, tokens: &crate::theme::Tokens) {
    ui.add_space(8.0);
    ui.painter().hline(ui.available_rect_before_wrap().x_range(), ui.cursor().top(), Stroke::new(1.0, tokens.border));
    ui.add_space(4.0);
    ui.label(egui::RichText::new(tl!(title)).small().strong().color(tokens.text_dim));
}

fn menu(ui: &mut Ui, app: &mut WordApp, label: &str, add: impl FnOnce(&mut Ui, &mut WordApp)) {
    ui.menu_button(tl!(label), |ui| add(ui, app));
}

fn item(ui: &mut Ui, app: &mut WordApp, label: &str, command: &str, params: Value) {
    if ui.button(tl!(label)).clicked() {
        let _ = app.run(command, params);
        ui.close();
    }
}

fn checkbox_command(ui: &mut Ui, app: &mut WordApp, state: &Value, key: &str, label: &str, command: &str) {
    let mut value = state.get(key).and_then(Value::as_bool).unwrap_or(false);
    if ui.checkbox(&mut value, tl!(label)).changed() {
        let _ = app.run(command, json!({"value": value}));
    }
}
