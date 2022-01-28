use crate::{font::Fonts, Color};

pub use crate::text::{Text, TextStyle};

#[derive(Clone)]
pub struct Sprite {
    pub texture: u32,
    pub color: Color,
    /// A rect, in the form \[x1, y1, x2, y2\].
    pub rect: [f32; 4],
    pub uv_rect: [f32; 4],
}

#[derive(Clone, Debug)]
// #[cfg_attr(feature = "serde", derive(Deserialize))]
pub enum Graphic {
    Panel(Panel),
    Texture(Texture),
    Icon(Icon),
    AnimatedIcon(AnimatedIcon),
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
impl From<Icon> for Graphic {
    fn from(v: Icon) -> Self {
        Self::Icon(v)
    }
}
impl From<AnimatedIcon> for Graphic {
    fn from(v: AnimatedIcon) -> Self {
        Self::AnimatedIcon(v)
    }
}
impl From<Text> for Graphic {
    fn from(text: Text) -> Self {
        Self::Text(text)
    }
}
impl Graphic {
    pub fn with_color(mut self, new_color: Color) -> Self {
        self.set_color(new_color);
        self
    }

    pub fn get_color(&self) -> Color {
        match self {
            Graphic::Panel(Panel { color, .. })
            | Graphic::Texture(Texture { color, .. })
            | Graphic::Icon(Icon { color, .. })
            | Graphic::AnimatedIcon(AnimatedIcon { color, .. }) => *color,
            Graphic::Text(x) => x.color(),
            Graphic::None => [255, 255, 255, 255].into(),
        }
    }

    pub fn set_color(&mut self, new_color: Color) {
        match self {
            Graphic::Panel(Panel {
                color, color_dirty, ..
            })
            | Graphic::Texture(Texture {
                color, color_dirty, ..
            })
            | Graphic::Icon(Icon {
                color, color_dirty, ..
            })
            | Graphic::AnimatedIcon(AnimatedIcon {
                color, color_dirty, ..
            }) => {
                *color = new_color;
                *color_dirty = true;
            }
            Graphic::Text(x) => x.set_color(new_color),
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
            | Graphic::Icon(Icon {
                color, color_dirty, ..
            })
            | Graphic::AnimatedIcon(AnimatedIcon {
                color, color_dirty, ..
            }) => {
                color.a = new_alpha;
                *color_dirty = true;
            }
            Graphic::Text(x) => x.color_mut().a = new_alpha,
            Graphic::None => {}
        }
    }

    pub fn need_rebuild(&self) -> bool {
        match self {
            Graphic::Panel(_) => false,
            Graphic::Texture(_) => false,
            Graphic::Icon(_) => false,
            Graphic::AnimatedIcon(_) => true,
            Graphic::Text(Text { text_dirty, .. }) => *text_dirty,
            Graphic::None => false,
        }
    }

    pub fn is_color_dirty(&self) -> bool {
        match self {
            Graphic::Panel(Panel { color_dirty, .. })
            | Graphic::Texture(Texture { color_dirty, .. })
            | Graphic::Icon(Icon { color_dirty, .. })
            | Graphic::AnimatedIcon(AnimatedIcon { color_dirty, .. })
            | Graphic::Text(Text { color_dirty, .. }) => *color_dirty,
            Graphic::None => false,
        }
    }

