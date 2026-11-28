//! Modify Style (Home › Styles, Manage Styles, the Styles pane, Table of Contents › Modify…): a
//! window with a list of pages on the left — General, Font, Indents & Spacing, Alignment, Text Flow,
//! Tabs, Borders & Shading, Numbering, and for text Font Effects and Position — each with its icon —
//! and a
//! preview of the style as the boxes leave it. OK sends everything as one `styles.modify`.
//!
//! The Font page sets Latin and Thai (complex script) text side by side: Word keeps a separate
//! font, size, bold and italic for each, and a Thai heading showed one size in the size box and
//! another on the page when only the Latin ones could be seen.

use doc4me_doc::props::{Highlight, LineSpacing, TabAlign, TabLeader, TabStop, Underline, VertAlign};
use doc4me_doc::{Align, CharProps, ParaProps, StyleKind, TextColor};
use egui::{Ui, vec2};
use serde::Serialize;
use serde_json::{Value, json};

use crate::WordApp;
use crate::theme::semibold;

trait SteadyUi {
    fn steady_label(&mut self, selected: bool, text: impl Into<egui::WidgetText>) -> egui::Response;
    fn steady_value<T: PartialEq>(&mut self, current: &mut T, value: T, text: impl Into<egui::WidgetText>) -> egui::Response;
}

impl SteadyUi for Ui {
    fn steady_label(&mut self, selected: bool, text: impl Into<egui::WidgetText>) -> egui::Response {
        self.selectable_label(selected, text)
    }

    fn steady_value<T: PartialEq>(&mut self, current: &mut T, value: T, text: impl Into<egui::WidgetText>) -> egui::Response {
        self.selectable_value(current, value, text)
    }
}

/// The pages, in order: (id, name, icon).
/// No Asian Typography page (owner, 2026-11-25: no East Asian support).
const PAGES: [(&str, &str, &str); 13] = [
    ("general", "General", "styles"),
    ("font", "Font", "fonts"),
    ("effects", "Font Effects", "effects"),
    ("flow", "Text Flow", "pageBreak"),
    ("alignment", "Alignment", "justify"),
    ("indents", "Indents & Spacing", "paraSpacing"),
    ("position", "Position", "superscript"),
    ("dropcap", "Drop Caps", "dropCap"),
    ("highlight", "Highlighting", "highlight"),
    ("area", "Area", "shading"),
    ("borders", "Borders & Shading", "borders"),
    ("tabs", "Tabs", "ruler"),
    ("numbering", "Numbering", "multilevel"),
];

/// Underline styles offered: (OOXML name, label).
pub(crate) const UNDERLINES: [(&str, &str); 9] = [
    ("none", "(none)"),
    ("single", "Single"),
    ("words", "Words only"),
    ("double", "Double"),
    ("thick", "Thick"),
    ("dotted", "Dotted"),
    ("dash", "Dashed"),
    ("dotDash", "Dot dash"),
    ("wave", "Wave"),
];

/// Tab stop alignments and leaders offered: (model value, label).
const TAB_ALIGNS: [(TabAlign, &str); 5] =
    [(TabAlign::Left, "Left"), (TabAlign::Center, "Center"), (TabAlign::Right, "Right"), (TabAlign::Decimal, "Decimal"), (TabAlign::Bar, "Bar")];
const TAB_LEADERS: [(TabLeader, &str); 5] = [
    (TabLeader::None, "None"),
    (TabLeader::Dot, "....."),
    (TabLeader::Hyphen, "-----"),
    (TabLeader::Underscore, "_____"),
    (TabLeader::MiddleDot, "·····"),
];

/// Border line styles offered: (OOXML name, label).
pub(crate) const LINES: [(&str, &str); 6] =
    [("single", "Single"), ("double", "Double"), ("dotted", "Dotted"), ("dashed", "Dashed"), ("thick", "Thick"), ("dotDash", "Dot dash")];

/// New multilevel lists a style can be numbered with: (`list.kind`, sample).
const NEW_LISTS: [(&str, &str); 4] =
    [("legal", "1. 1.1. 1.1.1."), ("outline", "I. A. 1. a."), ("thaiLegal", "๑. ๑.๑. ๑.๑.๑."), ("thaiOutline", "๑. ก. (๑) (ก)")];

const ALIGNS: [(&str, &str); 6] = [
    ("left", "Left"),
    ("center", "Center"),
    ("right", "Right"),
    ("justify", "Justify"),
    ("distribute", "Distributed"),
    ("thaiDistribute", "Thai Distribution"),
];

/// What the Modify Style window shows. Indents are in the interface unit; spacing in points.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleForm {
    pub page: String,
    pub id: String,
    pub character: bool,
    pub name: String,
    /// Style ids; empty for none.
    pub based_on: String,
    pub next: String,
    // Font: Latin text, then Thai (complex script) text.
    pub font: String,
    pub size: f32,
    pub bold: bool,
    pub italic: bool,
    pub font_cs: String,
    pub font_ea: String,
    pub size_cs: f32,
    pub bold_cs: bool,
    pub italic_cs: bool,
    pub color: String,
    pub highlight: Highlight,
    // Font Effects.
    /// `Underline`'s OOXML name (`none`, `single`, `double`…).
    pub underline: String,
    /// `RRGGBB`, or `auto` (the text's colour).
    pub underline_color: String,
    pub strike: bool,
    pub double_strike: bool,
    pub superscript: bool,
    pub subscript: bool,
    pub small_caps: bool,
    pub caps: bool,
    pub hidden: bool,
    pub outline: bool,
    pub shadow: bool,
    pub emboss: bool,
    pub engrave: bool,
    // Position.
    /// Character spacing, points.
    pub spacing: f32,
    /// Raised (+) or lowered (−), points.
    pub raise: f32,
    /// Width scale, percent.
    pub scale: f32,
    pub kern: bool,
    /// Kern from this font size up, points.
    pub kern_at: f32,
    // Indents & Spacing.
    pub left: f32,
    pub right: f32,
    /// Signed first-line indent: positive first line, negative hanging.
    pub first: f32,
    pub before: f32,
    pub after: f32,
    /// `multiple` (lines), `atLeast` or `exactly` (points).
    pub line_rule: String,
    pub line: f32,
    pub contextual: bool,
    // Alignment.
    pub align: String,
    pub rtl: bool,
    // Text Flow.
    pub widow: bool,
    pub keep_next: bool,
    pub keep_lines: bool,
    pub page_break: bool,
    pub no_hyphens: bool,
    pub no_line_numbers: bool,
    /// Body text is 9; outline levels 1–9 are stored as 0–8.
    pub outline_level: u8,
    pub kinsoku: bool,
    pub drop_cap: u8,
    // Tabs: position (interface unit), alignment, leader.
    pub tabs: Vec<(f32, TabAlign, TabLeader)>,
    /// The stop being edited.
    pub tab: usize,
    /// Stops the style inherits (positions in points), so deleting one writes a clear.
    #[serde(skip)]
    pub inherited_tabs: Vec<f32>,
    // Borders & Shading: the sides with a line, and that line.
    pub border_top: bool,
    pub border_bottom: bool,
    pub border_left: bool,
    pub border_right: bool,
    pub border_between: bool,
    pub border_style: String,
    /// Points.
    pub border_width: f32,
    pub border_color: String,
    /// Distance from the text, points.
    pub border_space: f32,
    /// `RRGGBB`, or empty for none.
    pub shading: String,
    // Numbering: `""` none, `num:<id>` a list of the document's, `kind:<legal…>` a new one.
    pub list: String,
    /// 1-based.
    pub list_level: u8,
    /// The document's lists: (`num:<id>`, how it numbers).
    #[serde(skip)]
    pub lists: Vec<(String, String)>,
    /// Paragraph styles to base on and follow with: (id, name).
    #[serde(skip)]
    pub styles: Vec<(String, String)>,
    /// The form as it opened: OK sends only what changed, so values the style inherits (from
    /// the style it's based on) stay inherited.
    #[serde(skip)]
    pub start: Option<Box<StyleForm>>,
    /// Baseline after opening Format…; its edits are carried in the extra property maps below.
    #[serde(skip)]
    pub baseline: Option<Box<StyleForm>>,
    #[serde(skip)]
    pub parent_baseline: Option<Box<StyleForm>>,
    /// Formatting set from Modify Style › Format… (including Language and Frame).
    #[serde(skip)]
    pub extra_chr: serde_json::Map<String, Value>,
    #[serde(skip)]
    pub extra_para: serde_json::Map<String, Value>,
    pub reset_formatting: bool,
}

