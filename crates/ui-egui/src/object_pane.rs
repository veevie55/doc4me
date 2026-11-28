//! Optional selection-aware controls for pictures, shapes, charts and SmartArt. The pane reuses
//! the same command IDs and object models as Doc4Me's contextual ribbon tabs.

use egui::{Ui, vec2};
use serde_json::json;

use crate::WordApp;
use crate::colors::Target;
use crate::options::MeasureExt;
use crate::ribbon::mi;

#[derive(Clone, Copy)]
enum Kind {
    Shape,
    Picture,
    Chart,
    SmartArt,
}

pub fn show(app: &mut WordApp, ui: &mut Ui) {
    if !app.session.view.object_pane {
        return;
    }
    let kind = if doc4me_engine::cmd::smart_art::selected_smart_art(&app.session).is_some() {
        Some(Kind::SmartArt)
    } else {
        match doc4me_engine::cmd::objects::selected(&app.session) {
            Some((_, doc4me_doc::para::InlineObject::Image { .. })) => Some(Kind::Picture),
            Some((_, doc4me_doc::para::InlineObject::Graphic { graphic, .. })) if graphic.chart.is_some() => Some(Kind::Chart),
            Some((_, doc4me_doc::para::InlineObject::Shape { .. })) => Some(Kind::Shape),
            _ => None,
        }
    };
    let title = match kind {
        Some(Kind::Shape) => "Shape",
        Some(Kind::Picture) => "Picture",
        Some(Kind::Chart) => "Chart",
        Some(Kind::SmartArt) => "SmartArt",
        None => "Object Formatting",
    };
    let t = crate::theme::Tokens::get(ui.ctx());
    egui::Panel::right("object_context_pane")
        .default_size(310.0)
        .resizable(true)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(10.0).stroke(egui::Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            crate::ribbon_keys::stop(ui, crate::ribbon_keys::Region::Pane, ui.max_rect());
            if crate::panes::header(ui, title) {
                let _ = app.run("view.objectPane", json!({"value": false}));
                return;
            }
            let Some(kind) = kind else {
                ui.label(tl!("Select a picture, shape, chart or SmartArt graphic to edit its formatting."));
                return;
            };
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match kind {
                Kind::Shape => shape(app, ui),
                Kind::Picture => picture(app, ui),
                Kind::Chart => chart(app, ui),
                Kind::SmartArt => smart_art(app, ui),
            });
        });
}

fn section(ui: &mut Ui, name: &str, add: impl FnOnce(&mut Ui)) {
    ui.collapsing(tl!(name), add);
}

fn row(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
        add(ui);
    });
}

