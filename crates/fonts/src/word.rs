//! Word-processor font resolution: map a requested family + bold/italic to an available face,
//! with metric-compatible substitutes for common proprietary document fonts, and Word-style line
//! metrics (Windows ascent/descent, which is what Word uses for single line spacing).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use skrifa::raw::TableProvider;

use crate::{FaceRef, FontDb};

/// A requested font resolved to a face.
#[derive(Clone, Copy, Debug)]
pub struct Resolved {
    pub face: FaceRef,
    /// The face isn't bold but bold was asked for: embolden when drawing.
    pub synth_bold: bool,
    /// The face isn't italic but italic was asked for: slant when drawing.
    pub synth_italic: bool,
    /// The requested family wasn't available.
    pub substituted: bool,
}

/// Substitutes to try, in order, for a family that isn't installed. Bundled open fonts come before
/// proprietary system fonts, so the same document gets the same freely licensed face on Windows
/// and Linux. Intos ↔ Aptos, Carlito ↔ Calibri, Caladea ↔ Cambria, Arimo ↔ Arial, and Tinos ↔
/// Times New Roman preserve the common Office metrics.
fn substitutes(family: &str) -> &'static [&'static str] {
    // An unknown sans or serif family is set in the platform's own (Arial and Times New Roman on
    // Windows, Helvetica Neue on a Mac), as Word sets it (zenodo-114: Garamond in Times
    // New Roman). Narrow families have a separate condensed open substitute below.
    // Carlito and Caladea, bundled on the desktop
    // since 1.1.3, come after them, so they stand in only where neither is installed (Linux), as
    // before they were bundled.
    const SANS: &[&str] = &["Helvetica Neue", "Arial", "Carlito", "Liberation Sans", "Noto Sans", "DejaVu Sans", "Source Sans 3"];
    const SERIF: &[&str] = &["Times New Roman", "Caladea", "Liberation Serif", "Georgia", "Noto Serif", "DejaVu Serif", "Source Serif 4"];
    // Calibri and Cambria themselves: their metric-compatible stand-ins first, wherever.
    const CALIBRI: &[&str] = &["Carlito", "Helvetica Neue", "Arial", "Liberation Sans", "Noto Sans", "DejaVu Sans", "Source Sans 3"];
    const CAMBRIA: &[&str] = &["Caladea", "Times New Roman", "Liberation Serif", "Georgia", "Noto Serif", "DejaVu Serif", "Source Serif 4"];
    const MONO: &[&str] = &["Cousine", "JetBrains Mono", "Courier New", "Liberation Mono", "Menlo", "Consolas", "DejaVu Sans Mono"];
    // Arial itself first: Helvetica and Arial Nova asked for on Windows, where Arial is installed
    // and they aren't, are set in Arial (Arial was drawn to Helvetica's widths), as Word does;
    // without it they fell through to Source Sans 3, a wider face (corpus govuk-052, zenodo-068).
    const GOTHAM: &[&str] = &["Montserrat", "Arimo", "Liberation Sans", "Source Sans 3"];
    const NARROW: &[&str] = &["Roboto", "Arimo", "Liberation Sans", "Source Sans 3"];
    const ARIAL: &[&str] = &["Arimo", "Arial", "Liberation Sans", "Helvetica", "Helvetica Neue", "Source Sans 3"];
    // Times New Roman first for the same reason: "Times" (a Mac font) has its widths, and on
    // Windows fell through to Source Serif 4 (corpus govuk-006, govuk-020).
    const TIMES: &[&str] = &["Tinos", "Times New Roman", "Liberation Serif", "Times", "Source Serif 4"];
    const SYMBOL: &[&str] = &["Noto Sans Symbols", "Noto Sans Symbols 2", "Source Sans 3", "Apple Symbols", "Segoe UI Symbol", "DejaVu Sans"];
    const CJK_SANS: &[&str] = &["Noto Sans CJK SC", "BIZ UDPGothic", "NanumGothic", "Doc4MeHangul", "Source Sans 3"];
    const CJK_SERIF: &[&str] = &["Shippori Mincho", "BIZ UDMincho", "Noto Sans CJK SC", "Source Serif 4"];
    // Verdana: DejaVu Sans (Bitstream Vera's design) sets English text 1.2% narrower in regular
    // and 0.2% wider in bold, and breaks 60% / 74% of lines at the same word (measured against
    // Microsoft's core-fonts Verdana); Carlito, the old fallback, was 21% narrower and broke none.
    // Inter (bundled, for the web) is the next nearest, 6.8% narrower.
    const VERDANA: &[&str] = &["DejaVu Sans", "Bitstream Vera Sans", "Inter", "Source Sans 3"];
    // Georgia: Gelasio is made metric-compatible with it; the bundled Source Serif 4 sets English
    // text 0.1% wider and breaks 54% of lines at the same word (Caladea: 8.5% narrower, 1%).
    const SOURCE_SANS: &[&str] = &["Source Sans 3", "Arial", "Liberation Sans"];
    const GEORGIA: &[&str] = &["Gelasio", "Source Serif 4", "Caladea", "Liberation Serif", "Noto Serif"];
    // Persian / Arabic document fonts. Naskh (book) faces such as B Nazanin, B Lotus, B Zar and
    // Traditional Arabic, then the sans faces (B Yekan, B Titr, IRANSans) people use for
    // headings and screens. Open fonts with full Persian coverage first.
    const NASKH: &[&str] = &["Noto Naskh Arabic", "XB Zar", "XB Niloofar", "Amiri", "Vazirmatn", "Noto Sans Arabic", "DejaVu Sans", "Source Serif 4"];
    const ARABIC_SANS: &[&str] = &["Vazirmatn", "Sahel", "Shabnam", "Samim", "Noto Sans Arabic", "Noto Naskh Arabic", "DejaVu Sans", "Source Sans 3"];
    // Palatino Linotype / Book Antiqua: macOS ships the same design as "Palatino"; open
    // look-alikes with matching metrics (TeX Gyre Pagella, URW P052 / Palladio) on Linux.
    const PALATINO: &[&str] = &[
        "Caladea",
        "Source Serif 4",
        "Palatino Linotype",
        "Palatino",
        "Book Antiqua",
        "TeX Gyre Pagella",
        "P052",
        "URW Palladio L",
        "Palladio",
        "Times New Roman",
        "Liberation Serif",
        "Georgia",
        "Noto Serif",
        "DejaVu Serif",
        "Source Serif 4",
    ];
    // Thai document fonts (TH Sarabun New / PSK and the other national fonts, the Windows UPC
    // families, Google's Thai families): the bundled Sarabun draws them.
    const THAI: &[&str] = &["Sarabun"];
    // Overpass is available in variable weights in the pinned corpus font input; unknown platform
    // builds fall back to the closest bundled office sans fonts.
    // Aptos, Word's default font since 2023: Intos (bundled) has its advances, kerning and
    // vertical metrics, as Intos Display, Narrow and Serif have Aptos Display's, Narrow's and
    // Serif's (Display is bundled; Narrow and Serif are used when installed).
    const APTOS: &[&str] = &["Intos", "Carlito", "Helvetica Neue", "Arial", "Liberation Sans", "Noto Sans", "DejaVu Sans", "Source Sans 3"];
    const APTOS_DISPLAY: &[&str] = &["Intos Display", "Intos", "Carlito", "Arial", "Liberation Sans", "Source Sans 3"];
    const APTOS_NARROW: &[&str] = &["Intos Narrow", "Intos", "Carlito", "Arial", "Liberation Sans", "Source Sans 3"];
    const APTOS_SERIF: &[&str] =
        &["Intos Serif", "Caladea", "Times New Roman", "Liberation Serif", "Georgia", "Noto Serif", "DejaVu Serif", "Source Serif 4"];
    let f = family.to_ascii_lowercase();
    if is_thai_document_font(&f) {
        return THAI;
    }
    if is_persian_naskh(&f) {
        return NASKH;
    }
    if is_persian_sans(&f) {
        return ARABIC_SANS;
    }
    match f.as_str() {
        "aptos" | "aptos light" | "aptos semibold" | "aptos extrabold" | "aptos black" => APTOS,
        "aptos display" => APTOS_DISPLAY,
        "aptos narrow" => APTOS_NARROW,
        "aptos serif" => APTOS_SERIF,
        "palatino linotype" | "book antiqua" | "palatino" => PALATINO,
        "gotham" | "gotham book" | "gotham medium" | "gotham bold" => GOTHAM,
        "arial narrow" | "arial nova cond light" => NARROW,
        "arial"
        | "arialmt"
        | "arial-italicmt"
        | "helvetica"
        | "helvetica neue"
        | "arial nova"
        | "arial unicode ms"
        | "lucida grande"
        | "lucida sans unicode"
        | "frutiger w01"
        | "frutigerlt-lightitalic"
        | "myriad pro"
        | "univers"
        | "apercu"
        | "gill sans mt"
        | "segoe ui"
        | "tahoma"
        | "shruti"
        | "raavi"
        | "latha"
        | "kartika"
        | "mangal"
        | "anjalinewlipi" => ARIAL,
        "times new roman" | "times" | "tms rmn" | "timesnewromanpsmt" | "garamond" | "bell mt" | "minion pro" | "minion pro med" | "arno pro"
        | "bookman old style" | "sitka banner" | "sitka heading" | "sitka text" => TIMES,
        "courier new" | "courier" | "consolas" | "lucida console" | "cascadia code" | "cascadia mono" => MONO,
        "ms mincho" | "ms pmincho" => CJK_SERIF,
        "stzhongsong" | "simsun" | "batang" | "pmingliu" | "ms gothic" | "ms pgothic" | "malgun gothic semilight" => CJK_SANS,
        "symbol" | "wingdings" | "wingdings 2" | "wingdings 3" | "segoe ui symbol" | "segoe ui emoji" | "webdings" => SYMBOL,
        "verdana" => VERDANA,
        // Source Sans 3 is a fallback for the older family name when its original OFL faces are unavailable.
        "source sans pro" | "source sans pro light" | "source sans pro semibold" => SOURCE_SANS,
        "pt serif" | "eu albertina" | "amasis mt pro medium" | "newsgoth cn bt" | "nexus sans web pro" | "inherit" => CAMBRIA,
        "georgia" => GEORGIA,
        "calibri" | "calibri light" => CALIBRI,
        "cambria" | "cambria math" => CAMBRIA,
        "constantia" | "baskerville old face" | "century" | "century schoolbook" | "sitka" => SERIF,
        _ if f.contains("mono") || (f.contains("code") && !f.contains("unicode")) => MONO,
        _ if f.contains("serif") && !f.contains("sans") => SERIF,
        _ => SANS,
    }
}

