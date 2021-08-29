use crate::{font::Fonts, text::layout::TextLayout, util::cmp_range};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

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

#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Default)]
pub struct ByteIndex(usize);

/// A selection of text. This can also represent a cursor.
pub struct Selection {
    /// The position of the cursor. Can also be the selection ends. Can be before or after the
    /// anchor.
    cursor: ByteIndex,
    /// The position where the selection starts. Can be before or after the cursor.
    anchor: ByteIndex,
    /// The x position of the cursor. This only updated in horizontal motion. This is use to keep
    /// the cursor motion align when moving vertically multiple lines.
    cursor_x: f32,
}
impl Selection {
    /// Set cursor and anchor to the same value.
    pub fn set_pos(&mut self, pos: ByteIndex) {
        self.cursor = pos;
        self.anchor = pos;
    }

    /// Check if the selection is empty, i.e., cursor == anchor
    pub fn is_empty(&self) -> bool {
        self.cursor == self.anchor
    }

    /// Return the position of the start of the selection. This is the min(cursor, anchor).
    pub fn start(&self) -> ByteIndex {
        if self.cursor < self.anchor {
            self.cursor
        } else {
            self.anchor
        }
    }

    /// Return the position of the end of the selection. This is the max(cursor, anchor).
    pub fn end(&self) -> ByteIndex {
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
                cursor: ByteIndex(0),
                cursor_x: 0.0,
                anchor: ByteIndex(0),
            },
        }
    }

    fn get_line_from_byte_index(&mut self, byte_index: usize, text_layout: &TextLayout) -> usize {
        let lines = text_layout.lines();
        lines
            .binary_search_by(|x| cmp_range(byte_index, x.byte_range.clone()))
            .unwrap_or(lines.len())
    }

    /// Get the position, given the byte index in the text string.
    pub fn get_position_from_byte_index(
        &mut self,
        byte_index: usize,
        text_layout: &TextLayout,
    ) -> Position {
        let line = self.get_line_from_byte_index(byte_index, text_layout);
        let byte_range = text_layout.lines()[line].byte_range.clone();
        if byte_range.is_empty() {
            return Position { line, collumn: 0 };
        }
        let offset = byte_index - byte_range.clone().start;
        let collumn = text_layout.text()[byte_range]
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .take_while(|x| *x <= offset)
            .count()
            - 1;
        Position { line, collumn }
    }

    /// Get the byte index in the text string, given a Position.
    pub fn get_byte_index(&mut self, position: Position, text_layout: &TextLayout) -> usize {
        let Position { line, collumn } = position;
        let byte_range = text_layout.lines()[line].byte_range.clone();
        let (offset, _) = text_layout.text()[byte_range.clone()]
            .grapheme_indices(true)
            .nth(collumn)
            .unwrap_or((0, ""));
        byte_range.start + offset
    }

    /// Get the byte range of the currently selected text. This range can be used to slice the
    /// text string
    pub fn selection_range(&mut self) -> Range<usize> {
        if self.selection.is_empty() {
            let byte = self.selection.cursor.0;
            byte..byte
        } else {
            let mut start = self.selection.cursor.0;
            let mut end = self.selection.anchor.0;
            if start > end {
                std::mem::swap(&mut start, &mut end);
            }
            start..end
        }
    }

    /// Offset a position by a give amount of graphene clusters. Offset to the right if
    /// delta_x is positive, and left if it is negative.
    fn offset_byte_index(
        &mut self,
        position: ByteIndex,
        delta_x: i32,
        text_layout: &TextLayout,
    ) -> ByteIndex {
        if delta_x == 0 {
            return position;
        }
        let byte_index = position.0;
        if delta_x > 0 {
            let n = delta_x as usize;
            let (offset, _) = text_layout.text()[byte_index..]
                .grapheme_indices(true)
                .take(n + 1)
                .last()
                .unwrap();
            let target = byte_index + offset;
            ByteIndex(target)
        } else {
            let n = (-delta_x) as usize;
            let offset = text_layout.text()[..byte_index]
                .grapheme_indices(true)
                .rev()
                .take(n)
                .map(|x| x.0)
                .last();
            if let Some(offset) = offset {
                ByteIndex(offset)
            } else {
                position
            }
        }
    }

    /// Return the x y position of the top of the caret, and its height, in pixels. The position is
    /// relative to the algnment anchor of the text.
    pub fn get_caret_position_and_height(&mut self, text_layout: &TextLayout) -> [f32; 3] {
        let byte_index = self.selection.cursor.0;
        let (height, descent) = {
            let line = self.get_line_from_byte_index(byte_index, text_layout);
            let line = &text_layout.lines()[line];
            (line.height(), line.descent)
        };
        let [x, y] = text_layout
            .pixel_position_from_byte_index(byte_index)
            .unwrap_or([0.0, 0.0]);
        [x, y - descent, height]
    }

    /// Move the cursor horizontaly, by the given number of graphene clusters. Move to the right if
    /// delta_x is positive, and left if it is negative. If expand_selection
    /// is true, the anchor of the selection will be preseved. Otherwise, the selection is clear.
    pub fn move_cursor_hor(
        &mut self,
        delta_x: i32,
        expand_selection: bool,
        text_layout: &TextLayout,
    ) {
        let cursor = self.selection.cursor;
        let cursor = self.offset_byte_index(cursor, delta_x, text_layout);
        self.selection.cursor_x = text_layout
            .pixel_position_from_byte_index(cursor.0)
            .unwrap_or([0.0, 0.0])[0];
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
    pub fn move_cursor_line_start(&mut self, expand_selection: bool, text_layout: &TextLayout) {
        let line = self.get_line_from_byte_index(self.selection.cursor.0, text_layout);
        let cursor = ByteIndex(text_layout.lines()[line].byte_range.start);
        if expand_selection {
            self.selection.cursor = cursor;
        } else {
            self.selection.set_pos(cursor);
        }
    }

    /// Move the cursor to the end of the currently line. If expand_selection is true, the anchor
    /// of the selection will be preserved. Otherwise, the selection is clear.
    pub fn move_cursor_line_end(&mut self, expand_selection: bool, text_layout: &TextLayout) {
        let line = self.get_line_from_byte_index(self.selection.cursor.0, text_layout);
        let cursor = ByteIndex(text_layout.lines()[line].byte_range.end - 1);
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
        text_layout: &TextLayout,
    ) {
        let curr_line = self.get_line_from_byte_index(self.selection.cursor.0, text_layout);
        let target = curr_line as isize + lines as isize;
        let lines = text_layout.lines();
        let cursor = if target < 0 {
            // if move to a negative line, go to the start of the text.
            ByteIndex(0)
        } else if target >= lines.len() as isize {
            // if move to a out of bounds line, go to the end of the text.
            ByteIndex(lines.last().unwrap().byte_range.end - 1)
        } else {
            let index = text_layout
                .byte_index_from_x_position(target as usize, self.selection.cursor_x)
                .unwrap();
            ByteIndex(index)
        };

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
        let range = self.selection_range();
        text_layout.replace_range(range.clone(), text, fonts);
        let byte_index = range.start + text.len();
        self.selection.set_pos(ByteIndex(byte_index));
        self.selection.cursor_x = text_layout
            .pixel_position_from_byte_index(byte_index)
            .unwrap_or([0.0, 0.0])[0];
    }

    /// If the selection is empty, delete horizontaly, by the given amount of graphene clusters.
    /// Deletes right if delta_x is positive, and deletes left if delta_x is negative. If there is
    /// selection, the selected text is deleted, and delta_x is ignored.
    pub fn delete_hor(&mut self, delta_x: i32, fonts: &Fonts, text_layout: &mut TextLayout) {
        if self.selection.is_empty() {
            let anchor = self.selection.anchor;
            let anchor = self.offset_byte_index(anchor, delta_x, text_layout);

            self.selection.anchor = anchor;
        }

        self.insert_text("", fonts, text_layout);
    }
}
