//! Optional contextual controls for tables. These use the same engine commands as the Table
//! Design and Table Layout ribbon tabs, so either surface edits the same document state.

use egui::{Ui, vec2};
use serde_json::json;

use crate::WordApp;
use crate::colors::{Remember, Target};
use crate::ribbon::mi;

pub fn show(app: &mut WordApp, ui: &mut Ui) {
    if !app.session.view.table_pane {
        return;
    }
    let t = crate::theme::Tokens::get(ui.ctx());
    egui::Panel::right("table_context_pane")
        .default_size(310.0)
        .resizable(true)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(10).stroke(egui::Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            crate::ribbon_keys::stop(ui, crate::ribbon_keys::Region::Pane, ui.max_rect());
            if crate::panes::header(ui, "Table") {
                let _ = app.run("view.tablePane", json!({"value": false}));
                return;
            }
            let id = egui::Id::new("table_context_pane_tab");
            let mut tab = ui.ctx().data_mut(|d| d.get_persisted::<usize>(id)).unwrap_or(0).min(2);
            ui.horizontal_wrapped(|ui| {
                for (i, title) in ["Styles", "Borders", "Rows and Columns"].into_iter().enumerate() {
                    if ui.selectable_value(&mut tab, i, tl!(title)).changed() {
                        ui.ctx().data_mut(|d| d.insert_persisted(id, tab));
                    }
                }
            });
            ui.separator();
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match tab {
                0 => styles(app, ui),
                1 => borders(app, ui),
                _ => rows_columns(app, ui),
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

fn styles(app: &mut WordApp, ui: &mut Ui) {
    let story = app.session.sel.focus.story;
    let table = app.session.sel.focus.path.cell().and_then(|(path, _, _)| app.session.doc.table(story, &path));
    let look = table.map(|t| t.props.look);
    let current_style = table.and_then(|t| t.props.style.clone());
    let in_table = table.is_some();
    let can_delete = current_style
        .as_ref()
        .and_then(|id| app.session.doc.styles.get(id))
        .is_some_and(|style| !doc4me_engine::cmd::table_style::is_builtin_table_style(style));

    section(ui, "Table Styles", |ui| {
        let mut choices: Vec<(bool, String, String)> = app
            .session
            .doc
            .styles
            .styles
            .iter()
            .filter(|style| style.kind == doc4me_doc::StyleKind::Table && !style.hidden)
            .map(|style| (style.builtin, style.id.clone(), crate::i18n::style_name(style)))
            .collect();
        choices.sort_by_key(|(builtin, _, _)| *builtin);
        egui::ScrollArea::horizontal().max_height(72.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                for (_, id, label) in &choices {
                    if in_table && crate::previews::table_style_tile(ui, app, id).on_hover_text(label).clicked() {
                        let _ = app.run("table.style", json!({"style": id}));
                    }
                }
            });
        });
        if !in_table {
            ui.label(tl!("Place the cursor in a table to apply a table style."));
        }
        row(ui, |ui| {
            crate::widgets::menu_button(ui, app, "styles", Some("More Styles"), "Table Styles", false, "table.style", |ui, app| {
                mi(ui, app, "New Table Style…", "ui.dialog", json!({"name": "newTableStyle"}));
                mi(ui, app, "Table AutoFormat…", "ui.dialog", json!({"name": "tableAutoFormat"}));
                if current_style.is_some() && ui.button(tl!("Modify Table Style…")).clicked() {
                    let _ = app.run("ui.dialog", json!({"name": "modifyTableStyle"}));
                    ui.close();
                }
                if ui.add_enabled(can_delete, egui::Button::new(tl!("Delete Table Style"))).clicked() {
                    let _ = app.run("table.deleteStyle", json!({}));
                    ui.close();
                }
            });
            crate::widgets::small(ui, app, "properties", Some("Properties"), "Table Properties", "table.properties", json!({}), false);
        });
    });

    section(ui, "Table Style Options", |ui| {
        let Some(look) = look else {
            ui.label(tl!("Place the cursor in a table to edit table style options."));
            return;
        };
        egui::Grid::new("table_context_look").num_columns(2).spacing(vec2(8.0, 4.0)).show(ui, |ui| {
            for options in [
                [("Header Row", "headerRow", look.header_row), ("First Column", "firstColumn", look.first_column)],
                [("Total Row", "totalRow", look.total_row), ("Last Column", "lastColumn", look.last_column)],
                [("Banded Rows", "bandedRows", look.banded_rows), ("Banded Columns", "bandedColumns", look.banded_columns)],
            ] {
                for (label, key, active) in options {
                    let mut value = active;
                    if ui.checkbox(&mut value, tl!(label)).changed() {
                        let _ = app.run("table.look", json!({key: value}));
                    }
                }
                ui.end_row();
            }
        });
    });
}

