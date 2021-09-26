use std::cmp::Ordering;
use std::collections::VecDeque;
use std::ops::Range;

use ab_glyph::{Font, Glyph, ScaleFont};

use crate::font::{FontId, Fonts};
use crate::text::SpannedString;
use crate::util::cmp_range;
use crate::Color;

use super::{ShapeSpan, StyleKind, StyleSpan};

#[cfg(test)]
mod test {
    use crate::font::{Font, FontId, Fonts};
    use crate::text::layout::{LayoutSettings, TextLayout};
    use crate::text::{Span, SpannedString, TextStyle};
    use crate::Color;

    fn fonts() -> (Fonts, Vec<FontId>) {
        let mut fonts = Fonts::new();
        let font_ids = vec![fonts.add(Font::new(include_bytes!(
            "..\\..\\examples\\CascadiaCode.ttf"
        ))),
        fonts.add(Font::new(include_bytes!(
            "..\\..\\examples\\CascadiaCode.ttf"
        )))];
        (fonts, font_ids)
    }

    #[test]
    fn layout_empty() {
        let (fonts, font_ids) = fonts();
        let font_id = font_ids[0];
        let text = SpannedString::from_string(
            "".to_string(),
            TextStyle {
                color: Color::WHITE,
                font_size: 16.0,
                font_id,
            },
        );
        let settings = LayoutSettings {
            max_width: None,
            horizontal_align: Default::default(),
            vertical_align: Default::default(),
        };
        let _text_layout = TextLayout::new(text, settings, &fonts);
    }

    #[test]
    fn replace_nothing() {
        let (fonts, font_ids) = fonts();
        let font_id = font_ids[0];
        let text = SpannedString::from_string(
            "H ".to_string(),
            TextStyle {
                color: Color::WHITE,
                font_size: 16.0,
                font_id,
            },
        );
        let settings = LayoutSettings {
            max_width: None,
            horizontal_align: Default::default(),
            vertical_align: Default::default(),
        };
        let mut text_layout = TextLayout::new(text, settings, &fonts);

        let lines = text_layout.lines.clone();
        let glyphs = text_layout.glyphs.clone();
        let rects = text_layout.rects.clone();
        let min_size = text_layout.min_size;

        text_layout.replace_range(0..0, "", &fonts);

        assert_eq!(
            glyphs,
            text_layout.glyphs,
            "byte_ranges:\n{:?}\n{:?}\n",
            glyphs
                .iter()
                .map(|x| x.glyph.position.clone())
                .collect::<Vec<_>>(),
            text_layout
                .glyphs
                .iter()
                .map(|x| x.glyph.position.clone())
                .collect::<Vec<_>>(),
        );
        assert_eq!(lines, text_layout.lines);
        assert_eq!(rects, text_layout.rects);
        assert_eq!(min_size, text_layout.min_size);
    }

    #[test]
    fn zero_width() {
        let (fonts, font_ids) = fonts();
        let font_id = font_ids[0];
        
        let text = SpannedString::from_string(
            "0123456".to_string(),
            TextStyle {
                color: Color::WHITE,
                font_size: 16.0,
                font_id,
            },
        );

        let settings = LayoutSettings {
            max_width: Some(0.0),
            horizontal_align: Default::default(),
            vertical_align: Default::default(),
        };
        let _text_layout = TextLayout::new(text, settings, &fonts);
    }

