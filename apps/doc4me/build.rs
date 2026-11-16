//! Windows only: embed the app icon and version info (VERSIONINFO) into `doc4me.exe`, so it
//! shows in Explorer, the taskbar, the Start menu and Alt-Tab. Builds are named by commit: the
//! file and product version strings Explorer shows are the commit (`DOC4ME_BUILD_SHA`, short), or
//! `dev`; only the numeric fields Windows requires keep Cargo's version.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `DOC4ME_REQUIRE_WINRES=1` turns it
//! into an error (for release builds).

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/doc4me.ico");
    println!("cargo:rerun-if-env-changed=DOC4ME_REQUIRE_WINRES");
    println!("cargo:rerun-if-env-changed=DOC4ME_BUILD_SHA");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let sha = std::env::var("DOC4ME_BUILD_SHA").unwrap_or_default();
    let sha = sha.trim();
    let commit = if (7..=64).contains(&sha.len()) && sha.bytes().all(|b| b.is_ascii_hexdigit()) { &sha[..sha.len().min(8)] } else { "dev" };
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/doc4me.ico")
        .set("ProductName", "Doc4Me")
        .set("FileDescription", "Doc4Me word processor")
        .set("CompanyName", "Doc4Me contributors")
        .set("LegalCopyright", "Copyright (c) 2026 ArtCraft Team and the WordCraft contributors, and the Doc4Me contributors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "doc4me.exe")
        .set("InternalName", "doc4me")
        .set("FileVersion", commit)
        .set("ProductVersion", commit);
    if let Err(e) = res.compile() {
        if std::env::var_os("DOC4ME_REQUIRE_WINRES").is_some() {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=doc4me.exe built without icon/version resources: {e}");
    }
}