fn borders(app: &mut WordApp, ui: &mut Ui) {
    section(ui, "Borders and Shading", |ui| {
        row(ui, |ui| {
            crate::widgets::split(
                ui,
                app,
                "shading",
                "Shading",
                "table.shading",
                json!({"color": app.canvas.last_shading.clone()}),
                false,
                None,
                |ui, app| {
                    mi(ui, app, "No Color", "table.shading", json!({"color": null}));
                    let theme = app.session.doc.settings.theme_colors.clone();
                    crate::colors::menu(ui, app, &theme, Target::cmd("table.shading").remember(Remember::Shading));
                },
            );
            crate::widgets::menu_button(ui, app, "borders", Some("Borders"), "Borders", false, "table.borders", |ui, app| {
                for (label, kind) in [
                    ("All Borders", "all"),
                    ("Outside Borders", "outside"),
                    ("Inside Borders", "inside"),
                    ("No Border", "none"),
                    ("Top Border", "top"),
                    ("Bottom Border", "bottom"),
                    ("Left Border", "left"),
                    ("Right Border", "right"),
                ] {
                    let mut params = crate::table_pen::border_params(app);
                    params["kind"] = json!(kind);
                    mi(ui, app, label, "table.borders", params);
                }
                ui.separator();
                ui.menu_button(tl!("Style"), |ui| {
                    for (label, style) in [
                        ("Single", "single"),
                        ("Double", "double"),
                        ("Dotted", "dotted"),
                        ("Dashed", "dashed"),
                        ("Thick", "thick"),
                        ("Triple", "triple"),
                        ("Dash Dot", "dotDash"),
                        ("Wave", "wave"),
                    ] {
                        let current = app.canvas.table_border.style.ooxml();
                        let look = crate::line_looks::word_look(style);
                        let response = crate::line_looks::swatch(ui, vec2(150.0, 22.0), look, tl!(label), current == style);
                        if response.clicked() {
                            let mut params = crate::table_pen::border_params(app);
                            params["style"] = json!(style);
                            let _ = app.run("table.borders", params);
                            ui.close();
                        }
                    }
                });
                ui.menu_button(tl!("Width:"), |ui| {
                    for (label, width) in
                        [("0.25 pt", 0.25), ("0.5 pt", 0.5), ("1 pt", 1.0), ("1.5 pt", 1.5), ("2 pt", 2.0), ("3 pt", 3.0), ("6 pt", 6.0)]
                    {
                        let mut params = crate::table_pen::border_params(app);
                        params["width"] = json!(width);
                        mi(ui, app, label, "table.borders", params);
                    }
                });
                ui.menu_button(tl!("Color"), |ui| {
                    let theme = app.session.doc.settings.theme_colors.clone();
                    let params = crate::table_pen::border_params(app);
                    crate::colors::menu(ui, app, &theme, Target::at("table.borders", params, "/color"));
                });
            });
        });
        row(ui, |ui| {
            use crate::table_pen::TableTool;
            crate::widgets::small(
                ui,
                app,
                "borderPainter",
                Some("Border Painter"),
                "Border Painter",
                "table.borderPainter",
                json!({}),
                app.canvas.table_tool == Some(TableTool::Paint),
            );
            crate::widgets::small(
                ui,
                app,
                "eraser",
                Some("Eraser"),
                "Eraser",
                "table.eraser",
                json!({}),
                app.canvas.table_tool == Some(TableTool::Erase),
            );
        });
    });
}

