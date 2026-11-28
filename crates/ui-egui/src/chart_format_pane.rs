//! Format the selected chart's areas, text and effects through the shared chart model.

use doc4me_chart_format::{Compound, Dash, Edit, Fill, Format, Kind, Part};
use egui::Ui;
use serde_json::json;

use crate::WordApp;

const TAB: &str = "doc4me_chart_format_tab";
const PART: &str = "doc4me_chart_format_part";
const CLOSED: &str = "doc4me_chart_format_closed_selection";

fn chart_parts(chart: &doc4me_doc::chart::ChartSpec) -> Vec<(Part, String)> {
    let mut parts = vec![(Part::ChartArea, tl!("Chart Area").to_string()), (Part::PlotArea, tl!("Plot Area").to_string())];
    if chart.title.is_some() { parts.push((Part::Title, tl!("Chart Title").to_string())); }
    if chart.legend.is_some() { parts.push((Part::Legend, tl!("Legend").to_string())); }
    if !chart.kind.is_round() {
        parts.push((Part::CategoryAxis, tl!("Horizontal Axis").to_string()));
        parts.push((Part::ValueAxis, tl!("Vertical Axis").to_string()));
        parts.push((Part::Gridlines, tl!("Gridlines").to_string()));
    }
    for (i, series) in chart.series.iter().take(doc4me_chart_format::MAX_SERIES).enumerate() {
        let name = if series.name.trim().is_empty() { format!("{} {}", tl!("Series"), i + 1) } else { series.name.clone() };
        parts.push((Part::Series(i), name));
    }
    parts
}

fn apply(app: &mut WordApp, part: Part, edits: Vec<Edit>) {
    for edit in edits {
        let (key, value) = edit.to_pair();
        let _ = app.run("chart.format", json!({"part": doc4me_chart_format::part_id(part), "edit": key, "value": value}));
    }
}

fn number(ui: &mut Ui, label: &str, value: f32, range: std::ops::RangeInclusive<f32>, suffix: &str) -> Option<f32> {
    let mut value = value;
    let response = ui.add(egui::Slider::new(&mut value, range).text(tl!(label)).suffix(suffix));
    response.changed().then_some(value)
}

fn color(ui: &mut Ui, theme: &[doc4me_doc::Rgb], value: Option<[u8; 3]>) -> Option<[u8; 3]> {
    let mut text = value.map(|[r, g, b]| doc4me_doc::Rgb(r, g, b).hex()).unwrap_or_default();
    let before = text.clone();
    crate::dialogs::color_field(ui, theme, &mut text, tl!("Automatic"), "");
    if text != before { doc4me_doc::Rgb::parse(&text).map(|rgb| [rgb.0, rgb.1, rgb.2]) } else { None }
}

fn fill(ui: &mut Ui, label: &str, fill: Fill, theme: &[doc4me_doc::Rgb], id: &str, edits: &mut Vec<Edit>) {
    let mut kind = fill.kind();
    egui::ComboBox::from_label(tl!(label)).selected_text(match kind { Kind::Auto => tl!("Automatic"), Kind::None => tl!("None"), Kind::Solid => tl!("Solid") }).show_ui(ui, |ui| {
        ui.selectable_value(&mut kind, Kind::Auto, tl!("Automatic"));
        ui.selectable_value(&mut kind, Kind::None, tl!("None"));
        ui.selectable_value(&mut kind, Kind::Solid, tl!("Solid"));
    });
    if kind != fill.kind() { edits.push(if id == "text" { Edit::TextFill(kind) } else { Edit::Fill(kind) }); }
    let solid = fill.solid();
    if let Some(c) = color(ui, theme, solid.map(|(c, _)| c)) {
        edits.push(if id == "text" { Edit::TextColor(c) } else { Edit::FillColor(c) });
    }
    if let Some((_, transparency)) = solid {
        let pct = (1.0 - transparency) * 100.0;
        if let Some(v) = number(ui, "Transparency", pct, 0.0..=100.0, "%") {
            edits.push(if id == "text" { Edit::TextTransparency(v) } else { Edit::FillTransparency(v) });
        }
    }
}