fn align_name(a: Align) -> &'static str {
    match a {
        Align::Left => "left",
        Align::Center => "center",
        Align::Right => "right",
        Align::Justify => "justify",
        Align::Distribute => "distribute",
        Align::ThaiDistribute => "thaiDistribute",
    }
}

fn align_of(s: &str) -> Align {
    match s {
        "center" => Align::Center,
        "right" => Align::Right,
        "justify" => Align::Justify,
        "distribute" => Align::Distribute,
        "thaiDistribute" => Align::ThaiDistribute,
        _ => Align::Left,
    }
}

fn choice_names(ui: &mut Ui, id: &str, width: f32, value: &mut String, choices: &[(&str, &str)]) {
    let shown = choices.iter().find(|(v, _)| *v == value).map(|(_, label)| tl!(label).to_string()).unwrap_or_else(|| value.clone());
    egui::ComboBox::from_id_salt(id).width(width).selected_text(shown).show_ui(ui, |ui| {
        for (v, label) in choices {
            ui.selectable_value(value, (*v).to_string(), tl!(label));
        }
    });
}

impl StyleForm {
    /// The form for style `id` as it resolves now; `None` for no such style or a table style
    /// (edited from Table Design).
    pub fn read(app: &WordApp, id: &str) -> Option<StyleForm> {
        let sheet = &app.session.doc.styles;
        let st = sheet.get(id)?;
        let character = match st.kind {
            StyleKind::Paragraph => false,
            StyleKind::Character => true,
            _ => return None,
        };
        let rc = sheet.resolve_char(
            if character { None } else { Some(id) },
            &CharProps { style: if character { Some(id.into()) } else { None }, ..Default::default() },
        );
        let rp = sheet.resolve_para(&ParaProps { style: Some(id.into()), ..Default::default() });
        let k = crate::options::unit().pt_per_unit();
        let (line_rule, line) = match rp.line_spacing {
            LineSpacing::Multiple(x) => ("multiple", x),
            LineSpacing::AtLeast(x) => ("atLeast", x),
            LineSpacing::Exactly(x) => ("exactly", x),
        };
        let kind = if character { StyleKind::Character } else { StyleKind::Paragraph };
        let bs = rp.borders.unwrap_or_default();
        let shown = |b: Option<doc4me_doc::props::Border>| b.is_some_and(|b| b.is_visible());
        let first_border = [bs.top, bs.bottom, bs.left, bs.right, bs.between].into_iter().flatten().find(|b| b.is_visible());
        // The document's numbered lists, one per definition, with how they number.
        let doc = &app.session.doc;
        let mut lists = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for n in &doc.numbering.nums {
            if !seen.insert(n.abstract_id) {
                continue;
            }
            let Some(levels) = doc4me_engine::cmd::lists::levels_of(&app.session, n.id) else { continue };
            if levels.first().is_none_or(|l| l.format == doc4me_doc::section::NumFormat::Bullet) {
                continue;
            }
            let levels = Value::Array(levels.iter().map(doc4me_engine::cmd::lists::level_json).collect());
            let sample = crate::dialogs_lists::SavedList { name: String::new(), levels }.sample();
            let name = doc.numbering.abstracts.iter().find(|a| a.id == n.abstract_id).and_then(|a| a.name.clone());
            lists.push((format!("num:{}", n.id), name.map_or(sample.clone(), |nm| format!("{nm}   {sample}"))));
        }
        // The tab stops the style's base gives it (the document default's for a style without one).
        let base_para = ParaProps { style: st.based_on.clone(), ..Default::default() };
        let inherited_tabs = if st.based_on.is_some() { sheet.resolve_para(&base_para).tabs.iter().map(|t| t.pos).collect() } else { Vec::new() };
        let mut styles: Vec<(String, String)> =
            sheet.styles.iter().filter(|s| s.kind == kind && !s.hidden && s.id != id).map(|s| (s.id.clone(), crate::i18n::style_name(s))).collect();
        styles.sort_by(|a, b| a.1.cmp(&b.1));
        let mut f = StyleForm {
            page: "general".into(),
            id: id.into(),
            character,
            name: st.name.clone(),
            based_on: st.based_on.clone().unwrap_or_default(),
            next: st.next.clone().unwrap_or_default(),
            font: rc.font.clone(),
            size: rc.size,
            bold: rc.bold,
            italic: rc.italic,
            font_cs: rc.font_cs.clone(),
            font_ea: rc.font_ea.clone().unwrap_or_else(|| rc.font.clone()),
            size_cs: rc.size_cs,
            bold_cs: rc.bold_cs,
            italic_cs: rc.italic_cs,
            color: match rc.color {
                TextColor::Rgb(c) => c.hex(),
                TextColor::Auto => "auto".into(),
            },
            highlight: Highlight::ALL.into_iter().find(|h| h.rgb() == rc.highlight).unwrap_or_default(),
            underline: rc.underline.ooxml().to_string(),
            underline_color: rc.underline_color.map_or("auto".to_string(), |c| c.hex()),
            strike: rc.strike,
            double_strike: rc.double_strike,
            hidden: rc.hidden,
            outline: rc.outline,
            shadow: rc.shadow,
            emboss: rc.emboss,
            engrave: rc.engrave,
            raise: rc.position,
            scale: rc.scale,
            kern: rc.kern > 0.0,
            kern_at: if rc.kern > 0.0 { rc.kern } else { 14.0 },
            superscript: rc.vert_align == VertAlign::Superscript,
            subscript: rc.vert_align == VertAlign::Subscript,
            small_caps: rc.small_caps,
            caps: rc.caps,
            spacing: rc.spacing,
            left: rp.indent_left / k,
            right: rp.indent_right / k,
            first: rp.indent_first / k,
            before: rp.space_before,
            after: rp.space_after,
            line_rule: line_rule.into(),
            line,
            contextual: rp.contextual_spacing,
            align: align_name(rp.align).into(),
            rtl: rp.bidi,
            widow: rp.widow_control,
            keep_next: rp.keep_next,
            keep_lines: rp.keep_lines,
            page_break: rp.page_break_before,
            no_hyphens: rp.suppress_hyphens,
            no_line_numbers: rp.suppress_line_numbers,
            outline_level: rp.outline_level.unwrap_or(9),
            kinsoku: rp.kinsoku,
            drop_cap: rp.drop_cap,
            tabs: rp.tabs.iter().filter(|t| t.align != TabAlign::Clear).map(|t| (t.pos / k, t.align, t.leader)).collect(),
            tab: 0,
            inherited_tabs,
            border_top: shown(bs.top),
            border_bottom: shown(bs.bottom),
            border_left: shown(bs.left),
            border_right: shown(bs.right),
            border_between: shown(bs.between),
            border_style: first_border.map_or("single".to_string(), |b| b.style.ooxml().to_string()),
            border_width: first_border.map_or(0.5, |b| b.width),
            border_color: first_border.and_then(|b| b.color).map_or("auto".to_string(), |c| c.hex()),
            border_space: first_border.map_or(1.0, |b| b.space),
            shading: rp.shading.map(|c| c.hex()).unwrap_or_default(),
            list: rp.numbering.filter(|n| n.num != 0).map(|n| format!("num:{}", n.num)).unwrap_or_default(),
            list_level: rp.numbering.filter(|n| n.num != 0).map_or(1, |n| n.level.min(8) + 1),
            lists,
            styles,
            start: None,
            baseline: None,
            parent_baseline: None,
            extra_chr: serde_json::Map::new(),
            extra_para: serde_json::Map::new(),
            reset_formatting: false,
        };
        f.start = Some(Box::new(f.clone()));
        Some(f)
    }

