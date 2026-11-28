//! The rest of Word's table mouse (resizing is `table_resize`): selection areas just above a
//! column, left of a row and along a cell's left edge inside it select the column, row or cell,
//! and a drag from one selects every one up to the pointer; insert buttons left of a row border
//! and above a column border add a row or column there; the move handle just past the table's
//! top-left corner selects the table or, dragged, moves it (the table then floats where it's
//! dropped, the text wrapping around it, as in Word); a double-click on a column border fits the
//! column left of it to its contents. The app sends each click or finished drag here in page
//! points, so each is one command (one undo step when it changes the document).

use doc4me_doc::para::Anchor;
use doc4me_doc::props::TableFloat;
use doc4me_doc::{Document, Path, Pos, StoryRef, Table};
use doc4me_geom::Rect;
use doc4me_layout::{DocLayout, Placed};
use serde_json::Value;

use super::table_resize::{self, CellBox, Edge, REACH, ResizeTarget};
use super::{sel_result, table};
use crate::{CmdError, CmdResult, CommandSpec, Selection, Session, p};

/// The column selection area: from this far above a table's top border (points) to this far
/// below it.
const COLUMN_ABOVE: f32 = 6.0;
const COLUMN_BELOW: f32 = 1.5;
/// How far left of a table (points) the row selection area and the row insert buttons reach.
const ROW_LEFT: f32 = 24.0;
/// The cell selection area's width inside a cell's left edge (points), at the least; it starts
/// past the column border's resize reach.
const CELL_EDGE: f32 = 5.5;
/// The insert buttons answer this near a border (points), and above the table this far out.
const INSERT_REACH: f32 = 3.5;
const INSERT_OUT: f32 = 14.0;
/// An insert button's centre sits this far out from the table's edge (points).
const BUTTON_OUT: f32 = 8.0;
/// The move handle's side and its gap from the table's top-left corner (points).
const HANDLE: f32 = 8.0;
const HANDLE_GAP: f32 = 2.0;
/// The farthest a drag moves a table (points; Word's 22 inches).
const MAX_MOVE: f32 = 1584.0;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("table.selectAt", "Select Table Cells by Clicking", "Table Layout › Table › Select", select_at)
            .params(
                r#"{"page": n, "x": pt, "y": pt, "toX"?: pt, "toY"?: pt, "add"?: bool, "reach"?: pt} (a click in page points, 0-based page, on a selection area of a table in the caret's story: just above a column selects the column, left of a row the row, along a cell's left edge inside it the cell, the move handle past the top-left corner the table; with toX/toY (a drag) every column, row or cell from there to that point; with `add` (Cmd/Ctrl) a column joins the columns already selected, side by side or not, or leaves them if it was one. `reach` (default 3) is the column borders' resize reach, which the cell areas keep clear of.)"#,
            )
            .pure(),
        CommandSpec::new("table.insertAt", "Insert Row or Column by Clicking", "Table Layout › Rows & Columns", insert_at).params(
            r#"{"page": n, "x": pt, "y": pt, "reach"?: pt} (a click on a table's insert button: left of a row border adds a row there, above a column border a column; the new row or column is selected, as in Word)"#,
        ),
        CommandSpec::new("table.moveDrag", "Move Table by Dragging", "Table Layout › Table", move_drag).params(
            r#"{"page": n, "x": pt, "y": pt, "toX": pt, "toY": pt, "reach"?: pt} (a drag from the move handle past a table's top-left corner: the table floats where it's dropped, the text wrapping around it; tables in the main text only)"#,
        ),
        CommandSpec::new("table.fitColumn", "AutoFit Column by Double-Clicking", "Table Layout › Cell Size", fit_column).params(
            r#"{"page": n, "x": pt, "y": pt, "reach"?: pt} (a double-click on a column border: the column left of it fits its contents and the columns right of it keep their widths, moving with the border, so the table narrows or widens (up to the right margin), as in Word)"#,
        ),
    ]
}

/// What the pointer is over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spot {
    /// The move handle.
    Handle,
    /// The insert button of horizontal border `n` (0 = the table's top, `n` = the bottom of row
    /// `n - 1`).
    InsertRow(usize),
    /// The insert button of grid line `n` (0 = the table's left edge).
    InsertColumn(usize),
    /// The selection area above grid column `n`.
    Column(usize),
    /// The selection area left of row `n`.
    Row(usize),
    /// The selection area along cell (`row`, `cell`)'s left edge.
    Cell(usize, usize),
}