fn style_ui(ui: &mut Ui, tab: usize, part: Part, format: Format, rounded: bool, theme: &[doc4me_doc::Rgb]) -> Vec<Edit> {
    let mut edits = Vec::new();
    if tab == 1 && part.has_fill() {
        ui.collapsing(tl!("Fill"), |ui| fill(ui, "Fill", format.fill, theme, "fill", &mut edits));
    }
    if tab == 1 { ui.collapsing(tl!("Border"), |ui| {
        let mut kind = format.border.kind();
        egui::ComboBox::from_label(tl!("Border")).selected_text(match kind { Kind::Auto => tl!("Automatic"), Kind::None => tl!("No line"), Kind::Solid => tl!("Solid") }).show_ui(ui, |ui| {
            ui.selectable_value(&mut kind, Kind::Auto, tl!("Automatic"));
            ui.selectable_value(&mut kind, Kind::None, tl!("No line"));
            ui.selectable_value(&mut kind, Kind::Solid, tl!("Solid"));
        });
        if kind != format.border.kind() { edits.push(Edit::Border(kind)); }
        let line = format.border.line().copied();
        if let Some(c) = color(ui, theme, line.map(|l| l.color)) { edits.push(Edit::BorderColor(c)); }
        let l = line.unwrap_or_default();
        if let Some(v) = number(ui, "Width", l.width, 0.0..=20.0, " pt") { edits.push(Edit::BorderWidth(v)); }
        if line.is_some() && let Some(v) = number(ui, "Transparency", l.transparency, 0.0..=100.0, "%") { edits.push(Edit::BorderTransparency(v)); }
        let mut compound = l.compound;
        egui::ComboBox::from_label(tl!("Compound type")).selected_text(tl!(compound.name())).show_ui(ui, |ui| {
            for c in Compound::ALL { ui.selectable_value(&mut compound, c, tl!(c.name())); }
        });
        if compound != l.compound { edits.push(Edit::Compound(compound)); }
        let mut dash = l.dash;
        egui::ComboBox::from_label(tl!("Dash type")).selected_text(tl!(dash.name())).show_ui(ui, |ui| {
            for d in Dash::ALL { ui.selectable_value(&mut dash, d, tl!(d.name())); }
        });
        if dash != l.dash { edits.push(Edit::Dash(dash)); }
    }); }
    if tab == 1 && part == Part::ChartArea {
        let mut on = rounded;
        if ui.checkbox(&mut on, tl!("Rounded corners")).changed() { edits.push(Edit::RoundedCorners(on)); }
    }
    if tab == 2 && part.has_effects() {
        ui.collapsing(tl!("Effects"), |ui| {
            let mut shadow = format.shadow.is_some();
            if ui.checkbox(&mut shadow, tl!("Shadow")).changed() { edits.push(Edit::Shadow(shadow)); }
            if shadow {
                let s = format.shadow.unwrap_or_default();
                if let Some(c) = color(ui, theme, Some(s.color)) { edits.push(Edit::ShadowColor(c)); }
                if let Some(v) = number(ui, "Blur", s.blur, 0.0..=100.0, " pt") { edits.push(Edit::ShadowBlur(v)); }
                if let Some(v) = number(ui, "Angle", s.angle, 0.0..=359.0, "°") { edits.push(Edit::ShadowAngle(v)); }
                if let Some(v) = number(ui, "Distance", s.distance, 0.0..=200.0, " pt") { edits.push(Edit::ShadowDistance(v)); }
                if let Some(v) = number(ui, "Transparency", s.transparency, 0.0..=100.0, "%") { edits.push(Edit::ShadowTransparency(v)); }
            }
            let glow = format.glow.unwrap_or_default();
            if let Some(v) = number(ui, "Glow size", if format.glow.is_some() { glow.size } else { 0.0 }, 0.0..=150.0, " pt") { edits.push(Edit::GlowSize(v)); }
            if format.glow.is_some() {
                if let Some(c) = color(ui, theme, Some(glow.color)) { edits.push(Edit::GlowColor(c)); }
                if let Some(v) = number(ui, "Transparency", glow.transparency, 0.0..=100.0, "%") { edits.push(Edit::GlowTransparency(v)); }
            }
            if let Some(v) = number(ui, "Soft edges", format.soft_edge, 0.0..=100.0, " pt") { edits.push(Edit::SoftEdge(v)); }
        });
    }
    if tab == 2 && !part.has_effects() {
        ui.label(egui::RichText::new(tl!("Gridlines have no effects.")).weak());
    }
    if tab == 3 && part.has_text() {
        ui.collapsing(tl!("Text"), |ui| {
            fill(ui, "Text Fill", format.text.fill, theme, "text", &mut edits);
            let mut outline = format.text.outline.is_some();
            if ui.checkbox(&mut outline, tl!("Text Outline")).changed() { edits.push(Edit::TextOutline(outline)); }
            if outline {
                let o = format.text.outline.unwrap_or_default();
                if let Some(c) = color(ui, theme, Some(o.color)) { edits.push(Edit::TextOutlineColor(c)); }
                if let Some(v) = number(ui, "Width", o.width, 0.0..=20.0, " pt") { edits.push(Edit::TextOutlineWidth(v)); }
            }
            let mut size = format.text.size.unwrap_or(10.0);
            let mut auto = format.text.size.is_none();
            if ui.checkbox(&mut auto, tl!("Automatic font size")).changed() { edits.push(Edit::FontSize(if auto { None } else { Some(size) })); }
            if !auto && let Some(v) = number(ui, "Font size", size, 1.0..=400.0, " pt") { size = v; edits.push(Edit::FontSize(Some(size))); }
        });
    }
    if tab == 3 && !part.has_text() {
        ui.label(egui::RichText::new(tl!("This part shows no text.")).weak());
    }
    if tab != 0 && !format.is_auto() && ui.button(tl!("Reset to Automatic")).clicked() { edits.push(Edit::Reset); }
    edits
}

