use crate::{event, widgets::OnFocusStyle, Behaviour, Context, Id, MouseEvent};

pub struct Slider {
    handle: Id,
    slide_area: Id, //TODO: I should remove this slide_area
    dragging: bool,
    mouse_x: f32,
    min_value: f32,
    max_value: f32,
    value: f32,
    style: OnFocusStyle,
}
impl Slider {
    pub fn new(
        handle: Id,
        slide_area: Id,
        min_value: f32,
        max_value: f32,
        start_value: f32,
        style: OnFocusStyle,
    ) -> Self {
        Self {
            handle,
            slide_area,
            dragging: false,
            mouse_x: 0.0,
            max_value,
            min_value,
            value: start_value,
            style,
        }
    }

    fn update_value(&mut self, ctx: &mut Context) {
        let area_rect = ctx.get_rect(self.slide_area);
        let mut rel_x = (self.mouse_x - area_rect[0]) / (area_rect[2] - area_rect[0]);
        rel_x = rel_x.max(0.0).min(1.0);
        self.value = rel_x * (self.max_value - self.min_value) + self.min_value;
    }

    fn set_handle_pos(&mut self, this: Id, ctx: &mut Context) {
        let this_rect = ctx.get_rect(this);
        let area_rect = ctx.get_rect(self.slide_area);

        let mut rel_x = (self.value - self.min_value) / (self.max_value - self.min_value);
        rel_x = rel_x.max(0.0).min(1.0);

        let margin_left = (area_rect[0] - this_rect[0]) / (this_rect[2] - this_rect[0]);
        let margin_right = (this_rect[2] - area_rect[2]) / (this_rect[2] - this_rect[0]);
        let x = margin_left + (1.0 - margin_left - margin_right) * rel_x;

        ctx.set_anchor_left(self.handle, x);
        ctx.set_anchor_right(self.handle, x);
    }
}
impl Behaviour for Slider {
    fn on_active(&mut self, this: Id, ctx: &mut Context) {
        self.set_handle_pos(this, ctx);
        let value = self.value;
        ctx.send_event(event::ValueSet { id: this, value });
        ctx.set_graphic(this, self.style.normal.clone());
    }

    fn on_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {
        if focus {
            ctx.set_graphic(this, self.style.focus.clone());
        } else {
            ctx.set_graphic(this, self.style.normal.clone());
        }
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        match event {
            MouseEvent::Enter => {}
            MouseEvent::Exit => {}
            MouseEvent::Down => {
                self.dragging = true;
                ctx.send_event(event::LockOver);
                self.update_value(ctx);
                self.set_handle_pos(this, ctx);
                let value = self.value;
                ctx.send_event(event::ValueChanged { id: this, value });
            }
            MouseEvent::Up => {
                self.dragging = false;
                self.set_handle_pos(this, ctx);
                let value = self.value;
                ctx.send_event(event::ValueSet { id: this, value });
                ctx.send_event(event::UnlockOver);
            }
            MouseEvent::Moved { x, .. } => {
                self.mouse_x = x;
                if self.dragging {
                    self.update_value(ctx);
                    self.set_handle_pos(this, ctx);
                    let value = self.value;
                    ctx.send_event(event::ValueChanged { id: this, value });
                }
            }
        }
        true
    }
}
