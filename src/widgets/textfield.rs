use crate::{
    event::{self, SetValue},
    graphics::Graphic,
    style::OnFocusStyle,
    text::TextInfo,
    Behaviour, Context, Id, InputFlags, KeyboardEvent, MouseButton, MouseEvent, MouseInfo,
};

use copypasta::{ClipboardContext, ClipboardProvider};
use std::any::Any;
use std::rc::Rc;
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

pub struct TextField<C: TextFieldCallback> {
    callback: C,
    caret: Id,
    label: Id,
    text: String,
    previous_text: String,
    caret_index: usize,
    selection_index: Option<usize>,
    text_info: TextInfo,
    text_width: f32,
    x_scroll: f32,
    on_focus: bool,
    mouse_x: f32,
    /// If it is non zero, the mouse is being dragged. 1 for single click, 2 for double click, etc...
    mouse_down: u8,
    drag_start: usize,
    style: Rc<OnFocusStyle>,
}
impl<C: TextFieldCallback> TextField<C> {
    pub fn new(text: String, caret: Id, label: Id, style: Rc<OnFocusStyle>, callback: C) -> Self {
        Self {
            callback,
            caret,
            label,
            previous_text: text.clone(),
            text,
            caret_index: 0,
            selection_index: None,
            text_info: TextInfo::default(),
            text_width: 0.0,
            x_scroll: 0.0,
            on_focus: false,
            mouse_x: 0.0,
            mouse_down: 0,
            drag_start: 0,
            style,
        }
    }

    fn update_text(&mut self, this: Id, ctx: &mut Context) {
        let fonts = ctx.get_fonts();
        if let Some((ref mut rect, Graphic::Text(text))) = ctx.get_rect_and_graphic(self.label) {
            let display_text = self.text.clone();
            text.set_text(&display_text);
            let min_size = text.compute_min_size(fonts).unwrap_or([0.0, 0.0]);
            self.text_width = min_size[0];
            rect.set_min_size(min_size);
            self.text_info = text.get_text_info(fonts, rect).clone();
            self.update_carret(this, ctx, true);
        }
    }

