//! The commit a build was made from, and the version number given to it, if any. Doc4Me names its
//! builds by commit: `--version`, the MCP server and web requests show it. A build may also be
//! given a version number (`DOC4ME_VERSION`, typed in the build or Release workflow); the About
//! dialog, `--version` and the MCP server then show it as well. Without one, the About dialog
//! shows neither.
//!
//! The build scripts and workflows set `DOC4ME_BUILD_SHA` (`packaging/env.sh`,
//! `packaging/windows/package.ps1`, `.github/workflows/build-*.yml`); a plain `cargo build`
//! without it is a development build.

/// The full commit SHA baked in at build time, if it looks like one.
pub fn commit_full() -> Option<&'static str> {
    option_env!("DOC4ME_BUILD_SHA").map(str::trim).filter(|s| valid(s))
}

/// The commit, shortened to 8 hex digits as `--version` and package names show
/// it, if the build has one.
pub fn commit() -> Option<&'static str> {
    commit_full().map(short)
}

/// `commit()`, or `dev` for a development build: one word for logs, `--version` and user agents.
pub fn commit_or_dev() -> &'static str {
    commit().unwrap_or("dev")
}

/// The version number this build was given (`DOC4ME_VERSION`), if it looks like one: `1.2.3` or
/// `1.2.3-rc.1`, an optional leading `v` dropped. A plain `cargo build` has none.
pub fn version() -> Option<&'static str> {
    option_env!("DOC4ME_VERSION").and_then(version_of)
}

/// `1.2.3 (abcdef12)` with a version, `1.2.3 (dev)` without a commit, or `commit_or_dev()`
/// without a version: what `--version` prints after the program's name.
pub fn describe() -> String {
    match version() {
        Some(v) => format!("{v} ({})", commit_or_dev()),
        None => commit_or_dev().to_string(),
    }
}

/// The version, else `commit_or_dev()`: one word, for the MCP server's version and user agents.
pub fn version_or_commit() -> &'static str {
    version().unwrap_or_else(commit_or_dev)
}

/// `s` as a version number: `MAJOR.MINOR.PATCH` (no leading zeros) with an optional
/// `-pre.release` tag of letters, digits and hyphens, at most 64 characters; a leading `v` is
/// dropped. Anything else (spaces, markup, build metadata) is no version.
fn version_of(s: &str) -> Option<&str> {
    let s = s.trim();
    let s = s.strip_prefix('v').unwrap_or(s);
    if s.len() > 64 {
        return None;
    }
    let (core, pre) = match s.split_once('-') {
        Some((c, p)) => (c, Some(p)),
        None => (s, None),
    };
    let nums: Vec<&str> = core.split('.').collect();
    let number = |n: &&str| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) && !(n.len() > 1 && n.starts_with('0'));
    if nums.len() != 3 || !nums.iter().all(number) {
        return None;
    }
    if let Some(p) = pre
        && p.split('.').any(|id| id.is_empty() || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
    {
        return None;
    }
    Some(s)
}

fn valid(s: &str) -> bool {
    (7..=64).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_hexdigit())
}

fn short(s: &str) -> &str {
    s.get(..8).unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_hex_commits_are_shown_and_they_are_shortened() {
        assert!(valid("552a0b68"));
        assert!(valid("552a0b68c3e1f0d4a9b7c6e5d4c3b2a1f0e9d8c7"));
        for bad in ["", "abc", "not-a-commit", "552a0b6 ", "v1.1.4", "552a0b68\n"] {
            assert!(!valid(bad), "{bad:?}");
        }
        assert_eq!(short("552a0b68c3e1f0d4a9b7c6e5d4c3b2a1f0e9d8c7"), "552a0b68");
        assert_eq!(short("552a0b6"), "552a0b6");
        assert!(commit_or_dev() == "dev" || valid(commit_or_dev()));
    }

    #[test]
    fn only_plain_version_numbers_are_shown() {
        assert_eq!(version_of("1.2.0"), Some("1.2.0"));
        assert_eq!(version_of("v1.2.0"), Some("1.2.0"));
        assert_eq!(version_of(" 2.0.10-rc.1 "), Some("2.0.10-rc.1"));
        assert_eq!(version_of("10.0.0-beta-2"), Some("10.0.0-beta-2"));
        for bad in ["", "v", "1.2", "1.2.3.4", "01.2.3", "1.2.3-", "1.2.3-a..b", "1.2.3+build", "1.2.3 beta", "1.2.3-<b>", "vv1.2.3", "1.2.x"] {
            assert_eq!(version_of(bad), None, "{bad:?}");
        }
        assert_eq!(version_of(&format!("1.2.3-{}", "a".repeat(60))), None);
        match version() {
            Some(v) => assert_eq!(describe(), format!("{v} ({})", commit_or_dev())),
            None => assert_eq!(describe(), commit_or_dev()),
        }
        assert!(version_or_commit() == commit_or_dev() || version().is_some());
    }
}
