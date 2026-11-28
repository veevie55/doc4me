//! Checks GitHub for Doc4Me releases. Checks run only on desktop; Help checks are manual and the
//! daily check is enabled only after the user opts in. The update page is opened in the browser;
//! this module never downloads or installs an update.

use egui::{Color32, Ui};
use serde_json::{Value, json};

use crate::WordApp;
use crate::theme::Tokens;

const LATEST_RELEASE: &str = "https://api.github.com/repos/veevie55/doc4me/releases/latest";
const MAX_RELEASE_BYTES: u64 = 256 * 1024;

/// A newer release: its version and its GitHub page.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Release {
    pub version: String,
    pub url: String,
}

/// What Help › Check for Updates' box says.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Checking,
    Latest,
    Available(Release),
    Failed(String),
}

/// The update request in progress and what it found.
#[derive(Default)]
pub(crate) struct UpdateState {
    pub(crate) pending: Option<crate::net::Pending>,
    pub(crate) manual: bool,
    pub(crate) found: Option<Release>,
}

/// The release in GitHub's response, if it is a published, final release.
fn release(v: &Value) -> Option<Release> {
    if v.get("draft").and_then(Value::as_bool) == Some(true) || v.get("prerelease").and_then(Value::as_bool) == Some(true) {
        return None;
    }
    let tag = v.get("tag_name")?.as_str()?;
    let version = tag.strip_prefix('v').unwrap_or(tag);
    if version.contains('-') || parts(version).is_none() {
        return None;
    }
    let url = v.get("html_url")?.as_str()?;
    url.starts_with("https://github.com/").then(|| Release { version: version.to_string(), url: url.to_string() })
}

/// A dotted numeric version; `None` for anything else.
fn parts(v: &str) -> Option<Vec<u64>> {
    let p: Option<Vec<u64>> = v.split('.').map(|n| n.parse().ok()).collect();
    p.filter(|p| !p.is_empty() && p.len() <= 4)
}

/// Is `latest` a later version than `current`?
fn is_newer(latest: &str, current: &str) -> bool {
    match (parts(latest), parts(current.split('-').next().unwrap_or(current))) {
        (Some(a), Some(b)) => {
            let n = a.len().max(b.len());
            let pad = |v: Vec<u64>| v.into_iter().chain(std::iter::repeat(0)).take(n).collect::<Vec<_>>();
            pad(a) > pad(b)
        }
        _ => false,
    }
}

/// This build's version.
fn current() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Today's local date, `YYYY-MM-DD`.
fn today() -> String {
    doc4me_engine::cmd::iso_from_unix_secs(doc4me_engine::cmd::now_local()).get(..10).unwrap_or("").to_string()
}

/// Only desktop builds offer the release checker. Web and non-network hosts keep their current
/// update behavior and never ask GitHub in the background.
pub fn available(app: &WordApp) -> bool {
    !cfg!(target_arch = "wasm32") && app.services.fetch.is_some()
}

/// Ask GitHub now. The request uses the shared network policy, including Trust Center settings.
fn start(app: &mut WordApp, manual: bool) -> Result<(), String> {
    if !available(app) {
        return Err(tl!("Update checks are unavailable in this build.").to_string());
    }
    // Count a failed attempt as today's check too; otherwise an offline computer asks every frame.
    app.ui.update_checked_on = today();
    let rx = crate::net::start(app, crate::net::FetchRequest::get(LATEST_RELEASE, MAX_RELEASE_BYTES))?;
    app.updates.pending = Some(crate::net::Pending::new(rx));
    app.updates.manual = manual;
    Ok(())
}

