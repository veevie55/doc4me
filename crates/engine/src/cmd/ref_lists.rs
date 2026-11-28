//! The References side panel's lists (owner, 2026-11-22): the table of contents, the list of
//! figures and the list of tables, each found in the document with its settings, and one
//! command that inserts, changes (rebuilding it where it is) or removes any of them.
//!
//! Settings live in the document, so a reopened file shows them again: a table of contents'
//! levels in its TOC field, a list of figures' caption label in its `TOC \c` field, the leader
//! in the entries' tab stops, the Thai academic report layout in its page label paragraph
//! (`continued`, with `ตารางที่\tหน้า` for the column heading), and how each line shows the
//! caption's label in a custom document property ([`citations::LABEL_MODE_PROP`]).

use doc4me_doc::para::OBJ;
use doc4me_doc::props::{TabAlign, TabLeader};
use doc4me_doc::{Block, Paragraph, Path, Pos, StoryRef};
use serde_json::{Value, json};

use super::citations::{self, LabelMode};
use super::continued;
use super::references::{self, FieldCode};
use super::sel_result;
use crate::{CmdError, CmdResult, CommandSpec, Selection, Session, p};

/// Which list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Contents,
    Figures,
    Tables,
}

impl Kind {
    pub const ALL: [Kind; 3] = [Kind::Contents, Kind::Figures, Kind::Tables];

    pub fn name(self) -> &'static str {
        match self {
            Kind::Contents => "contents",
            Kind::Figures => "figures",
            Kind::Tables => "tables",
        }
    }

    pub fn from_name(n: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.name() == n)
    }

    /// The caption labels that make a table of figures this kind's, Thai first.
    fn labels(self) -> &'static [&'static str] {
        match self {
            Kind::Contents => &[],
            Kind::Figures => &[citations::THAI_FIGURE, "Figure"],
            Kind::Tables => &[citations::THAI_TABLE, "Table"],
        }
    }

    /// The kind of the table of figures for caption `label`: tables for a table label, figures
    /// for any other (Figure, ภาพที่, รูปที่, Equation…).
    fn of_label(label: &str) -> Kind {
        if Kind::Tables.labels().contains(&label) { Kind::Tables } else { Kind::Figures }
    }
}

/// A list as the panel shows it: where it is (if it is in the document) and its settings, or
/// the settings a new one would start with.
#[derive(Clone, Debug, PartialEq)]
pub struct ListInfo {
    pub kind: Kind,
    /// The body block of its heading, while the list is in the document.
    pub block: Option<usize>,
    /// The page its heading is on, as the page shows its number (`ก`, `iii`, `5`).
    pub page: Option<String>,
    /// The Thai academic report layout: "หน้า" over the page numbers and "(ต่อ)" headings.
    pub report: bool,
    /// Heading levels a table of contents shows (1–9).
    pub levels: u8,
    pub leader: TabLeader,
    /// The caption label a list of figures or tables collects.
    pub label: String,
    pub mode: LabelMode,
    /// The report layout's column heading (`ตารางที่` over the numbers, left of "หน้า").
    pub column_heading: bool,
    pub title: String,
}

impl ListInfo {
    pub fn to_json(&self) -> Value {
        json!({
            "kind": self.kind.name(),
            "inDocument": self.block.is_some(),
            "page": self.page,
            "layout": if self.report { "report" } else { "general" },
            "levels": self.levels,
            "leader": leader_name(self.leader),
            "label": self.label,
            "show": self.mode.name(),
            "columnHeading": self.column_heading,
            "title": self.title,
        })
    }
}

pub fn leader_name(l: TabLeader) -> &'static str {
    match l {
        TabLeader::None => "none",
        TabLeader::Hyphen => "hyphen",
        TabLeader::Underscore => "underscore",
        TabLeader::MiddleDot => "middleDot",
        _ => "dot",
    }
}

/// A heading's text without its field markers.
fn heading_text(p: &Paragraph) -> String {
    p.plain_text().chars().filter(|c| *c != OBJ && !c.is_control()).collect::<String>().trim().to_string()
}

/// The page body block `i` starts on, as it is numbered there.
fn page_of_block(s: &mut Session, i: usize) -> String {
    let formats = references::page_formats(s);
    let l = s.layout();
    references::page_label_or_first(&l, &formats, &Pos { story: StoryRef::Body, path: Path::top(i), off: 0 }).1
}

/// The right-tab leader of the first entry in blocks `from..=to`.
fn leader_in(s: &Session, from: usize, to: usize) -> Option<TabLeader> {
    (from..=to).filter_map(|k| s.doc.body.get(k)?.as_para()).find_map(|p| {
        if continued::is_part(p, continued::TOC) || continued::is_part(p, continued::LIST) {
            return None;
        }
        p.props.tabs.as_ref()?.iter().find(|t| t.align == TabAlign::Right).map(|t| t.leader)
    })
}

