//! UI experiment (branch `ui-pages-layout`, 2026-11-23; first drawn on `ui-rail-panel`): the
//! formatting panes of the full-height right panel. With the one-line ribbon, the selection's
//! Text and Paragraph settings and the Page panel's Page and Design settings are panes made of
//! sections that fold. They hold what the ribbon line no longer shows (and the everyday commands
//! again, so a pane is complete on its own). Every control runs the same command as its ribbon
//! button.

use egui::{Ui, vec2};
use serde_json::{Value, json};

use crate::WordApp;
use crate::colors::{Remember, Target};
use crate::options::MeasureExt;
use crate::ribbon::{mi, saved_list_items, underline_sample};
use crate::widgets::{combo, menu_button, small, split};

/// A section of a pane that folds; open the first time, then as the user left it.
fn section(ui: &mut Ui, id: &str, title: &str, add: impl FnOnce(&mut Ui)) {
    ui.push_id(("format_pane_section", id), |ui| {
        ui.collapsing(tl!(title), |ui| {
            ui.spacing_mut().item_spacing = vec2(2.0, 4.0);
            add(ui);
        });
    });
}

fn bullet_list_items(ui: &mut Ui, app: &mut WordApp) {
    ui.separator();
    for c in ["•", "○", "▪", "◆", "➢", "✓", "–"] {
        if ui.button(egui::RichText::new(c).size(16.0)).clicked() {
            let _ = app.run("para.bullets", json!({"kind": c}));
            ui.close();
        }
    }
    mi(ui, app, "None", "para.bullets", json!({"off": true}));
    mi(ui, app, "Define New Bullet…", "ui.dialog", json!({"name": "defineBullet"}));
}

fn border_items(ui: &mut Ui, app: &mut WordApp) {
    let cmd = if crate::widgets::enabled(app, "table.borders") { "table.borders" } else { "para.borders" };
    for (label, kind) in [
        ("Bottom Border", "bottom"),
        ("Top Border", "top"),
        ("Left Border", "left"),
        ("Right Border", "right"),
        ("No Border", "none"),
        ("All Borders", "all"),
        ("Outside Borders", "outside"),
        ("Inside Borders", "inside"),
    ] {
        mi(ui, app, label, cmd, json!({"kind": kind}));
    }
    mi(ui, app, "Borders and Shading…", "para.borders", json!({}));
}

fn size_menu(ui: &mut Ui, app: &mut WordApp) {
    for (name, width, height) in doc4me_geom::PAPER_SIZES {
        mi(
            ui,
            app,
            &format!("{name}   {} × {}", crate::options::length(*width), crate::options::length(*height)),
            "layout.size",
            json!({"name": name}),
        );
    }
}

fn orientation_items(ui: &mut Ui, app: &mut WordApp) {
    mi(ui, app, "Portrait", "layout.orientation", json!({"value": "portrait"}));
    mi(ui, app, "Landscape", "layout.orientation", json!({"value": "landscape"}));
}

fn margins_items(ui: &mut Ui, app: &mut WordApp) {
    for (label, preset) in [
        ("Normal", "normal"),
        ("Narrow", "narrow"),
        ("Moderate", "moderate"),
        ("Wide", "wide"),
        ("Mirrored", "mirrored"),
        ("Office 2003 Default", "office2003"),
    ] {
        mi(ui, app, label, "layout.margins", json!({"preset": preset}));
    }
    mi(ui, app, "Custom Margins…", "ui.dialog", json!({"name": "pageSetup"}));
}

fn columns_items(ui: &mut Ui, app: &mut WordApp) {
    for (label, params) in [
        ("One", json!({"count": 1})),
        ("Two", json!({"count": 2})),
        ("Three", json!({"count": 3})),
        ("Left", json!({"preset": "left"})),
        ("Right", json!({"preset": "right"})),
    ] {
        mi(ui, app, label, "layout.columns", params);
    }
    mi(ui, app, "More Columns…", "ui.dialog", json!({"name": "columns"}));
}

/// A row of buttons that wraps onto the next line in a narrow pane.
fn buttons(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(1.0, 2.0);
        add(ui);
    });
}

