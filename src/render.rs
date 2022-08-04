use crate::{
    context::Context,
    font::FontId,
    graphics::{Graphic, Sprite},
    Color, Id, RenderContext, RenderDirtyFlags,
};
// use glyph_brush_draw_cache::{CachedBy, DrawCache, DrawCacheBuilder};
use crate::time::Instant;
use ab_glyph::{Font, GlyphId};
use std::ops::Range;
use texture_cache::{Cached, LruTextureCache, RectEntry};

#[derive(Debug)]
/// A glyph and a font_id
pub struct FontGlyph {
    pub glyph: ab_glyph::Glyph,
    pub font_id: FontId,
    pub color: Color,
}

pub trait GuiRenderer {
    fn update_font_texture(&mut self, font_texture: u32, rect: [u32; 4], data: &[u8]);
    fn resize_font_texture(&mut self, font_texture: u32, new_size: [u32; 2]);
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct GlyphKey {
    font_id: FontId,
    glyph: GlyphId,
    sub_pixel: (u8, u8),
    scale: (u16, u16),
}
impl GlyphKey {
    fn new(f: FontId, g: &ab_glyph::Glyph) -> Self {
        const SUB_PIXEL_PRECISION: u32 = 8;
        const SCALE_PRECISION: u32 = 8;
        GlyphKey {
            font_id: f,
            glyph: g.id,
            sub_pixel: (
                (((g.position.x * SUB_PIXEL_PRECISION as f32).round() as u32) % SUB_PIXEL_PRECISION)
                    as u8,
                (((g.position.y * SUB_PIXEL_PRECISION as f32).round() as u32) % SUB_PIXEL_PRECISION)
                    as u8,
            ),
            scale: (
                (g.scale.x * SCALE_PRECISION as f32).round() as u16,
                (g.scale.y * SCALE_PRECISION as f32).round() as u16,
            ),
        }
    }
}

pub struct GuiRender {
    draw_cache: LruTextureCache<GlyphKey, [f32; 4]>,
    font_texture: u32,
    white_texture: u32,
    last_sprites: Vec<Sprite>,
    last_sprites_map: Vec<(Id, Range<usize>)>,
    sprites: Vec<Sprite>,
    sprites_map: Vec<(Id, Range<usize>)>,
    last_anim_draw: Option<Instant>,
}
impl GuiRender {
    pub fn new(font_texture: u32, white_texture: u32, font_texture_size: [u32; 2]) -> Self {
        //TODO: change this to default dimensions, and allow resizing
        let draw_cache = LruTextureCache::new(font_texture_size[0], font_texture_size[1]);
        Self {
            draw_cache,
            font_texture,
            white_texture,
            last_sprites: Vec::new(),
            last_sprites_map: Vec::new(),
            sprites: Vec::new(),
            sprites_map: Vec::new(),
            last_anim_draw: None,
        }
    }

    /// Replace the current font texture by the given one.
    ///
    /// This invalidates the current glyph cache.
    pub fn set_font_texture(&mut self, font_texture: u32, font_texture_size: [u32; 2]) {
        self.font_texture = font_texture;
        self.draw_cache = LruTextureCache::new(font_texture_size[0], font_texture_size[1]);
    }

    pub fn clear_cache(&mut self, ctx: &mut Context) {
        self.last_sprites.clear();
        self.last_sprites_map.clear();
        let mut parents = vec![crate::Id::ROOT_ID];
        while let Some(parent) = parents.pop() {
            let graphic = ctx.get_graphic_mut(parent);
            match graphic {
                Graphic::Panel(x) => {
                    x.color_dirty = true;
                }
                Graphic::Texture(x) => {
                    x.color_dirty = true;
                }
                Graphic::Icon(x) => {
                    x.color_dirty = true;
                }
                Graphic::AnimatedIcon(x) => {
                    x.color_dirty = true;
                }
                Graphic::Text(x) => x.dirty(),
                Graphic::None => {}
            }
            parents.extend(ctx.get_active_children(parent).iter().rev())
        }
    }