    pub fn clear_dirty(&mut self) {
        match self {
            Graphic::Panel(Panel { color_dirty, .. }) => *color_dirty = false,
            Graphic::Texture(Texture { color_dirty, .. }) => *color_dirty = false,
            Graphic::Icon(Icon { color_dirty, .. }) => *color_dirty = false,
            Graphic::AnimatedIcon(AnimatedIcon { color_dirty, .. }) => *color_dirty = false,
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

    pub fn set_text(&mut self, new_text: &str) {
        if let Graphic::Text(text) = self {
            text.set_string(new_text);
        }
    }

    pub fn compute_min_size(&mut self, fonts: &Fonts) -> Option<[f32; 2]> {
        if let Graphic::Text(text) = self {
            text.compute_min_size(fonts)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct Icon {
    pub texture: u32,
    pub uv_rect: [f32; 4],
    pub size: [f32; 2],
    pub color: Color,
    pub color_dirty: bool,
}
impl Icon {
    pub fn new(texture: u32, uv_rect: [f32; 4], size: [f32; 2]) -> Self {
        Self {
            texture,
            uv_rect,
            size,
            color: [255, 255, 255, 255].into(),
            color_dirty: true,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.set_color(color);
        self
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
        self.color_dirty = true;
    }

    pub fn get_sprite(&self, rect: [f32; 4]) -> Sprite {
        let width = rect[2] - rect[0];
        let height = rect[3] - rect[1];
        let [w, h] = self.size;
        let x = rect[0] + (width - w) / 2.0;
        let y = rect[1] + (height - h) / 2.0;

        Sprite {
            texture: self.texture,
            color: self.color,
            rect: [x, y, x + w, y + h],
            uv_rect: self.uv_rect,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AnimatedIcon {
    pub texture: u32,
    pub fps: f32,
    pub curr_time: f32,
    pub frames: Vec<[f32; 4]>,
    pub size: [f32; 2],
    pub color: Color,
    pub color_dirty: bool,
}
impl AnimatedIcon {
    pub fn new(texture: u32, frames: Vec<[f32; 4]>, size: [f32; 2]) -> Self {
        Self {
            texture,
            frames,
            fps: 60.0,
            curr_time: 0.0,
            size,
            color: [255, 255, 255, 255].into(),
            color_dirty: true,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.set_color(color);
        self
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
        self.color_dirty = true;
    }

    pub fn get_sprite(&mut self, rect: [f32; 4], dt: f32) -> Sprite {
        let width = rect[2] - rect[0];
        let height = rect[3] - rect[1];
        let [w, h] = self.size;
        let x = rect[0] + (width - w) / 2.0;
        let y = rect[1] + (height - h) / 2.0;

        let sprite = Sprite {
            texture: self.texture,
            color: self.color,
            rect: [x, y, x + w, y + h],
            uv_rect: self.frames[(self.curr_time * self.fps) as usize],
        };

        self.curr_time = (self.curr_time + dt) % (self.frames.len() as f32 / self.fps);

        sprite
    }
}
#[derive(Debug)]
pub struct Texture {
    pub texture: u32,
    pub uv_rect: [f32; 4],
    pub color: Color,
    pub color_dirty: bool,
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
            color: [255, 255, 255, 255].into(),
            color_dirty: true,
        }
    }

    pub fn get_sprite(&self, rect: [f32; 4]) -> Sprite {
        Sprite {
            texture: self.texture,
            color: self.color,
            rect,
            uv_rect: self.uv_rect,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.set_color(color);
        self
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
        self.color_dirty = true;
    }
}

#[derive(Clone, Debug)]
pub struct Panel {
    pub texture: u32,
    pub uv_rects: [[f32; 4]; 9],
    pub border: [f32; 4],
    pub color: Color,
    pub color_dirty: bool,
}
impl Panel {
    #[allow(clippy::many_single_char_names)]
    pub fn new(texture: u32, uv_rect: [f32; 4], border: [f32; 4]) -> Self {
        // divide the given uv_rect in 9 equal sized ones
        let w = uv_rect[2];
        let h = uv_rect[3];

        let x = [uv_rect[0], uv_rect[0] + w / 3.0, uv_rect[0] + w * 2.0 / 3.0];
        let y = [uv_rect[1], uv_rect[1] + h / 3.0, uv_rect[1] + h * 2.0 / 3.0];

        let mut uv_rects = [[0.0; 4]; 9];
        for (i, uv_rect) in uv_rects.iter_mut().enumerate() {
            let n = i % 3;
            let m = i / 3;
            *uv_rect = [x[n], y[m], w / 3.0, h / 3.0];
        }

        Self {
            texture,
            uv_rects,
            border,
            color: [255, 255, 255, 255].into(),
            color_dirty: true,
        }
    }

    pub fn get_sprites(&self, rect: [f32; 4]) -> Vec<Sprite> {
        let width = (rect[2] - rect[0]).max(0.0);
        let height = (rect[3] - rect[1]).max(0.0);
        // TODO: make the border scale equaly
        let border = [
            self.border[0].min(width / 2.0).round(),
            self.border[1].min(height / 2.0).round(),
            self.border[2].min(width / 2.0).round(),
            self.border[3].min(height / 2.0).round(),
        ];
        let x1 = rect[0];
        let x2 = rect[0] + border[0];
        let x3 = rect[2] - border[2];

        let y1 = rect[1];
        let y2 = rect[1] + border[1];
        let y3 = rect[3] - border[3];

        let inner_width = x3 - x2;
        let inner_height = y3 - y2;

        let mut sprites = Vec::with_capacity(9);
        for i in 0..9 {
            let x = [x1, x2, x3][i % 3];
            let y = [y1, y2, y3][i / 3];
            let w = [border[0], inner_width, border[2]][i % 3];
            let h = [border[1], inner_height, border[3]][i / 3];
            sprites.push(Sprite {
                texture: self.texture,
                color: self.color,
                rect: [x, y, x + w, y + h],
                uv_rect: self.uv_rects[i],
            })
        }
        sprites
    }
}
