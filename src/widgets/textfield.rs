use crate::{
    event::{self, SetValue},
    graphics::Graphic,
    style::TextFieldStyle,
    text_layout::{LineMetrics, TextLayout},
    Behaviour, Context, Id, InputFlags, KeyboardEvent, MouseButton, MouseEvent, MouseInfo, Span,
};

use copypasta::{ClipboardContext, ClipboardProvider};
use std::{
    any::Any,
    time::{Duration, Instant},
};
use std::{ops::Range, rc::Rc};
use winit::{event::VirtualKeyCode, window::CursorIcon};

pub trait TextFieldCallback {
    fn on_submit(&mut self, this: Id, ctx: &mut Context, text: &mut String) -> bool;
    fn on_change(&mut self, this: Id, ctx: &mut Context, text: &str);
    fn on_unfocus(&mut self, this: Id, ctx: &mut Context, text: &mut String) -> bool;
}
impl<F: FnMut(Id, &mut Context, &mut String) -> bool + 'static> TextFieldCallback for F {
    fn on_submit(&mut self, this: Id, ctx: &mut Context, text: &mut String) -> bool {
        self(this, ctx, text)
    }
    fn on_change(&mut self, _: Id, _: &mut Context, _: &str) {}
    fn on_unfocus(&mut self, _: Id, _: &mut Context, _: &mut String) -> bool {
        true
    }
}
impl TextFieldCallback for () {
    fn on_submit(&mut self, _: Id, _: &mut Context, _: &mut String) -> bool {
        true
    }
    fn on_change(&mut self, _: Id, _: &mut Context, _: &str) {}
    fn on_unfocus(&mut self, _: Id, _: &mut Context, _: &mut String) -> bool {
        true
    }
}

struct BlinkCaret;

pub struct TextField<C: TextFieldCallback> {
    callback: C,
    caret: Id,
    label: Id,
    text: String,
    previous_text: String,
    /// Index of glyph where the caret is positioned
    caret_index: usize,
    selection_index: Option<usize>,
    text_layout: TextLayout,
    text_width: f32,
    this_width: f32,
    /// The amount in pixels that the text is scrolled to the left.
    /// When there is no scroll, its value is -MARGIN.
    x_scroll: f32,
    on_focus: bool,
    mouse_x: f32,
    /// If it is non zero, the mouse is being dragged. 1 for single click, 2 for double click, etc...
    mouse_down: u8,
    drag_start: usize,
    style: Rc<TextFieldStyle>,
    blink: bool,
    /// event_id of the last scheduled BlinkCaret event
    blink_event: Option<u64>,
}
impl<C: TextFieldCallback> TextField<C> {
    pub fn new(text: String, caret: Id, label: Id, style: Rc<TextFieldStyle>, callback: C) -> Self {
        Self {
            callback,
            caret,
            label,
            previous_text: text.clone(),
            text,
            caret_index: 0,
            selection_index: None,
            text_layout: TextLayout::new(),
            text_width: 0.0,
            this_width: 0.0,
            x_scroll: 0.0,
            on_focus: false,
            mouse_x: 0.0,
            mouse_down: 0,
            drag_start: 0,
            style,
            blink: false,
            blink_event: None,
        }
    }

    fn update_text(&mut self, this: Id, ctx: &mut Context) {
        let fonts = ctx.get_fonts();
        if let Some((rect, Graphic::Text(text))) = ctx.get_rect_and_graphic(self.label) {
            // add a extra char for the sake of the carret at end position.
            let display_text = self.text.clone() + " ";
            text.set_text(&display_text);
            let min_size = text.compute_min_size(fonts).unwrap_or([0.0, 0.0]);
            self.text_width = min_size[0];
            rect.set_min_size(min_size);
            self.text_layout = text.get_layout(fonts, rect).clone();
            let glyphs = self.text_layout.glyphs();
            if self.caret_index + 1 >= glyphs.len() {
                self.caret_index = glyphs.len().saturating_sub(1);
            }
            self.update_carret(this, ctx, true);
        }
    }

