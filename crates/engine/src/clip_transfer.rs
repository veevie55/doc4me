//! The system clipboard: what Copy and Cut hand to other programs and other Doc4Me windows, and
//! how Paste reads it back.
//!
//! Doc4Me's own copy ([`Session::clipboard`]) is a rich [`Fragment`] that lives in this process.
//! To carry text boxes, pictures, shapes, icons, tables, charts, 3D models, SmartArt, equations,
//! symbols and WordArt to another program, or to Doc4Me running in another process, a copy is
//! also offered in shared formats (see [`export`]):
//!
//! - **HTML** with pictures as `data:` URIs. Objects HTML can't describe (shapes, text boxes,
//!   WordArt, groups, charts, SmartArt, equations) become pictures of themselves, drawn by our
//!   own renderer, as Word does for programs that don't understand them. The HTML also carries
//!   the copy itself, losslessly, as a Word document (DOCX) in a comment ([`MARKER`]), which
//!   other programs ignore and Doc4Me reads back with everything editable.
//! - **RTF** with the same pictures (`\pngblip`), for programs that prefer it (Windows).
//! - **A picture** (PNG) when the copy is one object alone, for picture programs.
//! - **Plain text**.
//!
//! Everything read back from the clipboard comes from other programs and is hostile: sizes are
//! capped, the payload goes through the bounded DOCX reader, and a damaged payload is refused
//! (the caller then falls back to the HTML or text).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use doc4me_doc::edit::Fragment;
use doc4me_doc::para::{InlineObject, OBJ};
use doc4me_doc::props::CharProps;
use doc4me_doc::{Block, Document, Paragraph};

use crate::Session;

/// The comment that carries a copy between Doc4Me windows inside clipboard HTML:
/// `<!--Doc4MeClipboard v1 id=<token> blocks=<n> docx=<base64>-->`.
pub const MARKER: &str = "Doc4MeClipboard";
/// Largest DOCX payload written to or read from the clipboard (encoded size is a third more).
pub const MAX_PAYLOAD_BYTES: usize = 48 << 20;
/// Largest clipboard HTML Paste reads (the payload plus the HTML around it).
pub const MAX_HTML_BYTES: usize = 96 << 20;
/// Most objects drawn as pictures for one copy; later ones keep their text only.
const MAX_DRAWN: usize = 64;
/// Largest side, points, of an object drawn as a picture.
const MAX_DRAWN_SIDE: f32 = 3000.0;
/// Pixels per point for drawn objects (144 dpi: sharp on high-density screens and in print).
const DRAW_SCALE: f32 = 2.0;
/// Longest side, pixels, of a drawn object's picture: bigger objects are drawn at a lower scale.
const MAX_DRAWN_PIXELS: f32 = 2400.0;

/// What Copy puts on the system clipboard.
#[derive(Clone, Debug, Default)]
pub struct SystemClip {
    /// Plain text (paragraphs separated by `\n`; hosts convert to the system's line ends).
    pub text: String,
    /// An HTML fragment (the inside of `<body>`), with the lossless copy in a comment.
    pub html: String,
    /// The copy as RTF, when asked for ([`ExportOptions::rtf`]).
    pub rtf: Option<String>,
    /// PNG bytes when the copy is a single object (a picture, shape, chart, equation…).
    pub png: Option<Vec<u8>>,
    /// The token in the HTML's [`MARKER`]: the same token read back means this copy.
    pub token: String,
}

/// Which extra formats [`export`] makes (each costs time on a large copy).
#[derive(Clone, Copy, Debug, Default)]
pub struct ExportOptions {
    pub rtf: bool,
    pub png: bool,
}

/// A new token naming one copy. Not secret: it only tells our own copy from another's, and a
/// forged one could at most paste our own copy.
fn new_token() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    // The browser has neither a process id nor `SystemTime` (both panic there).
    #[cfg(not(target_arch = "wasm32"))]
    let (pid, t) = (std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0));
    #[cfg(target_arch = "wasm32")]
    let (pid, t) = (0u32, 0u64);
    format!("{pid:x}-{t:x}-{n:x}")
}

/// A document holding `f` and everything it refers to.
pub fn fragment_document(f: &Fragment) -> Document {
    let mut d = Document::new();
    if let Some(st) = &f.styles {
        d.styles = st.clone();
    }
    if let Some(n) = &f.numbering {
        d.numbering = n.clone();
    }
    d.media = f.media.clone();
    d.passthrough = f.passthrough.clone();
    d.parts = f.parts.clone();
    d.body = f.blocks.iter().cloned().map(Arc::new).collect();
    // A section break inside the copy belongs to the source's layout, not to the copy's end.
    d.ensure_nonempty();
    d
}

