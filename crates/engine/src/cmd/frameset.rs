//! Legacy HTML Frameset Properties. A frameset lives in an HTML page, not in a Word-processing
//! layout; its editable frame list and border/layout properties are retained by the HTML codec.

use doc4me_doc::{HtmlFrame, HtmlFrameScrolling, HtmlFrameset, Rgb};
use serde_json::{Value, json};

use crate::{CmdError, CmdResult, CommandSpec, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![CommandSpec::new("web.frameset", "Frameset Properties", "Web", frameset).params(
        r##"{"rows"?: string|null, "cols"?: string|null, "border"?: 0–100|null, "frameBorder"?: bool|null, "spacing"?: 0–100|null, "borderColor"?: "#RRGGBB"|null, "frames"?: [{"src": string, "name"?: string|null, "scrolling"?: "auto|yes|no"|null, "marginWidth"?: 0–1000|null, "marginHeight"?: 0–1000|null, "noResize"?: bool}]} — query/replace an imported HTML frameset; with no values opens Frameset Properties."##,
    )]
}

fn state(s: &Session) -> CmdResult {
    let Some(frameset) = &s.doc.settings.html_frameset else {
        return Err(CmdError::Failed("Frameset Properties applies to an HTML frameset document".into()));
    };
    Ok(json!({
        "rows": frameset.rows,
        "cols": frameset.cols,
        "border": frameset.border,
        "frameBorder": frameset.frame_border,
        "spacing": frameset.spacing,
        "borderColor": frameset.border_color.map(|c| format!("#{}", c.hex())),
        "frames": frameset.frames,
    }))
}

fn optional_tracks(v: &Value, key: &str, target: &mut Option<String>) -> Result<bool, CmdError> {
    match v.get(key) {
        None => Ok(false),
        Some(Value::Null) => {
            *target = None;
            Ok(true)
        }
        Some(Value::String(s)) => {
            let Some(s) = HtmlFrameset::clean_tracks(s) else {
                return Err(CmdError::Params(format!("`{key}` must be up to 16 comma-separated pixel, percentage or * tracks")));
            };
            *target = Some(s);
            Ok(true)
        }
        Some(_) => Err(CmdError::Params(format!("`{key}` must be a string or null"))),
    }
}

fn optional_number(v: &Value, key: &str, target: &mut Option<u16>, max: u16) -> Result<bool, CmdError> {
    match v.get(key) {
        None => Ok(false),
        Some(Value::Null) => {
            *target = None;
            Ok(true)
        }
        Some(n) => {
            let Some(n) = n.as_u64().and_then(|n| u16::try_from(n).ok()).filter(|n| *n <= max) else {
                return Err(CmdError::Params(format!("`{key}` must be 0 to {max} or null")));
            };
            *target = Some(n);
            Ok(true)
        }
    }
}

fn frame_from_value(v: &Value) -> Result<HtmlFrame, CmdError> {
    let Some(o) = v.as_object() else { return Err(CmdError::Params("each frame must be an object".into())) };
    let Some(src) = o.get("src").and_then(Value::as_str).map(str::trim).filter(|s| s.len() <= 2048 && doc4me_doc::link::is_safe_href(s)) else {
        return Err(CmdError::Params("each frame needs a safe relative or web `src` URL".into()));
    };
    let name = match o.get("name") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => {
            let s = s.trim();
            if s.is_empty() || s.len() > 128 || !s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':')) {
                return Err(CmdError::Params("frame `name` uses letters, digits, `_`, `-`, `.`, or `:` (up to 128 bytes)".into()));
            }
            Some(s.to_string())
        }
        Some(_) => return Err(CmdError::Params("frame `name` must be a string or null".into())),
    };
    let scrolling = match o.get("scrolling") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => {
            Some(HtmlFrameScrolling::parse(s).ok_or_else(|| CmdError::Params("frame `scrolling` is auto, yes, no or null".into()))?)
        }
        Some(_) => return Err(CmdError::Params("frame `scrolling` is auto, yes, no or null".into())),
    };
    let margin = |key: &str| -> Result<Option<u16>, CmdError> {
        match o.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(n) => n
                .as_u64()
                .and_then(|n| u16::try_from(n).ok())
                .filter(|n| *n <= 1000)
                .map(Some)
                .ok_or_else(|| CmdError::Params(format!("frame `{key}` must be 0 to 1000 or null"))),
        }
    };
    let no_resize = match o.get("noResize") {
        None => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => return Err(CmdError::Params("frame `noResize` must be true or false".into())),
    };
    Ok(HtmlFrame { src: src.to_string(), name, scrolling, margin_width: margin("marginWidth")?, margin_height: margin("marginHeight")?, no_resize })
}

