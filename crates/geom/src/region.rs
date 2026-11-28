//! Locale conventions for paper size, measurement units, and decimal separators.

use serde::{Deserialize, Serialize};

use crate::Unit;

const LETTER: &[&str] = &["BZ", "CA", "CL", "CO", "CR", "GT", "MX", "NI", "PA", "PH", "PR", "SV", "US", "VE"];
const INCHES: &[&str] = &["LR", "MM", "US"];
const COMMA_LANGUAGES: &[&str] = &[
    "af", "az", "be", "bg", "bs", "ca", "cs", "da", "de", "el", "es", "et", "eu", "fi", "fo", "fr", "gl", "hr", "hu", "hy", "id", "is", "it", "ka",
    "kk", "ky", "lb", "lt", "lv", "mk", "nb", "nl", "nn", "no", "pl", "pt", "ro", "ru", "sk", "sl", "sq", "sr", "sv", "tr", "uk", "uz", "vi",
];
const POINT_EXCEPTIONS: &[(&str, &[&str])] =
    &[("de", &["CH", "LI"]), ("it", &["CH"]), ("es", &["CU", "DO", "GT", "HN", "MX", "NI", "PA", "PE", "PR", "SV", "US", "419"])];
const COMMA_TERRITORIES: &[&str] = &[
    "AR", "AT", "BE", "BG", "BO", "BR", "CL", "CO", "CZ", "DE", "DK", "EC", "EE", "ES", "FI", "FR", "GR", "HR", "HU", "ID", "IS", "IT", "LT", "LU",
    "LV", "NL", "NO", "PL", "PT", "PY", "RO", "RS", "RU", "SE", "SI", "SK", "TR", "UA", "UY", "VE", "VN", "ZA",
];

/// A locale's language and territory. Defaults to `en-US` (Letter, inches, decimal point).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    pub language: String,
    pub territory: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub letter: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inches: Option<bool>,
}

impl Default for Region {
    fn default() -> Self {
        Region { language: "en".into(), territory: "US".into(), letter: None, inches: None }
    }
}

impl Region {
    /// Parse a BCP 47 or POSIX locale such as `pl-PL` or `de_AT.UTF-8@euro`.
    pub fn from_tag(tag: &str) -> Region {
        let tag: String = tag.chars().take(64).collect();
        let tag = tag.split(['.', '@']).next().unwrap_or_default().trim();
        if tag.is_empty() || tag.eq_ignore_ascii_case("c") || tag.eq_ignore_ascii_case("posix") {
            return Region::default();
        }
        let mut parts = tag.split(['-', '_']);
        let language = parts.next().unwrap_or_default();
        if !(2..=3).contains(&language.len()) || !language.chars().all(|c| c.is_ascii_alphabetic()) {
            return Region::default();
        }
        let language = language.to_ascii_lowercase();
        let mut territory = String::new();
        for p in parts {
            if p.len() == 1 {
                break;
            }
            let alpha = p.len() == 2 && p.chars().all(|c| c.is_ascii_alphabetic());
            let digits = p.len() == 3 && p.chars().all(|c| c.is_ascii_digit());
            if alpha || digits {
                territory = p.to_ascii_uppercase();
                break;
            }
        }
        if territory.is_empty() && language == "en" {
            territory = "US".into();
        }
        Region { language, territory, letter: None, inches: None }
    }

    pub fn with_territory(mut self, territory: &str) -> Region {
        let t = territory.trim();
        let ok = (t.len() == 2 && t.chars().all(|c| c.is_ascii_alphabetic())) || (t.len() == 3 && t.chars().all(|c| c.is_ascii_digit()));
        if ok {
            self.territory = t.to_ascii_uppercase();
        }
        self
    }

    pub fn with_paper_and_measurement(mut self, paper: Option<&str>, measurement: Option<&str>) -> Region {
        if let Some(t) = paper {
            self.letter = Some(Region::from_tag(t).uses_letter());
        }
        if let Some(t) = measurement {
            self.inches = Some(Region::from_tag(t).units() == Unit::Inches);
        }
        self
    }

    /// Read locale environment variables first, falling back to a platform locale if supplied.
    pub fn from_locale_vars(var: impl Fn(&str) -> Option<String>, platform: impl FnOnce() -> Option<String>) -> Region {
        let var = |name: &str| var(name).filter(|v| usable_locale(v));
        if let Some(tag) = var("DOC4ME_REGION").or_else(|| var("WORDCRAFT_REGION")).or_else(|| var("LC_ALL")) {
            return Region::from_tag(&tag);
        }
        let base = var("LC_NUMERIC").or_else(|| var("LANG")).unwrap_or_else(|| platform().unwrap_or_default());
        Region::from_tag(&base).with_paper_and_measurement(var("LC_PAPER").as_deref(), var("LC_MEASUREMENT").as_deref())
    }

    pub fn uses_letter(&self) -> bool {
        self.letter.unwrap_or_else(|| LETTER.contains(&self.territory.as_str()))
    }

    pub fn paper(&self) -> (&'static str, f32, f32) {
        if self.uses_letter() { ("Letter", 612.0, 792.0) } else { ("A4", 595.28, 841.89) }
    }

    pub fn units(&self) -> Unit {
        if self.inches.unwrap_or_else(|| INCHES.contains(&self.territory.as_str())) { Unit::Inches } else { Unit::Centimeters }
    }

    pub fn decimal(&self) -> char {
        let lang = self.language.as_str();
        let territory = self.territory.as_str();
        if COMMA_LANGUAGES.contains(&lang) {
            let point = POINT_EXCEPTIONS.iter().any(|(l, ts)| *l == lang && ts.contains(&territory));
            return if point { '.' } else { ',' };
        }
        if lang == "en" && COMMA_TERRITORIES.contains(&territory) { ',' } else { '.' }
    }

    pub fn format(&self, unit: Unit, pt: f32) -> String {
        unit.format_with(pt, self.decimal())
    }
}

fn usable_locale(v: &str) -> bool {
    let v = v.trim();
    !v.is_empty() && !v.eq_ignore_ascii_case("c") && !v.eq_ignore_ascii_case("posix") && !v.starts_with("C.")
}