/// Open family to use for a requested face when its original is proprietary or unavailable.
/// These aliases cover the proprietary fonts reported in the pagination corpus, plus fonts which
/// otherwise happen to be installed on the Windows measurement runner. Exact metric-compatible
/// open families are preferred; the remaining faces use the closest bundled open design.
fn open_substitute(family: &str) -> Option<&'static str> {
    let mut f = family.trim().trim_start_matches('*').to_ascii_lowercase();
    if f.ends_with("-bold-4304") {
        f.truncate(f.len().saturating_sub("-bold-4304".len()));
    }
    match f.as_str() {
        "arial"
        | "arialmt"
        | "arial-italicmt"
        | "arial nova"
        | "arial unicode ms"
        | "helvetica"
        | "helvetica neue"
        | "lucida grande"
        | "lucida sans unicode"
        | "frutiger w01"
        | "frutigerlt-lightitalic"
        | "myriad pro"
        | "univers"
        | "apercu"
        | "gill sans mt"
        | "tahoma"
        | "segoe ui"
        | "nexus sans web pro" => Some("Arimo"),
        "gotham" | "gotham book" | "gotham medium" | "gotham bold" => Some("Montserrat"),
        "arial narrow" | "arial nova cond light" | "roboto" => Some("Roboto"),
        "overpass" => Some("Overpass"),
        "overpass light" | "overpass extralight" | "overpass semibold" | "overpass extrabold" => Some("Overpass"),
        "calibri" | "calibri light" | "calibri semibold" | "calibri bold" => Some("Carlito"),
        "cambria" | "cambria math" | "eu albertina" | "amasis mt pro medium" | "newsgoth cn bt" | "inherit" => Some("Caladea"),
        "aptos" | "aptos light" | "aptos semibold" | "aptos extrabold" | "aptos black" => Some("Intos"),
        "aptos display" => Some("Intos Display"),
        "aptos narrow" => Some("Intos"),
        "times new roman"
        | "times"
        | "tms rmn"
        | "timesnewromanpsmt"
        | "garamond"
        | "bell mt"
        | "frutigerlt lightitalic"
        | "minion pro"
        | "minion pro med"
        | "arno pro"
        | "bookman old style"
        | "sitka banner"
        | "sitka heading"
        | "sitka text" => Some("Tinos"),
        "pt serif" => Some("Source Serif 4"),
        "verdana" => Some("Inter"),
        "courier new" | "courier" => Some("Cousine"),
        "wingdings" | "wingdings 2" | "wingdings 3" | "symbol" | "webdings" | "segoe ui symbol" | "segoe ui emoji" => Some("Noto Sans Symbols"),
        "ms mincho" | "ms pmincho" => Some("Shippori Mincho"),
        "stzhongsong" | "simsun" | "pmingliu" => Some("Noto Sans CJK SC"),
        "ms gothic" | "ms pgothic" => Some("BIZ UDPGothic"),
        "batang" => Some("NanumGothic"),
        "malgun gothic semilight" => Some("NanumGothic"),
        "shruti" => Some("Noto Sans Gujarati"),
        "raavi" => Some("Noto Sans Gurmukhi"),
        "latha" => Some("Noto Sans Tamil"),
        "kartika" => Some("Noto Sans Malayalam"),
        "mangal" | "anjalinewlipi" => Some("Noto Sans Devanagari"),
        "nyala" => Some("Noto Sans Ethiopic"),
        "source sans pro" => Some("Source Sans Pro"),
        "source sans pro light" | "source sans pro semibold" => Some("Source Sans 3"),
        "source serif 4" => Some("Source Serif 4"),
        _ => None,
    }
}

/// Thai document fonts that the bundled Sarabun stands in for when they aren't installed. `f` is
/// lowercase. The Thai national fonts are named "TH …" (TH Sarabun New, TH SarabunPSK,
/// TH Niramit AS …); the Windows Thai families end in "UPC" or "New".
fn is_thai_document_font(f: &str) -> bool {
    const NAMES: &[&str] = &[
        "sarabun",
        "sarabun new",
        "thsarabunnew",
        "thsarabunpsk",
        "angsana new",
        "cordia new",
        "browallia new",
        "leelawadee",
        "leelawadee ui",
        "thonburi",
        "ayuthaya",
        "silom",
        "sathu",
        "krungthep",
        "kanit",
        "prompt",
        "mitr",
        "taviraj",
        "trirong",
        "pridi",
        "chonburi",
        "athiti",
        "maitree",
        "niramit",
        "krub",
        "k2d",
        "kodchasan",
        "bai jamjuree",
        "chakra petch",
        "charmonman",
        "fahkwang",
        "srisakdi",
        "thasadith",
    ];
    // A family's static weights often have names of their own ("Sarabun Light", "Kanit SemiBold
    // Italic"): match the family they belong to.
    const WEIGHTS: &[&str] = &[
        " thin",
        " extralight",
        " extra light",
        " light",
        " regular",
        " medium",
        " semibold",
        " semi bold",
        " bold",
        " extrabold",
        " extra bold",
        " black",
    ];
    let mut base = f.strip_suffix(" italic").unwrap_or(f);
    if let Some(b) = WEIGHTS.iter().find_map(|w| base.strip_suffix(w)) {
        base = b;
    }
    NAMES.contains(&f) || NAMES.contains(&base) || f.starts_with("th ") || f.ends_with("upc") || f.contains("thai")
}

/// Naskh-style Persian/Arabic document fonts (the "B …" family names come from the common
/// Persian font packs; matched by name only).
fn is_persian_naskh(f: &str) -> bool {
    const NAMES: &[&str] = &[
        "b nazanin",
        "b lotus",
        "b zar",
        "b mitra",
        "b badr",
        "b yagut",
        "b roya",
        "b compset",
        "b nazanin bold",
        "b lotus bold",
        "nazanin",
        "lotus",
        "zar",
        "mitra",
        "badr",
        "yagut",
        "roya",
        "ir nazanin",
        "ir lotus",
        "ir zar",
        "ir mitra",
        "ir badr",
        "irnazanin",
        "irlotus",
        "irzar",
        "irmitra",
        "irbadr",
        "traditional arabic",
        "simplified arabic",
        "arabic typesetting",
        "sakkal majalla",
        "adobe arabic",
        "andalus",
        "times new roman (arabic)",
    ];
    NAMES.contains(&f)
}

/// Sans Persian/Arabic fonts for headings and screens.
fn is_persian_sans(f: &str) -> bool {
    const NAMES: &[&str] = &[
        "b yekan",
        "b titr",
        "b homa",
        "b koodak",
        "b traffic",
        "b jadid",
        "b elham",
        "b kamran",
        "b davat",
        "b ferdosi",
        "yekan",
        "titr",
        "homa",
        "koodak",
        "traffic",
        "ir titr",
        "ir homa",
        "irtitr",
        "irhoma",
        "iransans",
        "iransansweb",
        "iran sans",
        "iranyekan",
        "iran yekan",
        "iranyekanweb",
        "dana",
        "estedad",
        "dubai",
        "aldhabi",
        "urdu typesetting",
        "arabic transparent",
    ];
    NAMES.contains(&f) || f.starts_with("iransans") || f.starts_with("iranyekan")
}

fn style_name(bold: bool, italic: bool) -> &'static str {
    match (bold, italic) {
        (false, false) => "Regular",
        (true, false) => "Bold",
        (false, true) => "Italic",
        (true, true) => "Bold Italic",
    }
}

fn style_name_for(family: &str, bold: bool, italic: bool) -> String {
    if family.eq_ignore_ascii_case("Overpass") {
        if italic {
            return if bold { "Bold Italic" } else { "Italic" }.to_string();
        }
        if bold {
            return "Bold".to_string();
        }
        return "Regular".to_string();
    }
    style_name(bold, italic).to_string()
}

fn overpass_weight(family: &str) -> Option<f32> {
    match family.trim().to_ascii_lowercase().as_str() {
        "overpass extralight" => Some(200.0),
        "overpass light" => Some(300.0),
        "overpass semibold" => Some(600.0),
        "overpass extrabold" => Some(800.0),
        _ => None,
    }
}

