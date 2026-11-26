//! Legacy HTML Frameset Properties. The form stays local until Save, when the validated values
//! are sent through `web.frameset` so undo, HTML export and automation use the same edit path.

use egui::Ui;
use serde::Serialize;
use serde_json::{Value, json};

use crate::WordApp;

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FrameForm {
    pub src: String,
    pub name: String,
    /// 0 uses the browser default, then Auto, Yes and No.
    pub scrolling: usize,
    pub margin_width: String,
    pub margin_height: String,
    pub no_resize: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FramesetForm {
    pub rows: String,
    pub cols: String,
    pub border: String,
    /// 0 uses the HTML/browser default; 1 shows borders; 2 hides them.
    pub frame_border: usize,
    pub spacing: String,
    pub border_color: String,
    pub frames: Vec<FrameForm>,
    pub message: String,
}

impl FramesetForm {
    fn from_state(state: &Value) -> Self {
        let string = |key: &str| state.get(key).and_then(Value::as_str).unwrap_or_default().to_string();
        let number = |key: &str| state.get(key).and_then(Value::as_u64).map(|n| n.to_string()).unwrap_or_default();
        let frames = state
            .get("frames")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|frame| FrameForm {
                src: frame.get("src").and_then(Value::as_str).unwrap_or_default().to_string(),
                name: frame.get("name").and_then(Value::as_str).unwrap_or_default().to_string(),
                scrolling: match frame.get("scrolling").and_then(Value::as_str) {
                    Some("auto") => 1,
                    Some("yes") => 2,
                    Some("no") => 3,
                    _ => 0,
                },
                margin_width: frame.get("marginWidth").and_then(Value::as_u64).map(|n| n.to_string()).unwrap_or_default(),
                margin_height: frame.get("marginHeight").and_then(Value::as_u64).map(|n| n.to_string()).unwrap_or_default(),
                no_resize: frame.get("noResize").and_then(Value::as_bool).unwrap_or(false),
            })
            .collect();
        Self {
            rows: string("rows"),
            cols: string("cols"),
            border: number("border"),
            frame_border: match state.get("frameBorder").and_then(Value::as_bool) {
                Some(true) => 1,
                Some(false) => 2,
                None => 0,
            },
            spacing: number("spacing"),
            border_color: string("borderColor"),
            frames,
            message: String::new(),
        }
    }

    /// Build the complete command payload, validating and canonicalizing every user-editable
    /// value before the command can change the document.
    pub(crate) fn params(&self) -> Result<Value, String> {
        let tracks = |key: &str, value: &str| -> Result<Value, String> {
            let value = value.trim();
            if value.is_empty() {
                return Ok(Value::Null);
            }
            doc4me_doc::HtmlFrameset::clean_tracks(value)
                .map(Value::String)
                .ok_or_else(|| format!("{key} must be a comma-separated list of up to 16 pixel, percentage or * tracks"))
        };
        let optional_number = |key: &str, value: &str, max: u16| -> Result<Value, String> {
            let value = value.trim();
            if value.is_empty() {
                return Ok(Value::Null);
            }
            let number =
                value.parse::<u16>().ok().filter(|number| *number <= max).ok_or_else(|| format!("{key} must be a whole number from 0 to {max}"))?;
            Ok(json!(number))
        };

        let rows = tracks("Rows", &self.rows)?;
        let cols = tracks("Columns", &self.cols)?;
        let border = optional_number("Border width", &self.border, 100)?;
        let spacing = optional_number("Frame spacing", &self.spacing, 100)?;
        let border_color = if self.border_color.trim().is_empty() {
            Value::Null
        } else {
            let color = self.border_color.trim();
            let digits = color.strip_prefix('#').unwrap_or(color);
            if digits.len() != 6 || !digits.is_ascii() {
                return Err("Border color must be six hexadecimal digits, optionally prefixed with #".into());
            }
            let Some(color) = doc4me_doc::Rgb::parse(digits) else {
                return Err("Border color must be six hexadecimal digits, optionally prefixed with #".into());
            };
            json!(format!("#{}", color.hex()))
        };
        let frame_border = match self.frame_border {
            0 => Value::Null,
            1 => json!(true),
            2 => json!(false),
            _ => return Err("Choose a valid frame border setting".into()),
        };
        if self.frames.len() > doc4me_doc::MAX_HTML_FRAMES {
            return Err(format!("A frameset can contain up to {} frames", doc4me_doc::MAX_HTML_FRAMES));
        }
        let frames = self.frames.iter().enumerate().map(|(index, frame)| frame_params(frame, index + 1)).collect::<Result<Vec<_>, _>>()?;

        Ok(json!({
            "rows": rows,
            "cols": cols,
            "border": border,
            "frameBorder": frame_border,
            "spacing": spacing,
            "borderColor": border_color,
            "frames": frames,
        }))
    }
}