fn chart_ui(app: &mut WordApp, ui: &mut Ui, chart: &doc4me_doc::chart::ChartSpec) {
    ui.collapsing(tl!("Chart Layouts"), |ui| {
        let mut title = chart.title.clone().unwrap_or_else(|| tl!("Chart Title").to_string());
        ui.horizontal(|ui| {
            if ui.text_edit_singleline(&mut title).changed() { let _ = app.run("chart.title", json!({"text": title})); }
            if chart.title.is_some() && ui.button(tl!("Remove")).clicked() { let _ = app.run("chart.title", json!({"text": null})); }
        });
        let mut legend = chart.legend.map(|x| x.id()).unwrap_or("none").to_string();
        let legend_label = match legend.as_str() {
            "right" => tl!("Right"),
            "left" => tl!("Left"),
            "top" => tl!("Top"),
            "bottom" => tl!("Bottom"),
            _ => tl!("None"),
        };
        egui::ComboBox::from_label(tl!("Legend")).selected_text(legend_label).show_ui(ui, |ui| {
            for (id, name) in [("none", "None"), ("right", "Right"), ("left", "Left"), ("top", "Top"), ("bottom", "Bottom")] {
                ui.selectable_value(&mut legend, id.to_string(), tl!(name));
            }
        });
        if legend != chart.legend.map(|x| x.id()).unwrap_or("none") { let _ = app.run("chart.legend", json!({"pos": legend})); }
        let mut labels = chart.data_labels;
        if ui.checkbox(&mut labels, tl!("Data Labels")).changed() { let _ = app.run("chart.dataLabels", json!({"on": labels})); }
    });
    ui.collapsing(tl!("Data and Type"), |ui| {
        let mut kind = chart.kind;
        egui::ComboBox::from_label(tl!("Chart Type")).selected_text(tl!(kind.label())).show_ui(ui, |ui| {
            for option in doc4me_doc::chart::ChartType::ALL { ui.selectable_value(&mut kind, option, tl!(option.label())); }
        });
        if kind != chart.kind { let _ = app.run("chart.type", json!({"type": kind.id()})); }
        if ui.button(tl!("Edit Data")).clicked() { let _ = app.run("chart.editData", json!({})); }
    });
    ui.collapsing(tl!("Size"), |ui| {
        if let Some((_, object)) = doc4me_engine::cmd::objects::selected(&app.session)
            && let Some((width, height, _)) = object.frame()
        {
            for (label, key, current) in [("Width", "width", width), ("Height", "height", height)] {
                let mut value = current;
                if ui.add(egui::DragValue::new(&mut value).speed(0.5).range(36.0..=2000.0).suffix(" pt").prefix(format!("{}: ", tl!(label)))).changed() {
                    let _ = app.run("picture.size", json!({key: value, "lockAspect": false}));
                }
            }
        }
        let locked = doc4me_engine::cmd::objects::selection_locked(&app.session);
        let label = if locked { "Unlock" } else { "Lock" };
        if ui.button(tl!(label)).clicked() { let _ = app.run("shape.lock", json!({"locked": !locked})); }
    });
}