    /// Whether `pick` differs from the opening form (or there is none to compare with).
    fn changed<T: PartialEq>(&self, pick: impl Fn(&StyleForm) -> T) -> bool {
        self.baseline.as_deref().or(self.parent_baseline.as_deref()).or(self.start.as_deref()).is_none_or(|s| pick(s) != pick(self))
    }

    /// Keep the page fields in step with the older Format… dialogs, while retaining fields those
    /// dialogs expose that have no dedicated page here (for example language and frame settings).
    pub(crate) fn sync_format(&mut self, chr: &serde_json::Map<String, Value>, para: &serde_json::Map<String, Value>) {
        self.extra_chr = chr.clone();
        self.extra_para = para.clone();
        let c: CharProps = serde_json::from_value(Value::Object(chr.clone())).unwrap_or_default();
        if let Some(x) = c.font {
            self.font = x;
        }
        if let Some(x) = c.size {
            self.size = x;
        }
        if let Some(x) = c.bold {
            self.bold = x;
        }
        if let Some(x) = c.italic {
            self.italic = x;
        }
        if let Some(x) = c.font_cs {
            self.font_cs = x;
        }
        if let Some(x) = c.font_ea {
            self.font_ea = x;
        }
        if let Some(x) = c.size_cs {
            self.size_cs = x;
        }
        if let Some(x) = c.bold_cs {
            self.bold_cs = x;
        }
        if let Some(x) = c.italic_cs {
            self.italic_cs = x;
        }
        if let Some(x) = c.color {
            self.color = match x {
                TextColor::Rgb(rgb) => rgb.hex(),
                TextColor::Auto => "auto".into(),
            };
        }
        if let Some(x) = c.highlight {
            self.highlight = x;
        }
        if let Some(x) = c.underline {
            self.underline = x.ooxml().into();
        }
        if let Some(x) = c.underline_color {
            self.underline_color = x.hex();
        }
        if let Some(x) = c.strike {
            self.strike = x;
        }
        if let Some(x) = c.double_strike {
            self.double_strike = x;
        }
        if let Some(x) = c.hidden {
            self.hidden = x;
        }
        if let Some(x) = c.outline {
            self.outline = x;
        }
        if let Some(x) = c.shadow {
            self.shadow = x;
        }
        if let Some(x) = c.emboss {
            self.emboss = x;
        }
        if let Some(x) = c.engrave {
            self.engrave = x;
        }
        if let Some(x) = c.position {
            self.raise = x;
        }
        if let Some(x) = c.scale {
            self.scale = x;
        }
        if let Some(x) = c.kern {
            self.kern = x > 0.0;
            self.kern_at = x;
        }
        if let Some(x) = c.vert_align {
            self.superscript = x == VertAlign::Superscript;
            self.subscript = x == VertAlign::Subscript;
        }
        if let Some(x) = c.small_caps {
            self.small_caps = x;
        }
        if let Some(x) = c.caps {
            self.caps = x;
        }
        if let Some(x) = c.spacing {
            self.spacing = x;
        }

        if !self.character {
            let p: ParaProps = serde_json::from_value(Value::Object(para.clone())).unwrap_or_default();
            let k = crate::options::unit().pt_per_unit();
            if let Some(x) = p.align {
                self.align = align_name(x).into();
            }
            if let Some(x) = p.bidi {
                self.rtl = x;
            }
            if let Some(x) = p.indent_left {
                self.left = x / k;
            }
            if let Some(x) = p.indent_right {
                self.right = x / k;
            }
            if let Some(x) = p.indent_first {
                self.first = x / k;
            }
            if let Some(x) = p.space_before {
                self.before = x;
            }
            if let Some(x) = p.space_after {
                self.after = x;
            }
            if let Some(x) = p.line_spacing {
                match x {
                    LineSpacing::Multiple(x) => {
                        self.line_rule = "multiple".into();
                        self.line = x;
                    }
                    LineSpacing::AtLeast(x) => {
                        self.line_rule = "atLeast".into();
                        self.line = x;
                    }
                    LineSpacing::Exactly(x) => {
                        self.line_rule = "exactly".into();
                        self.line = x;
                    }
                }
            }
            if let Some(x) = p.contextual_spacing {
                self.contextual = x;
            }
            if let Some(x) = p.widow_control {
                self.widow = x;
            }
            if let Some(x) = p.keep_next {
                self.keep_next = x;
            }
            if let Some(x) = p.keep_lines {
                self.keep_lines = x;
            }
            if let Some(x) = p.page_break_before {
                self.page_break = x;
            }
            if let Some(x) = p.suppress_hyphens {
                self.no_hyphens = x;
            }
            if let Some(x) = p.suppress_line_numbers {
                self.no_line_numbers = x;
            }
            if let Some(x) = p.outline_level {
                self.outline_level = x;
            }
            if let Some(x) = p.kinsoku {
                self.kinsoku = x;
            }
            if let Some(x) = p.drop_cap {
                self.drop_cap = x;
            }
            if let Some(x) = p.tabs {
                self.tabs = x.into_iter().filter(|t| t.align != TabAlign::Clear).map(|t| (t.pos / k, t.align, t.leader)).collect();
                self.tab = self.tab.min(self.tabs.len().saturating_sub(1));
            }
            if let Some(b) = p.borders {
                let show = |b: Option<doc4me_doc::props::Border>| b.is_some_and(|b| b.is_visible());
                let first = [b.top, b.bottom, b.left, b.right, b.between].into_iter().flatten().find(|b| b.is_visible());
                self.border_top = show(b.top);
                self.border_bottom = show(b.bottom);
                self.border_left = show(b.left);
                self.border_right = show(b.right);
                self.border_between = show(b.between);
                if let Some(b) = first {
                    self.border_style = b.style.ooxml().into();
                    self.border_width = b.width;
                    self.border_color = b.color.map_or_else(|| "auto".into(), |c| c.hex());
                    self.border_space = b.space;
                }
            }
            if let Some(x) = p.shading {
                self.shading = x.hex();
            }
            if let Some(x) = p.numbering {
                self.list = format!("num:{}", x.num);
                self.list_level = x.level.min(8) + 1;
            }
        }
        // The saved original still powers Reset; this baseline makes edits after Format… override
        // the values that the Format dialog just staged.
        let mut baseline = self.clone();
        baseline.baseline = None;
        self.baseline = Some(Box::new(baseline));
    }

