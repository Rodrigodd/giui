use super::{DirtyFlags, Id, Widgets};
use crate::Rect;
use ab_glyph::{point, ScaleFont};
use glyph_brush_draw_cache::{
    ab_glyph::{Font, FontArc, PxScale},
    DrawCache, DrawCacheBuilder,
};
use glyph_brush_layout::{
    ab_glyph, FontId, GlyphPositioner, HorizontalAlign, Layout, SectionGeometry, SectionGlyph,
    SectionText, VerticalAlign,
};
use sprite_render::{Camera, Renderer, SpriteInstance, SpriteRender};
use std::cmp::Ordering;
use std::ops::Range;

pub trait GUIRender: 'static {}

pub struct GUISpriteRender {
    draw_cache: DrawCache,
    font_texture: u32,
    last_sprites: Vec<SpriteInstance>,
    last_sprites_map: Vec<(Id, Range<usize>)>,
    sprites: Vec<SpriteInstance>,
    sprites_map: Vec<(Id, Range<usize>)>,
}
impl GUIRender for GUISpriteRender {}
impl<'a> GUISpriteRender {
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

    pub fn prepare_render(&mut self, widgets: &mut Widgets, renderer: &mut dyn SpriteRender) {
        use crate::ROOT_ID;
        let mut parents = vec![ROOT_ID];
        self.sprites.clear();
        self.sprites_map.clear();
        let mut masks: Vec<(usize, [f32; 4], bool)> = Vec::new();
        let fonts = widgets.get_fonts();

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

        while let Some(parent) = parents.pop() {
            let mut mask = None;
            let mut mask_changed = false;
            if let Some((i, m, changed)) = masks.last() {
                if parents.len() < *i {
                    masks.pop();
                    if let Some((_, m, changed)) = masks.last() {
                        mask = Some(*m);
                        mask_changed = *changed;
                    }
                } else {
                    mask = Some(*m);
                    mask_changed = *changed;
                }
            }

            if let Some((rect, graphic)) = widgets.get_rect_and_graphic(parent) {
                if let Some(mask) = mask {
                    if intersection(rect.get_rect(), &mask).is_none() {
                        // skip all its children
                        continue;
                    }
                }
                let mut compute_sprite = true;
                if let Graphic::Mask = graphic {
                    compute_sprite = false;
                    let changed = rect.get_dirty_flags().contains(DirtyFlags::RECT);
                    if let Some(mask) = mask {
                        match intersection(rect.get_rect(), &mask) {
                            Some(rect) => {
                                masks.push((parents.len(), rect, changed || mask_changed))
                            }
                            None => continue, // skip all its children
                        }
                    } else {
                        masks.push((parents.len(), *rect.get_rect(), changed));
                    }
                }

                let len = self.sprites.len();
                if !rect.get_dirty_flags().contains(DirtyFlags::RECT)
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
                            if let Some(mask) = mask {
                                for mut sprite in painel.get_sprites().iter().cloned() {
                                    if cut_sprite(&mut sprite, &mask) {
                                        self.sprites.push(sprite);
                                    }
                                }
                            } else {
                                self.sprites.extend(painel.get_sprites().iter().cloned());
                            }
                        }
                        Graphic::Texture(Texture {
                            texture,
                            uv_rect,
                            color,
                            ..
                        }) => {
                            let rect = rect;
                            let size = rect.get_size();
                            let center = rect.get_center();
                            let mut sprite = SpriteInstance {
                                scale: [size.0, size.1],
                                angle: 0.0,
                                uv_rect: *uv_rect,
                                color: *color,
                                pos: [center.0, center.1],
                                texture: *texture,
                            };
                            if let Some(mask) = mask {
                                if cut_sprite(&mut sprite, &mask) {
                                    self.sprites.push(sprite);
                                }
                            } else {
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
                            let texture = self.font_texture;
                            //TODO: I should queue all the glyphs, before calling cache_queued
                            self.draw_cache
                                .cache_queued(&fonts, |rect, tex_data| {
                                    let mut data = Vec::with_capacity(tex_data.len() * 4);
                                    for byte in tex_data.iter() {
                                        data.push(255);
                                        data.push(255);
                                        data.push(255);
                                        data.push(*byte);
                                    }
                                    renderer.update_texture(
                                        texture,
                                        &data,
                                        Some([
                                            rect.min[0],
                                            rect.min[1],
                                            rect.width(),
                                            rect.height(),
                                        ]),
                                    );
                                })
                                .unwrap();
                            let mut rect = *rect.get_rect();
                            if let Some(mask) = mask {
                                rect = intersection(&rect, &mask).unwrap_or_default();
                            }
                            for glyph in glyphs {
                                if let Some((tex_coords, pixel_coords)) =
                                    self.draw_cache.rect_for(glyph.font_id.0, &glyph.glyph)
                                {
                                    if pixel_coords.min.x as f32 > rect[2]
                                        || pixel_coords.min.y as f32 > rect[3]
                                        || rect[0] > pixel_coords.max.x as f32
                                        || rect[1] > pixel_coords.max.y as f32
                                    {
                                        // glyph is totally outside the bounds
                                    } else {
                                        self.sprites.push(to_vertex(
                                            tex_coords,
                                            pixel_coords,
                                            rect,
                                            color,
                                        ));
                                    }
                                }
                            }
                        }
                        Graphic::Mask => {}
                    }
                }
                graphic.clear_dirty();
                if len != self.sprites.len() {
                    self.sprites_map.push((parent, len..self.sprites.len()));
                }
            }
            widgets.get_rect(parent).clear_dirty_flags();
            parents.extend(widgets.get_children(parent).iter().rev())
        }
    }

    pub fn render(&mut self, renderer: &mut dyn Renderer, camera: &mut Camera) {
        renderer.draw_sprites(camera, &self.sprites);
        std::mem::swap(&mut self.sprites, &mut self.last_sprites);
        std::mem::swap(&mut self.sprites_map, &mut self.last_sprites_map);
    }
}

