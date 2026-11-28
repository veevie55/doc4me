//! Home › Paragraph › Bullets › Define New Bullet. The dialog creates a private list definition
//! through `para.defineBullet`, so it does not mutate other lists that happen to use the same mark.

use egui::{Align2, FontFamily, FontId, Sense, Ui, pos2, vec2};
use serde::Serialize;
use serde_json::{Value, json};

use crate::WordApp;
use crate::theme::{Tokens, regular, semibold};

const ALIGNMENTS: [(&str, &str); 3] = [("left", "Left"), ("center", "Center"), ("right", "Right")];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefineBulletForm {
    pub ch: String,
    /// Empty means to use the paragraph's font.
    pub font: String,
    /// Empty means to use the text's size.
    pub size: String,
    pub bold: bool,
    pub italic: bool,
    /// `RRGGBB`; `auto` uses the text's color.
    pub color: String,
    pub align: String,
}

impl DefineBulletForm {
    pub fn new(app: &WordApp) -> Self {
        let doc = &app.session.doc;
        let level = doc
            .para_at(&app.session.sel.focus)
            .and_then(|p| doc.styles.resolve_para(&p.props).numbering)
            .filter(|n| n.num != 0)
            .and_then(|n| doc.numbering.level(n.num, n.level))
            .filter(|level| level.format == doc4me_doc::section::NumFormat::Bullet && level.text.chars().count() == 1);
        if let Some(level) = level {
            Self {
                ch: level.text.clone(),
                font: level.chr.font.clone().unwrap_or_default(),
                size: level.chr.size.map(|size| size.to_string()).unwrap_or_default(),
                bold: level.chr.bold.unwrap_or(false),
                italic: level.chr.italic.unwrap_or(false),
                color: match level.chr.color {
                    Some(doc4me_doc::props::TextColor::Rgb(color)) => color.hex(),
                    _ => String::new(),
                },
                align: match level.align {
                    doc4me_doc::props::Align::Center => "center",
                    doc4me_doc::props::Align::Right => "right",
                    _ => "left",
                }
                .into(),
            }
        } else {
            Self::plain("•")
        }
    }

    fn plain(ch: &str) -> Self {
        Self { ch: ch.into(), font: String::new(), size: String::new(), bold: false, italic: false, color: String::new(), align: "left".into() }
    }

    fn preview_size(&self) -> f32 {
        self.size.trim().parse::<f32>().ok().filter(|size| size.is_finite() && (1.0..=1638.0).contains(size)).unwrap_or(11.0).clamp(5.0, 20.0)
    }

    fn params(&self) -> Value {
        let mut params = json!({
            "char": self.ch,
            "font": self.font.trim(),
            "bold": self.bold,
            "italic": self.italic,
            "align": self.align,
            "color": if doc4me_doc::Rgb::parse(&self.color).is_some() { self.color.as_str() } else { "auto" },
        });
        if !self.size.trim().is_empty() {
            params["size"] = match self.size.trim().parse::<f64>() {
                Ok(size) if size.is_finite() => json!(size),
                _ => json!(self.size.trim()),
            };
        }
        params
    }
}

