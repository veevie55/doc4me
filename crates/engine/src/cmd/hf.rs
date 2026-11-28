//! Header and footer state that is not already covered by the navigation commands in `tools`.

use doc4me_doc::section::{HeaderSet, SectionProps};
use doc4me_doc::{PartKind, StoryRef};
use doc4me_layout::HfSlot;
use serde_json::{Value, json};

use crate::{CommandSpec, Session, p};

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

fn linked(s: &Session, section: usize, header: bool, slot: HfSlot) -> bool {
    let Some(previous) = section.checked_sub(1) else { return false };
    let Some((_, sp)) = s.doc.sections().get(section).copied() else { return false };
    match get_slot(set_of(sp, header), slot) {
        None => true,
        Some(id) => inherited(s, previous, header, slot) == Some(id),
    }
}

fn state(s: &mut Session) -> Value {
    let layout = s.layout();
    let sections = s.doc.sections();
    let (editing, header, section, slot) = match s.sel.focus.story {
        StoryRef::Part(id) if matches!(s.doc.parts.get(&id).map(|p| p.kind), Some(PartKind::Header | PartKind::Footer)) => {
            let header = s.doc.parts.get(&id).is_some_and(|p| p.kind == PartKind::Header);
            let page = layout
                .pages
                .get(s.page_hint)
                .filter(|p| if header { p.header_story == Some(id) } else { p.footer_story == Some(id) })
                .or_else(|| layout.pages.iter().find(|p| if header { p.header_story == Some(id) } else { p.footer_story == Some(id) }));
            match page {
                Some(page) => (true, header, page.section, page.hf_slot),
                None => (true, header, 0, HfSlot::Default),
            }
        }
        _ => {
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
        "linked": linked(s, section, header, slot),
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
