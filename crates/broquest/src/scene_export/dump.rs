//! Turns a frame's [`Scene`] into the JSON snapshot `script/motion` reads.
//!
//! Positions are in logical pixels from the window's top left. Items come in
//! painting order: each is a rectangle, shadow, path, underline, text run,
//! icon or image, clipped to the rectangle in its `clip`.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::Path;

use anyhow::Context as _;
use gpui::{
    AtlasKey, Background, Bounds, Corners, Edges, Hsla, Pixels, Point, Rgba, ScaledPixels, Scene,
    Size, decode_layer_tile,
};
use serde_json::{Value, json};

use super::{Recorder, glyph_key, tile_key};

/// The horizontal positions a glyph can be rasterized at within a pixel; see
/// gpui's `SUBPIXEL_VARIANTS_X`.
const SUBPIXEL_VARIANTS_X: f32 = 4.;

pub(super) fn write(
    recorder: &Recorder,
    scene: &Scene,
    path: &Path,
    viewport: Size<Pixels>,
    scale: f32,
) -> anyhow::Result<()> {
    let mut dump = Dump {
        recorder,
        scale,
        fonts: BTreeMap::new(),
        icons: BTreeSet::new(),
    };
    let mut items = Vec::new();
    dump.scene(scene, Point::default(), None, &mut items);
    items.sort_by_key(|item| (item.order, item.rank));
    let items = merge_text(items.into_iter().map(|item| item.value).collect());

    let snapshot = json!({
        "version": 1,
        "width": f32::from(viewport.width),
        "height": f32::from(viewport.height),
        "scale": scale,
        "fonts": dump.fonts,
        // Text in these families came out in a fallback font.
        "missing_fonts": recorder.missing_families,
        // The asset paths of the icons drawn, which the renderer reads from
        // the app's sources.
        "icons": dump.icons,
        // The top left of the first macOS traffic light, if the window has
        // them; the system draws them, so they are not among the items.
        "traffic_lights": recorder
            .traffic_lights
            .map(|position| [f32::from(position.x), f32::from(position.y)]),
        "items": items,
    });
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_vec(&snapshot)?)
        .with_context(|| format!("writing {}", path.display()))
}

/// A tile a scroll layer is composited with: its draw order, its content
/// mask and the offset from the layer's content to the window.
type LayerTile = (u32, Bounds<ScaledPixels>, Point<ScaledPixels>);

struct Dump<'a> {
    recorder: &'a Recorder,
    scale: f32,
    fonts: BTreeMap<String, Value>,
    icons: BTreeSet<String>,
}

/// An item and where it sorts: by draw order, then by primitive kind as gpui
/// batches them, then by the order it was painted in.
struct Item {
    order: (u32, u32),
    rank: u8,
    value: Value,
}

