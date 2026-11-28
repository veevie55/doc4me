//! Define New Multilevel List (Home › Paragraph › Multilevel List) and Track Changes Options
//! (Review › Tracking). Each fills a form from the session and ends in one command
//! (`list.define`, `review.trackingOptions`), so agents get the same result without the dialog.

use doc4me_doc::section::NumFormat;
use doc4me_engine::cmd::lists::{LEVELS, caret_list, level_json, levels_of};
use egui::{Sense, Ui, vec2};
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::dialogs::buttons;
use crate::theme::{Tokens, semibold};
use crate::{WordApp, options::MeasureExt as _};

/// Number styles a level can have: OOXML name and a sample.
const STYLES: [(&str, &str); 13] = [
    ("decimal", "1, 2, 3, …"),
    ("lowerLetter", "a, b, c, …"),
    ("upperLetter", "A, B, C, …"),
    ("lowerRoman", "i, ii, iii, …"),
    ("upperRoman", "I, II, III, …"),
    ("decimalZero", "01, 02, 03, …"),
    ("ordinal", "1st, 2nd, 3rd, …"),
    ("cardinalText", "One, Two, Three, …"),
    ("thaiNumbers", "๑, ๒, ๓, …"),
    ("thaiLetters", "ก, ข, ค, …"),
    ("thaiCounting", "หนึ่ง, สอง, สาม, …"),
    ("bullet", "•"),
    ("none", ""),
];

/// One level of the Define New Multilevel List form. Positions are in the interface unit.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelForm {
    pub text: String,
    pub format: String,
    pub start: u32,
    pub restart: bool,
    /// 1-based level it restarts after.
    pub restart_after: u8,
    /// Linked paragraph style id, empty for none.
    pub style: String,
    pub align: String,
    pub aligned_at: f32,
    pub indent_at: f32,
    pub follow: String,
    pub tab_on: bool,
    pub tab_at: f32,
    pub font: String,
    pub bold: bool,
    pub italic: bool,
    pub legal: bool,
}

/// A multilevel list saved by name in My Lists: positions in `levels` remain points when the
/// user's measurement unit changes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedList {
    pub name: String,
    pub levels: Value,
}

/// At most this many saved lists; names are trimmed and cut to 64 characters.
pub const MAX_SAVED_LISTS: usize = 50;
const MAX_LIST_NAME: usize = 64;

/// Settings may contain older or malformed entries: keep valid saved lists and drop the rest.
pub fn lenient_saved_lists<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<SavedList>, D::Error> {
    let v = Value::deserialize(d)?;
    Ok(v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| serde_json::from_value::<SavedList>(x.clone()).ok())
                .filter(|l| !l.name.trim().is_empty() && l.levels.is_array())
                .take(MAX_SAVED_LISTS)
                .collect()
        })
        .unwrap_or_default())
}

impl SavedList {
    /// How the first three levels number, as shown in the ribbon menu.
    pub fn sample(&self) -> String {
        let levels = (0..LEVELS).map(|i| LevelForm::read(i, self.levels.get(i).unwrap_or(&Value::Null))).collect();
        let f = ListForm { levels, level: 0, styles: Vec::new(), name: String::new(), save: false, target: None };
        (0..3).map(|k| f.sample(k)).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ")
    }

    /// `list.define` parameters to apply this list to the selected paragraphs.
    pub fn params(&self) -> Value {
        json!({"levels": self.levels, "name": self.name})
    }
}

impl LevelForm {
    fn read(i: usize, v: &Value) -> LevelForm {
        let k = crate::options::unit().pt_per_unit();
        let s = |key: &str| v.get(key).and_then(Value::as_str).unwrap_or("").to_string();
        let f = |key: &str| v.get(key).and_then(Value::as_f64).map(|x| x as f32 / k).filter(|x| x.is_finite());
        let restart_after = v.get("restartAfter").and_then(Value::as_u64);
        LevelForm {
            text: s("text"),
            format: s("format"),
            start: v.get("start").and_then(Value::as_u64).unwrap_or(1).min(32_767) as u32,
            restart: restart_after != Some(0),
            restart_after: restart_after.filter(|r| *r > 0).map(|r| r.min(8) as u8).unwrap_or(i as u8).max(1),
            style: s("style"),
            align: s("align"),
            aligned_at: f("alignedAt").unwrap_or(0.0),
            indent_at: f("indentAt").unwrap_or(0.0),
            follow: s("follow"),
            tab_on: f("tabAt").is_some(),
            tab_at: f("tabAt").or(f("indentAt")).unwrap_or(0.0),
            font: s("font"),
            bold: v.get("bold").and_then(Value::as_bool).unwrap_or(false),
            italic: v.get("italic").and_then(Value::as_bool).unwrap_or(false),
            legal: v.get("legal").and_then(Value::as_bool).unwrap_or(false),
        }
    }

