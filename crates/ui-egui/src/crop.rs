//! Crop mode on the canvas (Picture Format › Size › Crop, `ui.crop`): black crop handles replace
//! the selected picture's resize handles; dragging one trims that side (a corner, two sides)
//! while what stays visible keeps its scale and its place on the page, and dragging back out
//! restores what was cropped away (never past the source's edge). The cropped-away part shows
//! around the picture under a dark dim. Esc, Enter, a click outside the picture or Crop again
//! leave crop mode and keep the crop. A drop is one `picture.crop` with the crop and the new size,
//! so one undo step. Only unturned pictures: a turned one stays out of crop mode (status message).
//!
//! While dragging nothing is laid out: the preview is the picture's own source pixels, the part
//! that would stay visible bright and the rest dimmed.

use doc4me_doc::Pos;
use doc4me_doc::para::InlineObject;
use doc4me_layout::DocLayout;
use egui::{Color32, Painter, Pos2, Rect, Ui, pos2, vec2};
use serde_json::{Value, json};

use crate::WordApp;
use crate::frame::{self, Handle};
use crate::theme::Tokens;

/// The most of the source one side can crop away (as `picture.crop` clamps).
const MAX_SIDE: f32 = 0.45;
/// Pointer travel that turns a press on a handle into a drag (screen points).
const DRAG_START: f32 = 3.0;
/// Crop handles: arm length of a corner's L, length of an edge bar, thickness (screen points).
const ARM: f32 = 11.0;
const BAR: f32 = 16.0;
const THICK: f32 = 3.5;
/// The longest side of the source picture decoded for crop mode (pixels).
const SOURCE_MAX_PX: u32 = 1024;

/// Where a crop handle drag leaves a picture: its crop (left, top, right, bottom fractions of the
/// source), its new display size (points), and how far its frame's top-left corner moves (points).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cropped {
    pub crop: [f32; 4],
    pub w: f32,
    pub h: f32,
    pub dx: f32,
    pub dy: f32,
}

/// The display sizes `picture.crop` accepts (points).
const MIN_SIZE: f32 = 1.0;
const MAX_SIZE: f32 = 4000.0;

/// One axis of [`cropped`]: a `len` long side cropped `lo` / `hi`, its handle end `grab`
/// (-1 low, 1 high, 0 none) dragged by `d` points. Returns the new crops, length and how far the
/// low edge moved. The length stays within [`MIN_SIZE`]–[`MAX_SIZE`] at the same scale (the crop
/// stops there), unless it was already outside it.
fn axis(len: f32, lo: f32, hi: f32, grab: i8, d: f32) -> (f32, f32, f32, f32) {
    // The source's full length at this scale; the visible part is 1-lo-hi of it.
    let full = len / (1.0 - lo - hi).max(0.1);
    // How far the grabbed side's crop may go: at most MAX_SIDE, and the length within its range.
    let range = |cur: f32, other: f32| {
        let most = (1.0 - other - MIN_SIZE / full).min(MAX_SIDE).max(cur);
        let least = (1.0 - other - MAX_SIZE / full).max(0.0).min(cur);
        (least, most)
    };
    let (mut lo2, mut hi2) = (lo, hi);
    match grab {
        -1 => {
            let (least, most) = range(lo, hi);
            lo2 = (lo + d / full).max(least).min(most);
        }
        1 => {
            let (least, most) = range(hi, lo);
            hi2 = (hi - d / full).max(least).min(most);
        }
        _ => {}
    }
    (lo2, hi2, full * (1.0 - lo2 - hi2), (lo2 - lo) * full)
}

/// Drag crop handle `handle` of a `w`×`h` point picture cropped `crop` by (`dx`, `dy`) points: the
/// grabbed sides are trimmed (inward) or restored (outward) at the picture's current scale.
/// Hostile numbers (non-finite, non-positive sizes) leave it as it was; sizes are taken within
/// 1–4000 pt, as `picture.crop` takes them.
pub fn cropped(w: f32, h: f32, crop: [f32; 4], handle: Handle, dx: f32, dy: f32) -> Cropped {
    cropped_on_screen(w, h, crop, false, false, handle, dx, dy)
}

