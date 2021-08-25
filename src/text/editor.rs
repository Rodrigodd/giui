use crate::{font::Fonts, text::layout::TextLayout, util::cmp_range};
use std::ops::Range;

/// Represents a position in a text.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    /// The line in the text. The first line has value 0.
    line: usize,
    /// The EGC indice in the line of the text.
    collumn: usize,
}
impl std::fmt::Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}:{:?}", self.line, self.collumn)
    }
}

/// A selection of text. This can also represent a cursor.
pub struct Selection {
    /// The position of the cursor. Can also be the selection ends. Can be before or after the
    /// anchor.
    cursor: Position,
    /// The x position of the cursor. This only updated in horizontal motion. This is use to keep
    /// the cursor motion align when moving vertically multiple lines.
    cursor_x: f32,
    /// The position where the selection starts. Can be before or after the cursor.
    anchor: Position,
}
impl Selection {
    /// Set cursor and anchor to the same value.
    pub fn set_pos(&mut self, pos: Position) {
        self.cursor = pos;
        self.anchor = pos;
    }

    /// Check if the selection is empty, i.e., cursor == anchor
    pub fn is_empty(&self) -> bool {
        self.cursor == self.anchor
    }

    /// Return the position of the start of the selection. This is the min(cursor, anchor).
    pub fn start(&self) -> Position {
        if self.cursor < self.anchor {
            self.cursor
        } else {
            self.anchor
        }
    }

    /// Return the position of the end of the selection. This is the max(cursor, anchor).
    pub fn end(&self) -> Position {
        if self.cursor > self.anchor {
            self.cursor
        } else {
            self.anchor
        }
    }
}

/// A very simple text editor
pub struct TextEditor {
    /// The current selection. Also represent the cursor.
    selection: Selection,
}
impl TextEditor {
    /// Create a new TextEditor, with the cursor in line 0, collum 0.
    pub fn new() -> Self {
        Self {
            selection: Selection {
                cursor: Position {
                    line: 0,
                    collumn: 0,
                },
                cursor_x: 0.0,
                anchor: Position {
                    line: 0,
                    collumn: 0,
                },
            },
        }
    }

    /// Get the index of the glyph that contains the given byte index in its range.
    fn get_glyph_from_byte_index(
        &mut self,
        index: usize,
        text_layout: &mut TextLayout,
    ) -> Option<usize> {
        let glyph = text_layout
            .glyphs()
            .binary_search_by(|x| cmp_range(index, x.byte_range.clone()))
            .ok()?;
        Some(glyph)
    }

    /// Get the position, given the byte index in the text string.
    fn get_position_from_byte_index(
        &mut self,
        index: usize,
        text_layout: &mut TextLayout,
    ) -> Position {
        let glyph = self
            .get_glyph_from_byte_index(index, text_layout)
            .unwrap_or(0);
        let lines = text_layout.lines();
        let line = lines
            .binary_search_by(|x| cmp_range(glyph, x.glyph_range.clone()))
            .unwrap_or_else(|x| x);
        let byte_range = self.get_line_byte_range(line, text_layout);
        let offset = index - byte_range.clone().start;
        let collumn = text_layout.text()[byte_range]
            .char_indices()
            .map(|(i, _)| i)
            .take_while(|x| *x <= offset)
            .count()
            - 1;
        Position { line, collumn }
    }

    /// Get the byte index in the text string, given a Position.
    fn get_byte_index(&mut self, position: Position, text_layout: &mut TextLayout) -> usize {
        let Position { line, collumn } = position;
        let byte_range = self.get_line_byte_range(line, text_layout);
        let (offset, _) = text_layout.text()[byte_range.clone()]
            .char_indices()
            .nth(collumn)
            .unwrap_or((0, ' '));
        byte_range.start + offset
    }

    /// Get the x and y position of the caret, in pixels, for the given position.
    fn get_pixel_position(&mut self, position: Position, text_layout: &mut TextLayout) -> [f32; 2] {
        let byte = self.get_byte_index(position, text_layout);
        let glyph = self.get_glyph_from_byte_index(byte, text_layout);
        let glyph = if let Some(x) = glyph {
            x
        } else {
            return [0.0, 0.0];
        };
        let pos = text_layout.glyphs()[glyph].glyph.position;
        [pos.x, pos.y]
    }

