//! The main window's size, position and maximized state, saved with the UI preferences so the
//! next launch opens the window where the user left it (#23).
//!
//! Sizes and positions are in egui points: what [`egui::ViewportInfo`] reports and what
//! [`egui::ViewportBuilder`] takes.

use serde::{Deserialize, Serialize};

/// The smallest window restored: the desktop app's minimum inner size.
pub const MIN_SIZE: egui::Vec2 = egui::vec2(760.0, 480.0);
/// Larger saved sizes are corrupt. eframe also shrinks the start size to the largest monitor.
const MAX_SIZE: egui::Vec2 = egui::vec2(16384.0, 16384.0);
/// Saved positions farther from the origin than this are corrupt.
const MAX_COORD: f32 = 32768.0;
/// A saved pixels-per-point value outside this range is corrupt or unusable for monitor comparisons.
const MAX_MONITOR_SCALE: f32 = 16.0;

/// The window as it was when the app last closed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowGeometry {
    /// Top-left corner: of the frame, or of the content on macOS (where winit places a new
    /// window by it). `None` where the platform doesn't report it (Wayland).
    pub position: Option<[f32; 2]>,
    /// Content size of the un-maximized window.
    pub width: f32,
    pub height: f32,
    pub maximized: bool,
    /// Size of the monitor the window was on, to notice when that monitor is gone.
    pub monitor: Option<[f32; 2]>,
    /// Effective pixels per point used to measure `monitor`. Paired with `monitor` to compare
    /// physical dimensions when the display DPI or egui interface zoom changes. Older preferences
    /// omit it.
    pub monitor_pixels_per_point: Option<f32>,
}

impl WindowGeometry {
    /// The geometry to save, given the last saved one and the window as it is now. While the
    /// window is minimized, full screen or maximized, the last un-maximized size and position are
    /// kept, so un-maximizing after the next launch returns to them.
    pub fn track(prev: Option<Self>, info: &egui::ViewportInfo, content_size: egui::Vec2, pixels_per_point: f32) -> Option<Self> {
        if info.minimized == Some(true) || info.fullscreen == Some(true) {
            return prev;
        }
        // egui reads the maximized state on macOS only when the window opens (egui#3494), so a
        // window that opened zoomed would stay "maximized" for good. There the zoomed frame is
        // saved like any other size instead.
        let maximized = info.maximized == Some(true) && !cfg!(target_os = "macos");
        if maximized && let Some(prev) = prev {
            return Some(Self { maximized, ..prev });
        }
        let rect = if cfg!(target_os = "macos") { info.inner_rect } else { info.outer_rect };
        let size = info.inner_rect.map_or(content_size, |r| r.size());
        let position = rect.map(|r| [r.min.x, r.min.y]);
        let monitor = info.monitor_size.map(|s| [s.x, s.y]);
        Self { position, width: size.x, height: size.y, maximized, monitor, monitor_pixels_per_point: Some(pixels_per_point) }.sanitized().or(prev)
    }

    /// `None` for a size that isn't positive and finite. Otherwise the size is clamped to
    /// [`MIN_SIZE`]..=16384, and invalid position, monitor size or pixels-per-point value are dropped.
    pub fn sanitized(self) -> Option<Self> {
        let size = egui::vec2(self.width, self.height);
        if !size.is_finite() || size.x <= 0.0 || size.y <= 0.0 {
            return None;
        }
        let size = size.clamp(MIN_SIZE, MAX_SIZE);
        let position = self.position.filter(|p| p.iter().all(|c| c.is_finite() && c.abs() <= MAX_COORD));
        let monitor = self.monitor.filter(|m| m.iter().all(|c| c.is_finite() && *c > 0.0 && *c <= MAX_SIZE.x));
        let monitor_pixels_per_point = self.monitor_pixels_per_point.filter(|s| s.is_finite() && *s > 0.0 && *s <= MAX_MONITOR_SCALE);
        Some(Self { position, width: size.x, height: size.y, maximized: self.maximized, monitor, monitor_pixels_per_point })
    }

