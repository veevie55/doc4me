//! The commit a build was made from. Doc4Me names its builds by commit, not by version number:
//! `--version`, the MCP server and web requests show it (the About dialog shows no version or commit).
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
}
