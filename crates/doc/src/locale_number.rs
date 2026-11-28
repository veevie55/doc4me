//! Numbers written the way a language writes them: `1 250,50` in Polish, `1.250,50` in German,
//! `1,250.50` in English. Used by table sort and table formulas.

fn language(lang: Option<&str>) -> String {
    lang.unwrap_or("").trim().to_ascii_lowercase().replace('_', "-").split(['-', '.', '@']).next().unwrap_or_default().to_string()
}

/// The decimal separator of language `lang` (a BCP 47 tag; `None` means English).
pub fn decimal_separator(lang: Option<&str>) -> char {
    let tag = lang.unwrap_or("").trim().to_ascii_lowercase().replace('_', "-");
    // Regions that write a decimal point where the language usually writes a comma.
    const POINT_REGIONS: &[&str] =
        &["de-ch", "de-li", "it-ch", "es-mx", "es-us", "es-419", "es-pr", "es-do", "es-gt", "es-hn", "es-ni", "es-pa", "es-sv"];
    if POINT_REGIONS.iter().any(|r| tag == *r || tag.starts_with(&format!("{r}-"))) {
        return '.';
    }
    const COMMA: &[&str] = &[
        "af", "az", "be", "bg", "bs", "ca", "cs", "da", "de", "el", "es", "et", "eu", "fi", "fo", "fr", "gl", "hr", "hu", "hy", "id", "is", "it",
        "ka", "kk", "ky", "lb", "lt", "lv", "mk", "mn", "nb", "nl", "nn", "no", "pl", "pt", "ro", "ru", "sk", "sl", "sq", "sr", "sv", "tr", "uk",
        "uz", "vi",
    ];
    if COMMA.contains(&language(lang).as_str()) { ',' } else { '.' }
}

const SPACES: &[char] = &[' ', '\u{a0}', '\u{202f}', '\u{2009}'];
const CURRENCY: &[char] = &['$', '€', '£', '¥', '¢', '₽', '₴', '₺', '₹', '₩', '₪', '₫', '฿', '₱', '₸', '₼', '₾'];

/// Strip a currency symbol or a short currency word (`zł`, `kr`, `PLN`) from either end.
fn strip_currency(s: &str) -> &str {
    let mut s = s.trim_matches(|c: char| c.is_whitespace() || CURRENCY.contains(&c));
    if let Some((head, word)) = s.rsplit_once(SPACES)
        && (1..=3).contains(&word.chars().count())
        && word.chars().all(|c| c.is_alphabetic() || c == '.')
    {
        s = head;
    }
    if let Some((word, tail)) = s.split_once(SPACES)
        && word.chars().count() == 3
        && word.chars().all(|c| c.is_ascii_uppercase())
    {
        s = tail;
    }
    s.trim_matches(|c: char| c.is_whitespace() || CURRENCY.contains(&c))
}

/// Digits of an integer part with group separators `groups`: the first group has 1–3 digits,
/// every later one exactly 3.
fn grouped_digits(int: &str, groups: &[char]) -> Option<String> {
    if int.chars().all(|c| c.is_ascii_digit()) {
        return Some(int.to_string());
    }
    let parts: Vec<&str> = int.split(groups).collect();
    let (first, rest) = parts.split_first()?;
    // A first group of `0` is never grouping (`0.125` is a fraction, not 125).
    let ok = (1..=3).contains(&first.len())
        && !first.starts_with('0')
        && first.chars().all(|c| c.is_ascii_digit())
        && rest.iter().all(|g| g.len() == 3 && g.chars().all(|c| c.is_ascii_digit()));
    ok.then(|| parts.concat())
}

/// Parse `s` as a number written in language `lang`, or `None` when it isn't one. The decimal
/// separator is the language's; group separators are spaces (and `.` or `'` where the decimal
/// separator is a comma, `,` or `'` where it is a point) between groups of three digits. A
/// currency sign or short currency word is ignored, and so is a trailing `%` (`12 %` is 12, as
/// table sort reads it). In comma languages a lone `.` that cannot be a group separator (`1.5`)
/// is read as a decimal point. A trailing separator with no digits after it (`1.`, an ordinal
/// in a numbered column) is the whole number. A minus may be `-`, `−` or an en dash.
pub fn parse_number(s: &str, lang: Option<&str>) -> Option<f64> {
    let dec = decimal_separator(lang);
    let s = s.trim();
    let s = s.strip_suffix('%').map_or(s, str::trim_end);
    let mut body = strip_currency(s);
    let mut neg = false;
    if let Some(rest) = body.strip_prefix(['-', '\u{2212}', '\u{2013}']) {
        neg = true;
        body = strip_currency(rest);
    } else if let Some(rest) = body.strip_prefix('+') {
        body = strip_currency(rest);
    }
    if body.is_empty() || body.len() > 400 {
        return None;
    }
    let mut groups: Vec<char> = SPACES.to_vec();
    groups.extend(['\'', '\u{2019}']);
    groups.push(if dec == ',' { '.' } else { ',' });
    let (int, frac) = match body.split_once(dec) {
        Some((i, f)) => (i, Some(f)),
        None => (body, None),
    };
    let int_digits = match grouped_digits(int, &groups) {
        Some(d) => d,
        None if dec == ',' && frac.is_none() && int.matches('.').count() == 1 => {
            return parse_number(&format!("{}{}", if neg { "-" } else { "" }, int.replace('.', ",")), lang);
        }
        None => return None,
    };
    let frac_digits = match frac {
        Some(f) if f.chars().all(|c| c.is_ascii_digit()) => f,
        Some(_) => return None,
        None => "",
    };
    if int_digits.is_empty() && frac_digits.is_empty() {
        return None;
    }
    let v: f64 = format!("0{int_digits}.{frac_digits}0").parse().ok()?;
    if !v.is_finite() {
        return None;
    }
    Some(if neg { -v } else { v })
}

/// Write `v` the way language `lang` writes a formula result: whole numbers without decimals,
/// others with two decimals and the language's decimal separator (`5,75` in Polish).
pub fn format_number(v: f64, lang: Option<&str>) -> String {
    if !v.is_finite() {
        return "0".into();
    }
    if v.fract() == 0.0 && v.abs() < 1e15 {
        return format!("{}", v as i64);
    }
    let s = format!("{v:.2}");
    if decimal_separator(lang) == ',' { s.replace('.', ",") } else { s }
}