    /// Open the window at this size, position and maximized state.
    pub fn apply(self, builder: egui::ViewportBuilder) -> egui::ViewportBuilder {
        let builder = builder.with_inner_size([self.width, self.height]).with_maximized(self.maximized);
        match self.position {
            Some(p) => builder.with_position(p),
            None => builder,
        }
    }

    /// After a restore: where to move the window when the monitor it now reports differs from the
    /// one it was saved on (an external monitor was unplugged, say), so a window restored to a
    /// position no monitor covers any more doesn't open off screen. Centred on the primary
    /// monitor, whose top-left corner is the origin. A maximized window fills whichever monitor
    /// the system picks, so it stays put.
    pub fn rescue_position(&self, info: &egui::ViewportInfo, pixels_per_point: f32) -> Option<egui::Pos2> {
        if self.maximized || self.position.is_none() {
            return None;
        }
        let saved = egui::Vec2::from(self.monitor?);
        let now = info.monitor_size.filter(|s| s.is_finite() && s.x > 0.0 && s.y > 0.0)?;
        // `monitor_size` is in points and changes when either the OS display scale or egui zoom
        // changes. Compare physical pixels using the same effective scale that egui-winit uses to
        // convert the monitor size, so a scale-only change doesn't look like a different monitor.
        let same_monitor_size = match (
            self.monitor_pixels_per_point.filter(|s| s.is_finite() && *s > 0.0 && *s <= MAX_MONITOR_SCALE),
            valid_pixels_per_point(pixels_per_point),
        ) {
            (Some(saved_scale), Some(now_scale)) => (saved * saved_scale - now * now_scale).length() <= 1.0,
            _ => (now - saved).length() <= 1.0,
        };
        if same_monitor_size {
            return None;
        }
        let free = now - egui::vec2(self.width, self.height);
        Some(egui::pos2((free.x / 2.0).max(0.0), (free.y / 2.0).max(0.0)))
    }
}

/// Checks the restored geometry in `pending` once, and only once the monitor size is known: while
/// `info` doesn't report a usable one, `pending` is left in place for a later frame. Then it is taken
/// and [`WindowGeometry::rescue_position`] decides where to move the window, if anywhere.
pub fn take_rescue(pending: &mut Option<WindowGeometry>, info: &egui::ViewportInfo, pixels_per_point: f32) -> Option<egui::Pos2> {
    info.monitor_size.filter(|s| s.is_finite() && s.x > 0.0 && s.y > 0.0)?;
    pending.take()?.rescue_position(info, pixels_per_point)
}

