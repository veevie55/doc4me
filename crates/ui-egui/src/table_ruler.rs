//! Table borders on the rulers: the horizontal ruler marks the table's left edge and the caret
//! row's column borders; the vertical ruler marks row bottoms. Dragging uses the same resize
//! command and preview as dragging directly on the page.

use doc4me_geom::Rect;
use doc4me_layout::{DocLayout, Placed};
use egui::{Sense, Stroke, Ui, pos2, vec2};

use crate::WordApp;
use crate::theme::Tokens;

fn cells(app: &WordApp, layout: &DocLayout, page: usize) -> Vec<(usize, usize, Rect)> {
    let story = app.session.sel.focus.story;
    let (Some((tp, _, _)), Some(pg)) = (app.session.sel.focus.path.cell(), layout.pages.get(page)) else { return Vec::new() };
    pg.items
        .iter()
        .filter_map(|it| match it {
            Placed::Cell { rect, table, row, cell, story: st, .. } if *table == tp && *st == story => Some((*row, *cell, *rect)),
            _ => None,
        })
        .collect()
}

/// The caret cell's text area on `page`, as (left edge, width) in page points.
pub(crate) fn text_area(app: &WordApp, layout: &DocLayout, page: usize) -> Option<(f32, f32)> {
    let story = app.session.sel.focus.story;
    let (tp, r, c) = app.session.sel.focus.path.cell()?;
    let (_, _, b) = cells(app, layout, page).into_iter().find(|(row, cell, _)| *row == r && *cell == c)?;
    let table = app.session.doc.table(story, &tp)?;
    let cell = table.rows.get(r)?.cells.get(c)?;
    let style_margins = table.props.style.as_deref().and_then(|id| app.session.doc.styles.table_style(id)).and_then(|style| style.parts.cell_margins);
    let m = cell.props.margins.or(table.props.cell_margins).or(style_margins).unwrap_or([0.0, 5.4, 0.0, 5.4]);
    let (left, right) = (m[1].max(0.0), m[3].max(0.0));
    Some((b.x + left, (b.w - left - right).max(0.0)))
}

fn marker(painter: &egui::Painter, r: egui::Rect, t: &Tokens) {
    painter.rect_filled(r, 1.0, t.ruler_margin);
    painter.rect_stroke(r, 1.0, Stroke::new(1.0, t.text_dim), egui::StrokeKind::Inside);
    let c = r.center();
    let (along, across) = if r.height() > r.width() { (vec2(0.0, 3.0), vec2(1.5, 0.0)) } else { (vec2(3.0, 0.0), vec2(0.0, 1.5)) };
    for k in [-1.0, 0.0, 1.0] {
        let m = c + along * k;
        painter.line_segment([m - across, m + across], Stroke::new(1.0, t.text_dim));
    }
}

fn grip(app: &mut WordApp, ui: &Ui, layout: &DocLayout, zone: egui::Rect, column: bool, i: usize, at: (usize, f32, f32)) {
    let (name, label, cursor) = if column {
        ("ruler_column", tl!("Table Column Width"), egui::CursorIcon::ResizeColumn)
    } else {
        ("ruler_row", tl!("Table Row Height"), egui::CursorIcon::ResizeRow)
    };
    let r = ui.interact(zone, ui.id().with((name, i)), Sense::drag()).on_hover_cursor(cursor);
    r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Other, true, label));
    let r = if r.dragged() { r } else { r.on_hover_text(label) };
    if r.drag_started() {
        crate::table_resize::grab(app, layout, at.0, at.1, at.2);
    }
}

fn caret_row(app: &WordApp, layout: &DocLayout, page: usize) -> Vec<Rect> {
    let Some((_, r, _)) = app.session.sel.focus.path.cell() else { return Vec::new() };
    let mut row: Vec<Rect> = cells(app, layout, page).into_iter().filter(|(row, ..)| *row == r).map(|(.., b)| b).collect();
    row.sort_by(|a, b| a.x.total_cmp(&b.x));
    row
}

pub(crate) fn widths(app: &WordApp, ctx: &egui::Context, layout: &DocLayout, page: usize) -> Vec<(f32, f32)> {
    if !crate::table_resize::measuring(app, ctx, page) {
        return Vec::new();
    }
    let at = |x: f32| crate::table_resize::moved(app, ctx, page, true, x);
    caret_row(app, layout, page)
        .iter()
        .map(|b| {
            let (l, r) = (at(b.x), at(b.right()));
            ((l + r) / 2.0, r - l)
        })
        .collect()
}

/// Column markers on the horizontal ruler and the widths shown while Alt is held during a drag.
pub(crate) fn columns(app: &mut WordApp, ui: &mut Ui, layout: &DocLayout, page: usize, bar: egui::Rect, x0: f32, scale: f32) {
    let row = caret_row(app, layout, page);
    let Some(first) = row.first().copied() else { return };
    let t = Tokens::get(ui.ctx());
    let painter = ui.painter_at(bar.expand(1.0));
    let edges: Vec<(f32, f32)> = std::iter::once(first.x)
        .chain(row.iter().map(|b| b.right()))
        .map(|x| (x, x0 + crate::table_resize::moved(app, ui.ctx(), page, true, x) * scale))
        .collect();
    for (i, (x, sx)) in edges.into_iter().enumerate() {
        marker(&painter, egui::Rect::from_center_size(pos2(sx, bar.center().y), vec2(7.0, bar.height() - 2.0)), &t);
        let zone = egui::Rect::from_center_size(pos2(sx, bar.center().y), vec2(8.0, bar.height()));
        grip(app, ui, layout, zone, true, i, (page, x, first.y + first.h / 2.0));
    }
    for (mid, width) in widths(app, ui.ctx(), layout, page) {
        let at = pos2(x0 + mid * scale, bar.center().y);
        let text = painter.layout_no_wrap(crate::options::length(width), crate::theme::regular(9.5), t.text);
        painter.rect_filled(egui::Rect::from_center_size(at, text.size() + vec2(4.0, 0.0)), 1.0, t.ruler);
        painter.galley(at - text.size() / 2.0, text, t.text);
    }
}

/// Row-bottom markers on the vertical ruler.
pub(crate) fn rows(app: &mut WordApp, ui: &mut Ui, layout: &DocLayout, page: usize, bar: egui::Rect, y0: f32, scale: f32) {
    let mut rows: Vec<(usize, Rect)> = Vec::new();
    for (r, _, b) in cells(app, layout, page) {
        match rows.iter_mut().find(|(row, _)| *row == r) {
            Some((_, first)) if b.bottom() < first.bottom() - 0.01 || (b.bottom() <= first.bottom() + 0.01 && b.x < first.x) => *first = b,
            Some(_) => {}
            None => rows.push((r, b)),
        }
    }
    let t = Tokens::get(ui.ctx());
    let painter = ui.painter_at(bar.expand(1.0));
    for (r, b) in rows {
        let sy = y0 + crate::table_resize::moved(app, ui.ctx(), page, false, b.bottom()) * scale;
        marker(&painter, egui::Rect::from_center_size(pos2(bar.center().x, sy), vec2(bar.width() - 2.0, 7.0)), &t);
        let zone = egui::Rect::from_center_size(pos2(bar.center().x, sy), vec2(bar.width(), 8.0));
        grip(app, ui, layout, zone, false, r, (page, b.x + b.w / 2.0, b.bottom()));
    }
}