/// A face name used as a font name, as documents do ("Arial Bold", "Times New Roman Bold
/// Italic"): the family and the style it names, when `family` isn't itself an installed family.
/// Windows finds such faces by their full names, so Word draws "Arial Bold" with Arial's bold;
/// families whose names end in other words ("Calibri Light", "Arial Black") stay as they are.
pub fn face_name(family: &str) -> Option<(&str, bool, bool)> {
    if family.eq_ignore_ascii_case("*Tahoma-Bold-4304") {
        return Some(("Tahoma", true, false));
    }
    const OVERPASS_STYLES: &[(&str, bool, bool); 4] =
        &[(" extralight", false, false), (" light", false, false), (" semibold", true, false), (" extrabold", true, false)];
    let lower = family.to_ascii_lowercase();
    if lower.starts_with("overpass")
        && let Some((suffix, bold, italic)) = OVERPASS_STYLES.iter().find(|(suffix, ..)| lower.ends_with(suffix))
    {
        let base = family.get(..family.len() - suffix.len())?.trim_end();
        return Some((base, *bold, *italic));
    }
    const STYLES: [(&str, bool, bool); 7] = [
        (" bold italic", true, true),
        (" bold oblique", true, true),
        (" bolditalic", true, true),
        (" italic", false, true),
        (" oblique", false, true),
        (" bold", true, false),
        (" regular", false, false),
    ];
    let (suffix, bold, italic) = STYLES.iter().find(|(s, ..)| lower.ends_with(s))?;
    let base = family.get(..family.len() - suffix.len())?.trim_end();
    if base.is_empty() || FontDb::global().has_family(family) {
        return None;
    }
    Some((base, *bold, *italic))
}

/// The family `family` is drawn with when it isn't installed (Word's Font Substitution), or
/// `None` when it is.
pub fn substitute_for(family: &str) -> Option<String> {
    if let Some((base, ..)) = face_name(family) {
        return substitute_for(base);
    }
    let db = FontDb::global();
    if let Some(open) = open_substitute(family).filter(|name| db.has_family(name))
        && !open.eq_ignore_ascii_case(family)
    {
        return Some(open.to_string());
    }
    if db.has_family(family) {
        return None;
    }
    Some(substitutes(family).iter().find(|s| db.has_family(s)).copied().unwrap_or(crate::FALLBACK_FAMILY).to_string())
}

type Memo = (u64, HashMap<(String, bool, bool), Resolved>);

fn resolve_cache() -> &'static Mutex<Memo> {
    static CACHE: OnceLock<Mutex<Memo>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new((0, HashMap::new())))
}

const FAMILY_CACHE_MAX: usize = 4096;
/// Family names longer than this (bytes) are resolved but never remembered.
const FAMILY_KEY_MAX: usize = 256;

/// Resolve `family` with bold/italic to a face (memoised until user fonts are added or removed,
/// [`FontDb::generation`]).
pub fn resolve(family: &str, bold: bool, italic: bool) -> Resolved {
    let cache = resolve_cache();
    let db = FontDb::global();
    let generation = db.generation();
    let cacheable = family.len() <= FAMILY_KEY_MAX;
    let key = (if cacheable { family.to_string() } else { String::new() }, bold, italic);
    {
        let mut c = cache.lock().unwrap_or_else(|e| e.into_inner());
        if c.0 != generation {
            *c = (generation, HashMap::new());
        }
        if cacheable && let Some(r) = c.1.get(&key) {
            return *r;
        }
    }
    if let Some((base, b, i)) = face_name(family) {
        let r = if let Some(weight) = overpass_weight(family) {
            let base = resolve(base, false, false);
            let style = if i || italic { format!("Italic{{wght:{weight}}}") } else { format!("Regular{{wght:{weight}}}") };
            let face = db.face(&base.face.family, &style);
            Resolved { face: FaceRef::of(&face), substituted: base.substituted, synth_bold: false, synth_italic: false }
        } else {
            resolve(base, bold || b, italic || i)
        };
        let mut c = cache.lock().unwrap_or_else(|e| e.into_inner());
        if c.0 == generation {
            c.1.insert(key, r);
        }
        return r;
    }
    let (fam, substituted) = if let Some(open) = open_substitute(family).filter(|name| db.has_family(name)) {
        (open.to_string(), !open.eq_ignore_ascii_case(family))
    } else if db.has_family(family) {
        (family.to_string(), false)
    } else {
        let s = substitutes(family).iter().find(|s| db.has_family(s)).copied().unwrap_or(crate::FALLBACK_FAMILY);
        (s.to_string(), true)
    };
    if let Some(weight) = overpass_weight(family) {
        let style = if italic { format!("Italic{{wght:{weight}}}") } else { format!("Regular{{wght:{weight}}}") };
        let face = db.face(&fam, &style);
        let r = Resolved { synth_bold: false, synth_italic: italic && !face.italic, substituted, face: FaceRef::of(&face) };
        let mut c = cache.lock().unwrap_or_else(|e| e.into_inner());
        if cacheable && c.0 == generation {
            c.1.insert(key, r);
        }
        return r;
    }
    let style = if fam == "Montserrat" && family.to_ascii_lowercase().starts_with("gotham") {
        let weight = if bold {
            700
        } else if family.eq_ignore_ascii_case("Gotham Medium") {
            500
        } else {
            400
        };
        format!("{}{{wght:{weight}}}", style_name(bold, italic))
    } else if fam == "Roboto" && (family.eq_ignore_ascii_case("Arial Narrow") || family.eq_ignore_ascii_case("Arial Nova Cond Light")) {
        let weight = if bold {
            700
        } else if family.eq_ignore_ascii_case("Arial Nova Cond Light") {
            300
        } else {
            400
        };
        format!("{}{{wdth:75,wght:{weight}}}", style_name(bold, italic))
    } else {
        style_name_for(&fam, bold, italic)
    };
    let face = db.face(&fam, &style);
    let r = Resolved { synth_bold: bold && face.weight < 600.0, synth_italic: italic && !face.italic, substituted, face: FaceRef::of(&face) };
    let mut c = cache.lock().unwrap_or_else(|e| e.into_inner());
    if cacheable && c.0 == generation {
        if c.1.len() >= FAMILY_CACHE_MAX {
            c.1.clear();
        }
        c.1.insert(key, r);
    }
    r
}

/// The standard Unicode character for a Symbol or Wingdings character stored the way Word stores
/// it, as the private-use code U+F000 + the font's own code (U+F0B7 is the Symbol bullet). Used
/// when that font isn't installed: a substitute draws something else, or nothing, at the
/// private-use code. `None` for other fonts and for codes not in the table.
pub fn symbol_font_char(family: &str, c: char) -> Option<char> {
    let code = u32::from(c).checked_sub(0xF000)?;
    let u = match (family.to_ascii_lowercase().as_str(), code) {
        // Adobe Symbol encoding.
        ("symbol", 0xA7) => '\u{2663}',
        ("symbol", 0xA8) => '\u{2666}',
        ("symbol", 0xA9) => '\u{2665}',
        ("symbol", 0xAA) => '\u{2660}',
        ("symbol", 0xAE) => '\u{2192}',
        ("symbol", 0xB7) => '\u{2022}',
        ("symbol", 0xD7) => '\u{22C5}',
        ("symbol", 0xE0) => '\u{25CA}',
        // Wingdings characters common as list bullets.
        ("wingdings", 0x6C) => '\u{25CF}',
        ("wingdings", 0x6E) => '\u{25A0}',
        ("wingdings", 0x76) => '\u{2756}',
        ("wingdings", 0xA7) => '\u{25AA}',
        ("wingdings", 0xD8) => '\u{27A2}',
        ("wingdings", 0xFC) => '\u{2714}',
        _ => return None,
    };
    Some(u)
}

/// A line's (ascent, descent) from hhea values, both positive: the line gap goes above the ascent.
fn hhea_line(ascender: i16, descender: i16, line_gap: i16) -> (f64, f64) {
    (f64::from(ascender) + f64::from(line_gap.max(0)), f64::from(descender).abs())
}

/// A line's (ascent, descent) the way Word for Windows measures it (GDI text metrics): OS/2
/// usWinAscent and usWinDescent, plus the external leading, the part of the hhea line gap that
/// the win metrics don't already cover, above the ascent. Times New Roman (win 1825/443, hhea
/// 1825/-443, gap 87): 1912 + 443, like 13.8 pt at 12 pt. Sarabun (win 1286/567, hhea 1068/-232,
/// gap 0): 1286 + 567, 1.853 em, where Word for Mac's hhea gives 1.30 em.
fn win_line(win_ascent: u16, win_descent: u16, ascender: i16, descender: i16, line_gap: i16) -> (f64, f64) {
    let win = f64::from(win_ascent) + f64::from(win_descent);
    let hhea = f64::from(ascender) - f64::from(descender);
    let external = (f64::from(line_gap) - (win - hhea)).max(0.0);
    (f64::from(win_ascent) + external, f64::from(win_descent))
}

