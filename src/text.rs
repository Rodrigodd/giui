use std::ops::Range;

use crate::{
    font::{FontId, Fonts},
    render::FontGlyph,
    text_layout::{ColorRect, LayoutSettings, TextLayout},
    Color, Rect, RenderDirtyFlags,
};

pub mod editor;

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
                ( 2..5, TextStyle { color: blue, ..Default::default() }),
                ( 5..6, TextStyle { color: red, ..Default::default() }),
                ( 6..9, TextStyle { ..Default::default() }),
            ]
        );
        spanned.replace_range(3..7, "");
        assert_eq!(
            spanned.spans,
            vec![
                ( 0usize..1, TextStyle { ..Default::default() }),
                ( 1..2, TextStyle { color: gree, ..Default::default() }),
                ( 2..3, TextStyle { color: blue, ..Default::default() }),
                ( 3..5, TextStyle { ..Default::default() }),
            ]
        );
        assert_eq!(spanned.string, "01278");
        spanned.replace_range(3..3, "aaa");
        assert_eq!(spanned.string, "012aaa78");
        assert_eq!(
            spanned.spans,
            vec![
                ( 0..1, TextStyle { ..Default::default() }),
                ( 1..2, TextStyle { color: gree, ..Default::default() }),
                ( 2..3, TextStyle { color: blue, ..Default::default() }),
                ( 3..8, TextStyle { ..Default::default() }),
            ]
        );
    }

    #[test]
    fn replace_range() {
        let mut spanned = SpannedString::from_string("Hel".into(), Default::default());
        spanned.replace_range(1..2, "");
        assert_eq!(spanned.spans, vec![(0..2, Default::default())]);
    }

    #[test]
    fn replace_range2() {
        let mut spanned = SpannedString::from_string("01234567".into(), Default::default());
        spanned.push_str(
            "89a",
            TextStyle {
                font_size: 0.1,
                ..Default::default()
            },
        );
        spanned.push_str(
            "b",
            TextStyle {
                ..Default::default()
            },
        );

        spanned.replace_range(8..11, "_");
        assert_eq!(spanned.spans, vec![(0..10, Default::default())]);
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
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub color: Color,
    pub font_size: f32,
    pub font_id: FontId,
    pub background: Option<Color>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            font_size: 16.0,
            font_id: Default::default(),
            background: None,
        }
    }
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

    pub fn replace_range(&mut self, range: Range<usize>, string: &str) {
        let offset = string.len() as isize - range.len() as isize;
        // remove spans
        self.spans.retain(|(rb, sp)| {
            // range overlap a range
            if rb.start >= range.start && rb.end <= range.end {
                false
            } else {
                true
            }
        });
        let shift_range = |range: &mut Range<usize>| {
            if offset < 0 {
                range.start -= (-offset) as usize;
                range.end -= (-offset) as usize;
            } else {
                range.start += offset as usize;
                range.end += offset as usize;
            }
        };

        let mut insert_index = 0;
        let mut to_append = Vec::new();
        // cut and shift spans
        for (i, (rb, sp)) in self.spans.iter_mut().enumerate() {
            if rb.start > range.start && rb.end < range.end {
                unreachable!()
            } else if rb.start < range.start && rb.end > range.end {
                // range cut a range in three
                let mut r = range.end..rb.end;
                shift_range(&mut r);
                to_append.push((r, sp.clone()));
                rb.end = range.start;
            } else if rb.start < range.start && rb.end > range.start {
                // range.start cut a range
                rb.end = range.start;
            } else if rb.start < range.end && rb.end > range.end {
                // range.end cut a range
                rb.start = range.end;
                shift_range(rb);
            } else if rb.start >= range.end {
                // range is to the right, and must be shifted
                shift_range(rb);
            }

            if rb.end <= range.start {
                insert_index = i + 1;
            }
        }

        self.string.replace_range(range.clone(), string);
        if string.len() > 0 {
            let style = self.spans.get(0).map(|x| x.1.clone()).unwrap_or_default();
            // self.spans
            //     .insert(insert_index, (range.start..range.start + string.len(), style))
            self.spans
                .push((range.start..range.start + string.len(), style))
        }

        self.spans.append(&mut to_append);
        self.spans.sort_by_key(|x| x.0.start);
        self.spans.dedup_by(|a, b| {
            if a.1 == b.1 {
                b.0.end = a.0.end;
                true
            } else {
                false
            }
        });
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
            if rb.start >= range.start && rb.end <= range.end {
                // range overlap a range
                merge_span(sp, &span);
            } else if rb.start < range.start && rb.end > range.end {
                // range cut a range in three
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
        self.spans.dedup_by(|a, b| {
            if a.1 == b.1 {
                b.0.end = a.0.end;
                true
            } else {
                false
            }
        });
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

#[derive(Debug, Clone)]
enum InnerText {
    SpannedString(SpannedString),
    TextLayout(TextLayout),
    None,
}
impl Default for InnerText {
    fn default() -> Self {
        Self::None
    }
}
impl InnerText {
    fn to_spanned(&mut self) -> &mut SpannedString {
        match self {
            InnerText::SpannedString(x) => x,
            InnerText::TextLayout(x) => {
                if let InnerText::TextLayout(x) = std::mem::take(self) {
                    *self = Self::SpannedString(x.to_spanned());
                    self.to_spanned()
                } else {
                    unreachable!()
                }
            }
            _ => unreachable!(),
        }
    }

    fn is_spanned(&self) -> bool {
        matches!(self, Self::SpannedString(_))
    }

    fn as_layout(&mut self) -> &mut TextLayout {
        match self {
            InnerText::TextLayout(x) => x,
            _ => panic!("InnerText is not a TextLayout"),
        }
    }

    fn set_layout(&mut self, text_layout: TextLayout) {
        *self = Self::TextLayout(text_layout);
    }

    fn to_layout(&mut self, settings: &LayoutSettings, fonts: &Fonts) -> &mut TextLayout {
        let x = match std::mem::take(self) {
            InnerText::SpannedString(x) => x,
            InnerText::TextLayout(x) => x.to_spanned(),
            _ => unreachable!(),
        };
        *self = Self::TextLayout(TextLayout::new(x, settings, fonts));
        if let Self::TextLayout(x) = self {
            x
        } else {
            unreachable!()
        }
    }
}

#[derive(Debug)]
pub struct Text {
    text: InnerText,
    pub(crate) text_dirty: bool,
    glyphs: Vec<FontGlyph>,
    rects: Vec<ColorRect>,
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
            last_pos: Default::default(),
            min_size: Default::default(),
        }
    }
}
impl Text {
    pub fn new(text: String, align: (i8, i8), style: TextStyle) -> Self {
        Self {
            text: InnerText::SpannedString(SpannedString::from_string(text, style.clone())),
            style,
            align,
            color_dirty: true,
            text_dirty: true,
            glyphs: Default::default(),
            rects: Default::default(),
            last_pos: Default::default(),
            min_size: Default::default(),
        }
    }