impl Dump<'_> {
    /// Adds the primitives of `scene`, moved by `offset` (in scaled pixels)
    /// and clipped to `clip` as well as their own masks.
    fn scene(
        &mut self,
        scene: &Scene,
        offset: Point<ScaledPixels>,
        clip: Option<Bounds<ScaledPixels>>,
        items: &mut Vec<Item>,
    ) {
        let mut push = |order: u32, rank: u8, value: Option<Value>| {
            if let Some(value) = value {
                items.push(Item {
                    order: (order, 0),
                    rank,
                    value,
                });
            }
        };
        for shadow in &scene.shadows {
            let value = self.with_clip(
                json!({
                    "t": "shadow",
                    "b": self.bounds(shadow.bounds, offset),
                    "r": self.corners(shadow.corner_radii),
                    "blur": self.px(shadow.blur_radius),
                    "color": color(shadow.color),
                    "inset": shadow.inset != 0,
                    "eb": self.bounds(shadow.element_bounds, offset),
                    "er": self.corners(shadow.element_corner_radii),
                }),
                shadow.content_mask.bounds,
                offset,
                clip,
            );
            push(shadow.order, 0, Some(value));
        }
        for quad in &scene.quads {
            let fill = background(&quad.background);
            let border = quad.border_color.a > 0. && !edges_zero(&quad.border_widths);
            if fill.is_none() && !border {
                continue;
            }
            let mut value = json!({
                "t": "rect",
                "b": self.bounds(quad.bounds, offset),
                "r": self.corners(quad.corner_radii),
                "fill": fill,
            });
            if border {
                value["border"] = json!({
                    "color": color(quad.border_color),
                    "w": self.edges(quad.border_widths),
                    "style": format!("{:?}", quad.border_style).to_lowercase(),
                });
            }
            let value = self.with_clip(value, quad.content_mask.bounds, offset, clip);
            push(quad.order, 1, Some(value));
        }
        for path in &scene.paths {
            let mut d = String::new();
            for triangle in path.vertices.chunks_exact(3) {
                for (ix, vertex) in triangle.iter().enumerate() {
                    let p = vertex.xy_position + offset;
                    let command = if ix == 0 { 'M' } else { 'L' };
                    d.push_str(&format!(
                        "{command}{} {}",
                        round(p.x.0 / self.scale),
                        round(p.y.0 / self.scale)
                    ));
                }
                d.push('Z');
            }
            let value = self.with_clip(
                json!({ "t": "path", "d": d, "fill": background(&path.color) }),
                path.content_mask.bounds,
                offset,
                clip,
            );
            push(path.order, 2, Some(value));
        }
        for underline in &scene.underlines {
            let value = self.with_clip(
                json!({
                    "t": "underline",
                    "b": self.bounds(underline.bounds, offset),
                    "color": color(underline.color),
                    "thickness": self.px(underline.thickness),
                    // `PaddedBool32` keeps its value private; its debug form shows it.
                    "wavy": format!("{:?}", underline.wavy).contains('1'),
                }),
                underline.content_mask.bounds,
                offset,
                clip,
            );
            push(underline.order, 3, Some(value));
        }
        for sprite in &scene.monochrome_sprites {
            let value = self.mono_sprite(
                sprite.bounds,
                sprite.color,
                &sprite.tile,
                sprite.content_mask.bounds,
                offset,
                clip,
            );
            push(sprite.order, 4, value);
        }
        for sprite in &scene.subpixel_sprites {
            let value = self.mono_sprite(
                sprite.bounds,
                sprite.color,
                &sprite.tile,
                sprite.content_mask.bounds,
                offset,
                clip,
            );
            push(sprite.order, 5, value);
        }

        // Scroll layers are composited as tiles: polychrome sprites whose
        // texture no atlas allocates. Their content is drawn in place of the
        // tiles, moved to where the tiles are.
        let mut layer_tiles: BTreeMap<u32, Vec<LayerTile>> = BTreeMap::new();
        for sprite in &scene.polychrome_sprites {
            if let Some((layer, tile)) =
                decode_layer_tile(sprite.tile.texture_id, sprite.tile.tile_id)
            {
                if let Some(frame) = scene.layers.frames.iter().find(|frame| frame.key == layer) {
                    let tile_origin = frame.tile_bounds(tile).origin;
                    let origin = sprite.bounds.origin + offset;
                    layer_tiles.entry(layer.0).or_default().push((
                        sprite.order,
                        sprite.content_mask.bounds,
                        Point::new(origin.x - tile_origin.x, origin.y - tile_origin.y),
                    ));
                }
                continue;
            }
            let value = self.poly_sprite(sprite, offset, clip);
            push(sprite.order, 6, value);
        }
        for frame in &scene.layers.frames {
            let Some(tiles) = layer_tiles.get(&frame.key.0) else {
                continue;
            };
            let order = tiles.iter().map(|(order, ..)| *order).min().unwrap_or(0);
            let offset = tiles[0].2;
            let viewport = tiles
                .iter()
                .map(|(_, mask, _)| *mask)
                .reduce(|a, b| a.union(&b))
                .map(|mask| match clip {
                    Some(clip) => mask.intersect(&clip),
                    None => mask,
                });
            let mut content = Vec::new();
            match frame.content.scene() {
                Some(scene) => self.scene(scene, offset, viewport, &mut content),
                None => {
                    // Content in several parts only comes out per tile. A
                    // primitive over more than one tile comes out of each.
                    let mut seen = HashSet::new();
                    let coords = tiles_of(frame, tiles);
                    for (coord, tile_offset) in coords {
                        let mut tile_items = Vec::new();
                        self.scene(
                            &frame.tile_scene(coord),
                            tile_offset,
                            viewport,
                            &mut tile_items,
                        );
                        for item in tile_items {
                            if seen.insert(item.value.to_string()) {
                                content.push(item);
                            }
                        }
                    }
                }
            }
            content.sort_by_key(|item| (item.order, item.rank));
            for (ix, mut item) in content.into_iter().enumerate() {
                item.order = (order, ix as u32);
                item.rank = 6;
                items.push(item);
            }
        }
    }

    fn mono_sprite(
        &mut self,
        bounds: Bounds<ScaledPixels>,
        sprite_color: Hsla,
        tile: &gpui::AtlasTile,
        mask: Bounds<ScaledPixels>,
        offset: Point<ScaledPixels>,
        clip: Option<Bounds<ScaledPixels>>,
    ) -> Option<Value> {
        let value = match self.recorder.tiles.get(&tile_key(tile))? {
            AtlasKey::Glyph(params) => {
                let raster = self.recorder.raster_bounds.get(&glyph_key(params))?;
                let text = self
                    .recorder
                    .glyph_text
                    .get(&(params.font_id, params.glyph_id))
                    .cloned()
                    .unwrap_or_default();
                let font = self.font(params.font_id);
                let origin = bounds.origin + offset;
                let x = origin.x.0 - raster.origin.x.0 as f32
                    + params.subpixel_variant.x as f32 / SUBPIXEL_VARIANTS_X;
                let y = origin.y.0 - raster.origin.y.0 as f32;
                let size = f32::from(params.font_size);
                let advance = self
                    .recorder
                    .advances
                    .get(&(params.font_id, params.glyph_id))
                    .map(|advance| round(advance * size));
                let lead = self
                    .recorder
                    .leads
                    .get(&(params.font_id, params.glyph_id))
                    .map(|lead| round(lead * size));
                json!({
                    "t": "glyph",
                    "font": font,
                    "size": size,
                    "color": color(sprite_color),
                    "x": round(x / self.scale),
                    "y": round(y / self.scale),
                    "adv": advance,
                    "lead": lead,
                    "sp": self.space_advance(params.font_id).map(|advance| round(advance * size)),
                    "s": text,
                })
            }
            AtlasKey::Svg(params) => {
                let path = params.path.to_string();
                self.icons.insert(path.clone());
                json!({
                    "t": "icon",
                    "path": path,
                    "b": self.bounds(bounds, offset),
                    "color": color(sprite_color),
                })
            }
            AtlasKey::Image(_) => return None,
        };
        Some(self.with_clip(value, mask, offset, clip))
    }

    fn poly_sprite(
        &mut self,
        sprite: &gpui::PolychromeSprite,
        offset: Point<ScaledPixels>,
        clip: Option<Bounds<ScaledPixels>>,
    ) -> Option<Value> {
        let kind = match self.recorder.tiles.get(&tile_key(&sprite.tile)) {
            Some(AtlasKey::Glyph(params)) => {
                let text = self
                    .recorder
                    .glyph_text
                    .get(&(params.font_id, params.glyph_id))
                    .cloned()
                    .unwrap_or_default();
                json!({ "emoji": text })
            }
            Some(AtlasKey::Image(params)) => json!({ "image": params.image_id.0 }),
            _ => json!({}),
        };
        let mut value = json!({
            "t": "image",
            "b": self.bounds(sprite.bounds, offset),
            "r": self.corners(sprite.corner_radii),
            "opacity": sprite.opacity,
        });
        if let (Value::Object(value), Value::Object(kind)) = (&mut value, kind) {
            value.extend(kind);
        }
        Some(self.with_clip(value, sprite.content_mask.bounds, offset, clip))
    }

    fn font(&mut self, id: gpui::FontId) -> String {
        let key = id.0.to_string();
        if !self.fonts.contains_key(&key) {
            // A font no family asked for is one the text system fell back
            // to for a character the asked for font lacks.
            let fonts = self
                .recorder
                .fonts
                .get(&id)
                .map(Vec::as_slice)
                .unwrap_or_default();
            // Fonts asked for with different features resolve alike.
            let mut families = Vec::<String>::new();
            for font in fonts {
                if !families
                    .iter()
                    .any(|family| *family == font.family.as_ref())
                {
                    families.push(font.family.to_string());
                }
            }
            let value = json!({
                "families": families,
                "weight": fonts.first().map(|font| font.weight.0),
                "style": fonts.first().map(|font| format!("{:?}", font.style).to_lowercase()),
            });
            self.fonts.insert(key.clone(), value);
        }
        key
    }

    /// The advance of a space in `font`, as a fraction of the font size, if
    /// a line with a space in it was shaped in that font.
    fn space_advance(&self, font: gpui::FontId) -> Option<f32> {
        self.recorder
            .glyph_text
            .iter()
            .find(|((font_id, _), text)| *font_id == font && text.as_str() == " ")
            .and_then(|(key, _)| self.recorder.advances.get(key))
            .copied()
    }

    fn with_clip(
        &self,
        mut value: Value,
        mask: Bounds<ScaledPixels>,
        offset: Point<ScaledPixels>,
        clip: Option<Bounds<ScaledPixels>>,
    ) -> Value {
        let mask = Bounds {
            origin: mask.origin + offset,
            size: mask.size,
        };
        let mask = match clip {
            Some(clip) => mask.intersect(&clip),
            None => mask,
        };
        value["clip"] = self.bounds(mask, Point::default());
        value
    }

    fn px(&self, value: ScaledPixels) -> f32 {
        round(value.0 / self.scale)
    }

    fn bounds(&self, bounds: Bounds<ScaledPixels>, offset: Point<ScaledPixels>) -> Value {
        let origin = bounds.origin + offset;
        json!([
            self.px(origin.x),
            self.px(origin.y),
            self.px(bounds.size.width),
            self.px(bounds.size.height),
        ])
    }

    fn corners(&self, corners: Corners<ScaledPixels>) -> Value {
        json!([
            self.px(corners.top_left),
            self.px(corners.top_right),
            self.px(corners.bottom_right),
            self.px(corners.bottom_left),
        ])
    }

    fn edges(&self, edges: Edges<ScaledPixels>) -> Value {
        json!([
            self.px(edges.top),
            self.px(edges.right),
            self.px(edges.bottom),
            self.px(edges.left),
        ])
    }
}