/// A line's (ascent, descent) in font units from a face's OS/2 win metrics and hhea values:
/// [`win_line`] when the win metrics are set and sane (not zero, at most four ems), else
/// [`hhea_line`] (at most two ems), else `None`. With `prefer_hhea` the hhea line comes first.
fn pick_line(win: Option<(u16, u16)>, hhea: Option<(i16, i16, i16)>, upem: f64, prefer_hhea: bool) -> Option<(f64, f64)> {
    let ok = |m: &(f64, f64), max_ems: f64| m.0 > 0.0 && m.0 + m.1 <= upem * max_ems;
    let (asc, desc, gap) = hhea.unwrap_or((0, 0, 0));
    let win = win.filter(|(a, d)| *a > 0 && u32::from(*a) + u32::from(*d) > 0).map(|(a, d)| win_line(a, d, asc, desc, gap)).filter(|m| ok(m, 4.0));
    let hhea = hhea.map(|(a, d, g)| hhea_line(a, d, g)).filter(|m| ok(m, 2.0));
    if prefer_hhea { hhea.or(win) } else { win.or(hhea) }
}

/// Line metrics for a face in font units: (ascent, descent), both positive, the way Word for
/// Windows measures single line spacing ([`win_line`]), so a page holds the same lines here, in
/// Word on Windows and in print on every platform. hhea values (the line gap above the ascent)
/// when the face has no OS/2 table or its win metrics are zero or absurd (over four ems).
pub fn line_metrics(face: &crate::FontFace) -> (f64, f64) {
    face_line(face, false)
}

/// The cached line of `face`: [`pick_line`] over its tables, or its ascent and descent.
fn face_line(face: &crate::FontFace, prefer_hhea: bool) -> (f64, f64) {
    type Cache = Mutex<HashMap<(u32, bool), (f64, f64)>>;
    static CACHE: OnceLock<Cache> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(v) = cache.lock().unwrap_or_else(|e| e.into_inner()).get(&(face.id(), prefer_hhea)) {
        return *v;
    }
    let mut v = (face.ascent.max(1.0), face.descent.abs());
    if let Some(f) = face.skrifa() {
        let hhea = f.hhea().ok().map(|h| (h.ascender().to_i16(), h.descender().to_i16(), h.line_gap().to_i16()));
        let win = f.os2().ok().map(|o| (o.us_win_ascent(), o.us_win_descent()));
        if let Some(m) = pick_line(win, hhea, face.upem, prefer_hhea) {
            v = m;
        }
    }
    cache.lock().unwrap_or_else(|e| e.into_inner()).insert((face.id(), prefer_hhea), v);
    v
}

/// Word's single line of fonts whose open substitutes, or whose own win metrics, are spaced
/// differently, per em (ascent, descent): Cambria (drawn with Caladea, 1.30 em), Courier New (drawn
/// with JetBrains Mono where only the bundled fonts exist, 1.565 em), Verdana and Georgia (drawn
/// with DejaVu Sans and Source Serif 4) and Aptos, whose lines in Word's PDFs step 1.2207 em (its
/// hhea and typo metrics, 1923 + 577), not its win metrics' 1.2847.
fn requested_line(family: &str) -> Option<(f64, f64)> {
    match family.trim().to_ascii_lowercase().as_str() {
        "cambria" | "cambria math" => Some((1946.0 / 2048.0, 455.0 / 2048.0)),
        "courier new" | "courier" => Some((1705.0 / 2048.0, 615.0 / 2048.0)),
        // OS/2 Win metrics observed in the original fonts embedded by Word in govuk-068
        // (Nyala) and zenodo-095 (Gotham Book/Medium); no outlines or font programs copied.
        "nyala" => Some((1660.0 / 2048.0, 480.0 / 2048.0)),
        "gotham book" | "gotham medium" => Some((0.96, 0.24)),
        // Verdana (win 2059 / 430: 1.215 em; DejaVu Sans is 1.164) and Georgia (win 1878 / 449:
        // 1.136 em; Source Serif 4 is 1.371), every style alike.
        "verdana" => Some((2059.0 / 2048.0, 430.0 / 2048.0)),
        "georgia" => Some((1878.0 / 2048.0, 449.0 / 2048.0)),
        // Garamond, as Word's PDFs of zenodo-115 and zenodo-118 declare it (ascent 862, descent
        // 263 per 1000) and step it: 11 pt body lines at 283/240 spacing 14.59 pt apart on
        // average, 1.125 em of line. Tinos' own hhea line is 1.150 em.
        "garamond" => Some((0.862, 0.263)),
        // Fonts Word's lists and check boxes draw with, as embedded in Word's own PDFs (OS/2 win
        // ascent / descent; their hhea values are the same and have no line gap). Symbol's bullet
        // is taller than Calibri's or Arial's ascent, so a bulleted line is taller in Word: 20.4 pt
        // against 19.65 for 14 pt Calibri at 1.15 lines (zenodo-052).
        "symbol" => Some((2059.0 / 2048.0, 450.0 / 2048.0)),
        "wingdings" => Some((1841.0 / 2048.0, 432.0 / 2048.0)),
        "wingdings 2" => Some((1727.0 / 2048.0, 432.0 / 2048.0)),
        "segoe ui" | "segoe ui symbol" | "segoe ui emoji" | "segoe ui semibold" | "segoe ui light" | "segoe ui semilight" | "segoe ui black" => {
            Some((2210.0 / 2048.0, 514.0 / 2048.0))
        }
        "ms gothic" | "ms pgothic" => Some((220.0 / 256.0, 36.0 / 256.0)),
        // Every Aptos family (Display, Narrow, Serif, Mono, each weight): hhea 1923 / 577.
        f if is_aptos(f) => Some((1923.0 / 2048.0, 577.0 / 2048.0)),
        f if is_overpass(f) => Some((0.982, 0.284)),
        _ => None,
    }
}

/// Overpass and its weights. Word's PDFs embed the original static Overpass, whose line is
/// 1.266 em: zenodo-067 steps its 10.5 pt single-spaced Overpass paragraphs 13.294 pt on average
/// (Word places them 13.32 or 13.20 pt apart), with the 0.982 em ascent its embedded font
/// descriptor declares. The variable Overpass in the pinned open font input has win metrics of
/// (2316 + 756) / 2000 = 1.536 em, so its own line would be 21% taller.
fn is_overpass(family: &str) -> bool {
    let f = family.trim().to_ascii_lowercase();
    f == "overpass" || overpass_weight(&f).is_some()
}

/// Cambria Math, whose text lines Word steps like Cambria's (1946 + 455 per 2048 em), not by
/// its own much taller math metrics: zenodo-082's Word PDF embeds Cambria Math and steps its
/// 10.5 pt equation paragraphs (1.15 lines, 4 pt apart) 18.12 pt, 14.15 pt of line.
fn is_cambria_math(family: &str) -> bool {
    family.trim().eq_ignore_ascii_case("cambria math")
}

/// The single line, in points at `size`, of the font a document asks for when a fallback face
/// draws characters that font has but its stand-in lacks. Word draws them in the requested font,
/// so the line keeps that font's spacing instead of the fallback face's. Only these:
/// - Cambria and Cambria Math: Caladea has no Greek or Cyrillic and only some math symbols, and
///   Word's PDFs (zenodo-082) step their lines by Cambria's line whatever the letters.
/// - Symbol, Wingdings and Wingdings 2: their private-use bullets are drawn as standard
///   characters by whatever face has them, but Word draws them in the symbol font itself. Word
///   steps zenodo-156's Wingdings-bulleted 11 pt Calibri lines 13.44 pt, Calibri's line, where
///   Source Sans 3's line for the ▪ made them 14.59 pt.
///
/// Other families keep the fallback face's own line, because which letters their originals hold
/// isn't established here.
pub fn covered_fallback_line(family: &str, size: f64) -> Option<(f64, f64)> {
    let family = face_name(family).map_or(family, |(base, ..)| base);
    let f = family.trim().to_ascii_lowercase();
    if !matches!(f.as_str(), "cambria" | "cambria math" | "symbol" | "wingdings" | "wingdings 2") {
        return None;
    }
    requested_line(family).map(|(a, d)| (a * size, d * size))
}

/// Aptos, Aptos Display, Narrow, Serif, Mono and each weight.
fn is_aptos(family: &str) -> bool {
    let f = family.trim().to_ascii_lowercase();
    f == "aptos" || f.starts_with("aptos ")
}

/// Single line spacing for text a document sets in `family`, drawn with `face` ([`resolve`]),
/// at `size` points: (ascent, descent) in points. The face's own Word for Windows line
/// ([`line_metrics`]) when it is the font asked for, except Aptos ([`requested_line`]). A
/// substitute keeps the line of the font it stands in for where that is known
/// ([`requested_line`]), and otherwise its hhea line, which
/// metric-compatible substitutes share with their originals and which keeps a Thai font drawn
/// with Sarabun (Angsana New, Cordia New…) near its own small-letter spacing (1.30 em, not
/// Sarabun's 1.853 em).
pub fn single_line(family: &str, face: &crate::FontFace, substituted: bool, size: f64) -> (f64, f64) {
    // "Arial Bold" spaces lines as Arial does.
    let family = face_name(family).map_or(family, |(base, ..)| base);
    // Aptos's, Cambria Math's and the open variable Overpass's own win metrics aren't what Word
    // spaces them by, so their lines hold when they are installed.
    if let Some((a, d)) = requested_line(family).filter(|_| substituted || is_aptos(family) || is_cambria_math(family) || is_overpass(family)) {
        return (a * size, d * size);
    }
    if !substituted {
        let (a, d) = line_metrics(face);
        let k = size / face.upem.max(1.0);
        return (a * k, d * k);
    }
    let (a, d) = face_line(face, true);
    let k = size / face.upem.max(1.0);
    (a * k, d * k)
}