/// The table of figures of `kind` in the body: its heading block, last block and field code.
fn caption_list(s: &Session, kind: Kind) -> Option<(usize, usize, String)> {
    let mut i = 0;
    while i < s.doc.body.len() {
        let code = s.doc.body.get(i).and_then(|b| b.as_para()).and_then(citations::list_kind);
        if let Some(code) = code.filter(|c| c.starts_with("TOC \\c")) {
            let label = FieldCode::parse(&code).arg('c').unwrap_or("").to_string();
            if Kind::of_label(&label) == kind {
                return Some((i, list_end(s, i), code));
            }
        }
        i += 1;
    }
    None
}

/// The list body block `block` is part of (its heading, page label lines or entries), if any.
pub fn list_at(s: &Session, block: usize) -> Option<Kind> {
    if references::toc_range(s).is_some_and(|(a, b)| (a..=b).contains(&block)) {
        return Some(Kind::Contents);
    }
    [Kind::Figures, Kind::Tables].into_iter().find(|k| caption_list(s, *k).is_some_and(|(a, b, _)| (a..=b).contains(&block)))
}

/// The last block of the table of figures whose heading is block `head`.
fn list_end(s: &Session, head: usize) -> usize {
    let in_list = |p: &Paragraph| p.props.style.as_deref() == Some(citations::ENTRY_STYLE) || continued::is_part(p, continued::LIST);
    let mut last = head;
    while s.doc.body.get(last + 1).and_then(|b| b.as_para()).is_some_and(in_list) {
        last += 1;
    }
    last
}

/// The caption label a new list of `kind` collects: the Thai one when the document has Thai
/// captions of that kind or is set up for Thai, else the English one.
fn default_label(s: &Session, kind: Kind) -> String {
    let (thai, english) = match kind {
        Kind::Tables => (citations::THAI_TABLE, "Table"),
        _ => (citations::THAI_FIGURE, "Figure"),
    };
    if has_caption(s, thai) || (!has_caption(s, english) && thai_text(s)) { thai } else { english }.to_string()
}

/// Whether the document is written in Thai: its language is Thai, or it has Thai captions or
/// Thai text in its first paragraphs (a Thai report kept with an English language setting).
fn thai_text(s: &Session) -> bool {
    let thai = |t: &str| t.chars().any(|c| ('\u{0E00}'..='\u{0E7F}').contains(&c));
    super::default_lang(s).is_some_and(|l| l.starts_with("th"))
        || [citations::THAI_FIGURE, citations::THAI_TABLE].iter().any(|l| has_caption(s, l))
        || s.doc.body.iter().take(400).filter_map(|b| b.as_para()).any(|p| thai(&p.plain_text()))
}

/// Whether the body has a caption (Caption style) that starts with `label`, as the tables of
/// figures collect them. Tables and pictures without a caption don't count.
fn has_caption(s: &Session, label: &str) -> bool {
    s.doc.para_paths(StoryRef::Body).into_iter().any(|path| {
        s.doc.para(StoryRef::Body, &path).is_some_and(|p| p.props.style.as_deref() == Some("Caption") && p.plain_text().starts_with(label))
    })
}

/// A Thai caption label (ภาพที่, ตารางที่, รูปที่…).
fn is_thai(label: &str) -> bool {
    label.chars().any(|c| ('\u{0E00}'..='\u{0E7F}').contains(&c))
}