pub fn show(app: &mut WordApp, ui: &mut Ui) {
    let Some((pos, doc4me_doc::para::InlineObject::Graphic { graphic, .. })) = doc4me_engine::cmd::objects::selected(&app.session) else { return };
    let Some(chart) = graphic.chart.clone() else { return };
    let selection = format!("{pos:?}");
    let closed = egui::Id::new(CLOSED);
    if ui.data(|d| d.get_temp::<String>(closed)) == Some(selection.clone()) { return; }
    let theme = app.session.doc.settings.theme_colors.clone();
    let parts = chart_parts(&chart);
    egui::Panel::right("chart_format_pane")
        .default_size(310.0)
        .resizable(true)
        .frame(egui::Frame::NONE.fill(crate::theme::Tokens::get(ui.ctx()).panel).inner_margin(10).stroke(egui::Stroke::new(1.0, crate::theme::Tokens::get(ui.ctx()).border)))
        .show(ui, |ui| {
            crate::ribbon_keys::stop(ui, crate::ribbon_keys::Region::Pane, ui.max_rect());
            if crate::panes::header(ui, "Chart") {
                ui.data_mut(|d| d.insert_temp(closed, selection.clone()));
                return;
            }
            let locked = doc4me_engine::cmd::objects::selection_locked(&app.session);
            if locked {
                ui.horizontal(|ui| {
                    ui.label(tl!("Locked: it can't be moved, resized or edited."));
                    if ui.button(tl!("Unlock")).clicked() { let _ = app.run("shape.lock", json!({"locked": false})); }
                });
            }
            ui.add_enabled_ui(!locked, |ui| {
            let mut tab = ui.data(|d| d.get_temp::<usize>(egui::Id::new(TAB))).unwrap_or(0).min(3);
            ui.horizontal_wrapped(|ui| {
                for (i, label) in ["Chart", "Fill and Line", "Effects", "Text"].iter().enumerate() {
                    if ui.selectable_label(tab == i, tl!(label)).clicked() { tab = i; }
                }
            });
            ui.data_mut(|d| d.insert_temp(egui::Id::new(TAB), tab));
            if tab == 0 {
                chart_ui(app, ui, &chart);
                return;
            }
            let mut chosen = ui.data(|d| d.get_temp::<String>(egui::Id::new(PART))).and_then(|id| doc4me_chart_format::part_from_id(&id)).filter(|p| parts.iter().any(|(part, _)| p == part)).unwrap_or(Part::ChartArea);
            egui::ComboBox::from_label(tl!("Format")).selected_text(parts.iter().find(|(p, _)| *p == chosen).map(|(_, name)| name.as_str()).unwrap_or("Chart Area")).show_ui(ui, |ui| {
                for (part, label) in &parts { ui.selectable_value(&mut chosen, *part, label); }
            });
            ui.data_mut(|d| d.insert_temp(egui::Id::new(PART), doc4me_chart_format::part_id(chosen)));
            let edits = style_ui(ui, tab, chosen, chart.format.get(chosen), chart.format.rounded_corners, &theme);
            apply(app, chosen, edits);
            });
        });
}
