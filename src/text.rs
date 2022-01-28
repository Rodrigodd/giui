use std::ops::Range;

use crate::{
    font::{FontId, Fonts},
    render::FontGlyph,
    text::layout::{LayoutSettings, TextLayout},
    Color, Rect, RenderDirtyFlags,
};

use self::layout::ColorRect;

pub mod editor;
pub mod layout;
mod shaping;

#[cfg(test)]
mod test {
    use crate::font::FontId;
    use crate::Color;

    use super::{ShapeSpan, Span, SpannedString};

    #[rustfmt::skip]
    #[test]
    fn add_span() {
        let mut spanned = SpannedString::from_string("012345678".into(), Default::default());
        let a = FontId::new(1);
        let b = FontId::new(2);
        let c = FontId::new(3);
        spanned.add_span( 3..6, Span::FontId(a), );
        spanned.add_span( 1..4, Span::FontId(b),);
        spanned.add_span( 2..5, Span::FontId(c),);
        let default = ShapeSpan { byte_range: 0..0, font_size: 16.0, font_id: FontId::new(0) };
        assert_eq!(
            spanned.get_shape_spans(),
            vec![
                ShapeSpan { byte_range: 0..1, ..default.clone() },
                ShapeSpan { byte_range: 1..2, font_id: b, ..default.clone() },
                ShapeSpan { byte_range: 2..5, font_id: c, ..default.clone() },
                ShapeSpan { byte_range: 5..6, font_id: a, ..default.clone() },
                ShapeSpan { byte_range: 6..9, ..default.clone() },
            ]
        );
        spanned.replace_range(3..7, "");
        assert_eq!(
            spanned.get_shape_spans(),
            vec![
                ShapeSpan { byte_range: 0..1, ..default.clone() },
                ShapeSpan { byte_range: 1..2, font_id: b, ..default.clone() },
                ShapeSpan { byte_range: 2..3, font_id: c, ..default.clone() },
                ShapeSpan { byte_range: 3..5, ..default.clone() },
            ]
        );
        assert_eq!(spanned.string, "01278");
        spanned.replace_range(3..3, "aaa");
        assert_eq!(spanned.string, "012aaa78");
        assert_eq!(
            spanned.get_shape_spans(),
            vec![
                ShapeSpan { byte_range: 0..1, ..default.clone() },
                ShapeSpan { byte_range: 1..2, font_id: b, ..default.clone() },
                ShapeSpan { byte_range: 2..3, font_id: c, ..default.clone() },
                ShapeSpan { byte_range: 3..8, ..default.clone() },
            ]
        );
    }

    #[test]
    fn replace_range() {
        let mut spanned = SpannedString::from_string("Hel".into(), Default::default());
        spanned.replace_range(1..2, "");
        assert_eq!(
            spanned.get_shape_spans(),
            vec![ShapeSpan {
                byte_range: 0..2,
                font_size: 16.0,
                font_id: FontId::new(0),
            }]
        );
    }

    #[test]
    fn replace_range2() {
        let mut spanned = SpannedString::from_string("0123456789ab".into(), Default::default());
        spanned.add_span(8..11, Span::FontSize(0.1));
        spanned.replace_range(8..11, "_");
        assert_eq!(
            spanned.get_shape_spans(),
            vec![
                ShapeSpan {
                    byte_range: 0..8,
                    font_size: 16.0,
                    font_id: FontId::new(0),
                },
                ShapeSpan {
                    byte_range: 8..9,
                    font_size: 0.1,
                    font_id: FontId::new(0),
                },
                ShapeSpan {
                    byte_range: 9..10,
                    font_size: 16.0,
                    font_id: FontId::new(0),
                },
            ]
        );
    }

    #[test]
    fn replace_range3() {
        let mut spanned = SpannedString::from_string("012".into(), Default::default());
        spanned.replace_range(1..1, "_");
        assert_eq!(
            spanned.get_shape_spans(),
            vec![ShapeSpan {
                byte_range: 0..4,
                font_size: 16.0,
                font_id: FontId::new(0),
            }]
        );
        spanned.replace_range(1..2, "_");
        assert_eq!(
            spanned.get_shape_spans(),
            vec![ShapeSpan {
                byte_range: 0..4,
                font_size: 16.0,
                font_id: FontId::new(0),
            }]
        );
    }

