//! The system clipboard in more formats than text: what Copy offers other programs and other
//! Doc4Me windows, and the HTML and RTF Paste reads (see `doc4me_ui_egui::system_clipboard`).
//!
//! Windows takes every format in one go (text, HTML Format, Rich Text Format, PNG and a bitmap),
//! through `clipboard-win`. macOS uses `arboard` for HTML and text and adds RTF through AppKit;
//! Linux and the BSDs use `arboard` for HTML and text. The HTML carries pictures (`data:` URIs)
//! and the lossless copy.

use doc4me_engine::clip_transfer::{ExportOptions, SystemClip};
use doc4me_ui_egui::system_clipboard::{RichClipboard, SystemClipboard};

/// The service handed to the UI.
pub fn service() -> SystemClipboard {
    SystemClipboard {
        formats: ExportOptions { rtf: cfg!(any(target_os = "windows", target_os = "macos")), png: cfg!(target_os = "windows") },
        set: Box::new(set),
        get: Box::new(get),
    }
}

#[cfg(not(target_os = "windows"))]
thread_local! {
    /// One clipboard for the app's life: on X11 the copy is served by this connection, and a
    /// connection made per copy could drop it before another program asks.
    static CLIPBOARD: std::cell::RefCell<Option<arboard::Clipboard>> = const { std::cell::RefCell::new(None) };
}

#[cfg(not(target_os = "windows"))]
fn with_clipboard<T>(f: impl FnOnce(&mut arboard::Clipboard) -> Result<T, arboard::Error>) -> Result<T, String> {
    CLIPBOARD.with(|c| {
        let mut c = c.try_borrow_mut().map_err(|_| "the clipboard is busy".to_string())?;
        if c.is_none() {
            *c = Some(arboard::Clipboard::new().map_err(|e| e.to_string())?);
        }
        let clipboard = c.as_mut().ok_or("no clipboard")?;
        f(clipboard).map_err(|e| e.to_string())
    })
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn set(clip: &SystemClip) -> Result<(), String> {
    with_clipboard(|c| c.set_html(clip.html.as_str(), Some(clip.text.as_str())))
}

#[cfg(target_os = "macos")]
fn set(clip: &SystemClip) -> Result<(), String> {
    with_clipboard(|c| c.set_html(clip.html.as_str(), Some(clip.text.as_str())))?;
    if let Some(rtf) = clip.rtf.as_deref().filter(|rtf| !rtf.is_empty()) {
        macos_rtf::write(rtf);
    }
    Ok(())
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn get() -> RichClipboard {
    // Another program's clipboard may hold anything: arboard reads it as UTF-8 text or fails.
    RichClipboard { html: with_clipboard(|c| c.get().html()).ok(), rtf: None }
}

#[cfg(target_os = "macos")]
fn get() -> RichClipboard {
    // arboard reads HTML; AppKit provides the separate `public.rtf` pasteboard type.
    let html = with_clipboard(|c| c.get().html()).ok();
    let rtf = macos_rtf::read()
        .filter(|bytes| bytes.len() <= doc4me_engine::cmd::paste::MAX_PASTE_BYTES)
        .map(|bytes| String::from_utf8_lossy(&bytes).trim_end_matches('\0').to_string())
        .filter(|s| s.trim_start().starts_with("{\\rtf"));
    RichClipboard { html, rtf }
}

/// Largest clipboard format read (HTML may carry a Doc4Me copy).
#[cfg(target_os = "windows")]
const MAX_READ: usize = doc4me_engine::clip_transfer::MAX_HTML_BYTES;

#[cfg(target_os = "windows")]
fn set(clip: &SystemClip) -> Result<(), String> {
    use clipboard_win::{options::NoClear, raw, register_format};
    let _open = clipboard_win::Clipboard::new_attempts(10).map_err(|e| format!("couldn't open the clipboard: {e}"))?;
    raw::empty().map_err(|e| format!("couldn't empty the clipboard: {e}"))?;
    // Programs take the first format they understand: the richest first.
    if let Some(html) = register_format("HTML Format") {
        raw::set_html(html.get(), &clip.html).map_err(|e| format!("HTML: {e}"))?;
    }
    if let (Some(rtf), Some(f)) = (&clip.rtf, register_format("Rich Text Format")) {
        let mut bytes = rtf.as_bytes().to_vec();
        bytes.push(0);
        raw::set_without_clear(f.get(), &bytes).map_err(|e| format!("RTF: {e}"))?;
    }
    if let Some(png) = &clip.png {
        if let Some(f) = register_format("PNG") {
            raw::set_without_clear(f.get(), png).map_err(|e| format!("PNG: {e}"))?;
        }
        // A bitmap for programs that read nothing else; without one they still have the PNG.
        if let Some(bmp) = doc4me_engine::clip_transfer::bmp_from_png(png)
            && let Err(e) = raw::set_bitmap_with(&bmp, NoClear)
        {
            log::warn!("clipboard: no bitmap ({e})");
        }
    }
    let text = clip.text.replace("\r\n", "\n").replace('\n', "\r\n");
    raw::set_string_with(&text, NoClear).map_err(|e| format!("text: {e}"))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn get() -> RichClipboard {
    use clipboard_win::{raw, register_format};
    let Ok(_open) = clipboard_win::Clipboard::new_attempts(10) else { return RichClipboard::default() };
    let fits = |f: u32| raw::is_format_avail(f) && raw::size(f).is_some_and(|n| n.get() <= MAX_READ);
    let html = register_format("HTML Format").map(|f| f.get()).filter(|&f| fits(f)).and_then(|f| {
        let mut out = Vec::new();
        raw::get_html(f, &mut out).ok()?;
        String::from_utf8(out).ok()
    });
    let rtf = register_format("Rich Text Format").map(|f| f.get()).filter(|&f| fits(f)).and_then(|f| {
        let mut out = Vec::new();
        raw::get_vec(f, &mut out).ok()?;
        // RTF is 7-bit text ending with a NUL; anything else isn't RTF.
        let end = out.iter().position(|&b| b == 0).unwrap_or(out.len());
        out.truncate(end);
        String::from_utf8(out).ok()
    });
    RichClipboard { html, rtf }
}

#[cfg(target_os = "macos")]
mod macos_rtf {
    use objc2_app_kit::NSPasteboard;
    use objc2_foundation::{NSData, NSString};

    const RTF_TYPE: &str = "public.rtf";

    /// RTF from TextEdit, Pages, Word for Mac and other pasteboard clients.
    pub fn read() -> Option<Vec<u8>> {
        let data = NSPasteboard::generalPasteboard().dataForType(&NSString::from_str(RTF_TYPE))?;
        Some(data.to_vec())
    }

    /// Add RTF beside the HTML and text arboard has just written. Failure leaves that copy usable.
    pub fn write(rtf: &str) {
        let pasteboard = NSPasteboard::generalPasteboard();
        if !pasteboard.setData_forType(Some(&NSData::with_bytes(rtf.as_bytes())), &NSString::from_str(RTF_TYPE)) {
            log::warn!("clipboard: the RTF couldn't be added");
        }
    }
}