    /// Get the position of the glyph under right to the carret, relative to the top-left
    /// corner of the text control.
    fn get_glyph_pos(&mut self, caret: usize) -> [f32; 2] {
        let glyphs = self.text_layout.glyphs();
        if glyphs.is_empty() {
            unreachable!()
        }
        match glyphs.get(caret) {
            Some(glyph) => {
                let pos = glyph.glyph.position;
                [pos.x, pos.y]
            }
            None => panic!("index is {}, but len is {}", caret, glyphs.len()),
        }
    }

    fn get_line(&self, caret: usize) -> &LineMetrics {
        let lines = self.text_layout.line_metrics();
        let i = lines
            .binary_search_by(|x| crate::util::cmp_range(caret, x.start_glyph..x.end_glyph))
            .unwrap_or(0);
        &lines[i]
    }

    /// Get the position of the base of the caret, and its height.
    /// The bottom of the caret is in the descent of the line, and its top is in the ascent.
    fn get_caret_pos_and_height(&mut self, caret: usize) -> [f32; 3] {
        let line = self.get_line(caret).clone();
        let pos = self.get_glyph_pos(caret);
        [pos[0], pos[1] - line.descent, line.ascent - line.descent]
    }

    /// Get the byte range in the text for a caret position
    fn get_byte_range(&mut self, caret: usize) -> std::ops::Range<usize> {
        match self.text_layout.glyphs().get(caret) {
            Some(glyph) => glyph.byte_range.clone(),
            None => self.text.len()..self.text.len(),
        }
    }

    /// Get the index of the glyph located at certain line and x position
    fn get_caret_index_at_pos(&mut self, line: usize, x_pos: f32) -> usize {
        let lines = self.text_layout.line_metrics();
        let line = lines[line.max(lines.len() - 1)].clone();
        let range = line.start_glyph..line.end_glyph;
        if range.is_empty() {
            return line.start_glyph;
        }
        let line_glyphs = &self.text_layout.glyphs()[range];
        match line_glyphs.binary_search_by(|x| x.glyph.position.x.partial_cmp(&x_pos).unwrap()) {
            Ok(i) => line.start_glyph + i,
            Err(i) => {
                if i == 0 {
                    line.start_glyph
                } else {
                    (line.start_glyph + i - 1).min(line.end_glyph - 1)
                }
            }
        }
    }

    /// Get the caret located at certain byte index in the string
    fn get_caret_index(&mut self, indice: usize) -> usize {
        let glyphs = self.text_layout.glyphs();
        if glyphs.is_empty() {
            return 0;
        }
        match glyphs.binary_search_by(|x| crate::util::cmp_range(indice, x.byte_range.clone())) {
            Ok(x) => x,
            Err(x) => x.saturating_sub(1),
        }
    }