fn frame_params(frame: &FrameForm, index: usize) -> Result<Value, String> {
    let src = frame.src.trim();
    if src.is_empty() || src.len() > 2048 || !doc4me_doc::link::is_safe_href(src) {
        return Err(format!("Frame {index} needs a safe relative, HTTP(S) or mail URL (up to 2048 bytes)"));
    }
    let name = frame.name.trim();
    if !name.is_empty() && (name.len() > 128 || !name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':'))) {
        return Err(format!("Frame {index} name may use letters, digits, _, -, . and : (up to 128 bytes)"));
    }
    let scrolling = match frame.scrolling {
        0 => Value::Null,
        1 => json!("auto"),
        2 => json!("yes"),
        3 => json!("no"),
        _ => return Err(format!("Frame {index} has an invalid scrolling setting")),
    };
    let margin = |label: &str, value: &str| -> Result<Value, String> {
        let value = value.trim();
        if value.is_empty() {
            return Ok(Value::Null);
        }
        let number = value
            .parse::<u16>()
            .ok()
            .filter(|number| *number <= 1000)
            .ok_or_else(|| format!("Frame {index} {label} must be a whole number from 0 to 1000"))?;
        Ok(json!(number))
    };

    Ok(json!({
        "src": src,
        "name": if name.is_empty() { Value::Null } else { json!(name) },
        "scrolling": scrolling,
        "marginWidth": margin("margin width", &frame.margin_width)?,
        "marginHeight": margin("margin height", &frame.margin_height)?,
        "noResize": frame.no_resize,
    }))
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FramesetDialog {
    pub(crate) form: FramesetForm,
}

impl FramesetDialog {
    pub(crate) const fn name(&self) -> &'static str {
        "framesetProperties"
    }

    pub(crate) const fn title(&self) -> &'static str {
        "Frameset Properties"
    }
}

/// Open the dialog from the engine request. The command is also the source of truth for the
/// currently edited frameset, so the dialog never guesses at document defaults.
pub(crate) fn open(name: &str, app: &mut WordApp) -> Option<FramesetDialog> {
    if name != "framesetProperties" {
        return None;
    }
    let state = app.session.run("web.frameset", &json!({})).ok()?;
    Some(FramesetDialog { form: FramesetForm::from_state(&state) })
}

