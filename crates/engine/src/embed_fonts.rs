//! Fonts embedded when a Word document is saved (File › Options › Save › Embed fonts in the file,
//! [`Settings::embed_fonts`](doc4me_doc::Settings)).
//!
//! The document is laid out and every face its text is drawn in is considered: a face is embedded
//! under the font name the text asks for when it *is* that font (a substitute shown for a font
//! that isn't installed, or a fallback for characters the font lacks, is not embedded under a
//! name it doesn't have). Common system fonts are left out unless `embed_system_fonts` is on, as
//! in Word. Each face's own licence decides the rest (`doc4me_fonts::embed`). Faces that can't
//! be embedded are reported once, after the save ([`take_report`]).

use std::sync::Mutex;

use doc4me_doc::Document;
use doc4me_docx::EmbedFont;

/// Fonts that come with Windows, macOS or Office, which a reader has already: Word's "Do not
/// embed common system fonts" leaves them out. Compared case-insensitively.
const COMMON_SYSTEM_FONTS: &[&str] = &[
    "Arial",
    "Arial Black",
    "Calibri",
    "Calibri Light",
    "Cambria",
    "Cambria Math",
    "Candara",
    "Comic Sans MS",
    "Consolas",
    "Constantia",
    "Corbel",
    "Courier New",
    "Franklin Gothic Medium",
    "Gabriola",
    "Georgia",
    "Impact",
    "Lucida Console",
    "Lucida Sans Unicode",
    "Malgun Gothic",
    "Microsoft JhengHei",
    "Microsoft YaHei",
    "MS Gothic",
    "MS Mincho",
    "Palatino Linotype",
    "Segoe Print",
    "Segoe Script",
    "Segoe UI",
    "Segoe UI Symbol",
    "SimSun",
    "Symbol",
    "Tahoma",
    "Times New Roman",
    "Trebuchet MS",
    "Verdana",
    "Webdings",
    "Wingdings",
    "Yu Gothic",
    // Thai fonts that come with Windows and Office.
    "Angsana New",
    "AngsanaUPC",
    "Browallia New",
    "BrowalliaUPC",
    "Cordia New",
    "CordiaUPC",
    "DilleniaUPC",
    "EucrosiaUPC",
    "FreesiaUPC",
    "IrisUPC",
    "JasmineUPC",
    "KodchiangUPC",
    "Leelawadee",
    "Leelawadee UI",
    "LilyUPC",
    "Microsoft Sans Serif",
    // macOS.
    "Helvetica",
    "Helvetica Neue",
    "Menlo",
    "Thonburi",
];

/// What the last save left out, for the status line.
static REPORT: Mutex<Vec<(String, &'static str)>> = Mutex::new(Vec::new());

/// Whether `family` is one of [`COMMON_SYSTEM_FONTS`].
pub fn is_common_system_font(family: &str) -> bool {
    let family = family.trim();
    COMMON_SYSTEM_FONTS.iter().any(|f| f.eq_ignore_ascii_case(family))
}

/// The fonts to embed in `doc`, and the fonts that couldn't be (name, why).
pub fn collect(doc: &Document) -> (Vec<EmbedFont>, Vec<(String, &'static str)>) {
    let s = &doc.settings;
    let layout = doc4me_layout::layout(doc, &mut doc4me_layout::LayoutCache::new(), &doc4me_layout::LayoutOptions::default());
    let mut fonts: Vec<EmbedFont> = Vec::new();
    let mut skipped: Vec<(String, &'static str)> = Vec::new();
    for used in layout.used_faces() {
        let face = used.face.get();
        // The name the text asks for that this face is; none: a substitute or fallback.
        let Some(name) = used.fonts.iter().find(|f| f.eq_ignore_ascii_case(face.family.trim()) || f.eq_ignore_ascii_case(face.legacy_family.trim()))
        else {
            continue;
        };
        if !s.embed_system_fonts && is_common_system_font(name) {
            continue;
        }
        let bold = face.weight >= 600.0;
        let italic = face.italic;
        if fonts.iter().any(|f| f.family.eq_ignore_ascii_case(name) && f.bold == bold && f.italic == italic) {
            continue;
        }
        match doc4me_fonts::embed::prepare(face, s.embed_subset.then_some(&used.gids)) {
            Ok(p) => fonts.push(EmbedFont { family: name.clone(), bold, italic, data: p.data, subsetted: p.subsetted }),
            Err(why) => {
                if !skipped.iter().any(|(f, _)| f.eq_ignore_ascii_case(name)) {
                    skipped.push((name.clone(), why.reason()));
                }
            }
        }
    }
    (fonts, skipped)
}

/// Whether the font table `doc` was read with embeds fonts.
pub fn file_embeds_fonts(doc: &Document) -> bool {
    doc.passthrough.get("word/fontTable.xml").is_some_and(|t| t.windows(7).any(|w| w == b":embedR" || w == b":embedB" || w == b":embedI"))
}

/// Write `doc` as a Word package of `flavor`, embedding fonts as its settings say: with the
/// option on, the fonts it is drawn in; with it off, none (fonts the file embedded are dropped,
/// as Word does).
pub fn write(doc: &Document, flavor: doc4me_docx::Flavor) -> Result<Vec<u8>, String> {
    if doc.settings.embed_fonts {
        let (fonts, skipped) = collect(doc);
        if let Ok(mut r) = REPORT.lock() {
            *r = skipped;
        }
        doc4me_docx::write_as_embedding(doc, flavor, &fonts).map_err(|e| e.to_string())
    } else if file_embeds_fonts(doc) {
        doc4me_docx::write_as_embedding(doc, flavor, &[]).map_err(|e| e.to_string())
    } else {
        doc4me_docx::write_as(doc, flavor).map_err(|e| e.to_string())
    }
}

/// The fonts the last save couldn't embed, as a status line (`None`: all were). Taking it
/// clears it.
pub fn take_report() -> Option<String> {
    let skipped = REPORT.lock().map(|mut r| std::mem::take(&mut *r)).unwrap_or_default();
    match skipped.as_slice() {
        [] => None,
        [(font, why)] => Some(format!("{font} isn't embedded: {why}")),
        [first, rest @ ..] => Some(format!("{} isn't embedded: {}; {} more font(s) aren't either", first.0, first.1, rest.len())),
    }
}
