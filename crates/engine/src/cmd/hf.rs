//! Header and footer state that is not already covered by the navigation commands in `tools`.

use doc4me_doc::section::{HeaderSet, SectionProps};
use doc4me_doc::{PartKind, StoryRef};
use doc4me_layout::HfSlot;
use serde_json::{Value, json};

use crate::{CmdError, CmdResult, CommandSpec, Selection, Session, p};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("hf.state", "Header & Footer State", "Header & Footer", |s, _| Ok(state(s)))
            .params(r#"{} → {editing, header, section (0-based), sections, slot: "default|first|even", linked, canLink, differentFirst, differentOddEven, showDocumentText, headerFromTop, footerFromBottom, pageNumberFormat, pageNumberStart}"#)
            .pure(),
        CommandSpec::new("hf.showDocumentText", "Show Document Text", "Header & Footer › Options", |s, v| {
            s.view.hf_document_text = p::bool(v, "value").unwrap_or(!s.view.hf_document_text);
            Ok(json!({"value": s.view.hf_document_text}))
        })
        .pure()
        .params(r#"{"value"?: bool (false hides body text while editing a header or footer; default toggles)}"#),
        CommandSpec::new("hf.turnOnLandscape", "Turn on Landscape Pages", "Header & Footer › Options", |s, v| {
            let on = p::bool(v, "value").unwrap_or(!doc4me_layout::turns_landscape_headers(&s.doc));
            s.doc.set_custom_prop(doc4me_layout::TURN_HF_PROP, if on { "on" } else { "off" });
            s.touch();
            Ok(json!({"value": on}))
        })
        .params(r#"{"value"?: bool} (true: turn headers and footers on landscape pages of a portrait report for binding; default: toggle)"#),
    ]
}

fn get_slot(set: &HeaderSet, slot: HfSlot) -> Option<u32> {
    match slot {
        HfSlot::Default => set.default,
        HfSlot::First => set.first,
        HfSlot::Even => set.even,
    }
}

/// The header/footer story currently being edited, tied to the page that last opened it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HfContext {
    pub page: usize,
    pub section: usize,
    pub header: bool,
    pub slot: HfSlot,
    pub story: u32,
}

pub fn context(s: &mut Session) -> Option<HfContext> {
    let StoryRef::Part(id) = s.sel.focus.story else { return None };
    let header = match s.doc.parts.get(&id)?.kind {
        PartKind::Header => true,
        PartKind::Footer => false,
        _ => return None,
    };
    let layout = s.layout();
    let shows = |page: &doc4me_layout::Page| if header { page.header_story == Some(id) } else { page.footer_story == Some(id) };
    let page = match layout.pages.get(s.page_hint) {
        Some(page) if shows(page) => s.page_hint,
        _ => layout.pages.iter().position(shows)?,
    };
    let page_layout = layout.pages.get(page)?;
    Some(HfContext { page, section: page_layout.section, header, slot: page_layout.hf_slot, story: id })
}

/// The final body block in section `section`, used by `Document::section_mut`.
pub fn section_end(s: &Session, section: usize) -> Option<usize> {
    s.doc.sections().get(section).map(|(end, _)| *end)
}

/// The section of the page whose header or footer is being edited.
pub(crate) fn editing_section_end(s: &mut Session) -> Option<usize> {
    let ctx = context(s)?;
    section_end(s, ctx.section)
}

/// Keep the caret in the same page's header/footer when its displayed slot changes.
pub(crate) fn follow_page(s: &mut Session, before: Option<HfContext>) {
    let Some(before) = before else { return };
    s.touch();
    if context(s).is_some_and(|after| after.page == before.page) {
        return;
    }
    let command = if before.header { "insert.editHeader" } else { "insert.editFooter" };
    let _ = s.run(command, &json!({ "page": before.page }));
}

fn set_of(section: &SectionProps, header: bool) -> &HeaderSet {
    if header { &section.headers } else { &section.footers }
}

fn inherited(s: &Session, section: usize, header: bool, slot: HfSlot) -> Option<u32> {
    s.doc
        .sections()
        .iter()
        .take(section.saturating_add(1))
        .rev()
        .find_map(|(_, sp)| get_slot(set_of(sp, header), slot))
        .filter(|id| s.doc.parts.contains_key(id))
}

pub fn is_linked(s: &Session, section: usize, header: bool, slot: HfSlot) -> bool {
    let Some(previous) = section.checked_sub(1) else { return false };
    let Some((_, sp)) = s.doc.sections().get(section).copied() else { return false };
    match get_slot(set_of(sp, header), slot) {
        None => true,
        Some(id) => inherited(s, previous, header, slot) == Some(id),
    }
}

fn state(s: &mut Session) -> Value {
    let context = context(s);
    let sections = s.doc.sections();
    let (editing, header, section, slot) = match context {
        Some(ctx) => (true, ctx.header, ctx.section, ctx.slot),
        None => {
            let block = s.sel.focus.path.0.first().copied().unwrap_or(0) as usize;
            (false, true, s.doc.section_index_of(block), HfSlot::Default)
        }
    };
    let section_count = sections.len();
    let sp = sections.get(section).map(|(_, sp)| (*sp).clone()).unwrap_or_default();
    json!({
        "editing": editing,
        "header": header,
        "section": section,
        "sections": section_count,
        "slot": match slot { HfSlot::Default => "default", HfSlot::First => "first", HfSlot::Even => "even" },
        "linked": is_linked(s, section, header, slot),
        "canLink": section > 0,
        "differentFirst": sp.title_page,
        "differentOddEven": s.doc.settings.even_odd_headers,
        "showDocumentText": s.view.hf_document_text,
        "turnOnLandscape": doc4me_layout::turns_landscape_headers(&s.doc),
        "headerFromTop": sp.header,
        "footerFromBottom": sp.footer,
        "pageNumberFormat": sp.page_num_format.ooxml(),
        "pageNumberStart": sp.page_num_start,
    })
}