fn shape(app: &mut WordApp, ui: &mut Ui) {
    let selected = doc4me_engine::cmd::objects::selected(&app.session).map(|(_, object)| object.clone());
    let Some(doc4me_doc::para::InlineObject::Shape { fill, stroke, stroke_width, effects, extra, kind, w, h, .. }) = selected else { return };
    let theme = app.session.doc.settings.theme_colors.clone();
    let locked = doc4me_engine::cmd::objects::selection_locked(&app.session);

    section(ui, "Shape Styles", |ui| {
        row(ui, |ui| {
            crate::widgets::menu_button(ui, app, "shading", Some("Shape Fill"), "Shape Fill", false, "shape.fill", |ui, app| {
                mi(ui, app, "No Color", "shape.fill", json!({"color": null}));
                crate::colors::menu(ui, app, &theme, Target::cmd("shape.fill"));
                ui.separator();
                for percent in [0, 25, 50, 75, 100] {
                    mi(ui, app, &format!("{percent}% Transparency"), "shape.fill", json!({"transparency": percent}));
                }
            });
            crate::widgets::menu_button(ui, app, "pen", Some("Shape Outline"), "Shape Outline", false, "shape.outline", |ui, app| {
                mi(ui, app, "No Color", "shape.outline", json!({"color": null}));
                crate::colors::menu(ui, app, &theme, Target::cmd("shape.outline"));
                ui.separator();
                for width in [0.25, 0.5, 0.75, 1.0, 1.5, 2.25, 3.0, 4.5, 6.0] {
                    mi(ui, app, &format!("{width} pt"), "shape.outline", json!({"width": width}));
                }
                for (label, dash) in [("Solid", "solid"), ("Dot", "dot"), ("Dash", "dash"), ("Long Dash", "longDash"), ("Dash Dot", "dashDot")] {
                    mi(ui, app, label, "shape.outline", json!({"dash": dash}));
                }
                let color = stroke.unwrap_or(doc4me_doc::Rgb::BLACK).hex();
                mi(
                    ui,
                    app,
                    if extra.arrow_start { "Remove Start Arrowhead" } else { "Add Start Arrowhead" },
                    "shape.outline",
                    json!({"color": color, "arrowStart": !extra.arrow_start}),
                );
                mi(
                    ui,
                    app,
                    if extra.arrow_end { "Remove End Arrowhead" } else { "Add End Arrowhead" },
                    "shape.outline",
                    json!({"color": color, "arrowEnd": !extra.arrow_end}),
                );
            });
            crate::widgets::menu_button(
                ui,
                app,
                "shapeEffects",
                Some("Shape Effects"),
                "Shadow, glow and soft edges",
                false,
                "shape.effects",
                |ui, app| {
                    mi(ui, app, "No Shadow", "shape.effects", json!({"shadow": null}));
                    for (id, label) in doc4me_doc::effects::SHADOW_PRESETS {
                        mi(ui, app, label, "shape.effects", json!({"shadow": id}));
                    }
                    ui.separator();
                    mi(ui, app, "No Glow", "shape.effects", json!({"glow": null}));
                    for size in [5.0, 8.0, 11.0, 18.0] {
                        mi(ui, app, &format!("Glow {size} pt"), "shape.effects", json!({"glow": size}));
                    }
                    let glow_size = effects.glow.map_or(8.0, |glow| glow.size.max(0.0));
                    ui.menu_button(tl!("Glow Colors"), |ui| {
                        let target = Target::at("shape.effects", json!({"glow": {"size": glow_size}}), "/glow/color");
                        crate::colors::menu(ui, app, &theme, target);
                    });
                    ui.separator();
                    for radius in [1.0, 2.5, 5.0, 10.0, 25.0, 50.0] {
                        mi(ui, app, &format!("Soft Edge {radius} pt"), "shape.effects", json!({"softEdge": radius}));
                    }
                },
            );
        });
        if doc4me_engine::cmd::canvas::selected_canvas(&app.session).is_some() {
            row(ui, |ui| {
                crate::widgets::small(ui, app, "canvas", Some("Fit Canvas"), "Fit Drawing Canvas", "canvas.fit", json!({}), false);
                crate::widgets::small(ui, app, "canvas", Some("Expand"), "Expand Drawing Canvas", "canvas.expand", json!({}), false);
                crate::widgets::small(ui, app, "altText", Some("Alt Text"), "Alt Text", "view.altTextPane", json!({"value": true}), false);
            });
        }
        ui.label(
            egui::RichText::new(format!(
                "{} · {} · {} pt",
                fill.map_or_else(|| tl!("No Fill").to_string(), |color| color.hex()),
                stroke.map_or_else(|| tl!("No Line").to_string(), |color| color.hex()),
                stroke_width
            ))
            .small()
            .weak(),
        );
    });

    section(ui, "Size and Position", |ui| {
        let unit = crate::options::unit();
        let factor = unit.pt_per_unit();
        for (label, key, initial) in [("Width", "width", w), ("Height", "height", h)] {
            let mut value = initial / factor;
            ui.horizontal(|ui| {
                ui.label(tl!(label));
                if ui
                    .add(crate::numfield::NumField::new(&mut value).speed(1.0 / factor).range(0.05..=2000.0 / factor).measure(unit).max_decimals(2))
                    .changed()
                {
                    let _ = app.run("picture.size", json!({key: value * factor, "lockAspect": false}));
                }
            });
        }
        ui.horizontal(|ui| {
            crate::ribbon::rotation_field(ui, app);
        });
        row(ui, |ui| {
            crate::widgets::small(ui, app, "position", Some("Position"), "Position", "arrange.position", json!({}), false);
            crate::ribbon::wrap_menu(ui, app);
            crate::widgets::small(ui, app, "align", Some("Align"), "Align", "arrange.align", json!({"value": "center"}), false);
            crate::ribbon::rotate_menu(ui, app, Some("Rotate"));
            crate::widgets::small(
                ui,
                app,
                if locked { "lock" } else { "unlock" },
                Some(if locked { "Unlock" } else { "Lock" }),
                "Lock selected object",
                "shape.lock",
                json!({"locked": !locked}),
                locked,
            );
        });
        if kind == doc4me_doc::para::ShapeKind::RoundedRectangle {
            let mut radius = doc4me_doc::connector::corner_radius(extra.corner, w, h) / factor;
            ui.horizontal(|ui| {
                ui.label(tl!("Corner Radius"));
                if ui.add(crate::numfield::NumField::new(&mut radius).range(0.0..=w.min(h) / 2.0 / factor).measure(unit).max_decimals(2)).changed() {
                    let _ = app.run("shape.cornerRadius", json!({"radius": radius * factor}));
                }
            });
        }
    });

    section(ui, "Text", |ui| {
        row(ui, |ui| {
            crate::widgets::menu_button(
                ui,
                app,
                "textDirection",
                Some("Text Direction"),
                "Text Direction",
                false,
                "shape.textDirection",
                |ui, app| {
                    for (label, dir) in [("Horizontal", "horz"), ("Rotate 90°", "vert"), ("Rotate 270°", "vert270"), ("Stacked", "wordArtVert")] {
                        mi(ui, app, label, "shape.textDirection", json!({"dir": dir}));
                    }
                },
            );
            crate::widgets::menu_button(ui, app, "alignText", Some("Align Text"), "Align Text", false, "shape.alignText", |ui, app| {
                for (label, anchor) in [("Top", "t"), ("Middle", "ctr"), ("Bottom", "b")] {
                    mi(ui, app, label, "shape.alignText", json!({"anchor": anchor}));
                }
            });
            crate::widgets::small(ui, app, "altText", Some("Alt Text"), "Alt Text", "view.altTextPane", json!({"value": true}), false);
        });
        if doc4me_engine::cmd::objects::text_box_target(&app.session).is_some() {
            row(ui, |ui| {
                crate::widgets::small(ui, app, "textBoxLink", Some("Create Link"), "Create Link", "ui.linkTextBox", json!({}), false);
                crate::widgets::small(ui, app, "textBoxUnlink", Some("Break Link"), "Break Link", "shape.breakLink", json!({}), false);
            });
        }
    });
    section(ui, "WordArt Styles", |ui| {
        crate::widgets::menu_button(ui, app, "wordArt", Some("Styles"), "WordArt Styles", false, "", |ui, app| {
            crate::wordart_menus::style_gallery(ui, app, "wordArt.style");
        });
        row(ui, |ui| {
            crate::widgets::menu_button(ui, app, "textFill", Some("Text Fill"), "Text Fill", false, "", |ui, app| {
                crate::wordart_menus::fill_menu(ui, app, &theme);
            });
            let outline = crate::wordart_menus::current_outline(app);
            crate::widgets::menu_button(ui, app, "textOutline", Some("Text Outline"), "Text Outline", false, "", |ui, app| {
                crate::wordart_menus::outline_menu(ui, app, &theme, outline);
            });
            crate::widgets::menu_button(ui, app, "transform", Some("Text Effects"), "Text Effects", false, "", |ui, app| {
                crate::wordart_menus::effects_menu(ui, app, &theme, true);
            });
        });
    });
}