    /// The character formatting the form changes in the style. A Thai value goes along when its
    /// Latin one changed, so the engine doesn't copy the Latin one over it.
    pub fn chr(&self) -> CharProps {
        let c = |pick: fn(&StyleForm) -> String| self.changed(pick);
        let thai = |latin: bool, thai: bool| latin || thai;
        let font_cs = thai(self.changed(|f| f.font.clone()), self.changed(|f| f.font_cs.clone()));
        let size_cs = thai(self.changed(|f| f.size.to_bits()), self.changed(|f| f.size_cs.to_bits()));
        let bold_cs = thai(self.changed(|f| f.bold), self.changed(|f| f.bold_cs));
        let italic_cs = thai(self.changed(|f| f.italic), self.changed(|f| f.italic_cs));
        let vert = self.changed(|f| (f.superscript, f.subscript));
        let mut chr: CharProps = serde_json::from_value(Value::Object(self.extra_chr.clone())).unwrap_or_default();
        chr.overlay(&CharProps {
            font: Some(self.font.clone()).filter(|f| !f.trim().is_empty() && c(|f| f.font.clone())),
            size: self.changed(|f| f.size.to_bits()).then_some(self.size.clamp(1.0, 1638.0)),
            bold: self.changed(|f| f.bold).then_some(self.bold),
            italic: self.changed(|f| f.italic).then_some(self.italic),
            font_cs: Some(self.font_cs.clone()).filter(|f| !f.trim().is_empty() && font_cs),
            font_ea: Some(self.font_ea.clone()).filter(|f| !f.trim().is_empty() && c(|f| f.font_ea.clone())),
            size_cs: size_cs.then_some(self.size_cs.clamp(1.0, 1638.0)),
            bold_cs: bold_cs.then_some(self.bold_cs),
            italic_cs: italic_cs.then_some(self.italic_cs),
            color: if !c(|f| f.color.clone()) {
                None
            } else if self.color.trim().eq_ignore_ascii_case("auto") {
                Some(TextColor::Auto)
            } else {
                doc4me_doc::Rgb::parse(&self.color).map(TextColor::Rgb)
            },
            highlight: self.changed(|f| f.highlight).then_some(self.highlight),
            underline: self.changed(|f| f.underline.clone()).then_some(Underline::from_ooxml(&self.underline)),
            underline_color: if self.changed(|f| f.underline_color.clone()) { doc4me_doc::Rgb::parse(&self.underline_color) } else { None },
            strike: self.changed(|f| f.strike).then_some(self.strike),
            double_strike: self.changed(|f| f.double_strike).then_some(self.double_strike),
            hidden: self.changed(|f| f.hidden).then_some(self.hidden),
            outline: self.changed(|f| f.outline).then_some(self.outline),
            shadow: self.changed(|f| f.shadow).then_some(self.shadow),
            emboss: self.changed(|f| f.emboss).then_some(self.emboss),
            engrave: self.changed(|f| f.engrave).then_some(self.engrave),
            position: self.changed(|f| f.raise.to_bits()).then_some(self.raise.clamp(-1584.0, 1584.0)),
            scale: self.changed(|f| f.scale.to_bits()).then_some(self.scale.clamp(1.0, 600.0)),
            kern: self.changed(|f| (f.kern, f.kern_at.to_bits())).then_some(if self.kern { self.kern_at.clamp(1.0, 1638.0) } else { 0.0 }),
            vert_align: vert.then_some(if self.superscript {
                VertAlign::Superscript
            } else if self.subscript {
                VertAlign::Subscript
            } else {
                VertAlign::Baseline
            }),
            small_caps: self.changed(|f| f.small_caps).then_some(self.small_caps),
            caps: self.changed(|f| f.caps).then_some(self.caps),
            spacing: self.changed(|f| f.spacing.to_bits()).then_some(self.spacing.clamp(-100.0, 100.0)),
            ..Default::default()
        });
        chr
    }

    /// The paragraph formatting the form gives the style (none for a character style).
    pub fn para(&self) -> Option<ParaProps> {
        if self.character {
            return None;
        }
        let k = crate::options::unit().pt_per_unit();
        let line = match self.line_rule.as_str() {
            "atLeast" => LineSpacing::AtLeast(self.line.clamp(0.0, 1584.0)),
            "exactly" => LineSpacing::Exactly(self.line.clamp(0.7, 1584.0)),
            _ => LineSpacing::Multiple(self.line.clamp(0.06, 132.0)),
        };
        let ch = |pick: fn(&StyleForm) -> u32| self.changed(pick);
        let mut para: ParaProps = serde_json::from_value(Value::Object(self.extra_para.clone())).unwrap_or_default();
        para.overlay(&ParaProps {
            align: self.changed(|f| f.align.clone()).then_some(align_of(&self.align)),
            bidi: self.changed(|f| f.rtl).then_some(self.rtl),
            indent_left: ch(|f| f.left.to_bits()).then_some(self.left * k),
            indent_right: ch(|f| f.right.to_bits()).then_some(self.right * k),
            indent_first: ch(|f| f.first.to_bits()).then_some(self.first * k),
            space_before: ch(|f| f.before.to_bits()).then_some(self.before.clamp(0.0, 1584.0)),
            space_after: ch(|f| f.after.to_bits()).then_some(self.after.clamp(0.0, 1584.0)),
            line_spacing: self.changed(|f| (f.line_rule.clone(), f.line.to_bits())).then_some(line),
            contextual_spacing: self.changed(|f| f.contextual).then_some(self.contextual),
            widow_control: self.changed(|f| f.widow).then_some(self.widow),
            keep_next: self.changed(|f| f.keep_next).then_some(self.keep_next),
            keep_lines: self.changed(|f| f.keep_lines).then_some(self.keep_lines),
            page_break_before: self.changed(|f| f.page_break).then_some(self.page_break),
            suppress_hyphens: self.changed(|f| f.no_hyphens).then_some(self.no_hyphens),
            suppress_line_numbers: self.changed(|f| f.no_line_numbers).then_some(self.no_line_numbers),
            outline_level: self.changed(|f| f.outline_level).then_some(self.outline_level.min(9)),
            kinsoku: self.changed(|f| f.kinsoku).then_some(self.kinsoku),
            drop_cap: self.changed(|f| f.drop_cap).then_some(self.drop_cap.min(10)),
            tabs: self.changed(|f| f.tabs.iter().map(|t| (t.0.to_bits(), t.1, t.2)).collect::<Vec<_>>()).then(|| self.tab_stops(k)),
            borders: self.borders_changed().then(|| self.borders()),
            shading: doc4me_doc::Rgb::parse(&self.shading).filter(|_| self.changed(|f| f.shading.clone())),
            ..Default::default()
        });
        Some(para)
    }

    /// The style's own stops: the form's, and a clear for each inherited one deleted.
    fn tab_stops(&self, k: f32) -> Vec<TabStop> {
        let mut v: Vec<TabStop> =
            self.tabs.iter().map(|(pos, align, leader)| TabStop { pos: (pos * k).clamp(0.0, 1584.0), align: *align, leader: *leader }).collect();
        for pos in &self.inherited_tabs {
            if !v.iter().any(|t| (t.pos - pos).abs() < 0.5) {
                v.push(TabStop { pos: *pos, align: TabAlign::Clear, leader: TabLeader::None });
            }
        }
        v.sort_by(|a, b| a.pos.total_cmp(&b.pos));
        v
    }

    fn borders_changed(&self) -> bool {
        self.changed(|f| {
            (
                [f.border_top, f.border_bottom, f.border_left, f.border_right, f.border_between],
                f.border_style.clone(),
                f.border_width.to_bits(),
                f.border_color.clone(),
                f.border_space.to_bits(),
            )
        })
    }

