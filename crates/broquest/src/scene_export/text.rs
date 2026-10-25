//! The text system and sprite atlas wrappers, which record what the scene's
//! glyph and icon sprites stand for.

use std::borrow::Cow;
use std::sync::Arc;

use anyhow::Result;
use gpui::{
    AtlasKey, AtlasTile, Bounds, DevicePixels, Font, FontId, FontMetrics, FontRun, GlyphId, Hsla,
    LineLayout, MissingGlyphSink, Pixels, PlatformAtlas, PlatformTextSystem, RenderGlyphParams,
    Size, TextRenderingMode,
};

use super::{RECORDER, glyph_key, tile_key};

pub(super) struct ExportTextSystem {
    inner: Arc<dyn PlatformTextSystem>,
}

impl ExportTextSystem {
    pub(super) fn new(inner: Arc<dyn PlatformTextSystem>) -> Self {
        Self { inner }
    }
}

impl PlatformTextSystem for ExportTextSystem {
    fn add_fonts(&self, fonts: Vec<Cow<'static, [u8]>>) -> Result<()> {
        self.inner.add_fonts(fonts)
    }

    fn set_missing_glyph_sink(&self, sink: Option<Arc<dyn MissingGlyphSink>>) {
        self.inner.set_missing_glyph_sink(sink)
    }

    fn all_font_names(&self) -> Vec<String> {
        self.inner.all_font_names()
    }

    fn font_id(&self, descriptor: &Font) -> Result<FontId> {
        let id = self.inner.font_id(descriptor);
        let mut recorder = RECORDER.lock().unwrap();
        let Ok(id) = id else {
            let family = descriptor.family.to_string();
            if !recorder.missing_families.contains(&family) {
                recorder.missing_families.push(family);
            }
            return id;
        };
        let fonts = recorder.fonts.entry(id).or_default();
        if !fonts.contains(descriptor) {
            fonts.push(descriptor.clone());
        }
        Ok(id)
    }

    fn prewarm_fonts(&self, font_ids: &[FontId]) {
        self.inner.prewarm_fonts(font_ids)
    }

    fn font_metrics(&self, font_id: FontId) -> FontMetrics {
        self.inner.font_metrics(font_id)
    }

    fn typographic_bounds(&self, font_id: FontId, glyph_id: GlyphId) -> Result<Bounds<f32>> {
        self.inner.typographic_bounds(font_id, glyph_id)
    }

    fn advance(&self, font_id: FontId, glyph_id: GlyphId) -> Result<Size<f32>> {
        self.inner.advance(font_id, glyph_id)
    }

    fn glyph_for_char(&self, font_id: FontId, ch: char) -> Option<GlyphId> {
        let glyph = self.inner.glyph_for_char(font_id, ch)?;
        RECORDER
            .lock()
            .unwrap()
            .glyph_text
            .entry((font_id, glyph))
            .or_insert_with(|| ch.to_string());
        Some(glyph)
    }

    fn glyph_raster_bounds(&self, params: &RenderGlyphParams) -> Result<Bounds<DevicePixels>> {
        let bounds = self.inner.glyph_raster_bounds(params)?;
        RECORDER
            .lock()
            .unwrap()
            .raster_bounds
            .insert(glyph_key(params), bounds);
        Ok(bounds)
    }

    fn rasterize_glyph(
        &self,
        params: &RenderGlyphParams,
        raster_bounds: Bounds<DevicePixels>,
    ) -> Result<(Size<DevicePixels>, Vec<u8>)> {
        self.inner.rasterize_glyph(params, raster_bounds)
    }

    fn layout_line(&self, text: &str, font_size: Pixels, runs: &[FontRun]) -> LineLayout {
        let layout = self.inner.layout_line(text, font_size, runs);
        let mut recorder = RECORDER.lock().unwrap();
        let size = f32::from(font_size);
        let glyphs = layout
            .runs
            .iter()
            .flat_map(|run| run.glyphs.iter().map(move |glyph| (run.font_id, glyph)))
            .collect::<Vec<_>>();
        for (ix, (font_id, glyph)) in glyphs.iter().enumerate() {
            let end = glyphs
                .get(ix + 1)
                .map_or(layout.width, |(_, next)| next.position.x);
            if size > 0. {
                recorder
                    .advances
                    .entry((*font_id, glyph.id))
                    .or_insert(f32::from(end - glyph.position.x) / size);
            }
        }
        for run in &layout.runs {
            for (ix, glyph) in run.glyphs.iter().enumerate() {
                // A glyph covers the text up to where the next one starts; a
                // ligature covers several characters.
                let end = run
                    .glyphs
                    .get(ix + 1)
                    .map(|next| next.index)
                    .filter(|&end| end > glyph.index)
                    .unwrap_or_else(|| next_char_boundary(text, glyph.index));
                if let Some(fragment) = text.get(glyph.index..end) {
                    recorder
                        .glyph_text
                        .entry((run.font_id, glyph.id))
                        .or_insert_with(|| fragment.to_string());
                }
            }
        }
        layout
    }

    fn recommended_rendering_mode(&self, font_id: FontId, font_size: Pixels) -> TextRenderingMode {
        self.inner.recommended_rendering_mode(font_id, font_size)
    }

    fn glyph_dilation_for_color(&self, color: Hsla) -> u8 {
        self.inner.glyph_dilation_for_color(color)
    }
}

fn next_char_boundary(text: &str, index: usize) -> usize {
    text.get(index..)
        .and_then(|rest| rest.chars().next())
        .map_or(text.len(), |ch| index + ch.len_utf8())
}

pub(super) struct ExportAtlas {
    inner: Arc<dyn PlatformAtlas>,
}

impl ExportAtlas {
    pub(super) fn new(inner: Arc<dyn PlatformAtlas>) -> Self {
        Self { inner }
    }
}

impl PlatformAtlas for ExportAtlas {
    fn get_or_insert_with<'a>(
        &self,
        key: AtlasKey,
        build: &mut dyn FnMut() -> Result<Option<(Size<DevicePixels>, Cow<'a, [u8]>)>>,
    ) -> Result<Option<AtlasTile>> {
        let tile = self.inner.get_or_insert_with(key.clone(), build)?;
        if let Some(tile) = &tile {
            RECORDER.lock().unwrap().tiles.insert(tile_key(tile), key);
        }
        Ok(tile)
    }

    fn remove(&self, key: &AtlasKey) {
        RECORDER
            .lock()
            .unwrap()
            .tiles
            .retain(|_, recorded| recorded != key);
        self.inner.remove(key)
    }
}