/// A label on the left and a field on the right.
fn field(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(tl!(label)).small());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add);
    });
}

/// The Text pane: font, character formatting, colour and effects.
pub fn text(app: &mut WordApp, ui: &mut Ui) {
    let st = app.session.run("format.state", &json!({})).unwrap_or_default();
    let flag = |k: &str| st.get(k).and_then(Value::as_bool).unwrap_or(false);
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        section(ui, "font", "Font", |ui| {
            let font = st.get("font").and_then(Value::as_str).unwrap_or("").to_string();
            let warning = app.font_box_warning(&font);
            let font = app.previews.font_shown(&font);
            let fams = app.previews.families();
            let prev = app.previews.font_preview_fn();
            let w = ui.available_width();
            if let Some(f) = crate::widgets::font_combo_warned(ui, "pane_font", w, &font, &fams, Some(&*prev), warning.as_deref()) {
                let _ = if f == crate::widgets::ADD_FONT {
                    app.run("font.add", json!({}))
                } else if f == crate::widgets::LOAD_SYSTEM_FONTS {
                    app.run("font.loadSystem", json!({}))
                } else {
                    let f = app.previews.font_stored(&f);
                    app.run("format.font", json!({"name": f}))
                };
            }
            ui.horizontal(|ui| {
                let size = st
                    .get("size")
                    .and_then(Value::as_f64)
                    .map(|s| if s.fract() == 0.0 { format!("{s:.0}") } else { format!("{s}") })
                    .unwrap_or_default();
                let sizes: Vec<String> =
                    doc4me_engine::cmd::format::SIZES.iter().map(|s| if s.fract() == 0.0 { format!("{s:.0}") } else { s.to_string() }).collect();
                if let Some(v) = combo(ui, "pane_size", 64.0, &size, &sizes, None)
                    && let Ok(x) = v.trim().parse::<f64>()
                {
                    let _ = app.run("format.size", json!({"size": x}));
                }
                small(ui, app, "grow", None, "Increase Font Size", "format.growFont", json!({}), false);
                small(ui, app, "shrink", None, "Decrease Font Size", "format.shrinkFont", json!({}), false);
                menu_button(ui, app, "case", None, "Change Case", false, "format.changeCase", |ui, app| {
                    mi(ui, app, "Sentence case.", "format.changeCase", json!({"mode": "sentence"}));
                    mi(ui, app, "lowercase", "format.changeCase", json!({"mode": "lower"}));
                    mi(ui, app, "UPPERCASE", "format.changeCase", json!({"mode": "upper"}));
                    mi(ui, app, "Capitalize Each Word", "format.changeCase", json!({"mode": "title"}));
                    mi(ui, app, "tOGGLE cASE", "format.changeCase", json!({"mode": "toggle"}));
                });
            });
        });
        section(ui, "style", "Style", |ui| {
            buttons(ui, |ui| {
                small(ui, app, "bold", None, "Bold", "format.bold", json!({}), flag("bold"));
                small(ui, app, "italic", None, "Italic", "format.italic", json!({}), flag("italic"));
                split(ui, app, "underline", "Underline", "format.underline", json!({}), flag("underline"), None, |ui, app| {
                    for (l, s) in
                        [("Single", "single"), ("Double", "double"), ("Thick", "thick"), ("Dotted", "dotted"), ("Dashed", "dash"), ("Wave", "wave")]
                    {
                        underline_sample(ui, app, l, s);
                    }
                    mi(ui, app, "Words only", "format.underline", json!({"style": "words"}));
                });
                small(ui, app, "strike", None, "Strikethrough", "format.strikethrough", json!({}), flag("strike"));
                small(ui, app, "subscript", None, "Subscript", "format.subscript", json!({}), flag("subscript"));
                small(ui, app, "superscript", None, "Superscript", "format.superscript", json!({}), flag("superscript"));
                small(ui, app, "clear", None, "Clear All Formatting", "format.clear", json!({}), false);
            });
        });
        section(ui, "colour", "Color", |ui| {
            buttons(ui, |ui| {
                let fc = app.canvas.last_font_color.clone();
                let sw = doc4me_doc::Rgb::parse(&fc).map(crate::theme::c32);
                split(ui, app, "fontcolor", "Font Color", "format.color", json!({"color": fc}), false, sw, |ui, app| {
                    mi(ui, app, "Automatic", "format.color", json!({"color": "auto"}));
                    let theme = app.session.doc.settings.theme_colors.clone();
                    crate::colors::menu(ui, app, &theme, Target::cmd("format.color").remember(Remember::FontColor));
                });
                let hl = app.canvas.last_highlight.clone();
                split(ui, app, "highlight", "Text Highlight Color", "format.highlight", json!({"color": hl}), false, None, |ui, app| {
                    for h in doc4me_doc::props::Highlight::ALL.iter().skip(1) {
                        if ui.button(h.name()).clicked() {
                            app.canvas.last_highlight = h.ooxml().to_string();
                            let _ = app.run("format.highlight", json!({"color": h.ooxml()}));
                            ui.close();
                        }
                    }
                    mi(ui, app, "No Color", "format.highlight", json!({"color": "none"}));
                });
            });
        });
        section(ui, "effects", "Effects", |ui| {
            buttons(ui, |ui| {
                small(ui, app, "charborder", None, "Character Border", "format.border", json!({}), flag("border"));
                small(ui, app, "thaiDigits", None, "Thai Digits", "format.thaiDigits", json!({}), false);
                menu_button(ui, app, "char-spacing", None, "Character Spacing", false, "", |ui, app| {
                    for (label, points) in [("Very Tight", -1.5), ("Tight", -0.5), ("Normal", 0.0), ("Loose", 1.0), ("Very Loose", 3.0)] {
                        mi(ui, app, label, "format.spacing", json!({"points": points}));
                    }
                });
                menu_button(ui, app, "effects", None, "Text Effects and Typography", false, "", |ui, app| {
                    mi(ui, app, "Outline", "format.outline", json!({}));
                    mi(ui, app, "Shadow", "format.shadow", json!({}));
                    mi(ui, app, "Small Caps", "format.smallCaps", json!({}));
                    mi(ui, app, "All Caps", "format.allCaps", json!({}));
                    mi(ui, app, "Double Strikethrough", "format.doubleStrikethrough", json!({}));
                });
            });
            ui.add_space(2.0);
            if ui.button(tl!("Font…")).clicked() {
                let _ = app.run("format.fontDialog", json!({}));
            }
        });
    });
}

