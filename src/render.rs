use super::GUI;
use glyph_brush::{ab_glyph::*, *};
use sprite_render::{Camera, Renderer, SpriteInstance, SpriteRender};

pub trait GUIRender: 'static {}

pub struct GUISpriteRender {
    painels: Vec<Painel>,
    texts: Vec<Text>,
    glyph_brush: GlyphBrush<SpriteInstance, Extra, FontVec>,
    font_texture: u32,
    sprites: Vec<SpriteInstance>,
}
impl GUIRender for GUISpriteRender {}
impl<'a> GUISpriteRender {
    pub fn new(fonts: Vec<Vec<u8>>, font_texture: u32) -> Self {
        let fonts = fonts
            .into_iter()
            .map(|font| FontVec::try_from_vec(font).unwrap())
            .collect();
        let mut glyph_brush = GlyphBrushBuilder::using_fonts(fonts).build();
        glyph_brush.resize_texture(1024, 1024);
        Self {
            painels: Vec::new(),
            texts: Vec::new(),
            glyph_brush,
            font_texture,
            sprites: Vec::new(),
        }
    }

    pub fn add_painel(&mut self, painel: Painel) -> GraphicId {
        let id = GraphicId::Panel {
            index: self.painels.len(),
            color: painel.sprites[0].color,
        };
        self.painels.push(painel);
        id
    }
    pub fn add_text(&mut self, text: Text) -> GraphicId {
        let id = GraphicId::Text {
            index: self.texts.len(),
            color: text.color,
        };
        self.texts.push(text);
        id
    }
    pub fn get_text(&mut self, index: usize) -> &mut Text {
        &mut self.texts[index]
    }

    pub fn prepare_render(gui: &mut GUI<Self>, renderer: &mut dyn SpriteRender) {
        use crate::ROOT_ID;
        let mut parents = vec![ROOT_ID];
        gui.render().sprites.clear();
        while let Some(parent) = parents.pop() {
            if let Some(graphic) = gui.get_graphic(parent) {
                let graphic = graphic.clone();
                let rect = &gui.get_rect(parent);
                let pos = rect.get_top_left();
                let center = rect.get_center();
                let mut size = rect.get_size();
                let rect = *rect.get_rect();
                size.0 = size.0.max(0.0);
                size.1 = size.1.max(0.0);
                match graphic {
                    GraphicId::Panel { index, color } => {
                        let render = gui.render();
                        render.painels[index].set_rect(&rect);
                        render.painels[index].set_color(color);
                        render
                            .sprites
                            .extend(render.painels[index].get_sprites().iter().cloned());
                    }
                    GraphicId::Text { index, color } => {
                        use std::cmp::Ordering;
                        let render: &mut GUISpriteRender = gui.render();
                        let text = &render.texts[index];
                        let layout = {
                            let hor = match text.align.0.cmp(&0) {
                                Ordering::Less => HorizontalAlign::Left,
                                Ordering::Equal => HorizontalAlign::Center,
                                Ordering::Greater => HorizontalAlign::Right,
                            };
                            let vert = match text.align.1.cmp(&0) {
                                Ordering::Less => VerticalAlign::Top,
                                Ordering::Equal => VerticalAlign::Center,
                                Ordering::Greater => VerticalAlign::Bottom,
                            };
                            Layout::default().h_align(hor).v_align(vert)
                        };
                        let pos = {
                            let x = match text.align.0.cmp(&0) {
                                Ordering::Less => pos.0,
                                Ordering::Equal => center.0,
                                Ordering::Greater => pos.0 + size.0,
                            };
                            let y = match text.align.1.cmp(&0) {
                                Ordering::Less => pos.1,
                                Ordering::Equal => center.1,
                                Ordering::Greater => pos.1 + size.1,
                            };
                            (x, y)
                        };
                        let string: &str = &text.text;
                        let color = [
                            color[0] as f32 / 255.0,
                            color[1] as f32 / 255.0,
                            color[2] as f32 / 255.0,
                            color[3] as f32 / 255.0,
                        ];
                        let base_text = glyph_brush::Text::new(string)
                            .with_scale(text.scale)
                            .with_color(color);
                        render.glyph_brush.queue(
                            Section::default()
                                .add_text(base_text)
                                .with_screen_position(pos)
                                .with_bounds(size)
                                .with_layout(layout),
                        );
                        let texture = render.font_texture;
                        let brush_action = render.glyph_brush.process_queued(
                            |rect, tex_data| {
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
                            },
                            to_vertex,
                        );
                        match brush_action.unwrap() {
                            BrushAction::Draw(mut vertices) => {
                                render.sprites.append(&mut vertices);
                            }
                            BrushAction::ReDraw => {}
                        }
                    }
                }
            }
            for child in gui.get_childs(parent) {
                parents.push(child);
            }
        }
    }

    pub fn render(&mut self, renderer: &mut dyn Renderer, camera: &mut Camera) {
        renderer.draw_sprites(camera, &self.sprites);
    }
}