fn picture(app: &mut WordApp, ui: &mut Ui) {
    let selected = doc4me_engine::cmd::objects::selected(&app.session).map(|(_, object)| object.clone());
    let Some(doc4me_doc::para::InlineObject::Image { media, w, h, crop, alt, .. }) = selected else { return };
    let theme = app.session.doc.settings.theme_colors.clone();

    section(ui, "Adjust", |ui| {
        row(ui, |ui| {
            crate::widgets::menu_button(
                ui,
                app,
                "picture",
                Some("Corrections"),
                "Brightness, contrast and sharpness",
                false,
                "picture.corrections",
                |ui, app| {
                    for (label, brightness, contrast) in [
                        ("Brighter +20%", 20.0, 0.0),
                        ("Darker −20%", -20.0, 0.0),
                        ("More Contrast +20%", 0.0, 20.0),
                        ("Less Contrast −20%", 0.0, -20.0),
                    ] {
                        mi(ui, app, label, "picture.corrections", json!({"brightness": brightness, "contrast": contrast}));
                    }
                    mi(ui, app, "Sharpen", "picture.corrections", json!({"sharpen": 40.0}));
                    mi(ui, app, "Soften", "picture.corrections", json!({"sharpen": -40.0}));
                },
            );
            crate::widgets::menu_button(ui, app, "picture", Some("Color"), "Saturation and recolor", false, "picture.color", |ui, app| {
                for (label, value) in [("Full Saturation 100%", 100.0), ("Muted 50%", 50.0), ("Gray 0%", 0.0), ("Vivid 200%", 200.0)] {
                    mi(ui, app, label, "picture.color", json!({"mode": "saturation", "saturation": value}));
                }
                mi(ui, app, "Grayscale", "picture.color", json!({"mode": "grayscale"}));
                mi(ui, app, "Sepia", "picture.color", json!({"mode": "sepia"}));
                mi(ui, app, "Washout", "picture.color", json!({"mode": "washout"}));
            });
            crate::widgets::menu_button(
                ui,
                app,
                "picture",
                Some("Transparency"),
                "Picture transparency",
                false,
                "picture.transparency",
                |ui, app| {
                    for value in [0.0, 15.0, 30.0, 50.0, 65.0, 80.0, 95.0] {
                        mi(ui, app, &format!("{value:.0}%"), "picture.transparency", json!({"percent": value}));
                    }
                },
            );
        });
        row(ui, |ui| {
            crate::widgets::menu_button(
                ui,
                app,
                "picture",
                Some("Icon Color"),
                "Recolor a one-color picture, such as an icon",
                false,
                "picture.iconColor",
                |ui, app| {
                    crate::colors::menu(ui, app, &theme, Target::cmd("picture.iconColor"));
                },
            );
            crate::widgets::menu_button(ui, app, "picture", Some("Picture Style"), "Frames and effects", false, "picture.style", |ui, app| {
                for (label, style) in [
                    ("Simple Frame", "simpleFrame"),
                    ("Thick Frame", "thickFrame"),
                    ("Rounded", "rounded"),
                    ("Soft Edge", "softEdge"),
                    ("Shadow", "shadow"),
                ] {
                    mi(ui, app, label, "picture.style", json!({"style": style}));
                }
            });
            crate::widgets::menu_button(ui, app, "picture", Some("Border"), "Picture border", false, "picture.border", |ui, app| {
                for (label, width) in [("Thin", 2), ("Medium", 4), ("Thick", 8)] {
                    mi(ui, app, label, "picture.border", json!({"width": width}));
                }
                crate::colors::menu(ui, app, &theme, Target::cmd("picture.border"));
            });
        });
        row(ui, |ui| {
            crate::widgets::small(ui, app, "picture", Some("Change"), "Change Picture", "ui.changePicture", json!({}), false);
            crate::widgets::small(ui, app, "picture", Some("Reset"), "Reset Picture", "picture.reset", json!({}), false);
            crate::widgets::small(ui, app, "altText", Some("Alt Text"), "Alt Text", "view.altTextPane", json!({"value": true}), false);
            crate::widgets::small(ui, app, "crop", Some("Crop"), "Crop", "ui.crop", json!({}), false);
        });
    });

    section(ui, "Size and Position", |ui| {
        let unit = crate::options::unit();
        let factor = unit.pt_per_unit();
        for (label, key, initial) in [("Width", "width", w), ("Height", "height", h)] {
            let mut value = initial / factor;
            ui.horizontal(|ui| {
                ui.label(tl!(label));
                if ui
                    .add(crate::numfield::NumField::new(&mut value).speed(1.0 / factor).range(0.05..=2000.0 / factor).measure(unit).max_decimals(2))
                    .changed()
                {
                    let _ = app.run("picture.size", json!({key: value * factor}));
                }
            });
        }
        row(ui, |ui| {
            crate::widgets::small(ui, app, "position", Some("Position"), "Position", "arrange.position", json!({}), false);
            crate::ribbon::wrap_menu(ui, app);
            crate::ribbon::rotate_menu(ui, app, Some("Rotate"));
            crate::widgets::small(ui, app, "align", Some("Align"), "Align", "arrange.align", json!({"value": "center"}), false);
        });
        section(ui, "Crop", |ui| {
            for (label, index) in [("Left", 0), ("Top", 1), ("Right", 2), ("Bottom", 3)] {
                let mut value = crop[index] * 100.0;
                ui.horizontal(|ui| {
                    ui.label(tl!(label));
                    if ui.add(egui::DragValue::new(&mut value).range(0.0..=45.0).suffix("%")).changed() {
                        let mut values = crop;
                        values[index] = (value / 100.0).clamp(0.0, 0.45);
                        let _ = app.run("picture.crop", json!({"left": values[0], "top": values[1], "right": values[2], "bottom": values[3]}));
                    }
                });
            }
            if ui.button(tl!("Reset Crop")).clicked() {
                let _ = app.run("picture.crop", json!({"left": 0.0, "top": 0.0, "right": 0.0, "bottom": 0.0}));
            }
        });
    });
    let _ = (media, alt);
}

