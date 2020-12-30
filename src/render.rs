use crate::{
    context::Context,
    text::{FontGlyph, TextInfo},
    Id, Rect, RenderDirtyFlags,
};
use ab_glyph::{Font, FontArc};
use glyph_brush_draw_cache::{DrawCache, DrawCacheBuilder, Rectangle};
use std::ops::Range;

#[derive(Clone)]
pub struct Sprite {
    pub texture: u32,
    pub color: [u8; 4],
    pub rect: [f32; 4],
    pub uv_rect: [f32; 4],
}

pub struct GUIRender {
    draw_cache: DrawCache,
    font_texture: u32,
    last_sprites: Vec<Sprite>,
    last_sprites_map: Vec<(Id, Range<usize>)>,
    sprites: Vec<Sprite>,
    sprites_map: Vec<(Id, Range<usize>)>,
}
impl GUIRender {
    pub fn new(font_texture: u32) -> Self {
        //TODO: change this to default dimensions, and allow resizing
        let draw_cache = DrawCacheBuilder::default().dimensions(1024, 1024).build();
        Self {
            draw_cache,
            font_texture,
            last_sprites: Vec::new(),
            last_sprites_map: Vec::new(),
            sprites: Vec::new(),
            sprites_map: Vec::new(),
        }
    }

    pub fn clear_cache(&mut self, ctx: &mut Context) {
        self.last_sprites.clear();
        self.last_sprites_map.clear();
        let mut parents = vec![crate::ROOT_ID];
        while let Some(parent) = parents.pop() {
            if let Some((rect, graphic)) = ctx.get_rect_and_graphic(parent) {
                match graphic {
                    Graphic::Panel(x) => {
                        x.color_dirty = true;
                    }
                    Graphic::Texture(x) => {
                        x.color_dirty = true;
                    }
                    Graphic::Text(x) => x.dirty(),
                    Graphic::None => {}
                }
                rect.dirty_render_dirty_flags();
            } else {
                ctx.get_layouting(parent).dirty_render_dirty_flags();
            }
            parents.extend(ctx.get_children(parent).iter().rev())
        }
    }

    pub fn render<'a, F: FnMut(Rectangle<u32>, &[u8])>(&'a mut self, ctx: &mut Context, mut update_font_texure: F) -> &'a [Sprite] {
        use crate::ROOT_ID;
        let mut parents = vec![ROOT_ID];
        self.sprites.clear();
        self.sprites_map.clear();
        let mut masks: Vec<(usize, [f32; 4], bool)> = Vec::new();
        let fonts = ctx.get_fonts();

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

        'tree: while let Some(parent) = parents.pop() {
            let (mask, mask_changed) = {
                let rect = ctx.get_layouting(parent);
                let mut mask = *rect.get_rect();
                let mut mask_changed = rect
                    .get_render_dirty_flags()
                    .contains(RenderDirtyFlags::RECT);
                while let Some((i, m, changed)) = masks.last() {
                    let upper_mask;
                    if parents.len() < *i {
                        masks.pop();
                        continue;
                    } else {
                        upper_mask = *m;
                        mask_changed |= *changed;
                    }

                    if let Some(intersection) = intersection(&mask, &upper_mask) {
                        mask = intersection;
                    } else {
                        continue 'tree;
                    }
                    break;
                }
                masks.push((parents.len(), mask, mask_changed));
                (mask, mask_changed)
            };
            let mut compute_sprite = true;
            if let Some((rect, graphic)) = ctx.get_rect_and_graphic(parent) {
                let len = self.sprites.len();
                if !rect
                    .get_render_dirty_flags()
                    .contains(RenderDirtyFlags::RECT)
                    && !mask_changed
                    && !graphic.need_rebuild()
                {
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
                            let mut painel =
                                Painel::new(panel.texture, panel.uv_rect, panel.border);
                            painel.set_rect(rect);
                            painel.set_color(panel.color);
                            for mut sprite in painel.get_sprites().iter().cloned() {
                                if cut_sprite(&mut sprite, &mask) {
                                    self.sprites.push(sprite);
                                }
                            }
                        }
                        Graphic::Texture(Texture {
                            texture,
                            uv_rect,
                            color,
                            ..
                        }) => {
                            let rect = rect;
                            let mut sprite = Sprite {
                                texture: *texture,
                                color: *color,
                                rect: *rect.get_rect(),
                                uv_rect: *uv_rect,
                            };
                            if cut_sprite(&mut sprite, &mask) {
                                self.sprites.push(sprite);
                            }
                        }
                        Graphic::Text(ref mut text) => {
                            let color = text.color;
                            let glyphs = text.get_glyphs(rect, &fonts);

                            for glyph in glyphs {
                                self.draw_cache
                                    .queue_glyph(glyph.font_id.0, glyph.glyph.clone());
                            }
                            //TODO: I should queue all the glyphs, before calling cache_queued
                            self.draw_cache
                                .cache_queued(&fonts, |a,b| update_font_texure(a,b))
                                .unwrap();
                            for glyph in glyphs {
                                if let Some((tex_coords, pixel_coords)) =
                                    self.draw_cache.rect_for(glyph.font_id.0, &glyph.glyph)
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
                                            color,
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
            parents.extend(ctx.get_children(parent).iter().rev())
        }

        std::mem::swap(&mut self.sprites, &mut self.last_sprites);
        std::mem::swap(&mut self.sprites_map, &mut self.last_sprites_map);
        &self.last_sprites
    }
}