#[derive(Clone)]
pub enum GraphicId {
    Panel { index: usize, color: [u8; 4] },
    Text { index: usize, color: [u8; 4] },
}
impl GraphicId {
    pub fn set_color(&mut self, new_color: [u8; 4]) {
        match self {
            GraphicId::Panel { ref mut color, .. } => {
                *color = new_color;
            }
            GraphicId::Text { ref mut color, .. } => {
                *color = new_color;
            }
        }
    }
    pub fn set_alpha(&mut self, new_alpha: u8) {
        match self {
            GraphicId::Panel { ref mut color, .. } => {
                color[3] = new_alpha;
            }
            GraphicId::Text { ref mut color, .. } => {
                color[3] = new_alpha;
            }
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
        let mut sprites: [SpriteInstance; 9] = unsafe {
            std::mem::zeroed()
        };
        for i in 0..9 {
            sprites[i] = sprite.clone().with_uv_rect([
                uv_rect[0] + w*(i%3) as f32,
                uv_rect[1] + h*(i/3) as f32,
                w,
                h,
            ]);
        }
        Self {
            sprites,
            border,
        }
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
    pub fn set_border(&mut self, border: f32){
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
        let width = (rect[2]-rect[0]).max(0.0);
        let height = (rect[3]-rect[1]).max(0.0);
        let border = self.border.min(width/2.0).min(height/2.0).round();
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

pub struct Text {
    text: String,
    scale: f32,
    align: (i8, i8),
    color: [u8; 4],
}
impl Text {
    pub fn new(text: String, scale: f32, align: (i8, i8)) -> Self {
        Self {
            text,
            scale,
            align,
            color: [0, 0, 0, 255],
        }
    }

    pub fn with_color(mut self, color: [u8; 4]) -> Self {
        self.color = color;
        self
    }
    pub fn set_color(&mut self, color: [u8; 4]) {
        self.color = color;
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }
}

#[inline]
pub fn to_vertex(
    glyph_brush::GlyphVertex {
        mut tex_coords,
        pixel_coords,
        bounds,
        extra,
    }: glyph_brush::GlyphVertex,
) -> SpriteInstance {
    let gl_bounds = bounds;

    let mut gl_rect = Rect {
        min: point(pixel_coords.min.x as f32, pixel_coords.min.y as f32),
        max: point(pixel_coords.max.x as f32, pixel_coords.max.y as f32),
    };

    // handle overlapping bounds, modify uv_rect to preserve texture aspect
    if gl_rect.max.x > gl_bounds.max.x {
        let old_width = gl_rect.width();
        gl_rect.max.x = gl_bounds.max.x;
        tex_coords.max.x = tex_coords.min.x + tex_coords.width() * gl_rect.width() / old_width;
    }
    if gl_rect.min.x < gl_bounds.min.x {
        let old_width = gl_rect.width();
        gl_rect.min.x = gl_bounds.min.x;
        tex_coords.min.x = tex_coords.max.x - tex_coords.width() * gl_rect.width() / old_width;
    }
    if gl_rect.max.y > gl_bounds.max.y {
        let old_height = gl_rect.height();
        gl_rect.max.y = gl_bounds.max.y;
        tex_coords.max.y = tex_coords.min.y + tex_coords.height() * gl_rect.height() / old_height;
    }
    if gl_rect.min.y < gl_bounds.min.y {
        let old_height = gl_rect.height();
        gl_rect.min.y = gl_bounds.min.y;
        tex_coords.min.y = tex_coords.max.y - tex_coords.height() * gl_rect.height() / old_height;
    }

    SpriteInstance {
        pos: [
            (gl_rect.min.x + gl_rect.max.x) / 2.0,
            (gl_rect.min.y + gl_rect.max.y) / 2.0,
        ],
        scale: [gl_rect.width(), gl_rect.height()],
        color: [
            (extra.color[0] * 255.0) as u8,
            (extra.color[1] * 255.0) as u8,
            (extra.color[2] * 255.0) as u8,
            (extra.color[3] * 255.0) as u8,
        ],
        angle: 0.0,
        texture: 1,
        uv_rect: [
            tex_coords.min.x,
            tex_coords.min.y,
            tex_coords.width(),
            tex_coords.height(),
        ],
    }
}
