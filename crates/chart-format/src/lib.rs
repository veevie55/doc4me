//! How a chart's parts look, the same in every KrubOffice app (owner, 2026-11-19: "for chart
//! formatting, we need more settings", after Word's Format Chart Area pane): each part (chart
//! area, plot area, title, legend, the axes, the gridlines, each series) has a fill, a border,
//! effects (shadow, glow, soft edges) and, where it shows text, a text fill, outline and size.
//! Everything starts automatic: the app draws its own default until a person picks something.
//!
//! The three apps keep charts in the same DrawingML chart part (ECMA-376 Part 1 §21.2), so the
//! markup is written here too ([`sp_pr`], [`tx_pr`], [`run_props`]) and read back from a small
//! tree the app builds from its own XML reader ([`X`], [`read_sp_pr`], [`read_text`]).
//!
//! Shared source: keep this crate identical in all three KrubOffice repositories. No egui: the
//! apps' document models use it.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

/// An sRGB colour.
pub type Color = [u8; 3];

/// Points to EMUs (English Metric Units).
const EMU_PER_PT: f32 = 12_700.0;
/// The most series a chart keeps formats for.
pub const MAX_SERIES: usize = 256;

/// Automatic, nothing, or a solid colour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    #[default]
    Auto,
    None,
    Solid,
}

/// A fill: automatic (the app's default), none, or a solid colour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Fill {
    #[default]
    Auto,
    None,
    /// `transparency` in percent, 0–100.
    Solid {
        color: Color,
        transparency: f32,
    },
}

impl Fill {
    pub fn kind(&self) -> Kind {
        match self {
            Fill::Auto => Kind::Auto,
            Fill::None => Kind::None,
            Fill::Solid { .. } => Kind::Solid,
        }
    }

    /// The colour and its opacity (0–1), for a solid fill.
    pub fn solid(&self) -> Option<(Color, f32)> {
        match self {
            Fill::Solid { color, transparency } => Some((*color, 1.0 - transparency.clamp(0.0, 100.0) / 100.0)),
            _ => None,
        }
    }
}

/// A line's compound type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Compound {
    #[default]
    Single,
    Double,
    ThickThin,
    ThinThick,
    Triple,
}

impl Compound {
    pub const ALL: [Compound; 5] = [Compound::Single, Compound::Double, Compound::ThickThin, Compound::ThinThick, Compound::Triple];

    /// The English name the apps translate.
    pub fn name(self) -> &'static str {
        match self {
            Compound::Single => "Single",
            Compound::Double => "Double",
            Compound::ThickThin => "Thick Thin",
            Compound::ThinThick => "Thin Thick",
            Compound::Triple => "Triple",
        }
    }

    fn ooxml(self) -> &'static str {
        match self {
            Compound::Single => "sng",
            Compound::Double => "dbl",
            Compound::ThickThin => "thickThin",
            Compound::ThinThick => "thinThick",
            Compound::Triple => "tri",
        }
    }

    fn from_ooxml(v: &str) -> Compound {
        match v {
            "dbl" => Compound::Double,
            "thickThin" => Compound::ThickThin,
            "thinThick" => Compound::ThinThick,
            "tri" => Compound::Triple,
            _ => Compound::Single,
        }
    }
}

/// A line's dash.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Dash {
    #[default]
    Solid,
    Dotted,
    Dashed,
    LongDash,
    DashDot,
}

impl Dash {
    pub const ALL: [Dash; 5] = [Dash::Solid, Dash::Dotted, Dash::Dashed, Dash::LongDash, Dash::DashDot];

    /// The English name the apps translate.
    pub fn name(self) -> &'static str {
        match self {
            Dash::Solid => "Solid",
            Dash::Dotted => "Dotted",
            Dash::Dashed => "Dashed",
            Dash::LongDash => "Long dash",
            Dash::DashDot => "Dash dot",
        }
    }

    fn ooxml(self) -> &'static str {
        match self {
            Dash::Solid => "solid",
            Dash::Dotted => "sysDot",
            Dash::Dashed => "dash",
            Dash::LongDash => "lgDash",
            Dash::DashDot => "dashDot",
        }
    }

    fn from_ooxml(v: &str) -> Dash {
        match v {
            "sysDot" | "dot" | "sysDash" => Dash::Dotted,
            "dash" => Dash::Dashed,
            "lgDash" | "lgDashDot" | "lgDashDotDot" => Dash::LongDash,
            "dashDot" | "sysDashDot" | "sysDashDotDot" => Dash::DashDot,
            _ => Dash::Solid,
        }
    }

    /// Dash and gap lengths in multiples of the line width (`[]`: solid).
    pub fn pattern(self) -> &'static [f32] {
        match self {
            Dash::Solid => &[],
            Dash::Dotted => &[1.0, 1.0],
            Dash::Dashed => &[4.0, 3.0],
            Dash::LongDash => &[8.0, 3.0],
            Dash::DashDot => &[4.0, 3.0, 1.0, 3.0],
        }
    }
}