/// Draw the dialog. An invalid value stays open so it can be corrected.
pub fn body(app: &mut WordApp, ui: &mut Ui, form: &mut DefineBulletForm) -> bool {
    let tokens = Tokens::get(ui.ctx());
    ui.set_width(360.0);
    ui.label(egui::RichText::new(tl!("Bullet character")).font(semibold(12.0)));
    ui.horizontal(|ui| {
        ui.add(crate::widgets::text_line(&mut form.ch).desired_width(70.0).char_limit(8));
        ui.menu_button(tl!("More Symbols…"), |ui| {
            for ch in ["•", "○", "▪", "◆", "★", "✓", "➢", "–", "†", "❖"] {
                if ui.button(egui::RichText::new(ch).size(17.0)).clicked() {
                    form.ch = ch.into();
                    ui.close();
                }
            }
        });
    });

    ui.add_space(5.0);
    ui.horizontal(|ui| {
        ui.label(tl!("Font"));
        ui.add(crate::widgets::text_line(&mut form.font).hint_text(tl!("Default Paragraph Font")).desired_width(175.0));
        let families = app.previews.families();
        egui::ComboBox::from_id_salt("defineBulletFontPicker").selected_text(tl!("Fonts")).show_ui(ui, |ui| {
            if ui.selectable_label(form.font.is_empty(), tl!("Default Paragraph Font")).clicked() {
                form.font.clear();
                ui.close();
            }
            for family in families {
                let selected = app.previews.font_shown(&form.font) == family;
                if ui.selectable_label(selected, family.as_str()).clicked() {
                    form.font = app.previews.font_stored(&family);
                    ui.close();
                }
            }
        });
    });

    ui.horizontal(|ui| {
        ui.label(tl!("Size"));
        ui.add(crate::widgets::text_line(&mut form.size).hint_text(tl!("Auto")).desired_width(70.0));
        ui.label(tl!("pt"));
        ui.checkbox(&mut form.bold, tl!("Bold"));
        ui.checkbox(&mut form.italic, tl!("Italic"));
    });
    ui.horizontal(|ui| {
        ui.label(tl!("Color"));
        let theme = app.session.doc.settings.theme_colors.clone();
        crate::dialogs::color_field(ui, &theme, &mut form.color, "Automatic", "auto");
    });
    ui.horizontal(|ui| {
        ui.label(tl!("Alignment:"));
        for (key, label) in ALIGNMENTS {
            ui.radio_value(&mut form.align, key.to_string(), tl!(label));
        }
    });

    ui.add_space(5.0);
    ui.label(egui::RichText::new(tl!("Preview")).font(regular(12.0)).color(tokens.text_dim));
    preview(ui, form);
    let enabled = form.ch.chars().count() == 1 && !form.ch.chars().any(char::is_control);
    let (ok, cancel) = crate::dialogs_insert::buttons(ui, "OK", "Cancel", enabled);
    if ok {
        if let Err(error) = app.run("para.defineBullet", form.params()) {
            app.status(error);
            return false;
        }
    }
    ok || cancel
}

fn preview(ui: &mut Ui, form: &DefineBulletForm) {
    let tokens = Tokens::get(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(vec2(360.0, 78.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, egui::Color32::WHITE);
    painter.rect_stroke(rect, 2.0, egui::Stroke::new(1.0, tokens.border_strong), egui::StrokeKind::Inside);
    let scale = (rect.width() - 20.0) / 324.0;
    let left = rect.left() + 10.0;
    let aligned_at = left + 18.0 * scale;
    let text_at = left + 36.0 * scale;
    let font_family = if form.font.trim().is_empty() { FontFamily::Proportional } else { FontFamily::Name(form.font.trim().to_string().into()) };
    let font = FontId::new(form.preview_size() * scale, font_family);
    let color = doc4me_doc::Rgb::parse(&form.color).map(crate::theme::c32).unwrap_or(egui::Color32::BLACK);
    for row in 0..3 {
        let y = rect.top() + 14.0 + row as f32 * 24.0;
        if row != 1 {
            painter.line_segment([pos2(left, y), pos2(rect.right() - 10.0, y)], egui::Stroke::new(3.0, egui::Color32::from_gray(205)));
            continue;
        }
        let glyph_width = font.size * 1.15;
        let x = match form.align.as_str() {
            "center" => aligned_at - glyph_width / 2.0,
            "right" => aligned_at - glyph_width,
            _ => aligned_at,
        };
        let at = pos2(x + glyph_width / 2.0, y);
        painter.text(at, Align2::CENTER_CENTER, &form.ch, font.clone(), color);
        if form.bold {
            painter.text(at + vec2(0.35, 0.0), Align2::CENTER_CENTER, &form.ch, font.clone(), color);
        }
        let start = text_at.max(x + glyph_width + 4.0);
        painter.line_segment([pos2(start, y), pos2(rect.right() - 10.0, y)], egui::Stroke::new(3.0, egui::Color32::from_gray(130)));
    }
}
