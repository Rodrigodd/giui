use crate::font::{FontId, Fonts};
use crate::render::FontGlyph;
use crate::text_layout::TextLayout;
use crate::{Color, Rect, RenderDirtyFlags};

/// A description of the style of a text.
#[derive(Debug, Clone, Default)]
pub struct TextStyle {
    pub color: Color,
    pub font_size: f32,
    pub font_id: FontId,
    pub background: Option<Color>,
}
impl TextStyle {
    pub fn with_color(self, color: Color) -> Self {
        Self { color, ..self }
    }
}

/// A String with sections of it associated with diferents styles.
#[derive(Default, Debug, Clone)]
pub struct SpannedString {
    pub(crate) string: String,
    pub(crate) spans: Vec<(usize, TextStyle)>,
}
impl SpannedString {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.string.clear();
        self.spans.clear();
    }

    pub fn from_string(string: String, style: TextStyle) -> Self {
        Self {
            string,
            spans: vec![(
                0,
                TextStyle {
                    color: style.color,
                    font_size: style.font_size,
                    font_id: style.font_id,
                    background: None,
                },
            )],
        }
    }

    pub fn push_str(&mut self, string: &str, span: TextStyle) {
        let start = self.string.len();
        self.string.push_str(string);
        self.spans.push((start, span));
    }
}

#[derive(Debug)]
pub struct Text {
    text: SpannedString,
    pub(crate) text_dirty: bool,
    glyphs: Vec<FontGlyph>,
    layout: Option<TextLayout>,
    min_size: Option<[f32; 2]>,
    last_pos: [f32; 2],
    align: (i8, i8),
    pub(crate) color_dirty: bool,
    pub(crate) style: TextStyle,
}
impl Clone for Text {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            style: self.style.clone(),
            align: self.align,
            color_dirty: true,
            text_dirty: true,
            glyphs: Default::default(),
            layout: None,
            last_pos: Default::default(),
            min_size: Default::default(),
        }
    }
}
impl Text {
    pub fn new(text: String, align: (i8, i8), style: TextStyle) -> Self {
        Self {
            text: SpannedString::from_string(text, style.clone()),
            style,
            align,
            color_dirty: true,
            text_dirty: true,
            glyphs: Default::default(),
            layout: None,
            last_pos: Default::default(),
            min_size: Default::default(),
        }
    }

    pub fn from_spanned_string(text: SpannedString, align: (i8, i8)) -> Self {
        Self {
            style: text.spans[0].1.clone(),
            text,
            align,
            color_dirty: true,
            text_dirty: true,
            glyphs: Default::default(),
            layout: None,
            last_pos: Default::default(),
            min_size: Default::default(),
        }
    }

    pub fn dirty(&mut self) {
        self.text_dirty = true;
        self.min_size = None;
        self.layout = None;
    }

    pub fn get_font_size(&mut self) -> f32 {
        self.style.font_size
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        self.style.font_size = font_size;
        self.dirty();
    }

    pub fn set_text(&mut self, text: &str) {
        self.text.clear();
        self.text.push_str(text, TextStyle {
            color: self.style.color,
            font_size: self.style.font_size,
            font_id: self.style.font_id,
            background: None,
        });
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

    fn update_glyphs(&mut self, rect: &mut Rect, fonts: &Fonts) {
        use crate::text_layout::{HorizontalAlign::*, LayoutSettings, VerticalAlign::*};
        self.last_pos = self.get_align_anchor(*rect.get_rect());
        let mut layout = TextLayout::new();
        let rect = rect.get_rect();
        layout.reset(&LayoutSettings {
            max_width: Some(rect[2] - rect[0]),
            max_height: Some(rect[3] - rect[1]),
            horizontal_align: [Left, Center, Right][(self.align.0 + 1) as usize],
            vertical_align: [Top, Middle, Bottom][(self.align.1 + 1) as usize],
            ..Default::default()
        });
        layout.layout(fonts, &self.text);
        self.glyphs = layout
            .glyphs()
            .iter()
            .map(|x| {
                let mut glyph = x.glyph.clone();
                glyph.position.x += rect[0];
                glyph.position.y += rect[1];
                FontGlyph {
                    glyph,
                    font_id: x.font_id,
                    color: x.color,
                }
            })
            .collect();
        self.layout = Some(layout);
    }

    pub fn get_layout(&mut self, fonts: &Fonts, rect: &mut Rect) -> &TextLayout {
        if self.layout.is_none() {
            self.update_glyphs(rect, fonts);
        }
        self.layout.as_ref().unwrap()
    }

    pub fn get_glyphs(&mut self, rect: &mut Rect, fonts: &Fonts) -> &[FontGlyph] {
        let dirty_flags = rect.get_render_dirty_flags();
        let width_change = dirty_flags.contains(RenderDirtyFlags::WIDTH)
            && self.min_size.map_or(true, |x| rect.get_width() < x[0]);
        if self.layout.is_none() || self.text_dirty || width_change {
            self.update_glyphs(rect, fonts);
        } else if dirty_flags.contains(RenderDirtyFlags::RECT) {
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

    pub fn compute_min_size(&mut self, fonts: &Fonts) -> Option<[f32; 2]> {
        if self.min_size.is_none() {
            let mut layout = TextLayout::new();
            layout.layout(fonts, &self.text);
            self.min_size = Some(layout.min_size());
        }
        self.min_size
    }

    pub fn color(&self) -> Color {
        self.style.color
    }
}