/// A solid border's look. Width in points, transparency in percent.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Line {
    pub color: Color,
    pub width: f32,
    pub transparency: f32,
    pub compound: Compound,
    pub dash: Dash,
}

impl Default for Line {
    fn default() -> Self {
        Line { color: [0x59, 0x59, 0x59], width: 0.75, transparency: 0.0, compound: Compound::Single, dash: Dash::Solid }
    }
}

/// A border: automatic, none, or a solid line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Border {
    #[default]
    Auto,
    None,
    Solid(Line),
}

impl Border {
    pub fn kind(&self) -> Kind {
        match self {
            Border::Auto => Kind::Auto,
            Border::None => Kind::None,
            Border::Solid(_) => Kind::Solid,
        }
    }

    /// The line, when solid.
    pub fn line(&self) -> Option<&Line> {
        match self {
            Border::Solid(l) => Some(l),
            _ => None,
        }
    }
}

/// An outer shadow. Blur and distance in points, angle in degrees (0 = right, clockwise),
/// transparency in percent.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Shadow {
    pub color: Color,
    pub transparency: f32,
    pub blur: f32,
    pub angle: f32,
    pub distance: f32,
}

impl Default for Shadow {
    /// Office's Offset: Bottom Right.
    fn default() -> Self {
        Shadow { color: [0, 0, 0], transparency: 60.0, blur: 4.0, angle: 45.0, distance: 3.0 }
    }
}

/// A glow around the part. Size in points, transparency in percent.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Glow {
    pub color: Color,
    pub size: f32,
    pub transparency: f32,
}

impl Default for Glow {
    fn default() -> Self {
        Glow { color: [0xF2, 0xB7, 0x3C], size: 5.0, transparency: 60.0 }
    }
}

/// A text outline. Width in points.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Outline {
    pub color: Color,
    pub width: f32,
}

impl Default for Outline {
    fn default() -> Self {
        Outline { color: [0, 0, 0], width: 0.75 }
    }
}

/// How a part's text looks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Text {
    pub fill: Fill,
    pub outline: Option<Outline>,
    /// Font size in points; `None` is the app's automatic size.
    pub size: Option<f32>,
}

impl Text {
    pub fn is_auto(&self) -> bool {
        *self == Text::default()
    }
}

/// How one part of a chart looks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Format {
    pub fill: Fill,
    pub border: Border,
    pub shadow: Option<Shadow>,
    pub glow: Option<Glow>,
    /// Soft edges radius in points; 0 is none.
    pub soft_edge: f32,
    pub text: Text,
}

impl Format {
    /// Nothing picked: the app draws its default.
    pub fn is_auto(&self) -> bool {
        *self == Format::default()
    }

    /// The shape properties are automatic (only the text may be set).
    pub fn shape_is_auto(&self) -> bool {
        Format { text: Text::default(), ..*self }.is_auto()
    }

