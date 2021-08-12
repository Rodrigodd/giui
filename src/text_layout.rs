// derivative from <https://github.com/mooman219/fontdue>
// MIT License
//
//Copyright (c) 2019 Joe C (mooman219)
//
//Permission is hereby granted, free of charge, to any person obtaining a copy
//of this software and associated documentation files (the "Software"), to deal
//in the Software without restriction, including without limitation the rights
//to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//copies of the Software, and to permit persons to whom the Software is
//furnished to do so, subject to the following conditions:
//
//The above copyright notice and this permission notice shall be included in all
//copies or substantial portions of the Software.

use crate::{
    font::{FontId, Fonts},
    text::{SpannedString, TextStyle},
    unicode::{linebreak_property, wrap_mask, LINEBREAK_HARD, LINEBREAK_NONE},
    Color,
};
use std::ops::Range;

// #[cfg_attr(shaping, path = "text_layout/shaping_harfbuzz.rs")]
// #[cfg_attr(not(shaping), path = "text_layout/shaping_simple.rs")]
// mod shaping;

mod shaping_harfbuzz;
#[allow(dead_code)]
mod shaping_simple;

use shaping_harfbuzz as shaping;

use ab_glyph::{Font, Glyph, ScaleFont};

/// Horizontal alignment options for text when a max_width is provided.
#[derive(Copy, Clone, PartialEq)]
pub enum HorizontalAlign {
    /// Aligns text to the left of the region defined by the max_width.
    Left,
    /// Aligns text to the center of the region defined by the max_width.
    Center,
    /// Aligns text to the right of the region defined by the max_width.
    Right,
}

/// Vertical alignment options for text when a max_height is provided.
#[derive(Copy, Clone, PartialEq)]
pub enum VerticalAlign {
    /// Aligns text to the top of the region defined by the max_height.
    Top,
    /// Aligns text to the middle of the region defined by the max_height.
    Middle,
    /// Aligns text to the bottom of the region defined by the max_height.
    Bottom,
}

/// Wrap style is a hint for how strings of text should be wrapped to the next line. Line wrapping
/// can happen when the max width/height is reached.
#[derive(Copy, Clone, PartialEq)]
pub enum WrapStyle {
    /// Word will break lines by the Unicode line breaking algorithm (Standard Annex #14) This will
    /// generally break lines where you expect them to be broken at and will preserve words.
    Word,
    /// Letter will not preserve words, breaking into a new line after the nearest letter.
    Letter,
}

/// Settings to configure how text layout is constrained. Text layout is considered best effort and
/// layout may violate the constraints defined here if they prevent text from being laid out.
#[derive(Copy, Clone, PartialEq)]
pub struct LayoutSettings {
    /// An optional rightmost boundary on the text region. A line of text that exceeds the
    /// max_width is wrapped to the line below. If the width of a glyph is larger than the
    /// max_width, the glyph will overflow past the max_width. The application is responsible for
    /// handling the overflow.
    pub max_width: Option<f32>,
    /// An optional bottom boundary on the text region. This is used for positioning the
    /// vertical_align option. Text that exceeds the defined max_height will overflow past it. The
    /// application is responsible for handling the overflow.
    pub max_height: Option<f32>,
    /// The default is Left. This option does nothing if the max_width isn't set.
    pub horizontal_align: HorizontalAlign,
    /// The default is Top. This option does nothing if the max_height isn't set.
    pub vertical_align: VerticalAlign,
    /// The default is Word. Wrap style is a hint for how strings of text should be wrapped to the
    /// next line. Line wrapping can happen when the max width/height is reached.
    pub wrap_style: WrapStyle,
    /// The default is true. This option enables hard breaks, like new line characters, to
    /// prematurely wrap lines. If false, hard breaks will not prematurely create a new line.
    pub wrap_hard_breaks: bool,
}

