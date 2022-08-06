use std::rc::Rc;

use crate::{
    style::ButtonStyle, Behaviour, Context, Id, InputFlags, MouseButton, MouseEvent, MouseInfo,
};

pub struct Button<F: FnMut(Id, &mut Context)> {
    normal: bool,
    focusable: bool,
    focus: bool,
    on_click: F,
    style: Rc<ButtonStyle>,
}
impl<F: FnMut(Id, &mut Context)> Button<F> {
    pub fn new(style: Rc<ButtonStyle>, focusable: bool, on_click: F) -> Self {
        Self {
            normal: true,
            focus: false,
            focusable,
            on_click,
            style,
        }
    }
}
impl<F: FnMut(Id, &mut Context)> Behaviour for Button<F> {
    fn on_active(&mut self, this: Id, ctx: &mut Context) {
        ctx.set_graphic(this, self.style.normal.clone());
    }

    fn input_flags(&self) -> InputFlags {
        let mut flags = InputFlags::MOUSE;
        if self.focusable {
            flags |= InputFlags::FOCUS
        }
        flags
    }

    fn on_mouse_event(&mut self, mouse: MouseInfo, this: Id, ctx: &mut Context) {
        use MouseButton::*;
        if mouse.click() {
            (self.on_click)(this, ctx);
        }
        match mouse.event {
            MouseEvent::Enter => {
                self.normal = false;
                ctx.set_graphic(this, self.style.hover.clone());
            }
            MouseEvent::Exit => {
                self.normal = true;
                if self.focus {
                    ctx.set_graphic(this, self.style.focus.clone());
                } else {
                    ctx.set_graphic(this, self.style.normal.clone());
                }
            }
            MouseEvent::Down(Left) => {
                ctx.set_graphic(this, self.style.pressed.clone());
            }
            MouseEvent::Up(Left) => {
                ctx.set_graphic(this, self.style.hover.clone());
            }
            _ => {}
        }
    }

    fn on_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {
        self.focus = focus;
        if self.normal {
            if focus {
                ctx.set_graphic(this, self.style.focus.clone());
            } else {
                ctx.set_graphic(this, self.style.normal.clone());
            }
        }
    }
}