/// Render the form. `true` closes it; invalid or refused saves leave the edited values in place.
pub(crate) fn body(app: &mut WordApp, ui: &mut Ui, dialog: &mut FramesetDialog) -> bool {
    let form = &mut dialog.form;
    egui::Grid::new("frameset_properties").num_columns(2).spacing([12.0, 7.0]).show(ui, |ui| {
        ui.label(crate::tl!("Rows:"));
        ui.add(crate::widgets::text_line(&mut form.rows).hint_text(crate::tl!("e.g. 25%,*")));
        ui.end_row();
        ui.label(crate::tl!("Columns:"));
        ui.add(crate::widgets::text_line(&mut form.cols).hint_text(crate::tl!("e.g. 30%,*")));
        ui.end_row();
        ui.label(crate::tl!("Border width:"));
        ui.add(crate::widgets::text_line(&mut form.border).hint_text(crate::tl!("Pixels (0–100)")));
        ui.end_row();
        ui.label(crate::tl!("Frame borders:"));
        egui::ComboBox::from_id_salt("frameset_frame_border")
            .selected_text(crate::tl!(match form.frame_border {
                1 => "Show",
                2 => "Hide",
                _ => "Default",
            }))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut form.frame_border, 0, crate::tl!("Default"));
                ui.selectable_value(&mut form.frame_border, 1, crate::tl!("Show"));
                ui.selectable_value(&mut form.frame_border, 2, crate::tl!("Hide"));
            });
        ui.end_row();
        ui.label(crate::tl!("Frame spacing:"));
        ui.add(crate::widgets::text_line(&mut form.spacing).hint_text(crate::tl!("Pixels (0–100)")));
        ui.end_row();
        ui.label(crate::tl!("Border color:"));
        ui.add(crate::widgets::text_line(&mut form.border_color).hint_text("#RRGGBB"));
        ui.end_row();
    });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(crate::tl!("Frames:")).strong());
        ui.label(egui::RichText::new(format!("{} / {}", form.frames.len(), doc4me_doc::MAX_HTML_FRAMES)).small().weak());
        if ui.add_enabled(form.frames.len() < doc4me_doc::MAX_HTML_FRAMES, egui::Button::new(crate::tl!("Add frame"))).clicked() {
            form.frames.push(FrameForm::default());
        }
    });
    egui::ScrollArea::vertical().id_salt("frameset_frames").max_height(250.0).auto_shrink([false, false]).show(ui, |ui| {
        let mut remove = None;
        for (index, frame) in form.frames.iter_mut().enumerate() {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("{}", index + 1)).strong());
                    ui.add(crate::widgets::text_line(&mut frame.src).hint_text(crate::tl!("Frame URL")).desired_width(270.0));
                    if ui.button(crate::tl!("Remove")).clicked() {
                        remove = Some(index);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(crate::tl!("Name:"));
                    ui.add(crate::widgets::text_line(&mut frame.name).hint_text(crate::tl!("Optional")));
                    ui.label(crate::tl!("Scrolling:"));
                    egui::ComboBox::from_id_salt(("frameset_scrolling", index))
                        .selected_text(crate::tl!(match frame.scrolling {
                            1 => "Auto",
                            2 => "Yes",
                            3 => "No",
                            _ => "Default",
                        }))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut frame.scrolling, 0, crate::tl!("Default"));
                            ui.selectable_value(&mut frame.scrolling, 1, crate::tl!("Auto"));
                            ui.selectable_value(&mut frame.scrolling, 2, crate::tl!("Yes"));
                            ui.selectable_value(&mut frame.scrolling, 3, crate::tl!("No"));
                        });
                });
                ui.horizontal(|ui| {
                    ui.label(crate::tl!("Margin width:"));
                    ui.add(crate::widgets::text_line(&mut frame.margin_width).hint_text(crate::tl!("Optional, 0–1000")));
                    ui.label(crate::tl!("Margin height:"));
                    ui.add(crate::widgets::text_line(&mut frame.margin_height).hint_text(crate::tl!("Optional, 0–1000")));
                    ui.checkbox(&mut frame.no_resize, crate::tl!("No resize"));
                });
            });
        }
        if let Some(index) = remove {
            form.frames.remove(index);
        }
    });

    let validation = form.params().err();
    if let Some(message) = validation.as_deref().or_else(|| (!form.message.is_empty()).then_some(form.message.as_str())) {
        ui.add_space(4.0);
        ui.colored_label(ui.visuals().error_fg_color, crate::tl!(message));
    }
    let (ok, cancel) = buttons(ui, validation.is_none());
    if ok {
        match save(app, form) {
            Ok(()) => return true,
            Err(error) => {
                form.message = error.clone();
                app.status(error);
            }
        }
    }
    cancel
}

/// Validate once more at the action boundary, then send one complete replacement to the engine.
pub(crate) fn save(app: &mut WordApp, form: &FramesetForm) -> Result<(), String> {
    let params = form.params()?;
    app.run("web.frameset", params).map(|_| ())
}