    /// Applies one edit made in the pane.
    pub fn apply(&mut self, e: &Edit) {
        let line = |b: &mut Border| -> Line {
            match b {
                Border::Solid(l) => *l,
                _ => Line::default(),
            }
        };
        match *e {
            Edit::Fill(k) => {
                self.fill = match k {
                    Kind::Auto => Fill::Auto,
                    Kind::None => Fill::None,
                    Kind::Solid => match self.fill {
                        Fill::Solid { .. } => self.fill,
                        _ => Fill::Solid { color: [0x44, 0x72, 0xC4], transparency: 0.0 },
                    },
                }
            }
            Edit::FillColor(c) => {
                let transparency = match self.fill {
                    Fill::Solid { transparency, .. } => transparency,
                    _ => 0.0,
                };
                self.fill = Fill::Solid { color: c, transparency };
            }
            Edit::FillTransparency(t) => {
                if let Fill::Solid { transparency, .. } = &mut self.fill {
                    *transparency = clamp(t, 0.0, 100.0);
                }
            }
            Edit::Border(k) => {
                self.border = match k {
                    Kind::Auto => Border::Auto,
                    Kind::None => Border::None,
                    Kind::Solid => Border::Solid(line(&mut self.border)),
                }
            }
            Edit::BorderColor(c) => self.border = Border::Solid(Line { color: c, ..line(&mut self.border) }),
            Edit::BorderWidth(w) => self.border = Border::Solid(Line { width: clamp(w, 0.0, 100.0), ..line(&mut self.border) }),
            Edit::BorderTransparency(t) => self.border = Border::Solid(Line { transparency: clamp(t, 0.0, 100.0), ..line(&mut self.border) }),
            Edit::Compound(c) => self.border = Border::Solid(Line { compound: c, ..line(&mut self.border) }),
            Edit::Dash(d) => self.border = Border::Solid(Line { dash: d, ..line(&mut self.border) }),
            Edit::Shadow(on) => self.shadow = if on { Some(self.shadow.unwrap_or_default()) } else { None },
            Edit::ShadowColor(c) => self.shadow = Some(Shadow { color: c, ..self.shadow.unwrap_or_default() }),
            Edit::ShadowTransparency(t) => self.shadow = Some(Shadow { transparency: clamp(t, 0.0, 100.0), ..self.shadow.unwrap_or_default() }),
            Edit::ShadowBlur(b) => self.shadow = Some(Shadow { blur: clamp(b, 0.0, 100.0), ..self.shadow.unwrap_or_default() }),
            Edit::ShadowAngle(a) => self.shadow = Some(Shadow { angle: clamp(a, 0.0, 359.9), ..self.shadow.unwrap_or_default() }),
            Edit::ShadowDistance(d) => self.shadow = Some(Shadow { distance: clamp(d, 0.0, 200.0), ..self.shadow.unwrap_or_default() }),
            Edit::GlowSize(s) => {
                let s = clamp(s, 0.0, 150.0);
                self.glow = (s > 0.0).then(|| Glow { size: s, ..self.glow.unwrap_or_default() });
            }
            Edit::GlowColor(c) => self.glow = Some(Glow { color: c, ..self.glow.unwrap_or_default() }),
            Edit::GlowTransparency(t) => self.glow = Some(Glow { transparency: clamp(t, 0.0, 100.0), ..self.glow.unwrap_or_default() }),
            Edit::SoftEdge(r) => self.soft_edge = clamp(r, 0.0, 100.0),
            Edit::TextFill(k) => {
                self.text.fill = match k {
                    Kind::Auto => Fill::Auto,
                    Kind::None => Fill::None,
                    Kind::Solid => match self.text.fill {
                        Fill::Solid { .. } => self.text.fill,
                        _ => Fill::Solid { color: [0x40, 0x40, 0x40], transparency: 0.0 },
                    },
                }
            }
            Edit::TextColor(c) => {
                let transparency = match self.text.fill {
                    Fill::Solid { transparency, .. } => transparency,
                    _ => 0.0,
                };
                self.text.fill = Fill::Solid { color: c, transparency };
            }
            Edit::TextTransparency(t) => {
                if let Fill::Solid { transparency, .. } = &mut self.text.fill {
                    *transparency = clamp(t, 0.0, 100.0);
                }
            }
            Edit::TextOutline(on) => self.text.outline = if on { Some(self.text.outline.unwrap_or_default()) } else { None },
            Edit::TextOutlineColor(c) => self.text.outline = Some(Outline { color: c, ..self.text.outline.unwrap_or_default() }),
            Edit::TextOutlineWidth(w) => self.text.outline = Some(Outline { width: clamp(w, 0.0, 20.0), ..self.text.outline.unwrap_or_default() }),
            Edit::FontSize(s) => self.text.size = s.map(|s| clamp(s, 1.0, 400.0)),
            Edit::RoundedCorners(_) | Edit::Reset => {}
        }
    }
}

/// A finite value inside `lo..=hi` (`lo` for NaN).
fn clamp(v: f32, lo: f32, hi: f32) -> f32 {
    if v.is_finite() { v.clamp(lo, hi) } else { lo }
}

/// A part of a chart that can be formatted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Part {
    ChartArea,
    PlotArea,
    Title,
    Legend,
    CategoryAxis,
    ValueAxis,
    Gridlines,
    Series(usize),
}

impl Part {
    /// The English name the apps translate (a series is named by the app).
    pub fn name(self) -> &'static str {
        match self {
            Part::ChartArea => "Chart Area",
            Part::PlotArea => "Plot Area",
            Part::Title => "Chart Title",
            Part::Legend => "Legend",
            Part::CategoryAxis => "Horizontal Axis",
            Part::ValueAxis => "Vertical Axis",
            Part::Gridlines => "Gridlines",
            Part::Series(_) => "Series",
        }
    }

    /// It has an area to fill (not the axes or the gridlines, which are lines).
    pub fn has_fill(self) -> bool {
        !matches!(self, Part::CategoryAxis | Part::ValueAxis | Part::Gridlines)
    }

    /// It shows text.
    pub fn has_text(self) -> bool {
        matches!(self, Part::ChartArea | Part::Title | Part::Legend | Part::CategoryAxis | Part::ValueAxis)
    }

    /// It can have a shadow, glow and soft edges.
    pub fn has_effects(self) -> bool {
        !matches!(self, Part::Gridlines)
    }
}

