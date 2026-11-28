//! The Thai academic report layout of generated lists: the table of contents (สารบัญ), the list
//! of figures (สารบัญภาพ) and the list of tables (สารบัญตาราง), as Thai university report and
//! thesis guides set them out.
//!
//! - The heading is centred and bold, four points larger than body text (20 pt over 16 pt TH
//!   Sarabun), with one blank body-text line under it and then the word "หน้า" ("page") at the
//!   right margin, above the column of page numbers.
//! - Each further page of the list starts with the heading again followed by "(ต่อ)"
//!   ("continued"), the blank line and "หน้า".
//!
//! Word has no such feature, and Thai writers type these lines by hand after a page break. The
//! engine does the same, as real paragraphs inside the list that every Update Table rebuilds
//! from the layout: the continued heading starts its page (page break before), so a list's pages
//! don't move when text before the list changes, and a .docx keeps exactly what is printed (in a
//! TOC they sit inside the field result, so Word's own Update Table writes its plain TOC). The
//! list keeps this layout while it holds a page label paragraph: that is how an update without
//! settings, or a reopened file, knows to keep doing it.

use doc4me_doc::props::{Align, CharProps, NumRef, TabAlign, TabLeader, TabStop, TextColor};
use doc4me_doc::{Block, ParaProps, Paragraph, Path, Pos, StoryRef, Style, StyleKind};

use crate::{CmdError, Session};

/// The styles of a list's continued headings and page labels. They start with "TOC" in a table
/// of contents, so the TOC (and the .docx field around its entries) takes them in.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Styles {
    pub cont: &'static str,
    cont_name: &'static str,
    pub label: &'static str,
    label_name: &'static str,
}

/// A table of contents' styles.
pub(crate) const TOC: Styles = Styles { cont: "TOCContinued", cont_name: "TOC Continued", label: "TOCPageLabel", label_name: "TOC Page Label" };
/// A list of figures' or tables' styles.
pub(crate) const LIST: Styles = Styles { cont: "ListContinued", cont_name: "List Continued", label: "ListPageLabel", label_name: "List Page Label" };

/// The label over the page numbers when none is given.
pub const PAGE_LABEL: &str = "หน้า";

/// A label is a short line of text.
const MAX_LABEL: usize = 64;

/// At most this many continued headings in one list (a list of thousands of pages is not a
/// report's): each costs a layout.
const MAX_PAGES: usize = 400;

/// The `pageLabel` parameter: trimmed, without control characters, at most 64 characters. A
/// tab may separate a column heading at the left margin from the page label (`ตารางที่\tหน้า`:
/// the report guides' lists of figures and tables head their numbers' column too).
pub(crate) fn clean_label(t: &str) -> String {
    let (head, page) = t.rsplit_once('\t').unwrap_or(("", t));
    let clean = |t: &str| {
        let t: String = t.chars().filter(|c| !c.is_control() && *c != doc4me_doc::para::OBJ).take(MAX_LABEL).collect();
        t.trim().to_string()
    };
    let (head, page) = (clean(head), clean(page));
    if head.is_empty() || page.is_empty() { page } else { format!("{head}\t{page}") }
}

/// The column heading at the left of a page label line (`ตารางที่`), if it has one.
pub fn column_heading(label: &str) -> Option<&str> {
    label.split_once('\t').map(|(h, _)| h).filter(|h| !h.is_empty())
}

/// What follows the heading on its further pages: "(ต่อ)" under a Thai page label, else
/// "(continued)".
fn continued_word(label: &str) -> &'static str {
    if label.chars().any(|c| ('\u{0E00}'..='\u{0E7F}').contains(&c)) { "(ต่อ)" } else { "(continued)" }
}

/// True for a paragraph of the continued heading or page label styles `st`.
pub(crate) fn is_part(p: &Paragraph, st: Styles) -> bool {
    p.props.style.as_deref().is_some_and(|x| x == st.cont || x == st.label)
}

