//! Which hyperlink targets may leave the document: followed by the OS, or written into an
//! exported HTML/Markdown file where a browser would run them.

/// Whether Ctrl/⌘+click may hand a document's hyperlink to the system: web and email links only,
/// so a document can't launch `file:` paths, programs or custom-scheme handlers. Exact: a leading
/// space or a control character makes it not a link.
pub fn is_web_or_mail_link(url: &str) -> bool {
    let Some((scheme, _)) = url.split_once(':') else { return false };
    ["http", "https", "mailto"].iter().any(|s| scheme.eq_ignore_ascii_case(s))
}

/// Whether a hyperlink may be written as an `href` (or Markdown destination) in an exported file:
/// a web or email link, a bookmark (`#name`) or a relative path. Scripts (`javascript:`,
/// `vbscript:`), `data:`, `file:` and every other scheme are refused, read the way a browser reads
/// a URL: ASCII control characters and spaces are dropped first, so `java\nscript:` is a script.
/// A protocol-relative `//host` or UNC `\\host` reference names another machine (following it on
/// Windows sends the user's NTLM credentials there), so it is refused too (WordCraft #3862).
pub fn is_safe_href(url: &str) -> bool {
    // Exporters drop other control characters and noncharacters when they write the link
    // (`file\u{85}:` comes out `file:`), so a link holding one is refused, not judged as it
    // stands and written as something else (WordCraft #5426).
    if url.chars().any(|c| (c.is_control() && !c.is_ascii_control()) || matches!(c, '\u{FFFE}' | '\u{FFFF}')) {
        return false;
    }
    let cleaned: String = url.chars().filter(|c| !c.is_ascii_control() && *c != ' ').collect();
    if cleaned.replace('\\', "/").starts_with("//") {
        return false;
    }
    match scheme(url) {
        Some(s) => matches!(s.as_str(), "http" | "https" | "mailto"),
        None => !cleaned.is_empty(),
    }
}

/// The scheme a browser would see in `url` (RFC 3986 §3.1: a letter, then letters, digits, `+`,
/// `-` or `.`, up to the first `:`), lower-cased; `None` for a relative reference.
fn scheme(url: &str) -> Option<String> {
    let cleaned: String = url.chars().filter(|c| !c.is_ascii_control() && *c != ' ').collect();
    let (s, _) = cleaned.split_once(':')?;
    let mut chars = s.chars();
    let first = chars.next()?;
    (first.is_ascii_alphabetic() && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))).then(|| s.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// WordCraft #3862 and #5426: another machine (`//host`, `\\host`) and links hiding a scheme
    /// behind characters the exporters drop are not written as an `href`.
    #[test]
    fn remote_hosts_and_hidden_schemes_are_not_safe_hrefs() {
        for bad in [
            "//attacker.example/x",
            "\\\\attacker.example\\share\\x",
            "\\/attacker.example/x",
            " //attacker.example/x",
            "/\t/attacker.example/x",
            "file\u{85}:///etc/passwd",
            "java\u{9F}script:alert(1)",
            "/\u{FFFE}/attacker.example/x",
        ] {
            assert!(!is_safe_href(bad), "{bad:?}");
        }
        for ok in ["/docs/a.html", "a/b.html", "#top", "https://example.com", "mailto:a@example.com", "ภาษาไทย.html"] {
            assert!(is_safe_href(ok), "{ok:?}");
        }
    }

    #[test]
    fn only_web_and_mail_links_are_followable() {
        for ok in ["https://example.com/a?b#c", "http://example.com", "HTTPS://EXAMPLE.COM", "mailto:someone@example.com"] {
            assert!(is_web_or_mail_link(ok), "{ok}");
        }
        for bad in [
            "file:///etc/passwd",
            "C:\\Windows\\System32\\calc.exe",
            "/usr/bin/xterm",
            "javascript:alert(1)",
            "ms-msdt:/id",
            " https://example.com",
            "https\u{0}://example.com",
            "",
            "example.com",
            "#top",
        ] {
            assert!(!is_web_or_mail_link(bad), "{bad:?}");
        }
    }

    /// An exported `href` may be a web/mail link, a bookmark or a relative path: never a script
    /// or another scheme, however a browser would normalise it.
    #[test]
    fn exported_hrefs_allow_bookmarks_and_relative_paths_but_no_scripts() {
        for ok in ["https://example.com/x", "mailto:a@b.c", "#top", "other.html", "docs/guide.html#s2", "../up.html", "?q=1"] {
            assert!(is_safe_href(ok), "{ok}");
        }
        for bad in [
            "javascript:alert(1)",
            "JavaScript:alert(1)",
            "java\nscript:alert(1)",
            "\tjavascript:alert(1)",
            " javascript:alert(1)",
            "java\u{0}script:alert(1)",
            "vbscript:MsgBox(1)",
            "data:text/html,<script>alert(1)</script>",
            "file:///etc/passwd",
            "ms-msdt:/id",
            // Protocol-relative and UNC references name another machine (NTLM leak on Windows).
            "//attacker.example/share/x",
            "\\\\attacker.example\\share\\x",
            "/\\attacker.example\\x",
            " //attacker.example/x",
            "",
        ] {
            assert!(!is_safe_href(bad), "{bad:?}");
        }
    }
}