    /// The level as `list.define` takes it.
    pub fn params(&self, i: usize) -> Value {
        let k = crate::options::unit().pt_per_unit();
        json!({
            "format": self.format,
            "text": self.text,
            "start": self.start,
            "restartAfter": if !self.restart { json!(0) } else if i > 0 && (self.restart_after as usize) < i { json!(self.restart_after) } else { Value::Null },
            "style": if self.style.is_empty() { Value::Null } else { json!(self.style) },
            "align": self.align,
            "alignedAt": self.aligned_at * k,
            "indentAt": self.indent_at * k,
            "follow": self.follow,
            "tabAt": if self.tab_on && self.follow == "tab" { json!(self.tab_at * k) } else { Value::Null },
            "font": if self.font.trim().is_empty() { Value::Null } else { json!(self.font.trim()) },
            "bold": self.bold,
            "italic": self.italic,
            "legal": self.legal,
        })
    }
}

/// The Define New Multilevel List form: nine levels and the one being edited.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListForm {
    pub levels: Vec<LevelForm>,
    pub level: usize,
    /// Paragraph styles a level can link to: (id, name).
    #[serde(skip)]
    pub styles: Vec<(String, String)>,
    /// The list's name (kept with it in the document and shown in My Lists if saved).
    pub name: String,
    /// Save in My Lists, for every document.
    pub save: bool,
    #[serde(default)]
    pub target: Option<u32>,
}

impl ListForm {
    /// Starts from the caret's list, or from a new 1. a. i. list.
    pub fn read(app: &WordApp) -> ListForm {
        let s = &app.session;
        let levels = caret_list(s).and_then(|n| levels_of(s, n)).unwrap_or_else(|| doc4me_doc::numbering::levels_for(doc4me_doc::ListKind::Numbered));
        let levels = (0..LEVELS).map(|i| LevelForm::read(i, &levels.get(i).map(level_json).unwrap_or(Value::Null))).collect();
        let mut styles: Vec<(String, String)> = s
            .doc
            .styles
            .styles
            .iter()
            .filter(|st| st.kind == doc4me_doc::StyleKind::Paragraph && !st.hidden)
            .map(|st| (st.id.clone(), st.name.clone()))
            .collect();
        styles.sort_by(|a, b| a.1.cmp(&b.1));
        ListForm { levels, level: 0, styles, name: String::new(), save: false, target: None }
    }

    /// `list.define` parameters for the whole list.
    pub fn params(&self) -> Value {
        let levels: Vec<Value> = self.levels.iter().enumerate().map(|(i, l)| l.params(i)).collect();
        let name = self.name.trim();
        let mut params = if name.is_empty() { json!({"levels": levels}) } else { json!({"levels": levels, "name": name}) };
        if let Some(num) = self.target {
            params["num"] = json!(num);
        }
        params
    }

    /// Save or replace a named list, if the user checked the option.
    pub fn save_to(&self, saved: &mut Vec<SavedList>) {
        let name: String = self.name.trim().chars().take(MAX_LIST_NAME).collect();
        if !self.save || name.is_empty() {
            return;
        }
        let levels = self.params().get("levels").cloned().unwrap_or(Value::Null);
        saved.retain(|list| list.name != name);
        saved.push(SavedList { name, levels });
        if saved.len() > MAX_SAVED_LISTS {
            saved.remove(0);
        }
    }