/// The Paragraph pane: alignment, lists, spacing, indents, borders and text flow.
pub fn paragraph(app: &mut WordApp, ui: &mut Ui) {
    let rp = app.session.doc.para_at(&app.session.sel.focus).map(|p| app.session.doc.styles.resolve_para(&p.props));
    let Some(rp) = rp else { return };
    let align = rp.align.visual(rp.bidi);
    let lists = doc4me_engine::cmd::para::list_buttons(&app.session.doc, rp.numbering);
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        section(ui, "align", "Alignment", |ui| {
            use doc4me_doc::Align as A;
            buttons(ui, |ui| {
                small(ui, app, "alignLeft", None, "Align Left", "para.alignLeft", json!({}), align == A::Left);
                small(ui, app, "alignCenter", None, "Center", "para.alignCenter", json!({}), align == A::Center);
                small(ui, app, "alignRight", None, "Align Right", "para.alignRight", json!({}), align == A::Right);
                small(ui, app, "justify", None, "Justify", "para.justify", json!({}), align == A::Justify);
                small(ui, app, "thaiDistribute", None, "Thai Distribution", "para.thaiDistribute", json!({}), align == A::ThaiDistribute);
            });
        });
        section(ui, "lists", "Lists", |ui| {
            buttons(ui, |ui| {
                split(ui, app, "bullets", "Bullets", "para.bullets", json!({}), lists.bullets, None, |ui, app| {
                    for c in ["•", "○", "▪", "◆", "✓", "–"] {
                        if ui.button(egui::RichText::new(c).size(16.0)).clicked() {
                            let _ = app.run("para.bullets", json!({"kind": c}));
                            ui.close();
                        }
                    }
                    mi(ui, app, "None", "para.bullets", json!({"off": true}));
                    bullet_list_items(ui, app);
                });
                split(ui, app, "numbering", "Numbering", "para.numbering", json!({}), lists.numbering, None, |ui, app| {
                    for (l, k) in [("1. 2. 3.", "numbered"), ("1) 2) 3)", "numberedParen"), ("A. B. C.", "upperLetter"), ("a) b) c)", "lowerLetter")]
                    {
                        mi(ui, app, l, "para.numbering", json!({"kind": k}));
                    }
                    ui.separator();
                    for (l, k) in [("๑. ๒. ๓.", "thaiNumbers"), ("ก. ข. ค.", "thaiLetters"), ("หนึ่ง. สอง. สาม.", "thaiCounting")]
                    {
                        mi(ui, app, l, "para.numbering", json!({"kind": k}));
                    }
                    ui.separator();
                    mi(ui, app, "Restart at 1", "para.restartNumbering", json!({}));
                    mi(ui, app, "None", "para.numbering", json!({"off": true}));
                });
                split(ui, app, "multilevel", "Multilevel List", "para.multilevel", json!({}), lists.multilevel, None, |ui, app| {
                    mi(ui, app, "1. 1.1. 1.1.1.", "para.multilevel", json!({"kind": "legal"}));
                    mi(ui, app, "I. A. 1. a.", "para.multilevel", json!({"kind": "outline"}));
                    mi(ui, app, "๑. ๑.๑. ๑.๑.๑.", "para.multilevel", json!({"kind": "thaiLegal"}));
                    mi(ui, app, "๑. ก. (๑) (ก)", "para.multilevel", json!({"kind": "thaiOutline"}));
                    saved_list_items(ui, app);
                });
                small(ui, app, "outdent", None, "Decrease Indent", "para.outdent", json!({}), false);
                small(ui, app, "indent", None, "Increase Indent", "para.indent", json!({}), false);
                small(ui, app, "sort", None, "Sort", "para.sort", json!({}), false);
            });
        });
        let u = crate::options::unit();
        let k = u.pt_per_unit();
        const FIELD: egui::Vec2 = egui::Vec2::new(80.0, 20.0);
        section(ui, "spacing", "Spacing", |ui| {
            use doc4me_doc::props::LineSpacing as L;
            let line = match rp.line_spacing {
                L::Multiple(m) => format!("{m}"),
                L::AtLeast(p) => format!("≥ {p} pt"),
                L::Exactly(p) => format!("{p} pt"),
            };
            field(ui, "Line spacing", |ui| {
                let items: Vec<String> = ["1", "1.15", "1.5", "2", "2.5", "3"].iter().map(|s| s.to_string()).collect();
                if let Some(v) = combo(ui, "pane_line_spacing", FIELD.x, &line, &items, None)
                    && let Ok(x) = v.trim().parse::<f64>()
                {
                    let _ = app.run("para.lineSpacing", json!({"value": x}));
                }
            });
            let (mut b, mut a) = (rp.space_before, rp.space_after);
            field(ui, "Before paragraph", |ui| {
                if ui
                    .add_sized(FIELD, crate::numfield::NumField::new(&mut b).speed(1.0).range(0.0..=1584.0).suffix(format!(" {}", tl!("pt"))))
                    .changed()
                {
                    let _ = app.run("para.spacing", json!({"before": b}));
                }
            });
            field(ui, "After paragraph", |ui| {
                if ui
                    .add_sized(FIELD, crate::numfield::NumField::new(&mut a).speed(1.0).range(0.0..=1584.0).suffix(format!(" {}", tl!("pt"))))
                    .changed()
                {
                    let _ = app.run("para.spacing", json!({"after": a}));
                }
            });
        });
        section(ui, "indents", "Indents", |ui| {
            for (label, key, value) in
                [("Left", "left", rp.indent_left), ("Right", "right", rp.indent_right), ("First line", "firstLine", rp.indent_first)]
            {
                let mut v = value / k;
                field(ui, label, |ui| {
                    if ui
                        .add_sized(
                            FIELD,
                            crate::numfield::NumField::new(&mut v).speed(3.6 / k).range(-792.0 / k..=1584.0 / k).measure(u).max_decimals(2),
                        )
                        .changed()
                    {
                        let _ = app.run("para.indents", json!({ key: v * k }));
                    }
                });
            }
        });
        section(ui, "borders", "Borders & Shading", |ui| {
            buttons(ui, |ui| {
                let swatch = crate::colors::parse(&app.canvas.last_shading).map(|c| egui::Color32::from_rgb(c.0, c.1, c.2));
                split(ui, app, "shading", "Shading", "para.shading", json!({"color": app.canvas.last_shading.clone()}), false, swatch, |ui, app| {
                    mi(ui, app, "No Color", "para.shading", json!({"color": null}));
                    let theme = app.session.doc.settings.theme_colors.clone();
                    crate::colors::menu(ui, app, &theme, Target::cmd("para.shading").remember(Remember::Shading));
                });
                let cmd = if crate::widgets::enabled(app, "table.borders") { "table.borders" } else { "para.borders" };
                split(ui, app, "borders", "Borders", cmd, json!({"kind": "bottom"}), false, None, border_items);
            });
        });
        section(ui, "flow", "Text Flow", |ui| {
            for (label, id, on) in [
                ("Widow/Orphan control", "para.widowControl", rp.widow_control),
                ("Keep with next", "para.keepNext", rp.keep_next),
                ("Keep lines together", "para.keepLines", rp.keep_lines),
                ("Page break before", "para.pageBreakBefore", rp.page_break_before),
            ] {
                let mut v = on;
                if ui.checkbox(&mut v, tl!(label)).changed() {
                    let _ = app.run(id, json!({"value": v}));
                }
            }
            ui.add_space(2.0);
            if ui.button(tl!("Paragraph…")).clicked() {
                let _ = app.run("para.dialog", json!({}));
            }
        });
    });
}