/// The system clipboard formats for Doc4Me's own copy, or `None` when there is none. Records the
/// copy's token in the session, so Paste knows the HTML read back is this copy.
pub fn export(s: &mut Session, opts: ExportOptions) -> Option<SystemClip> {
    let f = s.clipboard.as_ref()?;
    let token = new_token();
    let clip = export_fragment(f, &token, opts);
    s.clipboard_token = Some(token);
    Some(clip)
}

/// [`export`] for any fragment, marked with `token`.
pub fn export_fragment(f: &Fragment, token: &str, opts: ExportOptions) -> SystemClip {
    let doc = fragment_document(f);
    let text = f.plain_text();
    // The lossless copy: a Word document. Too big (or unwritable) leaves the shared formats only.
    let payload = crate::io::save_bytes("clipboard.docx", &doc).ok().filter(|b| b.len() <= MAX_PAYLOAD_BYTES);
    // Other programs: objects they can't read become pictures of themselves.
    let mut shared = doc.clone();
    draw_objects_as_pictures(&mut shared);
    let mut html = String::new();
    if let Some(bytes) = &payload {
        html.push_str(&format!("<!--{MARKER} v1 id={token} blocks={} docx=", f.blocks.len()));
        html.push_str(&crate::cmd::insert::base64_encode(bytes));
        html.push_str("-->");
    }
    html.push_str(&html_body(&doc4me_formats::html::export(&shared)));
    let rtf = opts.rtf.then(|| doc4me_formats::rtf::export(&shared));
    let png = if opts.png { single_object_png(&doc) } else { None };
    SystemClip { text, html, rtf, png, token: token.to_string() }
}

/// The inside of an HTML document's `<body>` (all of it when there is no body).
fn html_body(html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<body").and_then(|i| lower.get(i..).and_then(|rest| rest.find('>')).map(|j| i + j + 1));
    let end = lower.rfind("</body>");
    match (start, end) {
        (Some(a), Some(b)) if a <= b => html.get(a..b).unwrap_or(html).trim().to_string(),
        _ => html.to_string(),
    }
}

/// The copy carried in clipboard HTML's [`MARKER`] comment.
#[derive(Clone, Debug, PartialEq)]
pub struct Payload {
    /// The token of the copy that wrote it.
    pub token: String,
    /// How many blocks the copy had (a Word document may need a paragraph after a table).
    pub blocks: usize,
    /// The DOCX bytes.
    pub docx: Vec<u8>,
}

/// The value of `key=` in the marker's header (`id=…`, `blocks=…`).
fn field<'a>(header: &'a str, key: &str) -> Option<&'a str> {
    header.split_ascii_whitespace().find_map(|w| w.strip_prefix(key)?.strip_prefix('='))
}

/// Find and decode the [`MARKER`] comment in clipboard HTML. `None` when there is none, or it is
/// damaged or too big.
pub fn payload_from_html(html: &str) -> Option<Payload> {
    if html.len() > MAX_HTML_BYTES {
        return None;
    }
    let start = html.find(&format!("<!--{MARKER} "))?;
    let rest = html.get(start + 4 + MARKER.len()..)?;
    let end = rest.find("-->")?;
    let body = rest.get(..end)?;
    let (header, data) = body.split_once("docx=")?;
    if header.split_ascii_whitespace().next() != Some("v1") {
        return None;
    }
    let token = field(header, "id")?.chars().take(64).collect::<String>();
    let blocks = field(header, "blocks")?.parse::<usize>().ok()?;
    if data.len() > MAX_PAYLOAD_BYTES.saturating_mul(4) / 3 + 8 {
        return None;
    }
    let docx = crate::cmd::insert::base64_decode(data.trim())?;
    if docx.is_empty() || docx.len() > MAX_PAYLOAD_BYTES {
        return None;
    }
    Some(Payload { token, blocks, docx })
}