    #[test]
    fn multi_style() {
        let (fonts, font_ids) = fonts();
        
        let mut text = SpannedString::from_string(
            "0123456".to_string(),
            TextStyle {
                color: Color::WHITE,
                font_size: 16.0,
                font_id: font_ids[0],
            },
        );

        let settings = LayoutSettings {
            max_width: Some(20.0),
            horizontal_align: Default::default(),
            vertical_align: Default::default(),
        };
        let text_layout = TextLayout::new(text.clone(), settings.clone(), &fonts);

        let other_style = Span::FontId(font_ids[1]);
        text.add_span(0..1, other_style);
        text.add_span(2..3, other_style);
        text.add_span(4..5, other_style);
        text.add_span(6..7, other_style);

        let text_layout2 = TextLayout::new(text, settings, &fonts);

        assert_eq!(text_layout.lines(), text_layout2.lines());
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Alignment {
    /// If the alignment is horizontal, align to the left. If is vertical, to the top.
    Start,
    /// Align to the center.
    Center,
    /// If the alignment is horizontal, align to the right. If is vertical, to the bottom.
    End,
}
impl Default for Alignment {
    fn default() -> Self {
        Self::Start
    }
}

/// The settings of the text layout.
#[derive(Clone, Debug, Default)]
pub struct LayoutSettings {
    /// The max width of the text layout. Any line of text that exceeds this width suffers a line
    /// break at the last break opportunity, as specified in UAX #14. If the line don't have a
    /// break opportunity, the line will be break at the first glyph to overflow. If there is only
    /// one glyph, the line will overflow.
    pub max_width: Option<f32>,
    /// The horizontal alignment of the text. The text is aligned towards the origin, (0, 0). If
    /// it have right alignment, for example, all glyphs will have a negative x position.
    pub horizontal_align: Alignment,
    /// The vertical alignment of the text. The text is aligned towards the origin, (0, 0). If it
    /// have bottom alignment, for example, all glyphs will have a negative y position.
    pub vertical_align: Alignment,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Line {
    /// The position of the top of the line, in pixels, relative to the line origin. It grows up.
    pub ascent: f32,
    /// The position of the bottom of the line, in pixels, relative to the line origin. It grows
    /// up. It is usually negative.
    pub descent: f32,
    /// The size of the gap beetween the bottom of this line and the top of the next, in pixels.
    pub line_gap: f32,
    /// The vertical position of the line origin, in pixels, relative to the layout origin.
    pub y: f32,
    /// The horizontal position of the line start, in pixels, relative to the layout origin.
    pub x: f32,
    /// The width of the line, in pixels.
    pub width: f32,
    /// The byte range of the original string represented by this line.
    pub byte_range: Range<usize>,
    /// The index of first glyph in this line.
    pub glyph_range: Range<usize>,
}
impl Line {
    /// Return the height of the line. That is ascent - descent.
    pub fn height(&self) -> f32 {
        self.ascent - self.descent
    }

    /// The width of this line, ignoring the last glyph is it is a whitespace.
    pub fn visible_width(&self, glyphs: &[GlyphPosition]) -> f32 {
        if self.glyph_range.is_empty() {
            return self.width;
        }
        let last_glyph = &glyphs[self.glyph_range.end - 1];
        let ignore = if last_glyph.is_whitespace {
            last_glyph.width
        } else {
            0.0
        };
        self.width - ignore
    }

    /// Move this line and all it's glyphs to the given position.
    pub fn move_to(&mut self, x: f32, y: f32, glyphs: &mut [GlyphPosition]) {
        let x_off = x - self.x;
        let y_off = y - self.y;
        self.x = x;
        self.y = y;

        for glyph in &mut glyphs[self.glyph_range.clone()] {
            glyph.glyph.position.x += x_off;
            glyph.glyph.position.y += y_off;
        }
    }
}

/// A positioned scaled glyph.
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphPosition {
    /// The glyph itself, with position and scale
    pub glyph: Glyph,
    /// The index of the font of this glyph
    pub font_id: FontId,
    /// The byte range of text string represented by this glyph.
    pub byte_range: Range<usize>,
    /// The width of this glyph.
    pub width: f32,
    /// The color of this glyph.
    pub color: Color,
    /// If this glyph represents a whitespace char.
    pub is_whitespace: bool,
}
impl GlyphPosition {
    /// The position of the right edge of this glyph. Equal to position.x + width.
    pub fn right(&self) -> f32 {
        self.glyph.position.x + self.width
    }
}

/// A rect associated with a color.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorRect {
    /// The [x1, y1, x2, y2] rect.
    pub rect: [f32; 4],
    /// The color of the rect.
    pub color: Color,
}

/// Performs the shaping and layout of a SpannedString, producing glyphs for rendering. The
/// layouted glyphs are positioned relative to the alignment anchor, and must be translated to the
/// desired location to be rendered.
#[derive(Clone, Debug)]
pub struct TextLayout {
    /// The text that this layout represents
    text: SpannedString,
    /// The layout settings, such as aligment and wrap width.
    settings: LayoutSettings,
    /// The lines of the layout.
    lines: Vec<Line>,
    /// The glyphs of the layout.
    glyphs: Vec<GlyphPosition>,
    /// Rects that add others drawings such as underlines, and selections.
    rects: Vec<ColorRect>,
    /// The minimum width and height required so that there is no line wrap or overflow
    min_size: [f32; 2],
}
impl TextLayout {
    /// Create a new TextLayout from the given SpannedString.
    pub fn new(mut text: SpannedString, settings: LayoutSettings, fonts: &Fonts) -> Self {
        // Add a extra glyph to the text, to be used as the final empty line (if the text has a
        // trailing "\n") and for the position of the last caret.
        let len = text.string.len();
        let last_font_size = text
            .shape_spans
            .last()
            .map_or(text.default.font_size, |x| x.font_size);
        text.string.push_str(" ");
        text.shape_spans.push(ShapeSpan {
            byte_range: len..len + 1,
            font_size: last_font_size,
            font_id: Default::default(),
        });

        let mut this = Self {
            text,
            settings,
            lines: Vec::new(),
            glyphs: Vec::new(),
            rects: Vec::new(),
            min_size: [0.0, 0.0],
        };
        this.layout(fonts);

        this
    }

    /// Return a string slice to the text that this TextLayout represents.
    pub fn text(&self) -> &str {
        let len = self.text.string.len();
        &self.text.string[0..len - 1]
    }

    /// Return the height of the layouted text, from the top of the first line to the bottom of the
    /// last.
    pub fn height(&self) -> f32 {
        let sum: f32 = self.lines.iter().map(|x| x.height() + x.line_gap).sum();
        sum - self.lines.last().map_or(0.0, |x| x.line_gap)
    }

    /// Returns the minimum width and height required so that there is no line wrap or overflow.
    pub fn min_size(&self) -> [f32; 2] {
        self.min_size
    }

    /// Return a slice of the glyphs in this layout. All glyphs are positioned relative to the
    /// alignment anchor, and must be translated to the desired location to be rendered.
    pub fn glyphs(&self) -> &[GlyphPosition] {
        &self.glyphs
    }

    /// Return a slice of the rects in this layout. All rects are positioned relative to the
    /// alignment anchor, and must be translated to the desired location to be rendered.
    pub fn rects(&self) -> &[ColorRect] {
        &self.rects
    }

    /// Return a slice of the glyphs in this layout. All glyphs are positioned relative to the
    /// alignment anchor, and must be translated to the desired location to be rendered.
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    /// Return the x y position, in pixels, of the caret when positioned at the given byte index.
    /// Returns None if it is out of bounds. Notice that a extra glyph is add at the end of the
    /// represented text, which represents the position of caret for byte_index == text.len().
    pub fn pixel_position_from_byte_index(&self, byte_index: usize) -> Option<[f32; 2]> {
        if self.glyphs.is_empty() {
            return None;
        }
        let x = self
            .glyphs
            .binary_search_by(|x| cmp_range(byte_index, x.byte_range.clone()))
            .ok()?;
        let glyph = &self.glyphs[x];
        let pos = [glyph.glyph.position.x, glyph.glyph.position.y];
        Some(pos)
    }

    /// Return the index of the line that contains the given y_position, in pixels, or the closest
    /// one. If y is above the first line, return 0. If y is below the last line, return the last
    /// line index.
    pub fn line_from_y_position(&self, y_position: f32) -> usize {
        let l = self.lines.binary_search_by(|l| {
            if l.y - l.ascent > y_position {
                Ordering::Greater
            } else if l.y - l.descent < y_position {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
        match l {
            Ok(i) => i,
            Err(0) => 0,
            Err(x) if x == self.lines.len() => self.lines.len() - 1,
            _ => unreachable!(),
        }
    }

    /// Return the byte index for the caret located at the given line and horizontal position in
    /// pixels. This is rounded to the closest possible caret position.
    pub fn byte_index_from_x_position(&self, line: usize, x_position: f32) -> usize {
        let line = self.lines.get(line).unwrap();
        let glyphs = &self.glyphs[line.glyph_range.clone()];
        let g = glyphs.binary_search_by(|g| {
            if x_position < g.glyph.position.x {
                Ordering::Greater
            } else if x_position > g.right() {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
        match g {
            Ok(i) => {
                // round to nearest
                let middle = {
                    let glyph = &glyphs[i];
                    glyph.glyph.position.x + glyph.width / 2.0
                };
                let i = if i < glyphs.len() - 1 && x_position > middle {
                    i + 1
                } else {
                    i
                };
                glyphs[i].byte_range.start
            }
            Err(0) => line.byte_range.start,
            Err(i) if i == glyphs.len() => glyphs.last().unwrap().byte_range.start,
            _ => unreachable!(),
        }
    }

    /// Return the byte index for the caret located closest to the given vertical and horizontal
    /// position, in pixels.
    pub fn byte_index_from_position(&self, x: f32, y: f32) -> usize {
        let line = self.line_from_y_position(y);
        let index = self.byte_index_from_x_position(line, x);
        index
    }

    /// Removes the specified range in the string, and replaces it with the given string. This
    /// recompute the layout. The given string doesn’t need to be the same length as the range.
    pub fn replace_range(&mut self, range: Range<usize>, text: &str, fonts: &Fonts) {
        // the string has a extra char, so check for out of bounds for len() - 1.
        assert!(
            range.end <= self.text.string.len() - 1,
            "range.end is {}, but string.len() is {}",
            range.end,
            self.text.string.len() - 1
        );
        self.text.replace_range(range, text);

        // clear everthing
        self.glyphs.clear();
        self.rects.clear();
        self.lines.clear();
        self.min_size = [0.0, 0.0];

        // recompute everthing
        self.layout(fonts);
    }

    /// Destroys self, returning the inner SpannedString.
    pub fn to_spanned(mut self) -> SpannedString {
        // remove the extra char at the end, before returning it.
        let len = self.text.string.len();
        self.text.replace_range(len - 1..len, "");

        self.text
    }

    pub fn spanned(&self) -> &SpannedString {
        &self.text
    }

    fn layout(&mut self, fonts: &Fonts) {
        use unicode_linebreak::{linebreaks, BreakOpportunity::*};
        let (allowed_breaks, mandatory_breaks) = {
            let mut left: Vec<usize> = Vec::new();
            let mut right: Vec<usize> = Vec::new();

            for x in linebreaks(&self.text.string) {
                if x.1 == Allowed {
                    left.push(x.0);
                } else {
                    right.push(x.0);
                }
            }

            (left, right)
        };

        let lines = self.layout_paragraphs(fonts, mandatory_breaks);

        self.compute_min_size(&lines);
        self.break_lines(lines, allowed_breaks);
        assert_eq!(self.lines[0].glyph_range.start, 0);
        assert_eq!(
            self.lines.last().unwrap().glyph_range.end,
            self.glyphs.len()
        );
        assert_eq!(self.lines[0].byte_range.start, 0);
        assert_eq!(
            self.lines.last().unwrap().byte_range.end,
            self.text.string.len()
        );
        self.position_lines();
        self.apply_styles();
    }

    /// Layout it paragraph in a LineLayout. Each paragraph is section of the text, separated by
    /// mandatory breaklines.
    fn layout_paragraphs(
        &mut self,
        fonts: &Fonts,
        mandatory_breaks: Vec<usize>,
    ) -> Vec<LineLayout> {
        // split the text in paragraphs
        for i in &mandatory_breaks {
            self.text.split_shape_span(*i);
        }

        let mut span_start = 0;
        let mut lines = Vec::new();
        for next_break in mandatory_breaks.into_iter() {
            let span_end = self
                .text
                .shape_spans
                .iter()
                .skip(span_start)
                .position(|x| x.byte_range.start == next_break)
                .map_or(self.text.shape_spans.len(), |x| x + span_start);
            let line = LineLayout::new(&self.text, span_start..span_end, fonts);
            lines.push(line);
            span_start = span_end;
        }

        lines
    }

    /// Compute the minimum size for the bound rect of this layout required so that there is no
    /// line wrap or overflow. Stores it in self.min_size
    fn compute_min_size(&mut self, lines: &[LineLayout]) {
        let height = {
            let sum: f32 = lines.iter().map(|x| x.height + x.line_gap).sum();
            sum - lines.last().map_or(0.0, |x| x.line_gap)
        };
        let width = lines
            .iter()
            .map(|x| x.width)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        self.min_size = [width, height];
    }

    /// If there is a max_width, break the given LineLayouts in multiple lines. All lines and
    /// glyphs are moved to self.lines and self.glyphs.
    fn break_lines(&mut self, mut lines: Vec<LineLayout>, allowed_breaks: Vec<usize>) {
        if let Some(max_width) = self.settings.max_width {
            let mut breaklines = allowed_breaks.into();
            for line in &mut lines {
                line.break_lines(max_width, &mut breaklines);
            }
        } else {
            for line in &mut lines {
                let value = line.form_line();
                line.lines.push(value);
            }
        }

        for line in lines {
            let start_glyph = self.glyphs.len();
            self.lines.extend(line.lines.into_iter().map(|mut x| {
                x.glyph_range.start += start_glyph;
                x.glyph_range.end += start_glyph;
                x
            }));
            let color = self.text.default.color;
            self.glyphs.extend(line.glyphs.into_iter().map(|mut x| {
                x.color = color;
                x
            }));
        }
    }

    /// Move all lines to the right position.
    fn position_lines(&mut self) {
        let height = self.height();
        let mut y = match self.settings.vertical_align {
            Alignment::Start => 0.0,
            Alignment::Center => -height / 2.0,
            Alignment::End => -height,
        };
        for line in &mut self.lines {
            y += line.ascent;
            let x = match self.settings.horizontal_align {
                Alignment::Start => 0.0,
                Alignment::Center => -line.visible_width(&self.glyphs) / 2.0,
                Alignment::End => -line.visible_width(&self.glyphs),
            };
            line.move_to(x, y, &mut self.glyphs);
            y += -line.descent + line.line_gap;
        }
    }

    /// Apply the styles describe in SpannedString.style_spans for each respective range of text.
    /// This change glyph color and add selections for example.
    fn apply_styles(&mut self) {
        for style in &self.text.style_spans {
            let StyleSpan {
                byte_range: range,
                kind,
            } = style;
            let glyph_range = {
                let start_glyph = self
                    .glyphs
                    .binary_search_by(|x| cmp_range(range.start, x.byte_range.clone()))
                    .unwrap();
                let end_glyph = self
                    .glyphs
                    .binary_search_by(|x| cmp_range(range.end, x.byte_range.clone()))
                    .unwrap();
                start_glyph..end_glyph
            };
            if glyph_range.is_empty() {
                continue;
            }
            match kind {
                &StyleKind::Color(color)
                | &StyleKind::Selection {
                    fg: Some(color), ..
                } => self.glyphs[glyph_range.clone()]
                    .iter_mut()
                    .for_each(move |x| x.color = color),
                _ => {}
            }
            match kind {
                &StyleKind::Selection { bg: color, .. } => {
                    let first_line = self
                        .lines
                        .binary_search_by(|x| cmp_range(range.start, x.byte_range.clone()))
                        .unwrap();
                    let glyphs = &self.glyphs;
                    let glyph_pos = |glyph_index: usize| {
                        let glyph = &glyphs[glyph_index];
                        [glyph.glyph.position.x, glyph.glyph.position.y]
                    };
                    let glyph_pos_end = |glyph_index: usize| {
                        let glyph = &glyphs[glyph_index];
                        [glyph.right(), glyph.glyph.position.y]
                    };
                    let start_pos = glyph_pos(glyph_range.start);
                    let end_pos = glyph_pos_end(glyph_range.end - 1);
                    let line = &self.lines[first_line];
                    if line.glyph_range.end > glyph_range.end {
                        let rect = [
                            start_pos[0],
                            start_pos[1] - line.ascent,
                            end_pos[0],
                            end_pos[1] - line.descent,
                        ];
                        self.rects.push(ColorRect { rect, color });
                    } else {
                        {
                            let end_pos = glyph_pos_end(line.glyph_range.end - 1);
                            let rect = [
                                start_pos[0],
                                start_pos[1] - line.ascent,
                                end_pos[0],
                                end_pos[1] - line.descent,
                            ];
                            self.rects.push(ColorRect { rect, color });
                        }
                        for line in self.lines[first_line..].iter().skip(1) {
                            let start_pos = glyph_pos(line.glyph_range.start);
                            if line.glyph_range.end > glyph_range.end {
                                let rect = [
                                    start_pos[0],
                                    start_pos[1] - line.ascent,
                                    end_pos[0],
                                    end_pos[1] - line.descent,
                                ];
                                self.rects.push(ColorRect { rect, color });
                                break;
                            } else {
                                let end_pos = glyph_pos_end(line.glyph_range.end - 1);
                                let rect = [
                                    start_pos[0],
                                    start_pos[1] - line.ascent,
                                    end_pos[0],
                                    end_pos[1] - line.descent,
                                ];
                                self.rects.push(ColorRect { rect, color });
                            };
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// The layout of a single line of text. This can be break in multiple line later.
#[derive(Debug)]
struct LineLayout {
    /// The glyphs
    glyphs: Vec<GlyphPosition>,
    /// The byte range of the original text that this line layout represents.
    byte_range: Range<usize>,
    /// Each section of text can have a diferent line measure. This vector preserves that. This is
    /// use when this layout is break in multiple lines.
    lines: Vec<Line>,
    /// The width of this line, in pixels.
    width: f32,
    /// The height of this line, in pixels.
    height: f32,
    /// The line gap of this line, in pixels.
    line_gap: f32,
}
impl LineLayout {
    /// Create a new layout for the given range of the given text.
    fn new(text: &SpannedString, span_range: Range<usize>, fonts: &Fonts) -> Self {
        let byte_range = {
            let start = text.shape_spans[span_range.start].byte_range.start;
            let end = text.shape_spans[span_range.end - 1].byte_range.end;
            start..end
        };
        let mut this = Self {
            glyphs: Vec::new(),
            byte_range,
            lines: Vec::new(),
            width: 0.0,
            height: 0.0,
            line_gap: 0.0,
        };

        for shape_span in &text.shape_spans[span_range] {
            let text = &text.string[shape_span.byte_range.clone()];
            this.append_run(fonts, shape_span, text, shape_span.byte_range.clone());
        }

        let last_glyph = this.glyphs.last().unwrap();
        this.width = last_glyph.right();

        this
    }

    fn append_run(
        &mut self,
        fonts: &Fonts,
        shape: &ShapeSpan,
        text: &str,
        byte_range: Range<usize>,
    ) {
        if shape.byte_range.is_empty() {
            return;
        }

        let font = fonts
            .get(shape.font_id)
            .expect("FontId is out of bounds")
            .as_scaled(shape.font_size);

        self.height = self.height.max(font.height());
        self.line_gap = self.height.max(font.line_gap());

        let current_line = match self.lines.last_mut() {
            Some(last) => {
                let equal = last.ascent == font.ascent()
                    && last.descent == font.descent()
                    && last.line_gap == font.descent();
                if equal {
                    last.byte_range.end = shape.byte_range.end;
                    last
                } else {
                    let value = Line {
                        ascent: font.ascent(),
                        descent: font.descent(),
                        line_gap: font.line_gap(),
                        y: last.y,
                        x: last.x + last.width,
                        width: 0.0,
                        byte_range: shape.byte_range.clone(),
                        glyph_range: {
                            let l = self.glyphs.len();
                            l..l
                        },
                    };
                    self.lines.push(value);
                    self.lines.last_mut().unwrap()
                }
            }
            None => {
                let value = Line {
                    ascent: font.ascent(),
                    descent: font.descent(),
                    line_gap: font.line_gap(),
                    y: 0.0,
                    x: 0.0,
                    width: 0.0,
                    byte_range: shape.byte_range.clone(),
                    glyph_range: {
                        let l = self.glyphs.len();
                        l..l
                    },
                };
                self.lines.push(value);
                self.lines.last_mut().unwrap()
            }
        };

        // the start position of this shape run
        let start_x = current_line.x + current_line.width;
        let start_y = current_line.y;

        let glyphs = super::shaping::shape(fonts, &text, shape);
        for mut glyph in glyphs {
            glyph.glyph.position.x += start_x;
            glyph.glyph.position.y += start_y;
            glyph.byte_range.start += byte_range.start;
            glyph.byte_range.end += byte_range.start;
            self.glyphs.push(glyph);
        }

        // assuming that there is always at least one glyph
        let last_glyph = self.glyphs.last().unwrap();
        let right_pos = last_glyph.right();
        current_line.width = right_pos - current_line.x;
        current_line.glyph_range.end = self.glyphs.len();
    }

    /// Merge all self.line in a single line, and return it. Clears self.line.
    fn form_line(&mut self) -> Line {
        // I assume that there is always at last on line in self.lines.
        let right_pos = self.glyphs.last().unwrap().right();
        let byte_index = self.lines.last().unwrap().byte_range.end;

        let mut curr_line = self.lines[0].clone();
        curr_line.byte_range.end = byte_index;
        curr_line.width = right_pos - curr_line.x;
        curr_line.glyph_range.end = self.glyphs.len();

        for line in self.lines[1..].iter_mut() {
            curr_line.ascent = curr_line.ascent.max(line.ascent);
            curr_line.descent = curr_line.descent.max(line.descent);
            curr_line.line_gap = curr_line.line_gap.max(line.line_gap);
        }
        self.lines.clear();

        curr_line
    }

    /// Merge all self.line that is before byte_index in a single line, and return it. glyph_index
    /// is the first glyph in the next line. This will split the line that contains the byte_index,
    /// and remove all lines before that split.
    fn form_line_until(
        glyphs: &[GlyphPosition],
        lines: &mut Vec<Line>,
        byte_index: usize,
        glyph_index: usize,
    ) -> Line {
        assert!(glyph_index > 0);
        // the pixel position of the break
        let split_pos = glyphs[glyph_index].glyph.position.x;
        let right_pos = glyphs[glyph_index - 1].right();

        // the line to be returned, but the ascent, descent and line gap need to be computed while
        // merging with the others lines
        let mut curr_line = lines[0].clone();
        curr_line.byte_range.end = byte_index;
        curr_line.glyph_range.end = glyph_index;
        curr_line.width = right_pos - curr_line.x;

        // if split happens in the first line, no need to compute ascent, etc
        let line = &lines[0];
        if line.byte_range.contains(&byte_index) {
            let mut split = line.clone();
            split.x = split_pos;
            split.width = line.x + line.width - split.x;
            split.glyph_range.start = glyph_index;
            split.byte_range.start = byte_index;

            lines[0] = split;
            return curr_line;
        }

        // remove mark
        const REMOVE: f32 = -10_000.0;

        lines[0].width = REMOVE;

        let merge_line = |curr_line: &mut Line, line: &mut Line| {
            curr_line.ascent = curr_line.ascent.max(line.ascent);
            curr_line.descent = curr_line.descent.max(line.descent);
            curr_line.line_gap = curr_line.line_gap.max(line.line_gap);
            // mark line to remove
            line.width = REMOVE;
        };

        for (l, line) in lines.iter_mut().enumerate().skip(1) {
            if line.byte_range.contains(&byte_index) {
                let mut split = line.clone();
                split.x = split_pos;
                split.width = line.x + line.width - split.x;
                split.glyph_range.start = glyph_index;
                split.byte_range.start = byte_index;

                merge_line(&mut curr_line, line);

                lines[l] = split;
                break;
            }

            merge_line(&mut curr_line, line);
        }

        // remove marked lines
        lines.retain(|line| line.width != REMOVE);

        curr_line
    }

    /// Greedily break the line in smaller ones, in a way that each line has width smaller than the
    /// given max_width.
    fn break_lines(&mut self, max_width: f32, linebreaks: &mut VecDeque<usize>) {
        if self.width < max_width {
            let value = self.form_line();
            self.lines.push(value);
            return;
        }
        let mut lines = Vec::new();
        // skip the first glyph, because there is no way to do a break line there.
        for (g, glyph) in self.glyphs.iter().enumerate().skip(1) {
            // a partial overflow of a whitespace glyph is ignored.
            let right = if glyph.is_whitespace {
                glyph.glyph.position.x
            } else {
                glyph.right()
            };
            let right_pos = right - self.lines[0].x;
            if right_pos > max_width {
                // find the last possible break position, if it exist
                let mut prev_break = None;
                let byte_index = glyph.byte_range.start;
                while let Some(&next) = linebreaks.front() {
                    if next <= byte_index {
                        prev_break = linebreaks.pop_front();
                    } else {
                        break;
                    }
                }

                // maybe found a break in the previous paragraph
                if prev_break.map_or(false, |x| x < self.lines[0].byte_range.start) {
                    prev_break = None;
                }

                // find the glyph index of the break point, or fallback to this glyph as breakpoint
                let (break_byte, break_glyph) = if let Some(prev_break) = prev_break {
                    let glyph_index = self.glyphs[..=g]
                        .iter()
                        .enumerate()
                        .rev()
                        .find(|x| x.1.byte_range.contains(&prev_break))
                        .map(|x| x.0)
                        .unwrap();
                    (prev_break, glyph_index)
                } else {
                    (byte_index, g)
                };

                // break the line
                let value =
                    Self::form_line_until(&self.glyphs, &mut self.lines, break_byte, break_glyph);
                lines.push(value);
            }
        }
        lines.push(self.form_line());

        self.lines = lines;
    }
}