/// The Page Setup pane: paper, margins, columns and breaks, header and footer.
pub fn page(app: &mut WordApp, ui: &mut Ui) {
    let sect = doc4me_engine::cmd::page::sect(&app.session);
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        section(ui, "paper", "Paper", |ui| {
            buttons(ui, |ui| {
                menu_button(ui, app, "size", Some("Size"), "Size", true, "layout.size", size_menu);
                menu_button(ui, app, "orientation", Some("Orientation"), "Orientation", true, "layout.orientation", orientation_items);
            });
        });
        let u = crate::options::unit();
        let k = u.pt_per_unit();
        section(ui, "margins", "Margins", |ui| {
            for (label, key, value) in [
                ("Top", "top", sect.margin_top),
                ("Bottom", "bottom", sect.margin_bottom),
                ("Left", "left", sect.margin_left),
                ("Right", "right", sect.margin_right),
            ] {
                let mut v = value / k;
                field(ui, label, |ui| {
                    if ui
                        .add_sized(
                            vec2(80.0, 20.0),
                            crate::numfield::NumField::new(&mut v).speed(3.6 / k).range(0.0..=1584.0 / k).measure(u).max_decimals(2),
                        )
                        .changed()
                    {
                        let _ = app.run("layout.margins", json!({ key: v * k }));
                    }
                });
            }
            buttons(ui, |ui| {
                menu_button(ui, app, "margins", Some("Presets"), "Margins", true, "layout.margins", margins_items);
            });
        });
        section(ui, "columns", "Columns & Breaks", |ui| {
            buttons(ui, |ui| {
                menu_button(ui, app, "columns", Some("Columns"), "Columns", true, "layout.columns", columns_items);
                menu_button(ui, app, "breaks", Some("Breaks"), "Breaks", true, "layout.break", |ui, app| {
                    for (l, k) in [("Page", "page"), ("Column", "column"), ("Next Page", "nextPage"), ("Continuous", "continuous")] {
                        mi(ui, app, l, "layout.break", json!({"kind": k}));
                    }
                });
            });
        });
        section(ui, "hf", "Header & Footer", |ui| {
            let mut first = sect.title_page;
            if ui.checkbox(&mut first, tl!("Different first page")).changed() {
                let _ = app.run("layout.differentFirstPage", json!({"value": first}));
            }
            buttons(ui, |ui| {
                small(ui, app, "header", Some("Edit Header"), "Edit Header", "insert.header", json!({}), false);
            });
        });
        ui.add_space(4.0);
        if ui.button(tl!("Page Setup…")).clicked() {
            let _ = app.run("ui.dialog", json!({"name": "pageSetup"}));
        }
    });
}

