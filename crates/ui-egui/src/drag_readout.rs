//! Position and size labels for floating objects while dragging.

use doc4me_geom::Unit;
use egui::{Color32, Painter, Pos2, Rect, vec2};

pub fn readout_text(unit: Unit, resizing: bool, r: doc4me_geom::Rect) -> String {
    let n = |pt: f32| unit.format_with(pt, crate::options::decimal());
    if resizing {
        format!("{} {}   {} {}", tl!("W:"), n(r.w), tl!("H:"), n(r.h))
    } else {
        format!("{} {}   {} {}", tl!("Left:"), n(r.x), tl!("Top:"), n(r.y))
    }
}

pub fn paint_readout(painter: &Painter, text: String, frame: Rect, page: Rect) {
    let galley = painter.layout_no_wrap(text, egui::FontId::proportional(12.0), Color32::WHITE);
    let (pad, gap) = (5.0, 6.0);
    let size = galley.size() + vec2(pad * 2.0, pad);
    let mut min = Pos2::new(frame.min.x, frame.max.y + gap);
    if min.y + size.y > page.max.y {
        min.y = (frame.min.y - gap - size.y).max(page.min.y);
    }
    min.x = min.x.min(page.max.x - size.x).max(page.min.x);
    let rect = Rect::from_min_size(min, size);
    painter.rect_filled(rect, egui::CornerRadius::same(4), Color32::from_rgba_unmultiplied(28, 28, 32, 232));
    painter.galley(rect.min + vec2(pad, pad / 2.0), galley, Color32::WHITE);
}

pub fn menu_lines(app: &mut crate::WordApp) -> Option<[String; 2]> {
    let (pos, _) = doc4me_engine::cmd::objects::selected(&app.session)?;
    let layout = app.session.layout();
    let object = layout.object(&pos, app.session.page_hint)?;
    let page = layout.pages.get(object.page)?;
    let unit = crate::options::unit();
    let page_width = (page.w - page.markup).max(0.0);
    Some([
        readout_text(unit, false, object.rect),
        format!(
            "{} {}   {} {}",
            tl!("Right:"),
            unit.format_with(page_width - object.rect.x - object.rect.w, crate::options::decimal()),
            tl!("Bottom:"),
            unit.format_with(page.h - object.rect.y - object.rect.h, crate::options::decimal())
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_and_sizes_use_the_selected_unit() {
        let cm = 72.0 / 2.54;
        let rect = doc4me_geom::Rect::new(1.5 * cm, 3.0 * cm, 10.0 * cm, 2.0 * cm);
        let position = readout_text(Unit::Centimeters, false, rect);
        let size = readout_text(Unit::Millimeters, true, rect);
        assert!(position.contains("1.5 cm") && position.contains("3 cm"), "{position}");
        assert!(size.contains("100 mm") && size.contains("20 mm"), "{size}");
    }
}