    #[test]
    fn replace_all() {
        let mut spanned = SpannedString::from_string("test".into(), Default::default());
        assert_eq!(
            spanned.get_shape_spans(),
            vec![ShapeSpan {
                byte_range: 0..4,
                font_size: 16.0,
                font_id: FontId::new(0),
            }]
        );
        spanned.add_span(
            0..4,
            Span::Selection {
                bg: Color::WHITE,
                fg: None,
            },
        );
        spanned.replace_range(0..4, "r");
        assert_eq!(
            spanned.get_shape_spans(),
            vec![ShapeSpan {
                byte_range: 0..1,
                font_size: 16.0,
                font_id: FontId::new(0),
            }]
        );
    }

    #[test]
    fn split_shape() {
        let mut spanned = SpannedString::from_string("0123456789ab".into(), Default::default());
        spanned.split_shape_span(4);
        spanned.split_shape_span(7);
        spanned.split_shape_span(1);
        spanned.split_shape_span(4);
        spanned.split_shape_span(40);
        spanned.split_shape_span(0);
        let default = ShapeSpan {
            byte_range: 0..10,
            font_size: 16.0,
            font_id: FontId::new(0),
        };
        assert_eq!(
            spanned.get_shape_spans(),
            vec![
                ShapeSpan {
                    byte_range: 0..1,
                    ..default.clone()
                },
                ShapeSpan {
                    byte_range: 1..4,
                    ..default.clone()
                },
                ShapeSpan {
                    byte_range: 4..7,
                    ..default.clone()
                },
                ShapeSpan {
                    byte_range: 7..12,
                    ..default.clone()
                },
            ]
        );
    }
}

/// A span of text of certain shape. This contains all information necessary for text shaping.
#[derive(Debug, Clone)]
pub(crate) struct ShapeSpan {
    pub byte_range: Range<usize>,
    pub font_size: f32,
    pub font_id: FontId,
    // diretion
    // language
    // script
}
impl PartialEq for ShapeSpan {
    fn eq(&self, other: &Self) -> bool {
        self.byte_range == other.byte_range
            && self.font_size == other.font_size
            && self.font_id == other.font_id
    }
}
impl Eq for ShapeSpan {}
impl ShapeSpan {
    /// Check if the two ShapeSpan are equal, disregarding the range.
    fn cmp_shape(&self, other: &Self) -> bool {
        self.font_size == other.font_size && self.font_id == other.font_id
    }
}

pub type Key = u32;

#[derive(Debug, Clone, PartialEq)]
struct InnerSpan {
    key: Key,
    byte_range: Range<usize>,
    span_type: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Span {
    FontSize(f32),
    FontId(FontId),
    Color(Color),
    Selection { bg: Color, fg: Option<Color> },
    Underline(Option<Color>),
}
impl Span {
    /// Tells if this span changes the shape of the text (like the font or text direction)
    fn is_shape_span(&self) -> bool {
        matches!(self, Self::FontSize(_) | Self::FontId(_))
    }
}

/// A description of the style of a text.
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub color: Color,
    pub font_size: f32,
    pub font_id: FontId,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            font_size: 16.0,
            font_id: Default::default(),
        }
    }
}

impl PartialEq for TextStyle {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color
            && self.font_size == other.font_size
            && self.font_id == other.font_id
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
    shape_spans: Vec<ShapeSpan>,
    spans: Vec<InnerSpan>,
    default_style: TextStyle,
}
impl SpannedString {
    pub fn new(default_style: TextStyle) -> Self {
        Self {
            default_style,
            ..Default::default()
        }
    }

    /// Returns a reference to the underline string.
    pub fn string(&self) -> &str {
        &self.string
    }