    fn update_carret(&mut self, this: Id, ctx: &mut Context, focus_caret: bool) {
        let mut caret_pos = self.get_caret_pos_and_height(self.caret_index);
        if let Some(event_id) = self.blink_event {
            ctx.cancel_scheduled_event(event_id);
        } else {
            self.blink = false;
        }

        const MARGIN: f32 = 5.0;

        let this_rect = *ctx.get_rect(this);
        self.this_width = this_rect[2] - this_rect[0];

        if self.this_width - MARGIN * 2.0 > self.text_width {
            self.x_scroll = -MARGIN;
        } else {
            self.x_scroll = self
                .x_scroll
                .min(self.text_width - self.this_width + MARGIN);
            if focus_caret {
                if caret_pos[0] - self.x_scroll > self.this_width - MARGIN {
                    self.x_scroll = caret_pos[0] - (self.this_width - MARGIN);
                }
                if caret_pos[0] - self.x_scroll < MARGIN {
                    self.x_scroll = caret_pos[0] - MARGIN;
                }
            } else {
                if self.text_width - self.x_scroll < self.this_width - MARGIN {
                    self.x_scroll = self.text_width - (self.this_width - MARGIN);
                }
                if self.x_scroll < -MARGIN {
                    self.x_scroll = -MARGIN;
                }
            }
        }

        ctx.set_margin_left(self.label, -self.x_scroll);

        caret_pos[0] -= self.x_scroll;

        if let Some(_) = self.selection_index {
            ctx.set_margins(self.caret, [0.0; 4]);
            if let Graphic::Text(text) = ctx.get_graphic_mut(self.label) {
                text.clear_spans();
                let range = self.selection_range().unwrap();
                text.add_span(
                    range,
                    Span {
                        color: self.style.selection_color.fg,
                        background: Some(self.style.selection_color.bg),
                        ..Default::default()
                    },
                );
            }
        } else {
            if let Graphic::Text(text) = ctx.get_graphic_mut(self.label) {
                text.clear_spans();
            }
            ctx.get_graphic_mut(self.caret)
                .set_color(self.style.caret_color);
            if self.on_focus {
                self.blink_event = Some(ctx.send_event_to_scheduled(
                    this,
                    BlinkCaret,
                    Instant::now() + Duration::from_millis(500),
                ));
            }
            if self.on_focus && !self.blink {
                ctx.set_margins(
                    self.caret,
                    [
                        caret_pos[0],
                        caret_pos[1] - caret_pos[2],
                        caret_pos[0] + 1.0,
                        caret_pos[1],
                    ],
                );
            } else {
                ctx.set_margins(self.caret, [0.0, 0.0, 0.0, 0.0]);
            }
        }
    }

    fn move_caret(&mut self, caret: usize, ctx: &mut Context) {
        if ctx.modifiers().shift() {
            if let Some(selection_index) = self.selection_index {
                if selection_index == caret {
                    self.selection_index = None;
                }
            } else {
                self.selection_index = Some(self.caret_index);
            }
        } else if let Some(selection_index) = self.selection_index {
            let start = selection_index;
            let end = self.caret_index;
            if (caret < self.caret_index) ^ (start > end) {
                self.caret_index = start;
            } else {
                self.caret_index = end;
            }
            self.selection_index = None;
            return;
        }
        self.caret_index = caret;
    }

    /// return the byte range of the selected text
    fn selection_range(&mut self) -> Option<Range<usize>> {
        let selection_index = self.selection_index?;
        let a = self.get_byte_range(self.caret_index);
        let b = self.get_byte_range(selection_index);
        let range = if a.start > b.start {
            b.start..a.start
        } else {
            a.start..b.start
        };
        Some(range)
    }

    fn delete_selection(&mut self, this: Id, ctx: &mut Context) {
        let range = self.selection_range().unwrap();
        let selection_index = self.selection_index.unwrap();
        if self.caret_index > selection_index {
            self.caret_index = selection_index;
        }
        self.selection_index = None;
        self.text.replace_range(range, "");
        self.update_text(this, ctx);
        self.callback.on_change(this, ctx, &self.text)
    }

    fn insert_char(&mut self, ch: char, this: Id, ctx: &mut Context) {
        let indice = self.get_byte_range(self.caret_index).start;
        self.text.insert(indice, ch);
        self.caret_index += 1;
        self.update_text(this, ctx);
        self.callback.on_change(this, ctx, &self.text)
    }

    fn get_word_start(&mut self, mut caret: usize) -> usize {
        let mut s = false;
        while caret != 0 {
            let indice = self.get_byte_range(caret).start;
            let whitespace = match self.text[indice..].chars().next() {
                Some(x) => x.is_whitespace(),
                None => false,
            };
            if !whitespace {
                s = true;
            } else if s {
                caret += 1;
                break;
            }
            caret -= 1;
        }
        caret
    }

    fn get_next_word_start(&mut self, mut caret: usize) -> usize {
        let mut s = false;
        loop {
            let indice = self.get_byte_range(caret).start;
            let whitespace = match self.text[indice..].chars().next() {
                Some(x) => x.is_whitespace(),
                None => {
                    caret = self.text_layout.glyphs().len() - 1;
                    break;
                }
            };
            if whitespace {
                s = true;
            } else if s {
                break;
            }
            caret += 1;
        }
        caret
    }