pub struct Text {
    color: [u8; 4],
    color_dirty: bool,
    text: String,
    text_dirty: bool,
    font_size: f32,
    align: (i8, i8),
    glyphs: Vec<FontGlyph>,
    text_info: Option<TextInfo>,
    last_pos: [f32; 2],
    min_size: Option<[f32; 2]>,
}
impl Clone for Text {
    fn clone(&self) -> Self {
        Self::new(self.color, self.text.clone(), self.font_size, self.align)
    }
}
impl Text {
    pub fn new(color: [u8; 4], text: String, font_size: f32, align: (i8, i8)) -> Text {
        Self {
            color,
            color_dirty: true,
            text,
            text_dirty: true,
            font_size,
            align,
            glyphs: Vec::new(),
            text_info: None,
            last_pos: [0.0, 0.0],
            min_size: None,
        }
    }

    fn dirty(&mut self) {
        self.text_dirty = true;
        self.min_size = None;
        self.text_info = None;
    }

    pub fn get_font_size(&mut self) -> f32 {
        self.font_size
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        self.font_size = font_size;
        self.dirty();
    }

    pub fn set_text(&mut self, text: &str) {
        self.text.clear();
        self.text += text;
        self.dirty();
    }

    pub fn get_align_anchor(&self, rect: [f32; 4]) -> [f32; 2] {
        let mut anchor = [0.0; 2];
        match self.align.0 {
            -1 => anchor[0] = rect[0],
            0 => anchor[0] = (rect[0] + rect[2]) / 2.0,
            _ => anchor[0] = rect[2],
        }
        match self.align.1 {
            -1 => anchor[1] = rect[1],
            0 => anchor[1] = (rect[1] + rect[3]) / 2.0,
            _ => anchor[1] = rect[3],
        }
        anchor
    }

    fn update_glyphs<F: Font>(&mut self, rect: &mut Rect, fonts: &[F]) {
        self.last_pos = self.get_align_anchor(*rect.get_rect());
        let (glyphs, text_info) = crate::text::text_glyphs_and_info(
            &self.text,
            0,
            self.font_size,
            &fonts,
            *rect.get_rect(),
            self.align,
        );
        self.glyphs = glyphs;
        self.text_info = Some(text_info);
    }

    pub fn get_text_info<F: Font>(&mut self, fonts: &[F], rect: &mut Rect) -> &TextInfo {
        if self.text_info.is_none() {
            self.update_glyphs(rect, fonts);
        }
        self.text_info.as_ref().unwrap()
    }

    pub fn get_glyphs<F: Font>(
        &mut self,
        rect: &mut Rect,
        fonts: &[F],
    ) -> &[crate::text::FontGlyph] {
        let dirty_flags = rect.get_render_dirty_flags();
        let width_change = dirty_flags.contains(RenderDirtyFlags::WIDTH)
            && self.min_size.map_or(true, |x| rect.get_width() < x[0]);
        if self.text_dirty || width_change {
            self.update_glyphs(rect, fonts);
        } else if dirty_flags.contains(RenderDirtyFlags::RECT) && !width_change {
            let rect = *rect.get_rect();
            let anchor = self.get_align_anchor(rect);
            let delta = [anchor[0] - self.last_pos[0], anchor[1] - self.last_pos[1]];
            self.last_pos = anchor;

            for glyph in &mut self.glyphs {
                glyph.glyph.position.x += delta[0];
                glyph.glyph.position.y += delta[1];
            }
            if let Some(ref mut text_info) = self.text_info {
                text_info.move_by(delta);
            }
        }
        &self.glyphs
    }

