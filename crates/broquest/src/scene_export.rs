//! Exports what the window paints as a vector snapshot, for the website's
//! motion demos. Built only with the `scene-export` feature.
//!
//! gpui hands every frame to the platform window as a [`gpui::Scene`]: quads,
//! shadows, paths and sprites in device pixels. Text and icons arrive as atlas
//! sprites, so the platform, its text system and its sprite atlas are wrapped
//! to remember which glyph or icon each atlas tile holds. A snapshot then
//! names the characters, fonts and icon paths instead of pixels; the icons
//! themselves are read from the assets when the snapshot is rendered.
//!
//! [`wrap`] wraps the platform the app runs on. [`request`] asks for the next
//! frame to be written as JSON; `script/motion` turns that into SVG.

mod dump;
mod platform;
mod text;
mod window;

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{LazyLock, Mutex};

use gpui::{
    AtlasKey, AtlasTextureKind, AtlasTile, Bounds, DevicePixels, Font, FontId, GlyphId, Platform,
};

pub(crate) use platform::ExportPlatform;

/// Wraps `inner` so that its windows can export their frames.
pub(crate) fn wrap(inner: Rc<dyn Platform>) -> Rc<dyn Platform> {
    Rc::new(ExportPlatform::new(inner))
}

/// Asks for the next frame drawn to be written to `path`. Returns a receiver
/// that resolves once the file is written, or with the error that stopped it.
pub(crate) fn request(path: PathBuf) -> futures::channel::oneshot::Receiver<anyhow::Result<()>> {
    let (sender, receiver) = futures::channel::oneshot::channel();
    RECORDER.lock().unwrap().request = Some((path, sender));
    receiver
}

type Request = (
    PathBuf,
    futures::channel::oneshot::Sender<anyhow::Result<()>>,
);

/// What the wrappers learn from the text system and the atlas, kept for the
/// life of the process: gpui caches glyph layouts and atlas tiles, so each is
/// only seen once.
#[derive(Default)]
struct Recorder {
    request: Option<Request>,
    /// The fonts asked for that resolved to each font; a missing family can
    /// resolve to the fallback another family resolves to.
    fonts: HashMap<FontId, Vec<Font>>,
    /// The families asked for that are not installed, which gpui drew in a
    /// fallback instead.
    missing_families: Vec<String>,
    /// Where the window asked macOS to put its traffic lights, which the
    /// system draws outside the scene.
    traffic_lights: Option<gpui::Point<gpui::Pixels>>,
    /// The text each glyph of a font was shaped from.
    glyph_text: HashMap<(FontId, GlyphId), String>,
    /// Each glyph's advance, as a fraction of the font size.
    advances: HashMap<(FontId, GlyphId), f32>,
    /// Glyph raster bounds, keyed by [`glyph_key`].
    raster_bounds: HashMap<GlyphKey, Bounds<DevicePixels>>,
    /// What each atlas tile holds, keyed by [`tile_key`].
    tiles: HashMap<TileKey, AtlasKey>,
}

static RECORDER: LazyLock<Mutex<Recorder>> = LazyLock::new(Default::default);

type TileKey = (u32, u8, u32);

fn tile_key(tile: &AtlasTile) -> TileKey {
    let kind = match tile.texture_id.kind {
        AtlasTextureKind::Monochrome => 0,
        AtlasTextureKind::Polychrome => 1,
        AtlasTextureKind::Subpixel => 2,
    };
    (tile.texture_id.index, kind, tile.tile_id.0)
}

/// A glyph's raster parameters that its raster bounds depend on.
type GlyphKey = (FontId, GlyphId, u32, (u8, u8), u32, bool);

fn glyph_key(params: &gpui::RenderGlyphParams) -> GlyphKey {
    (
        params.font_id,
        params.glyph_id,
        f32::from(params.font_size).to_bits(),
        (params.subpixel_variant.x, params.subpixel_variant.y),
        params.scale_factor.to_bits(),
        params.is_emoji,
    )
}
