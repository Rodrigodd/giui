use std::{
    any::Any,
    rc::Rc,
    time::{Duration, Instant},
};

use copypasta::{ClipboardContext, ClipboardProvider};
use winit::event::VirtualKeyCode;

use crate::{
    graphics::Graphic,
    style::TextFieldStyle,
    text::layout::TextLayout,
    text::{editor::TextEditor, Span},
    Behaviour, Context, Id, InputFlags, KeyboardEvent, MouseInfo,
};

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
    editor: TextEditor,
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
    pub fn new(caret: Id, label: Id, style: Rc<TextFieldStyle>, callback: C) -> Self {
        Self {
            callback,
            caret,
            label,
            editor: TextEditor::new(),
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
            // let display_text = self.text.clone() + " ";
            // text.set_text(&display_text);
            // let min_size = text.compute_min_size(fonts).unwrap_or([0.0, 0.0]);
            // self.text_width = min_size[0];
            // rect.set_min_size(min_size);
            let text_layout = text.get_layout(fonts, rect).clone();
            // let glyphs = self.text_layout.glyphs();
            // if self.caret_index + 1 >= glyphs.len() {
            //     self.caret_index = glyphs.len().saturating_sub(1);
            // }
            // let text_layout = self.editor.text_layout();
            let min_size = text_layout.min_size();
            // text.set_text_layout(text_layout);
            self.text_width = min_size[0];
            rect.set_min_size(min_size);
            self.update_carret(this, ctx, true);
        }
    }

    fn get_layout<'a>(&mut self, ctx: &'a mut Context) -> &'a mut TextLayout {
        let fonts = ctx.get_fonts();
        if let Some((rect, Graphic::Text(text))) = ctx.get_rect_and_graphic(self.label) {
            text.get_layout(fonts, rect)
        } else {
            panic!("TextField label graphic is not Text");
        }
    }

    fn text<'ctx>(&mut self, ctx: &'ctx mut Context) -> &'ctx str {
        self.get_layout(ctx).text()
    }

    fn update_carret(&mut self, this: Id, ctx: &mut Context, focus_caret: bool) {
        if let Some(event_id) = self.blink_event {
            ctx.cancel_scheduled_event(event_id);
        } else {
            self.blink = false;
        }

        let text_layout = self.get_layout(ctx);
        let mut caret_pos = self.editor.get_cursor_position_and_height(text_layout);

        const MARGIN: f32 = 5.0;

        let this_rect = *ctx.get_rect(this);
        self.this_width = this_rect[2] - this_rect[0];

        // caret_pos[0] += label_rect[0];
        // caret_pos[1] += label_rect[1] - this_rect[1];

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

        let text_layout = self.get_layout(ctx);
        let selection_range = self.editor.selection_range(text_layout);
        if selection_range.len() > 0 {
            ctx.set_margins(self.caret, [0.0; 4]);
            if let Graphic::Text(text) = ctx.get_graphic_mut(self.label) {
                text.clear_spans();
                text.add_span(
                    selection_range,
                    // Span {
                    //     color: self.style.selection_color.fg,
                    //     background: Some(self.style.selection_color.bg),
                    //     ..Default::default()
                    // },
                    Span::Selection(self.style.selection_color.bg),
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
                        caret_pos[0] + 3.0,
                        caret_pos[1],
                    ],
                );
            } else {
                ctx.set_margins(self.caret, [0.0, 0.0, 0.0, 0.0]);
            }
        }
    }
}
impl<C: TextFieldCallback> Behaviour for TextField<C> {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        let fonts = ctx.get_fonts();
        if let Some((rect, Graphic::Text(text))) = ctx.get_rect_and_graphic(self.label) {
            let min_size = text.compute_min_size(fonts).unwrap_or([0.0, 0.0]);
            self.text_width = min_size[0];
            rect.set_min_size(min_size);
            self.update_text(this, ctx);
            ctx.move_to_front(self.label);
            ctx.set_graphic(this, self.style.background.normal.clone());
        } else {
            panic!("TextField label graphic is not Text");
        }
    }

    fn on_event(&mut self, event: Box<dyn Any>, this: Id, ctx: &mut Context) {
        // if let Some(SetValue(text)) = event.downcast_ref::<SetValue<String>>() {
        //     let x = self.get_glyph_pos(self.caret_index)[0];
        //     self.text.clone_from(text);
        //     self.previous_text.clone_from(text);
        //     self.update_text(this, ctx);
        //     self.selection_index = None;
        //     self.caret_index = self.get_caret_index_at_pos(0, x);
        //     self.update_carret(this, ctx, true);
        //     self.callback.on_change(this, ctx, &self.text);
        // } else if event.is::<BlinkCaret>() {
        //     self.blink = !self.blink;
        //     self.update_carret(this, ctx, false);
        // }
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
        // use MouseButton::*;
        // match mouse.event {
        //     MouseEvent::Enter => {
        //         ctx.set_cursor(CursorIcon::Text);
        //     }
        //     MouseEvent::Exit => {
        //         ctx.set_cursor(CursorIcon::Default);
        //     }
        //     MouseEvent::Down(Left) => {
        //         if let Some(event_id) = self.blink_event.take() {
        //             ctx.cancel_scheduled_event(event_id);
        //         }
        //         if self.blink {
        //             self.update_carret(this, ctx, false);
        //         }

        //         let left = ctx.get_rect(this)[0] - self.x_scroll;
        //         let x = self.mouse_x - left;
        //         let caret = self.get_caret_index_at_pos(0, x);
        //         if caret == self.drag_start {
        //             match mouse.click_count {
        //                 0 => unreachable!(),
        //                 1 => {
        //                     self.caret_index = caret;
        //                     self.mouse_down = 1;
        //                     self.selection_index = None;
        //                 }
        //                 2 => {
        //                     let caret = self.caret_index;
        //                     self.caret_index = self.get_token_start(caret);
        //                     self.mouse_down = 2;
        //                     self.selection_index = Some(self.get_token_end(caret));
        //                 }
        //                 3..=u8::MAX => {
        //                     self.select_all(this, ctx);
        //                 }
        //             }
        //         } else {
        //             if mouse.click_count > 1 {
        //                 ctx.reset_click_count_to_one();
        //             }
        //             self.caret_index = caret;
        //             self.mouse_down = 1;
        //             self.selection_index = None;
        //         }
        //         self.drag_start = caret;
        //         self.update_carret(this, ctx, true);
        //         ctx.send_event(event::LockOver);
        //     }
        //     MouseEvent::Up(Left) => {
        //         self.mouse_down = 0;
        //         ctx.send_event(event::UnlockOver);
        //     }
        //     MouseEvent::Moved => {
        //         let [x, _] = mouse.pos;
        //         self.mouse_x = x;
        //         match self.mouse_down {
        //             0 => {}
        //             1 => {
        //                 let left = ctx.get_rect(this)[0] - self.x_scroll;
        //                 let x = self.mouse_x - left;
        //                 let caret_index = self.get_caret_index_at_pos(0, x);
        //                 if caret_index == self.caret_index {
        //                     return;
        //                 }
        //                 if let Some(selection_index) = self.selection_index {
        //                     if caret_index == selection_index {
        //                         self.selection_index = None;
        //                     }
        //                 } else {
        //                     self.selection_index = Some(self.caret_index);
        //                 }
        //                 self.caret_index = caret_index;
        //                 self.update_carret(this, ctx, true);
        //             }
        //             2..=u8::MAX => {
        //                 let left = ctx.get_rect(this)[0] - self.x_scroll;
        //                 let x = self.mouse_x - left;
        //                 let caret_index = self.get_caret_index_at_pos(0, x);
        //                 let selection_index = self.selection_index.unwrap_or(caret_index);
        //                 let caret_index = if selection_index > caret_index {
        //                     self.selection_index = Some(self.get_token_end(self.drag_start));
        //                     self.get_token_start(caret_index)
        //                 } else {
        //                     self.selection_index = Some(self.get_token_start(self.drag_start));
        //                     self.get_token_end(caret_index)
        //                 };
        //                 self.caret_index = caret_index;
        //                 self.update_carret(this, ctx, true);
        //             }
        //         }
        //     }
        //     MouseEvent::Up(_) => {}
        //     MouseEvent::Down(_) => {}
        //     MouseEvent::None => {}
        // }
    }

    fn on_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {
        self.on_focus = focus;
        if focus {
            ctx.set_graphic(this, self.style.background.focus.clone());
        } else {
            ctx.set_graphic(this, self.style.background.normal.clone());

            let mut text = self.text(ctx).to_owned();
            self.callback.on_unfocus(this, ctx, &mut text);
            self.update_text(this, ctx);
        }
        self.update_carret(this, ctx, true);
    }

    fn on_keyboard_event(&mut self, event: KeyboardEvent, this: Id, ctx: &mut Context) -> bool {
        if let Some(event_id) = self.blink_event.take() {
            ctx.cancel_scheduled_event(event_id);
        }
        self.update_carret(this, ctx, false);
        let fonts = ctx.get_fonts();
        let modifiers = ctx.modifiers();
        let text_layout = self.get_layout(ctx);
        match event {
            KeyboardEvent::Char(ch) => {
                println!("insert {}", ch);
                self.editor
                    .insert_text(ch.encode_utf8(&mut [0; 4]), fonts, text_layout);
                println!("text: {}", self.text(ctx));
                self.update_text(this, ctx);
            }
            KeyboardEvent::Pressed(key_code) => match key_code {
                VirtualKeyCode::Tab => return false, // allow change focus with tab
                // VirtualKeyCode::C | VirtualKeyCode::X => {
                //     if modifiers.ctrl() {
                //         if let Some(selection_index) = self.selection_index {
                //             let a = self.get_byte_range(selection_index).start;
                //             let b = self.get_byte_range(self.caret_index).start;
                //             let range = if a < b { a..b } else { b..a };
                //             let mut cliptobard = ClipboardContext::new().unwrap();
                //             let _ = cliptobard.set_contents(self.text[range].to_owned());
                //             if key_code == VirtualKeyCode::X {
                //                 self.delete_selection(this, ctx);
                //             }
                //         }
                //     }
                // }
                VirtualKeyCode::V => {
                    if modifiers.ctrl() {
                        let mut clipboard = ClipboardContext::new().unwrap();
                        if let Ok(text) = clipboard.get_contents() {
                            let text = text.replace(|x: char| x.is_control(), "");
                            self.editor.insert_text(&text, fonts, text_layout);
                            self.update_carret(this, ctx, true);
                        }
                    }
                }
                // VirtualKeyCode::A => {
                //     if modifiers.ctrl() {
                //         self.select_all(this, ctx);
                //     }
                // }
                VirtualKeyCode::Return => {
                    let mut text = self.text(ctx).to_owned();
                    self.callback.on_submit(this, ctx, &mut text);
                }
                VirtualKeyCode::Back => {
                    self.editor.delete_hor(-1, fonts, text_layout);
                    self.update_text(this, ctx);
                    let text = self.text(ctx).to_owned();
                    self.callback.on_change(this, ctx, &text);
                }
                VirtualKeyCode::Delete => {
                    self.editor.delete_hor(1, fonts, text_layout);
                    self.update_text(this, ctx);
                }
                VirtualKeyCode::Left => {
                    if modifiers.ctrl() {
                        self.editor
                            .move_cursor_hor(-2, modifiers.shift(), text_layout);
                    } else {
                        self.editor
                            .move_cursor_hor(-1, modifiers.shift(), text_layout);
                    }
                    self.update_carret(this, ctx, true);
                }
                VirtualKeyCode::Right => {
                    if modifiers.ctrl() {
                        self.editor
                            .move_cursor_hor(2, modifiers.shift(), text_layout);
                    } else {
                        self.editor
                            .move_cursor_hor(1, modifiers.shift(), text_layout);
                    }
                    self.update_carret(this, ctx, true);
                }
                VirtualKeyCode::Home => {
                    self.editor
                        .move_cursor_line_start(modifiers.shift(), text_layout);
                    self.update_carret(this, ctx, true);
                }
                VirtualKeyCode::End => {
                    self.editor
                        .move_cursor_line_end(modifiers.shift(), text_layout);
                    self.update_carret(this, ctx, true);
                }
                _ if !modifiers.is_empty() => {
                    return false;
                }
                _ => {}
            },
            KeyboardEvent::Release(_) => {}
        }
        true
    }
}
