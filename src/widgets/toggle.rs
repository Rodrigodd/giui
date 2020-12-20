use crate::{
    event,
    style::{ButtonStyle, OnFocusStyle},
    Behaviour, Context, Id, MouseEvent, MouseButton
};

pub struct Toggle {
    click: bool,
    enable: bool,
    button: Id,
    marker: Id,
    button_style: ButtonStyle,
    background_style: OnFocusStyle,
}
impl Toggle {
    pub fn new(
        button: Id,
        marker: Id,
        button_style: ButtonStyle,
        background_style: OnFocusStyle,
    ) -> Self {
        Self {
            click: false,
            enable: false,
            button,
            marker,
            button_style,
            background_style,
        }
    }
}
impl Behaviour for Toggle {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        ctx.send_event(event::ToggleChanged {
            id: this,
            value: self.enable,
        });
        ctx.set_graphic(this, self.background_style.normal.clone());
        ctx.set_graphic(self.button, self.button_style.normal.clone());
        if self.enable {
            ctx.get_graphic_mut(self.marker).set_alpha(255)
        } else {
            ctx.get_graphic_mut(self.marker).set_alpha(0)
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
                    ctx.send_event(event::ToggleChanged {
                        id: this,
                        value: self.enable,
                    });
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