    /// Remove all spans
    pub fn clear_spans(&mut self) {
        self.clear_shape_spans();
        self.spans.clear();
    }

    /// Clear the text, and all spans
    pub fn clear(&mut self) {
        self.string.clear();
        self.clear_spans();
    }

    /// Clear shape_spans. shape_spans is computed lazily.
    fn clear_shape_spans(&mut self) {
        self.shape_spans.clear()
    }

    /// Compute if need, and return the shape_spans
    fn get_shape_spans(&mut self) -> &[ShapeSpan] {
        if self.shape_spans.len() == 0 {
            self.shape_spans.push(ShapeSpan {
                byte_range: 0..self.string.len(),
                font_size: self.default_style.font_size,
                font_id: self.default_style.font_id,
            });
            let spans = std::mem::take(&mut self.spans);
            for span in &spans {
                if span.span_type.is_shape_span() {
                    self.add_shape_span(span.byte_range.clone(), span.span_type);
                }
            }
            self.spans = spans;
        }
        debug_assert_eq!(self.shape_spans[0].byte_range.start, 0);
        &self.shape_spans
    }

    pub fn from_string(string: String, style: TextStyle) -> Self {
        Self {
            shape_spans: vec![ShapeSpan {
                byte_range: 0..string.len(),
                font_size: style.font_size,
                font_id: style.font_id,
            }],
            spans: Vec::new(),
            default_style: style,
            string,
        }
    }

    // pub fn push_str(&mut self, string: &str, span: TextStyle) {
    //     let start = self.string.len();
    //     self.string.push_str(string);
    //     self.spans.push((start..self.string.len(), span));
    // }

    /// Replace a range of text with another. Any span that contains the start of that range will
    /// be exapaded to cover the replaced text. Spans that are entirely contained in the replace
    /// range will be removed. This may modify the range of any span after the replaced range.
    pub fn replace_range(&mut self, range: Range<usize>, string: &str) {
        // rename for readibilty
        let mut replace_range = range;

        if !replace_range.is_empty() {
            assert!(replace_range.end <= self.string.len());
        } else {
            replace_range.end = replace_range.start;
        }
        assert!(replace_range.start <= self.string.len());

        let overlap = |span_range: Range<usize>| {
            // replace_range overlap span_range

            // if the string is not empty, a non-empty span starting at replace_range.start will
            // be non-empty. Otherwise it will be empty.
            if string.is_empty() {
                replace_range.start <= span_range.start && span_range.end <= replace_range.end
            } else {
                replace_range.start < span_range.start && span_range.end <= replace_range.end
            }
        };

        // remove spans contained enturely in the replaced range
        self.shape_spans.retain(|x| !overlap(x.byte_range.clone()));
        self.spans.retain(|x| !overlap(x.byte_range.clone()));

        // shift spans
        for span_range in self
            .shape_spans
            .iter_mut()
            .map(|x| &mut x.byte_range)
            .chain(self.spans.iter_mut().map(|x| &mut x.byte_range))
        {
            let shift = |x: &mut usize| {
                if *x <= replace_range.start {
                    // noop
                } else if *x < replace_range.end {
                    // snap to end
                    *x = replace_range.start + string.len();
                } else {
                    // shift
                    *x -= replace_range.len();
                    *x += string.len();
                }
            };
            shift(&mut span_range.start);
            shift(&mut span_range.end);
            // ensure that len is not 0, because they were removed before
        }

        self.string.replace_range(replace_range.clone(), string);
    }

    /// Set the default style. This will not remove any spans that could be overwriting the default
    /// style.
    pub fn set_style(&mut self, style: TextStyle) {
        self.default_style = style;
    }

    pub fn remove_span(&mut self, key: Key) -> Option<(Range<usize>, Span)> {
        let index = self.spans.iter().position(|x| x.key == key)?;
        let removed = self.spans.remove(index);
        Some((removed.byte_range, removed.span_type))
    }