/// Measured adjustments for an open substitute. Selected Latin and Ethiopic fits use
/// PDF-declared advances and the line returned by `single_line`, with original line metrics
/// where known. They retain the open face's glyph height; their kerning is not established.
/// TH Sarabun New/PSK also use measured glyph scales, kerning and line metrics when drawn with
/// Sarabun, whose letters otherwise appear about 1.47 times as tall at the same point size.
#[derive(Clone, Copy)]
pub struct MetricFit {
    style: Option<&'static crate::thsarabun::ThSarabunStyle>,
    advances: &'static [(char, u16)],
    line: (f64, f64),
}

impl std::fmt::Debug for MetricFit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricFit").field("line", &self.line).finish()
    }
}

impl PartialEq for MetricFit {
    fn eq(&self, other: &Self) -> bool {
        let same_style = match (self.style, other.style) {
            (Some(a), Some(b)) => std::ptr::eq(a, b),
            (None, None) => true,
            _ => false,
        };
        same_style && std::ptr::eq(self.advances, other.advances) && self.line == other.line
    }
}

impl MetricFit {
    /// Glyph scale for the substitute; non-Thai fits retain the selected open face's height.
    pub fn scale(&self, thai: bool) -> f32 {
        self.style.map_or(1.0, |s| if thai { s.thai_scale } else { s.latin_scale })
    }

    /// Original advance in ems, where measured; unmeasured characters retain fallback advances.
    pub fn advance(&self, c: char) -> Option<f32> {
        self.advances.binary_search_by_key(&c, |e| e.0).ok().and_then(|i| self.advances.get(i)).map(|e| f32::from(e.1) / 1000.0)
    }

    /// Measured kerning in ems; PDF advance tables do not establish kerning pairs.
    pub fn kern(&self, left: char, right: char) -> f32 {
        if self.style.is_none() {
            return 0.0;
        }
        let k = crate::thsarabun::KERN;
        k.binary_search_by(|e| (e.0, e.1).cmp(&(left, right))).ok().and_then(|i| k.get(i)).map_or(0.0, |e| f32::from(e.2) / 1000.0)
    }

    /// Normalized line metrics, from `single_line` or the complete Thai fit.
    pub fn line(&self) -> (f64, f64) {
        self.line
    }

    /// The substitute should be shaped with its tabular figures (`tnum`): Cambria's digits all
    /// take 554/1000 em (592 bold, 528 italic, 573 bold italic), and Caladea's tabular figures are
    /// exactly that wide, where its default proportional ones (362-517) would sit in gaps.
    pub fn tabular_digits(&self) -> bool {
        [
            crate::latin_metrics::CAMBRIA_REGULAR,
            crate::latin_metrics::CAMBRIA_ITALIC,
            crate::latin_metrics::CAMBRIA_BOLD,
            crate::latin_metrics::CAMBRIA_BOLD_ITALIC,
        ]
        .iter()
        .any(|t| std::ptr::eq(*t, self.advances))
    }
}