fn chart(app: &mut WordApp, ui: &mut Ui) {
    let Some(chart) = doc4me_engine::cmd::chart::selected_chart(&app.session) else { return };
    section(ui, "Chart Layout", |ui| {
        row(ui, |ui| {
            crate::widgets::menu_button(ui, app, "chartTitle", Some("Chart Title"), "Chart Title", false, "chart.title", |ui, app| {
                mi(ui, app, "No Title", "chart.title", json!({"text": null}));
                mi(ui, app, "Above Chart", "chart.title", json!({"text": chart.title.clone().unwrap_or_else(|| tl!("Chart Title").to_string())}));
            });
            crate::widgets::menu_button(ui, app, "legend", Some("Legend"), "Legend", false, "chart.legend", |ui, app| {
                mi(ui, app, "No Legend", "chart.legend", json!({"pos": "none"}));
                for position in ["right", "top", "left", "bottom"] {
                    mi(ui, app, &format!("{} Legend", tl!(position)), "chart.legend", json!({"pos": position}));
                }
            });
            crate::widgets::small(
                ui,
                app,
                "dataLabels",
                Some(if chart.data_labels { "Hide Data Labels" } else { "Show Data Labels" }),
                "Data Labels",
                "chart.dataLabels",
                json!({"on": !chart.data_labels}),
                chart.data_labels,
            );
        });
        row(ui, |ui| {
            crate::widgets::small(ui, app, "chartData", Some("Edit Data"), "Edit Data", "chart.editData", json!({}), false);
            crate::widgets::small(ui, app, "changeChartType", Some("Change Chart Type"), "Change Chart Type", "chart.type", json!({}), false);
        });
    });
    section(ui, "Size", |ui| {
        if let Some((_, object)) = doc4me_engine::cmd::objects::selected(&app.session)
            && let Some((width, height, _)) = object.frame()
        {
            for (label, key, initial) in [("Width", "width", width), ("Height", "height", height)] {
                let mut value = initial;
                ui.horizontal(|ui| {
                    ui.label(tl!(label));
                    if ui.add(egui::DragValue::new(&mut value).range(24.0..=2000.0).suffix(" pt")).changed() {
                        let _ = app.run("picture.size", json!({key: value, "lockAspect": false}));
                    }
                });
            }
        }
    });
}