/// The Page panel's Design tab: the theme, the document's style set and the page background
/// (the Design ribbon tab, as a pane).
pub fn design(app: &mut WordApp, ui: &mut Ui) {
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        section(ui, "theme", "Theme", |ui| {
            buttons(ui, |ui| {
                menu_button(ui, app, "themes", Some("Themes"), "Themes", true, "design.themes", |ui, app| {
                    for (name, ..) in doc4me_engine::cmd::design::THEMES {
                        mi(ui, app, name, "design.theme", json!({"name": name}));
                    }
                });
                menu_button(ui, app, "colors", Some("Colors"), "Theme Colors", true, "design.themeColors", |ui, app| {
                    for (name, ..) in doc4me_engine::cmd::design::THEMES {
                        mi(ui, app, name, "design.themeColors", json!({"name": name}));
                    }
                });
                menu_button(ui, app, "fonts", Some("Fonts"), "Theme Fonts", true, "design.themeFonts", |ui, app| {
                    for (name, h, b, _) in doc4me_engine::cmd::design::THEMES {
                        crate::ribbon::mi_text(ui, app, &format!("{}: {h} / {b}", tl!(name)), "design.themeFonts", json!({"heading": h, "body": b}));
                    }
                });
            });
        });
        section(ui, "styleSet", "Style Set", |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(1.0, 1.0);
                crate::previews::style_set_gallery(app, ui);
            });
            buttons(ui, |ui| {
                menu_button(ui, app, "paraSpacing", Some("Paragraph Spacing"), "Paragraph Spacing", true, "design.paragraphSpacing", |ui, app| {
                    for (l, v) in [
                        ("Default", "default"),
                        ("No Paragraph Space", "none"),
                        ("Compact", "compact"),
                        ("Tight", "tight"),
                        ("Open", "open"),
                        ("Relaxed", "relaxed"),
                        ("Double", "double"),
                    ] {
                        mi(ui, app, l, "design.paragraphSpacing", json!({"value": v}));
                    }
                });
                small(ui, app, "effectsDesign", Some("Effects"), "Effects", "design.effects", json!({}), false);
                small(ui, app, "setDefault", Some("Set as Default"), "Set as Default", "design.setDefault", json!({}), false);
            });
        });
        section(ui, "background", "Page Background", |ui| {
            buttons(ui, |ui| {
                menu_button(ui, app, "watermark", Some("Watermark"), "Watermark", true, "design.watermark", |ui, app| {
                    for w in ["DRAFT", "CONFIDENTIAL", "DO NOT COPY", "SAMPLE", "ASAP", "URGENT"] {
                        mi(ui, app, w, "design.watermark", json!({"text": w}));
                    }
                    ui.separator();
                    mi(ui, app, "Custom Watermark…", "ui.dialog", json!({"name": "watermark"}));
                    mi(ui, app, "Remove Watermark", "design.watermark", json!({"remove": true}));
                });
                menu_button(ui, app, "pageColor", Some("Page Color"), "Page Color", true, "design.pageColor", |ui, app| {
                    mi(ui, app, "No Color", "design.pageColor", json!({"color": null}));
                    let theme = app.session.doc.settings.theme_colors.clone();
                    crate::colors::menu(ui, app, &theme, Target::cmd("design.pageColor"));
                });
                small(ui, app, "pageBorders", Some("Page Borders"), "Page Borders", "design.pageBorders", json!({}), false);
            });
        });
    });
}