    /// The sides with a line, each that line; the others explicitly off (so a base style's
    /// borders don't show through).
    fn borders(&self) -> doc4me_doc::props::Borders {
        use doc4me_doc::props::{Border, BorderStyle};
        let color = if self.border_color.trim().eq_ignore_ascii_case("auto") { None } else { doc4me_doc::Rgb::parse(&self.border_color) };
        let space = self.border_space.clamp(0.0, 31.0);
        let line = |on: bool, _: f32| {
            Some(if on {
                Border { style: BorderStyle::from_ooxml(&self.border_style), width: self.border_width.clamp(0.25, 6.0), color, space }
            } else {
                Border { style: BorderStyle::None, width: 0.0, color: None, space: 0.0 }
            })
        };
        doc4me_doc::props::Borders {
            top: line(self.border_top, 1.0),
            bottom: line(self.border_bottom, 1.0),
            left: line(self.border_left, 4.0),
            right: line(self.border_right, 4.0),
            between: line(self.border_between, 1.0),
            inside_v: None,
        }
    }

    /// `styles.modify` parameters for everything in the form.
    pub fn params(&self) -> Value {
        let mut v = json!({
            "style": self.id,
            "name": self.name,
            "basedOn": if self.based_on.is_empty() { Value::Null } else { json!(self.based_on) },
            "chr": serde_json::to_value(self.chr()).unwrap_or(Value::Null),
        });
        if self.reset_formatting {
            v["resetFormatting"] = json!(true);
        }
        if let Some(p) = self.para() {
            v["para"] = serde_json::to_value(p).unwrap_or(Value::Null);
            if !self.next.is_empty() {
                v["next"] = json!(self.next);
            }
            if self.changed(|f| f.shading.clone()) && self.shading.trim().is_empty() {
                v["clear"] = json!(["shading"]);
            }
            if self.changed(|f| (f.list.clone(), f.list_level)) {
                v["listLevel"] = json!(self.list_level.clamp(1, 9) - 1);
                v["list"] = if let Some(n) = self.list.strip_prefix("num:").and_then(|n| n.parse::<u64>().ok()) {
                    json!({"num": n})
                } else if let Some(k) = self.list.strip_prefix("kind:") {
                    json!({"kind": k})
                } else {
                    Value::Null
                };
            }
        }
        v
    }
}

/// Stage a reset without editing the document until Apply or OK.
fn reset_to_parent(app: &mut WordApp, f: &mut StyleForm) {
    let Some(original) = app.session.doc.styles.get(&f.id).cloned() else { return };
    if let Some(style) = std::sync::Arc::make_mut(&mut app.session.doc.styles).get_mut(&f.id) {
        style.para = ParaProps::default();
        style.chr = CharProps::default();
        style.based_on = (!f.based_on.is_empty()).then(|| f.based_on.clone());
    }
    let inherited = StyleForm::read(app, &f.id);
    if let Some(style) = std::sync::Arc::make_mut(&mut app.session.doc.styles).get_mut(&f.id) {
        *style = original;
    }
    if let Some(mut inherited) = inherited {
        inherited.page = f.page.clone();
        inherited.name = f.name.clone();
        inherited.next = f.next.clone();
        inherited.start = f.start.clone();
        inherited.parent_baseline = Some(Box::new(inherited.clone()));
        inherited.baseline = None;
        inherited.reset_formatting = true;
        *f = inherited;
    }
}

/// The Modify Style window. Returns true to close.
pub fn body(app: &mut WordApp, ui: &mut Ui, f: &mut StyleForm, back: &mut bool, resume: &mut Option<Box<crate::dialogs::Dialog>>) -> bool {
    // A fixed area, so the window keeps its size from page to page (a separator here grew to the
    // whole screen's height).
    const PAGE_H: f32 = 400.0;
    ui.allocate_ui_with_layout(vec2(760.0, PAGE_H), egui::Layout::left_to_right(egui::Align::Min), |ui| {
        ui.set_height(PAGE_H);
        // The pages.
        ui.vertical(|ui| {
            ui.set_width(150.0);
            for (id, name, icon) in
                PAGES.iter().filter(|(id, _, _)| !f.character || matches!(*id, "general" | "font" | "effects" | "position" | "highlight"))
            {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::hover());
                    let t = crate::theme::Tokens::get(ui.ctx());
                    // Accent colours (owner, 2026-11-25: style A), as on the ribbon.
                    crate::icons::paint(ui.painter(), r, icon, t.icon, t.accent);
                    if ui.steady_label(f.page == *id, tl!(name)).clicked() {
                        f.page = id.to_string();
                    }
                });
            }
        });
        let x = ui.cursor().min.x + 4.0;
        let top = ui.cursor().min.y;
        ui.painter().vline(x, top..=top + PAGE_H, ui.visuals().widgets.noninteractive.bg_stroke);
        ui.add_space(12.0);
        ui.vertical(|ui| {
            ui.set_width(590.0);
            match f.page.as_str() {
                "font" => font_page(app, ui, f),
                "effects" => effects_page(app, ui, f),
                "position" => position_page(ui, f),
                "highlight" => highlight_page(ui, f),
                "area" if !f.character => area_page(app, ui, f),
                "tabs" if !f.character => tabs_page(ui, f),
                "indents" if !f.character => indents_page(ui, f),
                "alignment" if !f.character => alignment_page(ui, f),
                "flow" if !f.character => flow_page(ui, f),
                "borders" if !f.character => borders_page(app, ui, f),
                "numbering" if !f.character => numbering_page(ui, f),
                "dropcap" if !f.character => drop_cap_page(ui, f),
                _ => general_page(app, ui, f),
            }
        });
    });
    ui.separator();
    ui.label(egui::RichText::new(tl!("Preview")).small().weak());
    let key = serde_json::to_string(&f.params()).unwrap_or_default();
    let (chr, para, id, reset_formatting, based_on) = (f.chr(), f.para(), f.id.clone(), f.reset_formatting, f.based_on.clone());
    crate::previews::style_edit_preview(
        app,
        ui,
        &id,
        f.character,
        &key,
        &|sheet| {
            if let Some(st) = sheet.get_mut(&id) {
                if reset_formatting {
                    st.chr = CharProps::default();
                    st.para = ParaProps::default();
                    st.based_on = (!based_on.is_empty()).then(|| based_on.clone());
                }
                st.chr.overlay(&chr);
                if let Some(p) = &para {
                    st.para.overlay(p);
                }
            }
        },
        vec2(720.0, 64.0),
    );
    let mut reset = false;
    let mut reset_parent = false;
    let mut format = None;
    let (mut ok, mut apply, mut cancel) = (false, false, false);
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let paragraph = !f.character;
        ui.menu_button(tl!("Format"), |ui| {
            for (label, which, para_only) in crate::style_format::ITEMS {
                if ui.add_enabled(paragraph || !para_only, egui::Button::new(tl!(label))).clicked() {
                    format = Some(which);
                    ui.close();
                }
            }
        });
        reset = ui.button(tl!("Reset")).clicked();
        reset_parent = ui.button(tl!("Reset to Parent")).clicked();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let keys = !crate::colors::is_open(ui.ctx());
            cancel = ui.button(tl!("Cancel")).clicked() || (keys && ui.input(|i| i.key_pressed(egui::Key::Escape)));
            apply = ui.button(tl!("Apply")).clicked();
            ok = ui.add(crate::widgets::primary_button(ui, tl!("OK"))).clicked() || (keys && ui.input(|i| i.key_pressed(egui::Key::Enter)));
        });
    });
    if let Some(which) = format {
        if let Some(mut dialog) = crate::dialogs::Dialog::modify_style(app, &f.id) {
            if let crate::dialogs::Dialog::ModifyStyle {
                back: dialog_back,
                resume: dialog_resume,
                name,
                font,
                size,
                bold,
                italic,
                color,
                before,
                after,
                extra_chr,
                extra_para,
                form,
                ..
            } = &mut dialog
            {
                **form = f.clone();
                *dialog_back = *back;
                *dialog_resume = resume.take();
                *name = f.name.clone();
                *font = f.font.clone();
                *size = f.size;
                *bold = f.bold;
                *italic = f.italic;
                *color = f.color.clone();
                *before = f.before;
                *after = f.after;
                *extra_chr = serde_json::to_value(f.chr()).ok().and_then(|v| v.as_object().cloned()).unwrap_or_default();
                *extra_para = f.para().and_then(|p| serde_json::to_value(p).ok()).and_then(|v| v.as_object().cloned()).unwrap_or_default();
            }
            crate::style_format::open(app, which, dialog);
        }
        return true;
    }
    if reset_parent {
        reset_to_parent(app, f);
    }
    if reset && let Some(start) = f.start.as_deref().cloned() {
        let page = f.page.clone();
        *f = StyleForm { page, start: Some(Box::new(start.clone())), ..start };
        f.baseline = None;
    }
    if ok || apply {
        match app.run("styles.modify", f.params()) {
            // Apply: the document shows the change and the window stays, now starting from it
            // (so the next Apply or OK sends only what changes after this).
            Ok(_) if apply => {
                if let Some(fresh) = StyleForm::read(app, &f.id) {
                    *f = StyleForm { page: f.page.clone(), tab: f.tab, ..fresh };
                }
            }
            Ok(_) => {}
            Err(e) => app.status(e),
        }
    }
    ok || cancel
}