pub struct Text {
    color: [u8; 4],
    color_dirty: bool,
    text: String,
    text_dirty: bool,
    font_size: f32,
    align: (i8, i8),
    glyphs: Vec<SectionGlyph>,
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
            last_pos: [0.0, 0.0],
            min_size: None,
        }
    }

    fn dirty(&mut self) {
        self.text_dirty = true;
        self.min_size = None;
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

    pub fn get_glyphs<F: Font>(&mut self, rect: &mut Rect, fonts: &[F]) -> &[SectionGlyph] {
        let dirty_flags = rect.get_dirty_flags();
        let width_change = dirty_flags.contains(DirtyFlags::WIDTH)
            && self.min_size.map_or(true, |x| rect.get_width() < x[0]);
        if self.text_dirty || width_change {
            let layout = {
                let hor = match self.align.0.cmp(&0) {
                    Ordering::Less => HorizontalAlign::Left,
                    Ordering::Equal => HorizontalAlign::Center,
                    Ordering::Greater => HorizontalAlign::Right,
                };
                let vert = match self.align.1.cmp(&0) {
                    Ordering::Less => VerticalAlign::Top,
                    Ordering::Equal => VerticalAlign::Center,
                    Ordering::Greater => VerticalAlign::Bottom,
                };
                Layout::default().h_align(hor).v_align(vert)
            };
            let screen_position = {
                let x = match self.align.0.cmp(&0) {
                    Ordering::Less => rect.get_rect()[0],
                    Ordering::Equal => rect.get_center().0,
                    Ordering::Greater => rect.get_rect()[2],
                };
                let y = match self.align.1.cmp(&0) {
                    Ordering::Less => rect.get_rect()[1],
                    Ordering::Equal => rect.get_center().1,
                    Ordering::Greater => rect.get_rect()[3],
                };
                (x, y)
            };

            self.last_pos = self.get_align_anchor(*rect.get_rect());

            self.glyphs = layout.calculate_glyphs(
                &fonts,
                &SectionGeometry {
                    screen_position,
                    bounds: rect.get_size(),
                },
                &[SectionText {
                    text: &self.text,
                    scale: PxScale::from(self.font_size),
                    font_id: FontId(0),
                }],
            )
        } else if dirty_flags.contains(DirtyFlags::RECT) && !width_change {
            let rect = *rect.get_rect();
            let anchor = self.get_align_anchor(rect);
            let delta = [anchor[0] - self.last_pos[0], anchor[1] - self.last_pos[1]];
            self.last_pos = anchor;

            for glyph in &mut self.glyphs {
                glyph.glyph.position.x += delta[0];
                glyph.glyph.position.y += delta[1];
            }
        }
        &self.glyphs
    }

    pub fn compute_min_size<F: Font>(&mut self, fonts: &[F]) -> Option<[f32; 2]> {
        if self.min_size.is_none() {
            let layout = {
                let hor = match self.align.0.cmp(&0) {
                    Ordering::Less => HorizontalAlign::Left,
                    Ordering::Equal => HorizontalAlign::Center,
                    Ordering::Greater => HorizontalAlign::Right,
                };
                let vert = match self.align.1.cmp(&0) {
                    Ordering::Less => VerticalAlign::Top,
                    Ordering::Equal => VerticalAlign::Center,
                    Ordering::Greater => VerticalAlign::Bottom,
                };
                Layout::default().h_align(hor).v_align(vert)
            };

            let geometry = SectionGeometry::default();

            let glyphs = layout.calculate_glyphs(
                &fonts,
                &geometry,
                &[SectionText {
                    text: &self.text,
                    scale: PxScale::from(self.font_size),
                    font_id: FontId(0),
                }],
            );

            self.min_size = Some(
                glyphs
                    .iter()
                    .fold(None, |b: Option<ab_glyph::Rect>, sg| {
                        let sfont = fonts[sg.font_id.0].as_scaled(sg.glyph.scale);
                        let pos = sg.glyph.position;
                        let lbound = ab_glyph::Rect {
                            min: point(
                                pos.x - sfont.h_side_bearing(sg.glyph.id),
                                pos.y - sfont.ascent(),
                            ),
                            max: point(
                                pos.x + sfont.h_advance(sg.glyph.id),
                                pos.y - sfont.descent(),
                            ),
                        };
                        b.map(|b| {
                            let min_x = b.min.x.min(lbound.min.x);
                            let max_x = b.max.x.max(lbound.max.x);
                            let min_y = b.min.y.min(lbound.min.y);
                            let max_y = b.max.y.max(lbound.max.y);
                            ab_glyph::Rect {
                                min: point(min_x, min_y),
                                max: point(max_x, max_y),
                            }
                        })
                        .or_else(|| Some(lbound))
                    })
                    .map(|b| [b.width(), b.height()])
                    .unwrap_or([0.0, 0.0]),
            );
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
    Mask,
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
            Graphic::Mask => [255, 255, 255, 255],
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
            Graphic::Mask => {}
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
            Graphic::Mask => {}
        }
    }

    pub fn need_rebuild(&self) -> bool {
        match self {
            Graphic::Panel(_) => false,
            Graphic::Texture(_) => false,
            Graphic::Text(Text { text_dirty, .. }) => *text_dirty,
            Graphic::Mask => false,
        }
    }

    pub fn is_color_dirty(&self) -> bool {
        match self {
            Graphic::Panel(Panel { color_dirty, .. }) => *color_dirty,
            Graphic::Texture(Texture { color_dirty, .. }) => *color_dirty,
            Graphic::Text(Text { color_dirty, .. }) => *color_dirty,
            Graphic::Mask => false,
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
            Graphic::Mask => {}
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
    sprites: [SpriteInstance; 9],
    border: f32,
}
impl Painel {
    #[allow(clippy::needless_range_loop)]
    pub fn new(texture: u32, uv_rect: [f32; 4], border: f32) -> Self {
        let w = uv_rect[2] / 3.0;
        let h = uv_rect[3] / 3.0;
        let sprite = SpriteInstance::new(0.0, 0.0, 1.0, 1.0, texture, [0.0, 0.0, 1.0, 1.0]);
        let mut sprites: [SpriteInstance; 9] = unsafe { std::mem::zeroed() };
        for i in 0..9 {
            sprites[i] = sprite.clone().with_uv_rect([
                uv_rect[0] + w * (i % 3) as f32,
                uv_rect[1] + h * (i / 3) as f32,
                w,
                h,
            ]);
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
        let x1 = rect[0] + border / 2.0;
        let x2 = (rect[0] + rect[2]) / 2.0;
        let x3 = rect[2] - border / 2.0;

        let y1 = rect[1] + border / 2.0;
        let y2 = (rect[1] + rect[3]) / 2.0;
        let y3 = rect[3] - border / 2.0;

        let inner_width = width - border * 2.0;
        let inner_height = height - border * 2.0;

        for i in 0..9 {
            self.sprites[i].set_position([x1, x2, x3][i % 3], [y1, y2, y3][i / 3]);
            self.sprites[i].set_size(
                [border, inner_width, border][i % 3],
                [border, inner_height, border][i / 3],
            );
        }
    }
    #[inline]
    pub fn get_sprites(&self) -> &[SpriteInstance] {
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
pub fn cut_sprite(sprite: &mut SpriteInstance, bounds: &[f32; 4]) -> bool {
    let mut rect = [
        sprite.pos[0] - sprite.get_width() / 2.0,
        sprite.pos[1] - sprite.get_height() / 2.0,
        sprite.pos[0] + sprite.get_width() / 2.0,
        sprite.pos[1] + sprite.get_height() / 2.0,
    ];
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
    sprite.pos = [(rect[0] + rect[2]) / 2.0, (rect[1] + rect[3]) / 2.0];
    sprite.scale = [(rect[2] - rect[0]), (rect[3] - rect[1])];

    !(sprite.scale[0] < 0.0 || sprite.scale[1] < 0.0)
}

#[inline]
pub fn to_vertex(
    tex_coords: ab_glyph::Rect,
    pixel_coords: ab_glyph::Rect,
    bounds: [f32; 4],
    color: [u8; 4],
) -> SpriteInstance {
    let mut sprite = SpriteInstance {
        pos: [
            (pixel_coords.min.x + pixel_coords.max.x) / 2.0,
            (pixel_coords.min.y + pixel_coords.max.y) / 2.0,
        ],
        scale: [pixel_coords.width(), pixel_coords.height()],
        color,
        angle: 0.0,
        texture: 1,
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