    fn get_token_start(&mut self, mut caret: usize) -> usize {
        let indice = self.get_byte_range(caret).start;
        let start = match self.text[indice..].chars().next() {
            Some(x) => x.is_whitespace(),
            None => false,
        };
        loop {
            let indice = self.get_byte_range(caret).start;
            let whitespace = match self.text[indice..].chars().next() {
                Some(x) => x.is_whitespace(),
                None => false,
            };
            if whitespace != start {
                caret += 1;
                break;
            }
            if caret == 0 {
                break;
            }
            caret -= 1;
        }
        caret
    }

    fn get_token_end(&mut self, mut caret: usize) -> usize {
        let indice = self.get_byte_range(caret).start;
        let start = match self.text[indice..].chars().next() {
            Some(x) => x.is_whitespace(),
            None => {
                return self.text_layout.glyphs().len().saturating_sub(1);
            }
        };
        loop {
            let indice = self.get_byte_range(caret).start;
            let whitespace = match self.text[indice..].chars().next() {
                Some(x) => x.is_whitespace(),
                None => {
                    caret = self.text_layout.glyphs().len().saturating_sub(1);
                    break;
                }
            };
            if whitespace != start {
                break;
            }
            caret += 1;
        }
        caret
    }

    fn select_all(&mut self, this: Id, ctx: &mut Context) {
        let start = 0;
        let end = self.get_line(self.caret_index).end_glyph.saturating_sub(1);
        self.selection_index = Some(start);
        self.caret_index = end;
        self.update_carret(this, ctx, false);
    }
}
impl<C: TextFieldCallback> Behaviour for TextField<C> {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        self.update_text(this, ctx);
        ctx.move_to_front(self.label);
        ctx.set_graphic(this, self.style.background.normal.clone());
    }

    fn on_event(&mut self, event: Box<dyn Any>, this: Id, ctx: &mut Context) {
        if let Some(SetValue(text)) = event.downcast_ref::<SetValue<String>>() {
            let x = self.get_glyph_pos(self.caret_index)[0];
            self.text.clone_from(text);
            self.previous_text.clone_from(text);
            self.update_text(this, ctx);
            self.selection_index = None;
            self.caret_index = self.get_caret_index_at_pos(0, x);
            self.update_carret(this, ctx, true);
            self.callback.on_change(this, ctx, &self.text);
        } else if event.is::<BlinkCaret>() {
            self.blink = !self.blink;
            self.update_carret(this, ctx, false);
        }
    }

    fn input_flags(&self) -> InputFlags {
        let mut flags = InputFlags::MOUSE | InputFlags::FOCUS;

        if self.text_width > self.this_width {
            flags |= InputFlags::SCROLL;
        }
        flags
    }

    fn on_scroll_event(&mut self, delta: [f32; 2], this: Id, ctx: &mut Context) {
        let delta = if delta[0].abs() != 0.0 {
            delta[0]
        } else {
            delta[1]
        };
        self.x_scroll -= delta;
        self.update_carret(this, ctx, false);
    }

    fn on_mouse_event(&mut self, mouse: MouseInfo, this: Id, ctx: &mut Context) {
        use MouseButton::*;
        match mouse.event {
            MouseEvent::Enter => {
                ctx.set_cursor(CursorIcon::Text);
            }
            MouseEvent::Exit => {
                ctx.set_cursor(CursorIcon::Default);
            }
            MouseEvent::Down(Left) => {
                if let Some(event_id) = self.blink_event.take() {
                    ctx.cancel_scheduled_event(event_id);
                }
                if self.blink {
                    self.update_carret(this, ctx, false);
                }

                let left = ctx.get_rect(this)[0] - self.x_scroll;
                let x = self.mouse_x - left;
                let caret = self.get_caret_index_at_pos(0, x);
                if caret == self.drag_start {
                    match mouse.click_count {
                        0 => unreachable!(),
                        1 => {
                            self.caret_index = caret;
                            self.mouse_down = 1;
                            self.selection_index = None;
                        }
                        2 => {
                            let caret = self.caret_index;
                            self.caret_index = self.get_token_start(caret);
                            self.mouse_down = 2;
                            self.selection_index = Some(self.get_token_end(caret));
                        }
                        3..=u8::MAX => {
                            self.select_all(this, ctx);
                        }
                    }
                } else {
                    if mouse.click_count > 1 {
                        ctx.reset_click_count_to_one();
                    }
                    self.caret_index = caret;
                    self.mouse_down = 1;
                    self.selection_index = None;
                }
                self.drag_start = caret;
                self.update_carret(this, ctx, true);
                ctx.send_event(event::LockOver);
            }
            MouseEvent::Up(Left) => {
                self.mouse_down = 0;
                ctx.send_event(event::UnlockOver);
            }
            MouseEvent::Moved => {
                let [x, _] = mouse.pos;
                self.mouse_x = x;
                match self.mouse_down {
                    0 => {}
                    1 => {
                        let left = ctx.get_rect(this)[0] - self.x_scroll;
                        let x = self.mouse_x - left;
                        let caret_index = self.get_caret_index_at_pos(0, x);
                        if caret_index == self.caret_index {
                            return;
                        }
                        if let Some(selection_index) = self.selection_index {
                            if caret_index == selection_index {
                                self.selection_index = None;
                            }
                        } else {
                            self.selection_index = Some(self.caret_index);
                        }
                        self.caret_index = caret_index;
                        self.update_carret(this, ctx, true);
                    }
                    2..=u8::MAX => {
                        let left = ctx.get_rect(this)[0] - self.x_scroll;
                        let x = self.mouse_x - left;
                        let caret_index = self.get_caret_index_at_pos(0, x);
                        let selection_index = self.selection_index.unwrap_or(caret_index);
                        let caret_index = if selection_index > caret_index {
                            self.selection_index = Some(self.get_token_end(self.drag_start));
                            self.get_token_start(caret_index)
                        } else {
                            self.selection_index = Some(self.get_token_start(self.drag_start));
                            self.get_token_end(caret_index)
                        };
                        self.caret_index = caret_index;
                        self.update_carret(this, ctx, true);
                    }
                }
            }
            MouseEvent::Up(_) => {}
            MouseEvent::Down(_) => {}
            MouseEvent::None => {}
        }
    }

    fn on_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {
        self.on_focus = focus;
        if focus {
            ctx.set_graphic(this, self.style.background.focus.clone());
        } else {
            ctx.set_graphic(this, self.style.background.normal.clone());

            let x = self.get_glyph_pos(self.caret_index)[0];
            if self.callback.on_unfocus(this, ctx, &mut self.text) {
                self.previous_text.clone_from(&self.text);
            } else {
                self.text.clone_from(&self.previous_text);
            }
            self.selection_index = None;
            self.caret_index = self.get_caret_index_at_pos(0, x);
            self.update_text(this, ctx);
        }
        self.update_carret(this, ctx, true);
    }

    fn on_keyboard_event(&mut self, event: KeyboardEvent, this: Id, ctx: &mut Context) -> bool {
        if let Some(event_id) = self.blink_event.take() {
            ctx.cancel_scheduled_event(event_id);
        }
        self.update_carret(this, ctx, false);
        match event {
            KeyboardEvent::Char(ch) => {
                if self.selection_index.is_some() {
                    self.delete_selection(this, ctx);
                }
                self.insert_char(ch, this, ctx);
            }
            KeyboardEvent::Pressed(key_code) => match key_code {
                VirtualKeyCode::Tab => return false, // allow change focus with tab
                VirtualKeyCode::C | VirtualKeyCode::X => {
                    if ctx.modifiers().ctrl() {
                        if let Some(selection_index) = self.selection_index {
                            let a = self.get_byte_range(selection_index).start;
                            let b = self.get_byte_range(self.caret_index).start;
                            let range = if a < b { a..b } else { b..a };
                            let mut cliptobard = ClipboardContext::new().unwrap();
                            let _ = cliptobard.set_contents(self.text[range].to_owned());
                            if key_code == VirtualKeyCode::X {
                                self.delete_selection(this, ctx);
                            }
                        }
                    }
                }
                VirtualKeyCode::V => {
                    if ctx.modifiers().ctrl() {
                        let mut clipboard = ClipboardContext::new().unwrap();
                        if let Ok(text) = clipboard.get_contents() {
                            let text = text.replace(|x: char| x.is_control(), "");
                            let range = self.get_byte_range(self.caret_index);
                            if let Some(selection_index) = self.selection_index {
                                let a = self.get_byte_range(selection_index);
                                let b = range;
                                let range = if a.start < b.start {
                                    a.start..b.start
                                } else {
                                    b.start..a.start
                                };
                                self.text.replace_range(range.clone(), &text);
                                self.selection_index = None;
                                self.update_text(this, ctx); // TODO: is is not working?
                                self.caret_index = self.get_caret_index(range.start + text.len());
                                self.callback.on_change(this, ctx, &self.text)
                            } else {
                                self.text.insert_str(range.start, &text);
                                self.update_text(this, ctx);
                                self.caret_index = self.get_caret_index(range.start + text.len());
                                self.callback.on_change(this, ctx, &self.text)
                            }
                            self.update_carret(this, ctx, true);
                        }
                    }
                }
                VirtualKeyCode::A => {
                    if ctx.modifiers().ctrl() {
                        self.select_all(this, ctx);
                    }
                }
                VirtualKeyCode::Return => {
                    let x = self.get_glyph_pos(self.caret_index)[0];
                    if self.callback.on_submit(this, ctx, &mut self.text) {
                        self.previous_text.clone_from(&self.text);
                    } else {
                        self.text.clone_from(&self.previous_text);
                    }
                    self.update_text(this, ctx);
                    self.selection_index = None;
                    self.caret_index = self.get_caret_index_at_pos(0, x);
                    self.update_carret(this, ctx, true);
                }
                VirtualKeyCode::Back | VirtualKeyCode::Delete if self.selection_index.is_some() => {
                    self.delete_selection(this, ctx);
                }
                VirtualKeyCode::Back => {
                    if self.caret_index == 0 {
                        return true;
                    }
                    self.caret_index -= 1;
                    let range = self.get_byte_range(self.caret_index);
                    self.text.replace_range(range, "");
                    self.update_text(this, ctx);
                    self.callback.on_change(this, ctx, &self.text);
                }
                VirtualKeyCode::Delete => {
                    if self.caret_index + 1 < self.text_layout.glyphs().len() {
                        let range = self.get_byte_range(self.caret_index);
                        self.text.replace_range(range, "");
                        self.update_text(this, ctx);
                    }
                }
                VirtualKeyCode::Left => {
                    if self.caret_index == 0 {
                        self.move_caret(0, ctx);
                    } else if ctx.modifiers().ctrl() {
                        let caret = self.get_word_start(self.caret_index - 1);
                        self.move_caret(caret, ctx);
                    } else {
                        self.move_caret(self.caret_index - 1, ctx);
                    }
                    self.update_carret(this, ctx, true);
                }
                VirtualKeyCode::Right => {
                    if self.caret_index + 1 >= self.text_layout.glyphs().len() {
                        self.move_caret(self.caret_index, ctx);
                    } else if ctx.modifiers().ctrl() {
                        let caret = self.get_next_word_start(self.caret_index);
                        self.move_caret(caret, ctx);
                    } else {
                        self.move_caret(self.caret_index + 1, ctx);
                    }
                    self.update_carret(this, ctx, true);
                }
                VirtualKeyCode::Home => {
                    self.move_caret(0, ctx);
                    self.update_carret(this, ctx, true);
                }
                VirtualKeyCode::End => {
                    self.move_caret(
                        self.get_line(self.caret_index).end_glyph.saturating_sub(1),
                        ctx,
                    );
                    self.update_carret(this, ctx, true);
                }
                _ if !ctx.modifiers().is_empty() => {
                    return false;
                }
                _ => {}
            },
            KeyboardEvent::Release(_) => {}
        }
        true
    }
}