/// The list of `kind` as it is in the document, or the settings a new one starts with.
pub fn list_info(s: &mut Session, kind: Kind) -> ListInfo {
    let thai_doc = super::default_lang(s).is_some_and(|l| l.starts_with("th"));
    match kind {
        Kind::Contents => {
            let found = references::toc_range(s).and_then(|(a, b)| Some((a, b, s.doc.body.get(a)?.as_para().and_then(references::toc_field)?)));
            let Some((start, end, instr)) = found else {
                return ListInfo {
                    kind,
                    block: None,
                    page: None,
                    report: false,
                    levels: 3,
                    leader: TabLeader::Dot,
                    label: String::new(),
                    mode: LabelMode::Full,
                    column_heading: false,
                    title: if thai_doc { "สารบัญ" } else { "Contents" }.to_string(),
                };
            };
            let f = FieldCode::parse(&instr);
            let levels = f.arg('o').and_then(references::level_range).map_or(3, |r| r.1);
            let report = continued::label_in(s, start + 1, end, continued::TOC).is_some();
            let leader = leader_in(s, start + 1, end).unwrap_or(if report { TabLeader::None } else { TabLeader::Dot });
            let title = s.doc.body.get(start).and_then(|b| b.as_para()).map(heading_text).unwrap_or_default();
            let page = page_of_block(s, start);
            ListInfo {
                kind,
                block: Some(start),
                page: Some(page),
                report,
                levels,
                leader,
                label: String::new(),
                mode: LabelMode::Full,
                column_heading: false,
                title,
            }
        }
        _ => {
            let Some((head, end, code)) = caption_list(s, kind) else {
                let label = default_label(s, kind);
                let mode = citations::label_mode(&s.doc, &label);
                return ListInfo {
                    kind,
                    block: None,
                    page: None,
                    report: false,
                    levels: 3,
                    leader: TabLeader::Dot,
                    label,
                    mode,
                    column_heading: false,
                    title: String::new(),
                };
            };
            let label = FieldCode::parse(&code).arg('c').unwrap_or("").to_string();
            let page_label = continued::label_in(s, head + 1, end, continued::LIST);
            let report = page_label.is_some();
            let column_heading = page_label.as_deref().and_then(continued::column_heading).is_some();
            let leader = leader_in(s, head + 1, end).unwrap_or(if report { TabLeader::None } else { TabLeader::Dot });
            let mode = citations::label_mode(&s.doc, &label);
            let title = s.doc.body.get(head).and_then(|b| b.as_para()).map(heading_text).unwrap_or_default();
            let page = page_of_block(s, head);
            ListInfo { kind, block: Some(head), page: Some(page), report, levels: 3, leader, label, mode, column_heading, title }
        }
    }
}

/// The default heading of a list of `kind` in a layout: what a new list gets, and what a
/// change of layout replaces (a heading the writer typed stays).
fn default_title(kind: Kind, report: bool, thai: bool) -> &'static str {
    match (kind, report, thai) {
        (Kind::Contents, true, _) | (Kind::Contents, false, true) => "สารบัญ",
        (Kind::Contents, false, false) => "Contents",
        (Kind::Figures, _, true) => "สารบัญภาพ",
        (Kind::Figures, _, false) => "List of Figures",
        (Kind::Tables, _, true) => "สารบัญตาราง",
        (Kind::Tables, _, false) => "List of Tables",
    }
}

fn is_default_title(kind: Kind, t: &str) -> bool {
    t.is_empty() || [(false, false), (false, true), (true, false), (true, true)].iter().any(|(r, th)| default_title(kind, *r, *th) == t)
}

/// A caption label as a field argument: no quotes or backslashes, one short line.
fn clean_label(t: &str) -> String {
    let t: String = t.chars().filter(|c| !matches!(c, '"' | '\\' | ';' | '=') && !c.is_control()).take(64).collect();
    t.trim().to_string()
}

/// Remove body blocks `from..=to`, keeping at least one paragraph in the body.
fn remove_blocks(s: &mut Session, from: usize, to: usize) -> Result<(), CmdError> {
    for k in (from..=to).rev() {
        s.doc.remove_block(StoryRef::Body, &Path::top(k))?;
    }
    if s.doc.body.is_empty() {
        s.doc.insert_block(StoryRef::Body, &Path::top(0), Block::Para(Paragraph::new()))?;
    }
    Ok(())
}