/// The parts of one chart.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Formats {
    pub chart_area: Format,
    pub plot_area: Format,
    pub title: Format,
    pub legend: Format,
    pub category_axis: Format,
    pub value_axis: Format,
    pub gridlines: Format,
    /// One per series, in order; missing ones are automatic.
    pub series: Vec<Format>,
    /// The chart area's corners are rounded.
    pub rounded_corners: bool,
}

impl Formats {
    /// Nothing picked anywhere.
    pub fn is_auto(&self) -> bool {
        *self == Formats::default()
            || (self.series.iter().all(Format::is_auto) && Formats { series: Vec::new(), ..self.clone() } == Formats::default())
    }

    /// `part`'s format (automatic for a series without one).
    pub fn get(&self, part: Part) -> Format {
        match part {
            Part::ChartArea => self.chart_area,
            Part::PlotArea => self.plot_area,
            Part::Title => self.title,
            Part::Legend => self.legend,
            Part::CategoryAxis => self.category_axis,
            Part::ValueAxis => self.value_axis,
            Part::Gridlines => self.gridlines,
            Part::Series(i) => self.series.get(i).copied().unwrap_or_default(),
        }
    }

    /// `part`'s format to change; `None` for a series past [`MAX_SERIES`].
    pub fn get_mut(&mut self, part: Part) -> Option<&mut Format> {
        Some(match part {
            Part::ChartArea => &mut self.chart_area,
            Part::PlotArea => &mut self.plot_area,
            Part::Title => &mut self.title,
            Part::Legend => &mut self.legend,
            Part::CategoryAxis => &mut self.category_axis,
            Part::ValueAxis => &mut self.value_axis,
            Part::Gridlines => &mut self.gridlines,
            Part::Series(i) => {
                if i >= MAX_SERIES {
                    return None;
                }
                if self.series.len() <= i {
                    self.series.resize(i + 1, Format::default());
                }
                self.series.get_mut(i)?
            }
        })
    }

    /// Applies an edit to `part`; `Reset` makes it automatic again.
    pub fn apply(&mut self, part: Part, e: &Edit) {
        match e {
            Edit::RoundedCorners(on) => self.rounded_corners = *on,
            Edit::Reset => {
                if let Some(f) = self.get_mut(part) {
                    *f = Format::default();
                }
            }
            e => {
                if let Some(f) = self.get_mut(part) {
                    f.apply(e);
                }
            }
        }
        while self.series.last().is_some_and(Format::is_auto) {
            self.series.pop();
        }
    }
}

/// A change made in the pane. Sizes in points, transparencies in percent, angles in degrees.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Edit {
    Fill(Kind),
    FillColor(Color),
    FillTransparency(f32),
    Border(Kind),
    BorderColor(Color),
    BorderWidth(f32),
    BorderTransparency(f32),
    Compound(Compound),
    Dash(Dash),
    /// The chart area's corners (whatever part is chosen).
    RoundedCorners(bool),
    Shadow(bool),
    ShadowColor(Color),
    ShadowTransparency(f32),
    ShadowBlur(f32),
    ShadowAngle(f32),
    ShadowDistance(f32),
    /// 0 removes the glow.
    GlowSize(f32),
    GlowColor(Color),
    GlowTransparency(f32),
    /// 0 removes the soft edges.
    SoftEdge(f32),
    TextFill(Kind),
    TextColor(Color),
    TextTransparency(f32),
    TextOutline(bool),
    TextOutlineColor(Color),
    TextOutlineWidth(f32),
    /// `None`: automatic.
    FontSize(Option<f32>),
    /// The part back to automatic.
    Reset,
}

