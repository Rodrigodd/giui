use super::GUI;
use glyph_brush_draw_cache::{
    ab_glyph::{Font, FontArc, PxScale},
    DrawCache, DrawCacheBuilder,
};
use glyph_brush_layout::{ab_glyph::*, *};
use sprite_render::{Camera, Renderer, SpriteInstance, SpriteRender};
use std::cmp::Ordering;

pub trait GUIRender: 'static {}

pub struct GUISpriteRender {
    draw_cache: DrawCache,
    font_texture: u32,
    sprites: Vec<SpriteInstance>,
}
impl GUIRender for GUISpriteRender {}
impl<'a> GUISpriteRender {
    pub fn new(font_texture: u32) -> Self {
        //TODO: change this to default dimensions, and allow resizing
        let draw_cache = DrawCacheBuilder::default().dimensions(1024, 1024).build();
        Self {
            draw_cache,
            font_texture,
            sprites: Vec::new(),
        }
    }

    pub fn prepare_render(gui: &mut GUI<Self>, renderer: &mut dyn SpriteRender) {
        use crate::ROOT_ID;
        let mut parents = vec![ROOT_ID];
        gui.render().sprites.clear();
        let mut masks: Vec<(usize, [f32; 4])> = Vec::new();

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
            // println!("{}: {:?}", parents.len(), parent);
            let mut mask = None;
            if let Some((i, m)) = masks.last() {
                if parents.len() < *i {
                    masks.pop();
                    mask = masks.last().map(|x| &x.1);
                } else {
                    mask = Some(m);
                }
            }
            if let Some(graphic) = gui.get_graphic(parent) {
                let graphic = graphic.clone();
                let rect = &gui.get_rect(parent);
                let pos = rect.get_top_left();
                let center = rect.get_center();
                let mut size = rect.get_size();
                let mut rect = *rect.get_rect();
                size.0 = size.0.max(0.0);
                size.1 = size.1.max(0.0);
                if let Some(mask) = mask {
                    if intersection(&rect, &mask).is_none() {
                        // skip all its children
                        continue;
                    }
                }
                match graphic {
                    Graphic::Panel {
                        texture,
                        uv_rect,
                        color,
                        border,
                    } => {
                        let render = gui.render();
                        let mut painel = Painel::new(texture, uv_rect, border);
                        painel.set_rect(&rect);
                        painel.set_color(color);
                        if let Some(mask) = mask {
                            for mut sprite in painel.get_sprites().iter().cloned() {
                                if cut_sprite(&mut sprite, mask) {
                                    render.sprites.push(sprite);
                                }
                            }
                        } else {
                            render.sprites.extend(painel.get_sprites().iter().cloned());
                        }
                    }
                    Graphic::Text {
                        color,
                        text,
                        font_size,
                        align,
                    } => {
                        let fonts = gui.get_fonts();
                        let render: &mut GUISpriteRender = gui.render();
                        let layout = {
                            let hor = match align.0.cmp(&0) {
                                Ordering::Less => HorizontalAlign::Left,
                                Ordering::Equal => HorizontalAlign::Center,
                                Ordering::Greater => HorizontalAlign::Right,
                            };
                            let vert = match align.1.cmp(&0) {
                                Ordering::Less => VerticalAlign::Top,
                                Ordering::Equal => VerticalAlign::Center,
                                Ordering::Greater => VerticalAlign::Bottom,
                            };
                            Layout::default().h_align(hor).v_align(vert)
                        };
                        let screen_position = {
                            let x = match align.0.cmp(&0) {
                                Ordering::Less => pos.0,
                                Ordering::Equal => center.0,
                                Ordering::Greater => pos.0 + size.0,
                            };
                            let y = match align.1.cmp(&0) {
                                Ordering::Less => pos.1,
                                Ordering::Equal => center.1,
                                Ordering::Greater => pos.1 + size.1,
                            };
                            (x, y)
                        };

                        let glyphs = layout.calculate_glyphs(
                            &fonts,
                            &SectionGeometry {
                                screen_position,
                                bounds: size,
                            },
                            &[SectionText {
                                text: &text,
                                scale: PxScale::from(font_size),
                                font_id: FontId(0),
                            }],
                        );

                        for glyph in &glyphs {
                            render
                                .draw_cache
                                .queue_glyph(glyph.font_id.0, glyph.glyph.clone());
                        }
                        let texture = render.font_texture;
                        //TODO: I should queue all the glyphs, before calling cache_queued
                        render
                            .draw_cache
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
                                    Some([rect.min[0], rect.min[1], rect.width(), rect.height()]),
                                );
                            })
                            .unwrap();
                        if let Some(mask) = mask {
                            rect = intersection(&rect, mask).unwrap_or_default();
                        }
                        for glyph in &glyphs {
                            if let Some((tex_coords, pixel_coords)) =
                                render.draw_cache.rect_for(glyph.font_id.0, &glyph.glyph)
                            {
                                if pixel_coords.min.x as f32 > rect[2]
                                    || pixel_coords.min.y as f32 > rect[3]
                                    || rect[0] > pixel_coords.max.x as f32
                                    || rect[1] > pixel_coords.max.y as f32
                                {
                                    // glyph is totally outside the bounds
                                } else {
                                    render.sprites.push(to_vertex(
                                        tex_coords,
                                        pixel_coords,
                                        rect,
                                        color,
                                    ));
                                }
                            }
                        }
                    }
                    Graphic::Mask => {
                        if let Some(mask) = mask {
                            match intersection(&rect, mask) {
                                Some(rect) => masks.push((parents.len(), rect)),
                                None => continue, // skip all its children
                            }
                        } else {
                            masks.push((parents.len(), rect));
                        }
                    }
                }
            }
            parents.extend(gui.get_childs(parent).iter().rev())
        }
    }

    pub fn render(&mut self, renderer: &mut dyn Renderer, camera: &mut Camera) {
        renderer.draw_sprites(camera, &self.sprites);
    }
}