/// A spot and the table's drawn geometry on the page (`geom.edge` means nothing here).
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub spot: Spot,
    pub geom: ResizeTarget,
}

/// An insert button: its centre and the border it inserts at, end to end (page points).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InsertMark {
    pub center: (f32, f32),
    pub from: (f32, f32),
    pub to: (f32, f32),
}

/// The move handle's square (page points), just past the table's top-left corner.
pub fn handle(geom: &ResizeTarget) -> Rect {
    Rect::new(geom.left() - HANDLE_GAP - HANDLE, geom.top() - HANDLE_GAP - HANDLE, HANDLE, HANDLE)
}

/// Whether the table's first row is on this page (its move handle is there).
fn starts_here(geom: &ResizeTarget) -> bool {
    geom.rows.first().is_some_and(|r| r.row == 0)
}

fn on_handle(geom: &ResizeTarget, x: f32, y: f32, reach: f32) -> bool {
    let h = handle(geom);
    starts_here(geom) && x >= h.x - reach / 2.0 && x <= h.right() + reach / 2.0 && y >= h.y - reach / 2.0 && y <= h.bottom() + reach / 2.0
}

fn clean_reach(reach: f32) -> f32 {
    if reach.is_finite() { reach.clamp(0.5, 24.0) } else { REACH }
}

/// Of `lines` (index, position), the one nearest `at` within `reach`.
fn nearest(lines: impl Iterator<Item = (usize, f32)>, at: f32, reach: f32) -> Option<usize> {
    lines.map(|(i, v)| (i, (at - v).abs())).filter(|(_, d)| *d <= reach).min_by(|a, b| a.1.total_cmp(&b.1)).map(|(i, _)| i)
}

/// The grid column at page x `x` (the first or the last beyond the table).
fn column_at(geom: &ResizeTarget, x: f32) -> usize {
    let n = geom.cols.len().saturating_sub(1);
    geom.cols.iter().skip(1).take_while(|cx| **cx <= x).count().min(n.saturating_sub(1))
}

/// The drawn row at page y `y` (the first or the last beyond the table).
fn row_at(geom: &ResizeTarget, y: f32) -> usize {
    geom.rows.iter().find(|r| y < r.bottom).or(geom.rows.last()).map_or(0, |r| r.row)
}

/// The spot of one table at (`x`, `y`): the handle, then the insert buttons, then the selection
/// areas. `cells` are its cell boxes on the page.
fn spot_in(geom: &ResizeTarget, cells: &[&CellBox], x: f32, y: f32, reach: f32) -> Option<Spot> {
    if on_handle(geom, x, y, reach) {
        return Some(Spot::Handle);
    }
    let (left, right, top, bottom) = (geom.left(), geom.right(), geom.top(), geom.bottom());
    // Above the table: the insert buttons over its column borders, and the column areas.
    if y >= top - INSERT_OUT && y <= top + COLUMN_BELOW && x >= left - INSERT_REACH && x <= right + INSERT_REACH {
        let lines = geom.cols.iter().copied().enumerate().filter(|(i, _)| geom.rows.iter().any(|r| r.edges.contains(i)));
        if y < top
            && let Some(line) = nearest(lines, x, INSERT_REACH)
        {
            return Some(Spot::InsertColumn(line));
        }
        if y >= top - COLUMN_ABOVE && x >= left && x <= right {
            return Some(Spot::Column(column_at(geom, x)));
        }
    }
    // Left of the table: the insert buttons beside its row borders, and the row areas.
    if x >= left - ROW_LEFT && x < left && y >= top - INSERT_REACH && y <= bottom + INSERT_REACH {
        let borders = geom.rows.iter().flat_map(|r| [(r.row, r.top), (r.row + 1, r.bottom)]);
        if let Some(line) = nearest(borders, y, INSERT_REACH) {
            return Some(Spot::InsertRow(line));
        }
        if let Some(r) = geom.rows.iter().find(|r| y >= r.top && y <= r.bottom) {
            return Some(Spot::Row(r.row));
        }
    }
    // Inside a cell, along its left edge.
    let edge = CELL_EDGE.max(reach + 2.5);
    cells.iter().find(|b| x >= b.rect.x && x <= b.rect.x + edge && y >= b.rect.y && y <= b.rect.bottom()).map(|b| Spot::Cell(b.row, b.cell))
}

