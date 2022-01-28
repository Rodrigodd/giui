use std::ops::Range;

use crate::{graphics::Graphic, Behaviour, Context, Id, InputFlags, MouseEvent, MouseInfo};

pub trait InteractiveTextCallback {
    /// Similar to [`Behaviour::on_mouse_event`], but limited to the bounds of the text span.
    fn on_mouse_event(&mut self, mouse: MouseInfo, this: Id, ctx: &mut Context);
}
impl<T: FnMut(MouseInfo, Id, &mut Context)> InteractiveTextCallback for T {
    fn on_mouse_event(&mut self, mouse: MouseInfo, this: Id, ctx: &mut Context) {
        (self)(mouse, this, ctx)
    }
}

pub struct InteractiveText {
    actions: Vec<(Range<usize>, Box<dyn InteractiveTextCallback>, bool)>,
}
impl InteractiveText {
    pub fn new(actions: Vec<(Range<usize>, Box<dyn InteractiveTextCallback>)>) -> Self {
        Self {
            actions: actions
                .into_iter()
                .map(|(range, cb)| (range, cb, false))
                .collect(),
        }
    }
}
impl Behaviour for InteractiveText {
    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE
    }

    fn on_mouse_event(&mut self, mouse: MouseInfo, this: Id, ctx: &mut Context) {
        let fonts = ctx.get_fonts();
        let (rect, text) = match ctx.get_rect_and_graphic(this) {
            Some((a, Graphic::Text(b))) => (a, b),
            _ => return,
        };

        // TODO: the rules for mouse.click_count are not being respected when calling
        // action.on_mouse_event.
        match mouse.event {
            MouseEvent::Enter => {}
            MouseEvent::Exit => {
                for (_, action, is_on) in self.actions.iter_mut() {
                    if *is_on {
                        *is_on = false;
                        action.on_mouse_event(mouse.clone(), this, ctx);
                    }
                }
            }
            MouseEvent::Down(_) | MouseEvent::Up(_) => {
                for (_, action, is_on) in self.actions.iter_mut() {
                    if *is_on {
                        action.on_mouse_event(mouse.clone(), this, ctx);
                    }
                }
            }
            MouseEvent::Moved => {
                let anchor = text.get_align_anchor(rect.rect);
                let text_layout = text.get_layout(fonts, rect);
                let x = mouse.pos[0] - anchor[0];
                let y = mouse.pos[1] - anchor[1];
                let byte_index = text_layout.byte_index_from_position(x, y);
                let contains = |r: &Range<usize>| byte_index.map_or(false, |x| r.contains(&x));
                for (r, action, was_on) in self.actions.iter_mut() {
                    let is_on = contains(r);
                    if !*was_on && is_on {
                        let mut mouse = mouse.clone();
                        mouse.event = MouseEvent::Enter;
                        action.on_mouse_event(mouse, this, ctx);
                    } else if *was_on && !is_on {
                        let mut mouse = mouse.clone();
                        mouse.event = MouseEvent::Exit;
                        action.on_mouse_event(mouse, this, ctx);
                    }
                    if is_on {
                        let mut mouse = mouse.clone();
                        mouse.event = MouseEvent::Moved;
                        action.on_mouse_event(mouse, this, ctx);
                    }
                    *was_on = is_on;
                }
            }
            MouseEvent::None => {}
        }
    }
}