/// Every frame: show the first-run permission prompt, check daily when enabled, and take answers.
pub fn tick(app: &mut WordApp) {
    if !app.update_started {
        app.update_started = true;
    }
    if available(app) && !app.ui.update_asked && app.dialog.is_none() {
        app.ui.update_asked = true;
        app.dialog = Some(crate::dialogs::Dialog::UpdatePermission);
    }
    if available(app) && app.ui.check_updates && app.ui.update_asked && app.ui.update_checked_on != today() {
        if let Err(e) = start(app, false) {
            log::debug!("couldn't check for Doc4Me updates: {e}");
        }
    }
    let Some(answer) = app.updates.pending.as_ref().and_then(crate::net::Pending::poll) else { return };
    app.updates.pending = None;
    let answer = answer.and_then(|bytes| serde_json::from_slice(&bytes).map_err(|e| format!("GitHub returned invalid release data: {e}"))).and_then(
        |v: Value| {
            if v.get("tag_name").and_then(Value::as_str).is_some() {
                Ok(v)
            } else {
                let message = v.get("message").and_then(Value::as_str).unwrap_or("GitHub did not return a release version");
                Err(message.to_string())
            }
        },
    );
    let manual = std::mem::take(&mut app.updates.manual);
    take_answer(app, answer, manual);
}

/// Interpret the answer and update the status box or the release banner.
fn take_answer(app: &mut WordApp, answer: Result<Value, String>, manual: bool) {
    let newer = match &answer {
        Ok(v) => release(v).filter(|r| is_newer(&r.version, current())),
        Err(_) => None,
    };
    if manual && matches!(app.dialog, Some(crate::dialogs::Dialog::UpdateStatus { .. })) {
        let status = match (answer, newer) {
            (_, Some(r)) => Status::Available(r),
            (Ok(_), None) => Status::Latest,
            (Err(e), None) => Status::Failed(e),
        };
        app.dialog = Some(crate::dialogs::Dialog::UpdateStatus { status });
        return;
    }
    if let Some(r) = newer
        && (manual || app.ui.update_skipped != r.version)
    {
        app.updates.found = Some(r);
    }
}

fn buttons<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R> {
    let width = ui.available_width();
    ui.allocate_ui_with_layout(egui::vec2(width, ui.spacing().interact_size.y), egui::Layout::right_to_left(egui::Align::Center), add)
}

/// One-time question for the daily check. The explicit Help command remains available either way.
pub fn permission_box(app: &mut WordApp, ui: &mut Ui) -> bool {
    ui.label(egui::RichText::new(tl!("Check for updates automatically?")).font(crate::theme::semibold(15.0)));
    ui.label(tl!("Doc4Me can ask GitHub once a day when it starts whether a newer version has been released, then offer the download."));
    ui.label(tl!("Only the release request is sent. No document or personal information is included. Change this in File › Options › General."));
    ui.add_space(8.0);
    let (yes, no) = buttons(ui, |ui| {
        let no = ui.button(tl!("Don't Check")).clicked();
        (ui.add(crate::widgets::primary_button(ui, tl!("Check Automatically"))).clicked(), no)
    })
    .inner;
    let esc = ui.input(|i| i.key_pressed(egui::Key::Escape));
    if yes || no || esc {
        let _ = app.run("ui.checkUpdates", json!({"value": yes}));
        if yes && let Err(e) = start(app, false) {
            log::debug!("couldn't start the automatic Doc4Me update check: {e}");
        }
        return true;
    }
    false
}