    /// The number level `k` shows with every level at its start value, as in the preview.
    pub fn sample(&self, k: usize) -> String {
        let Some(l) = self.levels.get(k) else { return String::new() };
        match l.format.as_str() {
            "none" => return String::new(),
            "bullet" => return l.text.clone(),
            _ => {}
        }
        let mut out = String::new();
        for piece in crate::number_text::parse(&l.text) {
            match piece {
                crate::number_text::Piece::Level(level) => out.push_str(&self.level_number(k, usize::from(level) - 1)),
                crate::number_text::Piece::Text(c) => out.push(c),
            }
        }
        out
    }

    /// Level `j`'s sample number as level `k` displays it, including legal numbering.
    pub fn level_number(&self, k: usize, j: usize) -> String {
        let j = j.min(LEVELS - 1);
        let legal = self.levels.get(k).is_some_and(|level| level.legal);
        let level = self.levels.get(j);
        let format = if legal && j != k { NumFormat::Decimal } else { level.map(|x| NumFormat::from_ooxml(&x.format)).unwrap_or_default() };
        format.format(level.map(|x| x.start).unwrap_or(1).max(1))
    }
}

/// Define New Multilevel List. Returns true to close.
pub fn define_list(app: &mut WordApp, ui: &mut Ui, f: &mut ListForm) -> bool {
    let unit = crate::options::unit();
    fn len(v: &mut f32, unit: doc4me_geom::Unit) -> egui::DragValue<'_> {
        egui::DragValue::new(v).speed(0.01).range(-10.0..=22.0).clamp_existing_to_range(false).measure(unit).max_decimals(2)
    }
    let heading = |ui: &mut Ui, s: &str| {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(tl!(s)).font(semibold(12.5)));
    };
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.label(tl!("Click level to modify:"));
            for i in 0..LEVELS {
                if ui.selectable_label(f.level == i, format!("{}", i + 1)).clicked() {
                    f.level = i;
                }
            }
        });
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(tl!("Preview")).small().weak());
            preview(ui, f);
        });
    });
    let i = f.level.min(LEVELS - 1);
    let styles = f.styles.clone();
    let chips: Vec<String> = (0..LEVELS).map(|level| f.level_number(i, level)).collect();
    let Some(l) = f.levels.get_mut(i) else { return true };
    heading(ui, "Number format");
    egui::Grid::new("mll_number").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
        ui.label(tl!("Enter formatting for number:"));
        let text_id = egui::Id::new(("mll_number_text", i));
        crate::number_text::show(ui, text_id, tl!("Number format"), &mut l.text, 260.0, |level| {
            chips.get(usize::from(level).saturating_sub(1)).cloned().unwrap_or_default()
        });
        ui.end_row();
        ui.label(tl!("Number style for this level:"));
        let current = STYLES.iter().find(|(k, _)| *k == l.format).map(|(_, s)| *s).unwrap_or("");
        egui::ComboBox::from_id_salt("mll_style").selected_text(style_label(current)).show_ui(ui, |ui| {
            for (k, s) in STYLES {
                if ui.selectable_label(l.format == k, style_label(s)).clicked() {
                    // A new style puts this level's number in the text if it had none (a bullet
                    // its character).
                    if k == "bullet" {
                        l.text = "•".into();
                    } else if l.format == "bullet" || !l.text.contains('%') {
                        l.text = format!("%{}.", i + 1);
                    }
                    l.format = k.to_string();
                }
            }
        });
        ui.end_row();
        if i > 0 {
            ui.label(tl!("Include level number from:"));
            egui::ComboBox::from_id_salt("mll_include").selected_text("").show_ui(ui, |ui| {
                for k in 0..i {
                    if ui.selectable_label(false, crate::i18n::fmt(tl!("Level {n}"), &[("n", &(k + 1).to_string())])).clicked() {
                        crate::number_text::insert_level_at_cursor(ui, text_id, &mut l.text, (k + 1) as u8);
                    }
                }
            });
            ui.end_row();
        }
        ui.label(tl!("Start at:"));
        ui.add(egui::DragValue::new(&mut l.start).range(0..=32_767).clamp_existing_to_range(false));
        ui.end_row();
        if i > 0 {
            ui.checkbox(&mut l.restart, tl!("Restart list after:"));
            ui.add_enabled_ui(l.restart, |ui| {
                let shown = crate::i18n::fmt(tl!("Level {n}"), &[("n", &l.restart_after.clamp(1, i as u8).to_string())]);
                egui::ComboBox::from_id_salt("mll_restart").selected_text(shown).show_ui(ui, |ui| {
                    for k in 1..=i as u8 {
                        ui.selectable_value(&mut l.restart_after, k, crate::i18n::fmt(tl!("Level {n}"), &[("n", &k.to_string())]));
                    }
                });
            });
            ui.end_row();
        }
        ui.label(tl!("Link level to style:"));
        let shown = styles.iter().find(|(id, _)| *id == l.style).map(|(_, n)| n.clone()).unwrap_or_else(|| tl!("(no style)").to_string());
        egui::ComboBox::from_id_salt("mll_link").selected_text(shown).show_ui(ui, |ui| {
            ui.selectable_value(&mut l.style, String::new(), tl!("(no style)"));
            for (id, name) in &styles {
                ui.selectable_value(&mut l.style, id.clone(), name);
            }
        });
        ui.end_row();
        ui.label(tl!("Font:"));
        ui.horizontal(|ui| {
            ui.add(crate::widgets::text_line(&mut l.font).desired_width(110.0));
            ui.checkbox(&mut l.bold, tl!("Bold"));
            ui.checkbox(&mut l.italic, tl!("Italic"));
        });
        ui.end_row();
        ui.label("");
        ui.checkbox(&mut l.legal, tl!("Legal style numbering"));
        ui.end_row();
    });
    heading(ui, "Position");
    egui::Grid::new("mll_position").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
        ui.label(tl!("Number alignment:"));
        ui.horizontal(|ui| {
            for (v, s) in [("left", "Left"), ("center", "Center"), ("right", "Right")] {
                ui.radio_value(&mut l.align, v.to_string(), tl!(s));
            }
        });
        ui.end_row();
        ui.label(tl!("Aligned at:"));
        ui.add(len(&mut l.aligned_at, unit));
        ui.end_row();
        ui.label(tl!("Text indent at:"));
        ui.add(len(&mut l.indent_at, unit));
        ui.end_row();
        ui.label(tl!("Follow number with:"));
        let follow = |k: &str| match k {
            "space" => tl!("Space"),
            "nothing" => tl!("Nothing"),
            _ => tl!("Tab character"),
        };
        egui::ComboBox::from_id_salt("mll_follow").selected_text(follow(&l.follow)).show_ui(ui, |ui| {
            for k in ["tab", "space", "nothing"] {
                ui.selectable_value(&mut l.follow, k.to_string(), follow(k));
            }
        });
        ui.end_row();
        ui.add_enabled(l.follow == "tab", egui::Checkbox::new(&mut l.tab_on, tl!("Add tab stop at:")));
        ui.add_enabled(l.follow == "tab" && l.tab_on, len(&mut l.tab_at, unit));
        ui.end_row();
    });
    heading(ui, "Name");
    ui.horizontal(|ui| {
        ui.label(tl!("List name:"));
        ui.add(egui::TextEdit::singleline(&mut f.name).desired_width(200.0).char_limit(MAX_LIST_NAME).hint_text(tl!("Optional")));
    });
    ui.add_enabled(!f.name.trim().is_empty(), egui::Checkbox::new(&mut f.save, tl!("Save in My Lists, for every document")));
    let (ok, cancel) = buttons(ui, tl!("OK"));
    if ok {
        match app.run("list.define", f.params()) {
            Ok(_) => f.save_to(&mut app.ui.saved_lists),
            Err(e) => app.status(e),
        }
    }
    ok || cancel
}

