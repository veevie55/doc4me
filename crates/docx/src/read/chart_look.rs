//! The chart pane's formats (owner, 2026-11-19; `doc4me_chart_format`) as graphic items: a part's
//! shadow and glow, its fill and soft edges, its border, and lines (axes, gridlines) in a
//! border's look. Graphic items have no transparency or blur, so these are drawn over the
//! chart's white the way they would look on it: a transparent colour is mixed with white, a
//! shadow or glow is a few rings fading out, soft edges a few rings fading in.

use doc4me_chart_format::{Border, Compound, Format, Line};
use doc4me_doc::graphic::{GraphicItem, PathSeg};
use doc4me_doc::para::ShapeKind;
use doc4me_doc::props::Rgb;

/// Rings a shadow, glow or soft edge is drawn with.
const RINGS: usize = 5;

/// A format's colour at `opacity` (0–1) over `under`.
pub(crate) fn mix(c: [u8; 3], opacity: f32, under: Rgb) -> Rgb {
    let a = if opacity.is_finite() { opacity.clamp(0.0, 1.0) } else { 1.0 };
    let m = |x: u8, u: u8| (f32::from(x) * a + f32::from(u) * (1.0 - a)).round().clamp(0.0, 255.0) as u8;
    Rgb(m(c[0], under.0), m(c[1], under.1), m(c[2], under.2))
}

fn opacity(t: f32) -> f32 {
    if t.is_finite() { 1.0 - t.clamp(0.0, 100.0) / 100.0 } else { 1.0 }
}

fn pts(v: f32, max: f32) -> f32 {
    if v.is_finite() { v.clamp(0.0, max) } else { 0.0 }
}

/// The part's text colour over white, when the pane gave it a solid one.
pub(crate) fn text_color(f: &Format) -> Option<Rgb> {
    f.text.fill.solid().map(|(c, a)| mix(c, a, Rgb::WHITE))
}

/// The part's font size in points, when the pane set one.
pub(crate) fn text_size(f: &Format) -> Option<f32> {
    f.text.size.filter(|s| s.is_finite()).map(|s| s.clamp(1.0, 400.0))
}

fn shape(out: &mut Vec<GraphicItem>, r: [f32; 4], rounded: bool, fill: Option<Rgb>, stroke: Option<Rgb>, width: f32) {
    if r[2] <= 0.0 || r[3] <= 0.0 {
        return;
    }
    if !rounded {
        out.push(GraphicItem::Shape { rect: r, kind: ShapeKind::Rectangle, fill, stroke, stroke_width: width });
        return;
    }
    // Small corners, as Word rounds a chart area's: 3 % of the shorter side, at most 8 pt.
    let k = (r[2].min(r[3]) * 0.03).min(8.0);
    let (x0, y0, x1, y1) = (r[0], r[1], r[0] + r[2], r[1] + r[3]);
    let c = k * 0.4477;
    let segs = vec![
        PathSeg::Move(x0 + k, y0),
        PathSeg::Line(x1 - k, y0),
        PathSeg::Cubic(x1 - c, y0, x1, y0 + c, x1, y0 + k),
        PathSeg::Line(x1, y1 - k),
        PathSeg::Cubic(x1, y1 - c, x1 - c, y1, x1 - k, y1),
        PathSeg::Line(x0 + k, y1),
        PathSeg::Cubic(x0 + c, y1, x0, y1 - c, x0, y1 - k),
        PathSeg::Line(x0, y0 + k),
        PathSeg::Cubic(x0, y0 + c, x0 + c, y0, x0 + k, y0),
        PathSeg::Close,
    ];
    out.push(GraphicItem::Path { segs, fill, stroke, stroke_width: width });
}

fn grow(r: [f32; 4], d: f32) -> [f32; 4] {
    [r[0] - d, r[1] - d, (r[2] + 2.0 * d).max(0.0), (r[3] + 2.0 * d).max(0.0)]
}

/// The shadow and glow behind box `r` ([x, y, w, h]).
pub(crate) fn effects(out: &mut Vec<GraphicItem>, r: [f32; 4], rounded: bool, f: &Format) {
    if let Some(s) = &f.shadow {
        let a = (if s.angle.is_finite() { s.angle } else { 0.0 }).to_radians();
        let d = pts(s.distance, 200.0);
        let at = [r[0] + a.cos() * d, r[1] + a.sin() * d, r[2], r[3]];
        let blur = pts(s.blur, 100.0);
        // From the faint outside in: each ring a little smaller and stronger.
        for i in 0..RINGS {
            let t = (i + 1) as f32 / RINGS as f32;
            let c = mix(s.color, opacity(s.transparency) * t * 0.6, Rgb::WHITE);
            shape(out, grow(at, blur * (1.0 - t) / 2.0), rounded, Some(c), None, 0.0);
        }
    }
    if let Some(g) = &f.glow {
        let size = pts(g.size, 150.0);
        for i in 0..RINGS {
            let t = (i + 1) as f32 / RINGS as f32;
            let c = mix(g.color, opacity(g.transparency) * t * 0.7, Rgb::WHITE);
            shape(out, grow(r, size * (1.0 - t * 0.8)), true, Some(c), None, 0.0);
        }
    }
}

/// The solid fill of box `r` with its soft edges; `false` when the fill isn't solid.
pub(crate) fn fill(out: &mut Vec<GraphicItem>, r: [f32; 4], rounded: bool, f: &Format) -> bool {
    let Some((c, a)) = f.fill.solid() else { return false };
    let soft = pts(f.soft_edge, r[2].min(r[3]) / 2.0);
    if soft > 0.0 {
        for i in 0..RINGS {
            let t = (i + 1) as f32 / RINGS as f32;
            shape(out, grow(r, -soft * (t - 1.0 / RINGS as f32)), rounded, Some(mix(c, a * t, Rgb::WHITE)), None, 0.0);
        }
    } else {
        shape(out, r, rounded, Some(mix(c, a, Rgb::WHITE)), None, 0.0);
    }
    true
}