/// [`cropped`] for a picture flipped left to right (`flip_h`) and/or top to bottom (`flip_v`):
/// `handle` and the drag are on screen, the crop is the source's. Flipped, the side under a
/// handle is the source's opposite side, so that's the one that moves.
#[allow(clippy::too_many_arguments)]
pub fn cropped_on_screen(w: f32, h: f32, crop: [f32; 4], flip_h: bool, flip_v: bool, handle: Handle, dx: f32, dy: f32) -> Cropped {
    let fin = |x: f32| if x.is_finite() { x } else { 0.0 };
    let crop = crop.map(|c| fin(c).clamp(0.0, MAX_SIDE));
    let (w, h) = (fin(w), fin(h));
    let (dx, dy) = (fin(dx), fin(dy));
    if w <= 0.0 || h <= 0.0 || (dx == 0.0 && dy == 0.0) {
        return Cropped { crop, w, h, dx: 0.0, dy: 0.0 };
    }
    let (w, h) = (w.min(MAX_SIZE), h.min(MAX_SIZE));
    // On screen order: (left, right) and (top, bottom) as they show.
    let (sl, sr) = if flip_h { (crop[2], crop[0]) } else { (crop[0], crop[2]) };
    let (st, sb) = if flip_v { (crop[3], crop[1]) } else { (crop[1], crop[3]) };
    let (l, r, w2, mx) = axis(w, sl, sr, handle.x, dx);
    let (t, b, h2, my) = axis(h, st, sb, handle.y, dy);
    let (l, r) = if flip_h { (r, l) } else { (l, r) };
    let (t, b) = if flip_v { (b, t) } else { (t, b) };
    Cropped { crop: [l, t, r, b], w: w2, h: h2, dx: mx, dy: my }
}

/// The crop as it shows on screen: (left, top, right, bottom) of the displayed picture.
fn on_screen(crop: [f32; 4], flip_h: bool, flip_v: bool) -> [f32; 4] {
    let [l, t, r, b] = crop;
    let (l, r) = if flip_h { (r, l) } else { (l, r) };
    let (t, b) = if flip_v { (b, t) } else { (t, b) };
    [l, t, r, b]
}

/// A crop handle being dragged.
pub struct CropDrag {
    /// The picture being cropped: the drop applies only while it's still the one in crop mode.
    pos: Pos,
    handle: Handle,
    flip_h: bool,
    flip_v: bool,
    /// The picture's frame where the drag started (page points) and its page.
    page: usize,
    start: doc4me_geom::Rect,
    /// Its display size and crop then.
    w: f32,
    h: f32,
    crop: [f32; 4],
    press: Pos2,
    moved: bool,
    /// Where the drag puts it now.
    now: Cropped,
}

impl CropDrag {
    fn follow(&mut self, at: Pos2, scale: f32) {
        if !self.moved && at.distance(self.press) < DRAG_START {
            return;
        }
        self.moved = true;
        let d = (at - self.press) / scale.max(0.01);
        self.now = cropped_on_screen(self.w, self.h, self.crop, self.flip_h, self.flip_v, self.handle, d.x, d.y);
    }
}

/// The selected picture, if it's unturned (it may be flipped).
struct Picture {
    pos: Pos,
    w: f32,
    h: f32,
    crop: [f32; 4],
    media: String,
    flip_h: bool,
    flip_v: bool,
}

fn picture(app: &WordApp) -> Option<Picture> {
    match crate::objects::selected(app)? {
        (pos, InlineObject::Image { w, h, crop, media, float, .. }) if doc4me_geom::normalize_degrees(float.rot) == 0.0 => {
            Some(Picture { pos, w: *w, h: *h, crop: *crop, media: media.clone(), flip_h: float.flip_h, flip_v: float.flip_v })
        }
        _ => None,
    }
}

/// The picture in crop mode: crop mode is on and that picture is still the selection.
pub fn active(app: &WordApp) -> Option<Pos> {
    let on = app.canvas.crop.as_ref()?;
    picture(app).map(|p| p.pos).filter(|p| p == on)
}

/// Whether Crop can be clicked: a picture is selected (or crop mode is on, to leave it).
pub fn can_toggle(app: &WordApp) -> bool {
    active(app).is_some() || matches!(crate::objects::selected(app), Some((_, InlineObject::Image { .. })))
}

/// `ui.crop`: crop mode on for the selected picture, or off (keeping the crop).
pub fn toggle(app: &mut WordApp) -> Result<Value, String> {
    if active(app).is_some() {
        leave(app);
        return Ok(json!({"crop": false}));
    }
    let Some((_, InlineObject::Image { .. })) = crate::objects::selected(app) else {
        return Err(tl!("select a picture first").to_string());
    };
    let Some(Picture { pos, .. }) = picture(app) else {
        let msg = tl!("Crop on the page works on unturned pictures: set Rotation to 0° first, or use the crop boxes").to_string();
        app.status(msg.clone());
        return Err(msg);
    };
    app.canvas.crop = Some(pos);
    app.canvas.crop_drag = None;
    Ok(json!({"crop": true}))
}

fn leave(app: &mut WordApp) {
    app.canvas.crop = None;
    app.canvas.crop_drag = None;
}