fn style_label(s: &str) -> String {
    match s {
        "" => tl!("None").to_string(),
        "•" => format!("• {}", tl!("Bullet")),
        s => s.to_string(),
    }
}

/// All nine levels, each number at its aligned-at position followed by a grey line of text at
/// its text indent; the level being edited in the text colour.
fn preview(ui: &mut Ui, f: &ListForm) {
    let t = Tokens::get(ui.ctx());
    let row = 19.0;
    let (rect, _) = ui.allocate_exact_size(vec2(300.0, row * LEVELS as f32 + 8.0), Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 2.0, egui::Color32::WHITE);
    p.rect_stroke(rect, 2.0, egui::Stroke::new(1.0, t.border_strong), egui::StrokeKind::Inside);
    let k = crate::options::unit().pt_per_unit();
    // Positions in points, scaled so 4.5 inches fit.
    let scale = (rect.width() - 16.0) / 324.0;
    let x_of = |v: f32| rect.left() + 8.0 + (v * k * scale).clamp(0.0, rect.width() - 40.0);
    for (i, l) in f.levels.iter().enumerate() {
        let y = rect.top() + 4.0 + row * i as f32 + row / 2.0;
        let on = i == f.level;
        let ink = if on { egui::Color32::BLACK } else { egui::Color32::from_gray(150) };
        let label = f.sample(i);
        let font = egui::FontId::proportional(10.5);
        let galley = p.layout_no_wrap(label, font, ink);
        let w = galley.size().x;
        let x = x_of(l.aligned_at);
        let nx = match l.align.as_str() {
            "center" => x - w / 2.0,
            "right" => x - w,
            _ => x,
        };
        p.galley(egui::pos2(nx, y - galley.size().y / 2.0), galley, ink);
        let tx = match l.follow.as_str() {
            "space" => nx + w + 3.0,
            "nothing" => nx + w,
            _ => x_of(l.indent_at).max(nx + w + 2.0),
        };
        let bar = egui::Rect::from_min_max(egui::pos2(tx, y - 2.0), egui::pos2(rect.right() - 8.0, y + 2.0));
        if bar.width() > 0.0 {
            p.rect_filled(bar, 1.0, if on { egui::Color32::from_gray(120) } else { egui::Color32::from_gray(205) });
        }
    }
}