#[derive(Clone)]
pub enum Graphic {
    Panel {
        texture: u32,
        uv_rect: [f32; 4],
        color: [u8; 4],
        border: f32,
    },
    Text {
        color: [u8; 4],
        text: String,
        font_size: f32,
        align: (i8, i8),
    },
    Mask,
}
impl Graphic {
    pub fn with_color(mut self, new_color: [u8; 4]) -> Self {
        match self {
            Graphic::Panel { ref mut color, .. } => {
                *color = new_color;
            }
            Graphic::Text { ref mut color, .. } => {
                *color = new_color;
            }
            Graphic::Mask => {}
        }
        self
    }

    pub fn set_color(&mut self, new_color: [u8; 4]) {
        match self {
            Graphic::Panel { ref mut color, .. } => {
                *color = new_color;
            }
            Graphic::Text { ref mut color, .. } => {
                *color = new_color;
            }
            Graphic::Mask => {}
        }
    }
    pub fn set_alpha(&mut self, new_alpha: u8) {
        match self {
            Graphic::Panel { ref mut color, .. } => {
                color[3] = new_alpha;
            }
            Graphic::Text { ref mut color, .. } => {
                color[3] = new_alpha;
            }
            Graphic::Mask => {}
        }
    }

    pub fn with_border(mut self, new_border: f32) -> Self {
        match self {
            Graphic::Panel { ref mut border, .. } => {
                *border = new_border;
            }
            _ => panic!("call 'with_boder' in a non Graphic::Panel variant"),
        }
        self
    }

    pub fn set_text(&mut self, new_text: &str) {
        if let Graphic::Text { ref mut text, .. } = self {
            *text = new_text.to_owned();
        }
    }

    pub fn compute_min_size(&mut self, fonts: &[FontArc]) -> Option<[f32; 2]> {
        if let Graphic::Text {
            text,
            font_size,
            align,
            ..
        } = self
        {
            let layout = {
                let hor = match align.0.cmp(&0) {
                    Ordering::Less => HorizontalAlign::Left,
                    Ordering::Equal => HorizontalAlign::Center,
                    Ordering::Greater => HorizontalAlign::Right,
                };
                let vert = match align.1.cmp(&0) {
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
                    text: &text,
                    scale: PxScale::from(*font_size),
                    font_id: FontId(0),
                }],
            );

            glyphs
                .iter()
                .fold(None, |b: Option<Rect>, sg| {
                    let sfont = fonts[sg.font_id.0].as_scaled(sg.glyph.scale);
                    let pos = sg.glyph.position;
                    let lbound = Rect {
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
                        Rect {
                            min: point(min_x, min_y),
                            max: point(max_x, max_y),
                        }
                    })
                    .or_else(|| Some(lbound))
                })
                .map(|b| [b.width(), b.height()])
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
    pub fn set_rect(&mut self, rect: &[f32; 4]) {
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
    tex_coords: Rect,
    pixel_coords: Rect,
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
