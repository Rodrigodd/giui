use std::{
    any::Any,
    rc::Rc,
    time::{Duration, Instant},
};

use copypasta::{ClipboardContext, ClipboardProvider};
use winit::{event::VirtualKeyCode, window::CursorIcon};

use crate::{
    event::SetValue,
    graphics::Graphic,
    style::TextFieldStyle,
    text::layout::TextLayout,
    text::{editor::TextEditor, Span},
    Behaviour, Context, Id, InputFlags, KeyboardEvent, MouseEvent, MouseInfo,
};

/// The callback that handle the events dispatched by the TextField.
pub trait TextFieldCallback {
    /// Called when the key Enter is pressed in the TextField. The text of the TextField can be
    /// change, while handling this event.
    fn on_submit(&mut self, this: Id, ctx: &mut Context, text: &mut String);
    /// Called ever time the text of the TextField changes.
    fn on_change(&mut self, this: Id, ctx: &mut Context, text: &str);
    /// Called when the TextField is unfocused. The text of the TextField can be
    /// change, while handling this event.
    fn on_unfocus(&mut self, this: Id, ctx: &mut Context, text: &mut String);
}
impl<F: FnMut(Id, &mut Context, &mut String) + 'static> TextFieldCallback for F {
    fn on_submit(&mut self, this: Id, ctx: &mut Context, text: &mut String) {
        self(this, ctx, text)
    }
    fn on_change(&mut self, _: Id, _: &mut Context, _: &str) {}
    fn on_unfocus(&mut self, _: Id, _: &mut Context, _: &mut String) {}
}
impl TextFieldCallback for () {
    fn on_submit(&mut self, _: Id, _: &mut Context, _: &mut String) {}
    fn on_change(&mut self, _: Id, _: &mut Context, _: &str) {}
    fn on_unfocus(&mut self, _: Id, _: &mut Context, _: &mut String) {}
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
            let text_layout = text.get_layout(fonts, rect).clone();
            let min_size = text_layout.min_size();
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
        let mut caret_pos = self.editor.get_caret_position_and_height(text_layout);

        const MARGIN: f32 = 5.0;

        let this_rect = ctx.get_rect(this);
        self.this_width = this_rect[2] - this_rect[0];
        let label_rect = ctx.get_rect(self.label);
        if let Graphic::Text(x) = ctx.get_graphic_mut(self.label) {
            let anchor = x.get_align_anchor(label_rect);
            caret_pos[0] += anchor[0] - label_rect[0];
            caret_pos[1] += anchor[1] - label_rect[1];
        } else {
        }

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