fn rows_columns(app: &mut WordApp, ui: &mut Ui) {
    section(ui, "Rows and Columns", |ui| {
        row(ui, |ui| {
            crate::widgets::small(ui, app, "insertAbove", Some("Insert Above"), "Insert Above", "table.insertRowAbove", json!({}), false);
            crate::widgets::small(ui, app, "insertBelow", Some("Insert Below"), "Insert Below", "table.insertRowBelow", json!({}), false);
            crate::widgets::small(ui, app, "insertLeft", Some("Insert Left"), "Insert Left", "table.insertColumnLeft", json!({}), false);
            crate::widgets::small(ui, app, "insertRight", Some("Insert Right"), "Insert Right", "table.insertColumnRight", json!({}), false);
            crate::widgets::menu_button(ui, app, "deleteTable", Some("Delete"), "Delete", false, "table.deleteTable", |ui, app| {
                mi(ui, app, "Delete Cells", "table.deleteCells", json!({}));
                mi(ui, app, "Delete Columns", "table.deleteColumn", json!({}));
                mi(ui, app, "Delete Rows", "table.deleteRow", json!({}));
                mi(ui, app, "Delete Table", "table.deleteTable", json!({}));
            });
        });
    });
    section(ui, "Merge and Split", |ui| {
        row(ui, |ui| {
            crate::widgets::small(ui, app, "merge", Some("Merge Cells"), "Merge Cells", "table.merge", json!({}), false);
            crate::widgets::small(ui, app, "splitCells", Some("Split Cells"), "Split Cells", "table.split", json!({}), false);
            crate::widgets::small(ui, app, "splitTable", Some("Split Table"), "Split Table", "table.splitTable", json!({}), false);
        });
    });
    section(ui, "Cell Size", |ui| {
        crate::widgets::menu_button(ui, app, "autofit", Some("AutoFit"), "AutoFit", false, "table.autofit", |ui, app| {
            mi(ui, app, "AutoFit Contents", "table.autofit", json!({"mode": "contents"}));
            mi(ui, app, "AutoFit Window", "table.autofit", json!({"mode": "window"}));
            mi(ui, app, "Fixed Column Width", "table.autofit", json!({"mode": "fixed"}));
        });
        crate::ribbon::cell_size_boxes(ui, app);
        row(ui, |ui| {
            crate::widgets::small(ui, app, "distributeRows", Some("Distribute Rows"), "Distribute Rows", "table.distributeRows", json!({}), false);
            crate::widgets::small(
                ui,
                app,
                "distributeCols",
                Some("Distribute Columns"),
                "Distribute Columns",
                "table.distributeColumns",
                json!({}),
                false,
            );
        });
    });
    section(ui, "Alignment", |ui| {
        egui::Grid::new("table_context_cell_alignment").num_columns(3).spacing(vec2(2.0, 2.0)).show(ui, |ui| {
            for (i, value) in ["topLeft", "topCenter", "topRight", "centerLeft", "center", "centerRight", "bottomLeft", "bottomCenter", "bottomRight"]
                .into_iter()
                .enumerate()
            {
                crate::widgets::small(ui, app, "cellAlign", None, value, "table.cellAlign", json!({"value": value}), false);
                if i % 3 == 2 {
                    ui.end_row();
                }
            }
        });
        row(ui, |ui| {
            crate::widgets::small(ui, app, "textDirection", Some("Text Direction"), "Text Direction", "table.textDirection", json!({}), false);
            crate::widgets::small(ui, app, "cellMargins", Some("Cell Margins"), "Cell Margins", "table.cellMargins", json!({}), false);
        });
    });
    section(ui, "Data", |ui| {
        row(ui, |ui| {
            crate::widgets::small(ui, app, "sort", Some("Sort"), "Sort", "table.sort", json!({}), false);
            crate::widgets::small(ui, app, "formula", Some("Formula"), "Formula", "table.formula", json!({}), false);
            crate::widgets::small(ui, app, "repeatHeader", Some("Repeat Header Rows"), "Repeat Header Rows", "table.repeatHeader", json!({}), false);
            crate::widgets::small(ui, app, "toText", Some("Convert to Text"), "Convert to Text", "table.toText", json!({}), false);
        });
    });
}