    pub fn compute_min_size<F: Font>(&mut self, fonts: &[F]) -> Option<[f32; 2]> {
        if self.min_size.is_none() {
            let (_, text_info) = crate::text::text_glyphs_and_info(
                &self.text,
                0,
                self.font_size,
                &fonts,
                [0.0, 0.0, f32::INFINITY, f32::INFINITY],
                (-1, -1),
            );
            self.min_size = Some(text_info.get_size());
        }
        self.min_size
    }
}

pub struct Texture {
    texture: u32,
    uv_rect: [f32; 4],
    color: [u8; 4],
    color_dirty: bool,
}
impl Clone for Texture {
    fn clone(&self) -> Self {
        Self::new(self.texture, self.uv_rect).with_color(self.color)
    }
}
impl Texture {
    pub fn new(texture: u32, uv_rect: [f32; 4]) -> Self {
        Self {
            texture,
            uv_rect,
            color: [255, 255, 255, 255],
            color_dirty: true,
        }
    }

    pub fn with_color(mut self, color: [u8; 4]) -> Self {
        self.set_color(color);
        self
    }

    pub fn set_color(&mut self, color: [u8; 4]) {
        self.color = color;
        self.color_dirty = true;
    }
}

#[derive(Clone)]
pub struct Panel {
    texture: u32,
    uv_rect: [f32; 4],
    color: [u8; 4],
    color_dirty: bool,
    border: f32,
}
impl Panel {
    pub fn new(texture: u32, uv_rect: [f32; 4], border: f32) -> Self {
        Self {
            texture,
            uv_rect,
            color: [255, 255, 255, 255],
            color_dirty: true,
            border,
        }
    }
}

#[derive(Clone)]
pub enum Graphic {
    Panel(Panel),
    Texture(Texture),
    Text(Text),
    None,
}
impl Default for Graphic {
    fn default() -> Self {
        Self::None
    }
}
impl From<Panel> for Graphic {
    fn from(panel: Panel) -> Self {
        Self::Panel(panel)
    }
}
impl From<Texture> for Graphic {
    fn from(texture: Texture) -> Self {
        Self::Texture(texture)
    }
}
impl From<Text> for Graphic {
    fn from(text: Text) -> Self {
        Self::Text(text)
    }
}
impl Graphic {
    pub fn with_color(mut self, new_color: [u8; 4]) -> Self {
        self.set_color(new_color);
        self
    }

    pub fn get_color(&self) -> [u8; 4] {
        match self {
            Graphic::Panel(Panel { color, .. }) => *color,
            Graphic::Texture(Texture { color, .. }) => *color,
            Graphic::Text(Text { color, .. }) => *color,
            Graphic::None => [255, 255, 255, 255],
        }
    }

    pub fn set_color(&mut self, new_color: [u8; 4]) {
        match self {
            Graphic::Panel(Panel {
                color, color_dirty, ..
            })
            | Graphic::Texture(Texture {
                color, color_dirty, ..
            })
            | Graphic::Text(Text {
                color, color_dirty, ..
            }) => {
                *color = new_color;
                *color_dirty = true;
            }
            Graphic::None => {}
        }
    }
    pub fn set_alpha(&mut self, new_alpha: u8) {
        match self {
            Graphic::Panel(Panel {
                color, color_dirty, ..
            })
            | Graphic::Texture(Texture {
                color, color_dirty, ..
            })
            | Graphic::Text(Text {
                color, color_dirty, ..
            }) => {
                color[3] = new_alpha;
                *color_dirty = true;
            }
            Graphic::None => {}
        }
    }

    pub fn need_rebuild(&self) -> bool {
        match self {
            Graphic::Panel(_) => false,
            Graphic::Texture(_) => false,
            Graphic::Text(Text { text_dirty, .. }) => *text_dirty,
            Graphic::None => false,
        }
    }

    pub fn is_color_dirty(&self) -> bool {
        match self {
            Graphic::Panel(Panel { color_dirty, .. }) => *color_dirty,
            Graphic::Texture(Texture { color_dirty, .. }) => *color_dirty,
            Graphic::Text(Text { color_dirty, .. }) => *color_dirty,
            Graphic::None => false,
        }
    }