/// The tiles a layer composites, each with the offset from its tile space to
/// the window.
fn tiles_of(
    frame: &gpui::LayerFrame,
    tiles: &[LayerTile],
) -> Vec<(gpui::TileCoord, Point<ScaledPixels>)> {
    let side = frame.tile_size as f32;
    let offset = tiles[0].2;
    // Every tile of the layer shares one offset to the window; the tiles in
    // view are those whose content-space bounds fall in the viewport.
    let viewport = tiles
        .iter()
        .map(|(_, mask, _)| *mask)
        .reduce(|a, b| a.union(&b))
        .unwrap_or_default();
    let left = ((viewport.origin.x.0 - offset.x.0) / side).floor() as i32;
    let top = ((viewport.origin.y.0 - offset.y.0) / side).floor() as i32;
    let right = ((viewport.origin.x.0 + viewport.size.width.0 - offset.x.0) / side).ceil() as i32;
    let bottom = ((viewport.origin.y.0 + viewport.size.height.0 - offset.y.0) / side).ceil() as i32;
    let mut coords = Vec::new();
    for y in top..bottom {
        for x in left..right {
            let coord = gpui::TileCoord { x, y };
            let tile_origin = frame.tile_bounds(coord).origin;
            coords.push((coord, offset + tile_origin));
        }
    }
    coords
}