pub(crate) fn heading(ui: &mut Ui, s: &str) {
    ui.add_space(4.0);
    ui.label(egui::RichText::new(tl!(s)).font(semibold(12.5)));
}

fn style_display_name(app: &WordApp, f: &StyleForm) -> String {
    app.session.doc.styles.get(&f.id).filter(|style| style.name == f.name).map_or_else(|| f.name.clone(), crate::i18n::style_name)
}

fn general_page(app: &WordApp, ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Style");
    egui::Grid::new("ms_general").num_columns(2).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Name:"));
        let mut display_name = style_display_name(app, f);
        if ui.add(egui::TextEdit::singleline(&mut display_name).desired_width(300.0)).changed() {
            f.name = display_name.clone();
        }
        ui.end_row();
        let named = |id: &str| f.styles.iter().find(|(s, _)| s == id).map(|(_, n)| n.clone());
        ui.label(tl!("Based on:"));
        let shown = named(&f.based_on).unwrap_or_else(|| tl!("(no style)").to_string());
        egui::ComboBox::from_id_salt("ms_based").selected_text(shown).width(300.0).show_ui(ui, |ui| {
            ui.steady_value(&mut f.based_on, String::new(), tl!("(no style)"));
            for (id, name) in &f.styles {
                ui.steady_value(&mut f.based_on, id.clone(), name);
            }
        });
        ui.end_row();
        if !f.character {
            ui.label(tl!("Style for following paragraph:"));
            let shown = if f.next.is_empty() || f.next == f.id { display_name.clone() } else { named(&f.next).unwrap_or_else(|| f.next.clone()) };
            egui::ComboBox::from_id_salt("ms_next").selected_text(shown).width(300.0).show_ui(ui, |ui| {
                ui.steady_value(&mut f.next, f.id.clone(), &display_name);
                for (id, name) in &f.styles {
                    ui.steady_value(&mut f.next, id.clone(), name);
                }
            });
            ui.end_row();
        }
        ui.label(tl!("Style type:"));
        ui.label(if f.character { tl!("Character") } else { tl!("Paragraph") });
        ui.end_row();
    });
}

fn font_page(app: &mut WordApp, ui: &mut Ui, f: &mut StyleForm) {
    let fams = app.previews.families();
    let prev = app.previews.font_preview_fn();
    let mut column = |ui: &mut Ui, title: &str, salt: &str, font: &mut String, size: &mut f32, bold: &mut bool, italic: &mut bool| {
        ui.vertical(|ui| {
            heading(ui, title);
            egui::Grid::new(salt).num_columns(2).spacing(vec2(8.0, 8.0)).show(ui, |ui| {
                ui.label(tl!("Font:"));
                let shown = app.previews.font_shown(font);
                if let Some(pick) = crate::widgets::font_combo(ui, &format!("{salt}_font"), 190.0, &shown, &fams, Some(&*prev)) {
                    if pick == crate::widgets::ADD_FONT {
                        let _ = app.run("font.add", json!({}));
                    } else {
                        *font = app.previews.font_stored(&pick);
                    }
                }
                ui.end_row();
                ui.label(tl!("Size:"));
                ui.add(crate::numfield::NumField::new(size).range(1.0..=1638.0).speed(0.5));
                ui.end_row();
                ui.label("");
                ui.horizontal(|ui| {
                    ui.checkbox(bold, tl!("Bold"));
                    ui.checkbox(italic, tl!("Italic"));
                });
                ui.end_row();
            });
        });
    };
    ui.horizontal_top(|ui| {
        column(ui, "Latin text", "ms_latin", &mut f.font, &mut f.size, &mut f.bold, &mut f.italic);
        ui.add_space(16.0);
        column(ui, "Thai text (complex script)", "ms_thai", &mut f.font_cs, &mut f.size_cs, &mut f.bold_cs, &mut f.italic_cs);
    });
    if ui.small_button(tl!("Thai text like Latin text")).on_hover_text(tl!("Copy the Latin font, size, bold and italic to Thai text")).clicked() {
        f.font_cs = f.font.clone();
        f.size_cs = f.size;
        f.bold_cs = f.bold;
        f.italic_cs = f.italic;
    }
    // No East Asian font box (owner, 2026-11-25: no East Asian support); the style keeps
    // whatever East Asian font its file has.
    egui::Grid::new("ms_color").num_columns(2).spacing(vec2(8.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Font color:"));
        let theme = app.session.doc.settings.theme_colors.clone();
        crate::dialogs::color_field(ui, &theme, &mut f.color, "Automatic", "auto");
        ui.end_row();
    });
}

fn effects_page(app: &mut WordApp, ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Underline");
    egui::Grid::new("ms_underline").num_columns(2).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Style:"));
        choice_names(ui, "ms_ul", 140.0, &mut f.underline, &UNDERLINES);
        ui.end_row();
        ui.label(tl!("Underline color:"));
        let theme = app.session.doc.settings.theme_colors.clone();
        crate::dialogs::color_field(ui, &theme, &mut f.underline_color, "Automatic", "auto");
        ui.end_row();
    });
    heading(ui, "Effects");
    egui::Grid::new("ms_fx").num_columns(3).spacing(vec2(16.0, 6.0)).show(ui, |ui| {
        if ui.checkbox(&mut f.strike, tl!("Strikethrough")).changed() && f.strike {
            f.double_strike = false;
        }
        if ui.checkbox(&mut f.double_strike, tl!("Double strikethrough")).changed() && f.double_strike {
            f.strike = false;
        }
        ui.checkbox(&mut f.hidden, tl!("Hidden"));
        ui.end_row();
        if ui.checkbox(&mut f.superscript, tl!("Superscript")).changed() && f.superscript {
            f.subscript = false;
        }
        if ui.checkbox(&mut f.subscript, tl!("Subscript")).changed() && f.subscript {
            f.superscript = false;
        }
        ui.end_row();
        if ui.checkbox(&mut f.small_caps, tl!("Small caps")).changed() && f.small_caps {
            f.caps = false;
        }
        if ui.checkbox(&mut f.caps, tl!("All caps")).changed() && f.caps {
            f.small_caps = false;
        }
        ui.end_row();
        ui.checkbox(&mut f.outline, tl!("Outline"));
        ui.checkbox(&mut f.shadow, tl!("Shadow"));
        ui.end_row();
        // Emboss and engrave exclude each other, as in Word.
        if ui.checkbox(&mut f.emboss, tl!("Emboss")).changed() && f.emboss {
            f.engrave = false;
        }
        if ui.checkbox(&mut f.engrave, tl!("Engrave")).changed() && f.engrave {
            f.emboss = false;
        }
        ui.end_row();
    });
}

