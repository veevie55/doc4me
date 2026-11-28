//! KrubOffice mouse pointers: the arrow drawn as one of eight original pictures instead of the
//! system's. Only the plain arrow changes; every other pointer (text I-beam, hand, resize arrows)
//! stays the system's. The same file is in KhianKrub, KhitKrub, PhutKrub and the KrubOffice Start
//! Center, so all four offer the same choice.
//!
//! egui can't hand the system a picture for its pointer, so while the arrow would show, the
//! system pointer is hidden and the picture is painted at the pointer on egui's topmost layer.
//! Call [`paint`] once at the very end of each frame, after everything that may set the cursor.
//!
//! The pictures (`assets/pointers/*.png`, 96 × 96, three pixels per point) are original work made
//! for KrubOffice; see ATTRIBUTION.md.

use egui::{Color32, Context, CursorIcon, Id, LayerId, Order, Pos2, Rect, TextureHandle, TextureOptions, Vec2, pos2, vec2};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The pointer the person chose. `System` (the default) leaves the system's arrow alone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Pointer {
    #[default]
    System,
    White,
    Black,
    Gold,
    Large,
    Cat,
    Rabbit,
    Chick,
    Elephant,
}

/// The size of a pointer picture in points: the pictures are drawn on a 32 × 32 grid.
const SIZE: f32 = 32.0;

impl Pointer {
    /// In menu order.
    pub const ALL: [Pointer; 9] = [
        Pointer::System,
        Pointer::White,
        Pointer::Black,
        Pointer::Gold,
        Pointer::Large,
        Pointer::Cat,
        Pointer::Rabbit,
        Pointer::Chick,
        Pointer::Elephant,
    ];

    /// The value saved in the settings and taken by the `pointer` command.
    pub fn code(self) -> &'static str {
        match self {
            Pointer::System => "system",
            Pointer::White => "white",
            Pointer::Black => "black",
            Pointer::Gold => "gold",
            Pointer::Large => "large",
            Pointer::Cat => "cat",
            Pointer::Rabbit => "rabbit",
            Pointer::Chick => "chick",
            Pointer::Elephant => "elephant",
        }
    }

    pub fn from_code(code: &str) -> Option<Pointer> {
        Pointer::ALL.into_iter().find(|p| p.code().eq_ignore_ascii_case(code.trim()))
    }

    /// The English label (translated where shown).
    pub fn label(self) -> &'static str {
        match self {
            Pointer::System => "System pointer",
            Pointer::White => "White arrow",
            Pointer::Black => "Black arrow",
            Pointer::Gold => "Gold arrow",
            Pointer::Large => "Large arrow",
            Pointer::Cat => "Cat",
            Pointer::Rabbit => "Rabbit",
            Pointer::Chick => "Chick",
            Pointer::Elephant => "Elephant",
        }
    }

    /// The picture (PNG), or `None` for the system pointer.
    fn png(self) -> Option<&'static [u8]> {
        Some(match self {
            Pointer::System => return None,
            Pointer::White => include_bytes!("../../../assets/pointers/classic-white.png"),
            Pointer::Black => include_bytes!("../../../assets/pointers/classic-black.png"),
            Pointer::Gold => include_bytes!("../../../assets/pointers/gold.png"),
            Pointer::Large => include_bytes!("../../../assets/pointers/big-white.png"),
            Pointer::Cat => include_bytes!("../../../assets/pointers/animal-cat.png"),
            Pointer::Rabbit => include_bytes!("../../../assets/pointers/animal-rabbit.png"),
            Pointer::Chick => include_bytes!("../../../assets/pointers/animal-chick.png"),
            Pointer::Elephant => include_bytes!("../../../assets/pointers/animal-elephant.png"),
        })
    }

    /// Where the click lands, in points from the picture's top-left corner.
    fn hotspot(self) -> Vec2 {
        match self {
            Pointer::White | Pointer::Black | Pointer::Gold => vec2(4.0, 3.0),
            Pointer::Large => vec2(3.0, 2.0),
            _ => vec2(2.0, 2.0),
        }
    }

    /// The picture as a texture, decoded once per context and kept in egui's memory. `None` for
    /// the system pointer, or if the picture can't be decoded (then the system arrow shows).
    pub fn texture(self, ctx: &Context) -> Option<TextureHandle> {
        let png = self.png()?;
        let id = Id::new(("krub_pointer", self.code()));
        if let Some(cached) = ctx.data(|d| d.get_temp::<Option<TextureHandle>>(id)) {
            return cached;
        }
        let texture = image::load_from_memory_with_format(png, image::ImageFormat::Png).ok().map(|img| {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
            ctx.load_texture(format!("krub-pointer-{}", self.code()), image, TextureOptions::LINEAR)
        });
        ctx.data_mut(|d| d.insert_temp(id, texture.clone()));
        texture
    }

    /// A small picture of the pointer for menus, `side` points square.
    pub fn preview(self, ui: &mut egui::Ui, side: f32) {
        let (rect, _) = ui.allocate_exact_size(vec2(side, side), egui::Sense::hover());
        if let Some(texture) = self.texture(ui.ctx()) {
            ui.painter().image(texture.id(), rect, full_uv(), Color32::WHITE);
        }
    }
}

fn full_uv() -> Rect {
    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0))
}

impl Serialize for Pointer {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.code())
    }
}

/// An unknown or damaged saved value reads as the system pointer.
impl<'de> Deserialize<'de> for Pointer {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = serde_json::Value::deserialize(d)?;
        Ok(v.as_str().and_then(Pointer::from_code).unwrap_or_default())
    }
}

/// Paint the chosen pointer for this frame. Call after all of the frame's widgets, so the cursor
/// they asked for is known. Does nothing for the system pointer, when the pointer is outside the
/// window, or when a widget asked for any pointer other than the plain arrow.
pub fn paint(ctx: &Context, pointer: Pointer) {
    if pointer == Pointer::System {
        return;
    }
    if ctx.output(|o| o.cursor_icon) != CursorIcon::Default {
        return;
    }
    let Some(pos) = ctx.input(|i| if i.any_touches() { None } else { i.pointer.hover_pos() }) else {
        return;
    };
    let Some(texture) = pointer.texture(ctx) else {
        return;
    };
    ctx.set_cursor_icon(CursorIcon::None);
    // A pointer doesn't grow with the interface size: keep it at the system's scale.
    let zoom = ctx.zoom_factor();
    let scale = if zoom.is_finite() && zoom > 0.0 { 1.0 / zoom } else { 1.0 };
    let min: Pos2 = pos - pointer.hotspot() * scale;
    let rect = Rect::from_min_size(min, Vec2::splat(SIZE * scale));
    let painter = ctx.layer_painter(LayerId::new(Order::Debug, Id::new("krub_pointer_layer")));
    painter.image(texture.id(), rect, full_uv(), Color32::WHITE);
}
