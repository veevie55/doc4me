//! Fonts the user adds without rebuilding (#582): `font.add`, `font.list`, `font.remove`.
//!
//! An added font joins the process-wide font database on top of the bundled and system fonts
//! (never replacing a face already there), so it shows in the font lists, lays out, renders and
//! embeds in PDFs like any other. Where it is kept between runs is [`FontDb::load_user_fonts`]'s
//! folder (desktop); without one it lasts for the session (the web shell stores it itself).

use doc4me_fonts::{FontDb, UserFont};
use serde_json::{Value, json};

use crate::cmd::insert::base64_decode;
use crate::{CmdError, CmdResult, CommandSpec, Session, p};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("font.add", "Add Font…", "Home › Font", add)
            .params(r#"{"path"?: string, "data"?: base64, "name"?: string (file name for "data")} — a TTF, OTF, TTC, WOFF or WOFF2 file; none: pick a file"#)
            .pure(),
        CommandSpec::new("font.list", "Added Fonts", "Home › Font", list).params(r#"{"all"?: bool (also every available family)}"#).pure(),
        CommandSpec::new("font.remove", "Remove Added Font", "Home › Font", remove).params(r#"{"family": string}"#).pure(),
        CommandSpec::new("font.missing", "Missing Fonts", "Status Bar", missing)
            .params(r#"{} → {"missing": [{"font": string, "shownAs": [string], "metricCompatible": bool, "download"?: object}]} — fonts absent from the current layout"#)
            .pure(),
    ]
}

fn font_json(f: &UserFont) -> Value {
    json!({
        "file": f.file,
        "families": f.families(),
        "faces": f.faces.iter().map(|(family, style)| json!({"family": family, "style": style})).collect::<Vec<_>>(),
        "path": f.path.as_ref().map(|p| p.to_string_lossy().to_string()),
    })
}

/// Read a font file named by the user, refusing one larger than the cap before reading it.
#[cfg(not(target_arch = "wasm32"))]
fn read_font_file(path: &str) -> Result<Vec<u8>, CmdError> {
    // Through `io`, which keeps automation inside its read root (#769).
    crate::io::read_file_capped(path, doc4me_fonts::MAX_FONT_FILE_BYTES as u64, "font file").map_err(CmdError::Failed)
}

/// Adding or removing a font writes to the user font folder: automation confined to a write root
/// (#769) may do that only when the folder is inside it.
fn check_font_store(dir: Option<&std::path::Path>) -> Result<(), CmdError> {
    match dir {
        Some(dir) => crate::file_access::check_write(&dir.to_string_lossy()).map_err(CmdError::Failed),
        None => Ok(()),
    }
}

fn add(s: &mut Session, v: &Value) -> CmdResult {
    if p::str(v, "url").is_some() {
        return Err(CmdError::Failed(
            "adding a font from a link isn't available yet: download the font file and add it with {\"path\"} or {\"data\"}".into(),
        ));
    }
    let (name, bytes) = if let Some(d) = p::str(v, "data") {
        // Base64 is a third larger than the file: refuse before decoding an oversized one.
        if d.len() / 4 * 3 > doc4me_fonts::MAX_FONT_FILE_BYTES + 3 {
            return Err(CmdError::Failed(format!("the font file is larger than {} MB", doc4me_fonts::MAX_FONT_FILE_BYTES >> 20)));
        }
        let bytes = base64_decode(d).ok_or_else(|| CmdError::Params("bad base64 data".into()))?;
        (p::str(v, "name").unwrap_or("font").to_string(), bytes)
    } else if let Some(path) = p::str(v, "path") {
        #[cfg(not(target_arch = "wasm32"))]
        {
            (path.to_string(), read_font_file(path)?)
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = path;
            return Err(CmdError::Failed("reading files by path isn't available on the web".into()));
        }
    } else {
        // From the ribbon: the UI shows a file picker and runs this again with the file.
        s.push_ui_request(json!({"open": "addFont"}));
        return Ok(json!({"pending": true}));
    };
    check_font_store(FontDb::global().user_font_dir().as_deref())?;
    let r = FontDb::global().install_font(&name, &bytes).map_err(|e| CmdError::Failed(e.to_string()))?;
    // Text already set in the family (shown in a fallback until now) takes the new font.
    s.relayout();
    let families = r.font.families().join(", ");
    s.status = match &r.warning {
        Some(w) => format!("Added {families} ({w})"),
        None => format!("Added {families}"),
    };
    let mut out = font_json(&r.font);
    if let Some(w) = r.warning {
        out["warning"] = json!(w);
    }
    Ok(out)
}

fn list(_: &mut Session, v: &Value) -> CmdResult {
    let db = FontDb::global();
    let mut out = json!({
        "fonts": db.user_fonts().iter().map(font_json).collect::<Vec<_>>(),
        "folder": db.user_font_dir().map(|p| p.to_string_lossy().to_string()),
    });
    if p::bool(v, "all") == Some(true) {
        out["families"] = json!(db.families());
    }
    Ok(out)
}

fn remove(s: &mut Session, v: &Value) -> CmdResult {
    let family = p::str(v, "family").filter(|f| !f.trim().is_empty()).ok_or_else(|| CmdError::Params("family is required".into()))?;
    check_font_store(FontDb::global().user_font_dir().as_deref())?;
    let gone = FontDb::global().remove_user_font(family.trim()).map_err(|e| CmdError::Failed(e.to_string()))?;
    s.relayout();
    s.status = format!("Removed {}", family.trim());
    Ok(json!({"removed": gone.iter().map(font_json).collect::<Vec<_>>()}))
}

/// A font the visible document layout asked for but could not find.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissingFont {
    pub font: String,
    pub shown_as: Vec<String>,
    pub metric_compatible: bool,
    pub download: Option<&'static doc4me_fonts::download::FontDownload>,
}

/// The family text in `font` resolves to when that family is unavailable.
pub fn shown_as(font: &str) -> Option<String> {
    let resolved = doc4me_fonts::word::resolve(font.trim(), false, false);
    resolved.substituted.then(|| resolved.face.get().family.clone())
}

pub fn warns(font: &str, substitute: &str) -> bool {
    let symbol = matches!(font.trim().to_ascii_lowercase().as_str(), "symbol" | "wingdings" | "wingdings 2" | "wingdings 3" | "webdings");
    !symbol && !doc4me_fonts::word::metric_compatible(font, substitute)
}

pub fn font_key(font: &str) -> String {
    reported_font_name(font).trim().to_lowercase()
}

fn reported_font_name(font: &str) -> &str {
    font.char_indices().nth(256).and_then(|(i, _)| font.get(..i)).unwrap_or(font)
}

/// Missing families and the faces the current layout uses instead.
pub fn missing_fonts(layout: &doc4me_layout::DocLayout) -> Vec<MissingFont> {
    let mut out = std::collections::BTreeMap::<String, MissingFont>::new();
    for (recorded, face) in layout.substituted_fonts() {
        let font = reported_font_name(&recorded).trim();
        if font.is_empty() {
            continue;
        }
        let family = face.get().family.clone();
        let entry = out.entry(font_key(font)).or_insert_with(|| MissingFont {
            font: font.to_string(),
            shown_as: Vec::new(),
            metric_compatible: !warns(font, &family),
            download: doc4me_fonts::download::download_for(font),
        });
        if !entry.shown_as.contains(&family) {
            entry.shown_as.push(family);
        }
    }
    out.into_values().collect()
}

fn missing(s: &mut Session, _: &Value) -> CmdResult {
    let found = missing_fonts(&s.layout());
    let changing: Vec<&str> = found.iter().filter(|f| !f.metric_compatible).map(|f| f.font.as_str()).collect();
    let names = changing
        .iter()
        .take(5)
        .map(|f| f.char_indices().nth(40).and_then(|(i, _)| f.get(..i)).map(|s| format!("{s}…")).unwrap_or_else(|| (*f).to_string()))
        .collect::<Vec<_>>()
        .join(", ");
    s.status = match (found.is_empty(), changing.len()) {
        (true, _) => "Every font the document shows is installed".into(),
        (false, 0) => "Missing fonts are shown in fonts that measure the same; line breaks don't move".into(),
        (_, n) if n > 5 => format!("Not installed: {names} and {} more", n - 5),
        _ => format!("Not installed: {names}"),
    };
    let fonts = found
        .into_iter()
        .map(|f| {
            let mut value = json!({"font": f.font, "shownAs": f.shown_as, "metricCompatible": f.metric_compatible});
            if let Some(d) = f.download {
                value["download"] = json!({"name": d.name, "publisher": d.publisher, "url": d.url, "windows": d.windows, "restricted": d.restricted});
            }
            value
        })
        .collect::<Vec<_>>();
    Ok(json!({"missing": fonts}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confined_automation_changes_only_a_font_store_inside_its_write_root() {
        let root = std::env::temp_dir().join(format!("doc4me-font-store-{}", std::process::id()));
        let outside = std::env::temp_dir().join(format!("doc4me-font-store-out-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        let write = crate::file_access::AutomationRoots::new(None, Some(&root)).unwrap();
        let read_only = crate::file_access::AutomationRoots::new(Some(&root), None).unwrap();
        crate::file_access::confine(write.as_ref(), || {
            assert!(check_font_store(None).is_ok(), "no folder: the font lasts for the session");
            assert!(check_font_store(Some(&root.join("fonts"))).is_ok());
            assert!(check_font_store(Some(&outside)).is_err());
        });
        crate::file_access::confine(read_only.as_ref(), || assert!(check_font_store(Some(&root)).is_err()));
        assert!(check_font_store(Some(&outside)).is_ok(), "unconfined: the person at the keyboard");
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&outside);
    }
}