impl Edit {
    /// The edit as JSON-like key and value, for an app command (`{"edit": key, "value": v}`).
    /// Colours are `"RRGGBB"`, kinds `auto`/`none`/`solid`.
    pub fn to_pair(&self) -> (&'static str, String) {
        let kind = |k: Kind| match k {
            Kind::Auto => "auto",
            Kind::None => "none",
            Kind::Solid => "solid",
        };
        let n = |v: f32| format!("{v}");
        let hex = |c: Color| format!("{:02X}{:02X}{:02X}", c[0], c[1], c[2]);
        match *self {
            Edit::Fill(k) => ("fill", kind(k).into()),
            Edit::FillColor(c) => ("fillColor", hex(c)),
            Edit::FillTransparency(v) => ("fillTransparency", n(v)),
            Edit::Border(k) => ("border", kind(k).into()),
            Edit::BorderColor(c) => ("borderColor", hex(c)),
            Edit::BorderWidth(v) => ("borderWidth", n(v)),
            Edit::BorderTransparency(v) => ("borderTransparency", n(v)),
            Edit::Compound(c) => ("compound", c.ooxml().into()),
            Edit::Dash(d) => ("dash", d.ooxml().into()),
            Edit::RoundedCorners(b) => ("roundedCorners", b.to_string()),
            Edit::Shadow(b) => ("shadow", b.to_string()),
            Edit::ShadowColor(c) => ("shadowColor", hex(c)),
            Edit::ShadowTransparency(v) => ("shadowTransparency", n(v)),
            Edit::ShadowBlur(v) => ("shadowBlur", n(v)),
            Edit::ShadowAngle(v) => ("shadowAngle", n(v)),
            Edit::ShadowDistance(v) => ("shadowDistance", n(v)),
            Edit::GlowSize(v) => ("glowSize", n(v)),
            Edit::GlowColor(c) => ("glowColor", hex(c)),
            Edit::GlowTransparency(v) => ("glowTransparency", n(v)),
            Edit::SoftEdge(v) => ("softEdge", n(v)),
            Edit::TextFill(k) => ("textFill", kind(k).into()),
            Edit::TextColor(c) => ("textColor", hex(c)),
            Edit::TextTransparency(v) => ("textTransparency", n(v)),
            Edit::TextOutline(b) => ("textOutline", b.to_string()),
            Edit::TextOutlineColor(c) => ("textOutlineColor", hex(c)),
            Edit::TextOutlineWidth(v) => ("textOutlineWidth", n(v)),
            Edit::FontSize(s) => ("fontSize", s.map_or_else(|| "auto".into(), n)),
            Edit::Reset => ("reset", "true".into()),
        }
    }

    /// The edit from [`Edit::to_pair`]'s key and value; `None` when either is not understood.
    pub fn from_pair(key: &str, value: &str) -> Option<Edit> {
        let kind = || match value {
            "auto" => Some(Kind::Auto),
            "none" => Some(Kind::None),
            "solid" => Some(Kind::Solid),
            _ => None,
        };
        let n = || value.trim().parse::<f32>().ok().filter(|v| v.is_finite());
        let b = || match value {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        };
        let c = || parse_hex(value);
        Some(match key {
            "fill" => Edit::Fill(kind()?),
            "fillColor" => Edit::FillColor(c()?),
            "fillTransparency" => Edit::FillTransparency(n()?),
            "border" => Edit::Border(kind()?),
            "borderColor" => Edit::BorderColor(c()?),
            "borderWidth" => Edit::BorderWidth(n()?),
            "borderTransparency" => Edit::BorderTransparency(n()?),
            "compound" => Edit::Compound(Compound::ALL.into_iter().find(|x| x.ooxml() == value)?),
            "dash" => Edit::Dash(Dash::ALL.into_iter().find(|x| x.ooxml() == value)?),
            "roundedCorners" => Edit::RoundedCorners(b()?),
            "shadow" => Edit::Shadow(b()?),
            "shadowColor" => Edit::ShadowColor(c()?),
            "shadowTransparency" => Edit::ShadowTransparency(n()?),
            "shadowBlur" => Edit::ShadowBlur(n()?),
            "shadowAngle" => Edit::ShadowAngle(n()?),
            "shadowDistance" => Edit::ShadowDistance(n()?),
            "glowSize" => Edit::GlowSize(n()?),
            "glowColor" => Edit::GlowColor(c()?),
            "glowTransparency" => Edit::GlowTransparency(n()?),
            "softEdge" => Edit::SoftEdge(n()?),
            "textFill" => Edit::TextFill(kind()?),
            "textColor" => Edit::TextColor(c()?),
            "textTransparency" => Edit::TextTransparency(n()?),
            "textOutline" => Edit::TextOutline(b()?),
            "textOutlineColor" => Edit::TextOutlineColor(c()?),
            "textOutlineWidth" => Edit::TextOutlineWidth(n()?),
            "fontSize" => Edit::FontSize(if value == "auto" { None } else { Some(n()?) }),
            "reset" => Edit::Reset,
            _ => return None,
        })
    }
}