/// Esc and Enter leave crop mode, keeping the crop. Returns true if the key was taken.
pub fn key(app: &mut WordApp, key: egui::Key) -> bool {
    if active(app).is_none() {
        leave(app);
        return false;
    }
    if matches!(key, egui::Key::Escape | egui::Key::Enter) {
        leave(app);
        return true;
    }
    false
}

fn screen(pages: &[Rect], scale: f32, page: usize, r: doc4me_geom::Rect) -> Option<Rect> {
    pages.get(page).map(|p| frame::to_screen(*p, scale, r))
}

/// The picture in crop mode, laid out: its frame (page points) and page.
fn laid_out(app: &WordApp, layout: &DocLayout) -> Option<(usize, doc4me_geom::Rect)> {
    let pos = active(app)?;
    layout.object(&pos, app.session.page_hint).map(|o| (o.page, o.rect))
}

/// Presses and drags in crop mode. `Some(true)`: taken; `Some(false)` / `None`: not about crop
/// mode, so the usual object handling goes on (a press outside the picture has just left it).
pub fn pointer(app: &mut WordApp, ui: &Ui, resp: &egui::Response, pages: &[Rect], layout: &DocLayout, scale: f32, at: Pos2) -> Option<bool> {
    if let Some(mut d) = app.canvas.crop_drag.take() {
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            return Some(true); // the drag is dropped without a change; crop mode stays on
        }
        if active(app).as_ref() != Some(&d.pos) {
            // The picture changed under the drag (undo, another selection): drop it unapplied.
            return Some(true);
        }
        if ui.input(|i| i.pointer.primary_down()) {
            d.follow(at, scale);
            app.canvas.crop_drag = Some(d);
        } else if d.moved {
            let c = d.now;
            let _ = app.run(
                "picture.crop",
                json!({"left": c.crop[0], "top": c.crop[1], "right": c.crop[2], "bottom": c.crop[3], "width": c.w, "height": c.h}),
            );
        }
        return Some(true);
    }
    if active(app).is_none() {
        leave(app);
        return None;
    }
    if !(ui.input(|i| i.pointer.primary_pressed()) && resp.contains_pointer()) {
        return None;
    }
    let (page, rect) = laid_out(app, layout)?;
    let Picture { pos, w, h, crop, flip_h, flip_v, .. } = picture(app)?;
    let f = screen(pages, scale, page, rect)?;
    if let Some(handle) = frame::handle_at(f, 0.0, at) {
        let now = Cropped { crop, w, h, dx: 0.0, dy: 0.0 };
        app.canvas.crop_drag = Some(CropDrag { pos, handle, flip_h, flip_v, page, start: rect, w, h, crop, press: at, moved: false, now });
        return Some(true);
    }
    if f.contains(at) {
        return Some(true); // inside the picture: crop mode stays on
    }
    // Outside: crop mode ends with the crop kept, and the press goes on as usual.
    leave(app);
    None
}

/// The cursor over a crop handle (or while dragging one).
pub fn cursor(app: &WordApp, layout: &DocLayout, pages: &[Rect], scale: f32, at: Pos2) -> Option<egui::CursorIcon> {
    if let Some(d) = &app.canvas.crop_drag {
        return Some(d.handle.cursor());
    }
    let (page, rect) = laid_out(app, layout)?;
    let f = screen(pages, scale, page, rect)?;
    match frame::handle_at(f, 0.0, at) {
        Some(h) => Some(h.cursor()),
        None if f.contains(at) => Some(egui::CursorIcon::Default),
        None => None,
    }
}

/// The source picture as a texture (downscaled), kept in egui's memory per media item; None when
/// it can't be decoded (then the cropped-away area is just dimmed).
fn source(app: &WordApp, ctx: &egui::Context, media: &str) -> Option<egui::TextureHandle> {
    let bytes = app.session.doc.media.get(media)?;
    let id = egui::Id::new(("crop_source", media, bytes.len()));
    if let Some(t) = ctx.data(|d| d.get_temp::<Option<egui::TextureHandle>>(id)) {
        return t;
    }
    let tex = image::load_from_memory(bytes).ok().map(|img| {
        let img = if img.width().max(img.height()) > SOURCE_MAX_PX { img.thumbnail(SOURCE_MAX_PX, SOURCE_MAX_PX) } else { img };
        let rgba = img.to_rgba8();
        let ci = egui::ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], rgba.as_raw());
        ctx.load_texture(format!("crop_source_{media}"), ci, egui::TextureOptions::LINEAR)
    });
    ctx.data_mut(|d| d.insert_temp(id, tex.clone()));
    tex
}