/// Track Changes Options (Review › Tracking): what markup shows and how revisions look.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackForm {
    pub comments: bool,
    pub ink: bool,
    pub insertions_deletions: bool,
    pub formatting: bool,
    pub balloons: String,
    pub insert_mark: String,
    /// Hex, empty = by author.
    pub insert_color: String,
    pub delete_mark: String,
    pub delete_color: String,
    pub changed_lines: String,
    /// Hex, empty = automatic.
    pub changed_lines_color: String,
    pub track_formatting: bool,
}

impl TrackForm {
    pub fn read(app: &WordApp) -> TrackForm {
        let v = doc4me_engine::cmd::review::tracking_json(&app.session.prefs.markup);
        let s = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        let c = |k: &str| doc4me_doc::Rgb::parse(&s(k)).map(|c| c.hex()).unwrap_or_default();
        let b = |k: &str| v.get(k).and_then(Value::as_bool).unwrap_or(true);
        TrackForm {
            comments: b("comments"),
            ink: b("ink"),
            insertions_deletions: b("insertionsDeletions"),
            formatting: b("formatting"),
            balloons: s("balloons"),
            insert_mark: s("insertMark"),
            insert_color: c("insertColor"),
            delete_mark: s("deleteMark"),
            delete_color: c("deleteColor"),
            changed_lines: s("changedLines"),
            changed_lines_color: c("changedLinesColor"),
            track_formatting: b("trackFormatting"),
        }
    }

    /// `review.trackingOptions` parameters.
    pub fn params(&self) -> Value {
        let or = |c: &str, auto: &str| if c.is_empty() { auto.to_string() } else { c.to_string() };
        json!({
            "comments": self.comments,
            "ink": self.ink,
            "insertionsDeletions": self.insertions_deletions,
            "formatting": self.formatting,
            "balloons": self.balloons,
            "insertMark": self.insert_mark,
            "insertColor": or(&self.insert_color, "byAuthor"),
            "deleteMark": self.delete_mark,
            "deleteColor": or(&self.delete_color, "byAuthor"),
            "changedLines": self.changed_lines,
            "changedLinesColor": or(&self.changed_lines_color, "auto"),
            "trackFormatting": self.track_formatting,
        })
    }
}