/// The page label of the list in body blocks `from..=to`: the text of its first non-empty page
/// label paragraph. `None` when the list doesn't have the report layout.
pub(crate) fn label_in(s: &Session, from: usize, to: usize, st: Styles) -> Option<String> {
    (from..=to).filter_map(|i| s.doc.body.get(i)?.as_para()).find_map(|p| {
        let t = clean_label(&p.plain_text());
        (p.props.style.as_deref() == Some(st.label) && !t.is_empty()).then_some(t)
    })
}

/// Add the continued heading and page label styles `st` if the document doesn't have them.
pub(crate) fn add_styles(s: &mut Session, st: Styles) {
    if s.doc.styles.get(st.cont).is_none() {
        std::sync::Arc::make_mut(&mut s.doc.styles).upsert(Style {
            id: st.cont.into(),
            name: st.cont_name.into(),
            kind: StyleKind::Paragraph,
            based_on: Some("TOCHeading".into()),
            para: ParaProps { page_break_before: Some(true), outline_level: Some(9), ..Default::default() },
            hidden: true,
            ..Default::default()
        });
    }
    if s.doc.styles.get(st.label).is_none() {
        std::sync::Arc::make_mut(&mut s.doc.styles).upsert(Style {
            id: st.label.into(),
            name: st.label_name.into(),
            kind: StyleKind::Paragraph,
            based_on: Some("Normal".into()),
            para: ParaProps {
                align: Some(Align::Right),
                space_before: Some(0.0),
                space_after: Some(0.0),
                keep_next: Some(true),
                ..Default::default()
            },
            chr: CharProps { bold: Some(true), bold_cs: Some(true), ..Default::default() },
            hidden: true,
            ..Default::default()
        });
    }
}

/// The report look of a list heading: centred, bold, four points over body text, in the body
/// text's font, black, unnumbered (a heading style may number chapters), with no space around it
/// and kept with the list. Returns its paragraph and text formatting.
pub(crate) fn heading_look(s: &Session) -> (ParaProps, CharProps) {
    let body = s.doc.styles.resolve_char(Some("Normal"), &CharProps::default());
    let para = ParaProps {
        align: Some(Align::Center),
        space_before: Some(0.0),
        space_after: Some(0.0),
        keep_next: Some(true),
        numbering: Some(NumRef { num: 0, level: 0 }),
        indent_left: Some(0.0),
        indent_first: Some(0.0),
        ..Default::default()
    };
    let size = (body.size + 4.0).clamp(1.0, 1638.0);
    let size_cs = (body.size_cs + 4.0).clamp(1.0, 1638.0);
    let chr = CharProps {
        font: Some(body.font.clone()),
        font_cs: Some(body.font_cs.clone()),
        size: Some(size),
        size_cs: Some(size_cs),
        bold: Some(true),
        bold_cs: Some(true),
        color: Some(TextColor::Auto),
        ..Default::default()
    };
    (para, chr)
}

/// The blank line and the page label that go under a heading (one body-text line each). With a
/// column heading the line runs from the left margin, the page label at a right tab on the
/// margin (`width`, the text width). The label line keeps to the margins whatever its style
/// inherits from Normal: no indents, and the tab stops in `inherited` (the style chain's) cleared,
/// so a stop set in Normal can't catch the tab before the page label.
fn label_paras(label: &str, st: Styles, width: f32, inherited: &[f32]) -> [Paragraph; 2] {
    let mut line = Paragraph::with_text(label, CharProps::default()).styled(st.label);
    line.props.indent_left = Some(0.0);
    line.props.indent_first = Some(0.0);
    line.props.indent_right = Some(0.0);
    if column_heading(label).is_some() {
        let right = (width - 0.5).max(1.0);
        line.props.align = Some(Align::Left);
        let mut tabs: Vec<TabStop> = inherited
            .iter()
            .filter(|p| (*p - right).abs() >= 0.5)
            .take(64)
            .map(|p| TabStop { pos: *p, align: TabAlign::Clear, leader: TabLeader::None })
            .collect();
        tabs.push(TabStop { pos: right, align: TabAlign::Right, leader: TabLeader::None });
        line.props.tabs = Some(tabs);
    }
    [Paragraph::new().styled(st.label), line]
}