impl Default for LayoutSettings {
    fn default() -> LayoutSettings {
        LayoutSettings {
            max_width: None,
            max_height: None,
            horizontal_align: HorizontalAlign::Left,
            vertical_align: VerticalAlign::Top,
            wrap_style: WrapStyle::Word,
            wrap_hard_breaks: true,
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
    /// The range of the slice of the string used to generate this glyph
    pub byte_range: Range<usize>,
    /// The width of the glyph. This does not take the kerning to next glyph into account.
    pub width: f32,
    /// The color of this glyph
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct LineMetrics {
    /// The width, in pixels, of the line.
    pub line_width: f32,
    /// The largest ascent for the line.
    pub ascent: f32,
    /// The smallest descent for the line. This is normaly negative.
    pub descent: f32,
    /// The largest new line size for the line.
    pub new_line_size: f32,
    /// The x position this line starts at.
    pub x_start: f32,
    /// The index of the first glyph in the line.
    pub start_glyph: usize,
    /// The index of the last glyph in the line plus one.
    pub end_glyph: usize,
    /// The index of the first rect in the line
    pub start_rect: usize,
    // The inde of the last rect in the line plus one.
    pub end_rect: usize,
}

impl Default for LineMetrics {
    fn default() -> Self {
        LineMetrics {
            line_width: 0.0,
            ascent: 0.0,
            descent: 0.0,
            x_start: 0.0,
            new_line_size: 0.0,
            start_glyph: 0,
            end_glyph: 0,
            start_rect: 0,
            end_rect: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColorRect {
    pub color: Color,
    // A rect, in [x, y, w, h] format.
    pub rect: [f32; 4],
}

/// Text layout requires a small amount of heap usage which is contained in the Layout struct. This
/// context is reused between layout calls. Reusing the Layout struct will greatly reduce memory
/// allocations and is advisable for performance.
#[derive(Debug, Clone)]
pub struct TextLayout {
    // Settings state
    x: f32,
    y: f32,
    wrap_mask: u8,
    max_width: f32,
    max_height: f32,
    vertical_align: f32,
    horizontal_align: f32,
    // Single line state
    output: Vec<GlyphPosition>,
    output_rects: Vec<ColorRect>,
    glyphs: Vec<GlyphPosition>,
    rects: Vec<ColorRect>,
    line_metrics: Vec<LineMetrics>,
    text: String,
    linebreak_prev: u8,
    linebreak_state: u8,
    linebreak_pos: f32,
    linebreak_idx: usize,
    current_pos: f32,
    current_ascent: f32,
    current_descent: f32,
    current_new_line: f32,
    current_px: f32,
    // start position of the current line
    start_pos: f32,
    height: f32,
    width: f32,
}
impl Default for TextLayout {
    fn default() -> Self {
        Self::new()
    }
}
impl TextLayout {
    /// Creates a layout instance. This requires the direction that the Y coordinate increases in.
    /// Layout needs to be aware of your coordinate system to place the glyphs correctly.
    pub fn new() -> TextLayout {
        let mut layout = TextLayout {
            // Settings state
            x: 0.0,
            y: 0.0,
            wrap_mask: 0,
            max_width: 0.0,
            max_height: 0.0,
            vertical_align: 0.0,
            horizontal_align: 0.0,
            // Line state
            output: Vec::new(),
            glyphs: Vec::new(),
            output_rects: Vec::new(),
            /// Rects are what form backgrounds, underlines, etc...
            rects: Vec::new(),
            line_metrics: Vec::new(),
            text: String::new(),
            linebreak_prev: 0,
            linebreak_state: 0,
            linebreak_pos: 0.0,
            linebreak_idx: 0,
            current_pos: 0.0,
            current_ascent: 0.0,
            current_descent: 0.0,
            current_new_line: 0.0,
            current_px: 0.0,
            start_pos: 0.0,
            height: 0.0,
            width: 0.0,
        };
        layout.reset(&LayoutSettings::default());
        layout
    }

    /// Resets the current layout settings and clears all appended text.
    pub fn reset(&mut self, settings: &LayoutSettings) {
        self.x = 0.0;
        self.y = 0.0;
        self.wrap_mask = wrap_mask(
            settings.wrap_style == WrapStyle::Word,
            settings.wrap_hard_breaks,
            settings.max_width.is_some(),
        );
        self.max_width = settings.max_width.unwrap_or(core::f32::MAX);
        self.max_height = settings.max_height.unwrap_or(core::f32::MAX);
        self.vertical_align = if settings.max_height.is_none() {
            0.0
        } else {
            match settings.vertical_align {
                VerticalAlign::Top => 0.0,
                VerticalAlign::Middle => 0.5,
                VerticalAlign::Bottom => 1.0,
            }
        };
        self.horizontal_align = if settings.max_width.is_none() {
            0.0
        } else {
            match settings.horizontal_align {
                HorizontalAlign::Left => 0.0,
                HorizontalAlign::Center => 0.5,
                HorizontalAlign::Right => 1.0,
            }
        };
        self.clear();
    }

    /// Keeps current layout settings but clears all appended text.
    pub fn clear(&mut self) {
        self.glyphs.clear();
        self.output.clear();
        self.line_metrics.clear();
        self.line_metrics.push(LineMetrics::default());
        self.text.clear();

        self.linebreak_prev = 0;
        self.linebreak_state = 0;
        self.linebreak_pos = 0.0;
        self.linebreak_idx = 0;
        self.current_pos = 0.0;
        self.current_ascent = 0.0;
        self.current_new_line = 0.0;
        self.current_px = 0.0;
        self.start_pos = 0.0;
        self.height = 0.0;
    }

    /// Gets the current height of the appended text.
    pub fn height(&self) -> f32 {
        if let Some(line) = self.line_metrics.last() {
            self.height + line.new_line_size
        } else {
            0.0
        }
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn min_size(&self) -> [f32; 2] {
        // increase a little the dimentions to avoid float point precision errors
        // when recomputing the layout with this size.
        [self.width() + 0.001, self.height() + 0.001]
    }

    /// Gets the current line count. If there's no text this still returns 1.
    pub fn lines(&self) -> usize {
        self.line_metrics.len()
    }

    /// Performs layout for text horizontally, and wrapping vertically. This makes a best effort
    /// attempt at laying out the text defined in the given styles with the provided layout
    /// settings. Text may overflow out of the bounds defined in the layout settings and it's up
    /// to the application to decide how to deal with this.
    ///
    /// Characters from the input string can only be omitted from the output, they are never
    /// reordered. The output buffer will always contain characters in the order they were defined
    /// in the styles.
    pub fn layout(&mut self, fonts: &Fonts, text: &SpannedString) {
        self.text.clone_from(&text.string);
        for i in 0..text.spans.len() {
            let (range, span) = &text.spans[i];
            self.append(fonts, range.clone(), span);
        }
    }

    fn append(&mut self, fonts: &Fonts, range: Range<usize>, style: &TextStyle) {
        // let text = &self.text[range];
        let font = fonts
            .get(style.font_id)
            .expect("FontId is out of bounds")
            .as_scaled(style.font_size);

        self.current_ascent = font.ascent().ceil();
        self.current_descent = font.descent().ceil();
        self.current_new_line = (font.ascent() - font.descent() + font.line_gap()).ceil();
        if let Some(line) = self.line_metrics.last_mut() {
            if self.current_ascent > line.ascent {
                line.ascent = self.current_ascent;
            }
            if self.current_descent < line.descent {
                line.descent = self.current_descent;
            }
            if self.current_new_line > line.new_line_size {
                line.new_line_size = self.current_new_line;
            }
        }

        let glyphs = shaping::shape(fonts, &self.text[range.clone()], style);
        // the position of the start of this section or line
        let mut section_start = self.current_pos;
        for mut glyph in glyphs {
            // correct the byte_range of the glyph
            glyph.byte_range.start += range.start;
            glyph.byte_range.end += range.start;

            let c = self.text[glyph.byte_range.start..].chars().next().unwrap();

            let linebreak = linebreak_property(&mut self.linebreak_state, c) & self.wrap_mask;
            if linebreak >= self.linebreak_prev {
                self.linebreak_prev = linebreak;
                self.linebreak_pos = self.current_pos;
                self.linebreak_idx = self.glyphs.len();
            }

            if c.is_control() {
                glyph.glyph.id = font.glyph_id(' ');
            }

            // dont consider trailing whitespace char before a line break
            let c_advance = if c.is_whitespace() && !c.is_control() {
                0.0
            } else {
                glyph.width
            };
            let line_width = self.current_pos + c_advance - self.start_pos;
            if linebreak == LINEBREAK_HARD || line_width > self.max_width {
                self.linebreak_prev = LINEBREAK_NONE;

                if let Some(background) = style.background {
                    // TODO: if the breakline happens inside the previous style,
                    // and the previous style has a background, the background
                    // must also be breaked.
                    let rect = [
                        section_start,
                        -self.current_ascent,
                        self.linebreak_pos - section_start,
                        self.current_ascent - self.current_descent,
                    ];
                    let rect = ColorRect {
                        color: background,
                        rect,
                    };
                    self.rects.push(rect);
                }

                // Close last line metric

                if let Some(line) = self.line_metrics.last_mut() {
                    line.end_glyph = self.linebreak_idx;
                    line.end_rect = self.rects.len();
                    line.line_width = self.linebreak_pos - self.start_pos;
                    if line.end_glyph > line.start_glyph {
                        let glyph = &self.glyphs[line.end_glyph - 1];
                        let range = glyph.byte_range.clone();
                        // dont consider trailing whitespace char before a line break
                        let c = self.text[range].chars().next().unwrap();
                        if c.is_whitespace() && c != '\u{0004}' {
                            line.line_width -= glyph.width;
                        }
                    }
                    self.height += line.new_line_size;

                    if line.line_width > self.width {
                        self.width = line.line_width;
                    }
                }

                // Create a new line

                let start_index = self.linebreak_idx;
                self.start_pos = self.linebreak_pos;
                self.line_metrics.push(LineMetrics {
                    line_width: 0.0,
                    ascent: self.current_ascent,
                    descent: self.current_descent,
                    x_start: self.start_pos,
                    new_line_size: self.current_new_line,
                    start_glyph: start_index,
                    end_glyph: 0,
                    start_rect: self.rects.len(),
                    end_rect: 0,
                });

                section_start = self.start_pos;
            }
            glyph.glyph.position.x += self.current_pos;
            self.current_pos += glyph.width;
            self.glyphs.push(glyph);
        }
        if let Some(background) = style.background {
            let rect = [
                section_start,
                -self.current_ascent,
                self.current_pos - section_start,
                self.current_ascent - self.current_descent,
            ];
            let rect = ColorRect {
                color: background,
                rect,
            };
            self.rects.push(rect);
        }
        if let Some(line) = self.line_metrics.last_mut() {
            line.line_width = self.current_pos - self.start_pos;
            if line.line_width > self.width {
                self.width = line.line_width;
            }
            line.end_glyph = self.glyphs.len();
            line.end_rect = self.rects.len();
        }
    }

    pub fn line_metrics(&self) -> &Vec<LineMetrics> {
        &self.line_metrics
    }

    /// Gets the current laid out glyphs. Additional layout may be performed lazily here.
    pub fn glyphs(&mut self) -> &Vec<GlyphPosition> {
        self.glyphs_and_rects().0
    }

    /// Gets the current laid out glyphs and rects. Additional layout may be performed lazily here.
    pub fn glyphs_and_rects(&mut self) -> (&Vec<GlyphPosition>, &Vec<ColorRect>) {
        if self.glyphs.len() == self.output.len() {
            return (&self.output, &self.output_rects);
        }

        unsafe { self.output.set_len(0) };
        self.output.reserve(self.glyphs.len());

        let mut y = self.y + ((self.max_height - self.height()) * self.vertical_align).floor();
        let mut idx = 0;
        let mut rect_idx = 0;
        for line in &self.line_metrics {
            let padding = self.max_width - line.line_width;
            let x = self.x - line.x_start + padding * self.horizontal_align;
            y += line.ascent;
            while idx < line.end_glyph {
                let mut glyph = self.glyphs[idx].clone();
                glyph.glyph.position.x += x;
                glyph.glyph.position.y += y;
                self.output.push(glyph);
                idx += 1;
            }
            while rect_idx < line.end_rect {
                let mut rect = self.rects[rect_idx].clone();
                rect.rect[0] += x;
                rect.rect[1] += y;
                self.output_rects.push(rect);
                rect_idx += 1;
            }
            y += line.new_line_size - line.ascent;
        }

        (&self.output, &self.output_rects)
    }
}