        let selection_range = self.editor.selection_range();
        if selection_range.len() > 0 {
            ctx.set_margins(self.caret, [0.0; 4]);
            if let Graphic::Text(text) = ctx.get_graphic_mut(self.label) {
                text.clear_selections();
                text.add_span(
                    selection_range,
                    Span::Selection {
                        bg: self.style.selection_color.bg,
                        fg: self.style.selection_color.fg,
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
}
impl<C: TextFieldCallback> Behaviour for TextField<C> {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        let fonts = ctx.get_fonts();
        if let Some((rect, Graphic::Text(text))) = ctx.get_rect_and_graphic(self.label) {
            text.set_wrap(false);
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
        if let Some(SetValue(text)) = event.downcast_ref::<SetValue<String>>() {
            let fonts = ctx.get_fonts();
            let text_layout = self.get_layout(ctx);
            self.editor.select_all(text_layout);
            self.editor.insert_text(&text, fonts, text_layout);
            self.update_text(this, ctx);
            self.callback.on_change(this, ctx, &text);
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
        use crate::MouseButton::*;
        let label_rect = ctx.get_rect(self.label);
        let anchor_x = if let Graphic::Text(x) = ctx.get_graphic_mut(self.label) {
            let anchor = x.get_align_anchor(label_rect);
            anchor[0]
        } else {
            panic!("TextField label graphic is not Text");
        };
        let text_layout = self.get_layout(ctx);
        match mouse.event {
            MouseEvent::Enter => {
                ctx.set_cursor(CursorIcon::Text);
            }
            MouseEvent::Exit => {
                ctx.set_cursor(CursorIcon::Default);
            }
            MouseEvent::Down(Left) => {
                let x = mouse.pos[0] - anchor_x;
                let byte_index = text_layout.byte_index_from_position(x, 0.0);
                if byte_index == self.drag_start {
                    match mouse.click_count {
                        0 => unreachable!(),
                        1 => {
                            self.editor
                                .move_cursor_to_byte_index(byte_index, false, text_layout);
                            self.mouse_down = 1;
                        }
                        2 => {
                            self.editor
                                .select_words_at_byte_range(byte_index..byte_index, text_layout);
                            self.mouse_down = 2;
                        }
                        3..=u8::MAX => {
                            self.editor.select_all(text_layout);
                            self.mouse_down = 3;
                        }
                    }
                } else {
                    self.editor
                        .move_cursor_to_byte_index(byte_index, false, text_layout);
                    self.mouse_down = 1;
                    if mouse.click_count > 1 {
                        ctx.reset_click_count_to_one();
                    }
                }
                self.drag_start = byte_index;
                if let Some(event_id) = self.blink_event.take() {
                    ctx.cancel_scheduled_event(event_id);
                }
                self.update_carret(this, ctx, true);
                ctx.lock_cursor(true);
            }
            MouseEvent::Up(Left) => {
                self.mouse_down = 0;
                ctx.lock_cursor(false);
            }
            MouseEvent::Moved => match self.mouse_down {
                0 => {}
                1 => {
                    let x = mouse.pos[0] - anchor_x;
                    let byte_index = text_layout.byte_index_from_position(x, 0.0);
                    self.editor
                        .move_cursor_to_byte_index(byte_index, true, text_layout);
                    self.update_carret(this, ctx, true);
                }
                2..=u8::MAX => {
                    let x = mouse.pos[0] - anchor_x;
                    let byte_index = text_layout.byte_index_from_position(x, 0.0);
                    self.editor
                        .select_words_at_byte_range(self.drag_start..byte_index, text_layout);
                    self.update_carret(this, ctx, true);
                }
            },
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

            let mut text = self.text(ctx).to_owned();
            self.callback.on_unfocus(this, ctx, &mut text);
            let fonts = ctx.get_fonts();
            let text_layout = self.get_layout(ctx);
            if text != text_layout.text() {
                self.editor.select_all(text_layout);
                self.editor.insert_text(&text, fonts, text_layout);
            }
            self.update_text(this, ctx);
        }
        self.update_carret(this, ctx, true);
    }

    fn on_keyboard_event(&mut self, event: KeyboardEvent, this: Id, ctx: &mut Context) -> bool {
        use crate::text::editor::HorizontalMotion::*;
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
                let text = self.text(ctx).to_owned();
                self.callback.on_change(this, ctx, &text);
            }
            KeyboardEvent::Pressed(key_code) => match key_code {
                VirtualKeyCode::Tab => return false, // allow change focus with tab
                VirtualKeyCode::C | VirtualKeyCode::X => {
                    if modifiers.ctrl() {
                        let range = self.editor.selection_range();
                        if !range.is_empty() {
                            let mut cliptobard = ClipboardContext::new().unwrap();
                            let _ = cliptobard.set_contents(text_layout.text()[range].to_owned());
                            if key_code == VirtualKeyCode::X {
                                self.editor.insert_text("", fonts, text_layout);
                            }
                        }
                    }
                }
                VirtualKeyCode::V => {
                    if modifiers.ctrl() {
                        let mut clipboard = ClipboardContext::new().unwrap();
                        if let Ok(text) = clipboard.get_contents() {
                            let text = text.replace(|x: char| x.is_control(), "");
                            self.editor.insert_text(&text, fonts, text_layout);
                            self.update_text(this, ctx);
                            let text = self.text(ctx).to_owned();
                            self.callback.on_change(this, ctx, &text);
                        }
                    }
                }
                VirtualKeyCode::A => {
                    if modifiers.ctrl() {
                        self.editor.select_all(text_layout);
                    }
                }
                VirtualKeyCode::Return => {
                    let mut text = text_layout.text().to_owned();
                    self.callback.on_submit(this, ctx, &mut text);
                    let text_layout = self.get_layout(ctx);
                    if text != text_layout.text() {
                        self.editor.select_all(text_layout);
                        self.editor.insert_text(&text, fonts, text_layout);
                    }
                }
                VirtualKeyCode::Back => {
                    if modifiers.ctrl() {
                        self.editor.delete_hor(Words(-1), fonts, text_layout);
                    } else {
                        self.editor.delete_hor(Cluster(-1), fonts, text_layout);
                    }
                    self.update_text(this, ctx);
                    let text = self.text(ctx).to_owned();
                    self.callback.on_change(this, ctx, &text);
                }
                VirtualKeyCode::Delete => {
                    if modifiers.ctrl() {
                        self.editor.delete_hor(Words(1), fonts, text_layout);
                    } else {
                        self.editor.delete_hor(Cluster(1), fonts, text_layout);
                    }
                    self.update_text(this, ctx);
                    let text = self.text(ctx).to_owned();
                    self.callback.on_change(this, ctx, &text);
                }
                VirtualKeyCode::Left => {
                    if modifiers.ctrl() {
                        self.editor
                            .move_cursor_hor(Words(-1), modifiers.shift(), text_layout);
                    } else {
                        self.editor
                            .move_cursor_hor(Cluster(-1), modifiers.shift(), text_layout);
                    }
                    self.update_carret(this, ctx, true);
                }
                VirtualKeyCode::Right => {
                    if modifiers.ctrl() {
                        self.editor
                            .move_cursor_hor(Words(1), modifiers.shift(), text_layout);
                    } else {
                        self.editor
                            .move_cursor_hor(Cluster(1), modifiers.shift(), text_layout);
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
