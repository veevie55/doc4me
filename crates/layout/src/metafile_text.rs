//! The text of a metafile picture (EMF `ExtTextOutW` runs), drawn as glyphs over the picture so
//! it is real text: visible, extractable from the PDF and searchable, as Word draws it. The
//! metafile reader has no fonts; the runs are shaped here with the face Word would use.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use doc4me_doc::Document;
use doc4me_doc::props::Rgb;
use doc4me_geom::{Rect, Spin};
use doc4me_metafile::Text;

use crate::display::Draw;

/// GDI text alignment flags.
const TA_RIGHT: u32 = 2;
const TA_CENTER: u32 = 6;
const TA_BOTTOM: u32 = 8;
const TA_BASELINE: u32 = 24;
/// Pictures whose text is kept between draws.
const CACHE_PICTURES: usize = 64;
/// Text runs drawn from one picture.
const MAX_RUNS: usize = 20_000;

/// A picture's size and text runs.
struct Runs {
    width: f32,
    height: f32,
    runs: Vec<Text>,
    /// An EMF+ dual file: its text is drawn from the glyphs' outlines (see
    /// [`doc4me_metafile::has_emf_plus`]).
    outlines: bool,
}

type Cache = Mutex<HashMap<(usize, usize), Option<Arc<Runs>>>>;

/// The text runs of the metafile `bytes`, parsed once per media (keyed by the buffer it lives in).
fn runs(bytes: &Arc<Vec<u8>>) -> Option<Arc<Runs>> {
    static CACHE: OnceLock<Cache> = OnceLock::new();
    let key = (Arc::as_ptr(bytes) as usize, bytes.len());
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(hit) = cache.lock().unwrap_or_else(|e| e.into_inner()).get(&key) {
        return hit.clone();
    }
    let parsed = doc4me_metafile::is_metafile(bytes)
        .then(|| doc4me_metafile::parse(bytes).ok())
        .flatten()
        .map(|pic| {
            let runs: Vec<Text> =
                pic.items.into_iter().filter_map(|it| if let doc4me_metafile::Item::Text(t) = it { Some(t) } else { None }).take(MAX_RUNS).collect();
            Arc::new(Runs { width: pic.width, height: pic.height, runs, outlines: doc4me_metafile::has_emf_plus(bytes) })
        })
        .filter(|r| !r.runs.is_empty());
    let mut c = cache.lock().unwrap_or_else(|e| e.into_inner());
    if c.len() >= CACHE_PICTURES {
        c.clear();
    }
    c.insert(key, parsed.clone());
    parsed
}

/// The text of the metafile picture `media` drawn in `rect` with `crop`, kept to `rect`. Nothing
/// when the media isn't a metafile or has no text.
pub(crate) fn draws(doc: &Document, media: &str, rect: Rect, crop: [f32; 4], alpha: f32) -> Option<Draw> {
    let bytes = doc.media.get(media)?;
    let runs = runs(bytes)?;
    let pic = doc4me_metafile::Picture { width: runs.width, height: runs.height, items: Vec::new() };
    let target = kurbo::Rect::new(f64::from(rect.x), f64::from(rect.y), f64::from(rect.x + rect.w), f64::from(rect.y + rect.h));
    let tf = pic.fit(target, crop)?;
    let mut items = Vec::new();
    for t in &runs.runs {
        run(t, tf, alpha, runs.outlines, &mut items);
    }
    (!items.is_empty()).then_some(Draw::Clip { rect, items })
}