    fn update_carret(&mut self, this: Id, ctx: &mut Context, focus_caret: bool) {
        let this_rect = *ctx.get_rect(this);

        let mut caret_pos = self.text_info.get_caret_pos(self.caret_index);

        const MARGIN: f32 = 5.0;

        let this_width = this_rect[2] - this_rect[0];
        if this_width > self.text_width {
            self.x_scroll = -MARGIN;
        } else if focus_caret {
            if caret_pos[0] - self.x_scroll > this_width - MARGIN {
                self.x_scroll = caret_pos[0] - (this_width - MARGIN);
            }
            if caret_pos[0] - self.x_scroll < MARGIN {
                self.x_scroll = caret_pos[0] - MARGIN;
            }
        } else {
            if self.text_width - self.x_scroll < this_width - MARGIN {
                self.x_scroll = self.text_width - (this_width - MARGIN);
            }
            if self.x_scroll < -MARGIN {
                self.x_scroll = -MARGIN;
            }
        }

        ctx.set_margin_left(self.label, -self.x_scroll);

        caret_pos[0] -= self.x_scroll;

        if let Some(selection_index) = self.selection_index {
            ctx.get_graphic_mut(self.caret)
                .set_color([51, 153, 255, 255]);
            let mut selection_pos = self.text_info.get_caret_pos(selection_index);
            selection_pos[0] -= self.x_scroll;
            let mut margins = [
                caret_pos[0],
                caret_pos[1] - self.text_info.get_line_heigth(),
                selection_pos[0],
                caret_pos[1],
            ];
            if margins[0] > margins[2] {
                margins.swap(0, 2);
            }
            if margins[1] > margins[3] {
                margins.swap(1, 3);
            }
            ctx.set_margins(self.caret, margins);
        } else {
            ctx.get_graphic_mut(self.caret).set_color([0, 0, 0, 255]);
            if self.on_focus {
                ctx.set_margins(
                    self.caret,
                    [
                        caret_pos[0],
                        caret_pos[1] - self.text_info.get_line_heigth(),
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

    fn delete_selection(&mut self, this: Id, ctx: &mut Context) {
        let selection_index = self.selection_index.unwrap();
        let a = self.text_info.get_indice(self.caret_index);
        let b = self.text_info.get_indice(selection_index);
        let range = if a > b { b..a } else { a..b };
        if self.caret_index > selection_index {
            self.caret_index = selection_index;
        }
        self.selection_index = None;
        self.text.replace_range(range, "");
        self.update_text(this, ctx);
        self.callback.on_change(this, ctx, &self.text)
    }

    fn insert_char(&mut self, ch: char, this: Id, ctx: &mut Context) {
        self.text
            .insert(self.text_info.get_indice(self.caret_index), ch);
        self.caret_index += 1;
        self.update_text(this, ctx);
        self.callback.on_change(this, ctx, &self.text)
    }

    fn get_word_start(&mut self, mut caret: usize) -> usize {
        let mut s = false;
        while caret != 0 {
            let whitespace = match self.text[self.text_info.get_indice(caret)..]
                .chars()
                .next()
            {
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
            let whitespace = match self.text[self.text_info.get_indice(caret)..]
                .chars()
                .next()
            {
                Some(x) => x.is_whitespace(),
                None => {
                    caret = self.text_info.len() - 1;
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
        let start = match self.text[self.text_info.get_indice(caret)..]
            .chars()
            .next()
        {
            Some(x) => x.is_whitespace(),
            None => false,
        };
        loop {
            let whitespace = match self.text[self.text_info.get_indice(caret)..]
                .chars()
                .next()
            {
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
        let start = match self.text[self.text_info.get_indice(caret)..]
            .chars()
            .next()
        {
            Some(x) => x.is_whitespace(),
            None => {
                return self.text_info.len() - 1;
            }
        };
        loop {
            let whitespace = match self.text[self.text_info.get_indice(caret)..]
                .chars()
                .next()
            {
                Some(x) => x.is_whitespace(),
                None => {
                    caret = self.text_info.len() - 1;
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
        let end = self
            .text_info
            .get_line_range(self.caret_index)
            .map_or(0, |x| x.end.saturating_sub(1));
        self.selection_index = Some(start);
        self.caret_index = end;
        self.update_carret(this, ctx, false);
    }
}
impl<C: TextFieldCallback> Behaviour for TextField<C> {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        self.update_text(this, ctx);
        ctx.move_to_front(self.label);
        ctx.set_graphic(this, self.style.normal.clone());
    }

    fn on_event(&mut self, event: Box<dyn Any>, this: Id, ctx: &mut Context) {
        if let Some(SetValue(text)) = event.downcast_ref::<SetValue<String>>() {
            let x = self.text_info.get_caret_pos(self.caret_index)[0];
            self.text.clone_from(text);
            self.previous_text.clone_from(text);
            self.update_text(this, ctx);
            self.selection_index = None;
            self.caret_index = self.text_info.get_caret_index_at_pos(0, x);
            self.update_carret(this, ctx, true);
            self.callback.on_change(this, ctx, &self.text);
        }
    }

    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE | InputFlags::SCROLL | InputFlags::FOCUS
    }

    fn on_scroll_event(&mut self, delta: [f32; 2], this: Id, ctx: &mut Context) {
        let delta = if delta[0].abs() > delta[1].abs() {
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
                let left = ctx.get_rect(this)[0] - self.x_scroll;
                let x = self.mouse_x - left;
                let caret = self.text_info.get_caret_index_at_pos(0, x);
                if caret == self.drag_start {
                    match mouse.click_count {
                        0 => unreachable!(),
                        1 => {
                            self.caret_index = caret;
                            self.mouse_down = 1;
                            self.selection_index = None;
                        },
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
                    0 => {},
                    1 => {
                        let left = ctx.get_rect(this)[0] - self.x_scroll;
                        let x = self.mouse_x - left;
                        let caret_index = self.text_info.get_caret_index_at_pos(0, x);
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
                    },
                    2..=u8::MAX => {
                        let left = ctx.get_rect(this)[0] - self.x_scroll;
                        let x = self.mouse_x - left;
                        let caret_index = self.text_info.get_caret_index_at_pos(0, x);
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
            ctx.set_graphic(this, self.style.focus.clone());
        } else {
            ctx.set_graphic(this, self.style.normal.clone());

            let x = self.text_info.get_caret_pos(self.caret_index)[0];
            if self.callback.on_unfocus(this, ctx, &mut self.text) {
                self.previous_text.clone_from(&self.text);
            } else {
                self.text.clone_from(&self.previous_text);
            }
            self.selection_index = None;
            self.caret_index = self.text_info.get_caret_index_at_pos(0, x);
            self.update_text(this, ctx);
        }
        self.update_carret(this, ctx, true);
    }

    fn on_keyboard_event(&mut self, event: KeyboardEvent, this: Id, ctx: &mut Context) -> bool {
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
                            let a = self.text_info.get_indice(selection_index);
                            let b = self.text_info.get_indice(self.caret_index);
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
                            let indice = self.text_info.get_indice(self.caret_index);
                            if let Some(selection_index) = self.selection_index {
                                let a = self.text_info.get_indice(selection_index);
                                let b = indice;
                                let range = if a < b { a..b } else { b..a };
                                self.text.replace_range(range.clone(), &text);
                                self.selection_index = None;
                                self.update_text(this, ctx); // TODO: is is not working?
                                self.caret_index =
                                    self.text_info.get_caret_index(range.start + text.len());
                                self.callback.on_change(this, ctx, &self.text)
                            } else {
                                self.text.insert_str(indice, &text);
                                self.update_text(this, ctx);
                                self.caret_index =
                                    self.text_info.get_caret_index(indice + text.len());
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
                    let x = self.text_info.get_caret_pos(self.caret_index)[0];
                    if self.callback.on_submit(this, ctx, &mut self.text) {
                        self.previous_text.clone_from(&self.text);
                    } else {
                        self.text.clone_from(&self.previous_text);
                    }
                    self.update_text(this, ctx);
                    self.selection_index = None;
                    self.caret_index = self.text_info.get_caret_index_at_pos(0, x);
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
                    self.text
                        .remove(self.text_info.get_indice(self.caret_index));
                    self.update_text(this, ctx);
                    self.callback.on_change(this, ctx, &self.text);
                }
                VirtualKeyCode::Delete => {
                    if self.caret_index + 1 < self.text_info.len() {
                        self.text
                            .remove(self.text_info.get_indice(self.caret_index));
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
                    if self.caret_index + 1 >= self.text_info.len() {
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
                        self.text_info
                            .get_line_range(self.caret_index)
                            .map_or(0, |x| x.end.saturating_sub(1)),
                        ctx,
                    );
                    self.update_carret(this, ctx, true);
                }
                _ => {}
            },
        }
        true
    }
}