/// Help › Check for Updates' result.
pub fn status_box(app: &mut WordApp, ui: &mut Ui, status: &mut Status) -> bool {
    let heading = |ui: &mut Ui, text: String| {
        ui.label(egui::RichText::new(text).font(crate::theme::semibold(15.0)));
    };
    let esc = ui.input(|i| i.key_pressed(egui::Key::Escape));
    match status.clone() {
        Status::Checking => {
            ui.horizontal(|ui| {
                ui.add(egui::Spinner::new());
                ui.label(tl!("Checking for updates…"));
            });
            ui.add_space(8.0);
            buttons(ui, |ui| ui.button(tl!("Cancel")).clicked()).inner || esc
        }
        Status::Latest => {
            heading(ui, tl!("You're up to date").to_string());
            ui.label(crate::i18n::fmt(tl!("Doc4Me {version} is the latest version."), &[("version", current())]));
            ui.add_space(8.0);
            buttons(ui, |ui| ui.add(crate::widgets::primary_button(ui, tl!("OK"))).clicked()).inner
                || esc
                || ui.input(|i| i.key_pressed(egui::Key::Enter))
        }
        Status::Failed(why) => {
            heading(ui, tl!("Couldn't check for updates").to_string());
            ui.label(why);
            ui.label(tl!("Check the internet connection, then try again from Help › Check for Updates."));
            ui.add_space(8.0);
            buttons(ui, |ui| ui.add(crate::widgets::primary_button(ui, tl!("OK"))).clicked()).inner
                || esc
                || ui.input(|i| i.key_pressed(egui::Key::Enter))
        }
        Status::Available(r) => {
            heading(ui, crate::i18n::fmt(tl!("Doc4Me {version} is available."), &[("version", &r.version)]));
            ui.label(crate::i18n::fmt(tl!("You have {version}. The download page has what's new and how to install it."), &[("version", current())]));
            ui.add_space(8.0);
            let (download, later) = buttons(ui, |ui| {
                let later = ui.button(tl!("Later")).clicked();
                (ui.add(crate::widgets::primary_button(ui, tl!("Download"))).clicked(), later)
            })
            .inner;
            if download {
                app.canvas.open_url = Some(r.url);
            }
            download || later || esc
        }
    }
}

/// Help check and the release bar buttons: check, download, skip this version, or close the bar.
pub fn command(app: &mut WordApp, p: &Value) -> Result<Value, String> {
    match p.get("action").and_then(Value::as_str) {
        None => {}
        Some("check") => {
            app.updates.found = None;
            let status = match start(app, true) {
                Ok(()) => Status::Checking,
                Err(e) => Status::Failed(e),
            };
            app.dialog = Some(crate::dialogs::Dialog::UpdateStatus { status });
        }
        Some("download") => {
            if let Some(r) = app.updates.found.take() {
                app.canvas.open_url = Some(r.url);
            }
        }
        Some("skip") => {
            if let Some(r) = app.updates.found.take() {
                app.ui.update_skipped = r.version;
            }
        }
        Some("close") => app.updates.found = None,
        Some(other) => return Err(format!("unknown action `{other}`; use check, download, skip or close")),
    }
    Ok(json!({
        "current": current(),
        "available": app.updates.found.as_ref().map(|r| json!({"version": r.version, "url": r.url})),
        "checking": app.updates.pending.is_some(),
        "skipped": app.ui.update_skipped,
    }))
}

/// The banner under the ribbon while a newer release is available.
pub fn bar(app: &mut WordApp, ui: &mut Ui) {
    let Some(r) = app.updates.found.clone() else { return };
    let t = Tokens::get(ui.ctx());
    let fill = if t.dark { Color32::from_rgb(0x4D, 0x42, 0x1C) } else { Color32::from_rgb(0xFD, 0xF3, 0xD2) };
    egui::Panel::top("update_bar")
        .frame(egui::Frame::NONE.fill(fill).inner_margin(egui::Margin::symmetric(12, 6)).stroke(egui::Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                crate::icons::paint(
                    ui.painter(),
                    egui::Rect::from_center_size(ui.cursor().min + egui::vec2(10.0, 10.0), egui::vec2(18.0, 18.0)),
                    "info",
                    t.icon,
                    t.accent,
                );
                ui.add_space(24.0);
                let news = crate::i18n::fmt(tl!("Doc4Me {version} is available."), &[("version", &r.version)]);
                ui.label(egui::RichText::new(news).strong());
                if ui.button(tl!("Download")).clicked() {
                    let _ = app.run("ui.update", json!({"action": "download"}));
                }
                if ui.button(tl!("Skip this version")).clicked() {
                    let _ = app.run("ui.update", json!({"action": "skip"}));
                }
                if crate::widgets::icon_button(ui, "close", tl!("Close")).clicked() {
                    let _ = app.run("ui.update", json!({"action": "close"}));
                }
            });
        });
}