    pub fn render<'a, T: GuiRenderer>(
        &'a mut self,
        ctx: &mut RenderContext,
        mut renderer: T,
    ) -> (&'a [Sprite], bool) {
        self.sprites.clear();
        self.sprites_map.clear();
        let mut masks: Vec<(usize, [f32; 4], bool)> = Vec::new();

        fn intersection(a: &[f32; 4], b: &[f32; 4]) -> Option<[f32; 4]> {
            if a[0] > b[2] || a[2] < b[0] || a[1] > b[3] || a[3] < b[1] {
                return None;
            }
            Some([
                a[0].max(b[0]),
                a[1].max(b[1]),
                a[2].min(b[2]),
                a[3].min(b[3]),
            ])
        }

        let fonts = ctx.get_fonts();
        let scale_factor = ctx.scale_factor() as f32;

        // queue all glyphs for cache

        let mut queue = Vec::new();
        let mut add_to_queue = |f: FontId, mut g: ab_glyph::Glyph| {
            g.scale.x *= scale_factor;
            g.scale.y *= scale_factor;
            g.position.x *= scale_factor;
            g.position.y *= scale_factor;

            let outline = match fonts.get(f).unwrap().outline_glyph(g.clone()) {
                Some(x) => x,
                None => return,
            };
            let bounds = outline.px_bounds();
            let width = bounds.width() as u32;
            let height = bounds.height() as u32;
            queue.push(RectEntry {
                width,
                height,
                key: GlyphKey::new(f, &g),
                value: [
                    bounds.min.x - g.position.x,
                    bounds.min.y - g.position.y,
                    bounds.max.x - g.position.x,
                    bounds.max.y - g.position.y,
                ],
                entry_data: outline,
            })
        };

        let mut parents = vec![Id::ROOT_ID];
        while let Some(parent) = parents.pop() {
            parents.extend(ctx.get_active_children(parent).iter());
            if let (rect, Graphic::Text(text)) = ctx.get_rect_and_graphic(parent) {
                let (glyphs, _) = text.get_glyphs_and_rects(rect, fonts);
                for glyph in glyphs {
                    add_to_queue(glyph.font_id, glyph.glyph.clone());
                }
            }
        }

        // If `self.set_font_texture` was called, the draw_cache was cleared, and the texture
        // became invalid.
        let mut font_texture_valid = self.draw_cache.len() > 0;

        loop {
            // add the glyphs to the cache
            let added = match self.draw_cache.cache_rects(&mut queue) {
                Ok(Cached::Added(x) | Cached::Changed(x)) => x,
                Ok(Cached::Cleared(x)) => {
                    log::debug!("draw cache: cleared");
                    font_texture_valid = false;
                    x
                }
                Err(_) => {
                    let width = 2 * self.draw_cache.width();
                    let height = 2 * self.draw_cache.height();
                    self.draw_cache = LruTextureCache::new(width, height);
                    renderer.resize_font_texture(self.font_texture, [width, height]);
                    log::debug!("draw cache: rebuilded to {} x {}", width, height);
                    font_texture_valid = false;
                    // retry
                    continue;
                }
            };

            // render the glyphs and upload to the texture
            for entry in &queue[..added] {
                let rect = self.draw_cache.get_rect(&entry.key).unwrap();
                let outlined_glyph = &entry.entry_data;
                let g_width = rect.width as usize;
                let g_height = rect.height as usize;
                let mut pixels = vec![0; g_width * g_height];
                outlined_glyph.draw(|x, y, c| {
                    let i = y as usize * g_width + x as usize;
                    pixels[i] = (c * 256.0) as u8;
                });
                renderer.update_font_texture(
                    self.font_texture,
                    [rect.x, rect.y, rect.x + rect.width, rect.y + rect.height],
                    &pixels,
                )
            }

            break;
        }

        let mut is_animating = false;
        let dt = if let Some(x) = self.last_anim_draw {
            x.elapsed().as_secs_f32()
        } else {
            0.0
        };

        let scale_rect = |rect: [f32; 4]| rect.map(|x| x * scale_factor);

        let mut parents = vec![Id::ROOT_ID];
        'tree: while let Some(parent) = parents.pop() {
            let (mask, mask_changed) = {
                let rect = ctx.get_layouting(parent);
                let mask = *rect.get_rect();
                let mut mask = scale_rect([
                    mask[0].round(),
                    mask[1].round(),
                    mask[2].round(),
                    mask[3].round(),
                ]);
                let mut mask_changed = rect
                    .get_render_dirty_flags()
                    .contains(RenderDirtyFlags::RECT);
                while let Some((i, upper_mask, changed)) = masks.last() {
                    if parents.len() < *i {
                        masks.pop();
                        continue;
                    }

                    mask_changed |= *changed;

                    if let Some(intersection) = intersection(&mask, upper_mask) {
                        mask = intersection;
                    } else {
                        continue 'tree;
                    }
                    break;
                }
                masks.push((parents.len(), mask, mask_changed));
                (mask, mask_changed)
            };
            {
                let (rect, graphic) = ctx.get_rect_and_graphic(parent);
                let mut compute_sprite = true;
                let is_text = matches!(graphic, Graphic::Text(_));
                let graphic_is_dirty = !rect.get_render_dirty_flags().is_empty()
                    || mask_changed
                    || graphic.need_rebuild()
                    || (is_text && !font_texture_valid);

                let len = self.sprites.len();
                if !graphic_is_dirty {
                    if let Some(range) = self
                        .last_sprites_map
                        .get(self.sprites_map.len())
                        .filter(|x| x.0 == parent)
                        .map(|x| x.1.clone())
                        .or_else(|| {
                            self.last_sprites_map
                                .iter()
                                .find(|x| x.0 == parent)
                                .map(|x| x.1.clone())
                        })
                    {
                        compute_sprite = false;
                        let sprites = self.last_sprites[range].iter().cloned();
                        if graphic.is_color_dirty() {
                            self.sprites.extend(sprites.map(|mut x| {
                                x.color = graphic.get_color();
                                x
                            }));
                        } else {
                            self.sprites.extend(sprites);
                        }
                    }
                }
                if compute_sprite {
                    match graphic {
                        Graphic::Panel(panel) => {
                            let rect = scale_rect(*rect.get_rect());
                            for mut sprite in panel.get_sprites(rect).iter().cloned() {
                                if cut_sprite(&mut sprite, &mask) {
                                    self.sprites.push(sprite);
                                }
                            }
                        }
                        Graphic::Texture(x) => {
                            let rect = rect;
                            let rect = scale_rect(*rect.get_rect());
                            let mut sprite = x.get_sprite(rect);
                            if cut_sprite(&mut sprite, &mask) {
                                self.sprites.push(sprite);
                            }
                        }
                        Graphic::Icon(x) => {
                            let rect = rect;
                            let rect = scale_rect(*rect.get_rect());
                            let mut sprite = x.get_sprite(rect);
                            if cut_sprite(&mut sprite, &mask) {
                                self.sprites.push(sprite);
                            }
                        }
                        Graphic::AnimatedIcon(x) => {
                            is_animating = true;

                            let rect = rect;
                            let rect = scale_rect(*rect.get_rect());
                            let mut sprite = x.get_sprite(rect, dt);
                            if cut_sprite(&mut sprite, &mask) {
                                self.sprites.push(sprite);
                            }
                        }
                        Graphic::Text(ref mut text) => {
                            let (glyphs, rects) = text.get_glyphs_and_rects(rect, fonts);
                            for rect in rects {
                                let mut sprite = Sprite {
                                    texture: self.white_texture,
                                    color: rect.color,
                                    rect: rect.rect,
                                    uv_rect: [0.0, 0.0, 1.0, 1.0],
                                };
                                if cut_sprite(&mut sprite, &mask) {
                                    self.sprites.push(sprite);
                                }
                            }
                            for glyph in glyphs {
                                let g = {
                                    let mut g = glyph.glyph.clone();
                                    g.scale.x *= scale_factor;
                                    g.scale.y *= scale_factor;
                                    g.position.x *= scale_factor;
                                    g.position.y *= scale_factor;
                                    g
                                };
                                if let Some(rect) =
                                    self.draw_cache.get_rect(&GlyphKey::new(glyph.font_id, &g))
                                {
                                    // (tex_coords, pixel_coords)
                                    let tex_width = self.draw_cache.width() as f32;
                                    let tex_height = self.draw_cache.height() as f32;
                                    let tex_coords = [
                                        rect.x as f32 / tex_width,
                                        rect.y as f32 / tex_height,
                                        rect.width as f32 / tex_width,
                                        rect.height as f32 / tex_height,
                                    ];
                                    let px_bounds = rect.value;
                                    let pixel_coords = [
                                        px_bounds[0] + g.position.x,
                                        px_bounds[1] + g.position.y,
                                        px_bounds[2] + g.position.x,
                                        px_bounds[3] + g.position.y,
                                    ];
                                    if pixel_coords[0] as f32 > mask[2]
                                        || pixel_coords[1] as f32 > mask[3]
                                        || mask[0] > pixel_coords[2] as f32
                                        || mask[1] > pixel_coords[3] as f32
                                    {
                                        // glyph is totally outside the bounds
                                    } else {
                                        self.sprites.push(to_sprite(
                                            tex_coords,
                                            pixel_coords,
                                            mask,
                                            glyph.color,
                                            self.font_texture,
                                        ));
                                    }
                                }
                            }
                        }
                        Graphic::None => {}
                    }
                }
                graphic.clear_dirty();
                if len != self.sprites.len() {
                    self.sprites_map.push((parent, len..self.sprites.len()));
                }
            }
            ctx.get_layouting(parent).clear_render_dirty_flags();
            parents.extend(ctx.get_active_children(parent).iter().rev())
        }