    pub fn add_span(&mut self, range: Range<usize>, span: Span) -> Key {
        use std::sync::atomic::AtomicU32;
        static NEXT_KEY: AtomicU32 = AtomicU32::new(0);
        let key = NEXT_KEY.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        self.clear_shape_spans();
        self.spans.push(InnerSpan {
            key,
            byte_range: range,
            span_type: span,
        });
        key
    }

    fn add_shape_span(&mut self, range: Range<usize>, span: Span) {
        let mut to_append = Vec::new();
        for shape in &mut self.shape_spans {
            let rb = shape.byte_range.clone();
            if rb.start >= range.start && rb.end <= range.end {
                // range overlap a range
                merge_shape_span(shape, &span);
            } else if rb.start < range.start && rb.end > range.end {
                // range cut a range in three
                {
                    let mut middle = shape.clone();
                    merge_shape_span(&mut middle, &span);
                    middle.byte_range = range.clone();
                    to_append.push(middle);
                }
                {
                    let mut end = shape.clone();
                    end.byte_range = range.end..rb.end;
                    to_append.push(end);
                }
                shape.byte_range.end = range.start;
            } else if rb.start < range.start && rb.end > range.start {
                // range.start cut a range in two
                {
                    let mut end = shape.clone();
                    merge_shape_span(&mut end, &span);
                    end.byte_range = range.start..rb.end;
                    to_append.push(end);
                }
                shape.byte_range.end = range.start;
            } else if rb.start < range.end && rb.end > range.end {
                // range.end cut a range in two
                {
                    let mut beggin = shape.clone();
                    merge_shape_span(&mut beggin, &span);
                    beggin.byte_range = rb.start..range.end;
                    to_append.push(beggin);
                }
                shape.byte_range.start = range.end;
            }
        }
        self.shape_spans.append(&mut to_append);
        self.shape_spans.sort_by_key(|x| x.byte_range.start);
        self.shape_spans.dedup_by(|a, b| {
            if a.cmp_shape(b) {
                b.byte_range.end = a.byte_range.end;
                true
            } else {
                false
            }
        });
    }

    /// Find the shape span that contains the index, and split it in two. This is used in text
    /// shaping for splitting at hard breaklines.
    fn split_shape_span(&mut self, index: usize) {
        // compute shape spans
        self.get_shape_spans();

        let i = match self
            .shape_spans
            .binary_search_by(|x| crate::util::cmp_range(index, x.byte_range.clone()))
        {
            Ok(x) => x,
            // out of bounds, ignore
            Err(_) => return,
        };

        let span = &mut self.shape_spans[i];
        if span.byte_range.start == index {
            // there is already a split there
            return;
        }

        let mut clone = span.clone();
        span.byte_range.end = index;
        clone.byte_range.start = index;
        self.shape_spans.insert(i + 1, clone);
    }
}