/// The spot under page point (`x`, `y`) of a table in `story`, of nested tables the innermost's.
/// None on a column or row border or the resize handle: those resize (`table_resize`). `reach`
/// is their resize reach (points).
pub fn target_at(l: &DocLayout, doc: &Document, story: StoryRef, page: usize, x: f32, y: f32, reach: f32) -> Option<Target> {
    if !x.is_finite() || !y.is_finite() {
        return None;
    }
    let reach = clean_reach(reach);
    if table_resize::resize_target(l, doc, page, x, y, reach).is_some() {
        return None;
    }
    let boxes = table_resize::cell_boxes(l, page);
    table_resize::tables_on(l, doc, page)
        .into_iter()
        .filter(|g| g.story == story)
        .filter_map(|geom| {
            let cells: Vec<&CellBox> = boxes.iter().filter(|b| b.story == geom.story && b.table == geom.table).collect();
            spot_in(&geom, &cells, x, y, reach).map(|spot| Target { spot, geom })
        })
        .max_by_key(|t| t.geom.table.0.len())
}

/// The table whose move handle shows with the pointer at (`x`, `y`): the innermost table of
/// `story` starting on this page that the pointer is over, or on its way to the handle.
pub fn handle_table(l: &DocLayout, doc: &Document, story: StoryRef, page: usize, x: f32, y: f32, reach: f32) -> Option<ResizeTarget> {
    if !x.is_finite() || !y.is_finite() {
        return None;
    }
    let pad = clean_reach(reach) / 2.0;
    table_resize::tables_on(l, doc, page)
        .into_iter()
        .filter(|g| {
            let h = handle(g);
            g.story == story && starts_here(g) && x >= h.x - pad && x <= g.right() && y >= h.y - pad && y <= g.bottom()
        })
        .max_by_key(|g| g.table.0.len())
}

/// The insert button a target is, if it's one.
pub fn insert_mark(t: &Target) -> Option<InsertMark> {
    let g = &t.geom;
    match t.spot {
        Spot::InsertRow(line) => {
            // The bottom of the row above the border, or the top of the first row.
            let y = g.rows.iter().find(|r| r.row + 1 == line).map(|r| r.bottom).or_else(|| g.rows.iter().find(|r| r.row == line).map(|r| r.top))?;
            Some(InsertMark { center: (g.left() - BUTTON_OUT, y), from: (g.left(), y), to: (g.right(), y) })
        }
        Spot::InsertColumn(line) => {
            let x = g.cols.get(line).copied()?;
            Some(InsertMark { center: (x, g.top() - BUTTON_OUT), from: (x, g.top()), to: (x, g.bottom()) })
        }
        _ => None,
    }
}

/// Whether dragging the move handle moves the table: tables in the main text, not nested ones.
pub fn movable(geom: &ResizeTarget) -> bool {
    geom.story == StoryRef::Body && geom.table.0.len() == 1
}

/// Where a move-handle drag by (`dx`, `dy`) puts the table's top-left corner on `page`: on the
/// page, its part there above the bottom margin.
pub fn moved_to(l: &DocLayout, page: usize, geom: &ResizeTarget, dx: f32, dy: f32) -> (f32, f32) {
    let clean = |d: f32| if d.is_finite() { d.clamp(-MAX_MOVE, MAX_MOVE) } else { 0.0 };
    let (w, h) = (geom.right() - geom.left(), geom.bottom() - geom.top());
    let (page_w, body_bottom) = l.pages.get(page).map_or((0.0, 0.0), |pg| (pg.w, pg.body.bottom()));
    let x = (geom.left() + clean(dx)).clamp(0.0, (page_w - w).max(0.0));
    let y = (geom.top() + clean(dy)).clamp(0.0, (body_bottom - h).max(0.0));
    (x, y)
}

