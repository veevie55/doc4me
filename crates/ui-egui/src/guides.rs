//! Alignment guides: while a picture, shape or text box is dragged, its left, centre or right
//! edge (and top, middle or bottom) snaps to the page's edges and centre, the margins and the
//! edges and centres of the other floating objects on its page, and a thin line across the page
//! shows each place it snapped to. WordCraft's own design (Word for Mac shows no guides).
//!
//! The maths here is pure (points in, points out) so it's tested without egui; `objects.rs`
//! gathers the candidates and draws the lines.

use doc4me_geom::Rect;

/// How near (screen points) an edge or centre must come to a guide to snap to it.
pub const SNAP_SCREEN: f32 = 6.0;

/// Which way a guide runs: `X` is a vertical line at `at` (lefts, centres and rights snap to it),
/// `Y` a horizontal one (tops, middles and bottoms).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Axis {
    X,
    Y,
}

/// A place to snap to on the page, points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Guide {
    pub axis: Axis,
    pub at: f32,
}

impl Guide {
    pub fn x(at: f32) -> Guide {
        Guide { axis: Axis::X, at }
    }
    pub fn y(at: f32) -> Guide {
        Guide { axis: Axis::Y, at }
    }
}

/// What snapping does to a dragged rect: move it by (`dx`, `dy`) points, and show `lines`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snap {
    pub dx: f32,
    pub dy: f32,
    pub lines: Vec<Guide>,
}

/// Snap `moving` (the dragged object's bounds, points) to the nearest of `candidates` within
/// `threshold` points, x and y independently: its left, centre or right to an `X` guide, its top,
/// middle or bottom to a `Y` guide. Nearest wins; on a tie, the first candidate. The lines are
/// every guide an edge or centre sits on once moved.
pub fn snap(moving: Rect, candidates: &[Guide], threshold: f32) -> Snap {
    if !threshold.is_finite() || threshold < 0.0 {
        return Snap::default();
    }
    let xs = [moving.x, moving.x + moving.w / 2.0, moving.x + moving.w];
    let ys = [moving.y, moving.y + moving.h / 2.0, moving.y + moving.h];
    let mut out = Snap::default();
    for (axis, points) in [(Axis::X, xs), (Axis::Y, ys)] {
        let Some(d) = nearest(&points, candidates, axis, threshold) else { continue };
        match axis {
            Axis::X => out.dx = d,
            Axis::Y => out.dy = d,
        }
        for g in candidates.iter().filter(|g| g.axis == axis && g.at.is_finite()) {
            if points.iter().any(|p| (p + d - g.at).abs() <= LANDED) && !out.lines.contains(g) {
                out.lines.push(*g);
            }
        }
    }
    out
}

/// Snap the edges a resize drags (`hx`, `hy`: -1 left/top, 1 right/bottom, 0 that axis isn't
/// dragged) of `r` to the nearest of `candidates` within `threshold` points, so an object
/// stretched to the margin lands exactly on it. With `ratio` (a picture's corner), only the edge
/// nearer its guide snaps and the other follows to keep the proportions, the opposite corner
/// staying put. A snap that would make it smaller than `min` either way is left out. Returns the
/// snapped rect and the guides its dragged edges sit on.
pub fn snap_resize(r: Rect, (hx, hy): (i8, i8), candidates: &[Guide], threshold: f32, ratio: bool, min: f32) -> (Rect, Vec<Guide>) {
    if !threshold.is_finite() || threshold < 0.0 || !(r.w > 0.0 && r.h > 0.0) {
        return (r, Vec::new());
    }
    let edge = |h: i8, lo: f32, size: f32| match h {
        -1 => Some(lo),
        1 => Some(lo + size),
        _ => None,
    };
    let (ex, ey) = (edge(hx, r.x, r.w), edge(hy, r.y, r.h));
    let mut dx = ex.and_then(|e| nearest(&[e], candidates, Axis::X, threshold));
    let mut dy = ey.and_then(|e| nearest(&[e], candidates, Axis::Y, threshold));
    if ratio && ex.is_some() && ey.is_some() {
        match (dx, dy) {
            (Some(x), Some(y)) if y.abs() < x.abs() => dx = None,
            (Some(_), Some(_)) => dy = None,
            _ => {}
        }
    }
    let (mut x0, mut y0, mut x1, mut y1) = (r.x, r.y, r.x + r.w, r.y + r.h);
    let apply = |h: i8, d: f32, lo: &mut f32, hi: &mut f32| if h < 0 { *lo += d } else { *hi += d };
    if let Some(d) = dx {
        apply(hx, d, &mut x0, &mut x1);
    }
    if let Some(d) = dy {
        apply(hy, d, &mut y0, &mut y1);
    }
    if ratio && ex.is_some() && ey.is_some() {
        let aspect = r.w / r.h;
        if dx.is_some() {
            let h = (x1 - x0) / aspect;
            if hy < 0 { y0 = y1 - h } else { y1 = y0 + h }
        } else if dy.is_some() {
            let w = (y1 - y0) * aspect;
            if hx < 0 { x0 = x1 - w } else { x1 = x0 + w }
        }
    }
    let out = Rect::new(x0, y0, x1 - x0, y1 - y0);
    if dx.is_none() && dy.is_none() || !(out.w >= min && out.h >= min) || !out.w.is_finite() || !out.h.is_finite() {
        return (r, Vec::new());
    }
    let mut lines = Vec::new();
    for (axis, at) in [(Axis::X, edge(hx, out.x, out.w)), (Axis::Y, edge(hy, out.y, out.h))] {
        let Some(at) = at else { continue };
        for g in candidates.iter().filter(|g| g.axis == axis && (g.at - at).abs() <= LANDED) {
            if !lines.contains(g) {
                lines.push(*g);
            }
        }
    }
    (out, lines)
}

/// How near (points) an edge must be to a guide after snapping to count as on it.
const LANDED: f32 = 0.01;

/// The shortest move (within `threshold`) that puts one of `points` on an `axis` guide.
fn nearest(points: &[f32], candidates: &[Guide], axis: Axis, threshold: f32) -> Option<f32> {
    let mut best: Option<f32> = None;
    for g in candidates.iter().filter(|g| g.axis == axis && g.at.is_finite()) {
        for p in points.iter().filter(|p| p.is_finite()) {
            let d = g.at - p;
            if d.abs() <= threshold && best.is_none_or(|b| d.abs() < b.abs()) {
                best = Some(d);
            }
        }
    }
    best
}

/// The guides of a page `w` × `h` points with text area `body`, and of the other floating objects
/// on it (their bounds): edges and centres.
pub fn candidates(w: f32, h: f32, body: Rect, others: impl IntoIterator<Item = Rect>) -> Vec<Guide> {
    let mut g = Vec::new();
    let mut add = |r: Rect| {
        for x in [r.x, r.x + r.w / 2.0, r.x + r.w] {
            g.push(Guide::x(x));
        }
        for y in [r.y, r.y + r.h / 2.0, r.y + r.h] {
            g.push(Guide::y(y));
        }
    };
    add(Rect::new(0.0, 0.0, w, h));
    add(body);
    for r in others {
        add(r);
    }
    g.retain(|g| g.at.is_finite());
    g
}