/// The text width of the document's current section, for the page label's right tab.
fn text_width(s: &Session) -> f32 {
    super::page::sect(s).text_width()
}

/// The tab stops a page label line gets from its style chain (Normal's included).
fn inherited_tabs(s: &Session, st: Styles) -> Vec<f32> {
    s.doc.styles.resolve_para(&ParaProps { style: Some(st.label.into()), ..Default::default() }).tabs.iter().map(|t| t.pos).collect()
}

/// Put the blank line and page label under the heading at body block `head`. Returns how many
/// blocks went in.
pub(crate) fn insert_label(s: &mut Session, head: usize, label: &str, st: Styles) -> Result<usize, CmdError> {
    let paras = label_paras(label, st, text_width(s), &inherited_tabs(s, st));
    let n = paras.len();
    for (k, p) in paras.into_iter().enumerate() {
        s.doc.insert_block(StoryRef::Body, &Path::top(head + 1 + k), Block::Para(p))?;
    }
    Ok(n)
}

/// The continued heading for the list whose heading is body block `head`: the heading's text
/// and look, then "(ต่อ)", starting a new page.
fn continued_heading(s: &Session, head: usize, label: &str, st: Styles) -> Paragraph {
    let (text, para, chr) = match s.doc.body.get(head).and_then(|b| b.as_para()) {
        Some(h) => {
            let t: String = h.plain_text().chars().filter(|c| !c.is_control() && *c != doc4me_doc::para::OBJ).collect();
            let chr = CharProps { link: None, ..h.props_of_char(0).clone() };
            (t.trim().to_string(), h.props.clone(), chr)
        }
        None => (String::new(), ParaProps::default(), CharProps::default()),
    };
    let word = continued_word(label);
    let text = if text.is_empty() { word.to_string() } else { format!("{text} {word}") };
    let mut p = Paragraph::with_text(&text, chr.clone());
    p.mark = chr;
    p.props = ParaProps { style: Some(st.cont.into()), page_break_before: Some(true), fmt_change: None, num_change: None, ..para };
    p
}

/// The layout page holding `pos`.
fn page_of(l: &doc4me_layout::DocLayout, i: usize, off: usize) -> Option<usize> {
    l.caret(&Pos { story: StoryRef::Body, path: Path::top(i), off }).map(|c| c.page)
}

/// Where the list (heading at body block `head`, its blocks up to `end`) goes on to a new page,
/// start that page with the continued heading, blank line and page label. An entry that would
/// be split over two pages moves whole to the new page; one that is the first under a heading
/// stays where it is (it can't go anywhere better). Returns the list's new last block.
pub(crate) fn paginate(s: &mut Session, head: usize, mut end: usize, label: &str, st: Styles) -> Result<usize, CmdError> {
    for _ in 0..MAX_PAGES {
        s.touch();
        let l = s.layout();
        let Some(mut page) = page_of(&l, head, 0) else { break };
        let mut after_entry = false;
        let mut cut = None;
        for i in head + 1..=end {
            let Some(p) = s.doc.body.get(i).and_then(|b| b.as_para()) else { break };
            match p.props.style.as_deref() {
                Some(x) if x == st.cont => {
                    page = page_of(&l, i, 0).unwrap_or(page);
                    after_entry = false;
                }
                Some(x) if x == st.label => after_entry = false,
                _ => {
                    let last = page_of(&l, i, p.len()).unwrap_or(page);
                    if last > page && after_entry {
                        cut = Some(i);
                        break;
                    }
                    page = page.max(last);
                    after_entry = true;
                }
            }
        }
        let Some(i) = cut else { break };
        let mut paras = vec![continued_heading(s, head, label, st)];
        paras.extend(label_paras(label, st, text_width(s), &inherited_tabs(s, st)));
        for (k, p) in paras.into_iter().enumerate() {
            s.doc.insert_block(StoryRef::Body, &Path::top(i + k), Block::Para(p))?;
            end += 1;
        }
    }
    s.touch();
    Ok(end)
}