/// A choice from `options` ((value, label)) in a drop-down.
fn choice(ui: &mut Ui, id: &str, value: &mut String, options: &[(&str, &str)]) {
    let label = |k: &str| options.iter().find(|(v, _)| *v == k).map(|(_, l)| tl!(l).to_string()).unwrap_or_default();
    egui::ComboBox::from_id_salt(id).selected_text(label(value)).width(150.0).show_ui(ui, |ui| {
        for (v, l) in options {
            ui.selectable_value(value, v.to_string(), tl!(l));
        }
    });
}

/// A colour: `none_label` (by author, or `auto` when given) or a colour from the grid. `value` is
/// hex.
fn color(ui: &mut Ui, theme: &[doc4me_doc::Rgb], value: &mut String, none_label: &str, auto: Option<doc4me_doc::Rgb>) {
    ui.horizontal(|ui| {
        let id = ui.id().with("colorField");
        crate::colors::take_field(ui.ctx(), id, value);
        let c = doc4me_doc::Rgb::parse(value);
        let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
        let t = Tokens::get(ui.ctx());
        match c.or(auto) {
            Some(c) => {
                ui.painter().rect_filled(r, 2.0, crate::theme::c32(c));
            }
            // By author: the first author colours, side by side.
            None => {
                for (k, a) in [0u32, 1, 2].iter().enumerate() {
                    let w = r.width() / 3.0;
                    let part = egui::Rect::from_min_size(r.min + vec2(w * k as f32, 0.0), vec2(w, r.height()));
                    ui.painter().rect_filled(part, 0.0, crate::theme::c32(doc4me_layout::display::revision_color(*a)));
                }
            }
        }
        ui.painter().rect_stroke(r, 2.0, egui::Stroke::new(1.0, t.border_strong), egui::StrokeKind::Inside);
        let label = if c.is_some() { value.clone() } else { tl!(none_label).to_string() };
        ui.menu_button(label, |ui| {
            if ui.button(tl!(none_label)).clicked() {
                value.clear();
                ui.close();
            }
            crate::colors::field_menu(ui, theme, id, value);
        });
    });
}

/// Track Changes Options. Returns true to close.
pub fn track_options(app: &mut WordApp, ui: &mut Ui, f: &mut TrackForm) -> bool {
    let theme = app.session.doc.settings.theme_colors.clone();
    let heading = |ui: &mut Ui, s: &str| {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(tl!(s)).font(semibold(12.5)));
    };
    heading(ui, "Show");
    egui::Grid::new("tco_show").num_columns(2).spacing(vec2(16.0, 4.0)).show(ui, |ui| {
        ui.checkbox(&mut f.comments, tl!("Comments"));
        ui.checkbox(&mut f.ink, tl!("Ink"));
        ui.end_row();
        ui.checkbox(&mut f.insertions_deletions, tl!("Insertions and Deletions"));
        ui.checkbox(&mut f.formatting, tl!("Formatting"));
        ui.end_row();
    });
    heading(ui, "Balloons");
    for (v, l) in [
        ("revisions", "Show revisions in balloons"),
        ("inline", "Show all revisions inline"),
        ("commentsAndFormatting", "Show only comments and formatting in balloons"),
    ] {
        ui.radio_value(&mut f.balloons, v.to_string(), tl!(l));
    }
    heading(ui, "Advanced");
    egui::Grid::new("tco_advanced").num_columns(3).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
        ui.label(tl!("Insertions:"));
        choice(
            ui,
            "tco_ins",
            &mut f.insert_mark,
            &[
                ("underline", "Underline"),
                ("doubleUnderline", "Double Underline"),
                ("bold", "Bold"),
                ("italic", "Italic"),
                ("strikethrough", "Strikethrough"),
                ("colorOnly", "Color only"),
                ("none", "None"),
            ],
        );
        color(ui, &theme, &mut f.insert_color, "By author", None);
        ui.end_row();
        ui.label(tl!("Deletions:"));
        choice(
            ui,
            "tco_del",
            &mut f.delete_mark,
            &[
                ("strikethrough", "Strikethrough"),
                ("doubleStrikethrough", "Double Strikethrough"),
                ("hidden", "Hidden"),
                ("caret", "^"),
                ("hash", "#"),
                ("underline", "Underline"),
                ("colorOnly", "Color only"),
            ],
        );
        color(ui, &theme, &mut f.delete_color, "By author", None);
        ui.end_row();
        ui.label(tl!("Changed lines:"));
        choice(
            ui,
            "tco_bar",
            &mut f.changed_lines,
            &[("outside", "Outside border"), ("left", "Left border"), ("right", "Right border"), ("none", "None")],
        );
        color(ui, &theme, &mut f.changed_lines_color, "Automatic", Some(doc4me_layout::display::CHANGE_BAR));
        ui.end_row();
    });
    ui.add_space(4.0);
    ui.checkbox(&mut f.track_formatting, tl!("Track formatting"));
    let (ok, cancel) = buttons(ui, tl!("OK"));
    if ok && let Err(e) = app.run("review.trackingOptions", f.params()) {
        app.status(e);
    }
    ok || cancel
}

