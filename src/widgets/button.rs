use crate::{style::ButtonStyle, Behaviour, Context, Id, MouseButton, MouseEvent};

use std::rc::Rc;

pub struct Button<F: Fn(Id, &mut Context)> {
    state: u8, // 0 - normal, 1 - hover, 2 - pressed
    focus: bool,
    on_click: F,
    style: Rc<ButtonStyle>,
}
impl<F: Fn(Id, &mut Context)> Button<F> {
    pub fn new(style: Rc<ButtonStyle>, on_click: F) -> Self {
        Self {
            state: 0,
            focus: false,
            on_click,
            style,
        }
    }
}
impl<F: Fn(Id, &mut Context)> Behaviour for Button<F> {
    fn on_active(&mut self, this: Id, ctx: &mut Context) {
        ctx.set_graphic(this, self.style.normal.clone());
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        use MouseButton::*;
        match event {
            MouseEvent::Enter => {
                self.state = 1;
                ctx.set_graphic(this, self.style.hover.clone());
            }
            MouseEvent::Exit => {
                self.state = 0;
                if self.focus {
                    ctx.set_graphic(this, self.style.focus.clone());
                } else {
                    ctx.set_graphic(this, self.style.normal.clone());
                }
            }
            MouseEvent::Down(Left) => {
                self.state = 2;
                ctx.set_graphic(this, self.style.pressed.clone());
            }
            MouseEvent::Up(Left) => {
                if self.state == 2 {
                    (self.on_click)(this, ctx);
                }
                self.state = 1;
                ctx.set_graphic(this, self.style.hover.clone());
            }
            _ => {}
        }
        true
    }

    fn on_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {
        self.focus = focus;
        if self.state == 0 {
            if focus {
                ctx.set_graphic(this, self.style.focus.clone());
            } else {
                ctx.set_graphic(this, self.style.normal.clone());
            }
        }
    }
}
