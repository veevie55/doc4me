//! Publisher pages for fonts a document may name but this computer doesn't have.

const LIST: &str = include_str!("font_downloads.json");
const MAX_DOWNLOADS: usize = 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontDownload {
    pub name: String,
    pub publisher: String,
    pub url: Option<String>,
    pub windows: bool,
    pub restricted: bool,
}

fn parse(list: &str) -> Vec<FontDownload> {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(list) else {
        log::warn!("font downloads: the list isn't JSON");
        return Vec::new();
    };
    json.get("downloads")
        .and_then(|entries| entries.as_array())
        .into_iter()
        .flatten()
        .take(MAX_DOWNLOADS)
        .filter_map(|entry| {
            let field = |key: &str| entry.get(key).and_then(|value| value.as_str()).map(str::trim).filter(|value| !value.is_empty());
            let flag = |key: &str| entry.get(key).and_then(|value| value.as_bool()).unwrap_or(false);
            let (name, publisher) = (field("family")?, field("publisher")?);
            let url = match field("url") {
                Some(url) if url.starts_with("https://") => Some(url.to_string()),
                Some(_) => return None,
                None if flag("windows") => None,
                None => return None,
            };
            Some(FontDownload {
                name: name.to_string(),
                publisher: publisher.to_string(),
                url,
                windows: flag("windows"),
                restricted: flag("restricted"),
            })
        })
        .collect()
}

fn downloads() -> &'static [FontDownload] {
    static DOWNLOADS: std::sync::OnceLock<Vec<FontDownload>> = std::sync::OnceLock::new();
    DOWNLOADS.get_or_init(|| parse(LIST))
}

/// Where a family can be had, when the publisher is known.
pub fn download_for(family: &str) -> Option<&'static FontDownload> {
    family.split(';').map(str::trim).filter(|name| !name.is_empty()).find_map(|name| {
        let name = crate::word::charset_base_family(name).trim();
        downloads().iter().find(|download| {
            let (candidate, name) = (download.name.as_bytes(), name.as_bytes());
            name.get(..candidate.len()).is_some_and(|head| head.eq_ignore_ascii_case(candidate))
                && matches!(name.get(candidate.len()), None | Some(b' '))
        })
    })
}