/// Insert, change or remove a list (see [`specs`]).
fn apply(s: &mut Session, v: &Value) -> CmdResult {
    let kind = Kind::from_name(p::req_str(v, "kind")?).ok_or_else(|| CmdError::Params("`kind` is contents, figures or tables".into()))?;
    let cur = list_info(s, kind);
    if p::bool(v, "remove").unwrap_or(false) {
        if let Some(head) = cur.block {
            let end = match kind {
                Kind::Contents => {
                    references::range_toc_to_field(s)?;
                    references::toc_range(s).map_or(head, |r| r.1)
                }
                _ => list_end(s, head),
            };
            remove_blocks(s, head, end)?;
            s.sel = Selection::caret(Pos { story: StoryRef::Body, path: Path::top(head.min(s.doc.body.len().saturating_sub(1))), off: 0 });
            s.clamp_selection();
            s.touch();
        }
        return Ok(list_info(s, kind).to_json());
    }
    if cur.block.is_none() && !p::bool(v, "insert").unwrap_or(true) {
        return Ok(cur.to_json());
    }
    let report = match p::str(v, "layout") {
        Some("report") => true,
        Some("general") => false,
        Some(other) => return Err(CmdError::Params(format!("unknown layout `{other}` (general or report)"))),
        None => cur.report,
    };
    let leader = p::str(v, "leader")
        .map_or(if report != cur.report { if report { TabLeader::None } else { TabLeader::Dot } } else { cur.leader }, references::leader);
    let thai = thai_text(s);
    let keep_title = cur.block.is_some() && !is_default_title(kind, &cur.title);
    let title = if keep_title { cur.title.clone() } else { default_title(kind, report, thai).to_string() };
    match kind {
        Kind::Contents => {
            let levels = p::u64(v, "levels").map_or(cur.levels as u64, |l| l.clamp(1, 9));
            let mut params = json!({"levels": levels, "tabLeader": leader_name(leader), "continued": report});
            if keep_title || cur.block.is_none() {
                params["title"] = json!(title);
            } else {
                params["title"] = json!(default_title(kind, report, thai));
            }
            references::insert_toc(s, &params)?;
        }
        _ => {
            let label = p::str(v, "label").map(clean_label).filter(|l| !l.is_empty()).unwrap_or_else(|| cur.label.clone());
            // The heading speaks the captions' language: สารบัญตาราง over ตารางที่ captions,
            // List of Tables over Table ones (a heading the writer typed stays).
            let title = if keep_title { title } else { default_title(kind, report, is_thai(&label)).to_string() };
            if Kind::of_label(&label) != kind {
                return Err(CmdError::Params(format!("`{label}` captions make a list of {}", Kind::of_label(&label).name())));
            }
            let mode = match p::str(v, "show") {
                Some(m) => LabelMode::from_name(m).ok_or_else(|| CmdError::Params("`show` is full, number or none".into()))?,
                // A new Thai report list shows the numbers under a ตารางที่ / ภาพที่ column.
                None if report && !cur.report => LabelMode::Number,
                None => cur.mode,
            };
            let column = p::bool(v, "columnHeading").unwrap_or(if report && !cur.report { true } else { cur.column_heading });
            citations::set_label_mode(&mut s.doc, &label, mode);
            // Where it goes: in place of the old list, on an empty paragraph taken out after.
            let placeholder = match cur.block {
                Some(head) => {
                    let end = list_end(s, head);
                    remove_blocks(s, head, end)?;
                    s.doc.insert_block(StoryRef::Body, &Path::top(head), Block::Para(Paragraph::new()))?;
                    s.sel = Selection::caret(Pos { story: StoryRef::Body, path: Path::top(head), off: 0 });
                    Some(head)
                }
                None => None,
            };
            let instr = format!("TOC \\c \"{label}\"");
            if report {
                // Keep the page label's word ("หน้า", "Page") when the list had one.
                let page_word = if is_thai(&label) || thai { continued::PAGE_LABEL } else { "Page" };
                let page_label = if column { format!("{label}\t{page_word}") } else { page_word.to_string() };
                citations::report_list(s, &instr, &title, &page_label)?;
            } else {
                citations::generated_list(s, &instr, "")?;
            }
            if let Some(head) = placeholder {
                let after = list_end(s, head) + 1;
                if s.doc.body.get(after).and_then(|b| b.as_para()).is_some_and(|p| p.is_empty() && p.props.style.is_none()) {
                    s.doc.remove_block(StoryRef::Body, &Path::top(after))?;
                }
            }
            // The leader, on every entry: updates keep it from there.
            if let Some((head, end, _)) = caption_list(s, kind) {
                for k in head + 1..=end {
                    if let Ok(p) = s.doc.para_mut(StoryRef::Body, &Path::top(k))
                        && p.props.style.as_deref() == Some(citations::ENTRY_STYLE)
                        && let Some(tabs) = p.props.tabs.as_mut()
                    {
                        for t in tabs.iter_mut().filter(|t| t.align == TabAlign::Right) {
                            t.leader = leader;
                        }
                        p.touch();
                    }
                }
                s.sel = Selection::caret(Pos { story: StoryRef::Body, path: Path::top(head), off: 0 });
            }
            s.clamp_selection();
            s.touch();
        }
    }
    Ok(list_info(s, kind).to_json())
}

/// Update the table of contents and every list of figures and tables.
pub fn update_all(s: &mut Session) -> Result<(), CmdError> {
    references::update_toc(s)?;
    citations::update_generated_where(s, |k| k.starts_with("TOC \\c"))
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("references.list", "Table of Contents Settings", "References › Table of Contents", apply).params(
            r#"{"kind": "contents|figures|tables", "layout"?: "general|report" (report: the Thai academic report layout), "levels"?: 1-9 (contents), "leader"?: "dot|hyphen|underscore|none", "label"?: caption label (figures, tables), "show"?: "full|number|none" (each line: label and number, number only, neither), "columnHeading"?: bool (report layout: the caption label over the numbers), "insert"?: bool (default true: insert at the cursor when the document has none), "remove"?: bool} → the list's settings"#,
        ),
        CommandSpec::new("references.updateLists", "Update All", "References › Table of Contents", |s, _| {
            update_all(s)?;
            sel_result(s)
        }),
        CommandSpec::new("references.lists", "Tables of Contents", "References › Table of Contents", |s, _| {
            Ok(Value::Array(Kind::ALL.into_iter().map(|k| list_info(s, k).to_json()).collect()))
        }),
    ]
}