fn position_page(ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Position");
    egui::Grid::new("ms_pos").num_columns(2).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Raise (+) or lower (−) by (pt):"));
        ui.add(crate::numfield::NumField::new(&mut f.raise).range(-1584.0..=1584.0).speed(0.5).max_decimals(1));
        ui.end_row();
        ui.label(tl!("Scale width (%):"));
        ui.add(crate::numfield::NumField::new(&mut f.scale).range(1.0..=600.0).speed(1.0).max_decimals(0));
        ui.end_row();
    });
    heading(ui, "Spacing");
    egui::Grid::new("ms_charsp").num_columns(2).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Character spacing (pt):"));
        ui.add(crate::numfield::NumField::new(&mut f.spacing).speed(0.1).range(-20.0..=20.0).spinner());
        ui.end_row();
        ui.checkbox(&mut f.kern, tl!("Kerning for fonts (pt) and up:"));
        ui.add_enabled(f.kern, crate::numfield::NumField::new(&mut f.kern_at).range(1.0..=1638.0).speed(0.5));
        ui.end_row();
    });
}

fn tabs_page(ui: &mut Ui, f: &mut StyleForm) {
    let unit = crate::options::unit();
    heading(ui, "Tab stops");
    ui.horizontal_top(|ui| {
        // The stops, by position; one is edited on the right.
        ui.vertical(|ui| {
            ui.set_width(150.0);
            egui::ScrollArea::vertical().id_salt("ms_tabs").max_height(220.0).show(ui, |ui| {
                if f.tabs.is_empty() {
                    ui.label(egui::RichText::new(tl!("(no tab stops)")).weak());
                }
                for (i, (pos, _, _)) in f.tabs.iter().enumerate() {
                    if ui.steady_label(f.tab == i, format!("{pos:.2}{}", unit.suffix())).clicked() {
                        f.tab = i;
                    }
                }
            });
            ui.horizontal(|ui| {
                if ui.button(tl!("New")).clicked() {
                    // After the last stop, 0.5" (1.27 cm) on, as Word's default stops are.
                    let next = f.tabs.iter().map(|t| t.0).fold(0.0f32, f32::max) + 36.0 / unit.pt_per_unit();
                    f.tabs.push((next, TabAlign::Left, TabLeader::None));
                    f.tab = f.tabs.len() - 1;
                }
                if ui.add_enabled(f.tab < f.tabs.len(), egui::Button::new(tl!("Delete"))).clicked() {
                    f.tabs.remove(f.tab);
                    f.tab = f.tab.min(f.tabs.len().saturating_sub(1));
                }
                if ui.add_enabled(!f.tabs.is_empty(), egui::Button::new(tl!("Delete All"))).clicked() {
                    f.tabs.clear();
                    f.tab = 0;
                }
            });
        });
        ui.add_space(16.0);
        let Some((pos, align, leader)) = f.tabs.get_mut(f.tab) else { return };
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(tl!("Position:"));
                ui.add(crate::numfield::NumField::new(pos).speed(0.01).range(0.0..=22.0).suffix(unit.suffix()).max_decimals(2));
            });
            heading(ui, "Alignment");
            ui.horizontal_wrapped(|ui| {
                for (v, l) in TAB_ALIGNS {
                    ui.radio_value(align, v, tl!(l));
                }
            });
            heading(ui, "Leader");
            ui.horizontal_wrapped(|ui| {
                for (v, l) in TAB_LEADERS {
                    ui.radio_value(leader, v, tl!(l));
                }
            });
        });
    });
    // Keep the list in position order once a position is typed.
    let editing = f.tabs.get(f.tab).copied();
    f.tabs.sort_by(|a, b| a.0.total_cmp(&b.0));
    if let Some(e) = editing {
        f.tab = f.tabs.iter().position(|t| *t == e).unwrap_or(0);
    }
}

fn indents_page(ui: &mut Ui, f: &mut StyleForm) {
    let unit = crate::options::unit();
    fn len(v: &mut f32, unit: doc4me_geom::Unit) -> crate::numfield::NumField<'_> {
        crate::numfield::NumField::new(v).speed(0.01).range(-22.0..=22.0).suffix(unit.suffix()).max_decimals(2)
    }
    heading(ui, "Indentation");
    egui::Grid::new("ms_indent").num_columns(4).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Left:"));
        ui.add(len(&mut f.left, unit));
        ui.label(tl!("Right:"));
        ui.add(len(&mut f.right, unit));
        ui.end_row();
        // As in Word: Special (none, First line, Hanging) and By; hanging is a negative first line.
        ui.label(tl!("Special:"));
        let special = if f.first > 0.0 {
            "first"
        } else if f.first < 0.0 {
            "hanging"
        } else {
            "none"
        };
        let label = |s: &str| match s {
            "first" => tl!("First line"),
            "hanging" => tl!("Hanging"),
            _ => tl!("(none)"),
        };
        egui::ComboBox::from_id_salt("ms_special").selected_text(label(special)).show_ui(ui, |ui| {
            for s in ["none", "first", "hanging"] {
                if ui.steady_label(s == special, label(s)).clicked() {
                    let by = if f.first == 0.0 { 36.0 / unit.pt_per_unit() } else { f.first.abs() };
                    f.first = match s {
                        "first" => by,
                        "hanging" => -by,
                        _ => 0.0,
                    };
                }
            }
        });
        ui.label(tl!("By:"));
        let mut by = f.first.abs();
        if ui.add_enabled(special != "none", len(&mut by, unit)).changed() {
            f.first = if special == "hanging" { -by.abs() } else { by.abs() };
        }
        ui.end_row();
    });
    heading(ui, "Spacing");
    egui::Grid::new("ms_spacing").num_columns(4).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Before (pt):"));
        ui.add(crate::numfield::NumField::new(&mut f.before).range(0.0..=1584.0).speed(0.5));
        ui.label(tl!("Line spacing:"));
        let rule = |s: &str| match s {
            "atLeast" => tl!("At least"),
            "exactly" => tl!("Exactly"),
            _ => tl!("Multiple"),
        };
        let before = f.line_rule.clone();
        egui::ComboBox::from_id_salt("ms_line_rule").selected_text(rule(&f.line_rule)).show_ui(ui, |ui| {
            for s in ["multiple", "atLeast", "exactly"] {
                ui.steady_value(&mut f.line_rule, s.to_string(), rule(s));
            }
        });
        if f.line_rule != before {
            // Lines for Multiple, points for the others: a sensible start for each.
            f.line = if f.line_rule == "multiple" { 1.0 } else { 12.0 };
        }
        ui.end_row();
        ui.label(tl!("After (pt):"));
        ui.add(crate::numfield::NumField::new(&mut f.after).range(0.0..=1584.0).speed(0.5));
        ui.label(if f.line_rule == "multiple" { tl!("Lines:") } else { tl!("Points:") });
        ui.add(crate::numfield::NumField::new(&mut f.line).range(0.0..=1584.0).speed(0.05).max_decimals(2));
        ui.end_row();
    });
    ui.checkbox(&mut f.contextual, tl!("Don't add space between paragraphs of the same style"));
}