/// Joins glyphs into runs of text: the glyphs of one line in one font and
/// clip, with the spaces between them put back, which paint nothing and so
/// are not in the scene. Colours change within a run, as syntax highlighting
/// does: `colors` holds `[start, length, color]` spans of its characters.
/// gpui sorts sprites by draw order, which scatters the glyphs of a line, so
/// they are joined by position. A run takes the place of its last painted
/// glyph.
fn merge_text(items: Vec<Value>) -> Vec<Value> {
    let mut lines: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (ix, item) in items.iter().enumerate() {
        if item["t"] == "glyph" {
            let key = json!([item["font"], item["size"], item["y"], item["clip"]]);
            lines.entry(key.to_string()).or_default().push(ix);
        }
    }
    let mut runs_at: BTreeMap<usize, Vec<Value>> = BTreeMap::new();
    for mut glyphs in lines.into_values() {
        glyphs.sort_by(|a, b| {
            let x = |ix: &usize| items[*ix]["x"].as_f64().unwrap_or_default();
            x(a).total_cmp(&x(b))
        });
        let size = items[glyphs[0]]["size"].as_f64().unwrap_or(12.);
        let space = space_advance(&items, &glyphs)
            .unwrap_or(size * 0.28)
            .max(0.5);
        let mut run = Run::default();
        let mut previous_x = f64::NEG_INFINITY;
        let mut end = f64::NEG_INFINITY;
        for ix in glyphs {
            let glyph = &items[ix];
            // A ligature over spacers starts where its first spacer does.
            let lead = glyph["lead"].as_f64().unwrap_or_default();
            let x = glyph["x"].as_f64().unwrap_or_default() - lead;
            let text = glyph["s"].as_str().unwrap_or_default();
            let color = glyph["color"].as_str().unwrap_or_default();
            let advance = glyph["adv"].as_f64().unwrap_or(size * 0.6) + lead;
            if !run.chars.is_empty() {
                if (x - previous_x).abs() < 0.01 {
                    // The same glyph twice, from two tiles of a scroll layer.
                    continue;
                }
                let gap = x - end;
                let spaces = (gap / space).round();
                if gap > space * 0.5 && (spaces > 8. || (gap - spaces * space).abs() > space * 0.4)
                {
                    run.flush(&items, &mut runs_at);
                } else {
                    for n in 0..spaces.max(0.) as usize {
                        run.push(end + n as f64 * space, " ", None);
                    }
                }
            }
            let count = text.chars().count().max(1);
            // Each character of a ligature gets an even share of its advance.
            for (n, ch) in text.chars().enumerate() {
                let mut buf = [0; 4];
                run.push(
                    x + advance * n as f64 / count as f64,
                    ch.encode_utf8(&mut buf),
                    Some(color),
                );
            }
            run.last = run.last.max(ix);
            run.first.get_or_insert(ix);
            previous_x = x;
            end = x + advance;
        }
        run.flush(&items, &mut runs_at);
    }
    let mut merged = Vec::with_capacity(items.len());
    for (ix, item) in items.into_iter().enumerate() {
        if item["t"] != "glyph" {
            merged.push(item);
        }
        if let Some(runs) = runs_at.remove(&ix) {
            merged.extend(runs);
        }
    }
    merged
}

