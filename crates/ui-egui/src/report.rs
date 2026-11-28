//! Report a problem (แจ้งปัญหา): a button beside Share opens Chanon's Google Form in a new tab, with
//! the app version and the browser filled in. No document content is sent; the form is answered on
//! Google's page. The button shows only while a form is configured here.

/// The Google Form's prefill address, without its query (`…/viewform`); empty hides the button.
/// Chanon's form "แจ้งปัญหา เขียนครับ (KhianKrub)".
pub const FORM_URL: &str = "https://docs.google.com/forms/d/e/1FAIpQLSddU9FebLOY5spPMQFfnGadCSkn_Rw6jJBdUhxdwWWsGHjEWA/viewform";
/// The form's question ids (`entry.<number>`) for the two answers the app fills in.
pub const VERSION_ENTRY: &str = "entry.368913138";
pub const BROWSER_ENTRY: &str = "entry.1229712171";

/// The form's address with the version and browser filled in, or `None` when no form is configured.
pub fn report_url(version: &str, browser: &str) -> Option<String> {
    report_url_for(FORM_URL, VERSION_ENTRY, BROWSER_ENTRY, version, browser)
}

fn report_url_for(form: &str, version_entry: &str, browser_entry: &str, version: &str, browser: &str) -> Option<String> {
    if form.is_empty() {
        return None;
    }
    let mut url = format!("{form}?usp=pp_url");
    for (entry, value) in [(version_entry, version), (browser_entry, browser)] {
        if !entry.is_empty() && !value.is_empty() {
            // Long user-agent strings are cut, at a character boundary, to keep the address short.
            let value: String = value.chars().take(300).collect();
            url.push_str(&format!("&{entry}={}", encode(&value)));
        }
    }
    Some(url)
}

/// Percent-encode a query value (RFC 3986 unreserved characters are kept).
fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