/// The fragment a payload carries, with the pictures, styles, lists, text boxes, notes and
/// object data it uses. An error when the document can't be read.
pub fn fragment_from_payload(p: &Payload) -> Result<Fragment, String> {
    let mut doc = crate::io::open_bytes("clipboard.docx", &p.docx)?;
    doc.normalize_runs();
    // A Word document can't end with a table: the writer added a paragraph after it. Drop it
    // (only an empty one) so the paste ends where the copy did.
    while doc.body.len() > p.blocks.max(1) && doc.body.last().is_some_and(|b| b.as_para().is_some_and(Paragraph::is_empty)) {
        doc.body.pop();
    }
    for b in &doc.body {
        check_objects(b)?;
    }
    for part in doc.parts.values() {
        for b in &part.blocks {
            check_objects(b)?;
        }
    }
    let mut f = doc.body_fragment();
    for b in &mut f.blocks {
        if let Block::Para(p) = b {
            p.section = None;
        }
    }
    Ok(f)
}

/// Every paragraph's objects match its object characters (the reader keeps them in step; this
/// is the last check before the copy reaches the document).
fn check_objects(b: &Block) -> Result<(), String> {
    let mut todo = vec![b];
    while let Some(b) = todo.pop() {
        match b {
            Block::Para(p) => {
                if p.text.chars().filter(|&c| c == OBJ).count() != p.objects.len() {
                    return Err("the copied content is damaged".into());
                }
            }
            Block::Table(t) => todo.extend(t.rows.iter().flat_map(|r| &r.cells).flat_map(|c| &c.blocks).map(|b| &**b)),
        }
    }
    Ok(())
}

/// Whether other programs need `o` drawn as a picture to see it.
fn drawn_for_others(o: &InlineObject) -> bool {
    matches!(o, InlineObject::Shape { .. } | InlineObject::Group { .. } | InlineObject::Graphic { .. } | InlineObject::Equation { .. })
}

/// Replace the objects other programs can't read with pictures of themselves (at most
/// [`MAX_DRAWN`]); the pictures keep the objects' size and alternative text.
pub fn draw_objects_as_pictures(doc: &mut Document) {
    let source = doc.clone();
    let mut drawn = 0usize;
    let mut body: Vec<Block> = doc.body.iter().map(|b| (**b).clone()).collect();
    for b in &mut body {
        crate::cmd::paste::each_para(b, 0, &mut |p| {
            for o in &mut p.objects {
                if drawn >= MAX_DRAWN || !drawn_for_others(o) {
                    continue;
                }
                drawn += 1;
                let Some((png, w, h)) = object_png(&source, o) else { continue };
                let alt = match &*o {
                    InlineObject::Graphic { alt, .. } => alt.clone(),
                    InlineObject::Equation { linear, math, .. } => {
                        if linear.is_empty() {
                            doc4me_doc::math::to_linear(&math.nodes)
                        } else {
                            linear.clone()
                        }
                    }
                    other => other.plain_text().to_string(),
                };
                let media = doc.add_media(png, "png");
                *o = InlineObject::Image { media, w, h, alt, float: Default::default(), crop: [0.0; 4], ole: None, video: None };
            }
        });
    }
    doc.body = body.into_iter().map(Arc::new).collect();
}

/// The picture of a copy that is one object alone (a picture's own bytes when it is a PNG).
fn single_object_png(doc: &Document) -> Option<Vec<u8>> {
    let [b] = doc.body.as_slice() else { return None };
    let p = b.as_para()?;
    let [o] = p.objects.as_slice() else { return None };
    if p.text != OBJ.to_string() {
        return None;
    }
    if let InlineObject::Image { media, crop, .. } = o
        && *crop == [0.0; 4]
        && let Some(bytes) = doc.media.get(media)
        && bytes.starts_with(b"\x89PNG")
    {
        return Some(bytes.to_vec());
    }
    if !o.is_drawing() && !matches!(o, InlineObject::Equation { .. }) {
        return None;
    }
    object_png(doc, o).map(|(png, _, _)| png)
}

