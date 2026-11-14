//! Table Design and Table Layout tabs.

use doc4me_doc::props::{Align, Border, BorderStyle, Borders, CharProps, HeightRule, Rgb, TextDirection, VAlign};
use doc4me_doc::{Block, Paragraph, Path, Pos, StoryRef, Table, para_block};
use serde_json::{Value, json};

use super::sel_result;
use crate::{CmdError, CmdResult, CommandSpec, Selection, Session, p};

fn in_table(s: &Session) -> Option<&'static str> {
    if cell(s).is_ok() { None } else { Some("the cursor isn't in a table") }
}

pub fn specs() -> Vec<CommandSpec> {
    let t = |c: CommandSpec| c.when(in_table);
    vec![
        t(CommandSpec::new("table.insertRowAbove", "Insert Above", "Table Layout › Rows & Columns", |s, _| row(s, false))),
        t(CommandSpec::new("table.insertRowBelow", "Insert Below", "Table Layout › Rows & Columns", |s, _| row(s, true))),
        t(CommandSpec::new("table.insertColumnLeft", "Insert Left", "Table Layout › Rows & Columns", |s, _| col(s, false))),
        // Word's Insert Rows and Insert Cells dialogs.
        t(CommandSpec::new("table.insertRows", "Insert Rows", "Table Layout › Rows & Columns", insert_rows)
            .params(r#"{"count"?: 1-100 (default 1), "below"?: bool (default above)}"#)),
        t(CommandSpec::new("table.insertCells", "Insert Cells", "Table Layout › Rows & Columns", insert_cells)
            .params(r#"{"shift": "down" (the column's cells move down a row from here; the table gains a row) | "row" (a whole row above) | "column" (a whole column left)}"#)),
        t(CommandSpec::new("table.insertColumnRight", "Insert Right", "Table Layout › Rows & Columns", |s, _| col(s, true))),
        t(CommandSpec::new("table.deleteRow", "Delete Rows", "Table Layout › Rows & Columns › Delete", del_row)),
        t(CommandSpec::new("table.deleteColumn", "Delete Columns", "Table Layout › Rows & Columns › Delete", del_col)),
        t(CommandSpec::new("table.deleteTable", "Delete Table", "Table Layout › Rows & Columns › Delete", del_table)),
        t(CommandSpec::new("table.deleteCells", "Delete Cells", "Table Layout › Rows & Columns › Delete", delete_cells)),
        t(CommandSpec::new("table.merge", "Merge Cells", "Table Layout › Merge", merge)),
        t(CommandSpec::new("table.split", "Split Cells", "Table Layout › Merge", split_cells)
        .params(r#"{"columns"?: n}"#)),
        t(CommandSpec::new("table.splitTable", "Split Table", "Table Layout › Merge", split_table)),
        t(CommandSpec::new("table.style", "Table Styles", "Table Design › Table Styles", |s, v| {
            let st = p::req_str(v, "style")?;
            let id = super::table_style::table_style_id(s, st).ok_or_else(|| CmdError::Params(format!("no table style `{st}`")))?;
            with_table(s, |t| t.props.style = Some(id.clone()))
        })
        .params(r#"{"style": string}"#)),
        t(CommandSpec::new("table.look", "Table Style Options", "Table Design › Table Style Options", |s, v| {
            let set = |k: &str, cur: bool| p::bool(v, k).unwrap_or(cur);
            with_table(s, |t| {
                let l = &mut t.props.look;
                l.header_row = set("headerRow", l.header_row);
                l.total_row = set("totalRow", l.total_row);
                l.banded_rows = set("bandedRows", l.banded_rows);
                l.first_column = set("firstColumn", l.first_column);
                l.last_column = set("lastColumn", l.last_column);
                l.banded_columns = set("bandedColumns", l.banded_columns);
            })
        })
        .params(
            r#"{"headerRow"?: bool, "totalRow"?: bool, "bandedRows"?: bool, "firstColumn"?: bool, "lastColumn"?: bool, "bandedColumns"?: bool}"#,
        )),
        t(CommandSpec::new("table.shading", "Shading", "Table Design › Table Styles", |s, v| {
            let c = p::str(v, "color").and_then(Rgb::parse);
            with_cells(s, |cl| cl.props.shading = c)
        })
        .params(r#"{"color": "RRGGBB" | null}"#)),
        t(CommandSpec::new("table.borders", "Borders", "Table Design › Borders", borders)
            .params(r#"{"kind": "all|outside|inside|insideHorizontal|insideVertical|none|top|bottom|left|right", "style"?: "single|double|dotted|dashed|thick|triple|dotDash|wave|none", "width"?: pt, "color"?: "RRGGBB", "scope"?: "selection|table"} (an explicit cell selection is respected; without one the legacy default is the whole table)"#)),
        t(CommandSpec::new("table.cellAlign", "Alignment", "Table Layout › Alignment", |s, v| {
            let (h, va) = match p::str(v, "value").unwrap_or("topLeft") {
                "topLeft" => (Align::Left, VAlign::Top),
                "topCenter" => (Align::Center, VAlign::Top),
                "topRight" => (Align::Right, VAlign::Top),
                "centerLeft" => (Align::Left, VAlign::Center),
                "center" => (Align::Center, VAlign::Center),
                "centerRight" => (Align::Right, VAlign::Center),
                "bottomLeft" => (Align::Left, VAlign::Bottom),
                "bottomCenter" => (Align::Center, VAlign::Bottom),
                "bottomRight" => (Align::Right, VAlign::Bottom),
                x => return Err(CmdError::Params(format!("unknown alignment `{x}`"))),
            };
            with_cells(s, |cl| {
                cl.props.valign = va;
                for b in &mut cl.blocks {
                    if let Block::Para(p) = std::sync::Arc::make_mut(b) {
                        p.props.align = Some(h);
                        p.touch();
                    }
                }
            })
        })
        .params(r#"{"value": "topLeft|topCenter|…|bottomRight"}"#)),
        t(CommandSpec::new("table.textDirection", "Text Direction", "Table Layout › Alignment", text_direction)
            .params(r#"{"value"?: "horizontal|down|up"} (default: the next direction after the caret cell's, like Word's button)"#)),
        t(CommandSpec::new("table.autofit", "AutoFit", "Table Layout › Cell Size", autofit).params(r#"{"mode": "contents|window|fixed"}"#)),
        t(CommandSpec::new("table.distributeColumns", "Distribute Columns", "Table Layout › Cell Size", distribute_columns)),
        t(CommandSpec::new("table.distributeRows", "Distribute Rows", "Table Layout › Cell Size", distribute_rows)),
        t(CommandSpec::new("table.columnWidth", "Table Column Width", "Table Layout › Cell Size", |s, v| {
            let w = p::req_f32(v, "width")?.clamp(6.0, 1584.0);
            let cells = selected_cells(s);
            with_table(s, |t| set_column_widths(t, &cells, w))
        })
        .params(
            r#"{"width": pt} (every column the selection covers, the caret cell's without one; the table's preferred width is cleared so the columns get exactly this)"#,
        )),
        t(CommandSpec::new("table.rowHeight", "Table Row Height", "Table Layout › Cell Size", |s, v| {
            let h = p::req_f32(v, "height")?.clamp(1.0, 1584.0);
            let rule = match p::str(v, "rule") {
                None | Some("atLeast") => HeightRule::AtLeast,
                Some("exact") => HeightRule::Exact,
                Some(x) => return Err(CmdError::Params(format!("unknown height rule `{x}`"))),
            };
            let mut rows: Vec<usize> = selected_cells(s).into_iter().map(|(r, _)| r).collect();
            rows.dedup();
            with_table(s, |t| {
                for r in &rows {
                    if let Some(row) = t.rows.get_mut(*r) {
                        row.props.height = Some(h);
                        row.props.height_rule = rule;
                    }
                }
            })
        })
        .params(r#"{"height": pt, "rule"?: "atLeast|exact"} (every row the selection covers, the caret's without one)"#)),
        t(CommandSpec::new("table.repeatHeader", "Repeat Header Rows", "Table Layout › Data", repeat_header)
            .params(r#"{"value"?: bool} (selected rows; repeated headers must form a prefix at the table's start)"#)),
        // Table Properties › Alt Text: the title (`w:tblCaption`) and description (`w:tblDescription`).
        t(CommandSpec::new("table.altText", "Alt Text", "Table Layout › Table › Properties", |s, v| {
            let (title, text) = (p::str(v, "title"), p::str(v, "text"));
            if title.is_none() && text.is_none() {
                return Err(CmdError::Params("`text` or `title` (string) is required".into()));
            }
            let set = |x: Option<&str>, cur: &Option<String>| match x {
                Some(x) if x.trim().is_empty() => None,
                Some(x) => Some(doc4me_doc::para::cap_alt(x)),
                None => cur.clone(),
            };
            with_table(s, |t| {
                t.props.caption = set(title, &t.props.caption);
                t.props.description = set(text, &t.props.description);
            })
        })
        .params(r#"{"text"?: string, "title"?: string} (empty clears)"#)),
        t(CommandSpec::new("table.sort", "Sort", "Table Layout › Data", sort_table)
            .params(r#"{"column"?: n, "descending"?: bool, "header"?: bool}"#)),
        t(CommandSpec::new("table.toText", "Convert to Text", "Table Layout › Data", to_text).params(r#"{"separator"?: "tab|comma|paragraph"}"#)),
        t(CommandSpec::new("table.formula", "Formula", "Table Layout › Data", formula).params(r#"{"formula"?: "=SUM(ABOVE)"}"#)),
        t(CommandSpec::new("table.selectTable", "Select Table", "Table Layout › Table › Select", |s, _| {
            let (tp, _, _) = cell(s)?;
            select_cells(s, &tp, None)
        })
        .pure()),
        t(CommandSpec::new("table.selectRow", "Select Row", "Table Layout › Table › Select", |s, _| {
            let (tp, r, _) = cell(s)?;
            select_cells(s, &tp, Some((r, None)))
        })
        .pure()),
        t(CommandSpec::new("table.selectColumn", "Column", "Table Layout › Table › Select", select_column).pure()),
        t(CommandSpec::new("table.selectCell", "Select Cell", "Table Layout › Table › Select", |s, _| {
            let (tp, r, c) = cell(s)?;
            select_cells(s, &tp, Some((r, Some(c))))
        })
        .pure()),
        t(CommandSpec::new("table.rtl", "Right-to-Left Table", "Table Layout › Table", |s, v| {
            let (tp, _, _) = cell(s)?;
            let on = match p::bool(v, "value") {
                Some(b) => b,
                None => !s.doc.table(s.sel.focus.story, &tp).is_some_and(|t| t.props.bidi_visual),
            };
            with_table(s, |t| t.props.bidi_visual = on)
        })
        .params(r#"{"value"?: bool} (true: the first column stands at the right; without a value it toggles)"#)),
        t(CommandSpec::new("table.properties", "Properties", "Table Layout › Table", properties).params(
            r#"{"align"?: "left|center|right", "rtl"?: bool, "width"?: pt|null, "widthPct"?: percent, "indent"?: pt, "rowHeight"?: pt|null, "rowHeightRule"?: "atLeast|exact", "allowBreak"?: bool, "headerRow"?: bool, "columnWidth"?: pt, "cellWidth"?: pt|null, "valign"?: "top|center|bottom"} (row, column and cell settings apply to the caret's; without settings it returns the current ones)"#,
        )),
        // Word's Cell Options (Table Properties › Cell › Options…) and Table Positioning (Table ›
        // Text wrapping › Positioning…) dialogs.
        t(CommandSpec::new("table.cellOptions", "Cell Options", "Table Layout › Table › Properties", cell_options).params(
            r#"{"sameAsTable"?: bool (the table's cell margins), "top"?, "bottom"?, "left"?, "right"? (pt), "wrap"?: bool} (for the selected cells; without settings it returns the caret cell's)"#,
        )),
        t(CommandSpec::new("table.positioning", "Table Positioning", "Table Layout › Table › Properties", positioning).params(
            r#"{"around"?: bool (false puts the table back in the text), "horizontal"?: "left|center|right|inside|outside"|pt, "horizontalFrom"?: "column|margin|page", "vertical"?: "top|center|bottom|inside|outside"|pt, "verticalFrom"?: "paragraph|margin|page", "top"?, "bottom"?, "left"?, "right"? (pt from the text around it), "overlap"?: bool} (without settings it returns the current ones)"#,
        )),
        CommandSpec::new("table.fromText", "Convert Text to Table", "Insert › Tables", from_text).params(r#"{"separator"?: "tab|comma"}"#),
        CommandSpec::new("table.quick", "Quick Tables", "Insert › Tables", quick_table).params(r#"{"kind"?: "calendar|tabular|matrix"}"#),
    ]
}

/// Table Layout › AutoFit: Contents sizes the columns to their text, Window spreads them across
/// the text column, Fixed keeps the widths as they are.
fn autofit(s: &mut Session, v: &Value) -> CmdResult {
    let mode = p::str(v, "mode").unwrap_or("window");
    let (tp, _, _) = cell(s)?;
    let story = s.sel.focus.story;
    let tw = super::page::sect(s).text_width();
    match mode {
        "fixed" => with_table(s, |t| t.props.fixed = true),
        "contents" => {
            let t = s.doc.table(story, &tp).ok_or_else(|| CmdError::Disabled("the cursor isn't in a table".into()))?;
            let avail = (tw - t.props.indent.filter(|i| i.is_finite()).unwrap_or(0.0).max(0.0)).max(72.0);
            let widths = doc4me_layout::autofit_widths(&doc4me_layout::measure_table_columns(&s.doc, t), avail);
            with_table(s, |t| {
                t.props.fixed = false;
                t.props.width = None;
                t.props.width_pct = None;
                t.grid = widths.iter().map(|w| w.max(6.0)).collect();
                sync_cell_widths(t);
            })
        }
        "window" => with_table(s, |t| {
            t.props.fixed = false;
            t.props.width_pct = Some(100.0);
            let n = t.grid.len().max(1) as f32;
            t.grid.iter_mut().for_each(|g| *g = tw / n);
            sync_cell_widths(t);
        }),
        x => Err(CmdError::Params(format!("unknown AutoFit mode `{x}`"))),
    }
}

/// Give every cell the width of the grid columns it spans, so the cells' preferred widths
/// (saved to .docx) agree with the grid.
pub(crate) fn sync_cell_widths(t: &mut Table) {
    let grid = t.grid.clone();
    for row in &mut t.rows {
        let mut g = row.props.grid_before.min(doc4me_doc::table::MAX_COLS as u32) as usize;
        for c in &mut row.cells {
            let span = c.span();
            let w: f32 = grid.get(g..g.saturating_add(span).min(grid.len())).unwrap_or(&[]).iter().sum();
            if w > 0.0 {
                c.props.width = Some(w);
            }
            g += span;
        }
    }
}

/// Set the width of the grid column cell `c` of row `r` starts in. The table's preferred width
/// goes, so the layout doesn't scale the column back.
/// Give every grid column the `cells` (row, cell) pairs touch the width `w`.
fn set_column_widths(t: &mut Table, cells: &[(usize, usize)], w: f32) {
    let cols = grid_columns(t, cells);
    let mut any = false;
    for g in cols {
        if let Some(x) = t.grid.get_mut(g) {
            *x = w;
            any = true;
        }
    }
    if any {
        t.props.width = None;
        t.props.width_pct = None;
        sync_cell_widths(t);
    }
}

fn grid_columns(t: &Table, cells: &[(usize, usize)]) -> std::collections::BTreeSet<usize> {
    cells
        .iter()
        .flat_map(|&(r, c)| {
            let g = t.grid_col(r, c);
            let span = t.rows.get(r).and_then(|r| r.cells.get(c)).map_or(0, doc4me_doc::Cell::span);
            g..g.saturating_add(span).min(t.grid.len())
        })
        .collect()
}

fn distribute_columns(s: &mut Session, _: &Value) -> CmdResult {
    let (tp, _, _) = cell(s)?;
    let t = s.doc.table(s.sel.focus.story, &tp).ok_or_else(|| CmdError::Disabled("no table".into()))?;
    let cols =
        if s.selected_table_cells().is_some() || !s.sel.is_collapsed() { grid_columns(t, &selected_cells(s)) } else { (0..t.grid.len()).collect() };
    with_table(s, |t| {
        let total: f32 = cols.iter().filter_map(|c| t.grid.get(*c)).sum();
        let width = total / cols.len().max(1) as f32;
        for c in cols {
            if let Some(w) = t.grid.get_mut(c) {
                *w = width;
            }
        }
        sync_cell_widths(t);
    })
}

fn distribute_rows(s: &mut Session, _: &Value) -> CmdResult {
    let (tp, _, _) = cell(s)?;
    let story = s.sel.focus.story;
    let t = s.doc.table(story, &tp).ok_or_else(|| CmdError::Disabled("no table".into()))?;
    let rows: std::collections::BTreeSet<_> = if s.selected_table_cells().is_some() || !s.sel.is_collapsed() {
        selected_cells(s).into_iter().map(|(r, _)| r).collect()
    } else {
        (0..t.rows.len()).collect()
    };
    let mut heights = std::collections::BTreeMap::<usize, f32>::new();
    for page in &s.layout().pages {
        let mut local = std::collections::BTreeMap::<usize, f32>::new();
        for it in page.items.iter().chain(&page.header).chain(&page.footer) {
            if let doc4me_layout::Placed::Cell { rect, table, story: st, row, .. } = it
                && *st == story
                && *table == tp
                && rows.contains(row)
            {
                local.entry(*row).and_modify(|h| *h = h.min(rect.h)).or_insert(rect.h);
            }
        }
        for (r, h) in local {
            *heights.entry(r).or_default() += h;
        }
    }
    let h = heights.values().copied().fold(18.0f32, f32::max).clamp(1.0, 1584.0);
    with_table(s, |t| {
        for r in rows {
            if let Some(row) = t.rows.get_mut(r) {
                row.props.height = Some(h);
                row.props.height_rule = HeightRule::AtLeast;
            }
        }
    })
}

fn set_column_width(t: &mut Table, r: usize, c: usize, w: f32) {
    let g = t.grid_col(r, c);
    if let Some(x) = t.grid.get_mut(g) {
        *x = w;
        t.props.width = None;
        t.props.width_pct = None;
        sync_cell_widths(t);
    }
}

/// Table Properties: table, row, column and cell settings in one undoable step. Row, column and
/// cell settings apply to the caret's. Without any setting it returns the current values.
fn properties(s: &mut Session, v: &Value) -> CmdResult {
    let (tp, r, c) = cell(s)?;
    let keys = [
        "align",
        "rtl",
        "width",
        "widthPct",
        "indent",
        "rowHeight",
        "rowHeightRule",
        "allowBreak",
        "headerRow",
        "columnWidth",
        "cellWidth",
        "valign",
    ];
    if !keys.iter().any(|k| v.get(*k).is_some()) {
        let t = s.doc.table(s.sel.focus.story, &tp).cloned().unwrap_or_default();
        let mut out = serde_json::to_value(&t.props).unwrap_or(Value::Null);
        let row = t.rows.get(r);
        let cl = row.and_then(|x| x.cells.get(c));
        if let Some(o) = out.as_object_mut() {
            o.insert("row".into(), row.map(|x| serde_json::to_value(&x.props).unwrap_or(Value::Null)).unwrap_or(Value::Null));
            o.insert("columnWidth".into(), t.grid.get(t.grid_col(r, c)).map_or(Value::Null, |w| json!(w)));
            o.insert("cell".into(), cl.map(|x| serde_json::to_value(&x.props).unwrap_or(Value::Null)).unwrap_or(Value::Null));
        }
        return Ok(out);
    }
    let align = match p::str(v, "align") {
        None => None,
        Some("left") => Some(Align::Left),
        Some("center") => Some(Align::Center),
        Some("right") => Some(Align::Right),
        Some(x) => return Err(CmdError::Params(format!("unknown table alignment `{x}`"))),
    };
    let rule = match p::str(v, "rowHeightRule") {
        None => None,
        Some("atLeast") => Some(HeightRule::AtLeast),
        Some("exact") => Some(HeightRule::Exact),
        Some(x) => return Err(CmdError::Params(format!("unknown height rule `{x}`"))),
    };
    let valign = match p::str(v, "valign") {
        None => None,
        Some("top") => Some(VAlign::Top),
        Some("center") => Some(VAlign::Center),
        Some("bottom") => Some(VAlign::Bottom),
        Some(x) => return Err(CmdError::Params(format!("unknown vertical alignment `{x}`"))),
    };
    // `null` clears a preferred size (automatic); a number sets it.
    let size = |k: &str, lo: f32, hi: f32| -> Option<Option<f32>> {
        match v.get(k)? {
            Value::Null => Some(None),
            x => x.as_f64().map(|f| f as f32).filter(|f| f.is_finite()).map(|f| Some(f.clamp(lo, hi))),
        }
    };
    let width = size("width", 6.0, 1584.0);
    let width_pct = p::f32(v, "widthPct").map(|x| x.clamp(1.0, 100.0));
    let indent = p::f32(v, "indent").map(|x| x.clamp(-1584.0, 1584.0));
    let row_height = size("rowHeight", 1.0, 1584.0);
    let allow_break = p::bool(v, "allowBreak");
    let header = p::bool(v, "headerRow");
    let column_width = p::f32(v, "columnWidth").map(|x| x.clamp(6.0, 1584.0));
    let cell_width = size("cellWidth", 6.0, 1584.0);
    let rtl = p::bool(v, "rtl");
    with_table(s, |t| {
        if let Some(a) = align {
            t.props.align = Some(a);
        }
        if let Some(on) = rtl {
            t.props.bidi_visual = on;
        }
        if let Some(i) = indent {
            t.props.indent = Some(i);
        }
        if let Some(cw) = column_width {
            set_column_width(t, r, c, cw);
        }
        if let Some(w) = width {
            t.props.width = w;
            t.props.width_pct = None;
        }
        if let Some(pct) = width_pct {
            t.props.width_pct = Some(pct);
        }
        if let Some(row) = t.rows.get_mut(r) {
            if let Some(h) = row_height {
                row.props.height = h;
                row.props.height_rule = if h.is_some() { rule.unwrap_or(HeightRule::AtLeast) } else { HeightRule::Auto };
            } else if let Some(rule) = rule
                && row.props.height.is_some()
            {
                row.props.height_rule = rule;
            }
            if let Some(b) = allow_break {
                row.props.cant_split = !b;
            }
            if let Some(h) = header {
                row.props.header = h;
            }
            if let Some(cl) = row.cells.get_mut(c) {
                if let Some(w) = cell_width {
                    cl.props.width = w;
                }
                if let Some(va) = valign {
                    cl.props.valign = va;
                }
            }
        }
    })
}

/// The table's default cell margins (top, left, bottom, right): its own, else its style's, else
/// Word's.
fn table_margins(s: &Session, t: &Table) -> [f32; 4] {
    t.props
        .cell_margins
        .or_else(|| t.props.style.as_deref().and_then(|id| s.doc.styles.table_style(id)).and_then(|st| st.parts.cell_margins))
        .unwrap_or([0.0, 5.4, 0.0, 5.4])
}

/// `table.cellOptions`: the selected cells' margins, or the table's ("Same as the whole table"),
/// and Wrap text.
fn cell_options(s: &mut Session, v: &Value) -> CmdResult {
    let (tp, r, c) = cell(s)?;
    let t = s.doc.table(s.sel.focus.story, &tp).ok_or_else(|| CmdError::Disabled("the cursor isn't in a table".into()))?;
    let whole = table_margins(s, t);
    if !["sameAsTable", "top", "bottom", "left", "right", "wrap"].iter().any(|k| v.get(*k).is_some()) {
        let cp = t.rows.get(r).and_then(|x| x.cells.get(c)).map(|x| x.props.clone()).unwrap_or_default();
        let m = cp.margins.unwrap_or(whole);
        s.only_read();
        return Ok(json!({"sameAsTable": cp.margins.is_none(), "top": m[0], "left": m[1], "bottom": m[2], "right": m[3], "wrap": !cp.no_wrap}));
    }
    let same = p::bool(v, "sameAsTable");
    let sides = ["top", "left", "bottom", "right"].map(|k| p::f32(v, k).map(|x| x.clamp(0.0, 144.0)));
    let wrap = p::bool(v, "wrap");
    with_cells(s, |cl| {
        if same == Some(true) {
            cl.props.margins = None;
        } else if same == Some(false) || sides.iter().any(Option::is_some) {
            let mut m = cl.props.margins.unwrap_or(whole);
            for (x, side) in m.iter_mut().zip(sides) {
                if let Some(side) = side {
                    *x = side;
                }
            }
            cl.props.margins = Some(m);
        }
        if let Some(w) = wrap {
            cl.props.no_wrap = !w;
        }
    })
}

/// `table.positioning`: Text wrapping Around places the table (`w:tblpPr`) and the text after it
/// wraps around it; None puts it back in the text.
fn positioning(s: &mut Session, v: &Value) -> CmdResult {
    use doc4me_doc::para::{Anchor, FloatAlign};
    use doc4me_doc::props::TableFloat;
    let (tp, _, _) = cell(s)?;
    let story = s.sel.focus.story;
    let current = s.doc.table(story, &tp).and_then(|t| t.props.float);
    let keys = ["around", "horizontal", "horizontalFrom", "vertical", "verticalFrom", "top", "bottom", "left", "right", "overlap"];
    // Word's defaults for a table wrapped from Table Properties: at the left of the column, where
    // its paragraph is, 0.13" from the text at its sides, free to overlap.
    let wrapped = TableFloat { h_rel: Anchor::Column, v_rel: Anchor::Paragraph, dist: [9.0, 0.0, 9.0, 0.0], overlap: true, ..Default::default() };
    if !keys.iter().any(|k| v.get(*k).is_some()) {
        let f = current.unwrap_or(wrapped);
        let from = |a: Anchor, horizontal: bool| match a {
            Anchor::Margin => "margin",
            Anchor::Page => "page",
            _ if horizontal => "column",
            _ => "paragraph",
        };
        let at = |al: Option<FloatAlign>, off: f32, horizontal: bool| al.map_or(json!(off), |a| json!(a.ooxml(horizontal)));
        let d = f.dist_from_text();
        s.only_read();
        return Ok(json!({
            "around": current.is_some(),
            "horizontal": at(f.h_align, f.x, true), "horizontalFrom": from(f.h_rel, true),
            "vertical": at(f.v_align, f.y, false), "verticalFrom": from(f.v_rel, false),
            "left": d[0], "top": d[1], "right": d[2], "bottom": d[3], "overlap": f.overlap,
        }));
    }
    if p::bool(v, "around") == Some(false) {
        return with_table(s, |t| t.props.float = None);
    }
    let mut f = current.unwrap_or(wrapped);
    // A position is an alignment or an offset (points).
    let place = |k: &str, horizontal: bool| -> Result<Option<(Option<FloatAlign>, f32)>, CmdError> {
        match v.get(k) {
            None => Ok(None),
            Some(Value::String(a)) => {
                // `top` isn't a horizontal position, nor `left` a vertical one.
                let al = FloatAlign::from_ooxml(a).filter(|al| al.ooxml(horizontal) == a);
                al.map(|al| Some((Some(al), 0.0)))
                    .ok_or_else(|| CmdError::Params(format!("unknown {} position `{a}`", if horizontal { "horizontal" } else { "vertical" })))
            }
            Some(x) => x
                .as_f64()
                .map(|n| n as f32)
                .filter(|n| n.is_finite())
                .map(|n| Some((None, n.clamp(-1584.0, 1584.0))))
                .ok_or_else(|| CmdError::Params(format!("`{k}` is an alignment or a number of points"))),
        }
    };
    let from = |k: &str, choices: &[(&str, Anchor)]| -> Result<Option<Anchor>, CmdError> {
        match p::str(v, k) {
            None => Ok(None),
            Some(x) => choices.iter().find(|(n, _)| *n == x).map(|(_, a)| Some(*a)).ok_or_else(|| CmdError::Params(format!("unknown `{k}` `{x}`"))),
        }
    };
    if let Some((al, x)) = place("horizontal", true)? {
        (f.h_align, f.x) = (al, x);
    }
    if let Some((al, y)) = place("vertical", false)? {
        (f.v_align, f.y) = (al, y);
    }
    if let Some(a) = from("horizontalFrom", &[("column", Anchor::Column), ("margin", Anchor::Margin), ("page", Anchor::Page)])? {
        f.h_rel = a;
    }
    if let Some(a) = from("verticalFrom", &[("paragraph", Anchor::Paragraph), ("margin", Anchor::Margin), ("page", Anchor::Page)])? {
        f.v_rel = a;
    }
    for (i, k) in ["left", "top", "right", "bottom"].into_iter().enumerate() {
        if let Some(d) = p::f32(v, k) {
            f.dist[i] = d.clamp(0.0, 1584.0);
        }
    }
    if let Some(o) = p::bool(v, "overlap") {
        f.overlap = o;
    }
    with_table(s, move |t| t.props.float = Some(f))
}

fn cell(s: &Session) -> Result<(Path, usize, usize), CmdError> {
    let at = s
        .sel
        .focus
        .path
        .cell()
        .filter(|(tp, r, c)| s.doc.table(s.sel.focus.story, tp).and_then(|t| t.rows.get(*r)).and_then(|r| r.cells.get(*c)).is_some());
    at.ok_or_else(|| CmdError::Disabled("the cursor isn't in a table".into()))
}

fn with_table(s: &mut Session, f: impl FnOnce(&mut Table)) -> CmdResult {
    let (tp, _, _) = cell(s)?;
    let t = s.doc.table_mut(s.sel.focus.story, &tp)?;
    f(t);
    touch_cells(s, &tp)?;
    sel_result(s)
}

/// Selected cells: (row, cell) pairs covered by the selection (at least the caret's cell).
fn selected_cells(s: &Session) -> Vec<(usize, usize)> {
    if let Some(sel) = s.selected_table_cells() {
        return sel.cells.iter().copied().collect();
    }
    let (a, b) = s.sel.ordered();
    let (Some((ta, ra, ca)), Some((tb, rb, cb))) = (a.path.cell(), b.path.cell()) else { return Vec::new() };
    if a.story != b.story || ta != tb {
        return vec![(rb, cb)];
    }
    let Some(t) = s.doc.table(b.story, &tb) else { return Vec::new() };
    let (r0, r1) = (ra.min(rb), ra.max(rb));
    let (c0, c1) = (ca.min(cb), ca.max(cb));
    let mut v = Vec::new();
    for (r, row) in t.rows.iter().enumerate().take(r1.saturating_add(1)).skip(r0) {
        for c in c0..row.cells.len().min(c1.saturating_add(1)) {
            v.push((r, c));
        }
    }
    v
}

/// Keyboard row/column selection retains its direction, while formatting uses cell geometry.
pub(crate) fn capture_cell_selection(s: &mut Session) {
    s.table_selection = None;
    let (a, b) = s.sel.ordered();
    let (Some((ta, ra, ca)), Some((tb, rb, cb))) = (a.path.cell(), b.path.cell()) else { return };
    if a.story != b.story || ta != tb {
        return;
    }
    let Some(t) = s.doc.table(a.story, &ta) else { return };
    let ga = t.grid_col(ra, ca);
    let gb = t.grid_col(rb, cb);
    let end_a = ga.saturating_add(t.rows.get(ra).and_then(|r| r.cells.get(ca)).map_or(0, doc4me_doc::Cell::span));
    let end_b = gb.saturating_add(t.rows.get(rb).and_then(|r| r.cells.get(cb)).map_or(0, doc4me_doc::Cell::span));
    let (left, right) = (ga.min(gb), end_a.max(end_b));
    let mut cells = std::collections::BTreeSet::new();
    for (r, row) in t.rows.iter().enumerate().take(rb.saturating_add(1)).skip(ra) {
        for (c, cell) in row.cells.iter().enumerate() {
            let g = t.grid_col(r, c);
            if g < right && g.saturating_add(cell.span()) > left {
                cells.insert((r, c));
            }
        }
    }
    s.table_selection = Some(std::sync::Arc::new(crate::TableSelection { sel: s.sel.clone(), story: a.story, table: ta, cells, whole_table: false }));
}

fn with_cells(s: &mut Session, f: impl Fn(&mut doc4me_doc::Cell)) -> CmdResult {
    let (tp, _, _) = cell(s)?;
    let cells = selected_cells(s);
    let t = s.doc.table_mut(s.sel.focus.story, &tp)?;
    for (r, c) in cells {
        if let Some(cl) = t.rows.get_mut(r).and_then(|row| row.cells.get_mut(c)) {
            f(cl);
        }
    }
    touch_cells(s, &tp)?;
    sel_result(s)
}

/// Table Layout › Text Direction: turn the selected cells' text. Without a value it cycles the
/// caret cell's direction (horizontal → down → up) and gives every selected cell the result.
pub(crate) fn text_direction(s: &mut Session, v: &Value) -> CmdResult {
    let (tp, r, c) = cell(s)?;
    let dir = match p::str(v, "value") {
        Some("horizontal" | "lrTb") => TextDirection::Horizontal,
        Some("down" | "tbRl") => TextDirection::Down,
        Some("up" | "btLr") => TextDirection::Up,
        Some(x) => return Err(CmdError::Params(format!("unknown text direction `{x}`"))),
        None => {
            let t = s.doc.table(s.sel.focus.story, &tp).ok_or_else(|| CmdError::Disabled("the cursor isn't in a table".into()))?;
            t.rows.get(r).and_then(|row| row.cells.get(c)).map(|cl| cl.props.text_direction).unwrap_or_default().next()
        }
    };
    with_cells(s, |cl| cl.props.text_direction = dir)
}

/// Bump the revision of every paragraph in a table (table style changes alter their layout).
pub(crate) fn touch_cells(s: &mut Session, tp: &Path) -> Result<(), CmdError> {
    let story = s.sel.focus.story;
    let paths: Vec<Path> = s.doc.para_paths(story).into_iter().filter(|p| p.0.len() > tp.0.len() && p.0.starts_with(&tp.0)).collect();
    for p in paths {
        s.doc.para_mut(story, &p)?.touch();
    }
    Ok(())
}

fn fix_caret(s: &mut Session, tp: &Path, r: usize, c: usize) -> CmdResult {
    s.table_selection = None;
    let story = s.sel.focus.story;
    let Some(t) = s.doc.table(story, tp) else {
        s.sel = Selection::caret(s.doc.clamp(&Pos { story, path: tp.clone(), off: 0 }));
        return sel_result(s);
    };
    let r = r.min(t.rows.len().saturating_sub(1));
    let c = c.min(t.rows.get(r).map(|x| x.cells.len().saturating_sub(1)).unwrap_or(0));
    let mut path = tp.0.clone();
    path.extend([r as u32, c as u32, 0]);
    s.sel = Selection::caret(Pos { story, path: Path(path), off: 0 });
    sel_result(s)
}

fn row(s: &mut Session, below: bool) -> CmdResult {
    let (tp, r, c) = cell(s)?;
    let cells = selected_cells(s);
    let rows: std::collections::BTreeSet<_> = cells.iter().map(|x| x.0).collect();
    let n = rows.len().max(1);
    let at = if below { rows.last().copied().unwrap_or(r).saturating_add(1) } else { rows.first().copied().unwrap_or(r) };
    let t = s.doc.table_mut(s.sel.focus.story, &tp)?;
    if n > doc4me_doc::table::MAX_ROWS.saturating_sub(t.rows.len()) {
        return Err(CmdError::Params("too many table rows".into()));
    }
    for _ in 0..n {
        t.insert_row(at, r);
    }
    fix_caret(s, &tp, at, c)
}

/// `table.insertRows`: `count` rows above (or below) the cursor's row, like it.
fn insert_rows(s: &mut Session, v: &Value) -> CmdResult {
    let n = match v.get("count") {
        None | Some(Value::Null) => 1,
        Some(c) => c.as_u64().filter(|n| (1..=100).contains(n)).ok_or_else(|| CmdError::Params("`count` is 1 to 100".into()))?,
    };
    let below = p::bool(v, "below").unwrap_or(false);
    let (tp, r, c) = cell(s)?;
    let rows: std::collections::BTreeSet<_> = selected_cells(s).into_iter().map(|x| x.0).collect();
    let at = if below { rows.last().copied().unwrap_or(r).saturating_add(1) } else { rows.first().copied().unwrap_or(r) };
    let t = s.doc.table_mut(s.sel.focus.story, &tp)?;
    if n as usize > doc4me_doc::table::MAX_ROWS.saturating_sub(t.rows.len()) {
        return Err(CmdError::Params("too many table rows".into()));
    }
    for _ in 0..n {
        t.insert_row(at, r);
    }
    fix_caret(s, &tp, at, c)
}

/// `table.insertCells`. Shift cells down: as in Word, the cells of the cursor's column from its
/// row down move one row down into a new last row, leaving the cursor's cell empty.
fn insert_cells(s: &mut Session, v: &Value) -> CmdResult {
    match p::req_str(v, "shift")? {
        "row" => row(s, false),
        "column" => col(s, false),
        "down" => {
            let (tp, r, c) = cell(s)?;
            let story = s.sel.focus.story;
            let t = s.doc.table_mut(story, &tp)?;
            if t.rows.len() >= doc4me_doc::table::MAX_ROWS {
                return Err(CmdError::Params("too many table rows".into()));
            }
            let last = t.rows.len().saturating_sub(1);
            t.insert_row(last + 1, last);
            for i in (r + 1..t.rows.len()).rev() {
                let above = t.rows.get(i - 1).and_then(|x| x.cells.get(c)).map(|x| x.blocks.clone());
                if let (Some(blocks), Some(cell)) = (above, t.rows.get_mut(i).and_then(|x| x.cells.get_mut(c))) {
                    cell.blocks = blocks;
                }
            }
            if let Some(cell) = t.rows.get_mut(r).and_then(|x| x.cells.get_mut(c)) {
                cell.blocks = doc4me_doc::Cell::empty().blocks;
            }
            touch_cells(s, &tp)?;
            fix_caret(s, &tp, r, c)
        }
        other => Err(CmdError::Params(format!("`shift` is down, row or column, not `{other}`"))),
    }
}

fn col(s: &mut Session, right: bool) -> CmdResult {
    let (tp, r, c) = cell(s)?;
    let cells = selected_cells(s);
    let explicit = s.selected_table_cells().is_some() || !s.sel.is_collapsed();
    let t = s.doc.table_mut(s.sel.focus.story, &tp)?;
    let cols = grid_columns(t, &cells);
    let g = cols.first().copied().unwrap_or_else(|| t.grid_col(r, c));
    let at = if right { cols.last().copied().unwrap_or(g).saturating_add(1) } else { g };
    let n = if explicit { cols.len().max(1) } else { 1 };
    if n > doc4me_doc::table::MAX_COLS.saturating_sub(t.grid.len()) {
        return Err(CmdError::Params("too many table columns".into()));
    }
    let before: f32 = t.grid.iter().sum();
    for _ in 0..n {
        t.insert_col(at);
    }
    // Keep the table within its width: shrink all columns proportionally.
    let total: f32 = t.grid.iter().sum();
    if before > 0.0 {
        let k = before / total;
        t.grid.iter_mut().for_each(|w| *w *= k);
    }
    sync_cell_widths(t);
    let caret = t.cell_at_grid(r, at).unwrap_or(c);
    fix_caret(s, &tp, r, caret)
}

fn delete_cells(s: &mut Session, _: &Value) -> CmdResult {
    let (tp, r, c) = cell(s)?;
    let cells = selected_cells(s);
    let t = s.doc.table_mut(s.sel.focus.story, &tp)?;
    for (rr, cc) in cells.into_iter().rev() {
        if let Some(row) = t.rows.get_mut(rr)
            && cc < row.cells.len()
        {
            let span = row.cells.remove(cc).span();
            row.props.grid_after = row.props.grid_after.saturating_add(span as u32).min(doc4me_doc::table::MAX_COLS as u32);
        }
    }
    t.rows.retain(|r| !r.cells.is_empty());
    if t.rows.is_empty() {
        return remove_table(s, &tp);
    }
    sync_cell_widths(t);
    fix_caret(s, &tp, r, c)
}

fn del_row(s: &mut Session, _: &Value) -> CmdResult {
    let (tp, r, _) = cell(s)?;
    let rows: std::collections::BTreeSet<usize> = selected_cells(s).into_iter().map(|x| x.0).collect();
    let story = s.sel.focus.story;
    let t = s.doc.table_mut(story, &tp)?;
    if rows.len() >= t.rows.len() {
        return del_table(s, &Value::Null);
    }
    for rr in rows.iter().rev() {
        t.delete_row(*rr);
    }
    fix_caret(s, &tp, r, 0)
}

fn del_col(s: &mut Session, _: &Value) -> CmdResult {
    let (tp, r, c) = cell(s)?;
    let cells = selected_cells(s);
    let story = s.sel.focus.story;
    let t = s.doc.table_mut(story, &tp)?;
    if t.grid.len() <= 1 {
        return del_table(s, &Value::Null);
    }
    // Every grid column a selected cell covers goes (the caret's alone without a selection).
    let mut cols = std::collections::BTreeSet::new();
    for (rr, cc) in cells {
        let g = t.grid_col(rr, cc);
        let span = t.rows.get(rr).and_then(|x| x.cells.get(cc)).map_or(1, doc4me_doc::Cell::span);
        cols.extend(g..g.saturating_add(span).min(t.grid.len()));
    }
    if cols.is_empty() {
        cols.insert(t.grid_col(r, c));
    }
    if cols.len() >= t.grid.len() {
        return del_table(s, &Value::Null);
    }
    for g in cols.iter().rev() {
        t.delete_col(*g);
    }
    // Rows left without cells are dropped; if that was every row, no table is left.
    if t.rows.is_empty() {
        return remove_table(s, &tp);
    }
    fix_caret(s, &tp, r, c.saturating_sub(1))
}

fn del_table(s: &mut Session, _: &Value) -> CmdResult {
    let (tp, _, _) = cell(s)?;
    remove_table(s, &tp)
}

fn remove_table(s: &mut Session, tp: &Path) -> CmdResult {
    let story = s.sel.focus.story;
    s.doc.remove_block(story, tp)?;
    s.table_selection = None;
    s.sel = Selection::caret(s.doc.clamp(&Pos { story, path: tp.clone(), off: 0 }));
    sel_result(s)
}

fn merge(s: &mut Session, _: &Value) -> CmdResult {
    let (tp, _, _) = cell(s)?;
    let cells = selected_cells(s);
    if cells.len() < 2 {
        return Err(CmdError::Failed("select two or more cells to merge".into()));
    }
    let story = s.sel.focus.story;
    let (r0, r1) = (cells.iter().map(|x| x.0).min().unwrap_or(0), cells.iter().map(|x| x.0).max().unwrap_or(0));
    let t = s.doc.table_mut(story, &tp)?;
    let cols = grid_columns(t, &cells);
    let g0 = cols.first().copied().unwrap_or(0);
    let g1 = cols.last().copied().unwrap_or(g0);
    for r in r0..=r1 {
        for g in g0..=g1 {
            if let Some(c) = t.cell_at_grid(r, g)
                && !cells.contains(&(r, c))
            {
                return Err(CmdError::Params("select a rectangle of complete cells to merge".into()));
            }
        }
    }
    let c0 = t.cell_at_grid(r0, g0).unwrap_or(0);
    t.merge(r0, r1, g0, g1);
    fix_caret(s, &tp, r0, c0)
}

fn split_cells(s: &mut Session, v: &Value) -> CmdResult {
    let n = p::u64(v, "columns").unwrap_or(2).clamp(1, doc4me_doc::table::MAX_COLS as u64) as usize;
    let (tp, r, c) = cell(s)?;
    if selected_cells(s).len() > 1 {
        return Err(CmdError::Disabled("select one cell to split".into()));
    }
    let t = s.doc.table_mut(s.sel.focus.story, &tp)?;
    let g = t.grid_col(r, c);
    let span = t.rows.get(r).and_then(|row| row.cells.get(c)).map_or(1, doc4me_doc::Cell::span);
    let (r0, r1) = super::table_draw::vgroup(t, r, c);
    let merged = t.rows.get(r).and_then(|row| row.cells.get(c)).is_some_and(|c| c.props.vmerge != doc4me_doc::props::VMerge::None);
    if merged {
        for rr in r0..=r1 {
            let cell = t.cell_at_grid(rr, g).and_then(|cc| t.rows.get(rr).and_then(|row| row.cells.get(cc)));
            if cell.is_none_or(|cell| cell.span() != span || (rr == r0 && cell.props.vmerge != doc4me_doc::props::VMerge::Restart)) {
                return Err(CmdError::Disabled("repair the inconsistent vertical merge before splitting it".into()));
            }
        }
    }
    for rr in r0..=r1 {
        let cc = t.cell_at_grid(rr, g).ok_or_else(|| CmdError::Failed("the merged cell has no grid column".into()))?;
        let count = t.rows.get(rr).map_or(0, |row| row.cells.len());
        t.split_cell(rr, cc, n);
        if t.rows.get(rr).is_none_or(|row| row.cells.len() != count.saturating_add(n.saturating_sub(1))) {
            return Err(CmdError::Params("too many table grid columns for an equal split".into()));
        }
        if merged && let Some(row) = t.rows.get_mut(rr) {
            for cell in row.cells.iter_mut().skip(cc).take(n) {
                cell.props.vmerge = if rr == r0 { doc4me_doc::props::VMerge::Restart } else { doc4me_doc::props::VMerge::Continue };
            }
        }
    }
    sync_cell_widths(t);
    let caret = t.cell_at_grid(r0, g).unwrap_or(c);
    fix_caret(s, &tp, r0, caret)
}

fn split_table(s: &mut Session, _: &Value) -> CmdResult {
    let (tp, r, _) = cell(s)?;
    if r == 0 {
        return Err(CmdError::Failed("can't split at the first row".into()));
    }
    let story = s.sel.focus.story;
    let t = s.doc.table_mut(story, &tp)?;
    if t.rows.get(r).is_some_and(|row| row.cells.iter().any(|c| c.props.vmerge == doc4me_doc::props::VMerge::Continue)) {
        return Err(CmdError::Disabled("split the vertically merged cells before splitting the table".into()));
    }
    let tail_rows = t.rows.split_off(r);
    // Content controls closing after the table now close after its second part.
    let close = std::mem::take(&mut t.controls.close);
    let controls = doc4me_doc::ControlWrap { open: Vec::new(), close };
    let tail = Table { props: t.props.clone(), grid: t.grid.clone(), rows: tail_rows, controls };
    let after = tp.with_last(tp.last() + 1);
    s.doc.insert_block(story, &after, Block::Para(Paragraph::new()))?;
    s.doc.insert_block(story, &after.with_last(after.last() + 1), Block::Table(tail))?;
    s.sel = Selection::caret(Pos { story, path: after, off: 0 });
    sel_result(s)
}

fn repeat_header(s: &mut Session, v: &Value) -> CmdResult {
    let (tp, _, _) = cell(s)?;
    let rows: std::collections::BTreeSet<_> = selected_cells(s).into_iter().map(|(r, _)| r).collect();
    let t = s.doc.table(s.sel.focus.story, &tp).ok_or_else(|| CmdError::Disabled("no table".into()))?;
    let first = rows.first().copied().ok_or_else(|| CmdError::Disabled("no rows selected".into()))?;
    let on = p::bool(v, "value").unwrap_or(!rows.iter().all(|r| t.rows.get(*r).is_some_and(|r| r.props.header)));
    if on && t.rows.iter().take(first).any(|r| !r.props.header) {
        return Err(CmdError::Disabled("repeat the first row and consecutive rows beneath it".into()));
    }
    with_table(s, |t| {
        for (r, row) in t.rows.iter_mut().enumerate() {
            if rows.contains(&r) || (!on && r >= first) {
                row.props.header = on;
            }
        }
    })
}

pub(crate) fn paint_border(s: &mut Session, v: &Value) -> CmdResult {
    if !["page", "x0", "y0", "x1", "y1"].iter().any(|k| v.get(k).is_some()) {
        return borders(s, v);
    }
    let page = p::u64(v, "page").and_then(|n| usize::try_from(n).ok()).ok_or_else(|| CmdError::Params("`page` is required".into()))?;
    let (x0, y0, x1, y1) = (p::req_f32(v, "x0")?, p::req_f32(v, "y0")?, p::req_f32(v, "x1")?, p::req_f32(v, "y1")?);
    let (dx, dy) = ((x1 - x0).abs(), (y1 - y0).abs());
    let click = dx < 3.0 && dy < 3.0;
    let horizontal = dx >= dy;
    if !click && dx.min(dy) > 3.0 {
        return Err(CmdError::Params("paint a horizontal or vertical border".into()));
    }
    let layout = s.layout();
    let pg = layout.pages.get(page).ok_or_else(|| CmdError::Params(format!("no page {page}")))?;
    let mut hits = Vec::new();
    for it in pg.items.iter().chain(&pg.header).chain(&pg.footer) {
        let doc4me_layout::Placed::Cell { rect, table, story, row, cell, covered: false, .. } = it else { continue };
        for (side, at, a, b) in [
            (0, rect.y, rect.x, rect.right()),
            (1, rect.right(), rect.y, rect.bottom()),
            (2, rect.bottom(), rect.x, rect.right()),
            (3, rect.x, rect.y, rect.bottom()),
        ] {
            let h = side % 2 == 0;
            let (line, lo, hi) = if h { ((y0 + y1) / 2.0, x0.min(x1), x0.max(x1)) } else { ((x0 + x1) / 2.0, y0.min(y1), y0.max(y1)) };
            let distance = (at - line).abs();
            let overlaps = if click { lo >= a - 3.0 && hi <= b + 3.0 } else { hi.min(b) - lo.max(a) > 0.5 };
            if (click || h == horizontal) && distance <= 3.0 && overlaps {
                hits.push((distance, table.clone(), *story, *row, *cell, side));
                if hits.len() > 4096 {
                    return Err(CmdError::Params("paint a shorter border stroke".into()));
                }
            }
        }
    }
    hits.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| b.1.0.len().cmp(&a.1.0.len())));
    let (_, table, story, _, _, _) = hits.first().cloned().ok_or_else(|| CmdError::Failed("no table border within 3 pt of the stroke".into()))?;
    if click {
        hits.truncate(1);
    }
    let saved = (s.sel.clone(), s.table_selection.clone());
    let mut count = 0;
    for (_, tp, st, row, cell, side) in hits {
        if tp != table || st != story {
            continue;
        }
        s.sel.focus.story = story;
        set_cell_selection(s, &tp, std::collections::BTreeSet::from([(row, cell)]), false)?;
        let mut params = v.clone();
        params["kind"] = json!(["top", "right", "bottom", "left"].get(side).copied().unwrap_or("top"));
        params["scope"] = json!("selection");
        borders(s, &params)?;
        count += 1;
    }
    (s.sel, s.table_selection) = saved;
    Ok(json!({"paintedEdges":count}))
}

fn borders(s: &mut Session, v: &Value) -> CmdResult {
    let kind = p::str(v, "kind").unwrap_or("all");
    if !["all", "outside", "inside", "insideHorizontal", "insideVertical", "none", "top", "bottom", "left", "right"].contains(&kind) {
        return Err(CmdError::Params(format!("unknown border kind `{kind}`")));
    }
    let style = p::str(v, "style").unwrap_or("single");
    if !["single", "double", "dotted", "dashed", "thick", "triple", "dotDash", "wave", "none", "nil"].contains(&style) {
        return Err(CmdError::Params(format!("unknown border style `{style}`")));
    }
    let scope = p::str(v, "scope");
    if scope.is_some_and(|v| !["selection", "table"].contains(&v)) {
        return Err(CmdError::Params("border scope must be selection or table".into()));
    }
    let none = Border { style: BorderStyle::None, width: 0.0, color: None, space: 0.0 };
    let style = BorderStyle::from_ooxml(style);
    let b = if style == BorderStyle::None {
        none
    } else {
        Border { style, width: p::f32(v, "width").unwrap_or(0.5).clamp(0.25, 6.0), color: p::str(v, "color").and_then(Rgb::parse), space: 0.0 }
    };
    let whole = scope == Some("table")
        || s.selected_table_cells().is_some_and(|t| t.whole_table)
        || (scope.is_none() && s.selected_table_cells().is_none() && s.sel.is_collapsed());
    let cells = selected_cells(s).into_iter().collect();
    with_table(s, |t| {
        // DOCX left/right are logical start/end in a mirrored table. Commands and pointer
        // coordinates name the visible edge on the page.
        let kind = match (kind, t.props.bidi_visual) {
            ("left", true) => "right",
            ("right", true) => "left",
            _ => kind,
        };
        if !whole {
            selection_borders(t, &cells, kind, b, none);
            return;
        }
        let mut cur = t.props.borders.unwrap_or_default();
        match kind {
            "all" => cur = Borders::all(b),
            "none" => cur = Borders::all(none),
            "outside" => {
                cur.top = Some(b);
                cur.bottom = Some(b);
                cur.left = Some(b);
                cur.right = Some(b);
                cur.between = Some(none);
                cur.inside_v = Some(none);
            }
            "inside" => {
                cur.between = Some(b);
                cur.inside_v = Some(b);
                cur.top = Some(none);
                cur.bottom = Some(none);
                cur.left = Some(none);
                cur.right = Some(none);
            }
            "insideHorizontal" => cur.between = Some(b),
            "insideVertical" => cur.inside_v = Some(b),
            "top" => cur.top = Some(b),
            "bottom" => cur.bottom = Some(b),
            "left" => cur.left = Some(b),
            "right" => cur.right = Some(b),
            _ => {}
        }
        t.props.borders = Some(cur);
        if ["all", "none", "outside", "inside"].contains(&kind) {
            for c in t.rows.iter_mut().flat_map(|r| &mut r.cells) {
                c.props.borders = None;
            }
        } else {
            let all = t.rows.iter().enumerate().flat_map(|(r, row)| (0..row.cells.len()).map(move |c| (r, c))).collect();
            selection_borders(t, &all, kind, b, none);
        }
    })
}

fn set_border_edge(borders: &mut Borders, side: usize, border: Border) {
    match side {
        0 => borders.top = Some(border),
        1 => borders.right = Some(border),
        2 => borders.bottom = Some(border),
        3 => borders.left = Some(border),
        _ => {}
    }
}

/// Update selected edges and the other side of each shared edge. Other cell edges retain their
/// direct formatting, and an explicit nil on both sides prevents style borders reappearing.
fn selection_borders(t: &mut Table, cells: &std::collections::BTreeSet<(usize, usize)>, kind: &str, b: Border, none: Border) {
    let (Some(&(r0, _)), Some(&(r1, _))) = (cells.first(), cells.last()) else { return };
    let mut g0 = t.grid.len();
    let mut g1 = 0;
    for &(r, c) in cells {
        let start = t.grid_col(r, c);
        g0 = g0.min(start);
        g1 = g1.max(start.saturating_add(t.rows.get(r).and_then(|r| r.cells.get(c)).map_or(1, doc4me_doc::Cell::span)));
    }
    for &(r, c) in cells {
        let Some(cell) = t.rows.get(r).and_then(|r| r.cells.get(c)) else { continue };
        let left = t.grid_col(r, c);
        let right = left.saturating_add(cell.span());
        let outer = [r == r0, right == g1, r == r1, left == g0];
        for (side, outside) in outer.into_iter().enumerate() {
            let edge = match kind {
                "all" => Some(b),
                "none" => Some(none),
                "outside" => Some(if outside { b } else { none }),
                "inside" => Some(if outside { none } else { b }),
                "insideHorizontal" if side.is_multiple_of(2) && !outside => Some(b),
                "insideVertical" if !side.is_multiple_of(2) && !outside => Some(b),
                "top" if side == 0 && outside => Some(b),
                "right" if side == 1 && outside => Some(b),
                "bottom" if side == 2 && outside => Some(b),
                "left" if side == 3 && outside => Some(b),
                _ => None,
            };
            let Some(edge) = edge else { continue };
            let mut neighbours = Vec::new();
            if side == 1 || side == 3 {
                let g = if side == 1 { Some(right) } else { left.checked_sub(1) };
                if let Some(cc) = g.and_then(|g| t.cell_at_grid(r, g)) {
                    neighbours.push((r, cc));
                }
            } else {
                let rr = if side == 0 { r.checked_sub(1) } else { r.checked_add(1) };
                if let Some(rr) = rr {
                    for g in left..right.min(t.grid.len()) {
                        if let Some(cc) = t.cell_at_grid(rr, g)
                            && neighbours.last() != Some(&(rr, cc))
                        {
                            neighbours.push((rr, cc));
                        }
                    }
                }
            }
            if let Some(cell) = t.rows.get_mut(r).and_then(|r| r.cells.get_mut(c)) {
                set_border_edge(cell.props.borders.get_or_insert_with(Borders::default), side, edge);
            }
            for (rr, cc) in neighbours {
                if !cells.contains(&(rr, cc))
                    && let Some(cell) = t.rows.get_mut(rr).and_then(|r| r.cells.get_mut(cc))
                {
                    set_border_edge(cell.props.borders.get_or_insert_with(Borders::default), (side + 2) % 4, edge);
                }
            }
        }
    }
}

fn cell_text(c: &doc4me_doc::Cell) -> String {
    c.blocks.iter().filter_map(|b| b.as_para().map(|p| p.plain_text())).collect::<Vec<_>>().join("\n")
}

fn sort_table(s: &mut Session, v: &Value) -> CmdResult {
    let col = usize::try_from(p::u64(v, "column").unwrap_or(0)).map_err(|_| CmdError::Params("column is outside the table".into()))?;
    let desc = p::bool(v, "descending").unwrap_or(false);
    let header = p::bool(v, "header");
    let (tp, _, _) = cell(s)?;
    let t = s.doc.table(s.sel.focus.story, &tp).ok_or_else(|| CmdError::Disabled("no table".into()))?;
    if col >= t.grid.len() {
        return Err(CmdError::Params("column is outside the table".into()));
    }
    let selected: std::collections::BTreeSet<_> = selected_cells(s).into_iter().map(|(r, _)| r).collect();
    let range = if s.selected_table_cells().is_some() || !s.sel.is_collapsed() {
        (selected.first().copied().unwrap_or(0), selected.last().copied().unwrap_or(0).saturating_add(1))
    } else {
        (0, t.rows.len())
    };
    if t.rows.iter().take(range.1).skip(range.0).flat_map(|r| &r.cells).any(|c| c.props.vmerge != doc4me_doc::props::VMerge::None) {
        return Err(CmdError::Disabled("split vertically merged cells before sorting their rows".into()));
    }
    with_table(s, |t| {
        let prefix = t.rows.iter().take_while(|r| r.props.header).count();
        let skip = match header {
            Some(false) => 0,
            Some(true) => prefix.max(1),
            None => prefix,
        };
        let start = range.0.max(skip).min(range.1);
        let Some(body) = t.rows.get_mut(start..range.1) else { return };
        let header_flags: Vec<_> = body.iter().map(|r| r.props.header).collect();
        let key = |row: &doc4me_doc::Row| {
            let mut g = row.props.grid_before as usize;
            for cell in &row.cells {
                if col >= g && col < g.saturating_add(cell.span()) {
                    return cell_text(cell);
                }
                g = g.saturating_add(cell.span());
            }
            String::new()
        };
        body.sort_by(|a, b| {
            let ka = key(a);
            let kb = key(b);
            let o = match (ka.trim().replace(',', "").parse::<f64>(), kb.trim().replace(',', "").parse::<f64>()) {
                (Ok(x), Ok(y)) => x.total_cmp(&y),
                _ => ka.to_lowercase().cmp(&kb.to_lowercase()),
            };
            if desc { o.reverse() } else { o }
        });
        for (row, header) in body.iter_mut().zip(header_flags) {
            row.props.header = header;
        }
    })
}

fn to_text(s: &mut Session, v: &Value) -> CmdResult {
    let (tp, _, _) = cell(s)?;
    let sep = match p::str(v, "separator").unwrap_or("tab") {
        "comma" => ",",
        "paragraph" => "\n",
        "tab" => "\t",
        x => return Err(CmdError::Params(format!("unknown separator `{x}`"))),
    };
    let story = s.sel.focus.story;
    let t = s.doc.table(story, &tp).cloned().ok_or_else(|| CmdError::Failed("no table".into()))?;
    let wrapped = |w: &doc4me_doc::ControlWrap| !w.open.is_empty() || w.close != 0;
    if wrapped(&t.controls) || t.rows.iter().any(|r| wrapped(&r.controls) || r.cells.iter().any(|c| wrapped(&c.controls))) {
        return Err(CmdError::Disabled("remove the enclosing content control before converting this table".into()));
    }
    let mut out = Vec::new();
    for row in &t.rows {
        let mut row_blocks = Vec::new();
        for (c, cell) in row.cells.iter().enumerate() {
            let mut blocks: Vec<Block> = cell.blocks.iter().map(|b| (**b).clone()).collect();
            if blocks.is_empty() {
                blocks.push(Block::Para(Paragraph::new()));
            }
            if c > 0 && sep != "\n" {
                if let Some(Block::Para(left)) = row_blocks.last_mut() {
                    left.insert_text(left.len(), sep, &CharProps::default())?;
                    if matches!(blocks.first(), Some(Block::Para(_)))
                        && let Block::Para(right) = blocks.remove(0)
                    {
                        left.append(right);
                    }
                } else {
                    row_blocks.push(Block::Para(Paragraph::with_text(sep, CharProps::default())));
                }
            }
            row_blocks.extend(blocks);
        }
        out.extend(row_blocks);
    }
    if out.is_empty() {
        out.push(Block::Para(Paragraph::new()));
    }
    let blocks = s.doc.container_mut(story, &tp)?;
    if tp.last() >= blocks.len() {
        return Err(CmdError::Disabled("the table is gone".into()));
    }
    blocks.splice(tp.last()..tp.last() + 1, out.into_iter().map(std::sync::Arc::new));
    s.sel = Selection::caret(s.doc.clamp(&Pos { story, path: tp, off: 0 }));
    s.table_selection = None;
    sel_result(s)
}

/// `=SUM(ABOVE)`, `=SUM(LEFT)`, `=AVERAGE(ABOVE)`, `=COUNT(...)`, `=MAX`, `=MIN`, `=PRODUCT`.
fn formula(s: &mut Session, v: &Value) -> CmdResult {
    let (tp, r, c) = cell(s)?;
    let f = p::str(v, "formula").unwrap_or("=SUM(ABOVE)").to_ascii_uppercase();
    let valid = ["SUM", "AVERAGE", "COUNT", "MAX", "MIN", "PRODUCT"]
        .iter()
        .any(|name| ["ABOVE", "LEFT"].iter().any(|direction| f == format!("={name}({direction})")));
    if !valid {
        return Err(CmdError::Params("use SUM, AVERAGE, COUNT, MAX, MIN or PRODUCT with ABOVE or LEFT".into()));
    }
    let t = s.doc.table(s.sel.focus.story, &tp).cloned().ok_or_else(|| CmdError::Failed("no table".into()))?;
    let nums: Vec<f64> = if f.contains("LEFT") {
        t.rows
            .get(r)
            .map(|row| row.cells.iter().take(c).filter_map(|x| cell_text(x).trim().replace([',', '$'], "").parse().ok()).collect())
            .unwrap_or_default()
    } else {
        t.rows.iter().take(r).filter_map(|row| row.cells.get(c)).filter_map(|x| cell_text(x).trim().replace([',', '$'], "").parse().ok()).collect()
    };
    let nums: Vec<f64> = nums.into_iter().filter(|n| n.is_finite()).collect();
    let val = if nums.is_empty() {
        0.0
    } else if f.starts_with("=AVERAGE") {
        if nums.is_empty() { 0.0 } else { nums.iter().sum::<f64>() / nums.len() as f64 }
    } else if f.starts_with("=COUNT") {
        nums.len() as f64
    } else if f.starts_with("=MAX") {
        nums.iter().copied().fold(f64::MIN, f64::max)
    } else if f.starts_with("=MIN") {
        nums.iter().copied().fold(f64::MAX, f64::min)
    } else if f.starts_with("=PRODUCT") {
        nums.iter().product()
    } else {
        nums.iter().sum()
    };
    if !val.is_finite() {
        return Err(CmdError::Failed("the formula result is outside the numeric range".into()));
    }
    let text = if val.fract() == 0.0 && val.abs() < 1e15 { format!("{}", val as i64) } else { format!("{val:.2}") };
    let props = s.typing_props();
    let at = s.sel.focus.clone();
    let end = s.doc.insert_object(
        &at,
        doc4me_doc::InlineObject::Field { instr: f.clone(), result: text.clone(), locked: false, code: false, form: None },
        &props,
    )?;
    s.sel = Selection::caret(end);
    Ok(json!({"result": text}))
}

fn select_cells(s: &mut Session, tp: &Path, which: Option<(usize, Option<usize>)>) -> CmdResult {
    let story = s.sel.focus.story;
    let Some(t) = s.doc.table(story, tp) else { return sel_result(s) };
    let cells = t
        .rows
        .iter()
        .enumerate()
        .flat_map(|(r, row)| {
            row.cells
                .iter()
                .enumerate()
                .filter_map(move |(c, _)| which.is_none_or(|(rr, cc)| r == rr && cc.is_none_or(|cc| c == cc)).then_some((r, c)))
        })
        .collect();
    set_cell_selection(s, tp, cells, which.is_none())
}

fn select_column(s: &mut Session, _: &Value) -> CmdResult {
    let (tp, r, c) = cell(s)?;
    let t = s.doc.table(s.sel.focus.story, &tp).ok_or_else(|| CmdError::Disabled("no table".into()))?;
    let start = t.grid_col(r, c);
    let stop = start.saturating_add(t.rows.get(r).and_then(|r| r.cells.get(c)).map_or(1, doc4me_doc::Cell::span));
    let mut cells = std::collections::BTreeSet::new();
    for (r, row) in t.rows.iter().enumerate() {
        for (c, cl) in row.cells.iter().enumerate() {
            let left = t.grid_col(r, c);
            if left < stop && left.saturating_add(cl.span()) > start {
                cells.insert((r, c));
            }
        }
    }
    set_cell_selection(s, &tp, cells, false)
}

pub(crate) fn set_cell_selection(s: &mut Session, tp: &Path, cells: std::collections::BTreeSet<(usize, usize)>, whole: bool) -> CmdResult {
    let story = s.sel.focus.story;
    let (Some(&(r0, c0)), Some(&(r1, c1))) = (cells.first(), cells.last()) else {
        return Err(CmdError::Disabled("the table has no cells".into()));
    };
    let mut first = tp.0.clone();
    first.extend([r0 as u32, c0 as u32]);
    let mut last = tp.0.clone();
    last.extend([r1 as u32, c1 as u32]);
    let paths = s.doc.para_paths(story);
    let a = paths.iter().find(|p| p.0.starts_with(&first)).cloned().ok_or_else(|| CmdError::Failed("the first cell has no paragraph".into()))?;
    let b = paths.iter().rfind(|p| p.0.starts_with(&last)).cloned().ok_or_else(|| CmdError::Failed("the last cell has no paragraph".into()))?;
    let off = s.doc.para(story, &b).map_or(0, Paragraph::len);
    s.sel = Selection { anchor: Pos { story, path: a, off: 0 }, focus: Pos { story, path: b, off } };
    s.column = None;
    s.table_selection = Some(std::sync::Arc::new(crate::TableSelection { sel: s.sel.clone(), story, table: tp.clone(), cells, whole_table: whole }));
    sel_result(s)
}

/// Copy the selected cells as a table, rather than flattening their paragraphs into text.
pub(crate) fn copy_selection(s: &Session) -> Result<Option<doc4me_doc::edit::Fragment>, CmdError> {
    let Some(sel) = s.selected_table_cells() else { return Ok(None) };
    let source = s.doc.table(sel.story, &sel.table).ok_or_else(|| CmdError::Disabled("no table selected".into()))?;
    let mut copied = source.clone();
    if !sel.whole_table {
        let mut start = source.grid.len();
        let mut stop = 0;
        for &(r, c) in &sel.cells {
            if let Some(cell) = source.rows.get(r).and_then(|r| r.cells.get(c)) {
                let g = source.grid_col(r, c);
                start = start.min(g);
                stop = stop.max(g.saturating_add(cell.span()).min(source.grid.len()));
            }
        }
        let grid =
            source.grid.get(start..stop).filter(|g| !g.is_empty()).ok_or_else(|| CmdError::Failed("the selection has no grid columns".into()))?;
        copied.grid = grid.to_vec();
        copied.rows.clear();
        for (r, row) in source.rows.iter().enumerate() {
            let mut out = row.clone();
            out.cells = row
                .cells
                .iter()
                .enumerate()
                .filter(|(c, _)| sel.cells.contains(&(r, *c)))
                .map(|(c, cell)| {
                    let mut copied = cell.clone();
                    let g = source.grid_col(r, c);
                    let previous_selected =
                        r.checked_sub(1).and_then(|rr| source.cell_at_grid(rr, g).map(|cc| sel.cells.contains(&(rr, cc)))).unwrap_or(false);
                    if cell.props.vmerge == doc4me_doc::props::VMerge::Continue && !previous_selected {
                        for rr in (0..r).rev() {
                            let origin = source.cell_at_grid(rr, g).and_then(|cc| source.rows.get(rr).and_then(|r| r.cells.get(cc)));
                            let Some(origin) = origin else { break };
                            if origin.props.vmerge == doc4me_doc::props::VMerge::Restart {
                                copied.blocks = origin.blocks.clone();
                                break;
                            }
                            if origin.props.vmerge != doc4me_doc::props::VMerge::Continue {
                                break;
                            }
                        }
                        copied.props.vmerge = doc4me_doc::props::VMerge::Restart;
                    }
                    copied
                })
                .collect();
            let Some((first, _)) = row.cells.iter().enumerate().find(|(c, _)| sel.cells.contains(&(r, *c))) else { continue };
            let last = row.cells.iter().enumerate().rfind(|(c, _)| sel.cells.contains(&(r, *c))).map_or(first, |(c, _)| c);
            out.props.grid_before = source.grid_col(r, first).saturating_sub(start) as u32;
            let end = source.grid_col(r, last).saturating_add(row.cells.get(last).map_or(1, doc4me_doc::Cell::span));
            out.props.grid_after = stop.saturating_sub(end) as u32;
            copied.rows.push(out);
        }
        copied.props.width_pct = None;
        copied.props.width = Some(copied.grid.iter().sum());
    }
    Ok(Some(s.doc.blocks_fragment(vec![Block::Table(copied)])))
}

/// Delete cell contents without joining adjacent cells or flattening the table.
pub(crate) fn clear_selection(s: &mut Session) -> Result<Option<Pos>, CmdError> {
    if s.selected_table_cells().is_none() {
        return Ok(None);
    }
    let Some(sel) = s.table_selection.clone() else { return Ok(None) };
    let prefix = sel.table.0.len();
    let mut ranges = std::collections::BTreeMap::new();
    let mut paragraphs = Vec::new();
    for path in s.doc.para_paths(sel.story) {
        if !path.0.starts_with(&sel.table.0) {
            continue;
        }
        let pair = path.0.get(prefix).zip(path.0.get(prefix + 1)).map(|(&r, &c)| (r as usize, c as usize));
        if let Some(pair) = pair.filter(|p| sel.cells.contains(p)) {
            let a = Pos { story: sel.story, path: path.clone(), off: 0 };
            let b = Pos { off: s.doc.para(sel.story, &path).map_or(0, Paragraph::len), ..a.clone() };
            ranges.entry(pair).and_modify(|r: &mut (Pos, Pos)| r.1 = b.clone()).or_insert((a.clone(), b.clone()));
            paragraphs.push((a, b));
        }
    }
    for (a, b) in ranges.values() {
        super::controls::check_delete(s, a, b)?;
    }
    if s.doc.settings.track_changes {
        for (a, b) in paragraphs.iter().rev() {
            super::track_delete(s, a, b)?;
        }
    } else {
        let t = s.doc.table_mut(sel.story, &sel.table)?;
        for &(r, c) in &sel.cells {
            if let Some(cell) = t.rows.get_mut(r).and_then(|r| r.cells.get_mut(c)) {
                let mut empty = Paragraph::new();
                if let Some(p) = cell.blocks.iter().find_map(|b| b.as_para()) {
                    empty.props = p.props.clone();
                    empty.mark = p.mark.clone();
                }
                cell.blocks = vec![para_block(empty)];
            }
        }
    }
    let mut path = sel.table.0.clone();
    let (r, c) = sel.cells.first().copied().ok_or_else(|| CmdError::Disabled("no cells selected".into()))?;
    path.extend([r as u32, c as u32, 0]);
    let at = s.doc.clamp(&Pos { story: sel.story, path: Path(path), off: 0 });
    s.sel = Selection::caret(at.clone());
    s.table_selection = None;
    Ok(Some(at))
}

pub(crate) fn remove_selected_table(s: &mut Session) -> Result<Option<Pos>, CmdError> {
    let Some(sel) = s.selected_table_cells().filter(|t| t.whole_table).cloned() else { return Ok(None) };
    super::controls::check_delete(s, &sel.sel.anchor, &sel.sel.focus)?;
    s.doc.remove_block(sel.story, &sel.table)?;
    let at = s.doc.clamp(&Pos { story: sel.story, path: sel.table, off: 0 });
    s.sel = Selection::caret(at.clone());
    s.table_selection = None;
    Ok(Some(at))
}

pub(crate) fn prepare_replacement(s: &mut Session) -> Result<(), CmdError> {
    let Some(sel) = s.selected_table_cells().filter(|t| t.whole_table).cloned() else { return Ok(()) };
    super::controls::check_delete(s, &sel.sel.anchor, &sel.sel.focus)?;
    let blocks = s.doc.container_mut(sel.story, &sel.table)?;
    let block = blocks.get_mut(sel.table.last()).ok_or_else(|| CmdError::Disabled("the selected table is gone".into()))?;
    *block = para_block(Paragraph::new());
    s.sel = Selection::caret(Pos { story: sel.story, path: sel.table, off: 0 });
    s.table_selection = None;
    Ok(())
}

fn from_text(s: &mut Session, v: &Value) -> CmdResult {
    let sep = match p::str(v, "separator").unwrap_or("tab") {
        "tab" => '\t',
        "comma" => ',',
        x => return Err(CmdError::Params(format!("unknown separator `{x}`"))),
    };
    let (a, b) = s.sel.ordered();
    if a.path.parent() != b.path.parent() || a.story != b.story {
        return Err(CmdError::Failed("select paragraphs to convert".into()));
    }
    let count = b.path.last().saturating_sub(a.path.last()).saturating_add(1);
    if count > doc4me_doc::table::MAX_ROWS {
        return Err(CmdError::Failed("too many rows to convert to a table".into()));
    }
    let mut paragraphs = Vec::new();
    for i in a.path.last()..=b.path.last() {
        let p = s.doc.para(a.story, &a.path.with_last(i)).ok_or_else(|| CmdError::Failed("select only paragraphs to convert".into()))?;
        if p.section.is_some() {
            return Err(CmdError::Disabled("convert paragraphs within a single section".into()));
        }
        paragraphs.push(p.clone());
    }
    let cols = paragraphs.iter().map(|p| p.text.chars().filter(|c| *c == sep).count().saturating_add(1)).max().unwrap_or(1);
    if cols > doc4me_doc::table::MAX_COLS {
        return Err(CmdError::Failed("a table can have at most 63 columns".into()));
    }
    let width = super::page::sect(s).text_width();
    let mut t = Table::new(paragraphs.len().max(1), cols, width);
    for (r, mut p) in paragraphs.into_iter().enumerate() {
        let mut c = 0;
        while let Some(at) = p.text.find(sep) {
            let mut tail = p.split_off(at)?;
            tail.delete(0, sep.len_utf8())?;
            if let Some(cell) = t.rows.get_mut(r).and_then(|r| r.cells.get_mut(c)) {
                cell.blocks = vec![para_block(p)];
            }
            p = tail;
            c += 1;
        }
        if let Some(cell) = t.rows.get_mut(r).and_then(|r| r.cells.get_mut(c)) {
            cell.blocks = vec![para_block(p)];
        }
    }
    let blocks = s.doc.container_mut(a.story, &a.path)?;
    let end = b
        .path
        .last()
        .checked_add(1)
        .filter(|end| *end <= blocks.len())
        .ok_or_else(|| CmdError::Failed("selection is outside the document".into()))?;
    blocks.splice(a.path.last()..end, [std::sync::Arc::new(Block::Table(t))]);
    let mut first = a.path.0.clone();
    first.extend([0, 0, 0]);
    s.sel = Selection::caret(Pos { story: a.story, path: Path(first), off: 0 });
    s.table_selection = None;
    sel_result(s)
}

fn quick_table(s: &mut Session, v: &Value) -> CmdResult {
    let kind = p::str(v, "kind").unwrap_or("tabular");
    let data: Vec<Vec<&str>> = match kind {
        "calendar" => {
            let mut rows = vec![vec!["M", "T", "W", "T", "F", "S", "S"]];
            let days = [
                "", "", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22",
                "23", "24", "25", "26", "27", "28", "29", "30", "31", "", "", "",
            ];
            for ch in days.chunks(7) {
                rows.push(ch.to_vec());
            }
            rows
        }
        "matrix" => vec![
            vec!["", "Column A", "Column B", "Column C"],
            vec!["Row 1", "1", "2", "3"],
            vec!["Row 2", "4", "5", "6"],
            vec!["Row 3", "7", "8", "9"],
        ],
        _ => vec![
            vec!["Item", "Needed"],
            vec!["Books", "1"],
            vec!["Magazines", "3"],
            vec!["Notebooks", "1"],
            vec!["Paper pads", "1"],
            vec!["Pens", "3"],
            vec!["Pencils", "2"],
        ],
    };
    let cols = data.iter().map(Vec::len).max().unwrap_or(1);
    let width = super::page::sect(s).text_width();
    let mut t = Table::new(data.len(), cols, width);
    t.props.style = Some(if kind == "calendar" { "GridTable1Light" } else { "GridTable4AccentBlue" }.into());
    for (r, row) in data.iter().enumerate() {
        for (c, txt) in row.iter().enumerate() {
            if let Some(cl) = t.rows.get_mut(r).and_then(|x| x.cells.get_mut(c)) {
                cl.blocks = vec![para_block(Paragraph::with_text(txt, Default::default()))];
            }
        }
    }
    if let Some(r) = t.rows.first_mut() {
        r.props.header = true;
    }
    let story = s.sel.focus.story;
    let at = super::delete_selection(s)?;
    let new = s.doc.split_paragraph(&at)?;
    s.doc.insert_block(story, &new.path, Block::Table(t))?;
    let mut first = new.path.0.clone();
    first.extend([0, 0, 0]);
    s.sel = Selection::caret(Pos { story, path: Path(first), off: 0 });
    let _ = StoryRef::Body;
    sel_result(s)
}