fn flow_page(ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Pagination");
    ui.checkbox(&mut f.widow, tl!("Widow/Orphan control"));
    ui.checkbox(&mut f.keep_next, tl!("Keep with next"));
    ui.checkbox(&mut f.keep_lines, tl!("Keep lines together"));
    ui.checkbox(&mut f.page_break, tl!("Page break before"));
    heading(ui, "Formatting exceptions");
    ui.checkbox(&mut f.no_line_numbers, tl!("Suppress line numbers"));
    ui.checkbox(&mut f.no_hyphens, tl!("Don't hyphenate"));
}

fn borders_page(app: &mut WordApp, ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Borders");
    ui.horizontal(|ui| {
        if ui.button(tl!("None")).clicked() {
            (f.border_top, f.border_bottom, f.border_left, f.border_right, f.border_between) = (false, false, false, false, false);
        }
        if ui.button(tl!("Box")).clicked() {
            (f.border_top, f.border_bottom, f.border_left, f.border_right) = (true, true, true, true);
        }
    });
    egui::Grid::new("ms_sides").num_columns(3).spacing(vec2(16.0, 6.0)).show(ui, |ui| {
        ui.checkbox(&mut f.border_top, tl!("Top"));
        ui.checkbox(&mut f.border_bottom, tl!("Bottom"));
        ui.checkbox(&mut f.border_between, tl!("Between paragraphs"));
        ui.end_row();
        ui.checkbox(&mut f.border_left, tl!("Left"));
        ui.checkbox(&mut f.border_right, tl!("Right"));
        ui.end_row();
    });
    egui::Grid::new("ms_line").num_columns(2).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Style:"));
        choice_names(ui, "ms_line_style", 140.0, &mut f.border_style, &LINES);
        ui.end_row();
        ui.label(tl!("Width (pt):"));
        ui.add(crate::numfield::NumField::new(&mut f.border_width).range(0.25..=6.0).speed(0.25).max_decimals(2));
        ui.end_row();
        ui.label(tl!("Distance from text (pt):"));
        ui.add(crate::numfield::NumField::new(&mut f.border_space).range(0.0..=31.0).speed(0.5).max_decimals(1));
        ui.end_row();
        ui.label(tl!("Color:"));
        let theme = app.session.doc.settings.theme_colors.clone();
        crate::dialogs::color_field(ui, &theme, &mut f.border_color, "Automatic", "auto");
        ui.end_row();
    });
    heading(ui, "Shading");
    egui::Grid::new("ms_shade").num_columns(2).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Fill:"));
        let theme = app.session.doc.settings.theme_colors.clone();
        crate::dialogs::color_field(ui, &theme, &mut f.shading, "No Color", "");
        ui.end_row();
    });
}

fn numbering_page(ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Outline level");
    let selected = if f.outline_level == 9 { tl!("Body text").to_string() } else { format!("{} {}", tl!("Level"), f.outline_level + 1) };
    egui::ComboBox::from_id_salt("ms_outline_level").selected_text(selected).show_ui(ui, |ui| {
        ui.steady_value(&mut f.outline_level, 9, tl!("Body text"));
        for level in 0..9 {
            ui.steady_value(&mut f.outline_level, level, format!("{} {}", tl!("Level"), level + 1));
        }
    });
    heading(ui, "Numbering");
    ui.label(tl!("Number paragraphs in this style with:"));
    let shown = if f.list.is_empty() {
        tl!("(no numbering)").to_string()
    } else if let Some(k) = f.list.strip_prefix("kind:") {
        NEW_LISTS.iter().find(|(x, _)| *x == k).map_or(k.to_string(), |(_, s)| crate::i18n::fmt(tl!("New list: {sample}"), &[("sample", s)]))
    } else {
        f.lists.iter().find(|(id, _)| *id == f.list).map_or(f.list.clone(), |(_, s)| s.clone())
    };
    egui::ComboBox::from_id_salt("ms_list").selected_text(shown).width(360.0).show_ui(ui, |ui| {
        ui.steady_value(&mut f.list, String::new(), tl!("(no numbering)"));
        for (id, sample) in &f.lists {
            ui.steady_value(&mut f.list, id.clone(), sample);
        }
        for (k, sample) in NEW_LISTS {
            ui.steady_value(&mut f.list, format!("kind:{k}"), crate::i18n::fmt(tl!("New list: {sample}"), &[("sample", sample)]));
        }
    });
    ui.add_space(6.0);
    ui.add_enabled_ui(!f.list.is_empty(), |ui| {
        ui.horizontal(|ui| {
            ui.label(tl!("Level:"));
            egui::ComboBox::from_id_salt("ms_list_level").selected_text(f.list_level.to_string()).show_ui(ui, |ui| {
                for l in 1..=9u8 {
                    ui.steady_value(&mut f.list_level, l, l.to_string());
                }
            });
        });
    });
    ui.add_space(6.0);
    ui.label(egui::RichText::new(tl!("The level is linked to this style, as Define New Multilevel List's Link level to style does.")).small().weak());
}

fn highlight_page(ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Highlighting");
    if ui.steady_label(f.highlight == Highlight::None, tl!("No Color")).clicked() {
        f.highlight = Highlight::None;
    }
    egui::Grid::new("ms_highlight").spacing(vec2(4.0, 4.0)).show(ui, |ui| {
        for (index, highlight) in Highlight::ALL.iter().enumerate().skip(1) {
            if let Some(color) = highlight.rgb() {
                let (rect, response) = ui.allocate_exact_size(vec2(24.0, 24.0), egui::Sense::click());
                ui.painter().rect_filled(rect.shrink(2.0), 2.0, crate::theme::c32(color));
                if f.highlight == *highlight {
                    ui.painter().rect_stroke(rect, 2.0, egui::Stroke::new(1.5, crate::theme::Tokens::get(ui.ctx()).accent), egui::StrokeKind::Inside);
                }
                if response.on_hover_text(highlight.name()).clicked() {
                    f.highlight = *highlight;
                }
            }
            if index % 8 == 0 {
                ui.end_row();
            }
        }
    });
}

fn area_page(app: &WordApp, ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Area");
    egui::Grid::new("ms_area").num_columns(2).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Fill:"));
        let theme = app.session.doc.settings.theme_colors.clone();
        crate::dialogs::color_field(ui, &theme, &mut f.shading, "No Color", "");
        ui.end_row();
    });
}

fn drop_cap_page(ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Drop Caps");
    let mut enabled = f.drop_cap > 0;
    if ui.checkbox(&mut enabled, tl!("Display drop caps")).changed() {
        f.drop_cap = if enabled { 3 } else { 0 };
    }
    if enabled {
        ui.horizontal(|ui| {
            ui.label(tl!("Lines:"));
            ui.add(egui::DragValue::new(&mut f.drop_cap).range(2..=10));
        });
    }
}

fn alignment_page(ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Alignment");
    for (v, l) in ALIGNS {
        ui.radio_value(&mut f.align, v.to_string(), tl!(l));
    }
    heading(ui, "Text direction");
    ui.horizontal(|ui| {
        ui.radio_value(&mut f.rtl, false, tl!("Left-to-right"));
        ui.radio_value(&mut f.rtl, true, tl!("Right-to-left"));
    });
}