#[cfg(test)]
mod tests {
    use doc4me_engine::Session;
    use serde_json::json;

    use crate::dialogs::Dialog;
    use crate::{Services, WordApp};

    fn app() -> WordApp {
        WordApp::new(Session::new(doc4me_doc::Document::new()), Services::default())
    }

    /// #328: the ribbon opens Define New Multilevel List; OK defines the list and numbers the
    /// paragraph. Scripts without levels get an error, never the dialog.
    #[test]
    fn define_new_multilevel_list_opens_from_the_ribbon_and_applies() {
        let mut a = app();
        a.run("text.insert", json!({"text": "Intro"})).unwrap();
        assert_eq!(a.run("list.define", json!({})).unwrap(), json!({"pending": "defineList"}));
        let Some(Dialog::DefineList { form }) = &mut a.dialog else { panic!("dialog") };
        assert_eq!((form.levels.len(), form.sample(0), form.sample(1)), (9, "1.".to_string(), "a.".to_string()));
        form.levels[0].text = "Part %1:".into();
        form.levels[0].format = "upperRoman".into();
        form.levels[1].text = "%1.%2".into();
        form.levels[1].legal = true;
        assert_eq!((form.sample(0), form.sample(1)), ("Part I:".to_string(), "1.a".to_string()), "legal shows levels above in digits");
        let params = form.params();
        a.dialog = None;
        a.run("list.define", params).unwrap();
        let got = a.session.run("list.get", &json!({})).unwrap();
        assert_eq!((got["levels"][0]["text"].as_str(), got["levels"][0]["format"].as_str()), (Some("Part %1:"), Some("upperRoman")));
        // Opening it again starts from the caret's list.
        a.run("ui.dialog", json!({"name": "defineList"})).unwrap();
        assert!(matches!(&a.dialog, Some(Dialog::DefineList { form }) if form.sample(0) == "Part I:"));
        let mut b = app();
        assert!(b.execute("list.define", json!({})).is_err());
        assert!(b.dialog.is_none());
    }

    /// #328: the Tracking group's launcher opens Track Changes Options; OK changes the per-user
    /// preference that is saved with the interface settings.
    #[test]
    fn track_changes_options_open_from_the_launcher_and_persist() {
        let mut a = app();
        assert_eq!(a.run("review.trackingOptions", json!({})).unwrap(), json!({"pending": "trackChangesOptions"}));
        let Some(Dialog::TrackOptions { form }) = &mut a.dialog else { panic!("dialog") };
        assert_eq!((form.insert_mark.as_str(), form.balloons.as_str(), form.insert_color.as_str()), ("underline", "commentsAndFormatting", ""));
        form.insert_mark = "doubleUnderline".into();
        form.changed_lines = "right".into();
        form.changed_lines_color = "FF0000".into();
        let params = form.params();
        a.dialog = None;
        a.run("review.trackingOptions", params).unwrap();
        let m = &a.session.prefs.markup;
        assert_eq!(m.insert_mark, doc4me_layout::display::InsertMark::DoubleUnderline);
        assert_eq!(m.changed_lines_color, Some(doc4me_doc::Rgb(0xFF, 0, 0)));
        // Saved and restored with the preferences.
        let saved = a.prefs();
        let mut b = app();
        b.apply_prefs(saved);
        assert_eq!(b.session.prefs.markup.changed_lines, doc4me_layout::display::ChangeBar::Right);
    }
}