    /// Get the byte range of the currently selected text. This range can be used to slice the
    /// text string
    pub fn selection_range(&mut self, text_layout: &mut TextLayout) -> Range<usize> {
        if self.selection.is_empty() {
            let byte = self.get_byte_index(self.selection.cursor, text_layout);
            byte..byte
        } else {
            let mut start = self.get_byte_index(self.selection.cursor, text_layout);
            let mut end = self.get_byte_index(self.selection.anchor, text_layout);
            if start > end {
                std::mem::swap(&mut start, &mut end);
            }
            start..end
        }
    }

    /// Get the glyph range of the given line.
    fn get_line_glyph_range(&mut self, line: usize, text_layout: &mut TextLayout) -> Range<usize> {
        let lines = text_layout.lines();
        let l = if line < lines.len() {
            &lines[line]
        } else {
            lines.last().unwrap()
        };
        l.glyph_range.clone()
    }

    /// Get the byte range of the given line.
    fn get_line_byte_range(&mut self, line: usize, text_layout: &mut TextLayout) -> Range<usize> {
        let glyph_range = self.get_line_glyph_range(line, text_layout);
        let byte_range = {
            let glyphs = text_layout.glyphs();
            if glyphs.is_empty() {
                return 0..0;
            }
            if glyph_range.len() > 0 {
                glyphs[glyph_range.start].byte_range.start
                    ..glyphs[glyph_range.end - 1].byte_range.end
            } else {
                let r = glyphs[glyph_range.start].byte_range.start;
                r..r
            }
        };
        byte_range
    }

    /// Offset a position by a give amount of graphene clusters. Offset to the right if
    /// delta_x is positive, and left if it is negative.
    fn offset_position(
        &mut self,
        position: Position,
        delta_x: i32,
        text_layout: &mut TextLayout,
    ) -> Position {
        if delta_x == 0 {
            return position;
        }
        let byte_index = self.get_byte_index(position, text_layout);
        if delta_x > 0 {
            let n = delta_x as usize;
            let (offset, _) = text_layout.text()[byte_index..]
                .char_indices()
                .take(n + 1)
                .last()
                .unwrap();
            let target = byte_index + offset;
            self.get_position_from_byte_index(target, text_layout)
        } else {
            let n = (-delta_x) as usize;
            let offset = text_layout.text()[..byte_index]
                .char_indices()
                .rev()
                .take(n)
                .map(|x| x.0)
                .last();
            if let Some(offset) = offset {
                self.get_position_from_byte_index(offset, text_layout)
            } else {
                position
            }
        }
    }

    /// Return the x y position of the top of the caret, and its height.
    pub fn get_cursor_position_and_height(&mut self, text_layout: &mut TextLayout) -> [f32; 3] {
        let (descent, height) = text_layout
            .lines()
            .get(self.selection.cursor.line)
            .map_or((0.0, 0.0), |x| (x.descent, x.height()));
        let [x, y] = self.get_pixel_position(self.selection.cursor, text_layout);
        [x, y - descent, height]
    }

    /// Move the cursor horizontaly, by the given number of graphene clusters. Move to the right if
    /// delta_x is positive, and left if it is negative. If expand_selection
    /// is true, the anchor of the selection will be preseved. Otherwise, the selection is clear.
    pub fn move_cursor_hor(
        &mut self,
        delta_x: i32,
        expand_selection: bool,
        text_layout: &mut TextLayout,
    ) {
        let cursor = self.selection.cursor;
        let cursor = self.offset_position(cursor, delta_x, text_layout);
        self.selection.cursor_x = self.get_pixel_position(cursor, text_layout)[0];
        if expand_selection {
            self.selection.cursor = cursor;
        } else {
            if self.selection.is_empty() {
                self.selection.set_pos(cursor);
            } else {
                let cursor = if delta_x > 0 {
                    self.selection.end()
                } else {
                    self.selection.start()
                };
                self.selection.set_pos(cursor);
            }
        }
    }