#[derive(Default)]
struct Run {
    chars: Vec<(f64, String)>,
    /// `[start, length, color]` spans over `chars`.
    colors: Vec<(usize, usize, String)>,
    first: Option<usize>,
    last: usize,
}

impl Run {
    /// Adds a character at `x`. A space takes the colour of what it follows.
    fn push(&mut self, x: f64, ch: &str, color: Option<&str>) {
        let ix = self.chars.len();
        self.chars.push((round64(x), ch.to_string()));
        match (self.colors.last_mut(), color) {
            (Some(span), None) => span.1 += 1,
            (Some(span), Some(color)) if span.2 == color => span.1 += 1,
            (_, Some(color)) => self.colors.push((ix, 1, color.to_string())),
            (None, None) => {}
        }
    }

    fn flush(&mut self, items: &[Value], runs_at: &mut BTreeMap<usize, Vec<Value>>) {
        let Some(first) = self.first.take() else {
            return;
        };
        let first = &items[first];
        runs_at.entry(self.last).or_default().push(json!({
            "t": "text",
            "font": first["font"],
            "size": first["size"],
            "y": first["y"],
            "text": self.chars.iter().map(|(_, ch)| ch.as_str()).collect::<String>(),
            "x": self.chars.iter().map(|(x, _)| *x).collect::<Vec<_>>(),
            "colors": self.colors.iter().map(|(start, len, color)| json!([start, len, color])).collect::<Vec<_>>(),
            "clip": first["clip"],
        }));
        *self = Run::default();
    }
}