/// Cells whose drawn grid span intersects a row/column rectangle (inclusive grid indices).
fn cells_in(t: &Table, rows: (usize, usize), cols: (usize, usize)) -> std::collections::BTreeSet<(usize, usize)> {
    let (r0, r1) = (rows.0.min(rows.1).min(t.rows.len().saturating_sub(1)), rows.0.max(rows.1).min(t.rows.len().saturating_sub(1)));
    let (c0, c1) = (cols.0.min(cols.1).min(t.cols().saturating_sub(1)), cols.0.max(cols.1).min(t.cols().saturating_sub(1)));
    t.rows
        .iter()
        .enumerate()
        .take(r1.saturating_add(1))
        .skip(r0)
        .flat_map(|(r, row)| {
            row.cells.iter().enumerate().filter_map(move |(c, cell)| {
                let left = t.grid_col(r, c);
                let right = left.saturating_add(cell.span());
                (left <= c1 && right > c0).then_some((r, c))
            })
        })
        .collect()
}

fn select_grid(s: &mut Session, story: StoryRef, tp: &Path, rows: (usize, usize), cols: (usize, usize), whole: bool) -> CmdResult {
    let t = s.doc.table(story, tp).ok_or_else(|| CmdError::Failed("there's no table there".into()))?;
    let cells = cells_in(t, rows, cols);
    table::set_cell_selection(s, tp, cells, whole)
}

/// Cmd/Ctrl on a column area toggles that whole grid-column span in the selected columns.
fn add_columns(s: &mut Session, story: StoryRef, tp: &Path, span: (usize, usize), last_row: usize) -> CmdResult {
    let t = s.doc.table(story, tp).cloned().ok_or_else(|| CmdError::Failed("there's no table there".into()))?;
    let current = s.selected_table_cells().filter(|sel| sel.story == story && sel.table == *tp).map(|sel| sel.cells.clone());
    let mut picked = std::collections::BTreeSet::new();
    if let Some(current) = current {
        for g in 0..t.cols() {
            let every_row = (0..=last_row).all(|r| {
                t.rows.get(r).is_some_and(|row| {
                    row.cells
                        .iter()
                        .enumerate()
                        .any(|(c, cell)| current.contains(&(r, c)) && t.grid_col(r, c) <= g && t.grid_col(r, c).saturating_add(cell.span()) > g)
                })
            });
            if every_row {
                picked.insert(g);
            }
        }
    }
    let span_cols: Vec<usize> = (span.0..=span.1).collect();
    if !picked.is_empty() && span_cols.iter().all(|g| picked.contains(g)) {
        for g in &span_cols {
            picked.remove(g);
        }
        if picked.is_empty() {
            picked.extend(span_cols);
        }
    } else {
        picked.extend(span_cols);
    }
    if picked.is_empty() {
        return Err(CmdError::Failed("there are no columns to select".into()));
    }
    let mut cells = std::collections::BTreeSet::new();
    for (r, row) in t.rows.iter().enumerate() {
        for (c, cell) in row.cells.iter().enumerate() {
            let left = t.grid_col(r, c);
            let right = left.saturating_add(cell.span());
            if picked.iter().any(|g| left <= *g && right > *g) {
                cells.insert((r, c));
            }
        }
    }
    table::set_cell_selection(s, tp, cells, false)
}

/// The click: `page`, `x`, `y` and the resize reach.
fn point(v: &Value) -> Result<(usize, f32, f32, f32), CmdError> {
    let page =
        p::u64(v, "page").map(|n| usize::try_from(n).unwrap_or(usize::MAX)).ok_or_else(|| CmdError::Params("`page` (number) is required".into()))?;
    Ok((page, p::req_f32(v, "x")?, p::req_f32(v, "y")?, p::f32(v, "reach").unwrap_or(REACH)))
}

fn target(s: &mut Session, page: usize, x: f32, y: f32, reach: f32) -> Result<Target, CmdError> {
    let l = s.layout();
    if l.pages.get(page).is_none() {
        return Err(CmdError::Params(format!("no page {page}")));
    }
    target_at(&l, &s.doc, s.sel.focus.story, page, x, y, reach)
        .ok_or_else(|| CmdError::Failed("there's no table selection area or button there".into()))
}

