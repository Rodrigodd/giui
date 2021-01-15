use crate::{
    event::SetValue,
    style::{ButtonStyle, OnFocusStyle},
    Behaviour, Context, Id, MouseButton, MouseEvent,
};

use std::{any::Any, rc::Rc};

pub struct Toggle<F: Fn(Id, &mut Context, bool)> {
    click: bool,
    enable: bool,
    button: Id,
    marker: Id,
    button_style: Rc<ButtonStyle>,
    background_style: Rc<OnFocusStyle>,
    on_change: F,
}
impl<F: Fn(Id, &mut Context, bool)> Toggle<F> {
    pub fn new(
        button: Id,
        marker: Id,
        initial_value: bool,
        button_style: Rc<ButtonStyle>,
        background_style: Rc<OnFocusStyle>,
        on_change: F,
    ) -> Self {
        Self {
            click: false,
            enable: initial_value,
            button,
            marker,
            button_style,
            background_style,
            on_change,
        }
    }
}
impl<F: Fn(Id, &mut Context, bool)> Behaviour for Toggle<F> {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        (self.on_change)(this, ctx, self.enable);
        ctx.set_graphic(this, self.background_style.normal.clone());
        ctx.set_graphic(self.button, self.button_style.normal.clone());
        if self.enable {
            ctx.get_graphic_mut(self.marker).set_alpha(255)
        } else {
            ctx.get_graphic_mut(self.marker).set_alpha(0)
        }
    }

    fn on_event(&mut self, event: &dyn Any, this: Id, ctx: &mut Context) {
        if let Some(SetValue(x)) = event.downcast_ref() {
            self.enable = *x;
            (self.on_change)(this, ctx, self.enable);
            if self.enable {
                ctx.get_graphic_mut(self.marker).set_alpha(255)
            } else {
                ctx.get_graphic_mut(self.marker).set_alpha(0)
            }
        }
    }

    fn on_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {
        if focus {
            ctx.set_graphic(this, self.background_style.focus.clone());
        } else {
            ctx.set_graphic(this, self.background_style.normal.clone());
        }
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        use MouseButton::*;
        match event {
            MouseEvent::Enter => {
                let graphic = ctx.get_graphic_mut(self.button);
                graphic.set_color([190, 190, 190, 255]);
            }
            MouseEvent::Exit => {
                self.click = false;
                let graphic = ctx.get_graphic_mut(self.button);
                graphic.set_color([200, 200, 200, 255]);
            }
            MouseEvent::Down(Left) => {
                self.click = true;
                let graphic = ctx.get_graphic_mut(self.button);
                graphic.set_color([170, 170, 170, 255]);
            }
            MouseEvent::Up(Left) => {
                let graphic = ctx.get_graphic_mut(self.button);
                graphic.set_color([190, 190, 190, 255]);
                if self.click {
                    self.enable = !self.enable;
                    (self.on_change)(this, ctx, self.enable);
                    if self.enable {
                        ctx.get_graphic_mut(self.marker).set_alpha(255)
                    } else {
                        ctx.get_graphic_mut(self.marker).set_alpha(0)
                    }
                }
            }
            MouseEvent::Moved { .. } => {}
            MouseEvent::Up(_) => {}
            MouseEvent::Down(_) => {}
        }
        true
    }
}
