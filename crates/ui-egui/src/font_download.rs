//! Publisher links for fonts missing from the document currently on screen.

use serde::Serialize;

use crate::WordApp;
use crate::dialogs::Dialog;
use crate::theme::{Tokens, semibold};

const REMEMBERED: usize = 256;
const LISTED: usize = 12;
const COLUMNS: [f32; 3] = [170.0, 150.0, 190.0];

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub font: String,
    pub shown_as: Vec<String>,
    pub publisher: String,
    pub url: Option<String>,
    pub restricted: bool,
}

pub(crate) fn rows(app: &mut WordApp) -> Vec<Row> {
    let layout = app.session.layout();
    app.cached_missing_fonts(&layout)
        .0
        .iter()
        .filter_map(|f| {
            let d = f.download?;
            Some(Row {
                font: f.font.clone(),
                shown_as: f.shown_as.clone(),
                publisher: d.publisher.clone(),
                url: d.url.clone(),
                restricted: d.restricted,
            })
        })
        .take(LISTED)
        .collect()
}

pub(crate) fn open(app: &mut WordApp) -> Dialog {
    Dialog::FontDownload { fonts: rows(app), dont_show: !app.ui.font_download_notice }
}

fn remember(app: &mut WordApp, id: u64) {
    if app.font_notice_seen.len() >= REMEMBERED {
        app.font_notice_seen.remove(0);
    }
    app.font_notice_seen.push(id);
}

pub(crate) fn skip(app: &mut WordApp, id: u64) {
    if !app.font_notice_seen.contains(&id) {
        remember(app, id);
    }
}

impl WordApp {
    pub fn tell_about_font_downloads(&mut self) {
        self.font_notices = true;
    }
}

pub(crate) fn tick(app: &mut WordApp) {
    let id = app.session.document_id();
    if !app.font_notices || app.font_notice_seen.contains(&id) || app.dialog.is_some() || app.font_notice.is_some() || app.ui.backstage {
        return;
    }
    remember(app, id);
    if !app.ui.font_download_notice || app.session.path.is_none() {
        return;
    }
    let fonts = rows(app);
    if !fonts.is_empty() {
        app.dialog = Some(Dialog::FontDownload { fonts, dont_show: false });
    }
}

fn system_install_steps() -> &'static str {
    if cfg!(target_arch = "wasm32") {
        ""
    } else if cfg!(target_os = "windows") {
        tl!("Or install it for every app: select the font files, right-click and choose Install, then restart Doc4Me.")
    } else if cfg!(target_os = "macos") {
        tl!("Or install it for every app: double-click each font file and choose Install in Font Book, then restart Doc4Me.")
    } else {
        tl!("Or install it for every app: copy the font files to the .local/share/fonts folder in your home folder, then restart Doc4Me.")
    }
}

fn cell(ui: &mut egui::Ui, width: f32, text: egui::RichText) {
    ui.allocate_ui_with_layout(egui::vec2(width, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_width(width);
        ui.add(egui::Label::new(text).wrap());
    });
}

pub(crate) fn body(app: &mut WordApp, ui: &mut egui::Ui, fonts: &[Row], dont_show: &mut bool) -> bool {
    let t = Tokens::get(ui.ctx());
    ui.set_width(COLUMNS.iter().sum::<f32>() + 24.0);
    let mut open_url = None;
    let mut add = false;
    if fonts.is_empty() {
        ui.label(tl!("No downloadable fonts are listed for this document."));
    } else {
        let room = (ui.ctx().content_rect().height() - 260.0).max(80.0);
        ui.label(egui::RichText::new(tl!("This document uses fonts that aren't installed")).font(semibold(15.0)));
        ui.label(egui::RichText::new(tl!("Their text is shown in another font, so it may not look as it was written.")).color(t.text_dim));
        ui.add_space(8.0);
        egui::Frame::NONE.fill(t.panel).stroke(egui::Stroke::new(1.0, t.border)).corner_radius(8).inner_margin(egui::Margin::symmetric(10, 8)).show(
            ui,
            |ui| {
                ui.horizontal(|ui| {
                    for (width, heading) in COLUMNS.into_iter().zip(["Font in the document", "Shown with", "Get it from"]) {
                        cell(ui, width, egui::RichText::new(tl!(heading)).small().color(t.text_dim));
                    }
                });
                egui::ScrollArea::vertical().id_salt("font_downloads").max_height(room).show(ui, |ui| {
                    for f in fonts {
                        ui.horizontal(|ui| {
                            cell(ui, COLUMNS[0], egui::RichText::new(f.font.as_str()).strong());
                            cell(ui, COLUMNS[1], egui::RichText::new(f.shown_as.join(", ")));
                            ui.allocate_ui_with_layout(egui::vec2(COLUMNS[2], 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                                ui.set_width(COLUMNS[2]);
                                match &f.url {
                                    Some(url) => {
                                        let label = crate::i18n::fmt(tl!("Download from {publisher}"), &[("publisher", &f.publisher)]);
                                        if ui.link(label).on_hover_text(url.as_str()).clicked() {
                                            open_url = Some(url.clone());
                                        }
                                    }
                                    None => {
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(tl!("Comes with Windows; not offered on its own")).color(t.text_dim),
                                            )
                                            .wrap(),
                                        );
                                    }
                                }
                                if f.restricted {
                                    ui.label(
                                        egui::RichText::new(tl!("Its licence limits who may use it: read the terms first."))
                                            .small()
                                            .color(t.text_dim),
                                    );
                                }
                            });
                        });
                    }
                });
            },
        );
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(tl!("Fonts come with licence terms. Read the publisher's terms on the download page: following them is up to you."))
                .small()
                .color(t.text_dim),
        );
        if fonts.iter().any(|f| f.url.is_some()) {
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(tl!("Download the font, unpack the .zip file, then pick the font files:"));
                add = ui.button(tl!("Add Font…")).clicked();
            });
            let system = system_install_steps();
            if !system.is_empty() {
                ui.label(egui::RichText::new(system).small().color(t.text_dim));
            }
        }
    }
    if let Some(url) = open_url {
        app.canvas.open_url = Some(url);
    }
    ui.add_space(10.0);
    let mut close = ui.input(|i| i.key_pressed(egui::Key::Escape));
    ui.horizontal(|ui| {
        if !fonts.is_empty() && ui.checkbox(dont_show, tl!("Don't show this again")).changed() {
            app.ui.font_download_notice = !*dont_show;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            close |= ui.add(crate::widgets::primary_button(ui, tl!("Close"))).clicked();
        });
    });
    if add {
        let _ = app.run("font.add", serde_json::json!({}));
        return true;
    }
    close
}
