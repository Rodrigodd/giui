use ab_glyph::{Font, Glyph, ScaleFont};
use std::cmp::Ordering;
use std::ops::Range;
use xi_unicode::LineBreakIterator;

pub struct FontGlyph {
    pub glyph: Glyph,
    pub font_id: (usize,),
}

#[derive(Default, Clone)]
pub struct TextInfo {
    bounding_box: [f32; 4],
    indices: Vec<usize>,
    pub carret_pos: Vec<[f32; 2]>,
    lines: Vec<Range<usize>>,
    line_heigth: f32,
}
impl TextInfo {
    pub fn move_by(&mut self, delta: [f32; 2]) {
        for carret_pos in self.carret_pos.iter_mut() {
            carret_pos[0] += delta[0];
            carret_pos[1] += delta[1];
        }
    }

    /// Get the position of the caret for the given glyph index.
    /// The position is relative to to top left corner of the control rect.
    pub fn get_caret_pos(&self, index: usize) -> [f32; 2] {
        match self.carret_pos.get(index) {
            Some(x) => *x,
            None => self.carret_pos.last().map_or([0.0, 0.0], |x| *x),
        }
    }

    pub fn get_line(&self, index: usize) -> Option<usize> {
        self.lines
            .binary_search_by(|x| {
                if x.end <= index {
                    Ordering::Less
                } else if x.contains(&index) {
                    Ordering::Equal
                } else {
                    Ordering::Greater
                }
            })
            .ok()
    }

    pub fn get_line_range(&self, index: usize) -> Option<Range<usize>> {
        #[allow(clippy::reversed_empty_ranges)]
        Some(self.lines[self.get_line(index)?].clone())
    }

    pub fn get_caret_index_at_pos(&self, line: usize, x_pos: f32) -> usize {
        match self.carret_pos[self.lines[line].clone()]
            .binary_search_by(|x| x[0].partial_cmp(&x_pos).unwrap())
        {
            Ok(x) => (x + self.lines[line].start).min(self.lines[line].end - 1),
            Err(x) => {
                if x == 0 {
                    0
                } else {
                    (x + self.lines[line].start - 1).min(self.lines[line].end - 1)
                }
            }
        }
    }

    pub fn get_line_heigth(&self) -> f32 {
        self.line_heigth
    }

    pub fn get_caret_index(&self, indice: usize) -> usize {
        if self.indices.is_empty() {
            return 0;
        }
        match self.indices.binary_search(&indice) {
            Ok(x) => x,
            Err(x) => x - 1,
        }
    }