/// Draw crop mode: the cropped-away part dimmed around the picture, its frame and the crop
/// handles; while dragging, the picture as the drop would leave it. Returns whether crop mode
/// drew (then the usual frame isn't drawn).
pub fn paint(app: &WordApp, painter: &Painter, t: &Tokens, layout: &DocLayout, pages: &[Rect], scale: f32) -> bool {
    let Some((page, rect)) = laid_out(app, layout) else { return false };
    let Some(Picture { crop, media, flip_h, flip_v, .. }) = picture(app) else { return false };
    let drag = app.canvas.crop_drag.as_ref().filter(|d| d.moved);
    let (page, frame_pt, c) = match drag {
        Some(d) => {
            let n = d.now;
            (d.page, doc4me_geom::Rect::new(d.start.x + n.dx, d.start.y + n.dy, n.w, n.h), n.crop)
        }
        None => (page, rect, crop),
    };
    // The crop as it shows: a flipped picture shows its source's right on the left.
    let c = on_screen(c, flip_h, flip_v);
    // The whole source, at the picture's scale, around the visible part.
    let fw = frame_pt.w / (1.0 - c[0] - c[2]).max(0.1);
    let fh = frame_pt.h / (1.0 - c[1] - c[3]).max(0.1);
    let full_pt = doc4me_geom::Rect::new(frame_pt.x - c[0] * fw, frame_pt.y - c[1] * fh, fw, fh);
    let (Some(f), Some(full)) = (screen(pages, scale, page, frame_pt), screen(pages, scale, page, full_pt)) else { return false };
    let tex = source(app, painter.ctx(), &media);
    // Where a screen rect is in the source (0–1), mirrored as the picture is flipped.
    let uv = |r: Rect| {
        let (w, h) = (full.width().max(0.01), full.height().max(0.01));
        let u = |x: f32| if flip_h { 1.0 - (x - full.min.x) / w } else { (x - full.min.x) / w };
        let v = |y: f32| if flip_v { 1.0 - (y - full.min.y) / h } else { (y - full.min.y) / h };
        Rect::from_min_max(pos2(u(r.min.x), v(r.min.y)), pos2(u(r.max.x), v(r.max.y)))
    };
    if let (Some(tex), Some(_)) = (&tex, drag) {
        // Dragging: the page still shows the old crop, so show the new visible part too.
        painter.rect_filled(f, 0.0, Color32::WHITE);
        painter.image(tex.id(), f, uv(f), Color32::WHITE);
    }
    // The cropped-away bands: top, bottom, left, right.
    let bands = [
        Rect::from_min_max(full.min, pos2(full.max.x, f.min.y)),
        Rect::from_min_max(pos2(full.min.x, f.max.y), full.max),
        Rect::from_min_max(pos2(full.min.x, f.min.y), pos2(f.min.x, f.max.y)),
        Rect::from_min_max(pos2(f.max.x, f.min.y), pos2(full.max.x, f.max.y)),
    ];
    for b in bands.into_iter().filter(|b| b.width() > 0.0 && b.height() > 0.0) {
        if let Some(tex) = &tex {
            painter.image(tex.id(), b, uv(b), Color32::WHITE);
        }
        painter.rect_filled(b, 0.0, t.crop_dim);
    }
    painter.rect_stroke(full, 0.0, egui::Stroke::new(1.0, t.crop_dim), egui::StrokeKind::Outside);
    painter.rect_stroke(f, 0.0, egui::Stroke::new(1.0, t.crop_handle), egui::StrokeKind::Inside);
    paint_handles(painter, t, f);
    true
}

/// The crop handles of frame `f`: an L at each corner, a bar mid-edge, drawn just inside the
/// frame in the crop handle colour with a light halo.
fn paint_handles(painter: &Painter, t: &Tokens, f: Rect) {
    for h in frame::handles(f) {
        let (sx, sy) = (f32::from(h.x), f32::from(h.y));
        let c = pos2(f.center().x + sx * f.width() / 2.0, f.center().y + sy * f.height() / 2.0);
        let bars: Vec<Rect> = match (h.x, h.y) {
            (0, _) => vec![Rect::from_two_pos(c - vec2(BAR / 2.0, 0.0), c + vec2(BAR / 2.0, -sy * THICK))],
            (_, 0) => vec![Rect::from_two_pos(c - vec2(0.0, BAR / 2.0), c + vec2(-sx * THICK, BAR / 2.0))],
            _ => vec![Rect::from_two_pos(c, c + vec2(-sx * ARM, -sy * THICK)), Rect::from_two_pos(c, c + vec2(-sx * THICK, -sy * ARM))],
        };
        for b in &bars {
            painter.rect_filled(b.expand(1.0), 0.0, t.handle);
        }
        for b in bars {
            painter.rect_filled(b, 0.0, t.crop_handle);
        }
    }
}
