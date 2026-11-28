//! The font table written with fonts embedded on save (File › Options › Save › Embed fonts in
//! the file, `w:embedTrueTypeFonts`).
//!
//! The caller (the engine, which knows which faces the text is drawn in) hands over the font
//! files; this writes `word/fontTable.xml` with an entry per family, each embedded face as
//! `w:embedRegular`/`Bold`/`Italic`/`BoldItalic` pointing at an obfuscated `word/fonts/fontN.odttf`
//! with its `w:fontKey` (ECMA-376 Part 1, §17.8.1 and §17.8.3.3). The file's own font table entries
//! are kept (panose, charset, signature…), without the fonts they embedded before: what is
//! embedded is decided again at every save, as Word does.

use std::collections::HashSet;

use doc4me_doc::Document;

use super::PartRels;
use crate::package::FONT_TABLE_PART;
use crate::xml::{self, El, Node, W};

/// The relationship type of an embedded font.
const FONT_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/font";
/// Content type of an obfuscated font part.
pub(crate) const OBFUSCATED_FONT: &str = "application/vnd.openxmlformats-officedocument.obfuscatedFont";
/// Most font files embedded in one document (the reader takes as many).
const MAX_FONTS: usize = 64;
/// Most bytes of fonts embedded in one document.
const MAX_FONT_BYTES: usize = 256 * 1024 * 1024;
/// Most entries kept from the file's own font table.
const MAX_ENTRIES: usize = 4096;

/// A font file to embed: one face of a family.
#[derive(Clone, Debug)]
pub struct EmbedFont {
    /// The family name runs use (`w:rFonts`), and the font table entry's `w:name`.
    pub family: String,
    pub bold: bool,
    pub italic: bool,
    /// A plain TrueType file; obfuscated here.
    pub data: Vec<u8>,
    /// Only the glyphs the document uses (`w:subsetted`).
    pub subsetted: bool,
}

impl EmbedFont {
    /// The font table element naming this face's slot, in CT_Font's order.
    fn slot(&self) -> (usize, &'static str) {
        match (self.bold, self.italic) {
            (false, false) => (0, "w:embedRegular"),
            (true, false) => (1, "w:embedBold"),
            (false, true) => (2, "w:embedItalic"),
            (true, true) => (3, "w:embedBoldItalic"),
        }
    }
}

/// What [`font_table`] writes: the table, its relationships, and the font parts (name, bytes).
pub(crate) struct FontTable {
    pub xml: Vec<u8>,
    pub rels: PartRels,
    pub fonts: Vec<(String, Vec<u8>)>,
}

/// The font table for `doc` embedding `fonts`; `taken` says whether a part name is already used.
/// `None` when there is nothing to write: no fonts, and no table in the file.
pub(crate) fn font_table(doc: &Document, fonts: &[EmbedFont], taken: impl Fn(&str) -> bool) -> Option<FontTable> {
    let mut entries: Vec<El> = doc
        .passthrough
        .get(FONT_TABLE_PART)
        .and_then(|b| xml::parse(b).ok())
        .map(|t| t.children("w:font").take(MAX_ENTRIES).map(without_embeds).collect())
        .unwrap_or_default();
    if entries.is_empty() && fonts.is_empty() {
        return None;
    }
    let mut rels = PartRels::default();
    let mut parts = Vec::new();
    let mut slots: HashSet<(String, usize)> = HashSet::new();
    let mut bytes = 0usize;
    let mut next = 1u32;
    for font in fonts {
        let family = font.family.trim();
        let (order, element) = font.slot();
        if family.is_empty() || font.data.is_empty() || !slots.insert((family.to_lowercase(), order)) {
            continue;
        }
        if parts.len() >= MAX_FONTS || bytes.saturating_add(font.data.len()) > MAX_FONT_BYTES {
            log::warn!("docx: font embedding limit reached; {family} is not embedded");
            continue;
        }
        let name = loop {
            let candidate = format!("word/fonts/font{next}.odttf");
            next += 1;
            if !taken(&candidate) {
                break candidate;
            }
        };
        let guid = new_guid();
        let mut data = font.data.clone();
        if let Some(key) = crate::read::font_key(&guid) {
            for (i, b) in data.iter_mut().take(32).enumerate() {
                *b ^= key.get(i % 16).copied().unwrap_or(0);
            }
        }
        bytes += data.len();
        let target = name.strip_prefix("word/").unwrap_or(&name).to_string();
        let id = rels.add(FONT_REL, &target, false);
        let mut attrs = vec![("r:id".to_string(), id), ("w:fontKey".to_string(), guid)];
        if font.subsetted {
            attrs.push(("w:subsetted".to_string(), "1".to_string()));
        }
        let embed = El { name: element.to_string(), attrs, kids: Vec::new() };
        let at = match entries.iter().position(|e| e.attr("w:name").is_some_and(|n| n.trim().eq_ignore_ascii_case(family))) {
            Some(i) => i,
            None => {
                entries.push(El { name: "w:font".into(), attrs: vec![("w:name".into(), family.to_string())], kids: Vec::new() });
                entries.len() - 1
            }
        };
        if let Some(entry) = entries.get_mut(at) {
            insert_embed(entry, embed, order);
        }
        parts.push((name, data));
    }
    let mut w = W::new();
    w.open(
        "w:fonts",
        &[
            ("xmlns:w", "http://schemas.openxmlformats.org/wordprocessingml/2006/main"),
            ("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships"),
        ],
    );
    for e in &entries {
        w.raw(&e.to_xml());
    }
    w.close("w:fonts");
    Some(FontTable { xml: w.into_bytes(), rels, fonts: parts })
}

/// The embedding elements, in CT_Font's order.
const EMBEDS: [&str; 4] = ["w:embedRegular", "w:embedBold", "w:embedItalic", "w:embedBoldItalic"];

/// A font table entry without the fonts it embedded, and without anything outside the
/// WordprocessingML namespace (whose declarations this table doesn't repeat).
fn without_embeds(e: &El) -> El {
    let kids = e
        .kids
        .iter()
        .filter(|n| match n {
            Node::El(c) => c.name.starts_with("w:") && !EMBEDS.contains(&c.name.as_str()),
            Node::Text(_) => false,
        })
        .cloned()
        .collect();
    let attrs = e.attrs.iter().filter(|(k, _)| k.starts_with("w:")).cloned().collect();
    El { name: e.name.clone(), attrs, kids }
}

/// Put `embed` among `entry`'s embedding elements in slot order (they come last in CT_Font).
fn insert_embed(entry: &mut El, embed: El, order: usize) {
    let after = |n: &Node| matches!(n, Node::El(c) if EMBEDS.iter().position(|e| *e == c.name).is_some_and(|i| i > order));
    match entry.kids.iter().position(after) {
        Some(i) => entry.kids.insert(i, Node::El(embed)),
        None => entry.kids.push(Node::El(embed)),
    }
}

/// A fresh `{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}` key. Without randomness (it never fails on
/// the platforms built for) the all-zero key, which leaves the font unobfuscated.
fn new_guid() -> String {
    let mut b = [0u8; 16];
    if getrandom::fill(&mut b).is_err() {
        b = [0; 16];
    }
    let hex: String = b.iter().map(|x| format!("{x:02X}")).collect();
    let part = |r: std::ops::Range<usize>| hex.get(r).unwrap_or("").to_string();
    format!("{{{}-{}-{}-{}-{}}}", part(0..8), part(8..12), part(12..16), part(16..20), part(20..32))
}