    pub fn get_indice(&self, index: usize) -> usize {
        self.indices[index]
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn num_lines(&self) -> usize {
        self.lines.len()
    }

    pub fn get_size(&self) -> [f32; 2] {
        [
            self.bounding_box[2] - self.bounding_box[0],
            self.bounding_box[3] - self.bounding_box[1],
        ]
    }
}

pub fn text_glyphs_and_info<F: Font>(
    text: &str,
    font_id: usize,
    scale: f32,
    fonts: &[F],
    rect: [f32; 4],
    align: (i8, i8),
) -> (Vec<FontGlyph>, TextInfo) {
    let font = fonts[font_id].as_scaled(scale);
    if text.is_empty() {
        let x = match align.0 {
            -1 => rect[0],
            0 => (rect[0] + rect[2]) / 2.0,
            _ => rect[0] + rect[2],
        };
        let y = match align.1 {
            -1 => rect[1],
            0 => (rect[1] + rect[3] - font.height()) / 2.0,
            _ => rect[1] + rect[3] - font.height(),
        };
        #[allow(clippy::reversed_empty_ranges)]
        return (
            Vec::new(),
            TextInfo {
                bounding_box: [x, y, x, y + font.height()],
                indices: vec![0],
                carret_pos: vec![[x - rect[0], y + font.height() - rect[1]]],
                lines: vec![0..0],
                line_heigth: font.height(),
            },
        );
    }

    let align_line = |glyphs: &mut Vec<FontGlyph>,
                      carret_pos: &mut Vec<[f32; 2]>,
                      line: std::ops::Range<usize>,
                      mut last_is_whitespace: bool,
                      bounding_box: &mut [f32; 4]| {
        #[allow(clippy::len_zero)]
        if line.len() == 0 {
            let x = match align.0 {
                -1 => rect[0],
                0 => (rect[0] + rect[2]) / 2.0,
                _ => rect[0] + rect[2],
            };

            if bounding_box[0] > x {
                bounding_box[0] = x;
            }
            if bounding_box[1] < x {
                bounding_box[1] = x;
            }
            return;
        } else if line.len() == 1 && last_is_whitespace {
            last_is_whitespace = false;
        }
        let first = &glyphs[line.clone()][0];
        let last = if last_is_whitespace {
            glyphs[line.clone()].iter().rev().nth(1).unwrap()
        } else {
            glyphs[line.clone()].last().unwrap()
        };

        let right = last.glyph.position.x + font.h_advance(last.glyph.id);
        if right > bounding_box[2] {
            bounding_box[2] = right;
        }
        let left = first.glyph.position.x - font.h_side_bearing(first.glyph.id);
        if left < bounding_box[0] {
            bounding_box[0] = left;
        }

        if let 0 | 1 = align.0 {
            let width = right - left;
            let mut delta = rect[2] - rect[0] - width;
            if align.0 == 0 {
                delta /= 2.0;
            }
            for g in glyphs[line.clone()].iter_mut() {
                g.glyph.position.x += delta;
            }
            for c in carret_pos[line].iter_mut() {
                c[0] += delta;
            }
        }
    };

    let mut bounding_box = [rect[2], rect[1], rect[0], rect[1]];

    let mut x = rect[0];
    let mut y = rect[1] + font.ascent();

    let mut line_breaks = LineBreakIterator::new(text);
    let mut next_break = line_breaks.next();
    let mut last_break = None;

    let chars = text.char_indices();
    let mut glyphs = Vec::new();
    let mut is_whitespace = Vec::new();

    let mut line_start = 0;
    let mut last_glyph = None;

    let mut indices: Vec<usize> = Vec::new();
    let mut carret_pos: Vec<[f32; 2]> = Vec::new();
    let mut lines: Vec<Range<usize>> = Vec::new();

    for (i, mut c) in chars {
        // iterate the line_breaks, and handle if it reach a hard break
        if let Some(line_break) = next_break {
            if line_break.0 == i {
                if line_break.1 {
                    let line = line_start..glyphs.len();
                    align_line(
                        &mut glyphs,
                        &mut carret_pos,
                        line.clone(),
                        false,
                        &mut bounding_box,
                    );
                    lines.push(line);
                    line_start = glyphs.len();
                    y += font.height() + font.line_gap();
                    x = rect[0];
                    last_break = None;
                } else {
                    last_break = Some(glyphs.len());
                }
                next_break = line_breaks.next();
            }
        }

        // if its is control character, replace it by a whitespace
        if c.is_control() {
            c = ' ';
        }

        // if its is a breakable whitespace char, force the next soft break here by removing the last breaking point
        if c.is_whitespace() && c != '\u{00A0}' && c != '\u{202F}' && c != '\u{FEFF}' {
            last_break = None;
        }

        let mut glyph = font.scaled_glyph(c);
        let advance = font.h_advance(glyph.id);
        if let Some(last_glyph) = last_glyph {
            x += font.kern(last_glyph, glyph.id);
        }

        // breakline if exceeds the right edge
        // it will not break if part of a whitespace is after the rigth edge
        let mut rigth = x;
        if !c.is_whitespace() {
            rigth += advance;
        }
        if rigth > rect[2] && line_start != glyphs.len() {
            #[allow(clippy::never_loop)]
            loop {
                if let Some(last_break) = last_break.take() {
                    if last_break < glyphs.len() {
                        let line = line_start..last_break;
                        align_line(
                            &mut glyphs,
                            &mut carret_pos,
                            line.clone(),
                            is_whitespace[last_break - 1],
                            &mut bounding_box,
                        );
                        lines.push(line);
                        line_start = last_break;
                        let delta = [
                            rect[0] - glyphs[last_break].glyph.position.x,
                            font.height() + font.line_gap(),
                        ];
                        for g in glyphs[last_break..].iter_mut() {
                            g.glyph.position.x += delta[0];
                            g.glyph.position.y += delta[1];
                        }
                        for c in carret_pos[last_break..].iter_mut() {
                            c[0] += delta[0];
                            c[1] += delta[1];
                        }
                        let last_glyph = &glyphs.last().unwrap().glyph;
                        x = last_glyph.position.x + font.h_advance(last_glyph.id);
                        y += delta[1];
                        break;
                    }
                }
                // if the two if above is not entered
                let len = glyphs.len();
                let line = line_start..len;
                align_line(
                    &mut glyphs,
                    &mut carret_pos,
                    line.clone(),
                    is_whitespace[len - 1],
                    &mut bounding_box,
                );
                lines.push(line);
                y += font.height() + font.line_gap();
                x = rect[0];
                line_start = glyphs.len();
                break;
            }
        }

        glyph.position.x = x;
        glyph.position.y = y;

        indices.push(i);
        carret_pos.push([x - rect[0], y - font.descent() - rect[1]]);

        x += advance;

        last_glyph = Some(glyph.id);

        // if c.is_whitespace() {
        //     glyph.id = font.glyph_id('\u{B7}');
        // }

        glyphs.push(FontGlyph {
            glyph,
            font_id: (font_id,),
        });
        is_whitespace.push(c.is_whitespace());
    }

    //aling the last line
    let mut line = line_start..glyphs.len();
    align_line(
        &mut glyphs,
        &mut carret_pos,
        line.clone(),
        false,
        &mut bounding_box,
    );

    // add a caret position to the end of the text
    #[allow(clippy::len_zero)]
    if line.len() == 0 {
        let x = match align.0 {
            -1 => 0.0,
            0 => (rect[2] - rect[0]) / 2.0,
            _ => rect[2] - rect[0],
        };
        let last_caret = [x, y - rect[1] - font.descent()];
        carret_pos.push(last_caret);
        indices.push(text.len());
    } else {
        let last_glyph = &glyphs.last().unwrap().glyph;
        let last_caret = [
            last_glyph.position.x + font.h_advance(last_glyph.id) - rect[0],
            last_glyph.position.y - rect[1] - font.descent(),
        ];
        carret_pos.push(last_caret);
        indices.push(text.len());
    }

    // TODO: Instead of adding 1 here, I should not sub 1 in the other places,
    // such a way that line.end will be the position of the '\n'? (or in this case here, the position of text.len())
    line.end += 1;
    lines.push(line);

    bounding_box[3] = y - font.descent();

    // vertical alignment
    if let 0 | 1 = align.1 {
        let mut delta = rect[3] - (y - font.descent());
        if align.1 == 0 {
            delta /= 2.0;
        }
        for g in glyphs.iter_mut() {
            g.glyph.position.y += delta;
        }
        for c in carret_pos.iter_mut() {
            c[1] += delta;
        }
        bounding_box[1] += delta;
        bounding_box[3] += delta;
    }

    // remove blank glyphs
    let mut i = glyphs.len();
    while i != 0 {
        i -= 1;
        if is_whitespace[i] {
            glyphs.swap_remove(i);
        }
    }

    debug_assert!(!indices.is_empty());
    debug_assert_eq!(indices.len(), carret_pos.len());

    (
        glyphs,
        TextInfo {
            bounding_box,
            indices,
            carret_pos,
            lines,
            line_heigth: font.height(),
        },
    )
}
