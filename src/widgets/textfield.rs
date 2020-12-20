use crate::{
    event, render::Graphic, text::TextInfo, style::OnFocusStyle, Behaviour, Context, Id,
    KeyboardEvent, MouseEvent, MouseButton
};
use copypasta::{ClipboardContext, ClipboardProvider};
use std::any::Any;
use winit::event::VirtualKeyCode;

pub struct TextField {
    caret: Id,
    label: Id,
    text: String,
    caret_index: usize,
    selection_index: Option<usize>,
    text_info: TextInfo,
    text_width: f32,
    x_scroll: f32,
    on_focus: bool,
    mouse_x: f32,
    mouse_down: bool,
    style: OnFocusStyle,
}
impl TextField {
    pub fn new(caret: Id, label: Id, style: OnFocusStyle) -> Self {
        Self {
            caret,
            label,
            text: String::new(),
            caret_index: 0,
            selection_index: None,
            text_info: TextInfo::default(),
            text_width: 0.0,
            x_scroll: 0.0,
            on_focus: false,
            mouse_x: 0.0,
            mouse_down: false,
            style,
        }
    }
}
impl TextField {
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
            ctx.get_graphic_mut(self.caret).set_color([51, 153, 255, 255]);
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
    }

    fn insert_char(&mut self, ch: char, this: Id, ctx: &mut Context) {
        self.text
            .insert(self.text_info.get_indice(self.caret_index), ch);
        self.caret_index += 1;
        self.update_text(this, ctx);
    }
}
impl Behaviour for TextField {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        self.update_text(this, ctx);
        ctx.move_to_front(self.label);
        ctx.set_graphic(this, self.style.normal.clone());
    }

    fn on_event(&mut self, event: &dyn Any, this: Id, ctx: &mut Context) {
        if event.is::<event::ClearText>() {
            self.text.clear();
            self.caret_index = 0;
            self.selection_index = None;
            self.update_text(this, ctx);
        }
    }

    fn on_scroll_event(&mut self, delta: [f32; 2], this: Id, ctx: &mut Context) -> bool {
        let delta = if delta[0].abs() > delta[1].abs() {
            delta[0]
        } else {
            delta[1]
        };
        self.x_scroll -= delta;
        self.update_carret(this, ctx, false);

        true
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        use MouseButton::*;
        match event {
            MouseEvent::Enter => {}
            MouseEvent::Exit => {}
            MouseEvent::Down(Left) => {
                let left = ctx.get_rect(this)[0] - self.x_scroll;
                let x = self.mouse_x - left;
                self.caret_index = self.text_info.get_caret_index_at_pos(0, x);
                self.mouse_down = true;
                self.selection_index = None;
                self.update_carret(this, ctx, true);
                ctx.send_event(event::LockOver);
            }
            MouseEvent::Up(Left) => {
                self.mouse_down = false;
                ctx.send_event(event::UnlockOver);
            }
            MouseEvent::Moved { x, .. } => {
                self.mouse_x = x;
                if self.mouse_down {
                    let left = ctx.get_rect(this)[0] - self.x_scroll;
                    let x = self.mouse_x - left;
                    let caret_index = self.text_info.get_caret_index_at_pos(0, x);
                    if caret_index == self.caret_index {
                        return true;
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
            }
            MouseEvent::Up(_) => {}
            MouseEvent::Down(_) => {}
        }
        true
    }

    fn on_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {
        self.on_focus = focus;
        if focus {
            ctx.set_graphic(this, self.style.focus.clone());
        } else {
            ctx.set_graphic(this, self.style.normal.clone());
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
                            } else {
                                self.text.insert_str(indice, &text);
                                self.update_text(this, ctx);
                                self.caret_index =
                                    self.text_info.get_caret_index(indice + text.len());
                            }
                            self.update_carret(this, ctx, true);
                        }
                    }
                }
                VirtualKeyCode::A => {
                    if ctx.modifiers().ctrl() {
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
                VirtualKeyCode::Return => {
                    ctx.send_event(event::SubmitText {
                        id: this,
                        text: self.text.clone(),
                    });
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
                        let mut caret = self.caret_index - 1;
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
                        let mut caret = self.caret_index;
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