    /// Move the cursor to the start of the currently line. If expand_selection is true, the anchor
    /// of the selection will be preserved. Otherwise, the selection is clear.
    pub fn move_cursor_line_start(&mut self, expand_selection: bool, text_layout: &mut TextLayout) {
        let line_range = self.get_line_byte_range(self.selection.cursor.line, text_layout);
        let cursor = self.get_position_from_byte_index(line_range.start, text_layout);
        if expand_selection {
            self.selection.cursor = cursor;
        } else {
            self.selection.set_pos(cursor);
        }
    }

    /// Move the cursor to the end of the currently line. If expand_selection is true, the anchor
    /// of the selection will be preserved. Otherwise, the selection is clear.
    pub fn move_cursor_line_end(&mut self, expand_selection: bool, text_layout: &mut TextLayout) {
        let line_range = self.get_line_byte_range(self.selection.cursor.line, text_layout);
        let cursor = self.get_position_from_byte_index(line_range.end, text_layout);
        // let cursor = self.offset_position(cursor, -1, text_layout);
        if expand_selection {
            self.selection.cursor = cursor;
        } else {
            self.selection.set_pos(cursor);
        }
    }

    /// Move the cursor verticaly, by the given number of lines. Moves up, if lines is negative,
    /// and moves down, if positive. If expand_selection is true, the anchor of the selection will
    /// be preseved. Otherwise, the selection is clear.
    pub fn move_cursor_vert(
        &mut self,
        lines: i32,
        expand_selection: bool,
        text_layout: &mut TextLayout,
    ) {
        fn add(a: usize, b: i32) -> usize {
            if b >= 0 {
                a.saturating_add(b as usize)
            } else {
                a.saturating_sub((-b) as usize)
            }
        }

        let mut line = self.selection.cursor.line;
        line = add(line, lines);
        let num_lines = text_layout.lines().len();
        if line >= num_lines {
            line = num_lines - 1;
        }

        let line_range = self.get_line_glyph_range(line, text_layout);
        let cursor_x = self.selection.cursor_x;
        let glyph = text_layout.glyphs()[line_range]
            .iter()
            .take_while(|x| x.glyph.position.x < cursor_x)
            .last()
            .unwrap();

        // check if it more to the left or to the right of the glyph
        let s = (self.selection.cursor_x - glyph.glyph.position.x) / glyph.width;
        let byte = if s < 0.5 {
            glyph.byte_range.start
        } else {
            glyph.byte_range.end
        };
        let cursor = self.get_position_from_byte_index(byte, text_layout);

        if expand_selection {
            self.selection.cursor = cursor;
        } else {
            self.selection.set_pos(cursor);
        }
    }

    /// Replace the currently selected text by the given one. The currently selection can be empty,
    /// so this only inserts the text at the caret position. At the end, the cursor is moved to the
    /// end of the inserted text.
    ///
    /// If the given text is empty, this acts as a selection deletion.
    pub fn insert_text(&mut self, text: &str, fonts: &Fonts, text_layout: &mut TextLayout) {
        let range = self.selection_range(text_layout);
        text_layout.replace_range(range.clone(), text, fonts);
        let target = range.start + text.len();
        let pos = self.get_position_from_byte_index(target, text_layout);
        assert_eq!(self.get_byte_index(pos, text_layout), target);
        self.selection.set_pos(pos);
        self.selection.cursor_x = self.get_pixel_position(pos, text_layout)[0];
    }

    /// If the selection is empty, delete horizontaly, by the given amount of graphene clusters.
    /// Deletes right if delta_x is positive, and deletes left if delta_x is negative. If there is
    /// selection, the selected text is deleted, and delta_x is ignored.
    pub fn delete_hor(&mut self, delta_x: i32, fonts: &Fonts, text_layout: &mut TextLayout) {
        if self.selection.is_empty() {
            let anchor = self.selection.anchor;
            let anchor = self.offset_position(anchor, delta_x, text_layout);

            self.selection.anchor = anchor;
        }

        self.insert_text("", fonts, text_layout);
    }
}