/// The strokes of a compound line `w` wide: (offset inwards, width) of each.
fn strokes(compound: Compound, w: f32) -> Vec<(f32, f32)> {
    match compound {
        Compound::Single => vec![(0.0, w)],
        Compound::Double => vec![(-w / 3.0, w / 3.0), (w / 3.0, w / 3.0)],
        Compound::ThickThin => vec![(-w * 0.25, w * 0.5), (w * 0.4, w * 0.2)],
        Compound::ThinThick => vec![(-w * 0.4, w * 0.2), (w * 0.25, w * 0.5)],
        Compound::Triple => vec![(-w * 0.4, w * 0.2), (0.0, w * 0.2), (w * 0.4, w * 0.2)],
    }
}

/// A straight run from `a` to `b` cut into the dash of `l` (`sw` wide) as path pieces.
fn dashed(a: (f32, f32), b: (f32, f32), l: &Line, sw: f32) -> Vec<PathSeg> {
    let pattern: Vec<f32> = l.dash.pattern().iter().map(|p| p * sw.max(0.75)).collect();
    if pattern.is_empty() {
        return vec![PathSeg::Move(a.0, a.1), PathSeg::Line(b.0, b.1)];
    }
    let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
    if len <= 0.0 {
        return Vec::new();
    }
    let (ux, uy) = ((b.0 - a.0) / len, (b.1 - a.1) / len);
    let mut segs = Vec::new();
    let mut at = 0.0;
    let mut k = 0usize;
    // At most 2,000 pieces: a long hairline dash stays affordable.
    while at < len && segs.len() < 4_000 {
        let piece = pattern.get(k % pattern.len()).copied().unwrap_or(1.0).max(0.25);
        let end = (at + piece).min(len);
        if k.is_multiple_of(2) {
            segs.push(PathSeg::Move(a.0 + ux * at, a.1 + uy * at));
            segs.push(PathSeg::Line(a.0 + ux * end, a.1 + uy * end));
        }
        at = end;
        k += 1;
    }
    segs
}

/// The border of box `r`, when the format's border is solid.
pub(crate) fn border(out: &mut Vec<GraphicItem>, r: [f32; 4], rounded: bool, f: &Format) {
    let Border::Solid(l) = &f.border else { return };
    let w = pts(l.width, 20.0);
    if w <= 0.0 {
        return;
    }
    let color = mix(l.color, opacity(l.transparency), Rgb::WHITE);
    for (off, sw) in strokes(l.compound, w) {
        let b = grow(r, -off);
        if l.dash.pattern().is_empty() {
            shape(out, b, rounded, None, Some(color), sw);
        } else {
            let (x0, y0, x1, y1) = (b[0], b[1], b[0] + b[2], b[1] + b[3]);
            let mut segs = Vec::new();
            for (p, q) in [((x0, y0), (x1, y0)), ((x1, y0), (x1, y1)), ((x1, y1), (x0, y1)), ((x0, y1), (x0, y0))] {
                segs.extend(dashed(p, q, l, sw));
            }
            out.push(GraphicItem::Path { segs, fill: None, stroke: Some(color), stroke_width: sw });
        }
    }
}

/// A whole box: effects, fill (when solid), border.
pub(crate) fn part(out: &mut Vec<GraphicItem>, r: [f32; 4], rounded: bool, f: &Format) {
    effects(out, r, rounded, f);
    fill(out, r, rounded, f);
    border(out, r, rounded, f);
}

/// A line from `a` to `b` in a solid border's look; `false` (nothing drawn) otherwise.
pub(crate) fn line(out: &mut Vec<GraphicItem>, a: (f32, f32), b: (f32, f32), f: &Format) -> bool {
    let Border::Solid(l) = &f.border else { return false };
    let w = pts(l.width, 20.0);
    let color = mix(l.color, opacity(l.transparency), Rgb::WHITE);
    let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt().max(1e-3);
    let (nx, ny) = (-(b.1 - a.1) / len, (b.0 - a.0) / len);
    for (off, sw) in strokes(l.compound, w) {
        let segs = dashed((a.0 + nx * off, a.1 + ny * off), (b.0 + nx * off, b.1 + ny * off), l, sw);
        out.push(GraphicItem::Path { segs, fill: None, stroke: Some(color), stroke_width: sw });
    }
    true
}

/// The box around the items `out[from..]` (text and swatches), grown by `pad`.
pub(crate) fn bounds(out: &[GraphicItem], from: usize, pad: f32) -> Option<[f32; 4]> {
    let mut b: Option<[f32; 4]> = None;
    for it in out.get(from..).unwrap_or_default() {
        let r = match it {
            GraphicItem::Shape { rect, .. } | GraphicItem::Text { rect, .. } => *rect,
            _ => continue,
        };
        b = Some(match b {
            None => r,
            Some(o) => {
                let (x0, y0) = (o[0].min(r[0]), o[1].min(r[1]));
                let (x1, y1) = ((o[0] + o[2]).max(r[0] + r[2]), (o[1] + o[3]).max(r[1] + r[3]));
                [x0, y0, x1 - x0, y1 - y0]
            }
        });
    }
    b.map(|r| grow(r, pad))
}

/// Recolours the text items in `out[from..]` drawn in `was` to `to`.
pub(crate) fn recolor(out: &mut [GraphicItem], from: usize, was: Rgb, to: Rgb) {
    for it in out.get_mut(from..).unwrap_or_default() {
        if let GraphicItem::Text { color, .. } = it
            && *color == was
        {
            *color = to;
        }
    }
}