    pub fn from_spanned_string(text: SpannedString, align: (i8, i8)) -> Self {
        Self {
            style: text.spans[0].1.clone(),
            text: InnerText::SpannedString(text),
            align,
            color_dirty: true,
            text_dirty: true,
            glyphs: Default::default(),
            rects: Default::default(),
            last_pos: Default::default(),
            min_size: Default::default(),
        }
    }

    pub fn dirty(&mut self) {
        self.text_dirty = true;
        self.min_size = None;
    }

    pub fn get_font_size(&mut self) -> f32 {
        self.style.font_size
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        for (_, style) in &mut self.text.to_spanned().spans {
            style.font_size = font_size;
        }
        self.style.font_size = font_size;
        self.dirty();
    }

    pub fn add_span(&mut self, range: Range<usize>, span: Span) {
        self.text.to_spanned().add_span(range, span);
        self.dirty();
    }

    pub fn clear_spans(&mut self) {
        self.text.to_spanned().set_style(self.style.clone());
        self.dirty();
    }

    pub fn set_text(&mut self, text: &str) {
        self.text.to_spanned().clear();
        self.text.to_spanned().push_str(text, self.style.clone());
        self.dirty();
    }

    pub fn set_text_layout(&mut self, text: TextLayout) {
        self.text.set_layout(text);
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
        use crate::text_layout::{HorizontalAlign::*, VerticalAlign::*};
        self.last_pos = self.get_align_anchor(*rect.get_rect());
        let rect = rect.get_rect();
        let (glyphs, rects) = self
            .text
            .to_layout(
                &LayoutSettings {
                    max_width: Some(rect[2] - rect[0]),
                    max_height: Some(rect[3] - rect[1]),
                    horizontal_align: [Left, Center, Right][(self.align.0 + 1) as usize],
                    vertical_align: [Top, Middle, Bottom][(self.align.1 + 1) as usize],
                    ..Default::default()
                },
                fonts,
            )
            .glyphs_and_rects();
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
    }

    pub fn get_layout(&mut self, fonts: &Fonts, rect: &mut Rect) -> &mut TextLayout {
        if self.text.is_spanned() {
            self.update_glyphs(rect, fonts);
        }
        self.text.as_layout()
    }

    pub fn get_glyphs_and_rects(
        &mut self,
        rect: &mut Rect,
        fonts: &Fonts,
    ) -> (&[FontGlyph], &[ColorRect]) {
        let dirty_flags = rect.get_render_dirty_flags();
        let width_change = dirty_flags.contains(RenderDirtyFlags::WIDTH)
            && self.min_size.map_or(true, |x| rect.get_width() < x[0]);
        if self.text.is_spanned() || self.text_dirty || width_change {
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
            self.text.to_layout(&Default::default(), fonts);
            self.min_size = Some(self.text.as_layout().min_size());
        }
        self.min_size
    }

    pub fn color(&self) -> Color {
        self.style.color
    }
}