fn merge_shape_span(a: &mut ShapeSpan, b: &Span) {
    match b {
        Span::FontSize(x) => a.font_size = *x,
        Span::FontId(x) => a.font_id = *x,
        _ => unreachable!("the span b is not type shape"),
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
    fn as_spanned(&self) -> &SpannedString {
        match self {
            InnerText::SpannedString(x) => x,
            InnerText::TextLayout(x) => x.spanned(),
            InnerText::None => unreachable!(),
        }
    }
    fn to_spanned(&mut self) -> &mut SpannedString {
        match self {
            InnerText::SpannedString(x) => x,
            InnerText::TextLayout(_) => {
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
        *self = Self::TextLayout(TextLayout::new(x, settings.clone(), fonts));
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
    min_size: Option<[f32; 2]>,
    last_pos: [f32; 2],
    align: (i8, i8),
    wrap_line: bool,
    glyphs: Vec<FontGlyph>,
    rects: Vec<ColorRect>,
    pub(crate) color_dirty: bool,
}
impl Clone for Text {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            align: self.align,
            wrap_line: true,
            color_dirty: true,
            text_dirty: true,
            last_pos: Default::default(),
            glyphs: Vec::new(),
            rects: Vec::new(),
            min_size: Default::default(),
        }
    }
}
impl Text {
    pub fn new(text: String, align: (i8, i8), style: TextStyle) -> Self {
        Self {
            text: InnerText::SpannedString(SpannedString::from_string(text, style.clone())),
            align,
            wrap_line: true,
            color_dirty: true,
            text_dirty: true,
            last_pos: Default::default(),
            min_size: Default::default(),
            glyphs: Vec::new(),
            rects: Vec::new(),
        }
    }

    pub fn from_spanned_string(text: SpannedString, align: (i8, i8)) -> Self {
        Self {
            text: InnerText::SpannedString(text),
            align,
            wrap_line: true,
            color_dirty: true,
            text_dirty: true,
            last_pos: Default::default(),
            min_size: Default::default(),
            glyphs: Vec::new(),
            rects: Vec::new(),
        }
    }

    pub fn dirty(&mut self) {
        self.text_dirty = true;
        self.min_size = None;
    }

    pub fn get_font_size(&mut self) -> f32 {
        self.text.as_spanned().default_style.font_size
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        let spanned = self.text.to_spanned();
        spanned.default_style.font_size = font_size;
        for span in &mut spanned.shape_spans {
            span.font_size = font_size;
        }
        self.dirty();
    }

    pub fn remove_span(&mut self, key: Key) {
        self.text.to_spanned().remove_span(key);
        self.dirty();
    }

    pub fn add_span(&mut self, range: Range<usize>, span: Span) -> Key {
        self.dirty();
        self.text.to_spanned().add_span(range, span)
    }

    pub fn clear_spans(&mut self) {
        let spanned = self.text.to_spanned();
        spanned.clear_spans();
        self.dirty();
    }

    /// Return the length of the underlining string
    pub fn len(&self) -> usize {
        self.text.as_spanned().string().len()
    }

    /// Return a reference to the underlining string
    pub fn string(&self) -> &str {
        self.text.as_spanned().string()
    }

    /// Set the value of the underlining string, and clear all spans
    pub fn set_string(&mut self, text: &str) {
        let spanned = self.text.to_spanned();
        let style = spanned.default_style.clone();
        *spanned = SpannedString::from_string(text.into(), style);
        self.dirty();
    }

    pub fn set_wrap(&mut self, wrap: bool) {
        self.wrap_line = wrap;
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

    fn update_glyphs(&mut self, rect: &Rect, fonts: &Fonts) {
        use crate::text::layout::Alignment::*;
        let anchor_pos = self.get_align_anchor(*rect.get_rect());
        self.last_pos = anchor_pos;
        let rect = rect.get_rect();
        let layout = self.text.to_layout(
            &LayoutSettings {
                max_width: self.wrap_line.then(|| rect[2] - rect[0]),
                horizontal_align: [Start, Center, End][(self.align.0 + 1) as usize],
                vertical_align: [Start, Center, End][(self.align.1 + 1) as usize],
            },
            fonts,
        );
        self.glyphs = layout
            .glyphs()
            .iter()
            .map(|x| {
                let mut glyph = x.glyph.clone();
                glyph.position.x += anchor_pos[0];
                glyph.position.y += anchor_pos[1];
                FontGlyph {
                    glyph,
                    font_id: x.font_id,
                    color: x.color,
                }
            })
            .collect();
        self.rects = layout
            .rects()
            .iter()
            .cloned()
            .map(|mut x| {
                x.rect[0] += anchor_pos[0];
                x.rect[1] += anchor_pos[1];
                x.rect[2] += anchor_pos[0];
                x.rect[3] += anchor_pos[1];
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
        rect: &Rect,
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
            for rect in &mut self.rects {
                rect.rect[0] += delta[0];
                rect.rect[1] += delta[1];
                rect.rect[2] += delta[0];
                rect.rect[3] += delta[1];
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
        self.text.as_spanned().default_style.color
    }

    pub fn color_mut(&mut self) -> &mut Color {
        &mut self.text.to_spanned().default_style.color
    }

    pub fn set_color(&mut self, color: Color) {
        self.text.to_spanned().default_style.color = color;
    }
}