fn buttons(ui: &mut Ui, ok_enabled: bool) -> (bool, bool) {
    let mut result = (false, false);
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            result.1 = ui.button(crate::tl!("Cancel")).clicked() || ui.input(|input| input.key_pressed(egui::Key::Escape));
            result.0 = ui.add_enabled(ok_enabled, crate::widgets::primary_button(ui, crate::tl!("OK"))).clicked();
        });
    });
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form() -> FramesetForm {
        FramesetForm {
            rows: " 50% , * ".into(),
            cols: "640, *".into(),
            border: "2".into(),
            frame_border: 2,
            spacing: "3".into(),
            border_color: "#aabbcc".into(),
            frames: vec![FrameForm {
                src: " content.html ".into(),
                name: "main_frame".into(),
                scrolling: 3,
                margin_width: "4".into(),
                margin_height: String::new(),
                no_resize: true,
            }],
            message: String::new(),
        }
    }

    #[test]
    fn form_builds_a_complete_canonical_command_payload() {
        let params = form().params().unwrap();
        assert_eq!(params["rows"], "50%,1*");
        assert_eq!(params["cols"], "640,1*");
        assert_eq!(params["border"], 2);
        assert_eq!(params["frameBorder"], false);
        assert_eq!(params["spacing"], 3);
        assert_eq!(params["borderColor"], "#AABBCC");
        assert_eq!(params["frames"][0]["src"], "content.html");
        assert_eq!(params["frames"][0]["name"], "main_frame");
        assert_eq!(params["frames"][0]["scrolling"], "no");
        assert_eq!(params["frames"][0]["marginWidth"], 4);
        assert!(params["frames"][0]["marginHeight"].is_null());
        assert_eq!(params["frames"][0]["noResize"], true);
    }

    #[test]
    fn invalid_values_are_rejected_before_a_command_payload_exists() {
        let mut value = form();
        value.rows = "100%,javascript".into();
        assert!(value.params().unwrap_err().contains("Rows"));

        value = form();
        value.border = "101".into();
        assert!(value.params().unwrap_err().contains("Border width"));

        value = form();
        value.border_color = "red".into();
        assert!(value.params().unwrap_err().contains("Border color"));

        value = form();
        value.frames[0].src = "javascript:alert(1)".into();
        assert!(value.params().unwrap_err().contains("Frame 1"));

        value = form();
        value.frames[0].name = "bad name".into();
        assert!(value.params().unwrap_err().contains("Frame 1 name"));
    }

    #[test]
    fn open_reads_current_values_and_save_uses_the_command_for_undoable_edits() {
        let mut doc = doc4me_doc::Document::from_text("Fallback");
        doc.settings.html_frameset = Some(doc4me_doc::HtmlFrameset {
            rows: Some("*,*".into()),
            frames: vec![doc4me_doc::HtmlFrame { src: "main.html".into(), name: Some("main".into()), ..Default::default() }],
            ..Default::default()
        });
        let mut app = WordApp::new(doc4me_engine::Session::new(doc), crate::Services::default());
        let mut dialog = open("framesetProperties", &mut app).unwrap();
        assert_eq!(dialog.form.rows, "*,*");
        assert_eq!(dialog.form.frames[0].src, "main.html");

        dialog.form.cols = "70%,*".into();
        save(&mut app, &dialog.form).unwrap();
        assert_eq!(app.session.doc.settings.html_frameset.as_ref().and_then(|fs| fs.cols.as_deref()), Some("70%,1*"));
        app.session.run("edit.undo", &json!({})).unwrap();
        assert_eq!(app.session.doc.settings.html_frameset.as_ref().and_then(|fs| fs.cols.as_deref()), None);
    }

    #[test]
    fn invalid_save_keeps_document_unchanged() {
        let mut doc = doc4me_doc::Document::from_text("Fallback");
        doc.settings.html_frameset = Some(doc4me_doc::HtmlFrameset { rows: Some("*,*".into()), ..Default::default() });
        let mut app = WordApp::new(doc4me_engine::Session::new(doc), crate::Services::default());
        let before = app.session.doc.settings.html_frameset.clone();
        let mut invalid = form();
        invalid.frames[0].src = "file:///secret".into();
        assert!(save(&mut app, &invalid).is_err());
        assert_eq!(app.session.doc.settings.html_frameset, before);
    }
}
