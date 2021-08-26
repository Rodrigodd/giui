use crate::{
    context::Context,
    font::FontId,
    graphics::{Graphic, Sprite},
    Color, Id, RenderDirtyFlags,
};
use glyph_brush_draw_cache::{CachedBy, DrawCache, DrawCacheBuilder};
use std::{ops::Range, time::Instant};

#[derive(Debug)]
/// A glyph and a font_id
pub struct FontGlyph {
    pub glyph: ab_glyph::Glyph,
    pub font_id: FontId,
    pub color: Color,
}

pub trait GuiRenderer {
    fn update_font_texure(&mut self, font_texture: u32, rect: [u32; 4], data: &[u8]);
    fn resize_font_texture(&mut self, font_texture: u32, new_size: [u32; 2]);
}

pub struct GuiRender {
    draw_cache: DrawCache,
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
        let draw_cache = DrawCacheBuilder::default()
            .dimensions(font_texture_size[0], font_texture_size[1])
            .build();
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

    pub fn clear_cache(&mut self, ctx: &mut Context) {
        self.last_sprites.clear();
        self.last_sprites_map.clear();
        let mut parents = vec![crate::Id::ROOT_ID];
        while let Some(parent) = parents.pop() {
            if let Some((rect, graphic)) = ctx.get_rect_and_graphic(parent) {
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
                rect.dirty_render_dirty_flags();
            } else {
                ctx.get_layouting(parent).dirty_render_dirty_flags();
            }
            parents.extend(ctx.get_active_children(parent).iter().rev())
        }
    }

    pub fn render<'a, T: GuiRenderer>(
        &'a mut self,
        ctx: &mut Context,
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

        let mut font_texture_valid = true;
        loop {
            // queue all glyphs for cache
            let mut parents = vec![Id::ROOT_ID];
            while let Some(parent) = parents.pop() {
                parents.extend(ctx.get_active_children(parent).iter());
                if let Some((rect, Graphic::Text(text))) = ctx.get_rect_and_graphic(parent) {
                    let (glyphs, _) = text.get_glyphs_and_rects(rect, fonts);
                    for glyph in glyphs {
                        self.draw_cache
                            .queue_glyph(glyph.font_id.index(), glyph.glyph.clone());
                    }
                }
            }

            // update the font_texture
            let font_texture = self.font_texture;
            match self.draw_cache.cache_queued(fonts.as_slice(), |r, d| {
                renderer.update_font_texure(
                    font_texture,
                    [r.min[0], r.min[1], r.max[0], r.max[1]],
                    d,
                )
            }) {
                Ok(CachedBy::Adding) => {}
                Ok(CachedBy::Reordering) => {
                    font_texture_valid = false;
                }
                Err(_) => {
                    let (width, height) = self.draw_cache.dimensions();
                    self.draw_cache = DrawCacheBuilder::default()
                        .dimensions(width * 2, height * 2)
                        .build();
                    renderer.resize_font_texture(font_texture, [width * 2, height * 2]);
                    font_texture_valid = false;
                    continue;
                }
            }
            break;
        }

        let mut is_animating = false;
        let dt = if let Some(x) = self.last_anim_draw {
            x.elapsed().as_secs_f32()
        } else {
            0.0
        };

        let mut parents = vec![Id::ROOT_ID];
        'tree: while let Some(parent) = parents.pop() {
            let (mask, mask_changed) = {
                let rect = ctx.get_layouting(parent);
                let mask = *rect.get_rect();
                let mut mask = [
                    mask[0].round(),
                    mask[1].round(),
                    mask[2].round(),
                    mask[3].round(),
                ];
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
            if let Some((rect, graphic)) = ctx.get_rect_and_graphic(parent) {
                let mut compute_sprite = true;
                let is_text = matches!(graphic, Graphic::Text(_));
                let graphic_is_dirty = rect
                    .get_render_dirty_flags()
                    .contains(RenderDirtyFlags::RECT)
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
                            for mut sprite in panel.get_sprites(rect.rect).iter().cloned() {
                                if cut_sprite(&mut sprite, &mask) {
                                    self.sprites.push(sprite);
                                }
                            }
                        }
                        Graphic::Texture(x) => {
                            let rect = rect;
                            let mut sprite = x.get_sprite(*rect.get_rect());
                            if cut_sprite(&mut sprite, &mask) {
                                self.sprites.push(sprite);
                            }
                        }
                        Graphic::Icon(x) => {
                            let rect = rect;
                            let mut sprite = x.get_sprite(*rect.get_rect());
                            if cut_sprite(&mut sprite, &mask) {
                                self.sprites.push(sprite);
                            }
                        }
                        Graphic::AnimatedIcon(x) => {
                            is_animating = true;

                            let rect = rect;
                            let mut sprite = x.get_sprite(*rect.get_rect(), dt);
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
                                if let Some((tex_coords, pixel_coords)) = self
                                    .draw_cache
                                    .rect_for(glyph.font_id.index(), &glyph.glyph)
                                {
                                    if pixel_coords.min.x as f32 > mask[2]
                                        || pixel_coords.min.y as f32 > mask[3]
                                        || mask[0] > pixel_coords.max.x as f32
                                        || mask[1] > pixel_coords.max.y as f32
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
    tex_coords: ab_glyph::Rect,
    pixel_coords: ab_glyph::Rect,
    bounds: [f32; 4],
    color: Color,
    font_texture: u32,
) -> Sprite {
    let mut sprite = Sprite {
        texture: font_texture,
        color,
        rect: [
            pixel_coords.min.x,
            pixel_coords.min.y,
            pixel_coords.max.x,
            pixel_coords.max.y,
        ],
        uv_rect: [
            tex_coords.min.x,
            tex_coords.min.y,
            tex_coords.width(),
            tex_coords.height(),
        ],
    };

    cut_sprite(&mut sprite, &bounds);
    sprite
}