/// Draw `o` (with `source`'s styles, pictures and text boxes) on a page of its own and return the
/// PNG cropped to what was drawn, with its size in points.
pub fn object_png(source: &Document, o: &InlineObject) -> Option<(Vec<u8>, f32, f32)> {
    let mut o = o.clone();
    if let Some(f) = o.float_mut() {
        *f = Default::default();
    }
    let framed = o.frame().map(|(w, h, _)| (w, h));
    // An equation has no frame: room for a long display equation; the crop finds its size.
    let (w, h) = framed.unwrap_or((480.0, 160.0));
    if !(w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0 && w <= MAX_DRAWN_SIDE && h <= MAX_DRAWN_SIDE) {
        return None;
    }
    let mut d = Document::new();
    d.styles = source.styles.clone();
    d.numbering = source.numbering.clone();
    d.media = source.media.clone();
    d.parts = source.parts.clone();
    d.passthrough = source.passthrough.clone();
    let pad = 6.0;
    let sp = &mut d.last_section;
    sp.page_w = w + 2.0 * pad;
    sp.page_h = h + 2.0 * pad + 96.0;
    sp.landscape = false;
    (sp.margin_top, sp.margin_bottom, sp.margin_left, sp.margin_right) = (pad, pad, pad, pad);
    (sp.header, sp.footer, sp.gutter) = (0.0, 0.0, 0.0);
    sp.columns = Default::default();
    sp.page_borders = None;
    sp.line_numbers = None;
    let mut p = Paragraph::new();
    p.insert_object(0, o, &CharProps::default()).ok()?;
    d.body = vec![Arc::new(Block::Para(p))];
    let layout = doc4me_layout::layout(&d, &mut doc4me_layout::LayoutCache::new(), &Default::default());
    let page = layout.pages.first()?;
    let scale = DRAW_SCALE.min(MAX_DRAWN_PIXELS / w.max(h).max(1.0));
    let r = doc4me_render::render_page(&d, page, scale, &Default::default());
    let (x0, y0, x1, y1) = ink_bounds(&r)?;
    let pixels = r.to_straight();
    let img = image::RgbaImage::from_raw(r.width, r.height, pixels)?;
    let cropped = image::imageops::crop_imm(&img, x0, y0, x1 - x0, y1 - y0).to_image();
    let mut png = Vec::new();
    cropped.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).ok()?;
    let (pw, ph) = ((x1 - x0) as f32 / scale, (y1 - y0) as f32 / scale);
    // A drawing keeps its own size; an equation takes the size it was drawn at.
    let (w, h) = framed.unwrap_or((pw, ph));
    Some((png, w, h))
}

/// The box (x0, y0, x1, y1), pixels, holding everything that isn't white paper.
fn ink_bounds(r: &doc4me_render::Rendered) -> Option<(u32, u32, u32, u32)> {
    let (w, h) = (r.width as usize, r.height as usize);
    if w == 0 || h == 0 || r.pixels.len() != w.checked_mul(h)?.checked_mul(4)? {
        return None;
    }
    let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0usize, 0usize);
    for (i, px) in r.pixels.chunks_exact(4).enumerate() {
        if px.get(..3).is_some_and(|c| c.iter().all(|&v| v >= 250)) {
            continue;
        }
        let (x, y) = (i % w, i / w);
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x + 1);
        y1 = y1.max(y + 1);
    }
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    Some((u32::try_from(x0).ok()?, u32::try_from(y0).ok()?, u32::try_from(x1).ok()?, u32::try_from(y1).ok()?))
}

/// Most pixels in a clipboard bitmap made by [`bmp_from_png`] (a 24-bit DIB of 120 MB).
const MAX_BMP_PIXELS: u64 = 40_000_000;

/// A Windows bitmap file (`BITMAPFILEHEADER` + 40-byte `BITMAPINFOHEADER`, 24-bit, bottom-up)
/// of a PNG, on white: what programs that only read the clipboard's bitmap format get.
pub fn bmp_from_png(png: &[u8]) -> Option<Vec<u8>> {
    let img = image::load_from_memory_with_format(png, image::ImageFormat::Png).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 || u64::from(w) * u64::from(h) > MAX_BMP_PIXELS {
        return None;
    }
    let row = (usize::try_from(w).ok()?.checked_mul(3)?).checked_add(3)? & !3;
    let image_len = row.checked_mul(usize::try_from(h).ok()?)?;
    let off = 14 + 40;
    let total = off + image_len;
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&u32::try_from(total).ok()?.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&(off as u32).to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&i32::try_from(w).ok()?.to_le_bytes());
    out.extend_from_slice(&i32::try_from(h).ok()?.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    out.extend_from_slice(&u32::try_from(image_len).ok()?.to_le_bytes());
    out.extend_from_slice(&2835i32.to_le_bytes()); // 72 dpi
    out.extend_from_slice(&2835i32.to_le_bytes());
    out.extend_from_slice(&[0; 8]);
    let pad = row - w as usize * 3;
    for y in (0..h).rev() {
        for x in 0..w {
            let [r, g, b, a] = img.get_pixel(x, y).0;
            let over = |c: u8| ((u32::from(c) * u32::from(a) + 255 * (255 - u32::from(a)) + 127) / 255) as u8;
            out.extend_from_slice(&[over(b), over(g), over(r)]);
        }
        out.extend(std::iter::repeat_n(0u8, pad));
    }
    Some(out)
}
