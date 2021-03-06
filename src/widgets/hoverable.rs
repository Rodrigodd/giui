use crate::{Behaviour, Context, Id, InputFlags, MouseEvent, MouseInfo};

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

    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE
    }

    fn on_mouse_event(&mut self, mouse: MouseInfo, _this: Id, ctx: &mut Context) {
        match mouse.event {
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
            MouseEvent::Moved => {
                let [x, y] = {
                    let root = ctx.get_rect(Id::ROOT_ID);
                    [mouse.pos[0] - root[0], mouse.pos[1] - root[1]]
                };
                if self.is_over {
                    let [width, heigth] = ctx.get_size(crate::Id::ROOT_ID);
                    let x = x / width;
                    let y = y / heigth;
                    ctx.set_anchors(self.hover, [x, y, x, y]);
                }
            }
            MouseEvent::None => {}
        }
    }
}