/// A part from its command name (`chartArea`, `plotArea`, `title`, `legend`, `categoryAxis`,
/// `valueAxis`, `gridlines`, `series1`…).
pub fn part_from_id(id: &str) -> Option<Part> {
    Some(match id {
        "chartArea" => Part::ChartArea,
        "plotArea" => Part::PlotArea,
        "title" => Part::Title,
        "legend" => Part::Legend,
        "categoryAxis" => Part::CategoryAxis,
        "valueAxis" => Part::ValueAxis,
        "gridlines" => Part::Gridlines,
        s => {
            let n: usize = s.strip_prefix("series")?.parse().ok()?;
            if n == 0 || n > MAX_SERIES {
                return None;
            }
            Part::Series(n - 1)
        }
    })
}

/// [`part_from_id`]'s inverse.
pub fn part_id(p: Part) -> String {
    match p {
        Part::ChartArea => "chartArea".into(),
        Part::PlotArea => "plotArea".into(),
        Part::Title => "title".into(),
        Part::Legend => "legend".into(),
        Part::CategoryAxis => "categoryAxis".into(),
        Part::ValueAxis => "valueAxis".into(),
        Part::Gridlines => "gridlines".into(),
        Part::Series(i) => format!("series{}", i.saturating_add(1)),
    }
}

/// `"RRGGBB"` (a leading `#` allowed) to a colour.
pub fn parse_hex(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 || !s.is_ascii() {
        return None;
    }
    let p = |i: usize| s.get(i..i + 2).and_then(|h| u8::from_str_radix(h, 16).ok());
    Some([p(0)?, p(2)?, p(4)?])
}

// ---------------------------------------------------------------------------------------------
// Writing

fn emu(pt: f32) -> i64 {
    (clamp(pt, 0.0, 10_000.0) * EMU_PER_PT).round() as i64
}

fn color_xml(out: &mut String, c: Color, transparency: f32) {
    let alpha = ((100.0 - clamp(transparency, 0.0, 100.0)) * 1000.0).round() as i64;
    let _ = write!(out, "<a:srgbClr val=\"{:02X}{:02X}{:02X}\"", c[0], c[1], c[2]);
    if alpha < 100_000 {
        let _ = write!(out, "><a:alpha val=\"{alpha}\"/></a:srgbClr>");
    } else {
        out.push_str("/>");
    }
}

fn fill_xml(out: &mut String, f: &Fill) {
    match f {
        Fill::Auto => {}
        Fill::None => out.push_str("<a:noFill/>"),
        Fill::Solid { color, transparency } => {
            out.push_str("<a:solidFill>");
            color_xml(out, *color, *transparency);
            out.push_str("</a:solidFill>");
        }
    }
}

fn line_xml(out: &mut String, b: &Border) {
    match b {
        Border::Auto => {}
        Border::None => out.push_str("<a:ln><a:noFill/></a:ln>"),
        Border::Solid(l) => {
            let _ = write!(out, "<a:ln w=\"{}\" cmpd=\"{}\"><a:solidFill>", emu(l.width), l.compound.ooxml());
            color_xml(out, l.color, l.transparency);
            let _ = write!(out, "</a:solidFill><a:prstDash val=\"{}\"/></a:ln>", l.dash.ooxml());
        }
    }
}

fn effects_xml(out: &mut String, f: &Format) {
    if f.shadow.is_none() && f.glow.is_none() && f.soft_edge <= 0.0 {
        return;
    }
    // CT_EffectList's order: blur, fillOverlay, glow, innerShdw, outerShdw, prstShdw,
    // reflection, softEdge.
    out.push_str("<a:effectLst>");
    if let Some(g) = &f.glow {
        let _ = write!(out, "<a:glow rad=\"{}\">", emu(g.size));
        color_xml(out, g.color, g.transparency);
        out.push_str("</a:glow>");
    }
    if let Some(s) = &f.shadow {
        let dir = ((clamp(s.angle, 0.0, 359.9) * 60_000.0).round() as i64).rem_euclid(21_600_000);
        let _ = write!(out, "<a:outerShdw blurRad=\"{}\" dist=\"{}\" dir=\"{dir}\" algn=\"ctr\" rotWithShape=\"0\">", emu(s.blur), emu(s.distance));
        color_xml(out, s.color, s.transparency);
        out.push_str("</a:outerShdw>");
    }
    if f.soft_edge > 0.0 {
        let _ = write!(out, "<a:softEdge rad=\"{}\"/>", emu(f.soft_edge));
    }
    out.push_str("</a:effectLst>");
}

/// The part's `c:spPr` (fill, border, effects), or `""` when all of them are automatic.
pub fn sp_pr(f: &Format) -> String {
    if f.shape_is_auto() {
        return String::new();
    }
    let mut out = String::from("<c:spPr>");
    fill_xml(&mut out, &f.fill);
    line_xml(&mut out, &f.border);
    effects_xml(&mut out, f);
    out.push_str("</c:spPr>");
    out
}