fn select_at(s: &mut Session, v: &Value) -> CmdResult {
    let (page, x, y, reach) = point(v)?;
    let Target { spot, geom } = target(s, page, x, y, reach)?;
    let to = p::f32(v, "toX").zip(p::f32(v, "toY"));
    let story = geom.story;
    let t = s.doc.table(story, &geom.table).ok_or_else(|| CmdError::Failed("there's no table there".into()))?;
    let (last_row, last_col) = (t.rows.len().saturating_sub(1), t.cols().saturating_sub(1));
    if let (Spot::Column(g), true) = (spot, p::bool(v, "add").unwrap_or(false)) {
        let g1 = to.map_or(g, |(tx, _)| column_at(&geom, tx));
        return add_columns(s, story, &geom.table, (g.min(g1), g.max(g1)), last_row);
    }
    let (rows, cols, whole) = match spot {
        Spot::Column(g) => ((0, last_row), (g, to.map_or(g, |(tx, _)| column_at(&geom, tx))), false),
        Spot::Row(r) => ((r, to.map_or(r, |(_, ty)| row_at(&geom, ty))), (0, last_col), false),
        Spot::Cell(r, c) => {
            let g0 = t.grid_col(r, c);
            let g1 = g0.saturating_add(t.rows.get(r).and_then(|row| row.cells.get(c)).map_or(1, doc4me_doc::Cell::span)).saturating_sub(1);
            match to {
                Some((tx, ty)) => {
                    let g = column_at(&geom, tx);
                    ((r, row_at(&geom, ty)), (g0.min(g), g1.max(g)), false)
                }
                None => ((r, r), (g0, g1), false),
            }
        }
        Spot::Handle => ((0, last_row), (0, last_col), true),
        Spot::InsertRow(_) | Spot::InsertColumn(_) => return Err(CmdError::Failed("there's an insert button there, not a selection area".into())),
    };
    select_grid(s, story, &geom.table, rows, cols, whole)
}

fn beside_line(t: &Table, line: usize) -> Option<(usize, usize, bool)> {
    t.rows.iter().enumerate().find_map(|(r, row)| {
        let mut g = 0usize;
        for (c, cell) in row.cells.iter().enumerate() {
            if g == line {
                return Some((r, c, false));
            }
            g = g.saturating_add(cell.span());
            if g == line && c + 1 == row.cells.len() {
                return Some((r, c, true));
            }
        }
        None
    })
}

fn insert_at(s: &mut Session, v: &Value) -> CmdResult {
    let (page, x, y, reach) = point(v)?;
    let Target { spot, geom } = target(s, page, x, y, reach)?;
    let story = geom.story;
    let (r, c, after, rows, line) = match spot {
        Spot::InsertRow(line) => match line.checked_sub(1) {
            Some(above) => (above, 0, true, true, line),
            None => (0, 0, false, true, line),
        },
        Spot::InsertColumn(line) => {
            let t = s.doc.table(story, &geom.table).ok_or_else(|| CmdError::Failed("there's no table there".into()))?;
            let (r, c, after) = beside_line(t, line).ok_or_else(|| CmdError::Failed("there's no column border there".into()))?;
            (r, c, after, false, line)
        }
        _ => return Err(CmdError::Failed("there's a selection area there, not an insert button".into())),
    };
    let size = |s: &Session| s.doc.table(story, &geom.table).map(|t| (t.rows.len(), t.cols()));
    let before = size(s);
    let mut path = geom.table.0.clone();
    path.extend([r as u32, c as u32, 0]);
    s.column = None;
    s.table_selection = None;
    s.sel = Selection::caret(Pos { story, path: Path(path), off: 0 });
    if rows {
        table::row(s, after)?;
    } else {
        table::col(s, after)?;
    }
    if size(s) == before {
        return Err(CmdError::Failed("the table can't take another row or column".into()));
    }
    let (sel_rows, sel_cols) = if rows { ((line, line), (0, usize::MAX)) } else { ((0, usize::MAX), (line, line)) };
    select_grid(s, story, &geom.table, sel_rows, sel_cols, false)
}

fn corner(l: &DocLayout, story: StoryRef, table: &Path) -> Option<(usize, f32, f32)> {
    l.pages.iter().enumerate().find_map(|(pi, pg)| {
        let cells = pg.items.iter().filter_map(|it| match it {
            Placed::Cell { rect, table: t, story: st, .. } if *st == story && t == table => Some(*rect),
            _ => None,
        });
        cells.fold(None, |acc: Option<(f32, f32)>, r| Some(acc.map_or((r.x, r.y), |(x, y)| (x.min(r.x), y.min(r.y))))).map(|(x, y)| (pi, x, y))
    })
}

