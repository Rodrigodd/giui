use event::SetValue;

use crate::{
    event, style::OnFocusStyle, Behaviour, Context, Id, InputFlags, MouseButton, MouseEvent,
};

use std::{any::Any, rc::Rc};

pub struct SetMinValue(pub i32);
pub struct SetMaxValue(pub i32);

pub trait SliderCallback {
    fn on_change(&mut self, this: Id, ctx: &mut Context, value: i32);
    fn on_release(&mut self, this: Id, ctx: &mut Context, value: i32);
}
impl<F: Fn(Id, &mut Context, i32)> SliderCallback for F {
    fn on_change(&mut self, this: Id, ctx: &mut Context, value: i32) {
        self(this, ctx, value)
    }
    fn on_release(&mut self, _this: Id, _ctx: &mut Context, _value: i32) {}
}
impl SliderCallback for () {
    fn on_change(&mut self, _: Id, _: &mut Context, _: i32) {}
    fn on_release(&mut self, _: Id, _: &mut Context, _: i32) {}
}

pub struct Slider<C: SliderCallback> {
    handle: Id,
    slide_area: Id, //TODO: I should remove this slide_area
    dragging: bool,
    mouse_x: f32,
    min: i32,
    max: i32,
    value: i32,
    style: Rc<OnFocusStyle>,
    callback: C,
}
impl<C: SliderCallback> Slider<C> {
    pub fn new(
        handle: Id,
        slide_area: Id,
        min: i32,
        max: i32,
        start_value: i32,
        style: Rc<OnFocusStyle>,
        callback: C,
    ) -> Self {
        Self {
            handle,
            slide_area,
            dragging: false,
            mouse_x: 0.0,
            max,
            min,
            value: start_value,
            style,
            callback,
        }
    }

    fn update_value(&mut self, ctx: &mut Context) {
        let area_rect = ctx.get_rect(self.slide_area);
        let mut rel_x = (self.mouse_x - area_rect[0]) / (area_rect[2] - area_rect[0]);
        rel_x = rel_x.max(0.0).min(1.0);
        self.value = (rel_x * (self.max - self.min) as f32).round() as i32 + self.min;
    }

    fn set_handle_pos(&mut self, this: Id, ctx: &mut Context) {
        let this_rect = ctx.get_rect(this);
        let area_rect = ctx.get_rect(self.slide_area);

        let mut rel_x = (self.value - self.min) as f32 / (self.max - self.min) as f32;
        rel_x = rel_x.max(0.0).min(1.0);

        let margin_left = (area_rect[0] - this_rect[0]) / (this_rect[2] - this_rect[0]);
        let margin_right = (this_rect[2] - area_rect[2]) / (this_rect[2] - this_rect[0]);
        let x = margin_left + (1.0 - margin_left - margin_right) * rel_x;

        ctx.set_anchor_left(self.handle, x);
        ctx.set_anchor_right(self.handle, x);
    }
}
impl<C: SliderCallback> Behaviour for Slider<C> {
    fn on_active(&mut self, this: Id, ctx: &mut Context) {
        self.set_handle_pos(this, ctx);
        ctx.set_graphic(this, self.style.normal.clone());
    }

    fn on_event(&mut self, event: &dyn Any, this: Id, ctx: &mut Context) {
        if let Some(SetMaxValue(x)) = event.downcast_ref::<SetMaxValue>() {
            self.max = *x;
            self.set_handle_pos(this, ctx);
        } else if let Some(SetMinValue(x)) = event.downcast_ref::<SetMinValue>() {
            self.min = *x;
            self.set_handle_pos(this, ctx);
        } else if let Some(SetValue(x)) = event.downcast_ref::<SetValue<i32>>() {
            self.value = *x;
            self.set_handle_pos(this, ctx);
            self.callback.on_change(this, ctx, self.value);
        }
    }

    fn on_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {
        if focus {
            ctx.set_graphic(this, self.style.focus.clone());
        } else {
            ctx.set_graphic(this, self.style.normal.clone());
        }
    }

    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) {
        use MouseButton::*;
        match event {
            MouseEvent::Enter => {}
            MouseEvent::Exit => {}
            MouseEvent::Down(Left) => {
                self.dragging = true;
                ctx.send_event(event::LockOver);
                self.update_value(ctx);
                self.set_handle_pos(this, ctx);
                let value = self.value;
                self.callback.on_change(this, ctx, value);
            }
            MouseEvent::Up(Left) => {
                self.dragging = false;
                self.set_handle_pos(this, ctx);
                let value = self.value;
                self.callback.on_release(this, ctx, value);
                ctx.send_event(event::UnlockOver);
            }
            MouseEvent::Moved { x, .. } => {
                self.mouse_x = x;
                if self.dragging {
                    self.update_value(ctx);
                    self.set_handle_pos(this, ctx);
                    let value = self.value;
                    self.callback.on_change(this, ctx, value);
                }
            }
            MouseEvent::Up(_) => {}
            MouseEvent::Down(_) => {}
        }
    }
}