/// The advance of a space in the font of `glyphs`, if one was shaped.
fn space_advance(items: &[Value], glyphs: &[usize]) -> Option<f64> {
    if let Some(space) = items[glyphs[0]]["sp"].as_f64() {
        return Some(space);
    }
    // Monospaced text has every advance the same; otherwise a space is
    // about a quarter of an em.
    let mut advances = glyphs.iter().filter_map(|ix| items[*ix]["adv"].as_f64());
    let first = advances.next()?;
    let monospaced = advances.all(|advance| (advance - first).abs() < 0.01);
    monospaced.then_some(first)
}

fn round64(value: f64) -> f64 {
    (value * 100.).round() / 100.
}

fn edges_zero(edges: &Edges<ScaledPixels>) -> bool {
    edges.top.0 == 0. && edges.right.0 == 0. && edges.bottom.0 == 0. && edges.left.0 == 0.
}

fn round(value: f32) -> f32 {
    (value * 100.).round() / 100.
}

fn color(color: Hsla) -> String {
    let Rgba { r, g, b, a } = Rgba::from(color);
    format!(
        "rgba({},{},{},{})",
        (r * 255.).round(),
        (g * 255.).round(),
        (b * 255.).round(),
        round(a)
    )
}

/// A fill as `{ "type": "solid" | "linear" | "pattern", ... }`, or `None`
/// when it paints nothing. gpui keeps a background's parts private, so they
/// are read from its serialized form.
fn background(background: &Background) -> Option<Value> {
    let value = serde_json::to_value(background).ok()?;
    let hsla = |value: &Value| serde_json::from_value::<Hsla>(value.clone()).ok();
    match value["tag"].as_str()? {
        "Solid" => {
            let solid = hsla(&value["solid"])?;
            (solid.a > 0.).then(|| json!({ "type": "solid", "color": color(solid) }))
        }
        "LinearGradient" => {
            let stops = value["colors"]
                .as_array()?
                .iter()
                .filter_map(|stop| {
                    Some(json!([
                        color(hsla(&stop["color"])?),
                        stop["percentage"].as_f64()?
                    ]))
                })
                .collect::<Vec<_>>();
            Some(json!({
                "type": "linear",
                "angle": value["gradient_angle_or_pattern_height"],
                "stops": stops,
            }))
        }
        _ => {
            let solid = hsla(&value["solid"])?;
            Some(json!({ "type": "pattern", "color": color(solid) }))
        }
    }
}
