//! File tab (Backstage): new, open, save, export, print, properties; document inspection.

use doc4me_doc::{Block, Document, StoryRef};
use serde_json::{Value, json};

use super::{pos_json, sel_result};
use crate::{CmdError, CmdResult, CommandSpec, Session, p};

/// Longest user name kept, in characters; longer names are cut, not refused.
pub const MAX_AUTHOR_CHARS: usize = 255;

/// Sets the user name that new tracked changes and comments are recorded under. The name is
/// trimmed, stripped of control characters (they can't go into the saved XML) and cut to
/// [`MAX_AUTHOR_CHARS`]; a blank name is refused so revisions always carry an author.
pub(crate) fn set_author(s: &mut Session, v: &Value) -> CmdResult {
    let name: String = p::req_str(v, "name")?.trim().chars().filter(|c| !c.is_control()).take(MAX_AUTHOR_CHARS).collect();
    let name = name.trim_end();
    if name.is_empty() {
        return Err(CmdError::Params("`name` must not be blank".into()));
    }
    s.author = name.to_string();
    let mut r = sel_result(s)?;
    if let Some(o) = r.as_object_mut() {
        o.insert("name".into(), json!(s.author));
    }
    Ok(r)
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("file.new", "New", "File", new).key("Mod+N").params(r#"{"template"?: "blank|sample|letter|resume|report"}"#).pure(),
        CommandSpec::new("file.open", "Open", "File", open)
            .key("Mod+O / Mod+F12 / Mod+Alt+F2")
            .params(r#"{"path": string, "data"?: base64 (the file's bytes; `path` then only names it), "password"?: string (for a password-protected Word document)}"#)
            .pure(),
        CommandSpec::new("file.letterWizard", "Letter Wizard", "File › New", letter_wizard)
            .params(r#"{"senderName"?, "returnAddress"?, "date"?, "mailingInstructions"?, "recipientName"?, "recipientAddress"?, "attention"?, "subject"?, "salutation"?, "body"?, "closing"?, "senderJobTitle"?, "senderCompany"?, "senderInitials"?, "enclosures"?, "cc"?: string (lines separated by \n)} → a new letter from these; none of them opens the Letter Wizard"#)
            .pure(),
        CommandSpec::new("file.showRepairs", "Show Repairs", "File", show_repairs)
            .params(r#"{} → {"repairs": [{"message": string with {what}, "what": string}]}: what opening the document had to repair or leave out (opening a damaged file shows them)"#)
            .pure(),
        CommandSpec::new("file.save", "Save", "File", save)
            .key("Mod+S / Shift+F12 / Alt+Shift+F2")
            .params(r#"{"path"?: string, "password"?: string (encrypt this and later saves with it, as file.encrypt does), "markup"?: bool (Print Markup for a PDF, PNG, HTML, Markdown or text copy: comments shown; default file.printMarkup)}"#)
            .pure(),
        CommandSpec::new("file.saveAs", "Save As", "File", save_as).key("F12").params(r#"{"path": string, "password"?: string, "markup"?: bool (as file.save)}"#).pure(),
        // Pure: the password must not land in Repeat or a recorded macro, and it isn't an edit
        // to undo; it marks the document as changed itself.
        CommandSpec::new("file.encrypt", "Encrypt with Password", "File › Info › Protect Document", encrypt)
            .params(r#"{"password": string (Word documents are saved encrypted with it) | null or "" (saved unencrypted)}"#)
            .pure(),
        CommandSpec::new("file.exportPdf", "Export PDF", "File › Export", |s, v| {
            let path = p::req_str(v, "path")?;
            let path = if path.to_ascii_lowercase().ends_with(".pdf") { path.to_string() } else { format!("{path}.pdf") };
            let opts = export_options(s, v);
            crate::io::save_path_with(std::path::Path::new(&path), &s.doc, None, &opts).map_err(CmdError::Failed)?;
            Ok(json!({"path": path, "markup": opts.markup}))
        })
        .params(r#"{"path": string, "markup"?: bool (Print Markup: tracked changes marked up and each page scaled down beside the comments in balloons; default file.printMarkup)}"#)
        .pure(),
        CommandSpec::new("file.exportPng", "Export Page as PNG", "File › Export", export_png)
            .params(r#"{"path": string, "page"?: n (1-based), "scale"?: px per pt, "markup"?: bool (the page with its comments in balloons beside it; default file.printMarkup)}"#)
            .pure(),
        CommandSpec::new("file.exportJpeg", "Export Page as JPEG", "File › Export", export_jpeg)
            .params(
                r#"{"path": string (.jpg is added unless it ends in .jpg or .jpeg), "page"?: n (1-based), "scale"?: px per pt, "quality"?: 1-100 (default 90), "markup"?: bool (the page with its comments in balloons beside it; default file.printMarkup)}"#,
            )
            .pure(),
        // A print setting, like Word's: not part of the document, so no undo step.
        CommandSpec::new("file.printMarkup", "Print Markup", "File › Print", |s, v| {
            s.prefs.print_markup = p::bool(v, "value").unwrap_or(!s.prefs.print_markup);
            Ok(json!({"value": s.prefs.print_markup}))
        })
        .params(r#"{"value"?: bool (omit to toggle)} — PDFs, prints and copies show comments (balloons beside each page; a list at the end of HTML, Markdown and text) and tracked changes"#)
        .pure(),
        CommandSpec::new("file.printPdf", "Print to PDF", "File › Print", crate::print::print_pdf_to_file)
            .params(
                r#"{"path": string, "range"?: "all" | "current" | "selection" (the pages the selection touches) | "custom", "pages"?: "1-3,5" (implies custom), "parity"?: "all" | "odd" | "even", "copies"?: 1-999, "collate"?: bool (default true)}"#,
            )
            .pure(),
        CommandSpec::new("file.print", "Print", "File", |s, _| {
            s.push_ui_request(json!({"open": "print"}));
            sel_result(s)
        })
        .key("Mod+P / Mod+Shift+F12 / Mod+F2 / Mod+Alt+I")
        .pure(),
        // Options › Advanced › Compatibility options: the layout options of this document that
        // Doc4Me's layout follows.
        CommandSpec::new("file.compatibilityOptions", "Compatibility Options", "File › Options › Advanced", |s, v| {
            let mut changed = false;
            for (k, f) in [("doNotUseHtmlParagraphAutoSpacing", 0), ("splitPageBreakAndParagraphMark", 1)] {
                if let Some(b) = p::bool(v, k) {
                    let x = if f == 0 { &mut s.doc.settings.add_para_spacing } else { &mut s.doc.settings.split_page_break_mark };
                    changed |= *x != b;
                    *x = b;
                }
            }
            if changed {
                s.relayout();
            }
            Ok(json!({
                "mode": s.doc.settings.compat_mode,
                "doNotUseHtmlParagraphAutoSpacing": s.doc.settings.add_para_spacing,
                "splitPageBreakAndParagraphMark": s.doc.settings.split_page_break_mark,
            }))
        })
        .params(r#"{"doNotUseHtmlParagraphAutoSpacing"?: bool (space after and before add up instead of the larger winning), "splitPageBreakAndParagraphMark"?: bool (a page break ending a paragraph takes its mark to the next page)} → the options and the compatibility mode (15: Word 2013 and later)"#),
        // Options › Advanced › Font Substitution: the fonts the document names that aren't
        // installed, each with the font shown instead; `convert` puts that font in their place.
        CommandSpec::new("file.fontSubstitution", "Font Substitution", "File › Options › Advanced", font_substitution)
            .params(r#"{"convert"?: [font name] (replace these missing fonts with their substitutes, Convert Permanently)} → {"fonts": [{"font", "substitute"}], "converted"}"#),
        // F1: open Doc4Me's offline command help.
        CommandSpec::new("help.show", "Help", "Help", |s, _| {
            s.push_ui_request(json!({"open": "wordPerfectHelp"}));
            sel_result(s)
        })
        .key("F1")
        .pure(),
        CommandSpec::new("file.close", "Close", "File", |s, _| {
            s.push_ui_request(json!({"close": true}));
            sel_result(s)
        })
        .key("Mod+W / Mod+F4")
        .pure(),
        // Alt+F4: Word's Exit. Doc4Me has one document per window, so it closes it, asking first
        // about unsaved changes, as Close does.
        CommandSpec::new("file.exit", "Exit", "File", |s, _| {
            s.push_ui_request(json!({"close": true}));
            sel_result(s)
        })
        .key(super::non_mac("Alt+F4"))
        .pure(),
        CommandSpec::new("file.properties", "Properties", "File › Info", properties)
            .params(r#"{"title"?, "subject"?, "author"?, "keywords"?, "comments"?, "category"?, "custom"?: {name: string | null (removes)}}"#),
        CommandSpec::new("file.info", "Info", "File", info).pure(),
        CommandSpec::new("file.setAuthor", "User Name", "File › Options › General", set_author)
            .params(r#"{"name": string (author of new tracked changes and comments)}"#)
            .pure(),
        CommandSpec::new("document.inspect", "Inspect Document", "Agents", inspect).params(r#"{"text"?: bool}"#).pure(),
        CommandSpec::new("document.text", "Document Text", "Agents", |s, v| {
            let story = super::story_param(s, v);
            Ok(json!({"text": s.doc.plain_text(story)}))
        })
        .pure(),
        CommandSpec::new("document.paragraph", "Paragraph Details", "Agents", paragraph).params(r#"{"path": [n], "story"?: Story}"#).pure(),
        CommandSpec::new("document.selection", "Selection", "Agents", |s, _| {
            let mut r = sel_result(s)?;
            if let Some(o) = r.as_object_mut() {
                o.insert("text".into(), json!(s.selected_text()));
                o.insert("format".into(), super::format::state(s));
            }
            Ok(r)
        })
        .pure(),
        CommandSpec::new("document.layout", "Layout Summary", "Agents", layout_summary).pure(),
        CommandSpec::new("document.setText", "Replace Document Text", "Agents", |s, v| {
            let text = p::req_str(v, "text")?;
            let d = Document::from_text(text);
            s.doc.body = d.body;
            s.sel = crate::Selection::caret(s.doc.start_of(StoryRef::Body));
            sel_result(s)
        })
        .params(r#"{"text": string}"#),
    ]
}

fn new(s: &mut Session, v: &Value) -> CmdResult {
    let doc = match p::str(v, "template").unwrap_or("blank") {
        "sample" => crate::sample::sample_document(),
        "letter" => crate::sample::letter(),
        "resume" => crate::sample::resume(),
        "report" => crate::sample::report(),
        _ => {
            // A blank document is written in the editing language (the templates' text is English).
            let mut d = Document::new();
            if let Some(lang) = &s.editing_lang {
                std::sync::Arc::make_mut(&mut d.styles).default_chr.lang = Some(lang.clone());
            }
            // Options › Advanced › Bidirectional text › Default direction: right to left.
            if s.prefs.rtl_default {
                std::sync::Arc::make_mut(&mut d.styles).default_para.bidi = Some(true);
            }
            d
        }
    };
    s.set_document(doc);
    s.path = None;
    sel_result(s)
}

fn open(s: &mut Session, v: &Value) -> CmdResult {
    let Some(path) = p::str(v, "path") else {
        s.push_ui_request(json!({"open": "openFile"}));
        return sel_result(s);
    };
    let password = p::str(v, "password");
    let (mut doc, encrypted) = if let Some(data) = p::str(v, "data") {
        let bytes = super::insert::base64_decode(data).ok_or_else(|| CmdError::Params("bad base64".into()))?;
        crate::io::open_bytes_with(path, &bytes, password).map_err(CmdError::Failed)?
    } else {
        crate::io::open_path_with(std::path::Path::new(path), password).map_err(CmdError::Failed)?
    };
    // Plain text and Markdown carry no language: they are in the editing language.
    let ext = std::path::Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    if let Some(lang) = &s.editing_lang
        && matches!(ext.as_str(), "txt" | "text" | "md" | "markdown")
    {
        std::sync::Arc::make_mut(&mut doc.styles).default_chr.lang = Some(lang.clone());
    }
    s.set_document(doc);
    s.path = Some(path.into());
    // Like Word, a document opened with a password is saved with it again.
    if encrypted {
        s.password = password.map(crate::io::Password::new);
    }
    let substituted = unmatched_fonts(&s.doc);
    if !substituted.is_empty() {
        s.push_ui_request(json!({"fontsSubstituted": substituted, "with": "Sarabun"}));
    }
    // A damaged file opened with parts left out: Word's Show Repairs says what.
    if !s.doc.repairs.is_empty() {
        s.push_ui_request(json!({"open": "showRepairs"}));
    }
    Ok(json!({
        "path": path,
        "paragraphs": s.doc.paragraph_count(),
        "words": s.doc.word_count(),
        "encrypted": encrypted,
        "substitutedFonts": substituted,
        "repairs": s.doc.repairs.len(),
    }))
}

/// The Letter Wizard's fields by parameter name.
const LETTER_FIELDS: [&str; 16] = [
    "senderName",
    "returnAddress",
    "date",
    "mailingInstructions",
    "recipientName",
    "recipientAddress",
    "attention",
    "subject",
    "salutation",
    "body",
    "closing",
    "senderJobTitle",
    "senderCompany",
    "senderInitials",
    "enclosures",
    "cc",
];
/// Longest letter field, in characters (the body may be longer).
const MAX_LETTER_FIELD: usize = 2_000;
const MAX_LETTER_BODY: usize = 100_000;

/// Letter Wizard: a new letter from the fields given (as File › New › Letter does from the
/// template), replacing the document like File › New.
fn letter_wizard(s: &mut Session, v: &Value) -> CmdResult {
    let mut got = std::collections::HashMap::new();
    for k in LETTER_FIELDS {
        match v.get(k) {
            None | Some(Value::Null) => {}
            Some(Value::String(t)) => {
                let max = if k == "body" { MAX_LETTER_BODY } else { MAX_LETTER_FIELD };
                if t.chars().count() > max {
                    return Err(CmdError::Params(format!("`{k}` is at most {max} characters")));
                }
                // Control characters other than line breaks have no place in a letter.
                let clean: String = t.replace("\r\n", "\n").chars().filter(|c| *c == '\n' || !c.is_control()).collect();
                got.insert(k, clean);
            }
            Some(_) => return Err(CmdError::Params(format!("`{k}` must be text"))),
        }
    }
    if got.is_empty() {
        s.push_ui_request(json!({"open": "letterWizard"}));
        return Ok(json!({"opened": true}));
    }
    let mut take = |k: &str| got.remove(k).unwrap_or_default();
    let f = crate::sample::LetterFields {
        sender_name: take("senderName"),
        return_address: take("returnAddress"),
        date: take("date"),
        mailing_instructions: take("mailingInstructions"),
        recipient_name: take("recipientName"),
        recipient_address: take("recipientAddress"),
        attention: take("attention"),
        subject: take("subject"),
        salutation: take("salutation"),
        body: take("body"),
        closing: take("closing"),
        sender_title: take("senderJobTitle"),
        sender_company: take("senderCompany"),
        initials: take("senderInitials"),
        enclosures: take("enclosures"),
        cc: take("cc"),
    };
    let mut doc = crate::sample::letter_from(&f);
    if let Some(lang) = &s.editing_lang {
        std::sync::Arc::make_mut(&mut doc.styles).default_chr.lang = Some(lang.clone());
    }
    s.set_document(doc);
    s.path = None;
    let mut r = sel_result(s)?;
    if let Some(o) = r.as_object_mut() {
        o.insert("opened".into(), json!(false));
    }
    Ok(r)
}

/// Show Repairs: what reading the document repaired or left out.
fn show_repairs(s: &mut Session, _: &Value) -> CmdResult {
    Ok(json!({"repairs": s.doc.repairs}))
}

/// Fonts Doc4Me has no metrics for that a document may use; when one isn't installed the bundled
/// Sarabun draws it at its own size, so lines and pages can break differently, and the user is
/// told when the document opens. Compared without spaces or case.
const UNMATCHED_FONTS: &[&str] = &["thsarabunit๙", "thsarabunit9"];

/// The [`UNMATCHED_FONTS`] the document uses (in its styles or its text) that aren't installed,
/// each once, spelled as the document first spells it.
pub fn unmatched_fonts(doc: &Document) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut note = |name: &Option<String>| {
        let Some(name) = name.as_deref() else { return };
        let key: String = name.chars().filter(|c| !c.is_whitespace()).flat_map(char::to_lowercase).collect();
        if UNMATCHED_FONTS.contains(&key.as_str())
            && !found.iter().any(|f| f.chars().filter(|c| !c.is_whitespace()).flat_map(char::to_lowercase).eq(key.chars()))
            && doc4me_fonts::word::resolve(name, false, false).substituted
        {
            found.push(name.trim().to_string());
        }
    };
    let mut chr = |c: &doc4me_doc::props::CharProps| {
        note(&c.font);
        note(&c.font_cs);
        note(&c.font_ea);
    };
    chr(&doc.styles.default_chr);
    for st in &doc.styles.styles {
        chr(&st.chr);
    }
    let stories = std::iter::once(StoryRef::Body).chain(doc.parts.keys().map(|id| StoryRef::Part(*id)));
    for story in stories {
        for path in doc.para_paths(story) {
            if let Some(para) = doc.para(story, &path) {
                for run in &para.runs {
                    chr(&run.props);
                }
            }
        }
    }
    found
}

/// Normalized document font identifier (case and spaces aside).
fn font_key(n: &str) -> String {
    n.chars().filter(|c| !c.is_whitespace()).flat_map(char::to_lowercase).collect()
}

fn note_document_font(found: &mut Vec<String>, name: Option<&str>) {
    let Some(name) = name.map(str::trim).filter(|n| !n.is_empty() && !n.starts_with('+')) else { return };
    let key = font_key(name);
    if !found.iter().any(|f| font_key(f) == key) {
        found.push(name.to_string());
    }
}

fn graphic_fonts(graphic: &doc4me_doc::graphic::Graphic, found: &mut Vec<String>) {
    for item in &graphic.items {
        if let doc4me_doc::graphic::GraphicItem::Text { font, .. } = item {
            note_document_font(found, font.as_deref());
        }
    }
}

fn math_fonts(nodes: &[doc4me_doc::math::MNode], found: &mut Vec<String>) {
    use doc4me_doc::math::MNode as N;
    for node in nodes {
        match node {
            N::Run(run) => {
                note_document_font(found, run.font.as_deref());
            }
            N::Frac { num, den, .. } => {
                math_fonts(num, found);
                math_fonts(den, found);
            }
            N::Script { base, sub, sup, .. } => {
                math_fonts(base, found);
                math_fonts(sub, found);
                math_fonts(sup, found);
            }
            N::Rad { deg, e, .. } => {
                math_fonts(deg, found);
                math_fonts(e, found);
            }
            N::Nary { sub, sup, e, .. } => {
                math_fonts(sub, found);
                math_fonts(sup, found);
                math_fonts(e, found);
            }
            N::Delim { elems, .. } => {
                for elem in elems {
                    math_fonts(elem, found);
                }
            }
            N::Func { name, e } => {
                math_fonts(name, found);
                math_fonts(e, found);
            }
            N::Lim { e, lim, .. } => {
                math_fonts(e, found);
                math_fonts(lim, found);
            }
            N::Acc { e, .. } | N::Bar { e, .. } | N::BorderBox { e, .. } | N::Boxed { e } | N::GroupChr { e, .. } | N::Phant { e, .. } => {
                math_fonts(e, found)
            }
            N::EqArr { rows } => {
                for row in rows {
                    math_fonts(row, found);
                }
            }
            N::Matrix { rows, .. } => {
                for row in rows {
                    for cell in row {
                        math_fonts(cell, found);
                    }
                }
            }
        }
    }
}

fn object_fonts(object: &doc4me_doc::InlineObject, found: &mut Vec<String>) {
    match object {
        doc4me_doc::InlineObject::Graphic { graphic, .. } => graphic_fonts(graphic, found),
        doc4me_doc::InlineObject::Equation { math, .. } => math_fonts(&math.nodes, found),
        doc4me_doc::InlineObject::Group { children, .. } => {
            for child in children {
                object_fonts(&child.obj, found);
            }
        }
        _ => {}
    }
}

/// Every font name used by the theme, styles, text runs, graphic labels and math runs in all
/// stories, including paragraph marks: the first spelling of each (case and spaces aside).
fn document_fonts(doc: &Document) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    note_document_font(&mut found, Some(&doc.settings.major_font));
    note_document_font(&mut found, Some(&doc.settings.minor_font));
    let chr = |c: &doc4me_doc::props::CharProps, found: &mut Vec<String>| {
        for f in [&c.font, &c.font_cs, &c.font_ea] {
            note_document_font(found, f.as_deref());
        }
    };
    chr(&doc.styles.default_chr, &mut found);
    for st in &doc.styles.styles {
        chr(&st.chr, &mut found);
    }
    let stories = std::iter::once(StoryRef::Body).chain(doc.parts.keys().map(|id| StoryRef::Part(*id)));
    for story in stories {
        for path in doc.para_paths(story) {
            if let Some(para) = doc.para(story, &path) {
                chr(&para.mark, &mut found);
                for run in &para.runs {
                    chr(&run.props, &mut found);
                }
                for object in &para.objects {
                    object_fonts(object, &mut found);
                }
            }
        }
    }
    found
}

fn fix_font(name: &mut Option<String>, swap: &[(String, String)], converted: &mut usize) {
    if let Some(sub) = name.as_deref().and_then(|x| swap.iter().find(|(k, _)| *k == font_key(x))).map(|(_, sub)| sub.clone()) {
        *name = Some(sub);
        *converted += 1;
    }
}

fn fix_char_props(c: &mut doc4me_doc::props::CharProps, fix: &mut dyn FnMut(&mut Option<String>, &mut usize), converted: &mut usize) {
    fix(&mut c.font, converted);
    fix(&mut c.font_cs, converted);
    fix(&mut c.font_ea, converted);
}

fn convert_math_fonts(nodes: &mut [doc4me_doc::math::MNode], fix: &mut dyn FnMut(&mut Option<String>, &mut usize), converted: &mut usize) {
    use doc4me_doc::math::MNode as N;
    for node in nodes {
        match node {
            N::Run(run) => fix(&mut run.font, converted),
            N::Frac { num, den, .. } => {
                convert_math_fonts(num, fix, converted);
                convert_math_fonts(den, fix, converted);
            }
            N::Script { base, sub, sup, .. } => {
                convert_math_fonts(base, fix, converted);
                convert_math_fonts(sub, fix, converted);
                convert_math_fonts(sup, fix, converted);
            }
            N::Rad { deg, e, .. } => {
                convert_math_fonts(deg, fix, converted);
                convert_math_fonts(e, fix, converted);
            }
            N::Nary { sub, sup, e, .. } => {
                convert_math_fonts(sub, fix, converted);
                convert_math_fonts(sup, fix, converted);
                convert_math_fonts(e, fix, converted);
            }
            N::Delim { elems, .. } => {
                for elem in elems {
                    convert_math_fonts(elem, fix, converted);
                }
            }
            N::Func { name, e } => {
                convert_math_fonts(name, fix, converted);
                convert_math_fonts(e, fix, converted);
            }
            N::Lim { e, lim, .. } => {
                convert_math_fonts(e, fix, converted);
                convert_math_fonts(lim, fix, converted);
            }
            N::Acc { e, .. } | N::Bar { e, .. } | N::BorderBox { e, .. } | N::Boxed { e } | N::GroupChr { e, .. } | N::Phant { e, .. } => {
                convert_math_fonts(e, fix, converted)
            }
            N::EqArr { rows } => {
                for row in rows {
                    convert_math_fonts(row, fix, converted);
                }
            }
            N::Matrix { rows, .. } => {
                for row in rows {
                    for cell in row {
                        convert_math_fonts(cell, fix, converted);
                    }
                }
            }
        }
    }
}

fn convert_object_fonts(object: &mut doc4me_doc::InlineObject, fix: &mut dyn FnMut(&mut Option<String>, &mut usize), converted: &mut usize) {
    match object {
        doc4me_doc::InlineObject::Graphic { .. } => {
            // Chart/SmartArt package markup is preserved opaquely and the DOCX writer doesn't
            // serialize edits to GraphicItem text. Keep these fonts visible as unresolved rather
            // than claiming a permanent conversion that saving would discard.
        }
        doc4me_doc::InlineObject::Equation { math, .. } => {
            let before = *converted;
            convert_math_fonts(&mut math.nodes, fix, converted);
            if *converted != before {
                // Other equation edits invalidate retained OMML for the same reason: the writer
                // must serialize the edited model instead of copying stale source XML.
                math.omml.clear();
            }
        }
        doc4me_doc::InlineObject::Group { children, .. } => {
            for child in children {
                convert_object_fonts(&mut child.obj, fix, converted);
            }
        }
        _ => {}
    }
}

fn font_substitution_rows(doc: &Document) -> Vec<(String, String)> {
    document_fonts(doc)
        .into_iter()
        .filter_map(|font| {
            let fallback = doc4me_fonts::word::substitute_for(&font)?;
            let key = font.trim().to_lowercase();
            let substitute = doc
                .font_aliases
                .get(&key)
                .filter(|alias| !alias.alternate.trim().is_empty())
                .map(|alias| doc4me_fonts::word::substitute_for(&alias.alternate).unwrap_or_else(|| alias.alternate.clone()))
                .unwrap_or(fallback);
            Some((font, substitute))
        })
        .collect()
}

/// `file.fontSubstitution`.
fn font_substitution(s: &mut Session, v: &Value) -> CmdResult {
    let fonts = font_substitution_rows(&s.doc);
    let wanted: Vec<String> = v.get("convert").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).map(font_key).collect();
    let swap: Vec<(String, String)> =
        fonts.iter().filter(|(f, _)| wanted.contains(&font_key(f))).map(|(f, sub)| (font_key(f), sub.clone())).collect();
    let mut converted = 0;
    if !swap.is_empty() {
        let mut fix = |name: &mut Option<String>, n: &mut usize| fix_font(name, &swap, n);
        for f in [&mut s.doc.settings.major_font, &mut s.doc.settings.minor_font] {
            let mut o = Some(f.clone());
            fix(&mut o, &mut converted);
            *f = o.unwrap_or_default();
        }
        fix_char_props(&mut std::sync::Arc::make_mut(&mut s.doc.styles).default_chr, &mut fix, &mut converted);
        for st in &mut std::sync::Arc::make_mut(&mut s.doc.styles).styles {
            fix_char_props(&mut st.chr, &mut fix, &mut converted);
        }
        let stories: Vec<StoryRef> = std::iter::once(StoryRef::Body).chain(s.doc.parts.keys().map(|id| StoryRef::Part(*id))).collect();
        for story in stories {
            for path in s.doc.para_paths(story) {
                let para = s.doc.para_mut(story, &path)?;
                fix_char_props(&mut para.mark, &mut fix, &mut converted);
                for run in &mut para.runs {
                    fix_char_props(&mut run.props, &mut fix, &mut converted);
                }
                for object in &mut para.objects {
                    convert_object_fonts(object, &mut fix, &mut converted);
                }
                para.touch();
            }
        }
        s.relayout();
    }
    let left: Vec<Value> = font_substitution_rows(&s.doc).into_iter().map(|(f, sub)| json!({"font": f, "substitute": sub})).collect();
    Ok(json!({"fonts": left, "converted": converted}))
}

/// File › Info › Protect Document › Encrypt with Password: the password Word documents are saved
/// with from now on, or none.
fn encrypt(s: &mut Session, v: &Value) -> CmdResult {
    let password = match v.get("password") {
        Some(Value::Null) => None,
        Some(Value::String(pw)) if pw.is_empty() => None,
        Some(Value::String(pw)) => {
            doc4me_docx::check_password(pw).map_err(|e| CmdError::Params(e.to_string()))?;
            Some(crate::io::Password::new(pw))
        }
        _ => return Err(CmdError::Params("`password` must be a string, or null to remove the password".into())),
    };
    if password != s.password {
        s.password = password;
        s.dirty = true;
    }
    Ok(json!({"encrypted": s.password.is_some()}))
}

fn save(s: &mut Session, v: &Value) -> CmdResult {
    let path = match p::str(v, "path").map(std::path::PathBuf::from).or_else(|| s.path.clone()) {
        Some(p) => p,
        None => {
            s.push_ui_request(json!({"open": "saveAs"}));
            return Ok(json!({"saved": false}));
        }
    };
    if let Some(pw) = v.get("password").filter(|pw| !pw.is_null()) {
        let Some(pw) = pw.as_str().filter(|pw| !pw.is_empty()) else {
            return Err(CmdError::Params("`password` must be a non-empty string (file.encrypt removes a password)".into()));
        };
        if !crate::io::can_encrypt(&path.to_string_lossy()) {
            return Err(CmdError::Params("only Word documents (.docx, .docm, .dotx, .dotm) can be saved with a password".into()));
        }
        doc4me_docx::check_password(pw).map_err(|e| CmdError::Params(e.to_string()))?;
        s.password = Some(crate::io::Password::new(pw));
    }
    s.stamp_save();
    let password = s.password.as_ref().map(crate::io::Password::as_str);
    crate::io::save_path_with(&path, &s.doc_to_save(), password, &export_options(s, v)).map_err(CmdError::Failed)?;
    let encrypted = password.is_some() && crate::io::can_encrypt(&path.to_string_lossy());
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    if ["docx", "docm", "dotx", "dotm", "odt", "rtf", "json", "xml"].contains(&ext.as_str()) {
        s.path = Some(path.clone());
        s.dirty = false;
    }
    Ok(json!({"saved": true, "path": path.to_string_lossy(), "encrypted": encrypted}))
}

fn save_as(s: &mut Session, v: &Value) -> CmdResult {
    if p::str(v, "path").is_none() {
        s.push_ui_request(json!({"open": "saveAs"}));
        return Ok(json!({"saved": false}));
    }
    save(s, v)
}

/// How a copy is written: Print Markup from the command's `markup`, else the File › Print
/// setting (`file.printMarkup`); comments as Track Changes Options show them.
pub fn export_options(s: &Session, v: &Value) -> crate::io::ExportOptions {
    let markup = p::bool(v, "markup").unwrap_or(s.prefs.print_markup);
    crate::io::ExportOptions { markup, comments: markup && s.prefs.markup.comments }
}

fn export_png(s: &mut Session, v: &Value) -> CmdResult {
    // Only `.png` paths, so automation can't replace other files with page images
    // (WordCraft #3696).
    if !p::req_str(v, "path")?.to_ascii_lowercase().ends_with(".png") {
        return Err(CmdError::Params("path must end in .png".into()));
    }
    export_page(s, v, None)
}

fn export_jpeg(s: &mut Session, v: &Value) -> CmdResult {
    let quality = p::u64(v, "quality").unwrap_or(u64::from(crate::io_ext::JPEG_QUALITY)).clamp(1, 100) as u8;
    export_page(s, v, Some(quality))
}

/// Write one page as a picture: PNG, or JPEG at quality `jpeg` (on white, since JPEG has no
/// transparency).
fn export_page(s: &mut Session, v: &Value, jpeg: Option<u8>) -> CmdResult {
    let path = p::req_str(v, "path")?;
    let lower = path.to_ascii_lowercase();
    let path = if jpeg.is_some() && !lower.ends_with(".jpg") && !lower.ends_with(".jpeg") { format!("{path}.jpg") } else { path.to_string() };
    let page = crate::p::page_index(p::u64(v, "page").unwrap_or(1));
    let scale = p::f32(v, "scale").unwrap_or(2.0).clamp(0.1, 8.0);
    let l = s.export_layout_with(export_options(s, v).markup);
    let pg = l.pages.get(page).ok_or_else(|| CmdError::Params(format!("no page {}", page + 1)))?;
    let img = doc4me_render::render_page(&s.doc, pg, scale, &Default::default());
    let (bytes, format) = match jpeg {
        Some(q) => (img.to_jpeg(q), "JPEG"),
        None => (img.to_png(), "PNG"),
    };
    if bytes.is_empty() {
        return Err(CmdError::Failed(format!("{path}: the page image ({}×{} px) couldn't be encoded as {format}", img.width, img.height)));
    }
    crate::io::write_file(&path, &bytes).map_err(CmdError::Failed)?;
    Ok(json!({"path": path, "width": img.width, "height": img.height, "bytes": bytes.len()}))
}

fn properties(s: &mut Session, v: &Value) -> CmdResult {
    let c = &mut s.doc.core;
    let set = |dst: &mut String, k: &str| {
        if let Some(x) = p::str(v, k) {
            *dst = x.to_string();
        }
    };
    set(&mut c.title, "title");
    set(&mut c.subject, "subject");
    set(&mut c.creator, "author");
    set(&mut c.keywords, "keywords");
    set(&mut c.description, "comments");
    set(&mut c.category, "category");
    if let Some(custom) = v.get("custom") {
        let Some(m) = custom.as_object() else {
            return Err(CmdError::Params("`custom` must be an object of name → string or null".into()));
        };
        for (name, val) in m {
            if name.trim().is_empty() || name.chars().count() > MAX_CUSTOM_NAME {
                return Err(CmdError::Params(format!("custom property names are 1–{MAX_CUSTOM_NAME} characters")));
            }
            match val {
                Value::Null => {
                    s.doc.remove_custom_prop(name);
                }
                Value::String(x) if x.len() <= MAX_CUSTOM_VALUE => s.doc.set_custom_prop(name, x),
                _ => return Err(CmdError::Params(format!("custom property `{name}` must be a string (≤ {MAX_CUSTOM_VALUE} bytes) or null"))),
            }
        }
    }
    let mut out = serde_json::to_value(&s.doc.core).unwrap_or(Value::Null);
    if let Some(o) = out.as_object_mut() {
        o.insert("custom".into(), serde_json::to_value(&s.doc.custom_props).unwrap_or(Value::Null));
    }
    Ok(out)
}

/// Longest custom property name (Word's limit).
const MAX_CUSTOM_NAME: usize = 255;
/// Largest custom property value we accept from a command.
const MAX_CUSTOM_VALUE: usize = 1024 * 1024;

fn info(s: &mut Session, _: &Value) -> CmdResult {
    let l = s.layout();
    Ok(json!({
        "path": s.path.as_ref().map(|p| p.to_string_lossy().to_string()),
        "dirty": s.dirty,
        "pages": l.pages.len(),
        "words": s.doc.word_count(),
        "paragraphs": s.doc.paragraph_count(),
        "properties": s.doc.core,
        "trackChanges": s.doc.settings.track_changes,
        "comments": s.doc.comments.len(),
        "sections": s.doc.sections().len(),
        "encrypted": s.password.is_some(),
        "layoutMs": l.ms,
    }))
}

/// Structure for agents: blocks with text and formatting summary, selection, stats.
fn inspect(s: &mut Session, v: &Value) -> CmdResult {
    let with_text = p::bool(v, "text").unwrap_or(true);
    let mut blocks = Vec::new();
    for (i, b) in s.doc.body.iter().enumerate().take(2000) {
        match &**b {
            Block::Para(p) => {
                let style = p.props.style.clone().unwrap_or_else(|| "Normal".into());
                let mut o = json!({"index": i, "type": "paragraph", "style": style, "len": p.len()});
                if with_text {
                    o["text"] = json!(p.plain_text());
                }
                if let Some(n) = p.props.numbering.filter(|n| n.num != 0) {
                    o["list"] = json!({"num": n.num, "level": n.level});
                }
                if !p.props.is_empty() {
                    o["props"] = serde_json::to_value(&p.props).unwrap_or(Value::Null);
                }
                let runs: Vec<Value> = p
                    .run_ranges()
                    .filter(|(_, c)| !c.is_empty())
                    .map(|(r, c)| json!({"start": r.start, "end": r.end, "props": c}))
                    .take(50)
                    .collect();
                if !runs.is_empty() {
                    o["runs"] = json!(runs);
                }
                if !p.objects.is_empty() {
                    o["objects"] = serde_json::to_value(&p.objects).unwrap_or(Value::Null);
                }
                if p.section.is_some() {
                    o["sectionBreak"] = json!(true);
                }
                blocks.push(o);
            }
            Block::Table(t) => {
                let cells: Vec<Vec<String>> = t
                    .rows
                    .iter()
                    .map(|r| {
                        r.cells
                            .iter()
                            .map(|c| c.blocks.iter().filter_map(|b| b.as_para().map(|p| p.plain_text())).collect::<Vec<_>>().join("\n"))
                            .collect()
                    })
                    .collect();
                blocks.push(json!({"index": i, "type": "table", "rows": t.rows.len(), "cols": t.cols(), "style": t.props.style, "cells": cells}));
            }
        }
    }
    let l = s.layout();
    let parts: Vec<Value> =
        s.doc.parts.iter().map(|(id, p)| json!({"id": id, "kind": p.kind, "text": s.doc.plain_text(StoryRef::Part(*id))})).collect();
    Ok(json!({
        "blocks": blocks,
        "parts": parts,
        "selection": {"anchor": pos_json(&s.sel.anchor), "focus": pos_json(&s.sel.focus), "text": s.selected_text()},
        "pages": l.pages.len(),
        "words": s.doc.word_count(),
        "sections": s.doc.sections().iter().map(|(end, sp)| json!({"endBlock": end, "props": sp})).collect::<Vec<_>>(),
        "comments": s.doc.comments.len(),
        "trackChanges": s.doc.settings.track_changes,
        "properties": s.doc.core,
        "canUndo": s.can_undo(),
        "canRedo": s.can_redo(),
    }))
}

fn paragraph(s: &mut Session, v: &Value) -> CmdResult {
    let path: Vec<u32> = serde_json::from_value(v.get("path").cloned().unwrap_or(json!([0]))).map_err(|e| CmdError::Params(e.to_string()))?;
    let story = super::story_param(s, v);
    let p = s.doc.para(story, &doc4me_doc::Path(path)).ok_or_else(|| CmdError::Params("no paragraph there".into()))?;
    let resolved = s.doc.styles.resolve_para(&p.props);
    Ok(json!({"paragraph": p, "resolved": resolved}))
}

fn layout_summary(s: &mut Session, _: &Value) -> CmdResult {
    let l = s.layout();
    let pages: Vec<Value> = l
        .pages
        .iter()
        .map(|p| {
            let lines: usize = p.items.iter().map(|it| if let doc4me_layout::Placed::Lines { l0, l1, .. } = it { l1 - l0 } else { 0 }).sum();
            json!({"number": p.number, "w": p.w, "h": p.h, "section": p.section, "lines": lines, "firstBlock": p.first_block, "header": p.header_story, "footer": p.footer_story})
        })
        .collect();
    let caret = l.caret_on(&s.sel.focus, s.page_hint).map(|c| json!({"page": c.page, "x": c.x, "y": c.top, "h": c.height}));
    Ok(json!({"pages": pages, "caret": caret, "ms": l.ms}))
}

#[cfg(test)]
mod font_substitution_tests {
    use super::font_substitution;
    use crate::Session;
    use doc4me_doc::graphic::{Graphic, GraphicItem, TextAlign};
    use doc4me_doc::math::{MNode, MRun, Math};
    use doc4me_doc::para::{Float, InlineObject};
    use doc4me_doc::{Document, Pos, StoryRef};
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn font_substitution_lists_and_converts_graphic_and_equation_fonts() {
        let graphic_font = "Doc4Me Missing Graphic Font 76AF";
        let equation_font = "Doc4Me Missing Equation Font C351";
        let mut session = Session::new(Document::from_text("Graphic\nEquation"));
        for name in [graphic_font, equation_font] {
            session.doc.font_aliases.insert(name.to_lowercase(), doc4me_doc::FontAlias { font: name.into(), alternate: "Source Sans 3".into() });
        }
        let graphic = InlineObject::Graphic {
            w: 80.0,
            h: 32.0,
            alt: String::new(),
            float: Float::default(),
            graphic: Arc::new(Graphic {
                items: vec![GraphicItem::Text {
                    rect: [0.0, 0.0, 80.0, 32.0],
                    text: "Graphic label".into(),
                    size: 12.0,
                    color: Default::default(),
                    bold: false,
                    align: TextAlign::Left,
                    font: Some(graphic_font.into()),
                }],
                ..Default::default()
            }),
        };
        let equation = InlineObject::Equation {
            linear: "Equation label".into(),
            display: false,
            math: Math {
                nodes: vec![MNode::Run(MRun { text: "Equation label".into(), nor: true, font: Some(equation_font.into()), ..Default::default() })],
                omml: "<m:oMath><m:r><m:t>Equation label</m:t></m:r></m:oMath>".into(),
                ..Default::default()
            },
        };
        session.doc.insert_object(&Pos::body(0, 0), graphic, &Default::default()).unwrap();
        session.doc.insert_object(&Pos::body(1, 0), equation, &Default::default()).unwrap();

        let listed = font_substitution(&mut session, &json!({})).unwrap();
        let fonts = listed["fonts"].as_array().unwrap();
        assert!(fonts.iter().any(|row| row["font"] == graphic_font), "{fonts:?}");
        assert!(fonts.iter().any(|row| row["font"] == equation_font), "{fonts:?}");

        let converted = font_substitution(&mut session, &json!({"convert": [graphic_font, equation_font]})).unwrap();
        assert_eq!(converted["converted"], 1, "equation font converts; opaque graphic markup remains unconverted");
        let remaining = converted["fonts"].as_array().unwrap();
        assert!(remaining.iter().any(|row| row["font"] == graphic_font), "{remaining:?}");
        assert!(!remaining.iter().any(|row| row["font"] == equation_font), "{remaining:?}");

        let graphic_para = session.doc.para(StoryRef::Body, &doc4me_doc::Path::top(0)).unwrap();
        let InlineObject::Graphic { graphic, .. } = &graphic_para.objects[0] else { panic!("graphic is retained") };
        let GraphicItem::Text { font, .. } = &graphic.items[0] else { panic!("graphic text is retained") };
        assert_eq!(font.as_deref(), Some(graphic_font), "the command must not claim an unsaved graphic conversion");
        let equation_para = session.doc.para(StoryRef::Body, &doc4me_doc::Path::top(1)).unwrap();
        let InlineObject::Equation { math, .. } = &equation_para.objects[0] else { panic!("equation is retained") };
        let MNode::Run(run) = &math.nodes[0] else { panic!("equation run is retained") };
        assert_eq!(run.font.as_deref(), Some("Source Sans 3"));
        assert!(math.omml.is_empty(), "the stale source OMML must be invalidated after conversion");
        let reopened = doc4me_docx::read(&doc4me_docx::write(&session.doc).unwrap()).unwrap();
        let equation_para = reopened.para(StoryRef::Body, &doc4me_doc::Path::top(1)).unwrap();
        let InlineObject::Equation { math, .. } = &equation_para.objects[0] else { panic!("saved equation is retained") };
        let MNode::Run(run) = &math.nodes[0] else { panic!("saved equation run is retained") };
        assert_eq!(run.font.as_deref(), Some("Source Sans 3"), "the converted font survives DOCX round-trip");
    }
}
