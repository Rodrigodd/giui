use crate::{Behaviour, Context, Id, MouseEvent};

pub struct Hoverable {
    is_over: bool,
    text: String,
    hover: Id,
    label: Id,
}
impl Hoverable {
    pub fn new(hover: Id, label: Id, text: String) -> Self {
        Self {
            is_over: false,
            text,
            hover,
            label,
        }
    }
}
impl Behaviour for Hoverable {
    fn on_start(&mut self, _this: Id, ctx: &mut Context) {
        ctx.deactive(self.hover);
    }

    fn on_mouse_event(&mut self, event: MouseEvent, _this: Id, ctx: &mut Context) -> bool {
        match event {
            MouseEvent::Enter => {
                ctx.active(self.hover);
                ctx.get_graphic_mut(self.label).set_text(&self.text);
                ctx.dirty_layout(self.label);
                ctx.move_to_front(self.hover);
                self.is_over = true;
            }
            MouseEvent::Exit => {
                ctx.deactive(self.hover);
                self.is_over = false;
            }
            MouseEvent::Down(_) => {}
            MouseEvent::Up(_) => {}
            MouseEvent::Moved { x, y } => {
                if self.is_over {
                    let [width, heigth] = ctx.get_size(crate::ROOT_ID);
                    let x = x / width;
                    let y = y / heigth;
                    ctx.set_anchors(self.hover, [x, y, x, y]);
                }
            }
        }
        true
    }
}