fn move_drag(s: &mut Session, v: &Value) -> CmdResult {
    let (page, x, y, reach) = point(v)?;
    let (to_x, to_y) = (p::req_f32(v, "toX")?, p::req_f32(v, "toY")?);
    let Target { spot, geom } = target(s, page, x, y, reach)?;
    if spot != Spot::Handle {
        return Err(CmdError::Failed("there's no table move handle there".into()));
    }
    if !movable(&geom) {
        return Err(CmdError::Failed("only a table in the main text can be moved".into()));
    }
    let (gx, gy) = moved_to(&s.layout(), page, &geom, to_x - x, to_y - y);
    if (gx - geom.left()).abs() < 0.01 && (gy - geom.top()).abs() < 0.01 {
        return sel_result(s);
    }
    let t = s.doc.table_mut(geom.story, &geom.table)?;
    let mut f = t.props.float.unwrap_or(TableFloat {
        h_rel: Anchor::Column,
        v_rel: Anchor::Paragraph,
        dist: [9.0, 0.0, 9.0, 0.0],
        overlap: true,
        ..Default::default()
    });
    f.h_align = None;
    f.v_align = None;
    t.props.float = Some(f);
    table_resize::touch(s, &geom)?;
    if let Some((p0, x0, y0)) = corner(&s.layout(), geom.story, &geom.table)
        && p0 == page
    {
        let fix = |v: f32| if v.is_finite() { v.clamp(-MAX_MOVE, MAX_MOVE) } else { 0.0 };
        let t = s.doc.table_mut(geom.story, &geom.table)?;
        if let Some(f) = t.props.float.as_mut() {
            f.x = fix(f.x + gx - x0);
            f.y = fix(f.y + gy - y0);
        }
        table_resize::touch(s, &geom)?;
    }
    sel_result(s)
}

fn fit_column(s: &mut Session, v: &Value) -> CmdResult {
    let (page, x, y, reach) = point(v)?;
    let l = s.layout();
    let Some((target, line)) = table_resize::resize_target(&l, &s.doc, page, x, y, reach).and_then(|t| match t.edge {
        Edge::Column(line) => Some((t, line)),
        _ => None,
    }) else {
        return Err(CmdError::Failed("there's no column border there".into()));
    };
    let t = s.doc.table(target.story, &target.table).ok_or_else(|| CmdError::Failed("there's no table there".into()))?;
    let g = line.checked_sub(1).ok_or_else(|| CmdError::Failed("there's no column left of that border".into()))?;
    let (narrowest, widest) =
        doc4me_layout::measure_table_columns(&s.doc, t).get(g).copied().ok_or_else(|| CmdError::Failed("there's no such column".into()))?;
    let current = target.widths().get(g).copied().unwrap_or(0.0);
    let room = current + (room_right(&l, &s.doc, page, &target) - target.right()).max(0.0);
    let want = widest.min(room.max(narrowest));
    if let Some(widths) = target.plan(want - current, 0.0, true).widths {
        let t = s.doc.table_mut(target.story, &target.table)?;
        if widths.len() == t.cols() {
            t.grid = widths;
            t.props.width = None;
            t.props.width_pct = None;
            table::sync_cell_widths(t);
        }
        table_resize::touch(s, &target)?;
    }
    sel_result(s)
}

fn room_right(l: &DocLayout, doc: &Document, page: usize, target: &ResizeTarget) -> f32 {
    let margin = l.pages.get(page).map_or(target.right(), |pg| pg.body.right());
    let Some((outer, row, cell)) = target.table.cell() else { return margin };
    let Some(t) = doc.table(target.story, &outer) else { return margin };
    let boxes = table_resize::cell_boxes(l, page);
    let Some(b) = boxes.iter().find(|b| b.story == target.story && b.table == outer && b.row == row && b.cell == cell) else { return margin };
    let style = t.props.style.as_deref().and_then(|id| doc.styles.table_style(id));
    let margins = t.props.cell_margins.or(style.and_then(|props| props.parts.cell_margins)).unwrap_or([0.0, 5.4, 0.0, 5.4]);
    let inset = if t.props.bidi_visual { margins[1] } else { margins[3] };
    (b.rect.right() - inset).min(margin)
}
