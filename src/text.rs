use std::ops::Range;

use crate::font::{FontId, Fonts};
use crate::render::FontGlyph;
use crate::text_layout::{ColorRect, TextLayout};
use crate::{Color, Rect, RenderDirtyFlags};

#[cfg(test)]
mod test {
    use super::{Span, SpannedString, TextStyle};

    #[rustfmt::skip]
    #[test]
    fn add_span() {
        let mut spanned = SpannedString::from_string("012345678".into(), Default::default());
        let red = [255, 0, 0, 255].into();
        let gree = [0, 255, 0, 255].into();
        let blue = [0, 0, 255, 255].into();
        spanned.add_span( 3..6, Span { color: Some(red), ..Default::default() });
        spanned.add_span( 1..4, Span { color: Some(gree), ..Default::default() });
        spanned.add_span( 2..5, Span { color: Some(blue), ..Default::default() });
        assert_eq!(
            spanned.spans,
            vec![
                ( 0usize..1, TextStyle { ..Default::default() }),
                ( 1..2, TextStyle { color: gree, ..Default::default() }),
                ( 2..3, TextStyle { color: blue, ..Default::default() }),
                ( 3..4, TextStyle { color: blue, ..Default::default() }),
                ( 4..5, TextStyle { color: blue, ..Default::default() }),
                ( 5..6, TextStyle { color: red, ..Default::default() }),
                ( 6..9, TextStyle { ..Default::default() }),
            ]
        );
    }
}

/// A partial description of the style of a section of text.
#[derive(Debug, Clone, Default)]
pub struct Span {
    pub color: Option<Color>,
    pub font_size: Option<f32>,
    pub font_id: Option<FontId>,
    pub background: Option<Color>,
}

/// A description of the style of a text.
#[derive(Debug, Clone, Default)]
pub struct TextStyle {
    pub color: Color,
    pub font_size: f32,
    pub font_id: FontId,
    pub background: Option<Color>,
}

impl PartialEq for TextStyle {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color
            && self.font_size == other.font_size
            && self.font_id == other.font_id
            && self.background == other.background
    }
}

impl Eq for TextStyle {}
impl TextStyle {
    pub fn with_color(self, color: Color) -> Self {
        Self { color, ..self }
    }
}

/// A String with sections of it associated with diferents styles.
#[derive(Default, Debug, Clone)]
pub struct SpannedString {
    pub(crate) string: String,
    pub(crate) spans: Vec<(Range<usize>, TextStyle)>,
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
            spans: vec![(
                0..string.len(),
                TextStyle {
                    color: style.color,
                    font_size: style.font_size,
                    font_id: style.font_id,
                    background: None,
                },
            )],
            string,
        }
    }

    pub fn push_str(&mut self, string: &str, span: TextStyle) {
        let start = self.string.len();
        self.string.push_str(string);
        self.spans.push((start..self.string.len(), span));
    }

    /// Clear all spans, and replace with only one, with the given style.
    pub fn set_style(&mut self, style: TextStyle) {
        self.spans.clear();
        self.spans.push((
            0..self.string.len(),
            TextStyle {
                color: style.color,
                font_size: style.font_size,
                font_id: style.font_id,
                background: None,
            },
        ));
    }

    pub fn add_span(&mut self, range: Range<usize>, span: Span) {
        let mut to_append = Vec::new();
        for (rb, sp) in &mut self.spans {
            if rb.start > range.start && rb.end < range.end {
                // range overlap a range
                merge_span(sp, &span);
            } else if rb.start < range.start && rb.end > range.end {
                // range cut a range in tree
                let mut style = sp.clone();
                merge_span(&mut style, &span);
                to_append.push((range.start..range.end, style));
                to_append.push((range.end..rb.end, sp.clone()));
                rb.end = range.start;
            } else if rb.start < range.start && rb.end > range.start {
                // range.start cut a range in two
                let mut style = sp.clone();
                merge_span(&mut style, &span);
                to_append.push((range.start..rb.end, style));
                rb.end = range.start;
            } else if rb.start < range.end && rb.end > range.end {
                // range.end cut a range in two
                let mut style = sp.clone();
                merge_span(&mut style, &span);
                to_append.push((rb.start..range.end, style));
                rb.start = range.end;
            }
        }
        self.spans.append(&mut to_append);
        self.spans.sort_by_key(|x| x.0.start);
    }
}

fn merge_span(style: &mut TextStyle, span: &Span) {
    *style = TextStyle {
        color: span.color.unwrap_or(style.color),
        font_size: span.font_size.unwrap_or(style.font_size),
        font_id: span.font_id.unwrap_or(style.font_id),
        background: span.background.or(span.background),
    }
}

#[derive(Debug)]
pub struct Text {
    text: SpannedString,
    pub(crate) text_dirty: bool,
    glyphs: Vec<FontGlyph>,
    rects: Vec<ColorRect>,
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
            rects: Default::default(),
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
            rects: Default::default(),
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
            rects: Default::default(),
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
        for (_, style) in &mut self.text.spans {
            style.font_size = font_size;
        }
        self.style.font_size = font_size;
        self.dirty();
    }

    pub fn add_span(&mut self, range: Range<usize>, span: Span) {
        self.text.add_span(range, span);
        self.dirty();
    }

    pub fn clear_spans(&mut self) {
        self.text.set_style(self.style.clone());
        self.dirty();
    }

    pub fn set_text(&mut self, text: &str) {
        self.text.clear();
        self.text.push_str(text, self.style.clone());
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
        let (glyphs, rects) = layout.glyphs_and_rects();
        self.glyphs = glyphs
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
        self.rects = rects
            .iter()
            .cloned()
            .map(|mut x| {
                x.rect[0] += rect[0];
                x.rect[1] += rect[1];
                x
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

    pub fn get_glyphs_and_rects(
        &mut self,
        rect: &mut Rect,
        fonts: &Fonts,
    ) -> (&[FontGlyph], &[ColorRect]) {
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
        (&self.glyphs, &self.rects)
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