/// Measured advances for condensed Arial families drawn with Roboto, Gotham Book/Medium with
/// Montserrat, Nyala with Noto Sans Ethiopic, Garamond with Tinos or Times New Roman, Cambria
/// with Caladea and Calibri Light with Carlito; active only with the expected substitute.
/// TH Sarabun New/PSK additionally use their full scale, kerning and line fit with Sarabun.
/// Other requested families keep the selected face's metrics.
pub fn metric_fit(family: &str, bold: bool, italic: bool) -> Option<MetricFit> {
    let (family, bold) = if family.eq_ignore_ascii_case("*Tahoma-Bold-4304") { ("Tahoma", true) } else { (family, bold) };
    let measured = if family.eq_ignore_ascii_case("Arial Nova Cond Light") && !bold {
        Some(if italic { crate::latin_metrics::NOVA_COND_LIGHT_ITALIC } else { crate::latin_metrics::NOVA_COND_LIGHT_REGULAR })
    } else if family.eq_ignore_ascii_case("Arial Narrow") && !italic {
        Some(if bold { crate::latin_metrics::ARIAL_NARROW_BOLD } else { crate::latin_metrics::ARIAL_NARROW_REGULAR })
    } else if family.eq_ignore_ascii_case("Gotham Book") && !bold && !italic {
        Some(crate::latin_metrics::GOTHAM_BOOK)
    } else if family.eq_ignore_ascii_case("Gotham Medium") && !bold && !italic {
        Some(crate::latin_metrics::GOTHAM_MEDIUM)
    } else if family.eq_ignore_ascii_case("Nyala") {
        Some(crate::latin_metrics::NYALA)
    } else if family.eq_ignore_ascii_case("Garamond") && !(bold && italic) {
        Some(match (bold, italic) {
            (true, _) => crate::latin_metrics::GARAMOND_BOLD,
            (_, true) => crate::latin_metrics::GARAMOND_ITALIC,
            _ => crate::latin_metrics::GARAMOND_REGULAR,
        })
    } else if family.eq_ignore_ascii_case("Calibri Light") {
        // Calibri Light has no bold: Word emboldens the light face and keeps its widths
        // (zenodo-049's bold italic runs are set with Calibri-LightItalic in Word's PDF).
        Some(if italic { crate::latin_metrics::CALIBRI_LIGHT_ITALIC } else { crate::latin_metrics::CALIBRI_LIGHT_REGULAR })
    } else if family.eq_ignore_ascii_case("Cambria") {
        Some(match (bold, italic) {
            (true, true) => crate::latin_metrics::CAMBRIA_BOLD_ITALIC,
            (true, false) => crate::latin_metrics::CAMBRIA_BOLD,
            (false, true) => crate::latin_metrics::CAMBRIA_ITALIC,
            (false, false) => crate::latin_metrics::CAMBRIA_REGULAR,
        })
    } else {
        None
    };
    if let Some(advances) = measured {
        let r = resolve(family, bold, italic);
        let expected: &[&str] = if family.eq_ignore_ascii_case("Nyala") {
            &["Noto Sans Ethiopic"]
        } else if family.eq_ignore_ascii_case("Gotham Book") || family.eq_ignore_ascii_case("Gotham Medium") {
            &["Montserrat"]
        } else if family.eq_ignore_ascii_case("Cambria") {
            // Only its open stand-in: Windows draws Cambria itself.
            &["Caladea"]
        } else if family.eq_ignore_ascii_case("Calibri Light") {
            &["Carlito"]
        } else if family.eq_ignore_ascii_case("Garamond") {
            // Its serif stand-ins: Tinos where only open fonts exist, Times New Roman on Windows.
            &["Tinos", "Times New Roman"]
        } else {
            &["Roboto"]
        };
        if r.substituted && expected.contains(&r.face.family.as_str()) {
            return Some(MetricFit { style: None, advances, line: single_line(family, r.face.get(), true, 1.0) });
        }
    }
    // Layout asks for every shaped piece: rule most families out without allocating.
    if !family.trim_start().get(..2).is_some_and(|p| p.eq_ignore_ascii_case("th")) {
        return None;
    }
    let key: String = family.chars().filter(|c| !c.is_whitespace()).flat_map(char::to_lowercase).collect();
    if !matches!(key.as_str(), "thsarabunnew" | "thsarabunpsk") {
        return None;
    }
    let r = resolve(family, bold, italic);
    if !r.substituted || r.face.family != "Sarabun" {
        return None;
    }
    let style = match (bold, italic) {
        (false, false) => &crate::thsarabun::REGULAR,
        (true, false) => &crate::thsarabun::BOLD,
        (false, true) => &crate::thsarabun::ITALIC,
        (true, true) => &crate::thsarabun::BOLD_ITALIC,
    };
    let (a, d) = crate::thsarabun::LINE;
    Some(MetricFit { style: Some(style), advances: style.advances, line: (a / 1000.0, d / 1000.0) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condensed_latin_widths_are_measured_and_do_not_change_fallback_lines() {
        for advances in [
            crate::latin_metrics::NOVA_COND_LIGHT_REGULAR,
            crate::latin_metrics::NOVA_COND_LIGHT_ITALIC,
            crate::latin_metrics::ARIAL_NARROW_REGULAR,
            crate::latin_metrics::ARIAL_NARROW_BOLD,
            crate::latin_metrics::GOTHAM_BOOK,
            crate::latin_metrics::GOTHAM_MEDIUM,
            crate::latin_metrics::CAMBRIA_REGULAR,
            crate::latin_metrics::CAMBRIA_ITALIC,
            crate::latin_metrics::CAMBRIA_BOLD,
            crate::latin_metrics::CAMBRIA_BOLD_ITALIC,
            crate::latin_metrics::CALIBRI_LIGHT_REGULAR,
            crate::latin_metrics::CALIBRI_LIGHT_ITALIC,
        ] {
            assert!(advances.windows(2).all(|p| p[0].0 < p[1].0));
            let fit = MetricFit { style: None, advances, line: (1.0, 0.25) };
            assert!(fit.advance(' ').is_some_and(|w| w > 0.0));
            assert_eq!(fit.advance('漢'), None);
            assert_eq!(fit.scale(false), 1.0);
            assert_eq!(fit.line(), (1.0, 0.25));
            assert_eq!(fit.kern('A', 'V'), 0.0);
        }
        assert!(metric_fit("Arial Nova Cond Light", true, false).is_none());
        assert!(metric_fit("Arial", false, false).is_none());
    }

    #[test]
    fn narrow_substitutes_use_condensed_open_outlines_with_measured_advances() {
        let db = FontDb::global();
        if std::env::var_os("CRAFT_FONTS_REQUIRED").is_some() {
            assert!(db.has_family("Roboto"), "prepared open-font input is required");
        }
        if !db.has_family("Roboto") {
            return;
        }
        for (family, bold, italic) in [
            ("Arial Nova Cond Light", false, false),
            ("Arial Nova Cond Light", false, true),
            ("Arial Narrow", false, false),
            ("Arial Narrow", true, false),
        ] {
            let r = resolve(family, bold, italic);
            assert_eq!(r.face.family, "Roboto");
            assert!(r.face.style.contains("wdth:75"), "{}", r.face.style);
            let fit = metric_fit(family, bold, italic).unwrap();
            assert!(fit.advance('e').is_some());
            assert_eq!(fit.line(), single_line(family, r.face.get(), true, 1.0));
        }
    }

    #[test]
    fn legacy_tahoma_bold_alias_preserves_weight_and_metric_style() {
        assert_eq!(face_name("*Tahoma-Bold-4304"), Some(("Tahoma", true, false)));
        let a = resolve("*Tahoma-Bold-4304", false, false);
        let b = resolve("Tahoma", true, false);
        assert_eq!(a.face.style, b.face.style);
        assert_eq!(metric_fit("*Tahoma-Bold-4304", false, false), metric_fit("Tahoma", true, false));
    }

    #[test]
    fn gotham_substitutes_preserve_book_medium_and_bold_weights() {
        let db = FontDb::global();
        if std::env::var_os("CRAFT_FONTS_REQUIRED").is_some() {
            assert!(db.has_family("Montserrat"));
        }
        if !db.has_family("Montserrat") {
            return;
        }
        for (family, weight, fitted) in [("Gotham Book", 400.0, true), ("Gotham Medium", 500.0, true), ("Gotham Bold", 700.0, false)] {
            let r = resolve(family, false, false);
            assert_eq!(r.face.family, "Montserrat");
            assert_eq!(r.face.weight, weight);
            assert!(r.substituted);
            assert_eq!(metric_fit(family, false, false).is_some(), fitted);
        }
    }

    #[test]
    fn overpass_bold_means_weight_700_instead_of_extrabold_800() {
        assert_eq!(style_name_for("Overpass", true, false), "Bold");
        if FontDb::global().has_family("Overpass") {
            assert_eq!(resolve("Overpass", true, false).face.weight, 700.0);
        }
    }

    #[test]
    fn nyala_fitting_uses_word_metrics_instead_of_the_taller_open_fallback_line() {
        assert_eq!(requested_line("Nyala"), Some((1660.0 / 2048.0, 480.0 / 2048.0)));
        if FontDb::global().has_family("Noto Sans Ethiopic") {
            let fit = metric_fit("Nyala", false, false).unwrap();
            assert!(fit.advance('ሀ').is_some());
            assert_eq!(fit.line(), (1660.0 / 2048.0, 480.0 / 2048.0));
        }
    }

    #[test]
    fn the_line_gap_goes_above_the_ascent() {
        // Times New Roman (hhea 1825 / -443, gap 87), and a font with a large gap: Word puts the
        // first baseline the ascent plus the whole gap below the line top.
        assert_eq!(hhea_line(1825, -443, 87), (1912.0, 443.0));
        assert_eq!(hhea_line(1536, -512, 452), (1988.0, 512.0));
        // A negative gap adds nothing; the most negative descender doesn't overflow.
        assert_eq!(hhea_line(1000, -300, -50), (1000.0, 300.0));
        assert_eq!(hhea_line(1000, i16::MIN, 0), (1000.0, 32768.0));
    }

    #[test]
    fn single_spacing_uses_word_for_windows_metrics() {
        // Times New Roman and Arial: the win metrics equal hhea, so the whole gap is external
        // leading (13.8 pt at 12 pt). Calibri: the win metrics already cover hhea + gap (2500).
        assert_eq!(win_line(1825, 443, 1825, -443, 87), (1912.0, 443.0));
        assert_eq!(win_line(1854, 434, 1854, -434, 67), (1921.0, 434.0));
        assert_eq!(win_line(1950, 550, 1536, -512, 452), (1950.0, 550.0));
        // Sarabun: 1.853 em on Windows (hhea would give 1.30 em).
        assert_eq!(win_line(1286, 567, 1068, -232, 0), (1286.0, 567.0));
        // Extremes don't overflow; a negative gap adds nothing.
        assert_eq!(win_line(u16::MAX, u16::MAX, i16::MIN, i16::MAX, i16::MIN), (65535.0, 65535.0));
        assert_eq!(win_line(0, 0, 0, 0, 0), (0.0, 0.0));
        // The bundled Sarabun measures that way.
        let r = resolve("Sarabun", false, false);
        assert_eq!(r.face.family, "Sarabun");
        assert_eq!(line_metrics(&r.face), (1286.0, 567.0));
        // Zero win metrics fall back to hhea, as do absurd ones; hhea first when asked.
        assert_eq!(pick_line(Some((0, 0)), Some((800, -200, 0)), 1000.0, false), Some((800.0, 200.0)));
        assert_eq!(pick_line(Some((9000, 0)), Some((800, -200, 0)), 1000.0, false), Some((800.0, 200.0)));
        assert_eq!(pick_line(Some((1286, 567)), Some((1068, -232, 0)), 1000.0, true), Some((1068.0, 232.0)));
        assert_eq!(pick_line(None, None, 1000.0, false), None);
    }

    /// A substitute keeps the spacing of the font it stands in for: Cambria 1.172 em and Courier
    /// New 1.133 em whatever draws them, a Thai font drawn with Sarabun near 1.30 em, and the
    /// font asked for its own Word for Windows line (Sarabun 1.853 em).
    #[test]
    fn face_names_resolve_to_their_family_and_style() {
        // govuk-029's headings ask for "Arial Bold": Word draws Arial's bold, not a fallback.
        let a = resolve("Arial Bold", false, false);
        let b = resolve("Arial", true, false);
        assert_eq!(a.face.id(), b.face.id());
        assert_eq!((a.synth_bold, a.substituted), (b.synth_bold, b.substituted));
        assert_eq!(resolve("Times New Roman Bold Italic", false, false).face.id(), resolve("Times New Roman", true, true).face.id());
        assert_eq!(resolve("arial italic", false, false).face.id(), resolve("Arial", false, true).face.id());
        let face = crate::FontDb::global().face(crate::FALLBACK_FAMILY, "Regular");
        assert_eq!(single_line("Arial Bold", &face, true, 12.0), single_line("Arial", &face, true, 12.0));
        assert_eq!(substitute_for("Arial Bold"), substitute_for("Arial"));
        // Other words are part of the family name.
        assert_eq!(face_name("Calibri Light"), None);
        assert_eq!(face_name("Arial Black"), None);
        assert_eq!(face_name("Bold"), None);
        assert_eq!(face_name("Bembo Regular"), Some(("Bembo", false, false)));
    }

    /// Symbol and Wingdings keep Word's line when drawn with a substitute: a Symbol bullet makes
    /// a Calibri line taller, as in Word (win ascent 2059 / 2048 against Calibri's 1950).
    #[test]
    fn list_symbol_fonts_keep_their_line() {
        let face = crate::FontDb::global().face(crate::FALLBACK_FAMILY, "Regular");
        let em = |f: &str| {
            let (a, d) = single_line(f, &face, true, 1.0);
            ((a * 10000.0).round() / 10000.0, (d * 10000.0).round() / 10000.0)
        };
        assert_eq!(em("Symbol"), (1.0054, 0.2197));
        assert_eq!(em("Wingdings"), (0.8989, 0.2109));
        assert_eq!(em("Segoe UI Symbol"), (1.0791, 0.251));
        assert_eq!(em("MS Gothic"), (0.8594, 0.1406));
    }

    #[test]
    fn corpus_font_substitutions_prefer_bundled_open_stand_ins() {
        assert_eq!(substitutes("Univers").first(), Some(&"Arimo"));
        assert_eq!(substitutes("Arial Narrow").first(), Some(&"Roboto"));
        assert_eq!(substitutes("Gotham Book").first(), Some(&"Montserrat"));
        assert_eq!(substitutes("Garamond").first(), Some(&"Tinos"));
        assert_eq!(substitutes("Calibri").first(), Some(&"Carlito"));
        assert_eq!(substitutes("Calibri Light").first(), Some(&"Carlito"));
        assert_eq!(substitutes("Cambria").first(), Some(&"Caladea"));
        assert_eq!(substitutes("Cambria Math").first(), Some(&"Caladea"));
    }

    #[test]
    fn freely_licensed_source_sans_pro_uses_its_original_face() {
        assert_eq!(open_substitute("Source Sans Pro"), Some("Source Sans Pro"));
        let db = FontDb::global();
        if std::env::var_os("CRAFT_FONTS_REQUIRED").is_some() {
            assert!(db.has_family("Source Sans Pro"));
        }
        if db.has_family("Source Sans Pro") {
            for (bold, italic) in [(false, false), (true, false), (false, true), (true, true)] {
                let r = resolve("Source Sans Pro", bold, italic);
                assert_eq!(r.face.family, "Source Sans Pro");
                assert!(!r.substituted);
                assert_eq!(substitute_for("Source Sans Pro"), None);
                assert_eq!(r.face.italic, italic);
                assert_eq!(r.face.weight >= 600.0, bold);
            }
        }
    }

    #[test]
    fn open_original_roboto_is_not_replaced_with_arimo() {
        assert_eq!(open_substitute("Roboto"), Some("Roboto"));
        let db = crate::FontDb::global();
        if db.has_family("Roboto") {
            let resolved = resolve("Roboto", false, false);
            assert_eq!(resolved.face.family, "Roboto");
            assert!(!resolved.substituted);
            assert_eq!(substitute_for("Roboto"), None);
        }
    }

    #[test]
    fn pagination_corpus_font_aliases_resolve_to_open_faces_when_available() {
        let db = crate::FontDb::global();
        for (asked, expected) in [
            ("Arial", "Arimo"),
            ("Arial Narrow", "Roboto"),
            ("Arial Unicode MS", "Arimo"),
            ("Calibri", "Carlito"),
            ("Cambria", "Caladea"),
            ("Aptos", "Intos"),
            ("Garamond", "Tinos"),
            ("TimesNewRomanPSMT", "Tinos"),
            ("Courier", "Cousine"),
            ("PT Serif", "Source Serif 4"),
            ("Verdana", "Inter"),
            ("*Tahoma-Bold-4304", "Arimo"),
        ] {
            if db.has_family(expected) {
                assert_eq!(open_substitute(asked), Some(expected), "{asked}");
                let resolved = resolve(asked, false, false);
                assert_eq!(resolved.face.family, expected, "{asked}");
                assert!(resolved.substituted, "{asked}");
                assert_eq!(substitute_for(asked).as_deref(), Some(expected), "{asked}");
            }
        }
        for (asked, expected) in [
            ("Overpass", "Overpass"),
            ("MS Mincho", "Shippori Mincho"),
            ("SimSun", "Noto Sans CJK SC"),
            ("MS Gothic", "BIZ UDPGothic"),
            ("Batang", "NanumGothic"),
            ("Wingdings 3", "Noto Sans Symbols"),
            ("Shruti", "Noto Sans Gujarati"),
            ("Mangal", "Noto Sans Devanagari"),
            ("Nyala", "Noto Sans Ethiopic"),
        ] {
            if db.has_family(expected) {
                assert_eq!(open_substitute(asked), Some(expected), "{asked}");
                let resolved = resolve(asked, false, false);
                assert_eq!(resolved.face.family, expected, "{asked}");
                assert_eq!(resolved.substituted, !expected.eq_ignore_ascii_case(asked), "{asked}");
            }
        }
        for (asked, expected_weight) in
            [("Overpass ExtraLight", 200.0), ("Overpass Light", 300.0), ("Overpass SemiBold", 600.0), ("Overpass ExtraBold", 800.0)]
        {
            if db.has_family("Overpass") {
                let resolved = resolve(asked, false, false);
                assert_eq!(resolved.face.family, "Overpass", "{asked}");
                assert_eq!(resolved.face.weight, expected_weight, "{asked}");
                assert!(!resolved.substituted, "{asked}");
            }
        }
    }

    #[test]
    fn helvetica_and_times_use_the_open_metric_compatible_fonts_first() {
        for f in ["Helvetica", "helvetica", "Arial Nova"] {
            assert_eq!(substitutes(f).first(), Some(&"Arimo"), "{f}");
        }
        for f in ["Times", "times"] {
            assert_eq!(substitutes(f).first(), Some(&"Tinos"), "{f}");
        }
        assert_eq!(substitutes("Helvetica").get(1), Some(&"Arial"));
        assert_eq!(substitutes("Times").get(1), Some(&"Times New Roman"));
    }

    #[test]
    fn unicode_sans_names_do_not_select_a_monospace_substitute() {
        // Unicode contains "code", but says nothing about equal-width letters. In particular,
        // Arial Unicode MS's Latin text should use Arial's proportional stand-ins when absent.
        let mono = substitutes("Cascadia Code");
        for family in ["Arial Unicode MS", "Lucida Sans Unicode", "Missing Unicode Sans"] {
            assert_ne!(substitutes(family), mono, "{family}");
        }
        assert_eq!(substitutes("Missing Source Code"), mono);
        assert_eq!(substitutes("Missing Unicode Mono"), mono);
        for (bold, italic) in [(false, false), (true, false), (false, true), (true, true)] {
            let unicode = resolve("Arial Unicode MS", bold, italic);
            if !FontDb::global().has_family("Arial Unicode MS") {
                let arial = resolve("Arial", bold, italic);
                assert_eq!(unicode.face.id(), arial.face.id());
                assert_eq!((unicode.synth_bold, unicode.synth_italic), (arial.synth_bold, arial.synth_italic));
            }
            let width = |text| crate::shape(&unicode.face, text, &[], |c| c).iter().map(|g| i64::from(g.x_advance)).sum::<i64>();
            assert!(width("iiii") < width("WWWW"), "proportional Latin widths: {bold}/{italic}");
        }
    }

    #[test]
    fn substitutes_keep_the_requested_fonts_line() {
        let r = resolve("Sarabun", false, false);
        let em = |(a, d): (f64, f64)| ((a + d) * 1000.0).round() / 1000.0;
        assert_eq!(em(single_line("Sarabun", &r.face, false, 1.0)), 1.853);
        assert_eq!(em(single_line("Cordia New", &r.face, true, 1.0)), 1.3);
        assert_eq!(em(single_line("Cambria", &r.face, true, 1.0)), 1.172);
        assert_eq!(em(single_line("Courier New", &r.face, true, 1.0)), 1.133);
        // Points scale with the size.
        assert_eq!(em(single_line("Cambria", &r.face, true, 12.0)), 14.068);
        // Cambria Math is spaced like Cambria even when the face drawing it is its own (installed).
        assert_eq!(em(single_line("Cambria Math", &r.face, false, 1.0)), 1.172);
        assert_eq!(em(single_line("cambria math", &r.face, true, 10.5)), 12.31);
        // Letters a stand-in lacks keep Cambria's line in Cambria and Cambria Math runs only.
        assert_eq!(covered_fallback_line("Cambria Math", 10.5).map(em), Some(12.31));
        assert_eq!(covered_fallback_line("Cambria Bold", 1.0).map(em), Some(1.172));
        assert_eq!(covered_fallback_line(" cambria ", 1.0).map(em), Some(1.172));
        // Garamond keeps Word's 1.125 em line when another face stands in for it, and its measured
        // widths apply with Tinos or Times New Roman, not with other faces.
        assert_eq!(em(single_line("Garamond", &r.face, true, 1.0)), 1.125);
        assert_eq!(metric_fit("Garamond", true, true).map(|f| f.advance(' ')), None, "no bold italic widths were measured");
        if resolve("Garamond", false, false).face.family == "Tinos" {
            let fit = metric_fit("Garamond", false, false).expect("Garamond drawn with Tinos takes its widths");
            assert_eq!(fit.advance(' '), Some(0.25));
            assert_eq!(fit.advance('\u{10FFFF}'), None);
            assert_eq!(em(fit.line()), 1.125);
            assert!(metric_fit("Garamond", false, true).is_some() && metric_fit("Garamond", true, false).is_some());
        }
        // Cambria's measured widths (and tabular figures) apply with Caladea only.
        if resolve("Cambria", false, false).face.family == "Caladea" {
            for (bold, italic, digit) in [(false, false, 0.554), (true, false, 0.592), (false, true, 0.528), (true, true, 0.573)] {
                let fit = metric_fit("Cambria", bold, italic).expect("Cambria drawn with Caladea takes its widths");
                assert_eq!(fit.advance('0'), Some(digit));
                assert!(fit.tabular_digits());
                assert_eq!(em(fit.line()), 1.172);
            }
            assert_eq!(metric_fit("Cambria", false, false).and_then(|f| f.advance('e')), Some(0.488));
        }
        assert!(metric_fit("Garamond", false, false).is_none_or(|f| !f.tabular_digits()));
        // Calibri Light's narrower widths apply with Carlito, also to (synthetic) bold.
        if resolve("Calibri Light", false, false).face.family == "Carlito" {
            assert_eq!(metric_fit("Calibri Light", false, false).and_then(|f| f.advance('A')), Some(0.563));
            assert!(metric_fit("Calibri Light", false, true).is_some());
            assert_eq!(metric_fit("Calibri Light", true, true).and_then(|f| f.advance('A')), Some(0.563));
            assert!(metric_fit("Calibri", false, false).is_none());
        }
        // So do Symbol and Wingdings bullets drawn as standard characters by another face.
        // Overpass, every weight, steps 1.266 em whatever face draws it (13.293 pt at 10.5 pt).
        for name in ["Overpass", "Overpass Light", "overpass extrabold"] {
            assert_eq!(em(single_line(name, &r.face, false, 1.0)), 1.266, "{name}");
        }
        assert_eq!(em(single_line("Overpass", &r.face, true, 10.5)), 13.293);
        assert_eq!(covered_fallback_line("Wingdings", 11.0).map(em), Some(12.208));
        assert_eq!(covered_fallback_line("Symbol", 1.0).map(em), Some(1.225));
        assert_eq!(covered_fallback_line("wingdings 2", 1.0).map(em), Some(1.054));
        for other in ["Courier New", "Calibri", "Cambria Mathematics", "Sarabun", "Wingdings 3", "Webdings", ""] {
            assert_eq!(covered_fallback_line(other, 1.0), None, "{other}");
        }
    }

    #[test]
    fn th_sarabun_fonts_fit_sarabun_to_their_metrics() {
        for name in ["TH Sarabun New", "TH SarabunPSK", "th sarabun new", "THSarabunNew", "TH Sarabun PSK"] {
            let f = metric_fit(name, false, false).unwrap_or_else(|| panic!("{name}"));
            assert_eq!(f.advance('ก'), Some(0.386), "{name}");
            assert_eq!(f.advance('\u{0E48}'), Some(0.0), "{name}: tone marks take no width");
            assert_eq!(f.advance('😀'), None);
            assert!((f.scale(true) - 400.0 / 586.0).abs() < 1e-5 && (f.scale(false) - 0.68).abs() < 1e-5);
            assert_eq!(f.kern('A', 'V'), -0.05);
            assert_eq!(f.kern('ก', 'ข'), 0.0);
            assert_eq!(f.line(), (0.874, 0.457));
        }
        // Bold has its own widths and Thai scale (ก 407 high against Sarabun Bold's 586).
        let b = metric_fit("TH Sarabun New", true, false).unwrap();
        assert!((b.scale(true) - 407.0 / 586.0).abs() < 1e-5);
        assert_ne!(b.advance('ก'), Some(0.386));
        // Other families, and fonts that are themselves available, are drawn as they are; TH
        // SarabunIT๙ has no metrics here and is drawn with Sarabun at its own size.
        for name in ["Sarabun", "Arial", "TH Niramit AS", "TH SarabunIT๙", "TH SarabunIT9", "", "T", "THX", "ทดสอบ"] {
            assert!(metric_fit(name, false, false).is_none(), "{name}");
        }
    }

    /// Aptos isn't installed here: the bundled Intos (metric-compatible) draws it, Intos Display
    /// draws Aptos Display, and lines keep the spacing Word gives Aptos (1923 + 577 of 2048).
    #[test]
    fn aptos_is_drawn_with_the_bundled_intos() {
        for (asked, family) in [("Aptos", "Intos"), ("aptos", "Intos"), ("Aptos Display", "Intos Display"), ("Aptos SemiBold", "Intos")] {
            let r = resolve(asked, false, false);
            assert!(r.substituted, "{asked}");
            assert_eq!(r.face.family, family, "{asked}");
            let em = |(a, d): (f64, f64)| ((a + d) * 10000.0).round() / 10000.0;
            assert_eq!(em(single_line(asked, &r.face, true, 1.0)), 1.2207, "{asked}");
            // The same when Aptos itself is installed.
            assert_eq!(em(single_line(asked, &r.face, false, 1.0)), 1.2207, "{asked}");
        }
        for (bold, italic, style) in [(true, false, "Bold"), (false, true, "Italic"), (true, true, "Bold Italic")] {
            let r = resolve("Aptos", bold, italic);
            assert_eq!((r.face.family.as_str(), r.face.style.as_str()), ("Intos", style));
            assert!(!r.synth_bold && !r.synth_italic);
        }
        // Narrow and Serif are not bundled: their own look-alikes when installed, else the
        // next substitutes; Aptos Mono stays with the monospaced ones.
        assert_eq!(substitutes("Aptos Narrow").first(), Some(&"Intos Narrow"));
        assert_eq!(substitutes("Aptos Serif").first(), Some(&"Intos Serif"));
        assert_eq!(substitutes("Aptos Mono").first(), Some(&"Cousine"));
        assert_eq!(requested_line("Aptos Mono"), Some((1923.0 / 2048.0, 577.0 / 2048.0)));
        assert_eq!(requested_line("Aptosx"), None);
    }

    #[test]
    fn thai_families_named_by_weight_are_thai_document_fonts() {
        for f in ["Sarabun Light", "Sarabun ExtraBold", "Kanit Light", "Prompt SemiBold", "Mitr Medium Italic", "Sarabun Thin Italic"] {
            assert!(is_thai_document_font(&f.to_ascii_lowercase()), "{f}");
        }
        for f in ["Inter Light", "Arial Black", "Segoe UI Semibold", "Light"] {
            assert!(!is_thai_document_font(&f.to_ascii_lowercase()), "{f}");
        }
    }

    #[test]
    fn family_cache_stays_bounded() {
        for i in 0..(FAMILY_CACHE_MAX + 50) {
            let _ = resolve(&format!("Zz Missing Family {i}"), false, false);
        }
        assert!(resolve_cache().lock().unwrap_or_else(|e| e.into_inner()).1.len() <= FAMILY_CACHE_MAX);
        let long = "L".repeat(10_000);
        let _ = resolve(&long, false, false);
        assert!(!resolve_cache().lock().unwrap_or_else(|e| e.into_inner()).1.keys().any(|k| k.0.len() > 1000));
    }

    #[test]
    fn unknown_family_substitutes() {
        let r = resolve("Definitely Not A Font 123", false, false);
        assert!(r.substituted);
        let b = resolve("Source Serif 4", true, false);
        assert!(!b.substituted);
        assert!(!b.synth_bold);
        let (a, d) = line_metrics(&b.face);
        assert!(a > 0.0 && d >= 0.0);
    }

    #[test]
    fn palatino_family_prefers_palatino_look_alikes_over_generic_serif() {
        for name in ["Palatino Linotype", "Book Antiqua", "palatino"] {
            let list = substitutes(name);
            let pos = |n: &str| list.iter().position(|x| *x == n);
            assert!(pos("Palatino").is_some() && pos("TeX Gyre Pagella").is_some() && pos("P052").is_some(), "{name}");
            // Free bundled serif faces come before the proprietary platform fonts.
            assert!(pos("Caladea") < pos("Palatino") && pos("Source Serif 4") < pos("Times New Roman"), "{name}");
        }
        assert_eq!(substitutes("Garamond").first(), Some(&"Tinos"));
    }

    /// Verdana and Georgia go to the open fonts nearest their widths (DejaVu Sans, Source Serif
    /// 4) and keep Word's own line spacing for them.
    #[test]
    fn verdana_and_georgia_keep_their_widths_and_lines() {
        assert_eq!(substitutes("Verdana").first(), Some(&"DejaVu Sans"));
        assert!(substitutes("Georgia").iter().position(|x| *x == "Source Serif 4") < substitutes("Georgia").iter().position(|x| *x == "Caladea"));
        let r = resolve("Georgia", false, false);
        let em = |(a, d): (f64, f64)| ((a + d) * 1000.0).round() / 1000.0;
        assert_eq!(em(single_line("Georgia", &r.face, true, 1.0)), 1.136);
        assert_eq!(em(single_line("Verdana", &r.face, true, 1.0)), 1.215);
    }

    #[test]
    fn thai_document_fonts_are_drawn_with_the_bundled_sarabun() {
        for name in ["TH Sarabun New", "TH SarabunPSK", "TH SarabunIT๙", "TH Niramit AS", "Angsana New", "AngsanaUPC", "Cordia New", "Noto Sans Thai"]
        {
            assert_eq!(substitutes(name), &["Sarabun"], "{name}");
        }
        // Other families keep their substitutes ("Thai" only in a name like "Noto Sans Thai").
        assert_eq!(substitutes("Garamond").first(), Some(&"Tinos"));
        assert_ne!(substitutes("Tahoma"), &["Sarabun"]);
        assert_ne!(substitutes("Theinhardt"), &["Sarabun"]);
        // Not installed here, so the request resolves to Sarabun itself (bundled).
        let r = resolve("TH Sarabun New Doc4Me Test", false, false);
        assert!(r.substituted);
        assert_eq!(r.face.family, "Sarabun");
        let b = resolve("Sarabun", true, false);
        assert!(!b.substituted && !b.synth_bold);
        assert_eq!(b.face.style, "Bold");
    }

    #[test]
    fn symbol_font_chars_map_to_unicode() {
        assert_eq!(symbol_font_char("Symbol", '\u{F0B7}'), Some('\u{2022}'));
        assert_eq!(symbol_font_char("symbol", '\u{F0B7}'), Some('\u{2022}'));
        assert_eq!(symbol_font_char("Wingdings", '\u{F0A7}'), Some('\u{25AA}'));
        assert_eq!(symbol_font_char("Wingdings", '\u{F0D8}'), Some('\u{27A2}'));
        // Same code, other font; unknown codes; ordinary characters.
        assert_eq!(symbol_font_char("Arial", '\u{F0B7}'), None);
        assert_eq!(symbol_font_char("Symbol", '\u{F041}'), None);
        assert_eq!(symbol_font_char("Symbol", 'a'), None);
        assert_eq!(symbol_font_char("Symbol", '\u{10FFFF}'), None);
    }

    #[test]
    fn resolves_weights_named_as_their_own_family() {
        // Word stores a Medium face as the family "Inter Medium" (#219).
        let r = resolve("Inter Medium", false, false);
        assert!(!r.substituted && !r.synth_bold && !r.synth_italic);
        assert_eq!((r.face.family.as_str(), r.face.style.as_str()), ("Inter", "Medium"));
        // Bold and italic within it are drawn from the Medium face.
        let b = resolve("Inter Medium", true, true);
        assert_eq!(b.face.style, "Medium");
        assert!(!b.substituted && b.synth_bold && b.synth_italic);
    }

    #[test]
    fn synthesizes_missing_styles() {
        // JetBrains Mono ships only Regular here.
        let r = resolve("JetBrains Mono", true, true);
        assert!(r.synth_bold && r.synth_italic);
    }
}