fn slot_mut(set: &mut HeaderSet, slot: HfSlot) -> &mut Option<u32> {
    match slot {
        HfSlot::Default => &mut set.default,
        HfSlot::First => &mut set.first,
        HfSlot::Even => &mut set.even,
    }
}

pub(super) fn link_to_previous(s: &mut Session, v: &Value) -> CmdResult {
    let ctx = context(s);
    let header = match p::bool(v, "footer") {
        Some(footer) => !footer,
        None => ctx.is_none_or(|ctx| ctx.header),
    };
    let (section, slot, page) = match p::u64(v, "section") {
        Some(index) => {
            let index = usize::try_from(index).unwrap_or(usize::MAX);
            if index >= s.doc.sections().len() {
                return Err(CmdError::Params(format!("no section {index}")));
            }
            (index, ctx.map_or(HfSlot::Default, |ctx| ctx.slot), None)
        }
        None => match ctx {
            Some(ctx) => (ctx.section, ctx.slot, Some(ctx.page)),
            None => {
                let block = s.sel.focus.path.0.first().copied().unwrap_or(0) as usize;
                let section = if s.sel.focus.story == StoryRef::Body { s.doc.section_index_of(block) } else { 0 };
                (section, HfSlot::Default, None)
            }
        },
    };
    if section == 0 {
        return Err(CmdError::Disabled("the first section has no previous section to link to".into()));
    }
    let end = section_end(s, section).ok_or_else(|| CmdError::Failed("no such section".into()))?;
    let linked = is_linked(s, section, header, slot);
    let want = p::bool(v, "value").unwrap_or(!linked);
    let editing_this_section = ctx.is_some_and(|ctx| ctx.header == header && ctx.section == section);
    let story = if want {
        // Link to the prior section, dropping this slot's own reference when it has one.
        let own = s.doc.sections().get(section).and_then(|(_, sp)| get_slot(set_of(sp, header), slot));
        if own.is_some() {
            let current = s.doc.section_mut(end);
            *slot_mut(if header { &mut current.headers } else { &mut current.footers }, slot) = None;
            s.touch();
        }
        inherited(s, section, header, slot)
    } else {
        match (linked, inherited(s, section, header, slot)) {
            (false, own) => own,
            (true, shown) => {
                let kind = if header { PartKind::Header } else { PartKind::Footer };
                let style = if header { "Header" } else { "Footer" };
                let blocks = shown
                    .and_then(|id| s.doc.parts.get(&id))
                    .map(|part| part.blocks.clone())
                    .unwrap_or_else(|| vec![doc4me_doc::para_block(doc4me_doc::Paragraph::new().styled(style))]);
                let id = s.doc.add_part(kind, blocks);
                let current = s.doc.section_mut(end);
                *slot_mut(if header { &mut current.headers } else { &mut current.footers }, slot) = Some(id);
                s.touch();
                Some(id)
            }
        }
    };
    if editing_this_section {
        if let Some(page) = page {
            s.page_hint = page;
        }
        match story {
            Some(id) => s.sel = Selection::caret(s.doc.end_of(StoryRef::Part(id))),
            None => {
                if let Some(page) = page {
                    let command = if header { "insert.createHeader" } else { "insert.createFooter" };
                    let _ = s.run(command, &json!({ "page": page }));
                }
            }
        }
    }
    let story = if editing_this_section { context(s).map(|ctx| ctx.story).or(story) } else { story };
    Ok(json!({"linked": want, "section": section, "story": story}))
}

pub(super) fn nav(s: &mut Session, direction: i64) -> CmdResult {
    let ctx = context(s).ok_or_else(|| CmdError::Disabled("edit a header or footer first".into()))?;
    let target = (ctx.section as i64).saturating_add(direction);
    let section_count = s.doc.sections().len() as i64;
    if target < 0 || target >= section_count {
        let message = if direction > 0 { "this is the last section" } else { "this is the first section" };
        return Err(CmdError::Disabled(message.into()));
    }
    let layout = s.layout();
    let page =
        layout.pages.iter().position(|page| page.section as i64 == target).ok_or_else(|| CmdError::Failed("that section has no pages".into()))?;
    let command = if ctx.header { "insert.editHeader" } else { "insert.editFooter" };
    s.run(command, &json!({ "page": page }))
}

/// Header/footer labels and Same as Previous status drawn at the page edge during editing.
pub fn page_labels(s: &Session, layout: &doc4me_layout::DocLayout, page: usize) -> Option<[(String, bool); 2]> {
    let page = layout.pages.get(page)?;
    let many_sections = s.doc.sections().len() > 1;
    let slot_prefix = match page.hf_slot {
        HfSlot::Default => "",
        HfSlot::First => "First Page ",
        HfSlot::Even if s.doc.settings.even_odd_headers => "Even Page ",
        HfSlot::Even => "",
    };
    let odd_prefix = if page.hf_slot == HfSlot::Default && s.doc.settings.even_odd_headers { "Odd Page " } else { "" };
    let label = |kind: &str| {
        let text = format!("{slot_prefix}{odd_prefix}{kind}");
        if many_sections { format!("{text} -Section {}-", page.section + 1) } else { text }
    };
    Some([(label("Header"), is_linked(s, page.section, true, page.hf_slot)), (label("Footer"), is_linked(s, page.section, false, page.hf_slot))])
}
