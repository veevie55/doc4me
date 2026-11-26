//! Windows only: embed the app icon and version info (VERSIONINFO) into `doc4me.exe`, so it
//! shows in Explorer, the taskbar, the Start menu and Alt-Tab. Builds are named by commit: the
//! file and product version strings Explorer shows are the commit (`DOC4ME_BUILD_SHA`, short), or
//! `dev`. A build given a version number (`DOC4ME_VERSION`, see `crates/engine/src/build_info.rs`)
//! shows `1.2.3 (commit)` there and puts 1.2.3 in the numeric fields; otherwise those keep Cargo's version.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `DOC4ME_REQUIRE_WINRES=1` turns it
//! into an error (for release builds).

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/doc4me.ico");
    println!("cargo:rerun-if-env-changed=DOC4ME_REQUIRE_WINRES");
    println!("cargo:rerun-if-env-changed=DOC4ME_BUILD_SHA");
    println!("cargo:rerun-if-env-changed=DOC4ME_VERSION");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let sha = std::env::var("DOC4ME_BUILD_SHA").unwrap_or_default();
    let sha = sha.trim();
    let commit = if (7..=64).contains(&sha.len()) && sha.bytes().all(|b| b.is_ascii_hexdigit()) { &sha[..sha.len().min(8)] } else { "dev" };
    let version = std::env::var("DOC4ME_VERSION").ok().and_then(|v| numeric_version(&v).map(|n| (v.trim().trim_start_matches('v').to_string(), n)));
    let shown = match &version {
        Some((v, _)) => format!("{v} ({commit})"),
        None => commit.to_string(),
    };
    let mut res = winresource::WindowsResource::new();
    if let Some((_, (major, minor, patch))) = version {
        let packed = (u64::from(major) << 48) | (u64::from(minor) << 32) | (u64::from(patch) << 16);
        res.set_version_info(winresource::VersionInfo::FILEVERSION, packed).set_version_info(winresource::VersionInfo::PRODUCTVERSION, packed);
    }
    res.set_icon("../../assets/app-icon/doc4me.ico")
        .set("ProductName", "Doc4Me")
        .set("FileDescription", "Doc4Me word processor")
        .set("CompanyName", "Doc4Me contributors")
        .set("LegalCopyright", "Copyright (c) 2026 ArtCraft Team and the WordCraft contributors, and the Doc4Me contributors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "doc4me.exe")
        .set("InternalName", "doc4me")
        .set("FileVersion", &shown)
        .set("ProductVersion", &shown);
    if let Err(e) = res.compile() {
        if std::env::var_os("DOC4ME_REQUIRE_WINRES").is_some() {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=doc4me.exe built without icon/version resources: {e}");
    }
}

/// `1.2.3` or `v1.2.3-rc.1` as the three numbers VERSIONINFO holds (16 bits each), the
/// pre-release tag dropped; `None` for anything else, which then counts as no version, as
/// `build_info::version()` reads it (same rules, plus the 16-bit limit).
fn numeric_version(v: &str) -> Option<(u16, u16, u16)> {
    let v = v.trim();
    let v = v.strip_prefix('v').unwrap_or(v);
    if v.len() > 64 {
        return None;
    }
    let (core, pre) = match v.split_once('-') {
        Some((c, p)) => (c, Some(p)),
        None => (v, None),
    };
    if pre.is_some_and(|p| p.split('.').any(|id| id.is_empty() || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))) {
        return None;
    }
    let mut parts = core.split('.').map(|n| {
        let ok = !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) && !(n.len() > 1 && n.starts_with('0'));
        if ok { n.parse::<u16>().ok() } else { None }
    });
    let (major, minor, patch) = (parts.next()??, parts.next()??, parts.next()??);
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}