/// One text run placed by `tf` (picture to page).
fn run(t: &Text, tf: kurbo::Affine, alpha: f32, outlines: bool, out: &mut Vec<Draw>) {
    let c = tf.as_coeffs();
    let lin = |x: f64, y: f64| (c[0] * x + c[2] * y, c[1] * x + c[3] * y);
    let at = tf * kurbo::Point::new(f64::from(t.x), f64::from(t.y));
    let (dx, dy) = lin(f64::from(t.dir.0), f64::from(t.dir.1));
    let along = dx.hypot(dy);
    let (px, py) = lin(-f64::from(t.dir.1), f64::from(t.dir.0));
    let across = px.hypot(py);
    if !(along > 0.0 && across > 0.0 && at.x.is_finite() && at.y.is_finite()) {
        return;
    }
    let family = if t.face.trim().is_empty() { "Arial" } else { t.face.as_str() };
    let r = doc4me_fonts::word::resolve(family, t.bold, t.italic);
    let face = r.face;
    let upem = face.upem.max(1.0);
    let (asc, desc) = (face.ascent.abs(), face.descent.abs());
    // A cell height holds the em and the internal leading: the font's ascent plus descent.
    let height = f64::from(t.size) * across;
    let em = if t.cell && asc + desc > 0.0 { height * upem / (asc + desc) } else { height };
    // Too small to see or too big to be text (a hostile file): left out.
    if !(em.is_finite() && em > 0.05 && em < 2000.0) {
        return;
    }
    let shaped = doc4me_fonts::shape(&face, &t.text, &[], |c| c);
    if shaped.is_empty() {
        return;
    }
    // Where each character starts along the baseline: the file's advances when it gives them.
    let char_starts: Vec<usize> = t.text.char_indices().map(|(i, _)| i).collect();
    let advance_at: Option<Vec<f64>> = (t.advances.len() == char_starts.len()).then(|| {
        let mut acc = 0.0;
        let mut v = Vec::with_capacity(char_starts.len() + 1);
        for a in &t.advances {
            v.push(acc);
            acc += f64::from(*a) * along;
        }
        v.push(acc);
        v
    });
    let k = em / upem;
    let mut glyphs: Vec<(u32, f64, f64)> = Vec::with_capacity(shaped.len());
    let mut ranges = Vec::with_capacity(shaped.len());
    let mut pen = 0.0;
    for (gi, g) in shaped.iter().enumerate() {
        let x = match &advance_at {
            Some(v) => char_starts.iter().position(|s| *s == g.cluster).and_then(|ci| v.get(ci)).copied().unwrap_or(pen),
            None => pen,
        };
        glyphs.push((g.gid, x + f64::from(g.x_offset) * k, -f64::from(g.y_offset) * k));
        pen = x + f64::from(g.x_advance) * k;
        let end = shaped.iter().skip(gi + 1).map(|n| n.cluster).find(|n| *n > g.cluster).unwrap_or(t.text.len());
        ranges.push(g.cluster.min(end)..end);
    }
    let width = advance_at.as_ref().and_then(|v| v.last().copied()).unwrap_or(pen);
    let u0 = if t.align & TA_CENTER == TA_CENTER {
        -width / 2.0
    } else if t.align & TA_RIGHT != 0 {
        -width
    } else {
        0.0
    };
    let v0 = match t.align & TA_BASELINE {
        TA_BASELINE => 0.0,
        TA_BOTTOM => -desc / upem * em,
        _ => asc / upem * em,
    };
    let [cr, cg, cb, _] = t.color;
    let deg = dy.atan2(dx).to_degrees();
    if outlines {
        let turn = kurbo::Affine::rotate_about(deg.to_radians(), at);
        let mut path = doc4me_fonts::BezPath::new();
        let db = doc4me_fonts::FontDb::global();
        for (g, x, y) in &glyphs {
            let place = turn * kurbo::Affine::translate((at.x + u0 + x, at.y + v0 + y)) * kurbo::Affine::scale(k);
            path.extend((place * (*db.outline(&face, *g)).clone()).elements().iter().copied());
        }
        out.push(Draw::Art { path, fx: Box::default(), color: Rgb(cr, cg, cb), alpha });
        return;
    }
    let glyphs: Vec<(u32, f32, f32)> = glyphs.into_iter().map(|(g, x, y)| (g, (at.x + u0 + x) as f32, (at.y + v0 + y) as f32)).collect();
    let draw = Draw::Glyphs {
        face,
        size: em as f32,
        glyphs,
        color: Rgb(cr, cg, cb),
        alpha,
        synth_bold: r.synth_bold,
        synth_italic: r.synth_italic,
        text: t.text.clone(),
        link: None,
        ranges,
        preserve_words: false,
    };
    if deg.abs() < 0.01 {
        out.push(draw);
    } else {
        // Turned about the reference point (a degenerate rect there).
        out.push(Draw::Rotated { cx: at.x as f32, cy: at.y as f32, spin: Spin::new(deg as f32, false, false), items: vec![draw] });
    }
}