        std::mem::swap(&mut self.sprites, &mut self.last_sprites);
        std::mem::swap(&mut self.sprites_map, &mut self.last_sprites_map);

        if is_animating {
            self.last_anim_draw = Some(Instant::now());
        }

        (&self.last_sprites, is_animating)
    }
}

#[inline]
pub fn cut_sprite(sprite: &mut Sprite, bounds: &[f32; 4]) -> bool {
    let rect = &mut sprite.rect;
    if rect[0] < bounds[0] {
        let d = (bounds[0] - rect[0]) / (rect[2] - rect[0]);
        rect[0] = bounds[0];
        sprite.uv_rect[0] += sprite.uv_rect[2] * d;
        sprite.uv_rect[2] *= 1.0 - d;
    }
    if rect[2] > bounds[2] {
        let d = (rect[2] - bounds[2]) / (rect[2] - rect[0]);
        rect[2] = bounds[2];
        sprite.uv_rect[2] *= 1.0 - d;
    }

    if rect[1] < bounds[1] {
        let d = (bounds[1] - rect[1]) / (rect[3] - rect[1]);
        rect[1] = bounds[1];
        sprite.uv_rect[1] += sprite.uv_rect[3] * d;
        sprite.uv_rect[3] *= 1.0 - d;
    }
    if rect[3] > bounds[3] {
        let d = (rect[3] - bounds[3]) / (rect[3] - rect[1]);
        rect[3] = bounds[3];
        sprite.uv_rect[3] *= 1.0 - d;
    }

    !(rect[2] - rect[0] < 0.0 || rect[3] - rect[1] < 0.0)
}

#[inline]
pub fn to_sprite(
    tex_coords: [f32; 4],
    pixel_coords: [f32; 4],
    bounds: [f32; 4],
    color: Color,
    font_texture: u32,
) -> Sprite {
    let mut sprite = Sprite {
        texture: font_texture,
        color,
        rect: pixel_coords,
        uv_rect: tex_coords,
    };

    cut_sprite(&mut sprite, &bounds);
    sprite
}