fn frameset(s: &mut Session, v: &Value) -> CmdResult {
    let Some(current) = s.doc.settings.html_frameset.clone() else {
        s.only_read();
        return Err(CmdError::Failed("Frameset Properties applies to an HTML frameset document".into()));
    };
    let fields = v.as_object();
    if fields.is_none_or(|fields| fields.is_empty()) {
        s.only_read();
        s.push_ui_request(json!({"open": "framesetProperties"}));
        let mut result = state(s)?;
        if let Some(o) = result.as_object_mut() {
            o.insert("opened".into(), json!(true));
        }
        return Ok(result);
    }

    let mut next = current;
    let mut changed = optional_tracks(v, "rows", &mut next.rows)?;
    changed |= optional_tracks(v, "cols", &mut next.cols)?;
    changed |= optional_number(v, "border", &mut next.border, 100)?;
    changed |= optional_number(v, "spacing", &mut next.spacing, 100)?;
    match v.get("frameBorder") {
        None => {}
        Some(Value::Null) => {
            next.frame_border = None;
            changed = true;
        }
        Some(Value::Bool(b)) => {
            next.frame_border = Some(*b);
            changed = true;
        }
        Some(_) => return Err(CmdError::Params("`frameBorder` must be true, false or null".into())),
    }
    match v.get("borderColor") {
        None => {}
        Some(Value::Null) => {
            next.border_color = None;
            changed = true;
        }
        Some(Value::String(color)) => {
            let Some(color) = Rgb::parse(color) else { return Err(CmdError::Params("`borderColor` must be #RRGGBB or null".into())) };
            next.border_color = Some(color);
            changed = true;
        }
        Some(_) => return Err(CmdError::Params("`borderColor` must be #RRGGBB or null".into())),
    }
    match v.get("frames") {
        None => {}
        Some(Value::Null) => {
            next.frames.clear();
            changed = true;
        }
        Some(Value::Array(frames)) if frames.len() <= doc4me_doc::MAX_HTML_FRAMES => {
            next.frames = frames.iter().map(frame_from_value).collect::<Result<Vec<_>, _>>()?;
            changed = true;
        }
        Some(Value::Array(_)) => return Err(CmdError::Params(format!("`frames` has at most {} entries", doc4me_doc::MAX_HTML_FRAMES))),
        Some(_) => return Err(CmdError::Params("`frames` must be an array or null".into())),
    }
    if !changed {
        s.only_read();
        return state(s);
    }
    next.sanitize();
    s.doc.settings.html_frameset = Some(next);
    state(s)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::Session;

    #[test]
    fn frameset_properties_edit_html_state_and_undo() {
        let mut doc = doc4me_doc::Document::from_text("Fallback");
        doc.settings.html_frameset = Some(doc4me_doc::HtmlFrameset { rows: Some("*,*".into()), ..Default::default() });
        let mut s = Session::new(doc);
        let opened = s.run("web.frameset", &json!({})).unwrap();
        assert_eq!((opened["opened"].as_bool(), opened["rows"].as_str()), (Some(true), Some("*,*")));
        assert!(s.run("web.frameset", &json!({"rows": "100%,100%"})).is_ok());
        assert!(s.run("web.frameset", &json!({"rows": "100%,javascript"})).is_err());
        assert!(s.run("web.frameset", &json!({"frames": [{"src": "javascript:alert(1)"}]})).is_err());
        assert_eq!(s.doc.settings.html_frameset.as_ref().and_then(|fs| fs.rows.as_deref()), Some("100%,100%"));

        s.run(
            "web.frameset",
            &json!({
                "cols": "70%,*", "border": 2, "frameBorder": false, "spacing": 3, "borderColor": "#aabbcc",
                "frames": [{"src": "one.html", "name": "content", "scrolling": "no", "marginWidth": 4, "noResize": true}]
            }),
        )
        .unwrap();
        let html = doc4me_formats::html::export(&s.doc);
        let reopened = doc4me_formats::html::import(&html);
        assert_eq!(reopened.settings.html_frameset, s.doc.settings.html_frameset);

        s.run("edit.undo", &json!({})).unwrap();
        assert_eq!(s.doc.settings.html_frameset.as_ref().and_then(|fs| fs.cols.as_deref()), None);
    }

    #[test]
    fn frameset_properties_cannot_be_applied_to_a_normal_document() {
        let mut s = Session::new(doc4me_doc::Document::new());
        assert!(s.run("web.frameset", &json!({})).is_err());
        assert!(s.run("web.frameset", &json!({"border": 1})).is_err());
    }
}
