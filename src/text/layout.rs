use std::collections::VecDeque;
use std::ops::Range;

use ab_glyph::{Font, Glyph, ScaleFont};

use crate::font::{FontId, Fonts};
use crate::text::SpannedString;
use crate::Color;

use super::ShapeSpan;

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

#[derive(Default, Clone, Debug)]
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
#[derive(Debug, Clone)]
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
    /// The position of the right edge of this glyph. Equal to position.x + width. If the glyph is
    /// a whitespace, the width will be interpreted as zero.
    pub fn right(&self) -> f32 {
        self.glyph.position.x + if self.is_whitespace { 0.0 } else { self.width }
    }
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
    /// The minimum width and height required so that there is no line wrap or overflow
    min_size: [f32; 2],
}
impl TextLayout {
    /// Create a new TextLayout from the given SpannedString.
    pub fn new(text: SpannedString, settings: LayoutSettings, fonts: &Fonts) -> Self {
        let mut this = Self {
            text,
            settings,
            lines: Vec::new(),
            glyphs: Vec::new(),
            min_size: [0.0, 0.0],
        };
        this.layout(fonts);

        this
    }

    /// Return a string slice to the text that this TextLayout represents.
    pub fn text(&self) -> &str {
        &self.text.string
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

    /// Return a slice of the glyphs in this layout. All glyphs are positioned relative to the
    /// alignment anchor, and must be translated to the desired location to be rendered.
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn replace_range(&mut self, range: Range<usize>, text: &str, fonts: &Fonts) {
        todo!()
    }

    pub fn to_spanned(self) -> SpannedString {
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
        self.position_lines();
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
                Alignment::Center => -line.width / 2.0,
                Alignment::End => -line.width,
            };
            line.move_to(x, y, &mut self.glyphs);
            y += -line.descent + line.line_gap;
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
        this.width = last_glyph.glyph.position.x + last_glyph.width;

        this
    }

    fn append_run(&mut self, fonts: &Fonts, shape: &ShapeSpan, text: &str, byte_range: Range<usize>) {
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
        let right_pos = self.glyphs.last().unwrap().right();
        let byte_index = self.lines.last().unwrap().byte_range.end;

        let mut curr_line = self.lines[0].clone();
        curr_line.byte_range.end = byte_index;
        curr_line.width = right_pos - curr_line.x;

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
        let split_pos = glyphs[glyph_index].glyph.position.x;
        let right_pos = glyphs[glyph_index - 1].right();

        let mut curr_line = lines[0].clone();
        curr_line.byte_range.end = byte_index;
        curr_line.glyph_range.end = glyph_index;
        curr_line.width = right_pos - curr_line.x;

        // if split happens in the first line
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

                // lines.insert(l + 1, split);
                lines[l] = split;
                break;
            }

            merge_line(&mut curr_line, line);
        }

        // remove marked lines
        lines.retain(|line| line.width == REMOVE);

        curr_line
    }

    /// Greedily break the line in smaller ones, in a way that each line has width smaller than the
    /// given max_width.
    fn break_lines(&mut self, max_width: f32, linebreaks: &mut VecDeque<usize>) {
        let mut lines = Vec::new();
        // skip the first glyph, because there is no way to do a break line there.
        for (g, glyph) in self.glyphs.iter().enumerate().skip(1) {
            let right_pos = glyph.right() - self.lines[0].x;
            if right_pos > max_width {
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

                let value =
                    Self::form_line_until(&self.glyphs, &mut self.lines, break_byte, break_glyph);
                lines.push(value);
            }
        }
        lines.push(self.form_line());

        self.lines = lines;
    }
}