/// The attributes and children of an `a:defRPr` or `a:rPr` for `t`: (`" sz=\"1400\""`,
/// `"<a:ln>…</a:ln><a:solidFill>…</a:solidFill>"`). Both empty when automatic.
pub fn run_props(t: &Text) -> (String, String) {
    let attrs = t.size.map(|s| format!(" sz=\"{}\"", (clamp(s, 1.0, 400.0) * 100.0).round() as i64)).unwrap_or_default();
    let mut kids = String::new();
    // CT_TextCharacterProperties: ln before the fill.
    if let Some(o) = &t.outline {
        let _ = write!(kids, "<a:ln w=\"{}\"><a:solidFill>", emu(o.width));
        color_xml(&mut kids, o.color, 0.0);
        kids.push_str("</a:solidFill></a:ln>");
    }
    fill_xml(&mut kids, &t.fill);
    (attrs, kids)
}

/// The part's `c:txPr` for `t`, or `""` when automatic.
pub fn tx_pr(t: &Text) -> String {
    if t.is_auto() {
        return String::new();
    }
    let (attrs, kids) = run_props(t);
    format!("<c:txPr><a:bodyPr/><a:lstStyle/><a:p><a:pPr><a:defRPr{attrs}>{kids}</a:defRPr></a:pPr><a:endParaRPr lang=\"en-US\"/></a:p></c:txPr>")
}

/// `c:roundedCorners` for the chart space (always written: its default is on).
pub fn rounded_corners(on: bool) -> String {
    format!("<c:roundedCorners val=\"{}\"/>", u8::from(on))
}

// ---------------------------------------------------------------------------------------------
// Reading

/// A small XML tree the app builds from its own reader: the local name (no prefix), the
/// attributes by local name, and the child elements.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct X {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub kids: Vec<X>,
}

impl X {
    pub fn attr(&self, k: &str) -> Option<&str> {
        self.attrs.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str())
    }

    pub fn child(&self, name: &str) -> Option<&X> {
        self.kids.iter().find(|k| k.name == name)
    }

    fn num(&self, k: &str) -> Option<f32> {
        self.attr(k)?.trim().parse::<f64>().ok().filter(|v| v.is_finite()).map(|v| v as f32)
    }
}

/// The colour and transparency of a colour choice (`a:srgbClr`; other colour kinds are not
/// understood here and give `None`).
fn read_color(x: &X) -> Option<(Color, f32)> {
    let c = x.child("srgbClr")?;
    let color = parse_hex(c.attr("val")?)?;
    let alpha = c.child("alpha").and_then(|a| a.num("val")).unwrap_or(100_000.0);
    Some((color, clamp(100.0 - alpha / 1000.0, 0.0, 100.0)))
}

fn read_fill(parent: &X) -> Option<Fill> {
    if parent.child("noFill").is_some() {
        return Some(Fill::None);
    }
    let (color, transparency) = read_color(parent.child("solidFill")?)?;
    Some(Fill::Solid { color, transparency })
}

fn pt(x: &X, k: &str) -> Option<f32> {
    x.num(k).map(|v| clamp(v / EMU_PER_PT, 0.0, 10_000.0))
}

/// What of a `c:spPr` this crate understands: solid and no fills, solid and no lines, outer
/// shadow, glow and soft edges in sRGB. Anything else is automatic, so an app that must keep
/// it checks that writing the result gives back the same markup.
pub fn read_sp_pr(sp: &X) -> Format {
    let mut f = Format { fill: read_fill(sp).unwrap_or_default(), ..Format::default() };
    if let Some(ln) = sp.child("ln") {
        if ln.child("noFill").is_some() {
            f.border = Border::None;
        } else if let Some((color, transparency)) = ln.child("solidFill").and_then(read_color) {
            f.border = Border::Solid(Line {
                color,
                transparency,
                width: pt(ln, "w").unwrap_or(0.75),
                compound: ln.attr("cmpd").map(Compound::from_ooxml).unwrap_or_default(),
                dash: ln.child("prstDash").and_then(|d| d.attr("val")).map(Dash::from_ooxml).unwrap_or_default(),
            });
        }
    }
    if let Some(fx) = sp.child("effectLst") {
        if let Some(g) = fx.child("glow")
            && let Some((color, transparency)) = read_color(g)
        {
            f.glow = Some(Glow { color, transparency, size: pt(g, "rad").unwrap_or(0.0) });
        }
        if let Some(s) = fx.child("outerShdw")
            && let Some((color, transparency)) = read_color(s)
        {
            f.shadow = Some(Shadow {
                color,
                transparency,
                blur: pt(s, "blurRad").unwrap_or(0.0),
                distance: pt(s, "dist").unwrap_or(0.0),
                angle: clamp(s.num("dir").unwrap_or(0.0) / 60_000.0, 0.0, 359.9),
            });
        }
        if let Some(s) = fx.child("softEdge") {
            f.soft_edge = pt(s, "rad").unwrap_or(0.0);
        }
    }
    f
}