fn valid_pixels_per_point(pixels_per_point: f32) -> Option<f32> {
    (pixels_per_point.is_finite() && pixels_per_point > 0.0 && pixels_per_point <= MAX_MONITOR_SCALE).then_some(pixels_per_point)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(rect: egui::Rect, monitor: egui::Vec2) -> egui::ViewportInfo {
        info_scaled(rect, monitor, 1.0)
    }

    fn info_scaled(rect: egui::Rect, monitor: egui::Vec2, scale: f32) -> egui::ViewportInfo {
        egui::ViewportInfo {
            inner_rect: Some(rect),
            outer_rect: Some(rect),
            monitor_size: Some(monitor),
            native_pixels_per_point: Some(scale),
            ..Default::default()
        }
    }

    fn saved() -> WindowGeometry {
        WindowGeometry {
            position: Some([100.0, 50.0]),
            width: 1200.0,
            height: 800.0,
            maximized: false,
            monitor: Some([1920.0, 1080.0]),
            monitor_pixels_per_point: Some(1.0),
        }
    }

    #[test]
    fn records_the_window_and_restores_it() {
        let rect = egui::Rect::from_min_size(egui::pos2(100.0, 50.0), egui::vec2(1200.0, 800.0));
        let g = WindowGeometry::track(None, &info(rect, egui::vec2(1920.0, 1080.0)), egui::Vec2::ZERO, 1.0);
        assert_eq!(g, Some(saved()));
        let b = saved().apply(egui::ViewportBuilder::default());
        assert_eq!(b.inner_size, Some(egui::vec2(1200.0, 800.0)));
        assert_eq!(b.position, Some(egui::pos2(100.0, 50.0)));
        assert_eq!(b.maximized, Some(false));
    }

    #[test]
    fn maximized_keeps_the_normal_size_and_position() {
        let full = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1920.0, 1040.0));
        let mut i = info(full, egui::vec2(1920.0, 1080.0));
        i.maximized = Some(true);
        let g = WindowGeometry::track(Some(saved()), &i, egui::Vec2::ZERO, 1.0);
        if cfg!(target_os = "macos") {
            // The zoomed frame is saved as the size; the window never reopens "maximized".
            assert_eq!(g.map(|g| (g.position, g.width, g.height, g.maximized)), Some((Some([0.0, 0.0]), 1920.0, 1040.0, false)));
        } else {
            assert_eq!(g, Some(WindowGeometry { maximized: true, ..saved() }));
        }
        // Minimized or full screen: nothing changes.
        i.maximized = Some(false);
        i.minimized = Some(true);
        assert_eq!(WindowGeometry::track(Some(saved()), &i, egui::Vec2::ZERO, 1.0), Some(saved()));
        i.minimized = None;
        i.fullscreen = Some(true);
        assert_eq!(WindowGeometry::track(Some(saved()), &i, egui::Vec2::ZERO, 1.0), Some(saved()));
    }

    #[test]
    fn without_a_position_the_content_size_is_kept() {
        // Wayland reports neither rect.
        let i = egui::ViewportInfo::default();
        let g = WindowGeometry::track(None, &i, egui::vec2(1000.0, 700.0), 1.0);
        assert_eq!(
            g,
            Some(WindowGeometry {
                position: None,
                width: 1000.0,
                height: 700.0,
                maximized: false,
                monitor: None,
                monitor_pixels_per_point: Some(1.0)
            })
        );
        assert_eq!(g.map(|g| g.apply(egui::ViewportBuilder::default()).position), Some(None));
    }

    #[test]
    fn insane_values_are_clamped_or_dropped() {
        let zero = WindowGeometry { width: 0.0, height: 800.0, ..saved() };
        assert_eq!(zero.sanitized(), None);
        assert_eq!(WindowGeometry::default().sanitized(), None);
        let nan = WindowGeometry { width: f32::NAN, ..saved() };
        assert_eq!(nan.sanitized(), None);
        let tiny = WindowGeometry { width: 10.0, height: 1.0e9, ..saved() }.sanitized();
        assert_eq!(tiny.map(|g| (g.width, g.height)), Some((MIN_SIZE.x, 16384.0)));
        let far = WindowGeometry { position: Some([1.0e7, 0.0]), monitor: Some([f32::INFINITY, 1080.0]), ..saved() }.sanitized();
        assert_eq!(far.map(|g| (g.position, g.monitor)), Some((None, None)));
        let bad_scale = WindowGeometry { monitor_pixels_per_point: Some(f32::NAN), ..saved() }.sanitized();
        assert_eq!(bad_scale.and_then(|g| g.monitor_pixels_per_point), None);
        assert_eq!(saved().sanitized(), Some(saved()));
    }

    #[test]
    fn a_window_whose_monitor_is_gone_is_moved_on_screen() {
        let on = egui::Rect::from_min_size(egui::pos2(2000.0, 50.0), egui::vec2(1200.0, 800.0));
        // Same monitor: stays where it was.
        assert_eq!(saved().rescue_position(&info(on, egui::vec2(1920.0, 1080.0)), 1.0), None);
        // A different (smaller) monitor now: centred on it.
        assert_eq!(saved().rescue_position(&info(on, egui::vec2(1600.0, 900.0)), 1.0), Some(egui::pos2(200.0, 50.0)));
        // Larger than that monitor: its top-left corner.
        assert_eq!(saved().rescue_position(&info(on, egui::vec2(1024.0, 768.0)), 1.0), Some(egui::Pos2::ZERO));
        // Nothing to compare, or no saved position: no move.
        assert_eq!(WindowGeometry { monitor: None, ..saved() }.rescue_position(&info(on, egui::vec2(1600.0, 900.0)), 1.0), None);
        assert_eq!(WindowGeometry { position: None, ..saved() }.rescue_position(&info(on, egui::vec2(1600.0, 900.0)), 1.0), None);
        assert_eq!(WindowGeometry { maximized: true, ..saved() }.rescue_position(&info(on, egui::vec2(1600.0, 900.0)), 1.0), None);
        assert_eq!(saved().rescue_position(&egui::ViewportInfo::default(), 1.0), None);
    }

    #[test]
    fn a_dpi_change_on_the_same_physical_monitor_does_not_rescue_the_window() {
        let on = egui::Rect::from_min_size(egui::pos2(2000.0, 50.0), egui::vec2(1200.0, 800.0));
        // 1920×1080 points at 100% and 1280×720 points at 150% describe the same display pixels.
        let current = info_scaled(on, egui::vec2(1280.0, 720.0), 1.5);
        assert_eq!(saved().rescue_position(&current, 1.5), None);
    }

    #[test]
    fn a_different_physical_monitor_size_still_rescues_at_a_different_dpi() {
        let on = egui::Rect::from_min_size(egui::pos2(2000.0, 50.0), egui::vec2(1200.0, 800.0));
        // The logical size is unchanged, but at 150% the physical display is larger.
        let current = info_scaled(on, egui::vec2(1920.0, 1080.0), 1.5);
        assert_eq!(saved().rescue_position(&current, 1.5), Some(egui::pos2(360.0, 140.0)));
    }

    #[test]
    fn a_ui_zoom_change_on_the_same_physical_monitor_does_not_rescue_the_window() {
        let on = egui::Rect::from_min_size(egui::pos2(2000.0, 50.0), egui::vec2(1200.0, 800.0));
        // Native DPI stays at 100%, but 125% egui interface zoom converts this monitor to fewer
        // points. The effective pixels-per-point value must be used instead of the native scale.
        let current = info_scaled(on, egui::vec2(1536.0, 864.0), 1.0);
        let tracked = WindowGeometry::track(Some(saved()), &current, egui::Vec2::ZERO, 1.25).unwrap();
        assert_eq!(tracked.monitor_pixels_per_point, Some(1.25));
        assert_eq!(saved().rescue_position(&current, 1.25), None);
    }

    #[test]
    fn the_rescue_waits_for_the_monitor_size() {
        let on = egui::Rect::from_min_size(egui::pos2(2000.0, 50.0), egui::vec2(1200.0, 800.0));
        // The first frame doesn't know the monitor yet: nothing happens and the geometry is kept.
        let mut pending = Some(saved());
        assert_eq!(take_rescue(&mut pending, &egui::ViewportInfo::default(), 1.0), None);
        assert_eq!(pending, Some(saved()));
        // A later frame reports a different monitor: centred on it, and the check is done.
        assert_eq!(take_rescue(&mut pending, &info(on, egui::vec2(1600.0, 900.0)), 1.0), Some(egui::pos2(200.0, 50.0)));
        assert_eq!(pending, None);
        assert_eq!(take_rescue(&mut pending, &info(on, egui::vec2(1024.0, 768.0)), 1.0), None);
        // The same monitor: no move, and the check is done as well.
        let mut pending = Some(saved());
        assert_eq!(take_rescue(&mut pending, &info(on, egui::vec2(1920.0, 1080.0)), 1.0), None);
        assert_eq!(pending, None);
    }

    #[test]
    fn survives_the_prefs_round_trip() {
        let ui = crate::UiState { window: Some(saved()), ..Default::default() };
        let json = serde_json::to_string(&ui).unwrap();
        let back: crate::UiState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.window, Some(saved()));
        // Prefs written before #23 have no window.
        let old: crate::UiState = serde_json::from_str(r#"{"tab":"Insert"}"#).unwrap();
        assert_eq!(old.window, None);
        assert_eq!(old.tab, "Insert");

        // Window preferences written before monitor pixels-per-point tracking remain readable.
        let old_window: WindowGeometry =
            serde_json::from_str(r#"{"position":[100.0,50.0],"width":1200.0,"height":800.0,"maximized":false,"monitor":[1920.0,1080.0]}"#).unwrap();
        assert_eq!(old_window.monitor_pixels_per_point, None);
    }
}