    pub fn clear_dirty(&mut self) {
        match self {
            Graphic::Panel(Panel { color_dirty, .. }) => *color_dirty = false,
            Graphic::Texture(Texture { color_dirty, .. }) => *color_dirty = false,
            Graphic::Text(Text {
                color_dirty,
                text_dirty,
                ..
            }) => {
                *color_dirty = false;
                *text_dirty = false;
            }
            Graphic::None => {}
        }
    }

    pub fn with_border(mut self, new_border: f32) -> Self {
        match self {
            Graphic::Panel(Panel { ref mut border, .. }) => {
                *border = new_border;
            }
            _ => panic!("call 'with_boder' in a non Graphic::Panel variant"),
        }
        self
    }

    pub fn set_text(&mut self, new_text: &str) {
        if let Graphic::Text(text) = self {
            text.set_text(new_text);
        }
    }

    pub fn compute_min_size(&mut self, fonts: &[FontArc]) -> Option<[f32; 2]> {
        if let Graphic::Text(text) = self {
            text.compute_min_size(fonts)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Painel {
    sprites: [Sprite; 9],
    border: f32,
}
impl Painel {
    #[allow(clippy::needless_range_loop)]
    pub fn new(texture: u32, uv_rect: [f32; 4], border: f32) -> Self {
        let w = uv_rect[2] / 3.0;
        let h = uv_rect[3] / 3.0;
        let sprite = Sprite {
            texture,
            color: [255; 4],
            rect: [0.0, 0.0, 1.0, 1.0],
            uv_rect: [0.0, 0.0, 0.0, 0.0],
        };
        let mut sprites: [Sprite; 9] = unsafe { std::mem::zeroed() };
        for i in 0..9 {
            sprites[i] = sprite.clone();
            sprites[i].uv_rect = [
                uv_rect[0] + w * (i % 3) as f32,
                uv_rect[1] + h * (i / 3) as f32,
                w,
                h,
            ];
        }
        Self { sprites, border }
    }
    #[inline]
    pub fn set_color(&mut self, color: [u8; 4]) {
        for sprite in self.sprites.iter_mut() {
            sprite.color = color;
        }
    }
    #[inline]
    pub fn with_color(mut self, color: [u8; 4]) -> Self {
        self.set_color(color);
        self
    }
    #[inline]
    pub fn set_border(&mut self, border: f32) {
        self.border = border;
    }
    #[inline]
    pub fn with_border(mut self, border: f32) -> Self {
        self.set_border(border);
        self
    }
    #[inline]
    pub fn set_rect(&mut self, rect: &Rect) {
        let rect = *rect.get_rect();
        let rect = [
            rect[0].round(),
            rect[1].round(),
            rect[2].round(),
            rect[3].round(),
        ];
        let width = (rect[2] - rect[0]).max(0.0);
        let height = (rect[3] - rect[1]).max(0.0);
        let border = self.border.min(width / 2.0).min(height / 2.0).round();
        let x1 = rect[0];
        let x2 = rect[0] + border;
        let x3 = rect[2] - border;

        let y1 = rect[1];
        let y2 = rect[1] + border;
        let y3 = rect[3] - border;

        let inner_width = width - border * 2.0;
        let inner_height = height - border * 2.0;

        for i in 0..9 {
            let x = [x1, x2, x3][i % 3];
            let y = [y1, y2, y3][i / 3];
            let w = [border, inner_width, border][i % 3];
            let h = [border, inner_height, border][i / 3];
            self.sprites[i].rect = [x, y, x + w, y + h];
        }
    }
    #[inline]
    pub fn get_sprites(&self) -> &[Sprite] {
        &self.sprites
    }
}

// pub struct Text {
//     text: String,
//     scale: f32,
//     align: (i8, i8),
//     color: [u8; 4],
// }
// impl Text {
//     pub fn new(text: String, scale: f32, align: (i8, i8)) -> Self {
//         Self {
//             text,
//             scale,
//             align,
//             color: [0, 0, 0, 255],
//         }
//     }

//     pub fn with_color(mut self, color: [u8; 4]) -> Self {
//         self.color = color;
//         self
//     }
//     pub fn set_color(&mut self, color: [u8; 4]) {
//         self.color = color;
//     }
//     pub fn set_scale(&mut self, scale: f32) {
//         self.scale = scale;
//     }
//     pub fn set_text(&mut self, text: &str) {
//         self.text = text.to_string();
//     }
// }

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
    color: [u8; 4],
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