/// The text look of an `a:defRPr` or `a:rPr`.
pub fn read_text(rpr: &X) -> Text {
    Text {
        fill: read_fill(rpr).unwrap_or_default(),
        outline: rpr.child("ln").and_then(|ln| {
            let (color, _) = ln.child("solidFill").and_then(read_color)?;
            Some(Outline { color, width: pt(ln, "w").unwrap_or(0.75) })
        }),
        size: rpr.num("sz").map(|s| clamp(s / 100.0, 1.0, 400.0)),
    }
}

/// The part's format from its element (`c:plotArea`, `c:legend`, `c:ser`…): its `c:spPr` and
/// the first `a:defRPr` of its `c:txPr` (or, for a title, of its rich text).
pub fn read_part(el: &X) -> Format {
    let mut f = el.child("spPr").map(read_sp_pr).unwrap_or_default();
    let def = |p: &X| -> Option<Text> {
        let para = p.child("p")?;
        let rpr = para.child("pPr").and_then(|pp| pp.child("defRPr")).or_else(|| para.child("r").and_then(|r| r.child("rPr")))?;
        Some(read_text(rpr))
    };
    let text = el.child("txPr").and_then(def).or_else(|| el.child("tx").and_then(|t| t.child("rich")).and_then(def));
    if let Some(t) = text {
        f.text = t;
    }
    f
}

/// A colour choice this crate writes back the same: one `a:srgbClr` with at most an `a:alpha`.
fn color_understood(x: &X) -> bool {
    matches!(x.kids.as_slice(), [c] if c.name == "srgbClr" && c.kids.iter().all(|k| k.name == "alpha"))
}

/// Everything in this `c:spPr` is something [`sp_pr`] writes back (solid and no fills in sRGB,
/// solid and no lines with a preset dash, glow, outer shadow, soft edges), so the part can be
/// formatted from it without losing anything.
pub fn sp_pr_understood(sp: &X) -> bool {
    sp.kids.iter().all(|k| match k.name.as_str() {
        "noFill" => true,
        "solidFill" => color_understood(k),
        "ln" => k.kids.iter().all(|l| match l.name.as_str() {
            "noFill" | "prstDash" => true,
            "solidFill" => color_understood(l),
            _ => false,
        }),
        "effectLst" => k.kids.iter().all(|e| match e.name.as_str() {
            "glow" | "outerShdw" => color_understood(e),
            "softEdge" => true,
            _ => false,
        }),
        _ => false,
    })
}

/// Everything in this `a:defRPr` or `a:rPr` that sets a look is something [`run_props`] writes
/// back (attributes other than the size, such as `b` or `lang`, are not looks this crate keeps
/// and make it not understood when they set a weight or face).
pub fn text_understood(rpr: &X) -> bool {
    let attrs_ok = rpr.attrs.iter().all(|(k, _)| matches!(k.as_str(), "sz" | "lang" | "altLang" | "dirty" | "baseline" | "kern" | "spc"));
    attrs_ok
        && rpr.kids.iter().all(|k| match k.name.as_str() {
            "noFill" => true,
            "solidFill" => color_understood(k),
            "ln" => k.kids.iter().all(|l| l.name == "solidFill" && color_understood(l)),
            "latin" | "ea" | "cs" => false,
            _ => false,
        })
}

/// [`read_part`], but each half only when it is fully understood: a `c:spPr` with anything
/// [`sp_pr_understood`] doesn't know leaves the fill, border and effects automatic, and text
/// [`text_understood`] doesn't know leaves the text automatic. The app then keeps writing those
/// the way it did before.
pub fn read_part_strict(el: &X) -> Format {
    let mut f = match el.child("spPr") {
        Some(sp) if sp_pr_understood(sp) => read_sp_pr(sp),
        _ => Format::default(),
    };
    let rpr = |p: &X| -> Option<X> {
        let para = p.child("p")?;
        para.child("pPr").and_then(|pp| pp.child("defRPr")).or_else(|| para.child("r").and_then(|r| r.child("rPr"))).cloned()
    };
    let text = el.child("txPr").and_then(rpr).or_else(|| el.child("tx").and_then(|t| t.child("rich")).and_then(rpr));
    if let Some(r) = text.filter(text_understood) {
        f.text = read_text(&r);
    }
    f
}