fn smart_art(app: &mut WordApp, ui: &mut Ui) {
    let Some(graphic) = doc4me_engine::cmd::smart_art::selected_smart_art(&app.session) else { return };
    let current = app.smart_art_item.filter(|index| *index < graphic.items.len());
    section(ui, "Create Graphic", |ui| {
        row(ui, |ui| {
            let params = current.map_or(json!({}), |index| json!({"after": index}));
            crate::widgets::small(ui, app, "addShape", Some("Add Shape"), "Add Shape", "smartArt.addShape", params, false);
            let params = current.map_or(json!({}), |index| json!({"index": index}));
            crate::widgets::small(ui, app, "promote", Some("Promote"), "Promote", "smartArt.promote", params.clone(), false);
            crate::widgets::small(ui, app, "demote", Some("Demote"), "Demote", "smartArt.demote", params, false);
            crate::widgets::small(
                ui,
                app,
                "textPane",
                Some("Text Pane"),
                "Text Pane",
                "smartArt.textPane",
                json!({}),
                app.session.view.smart_art_pane,
            );
        });
    });
    section(ui, "Layouts", |ui| {
        egui::Grid::new("object_pane_smart_art_layouts").num_columns(3).spacing(vec2(4.0, 4.0)).show(ui, |ui| {
            for (i, layout) in doc4me_doc::smart_art::SmartArtLayout::ALL.into_iter().enumerate() {
                if ui.selectable_label(graphic.layout == layout, tl!(layout.label())).clicked() && graphic.layout != layout {
                    let _ = app.run("smartArt.layout", json!({"layout": layout.id()}));
                }
                if i % 3 == 2 {
                    ui.end_row();
                }
            }
        });
    });
    section(ui, "SmartArt Styles", |ui| {
        crate::widgets::menu_button(ui, app, "smartArtColors", Some("Change Colors"), "Change Colors", false, "smartArt.colors", |ui, app| {
            for colors in doc4me_doc::smart_art::SmartArtColors::ALL {
                let label = crate::dialogs_smart_art::colors_label(colors);
                if ui.selectable_label(graphic.colors == colors, label).clicked() {
                    let _ = app.run("smartArt.colors", json!({"variant": colors.id()}));
                    ui.close();
                }
            }
        });
        row(ui, |ui| {
            crate::widgets::small(ui, app, "resetGraphic", Some("Reset Graphic"), "Reset Graphic", "smartArt.reset", json!({}), false);
        });
    });
}
