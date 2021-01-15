use crate::{Behaviour, Context, Id, KeyboardEvent, Layout, LayoutContext, MinSizeContext, MouseButton, MouseEvent, event, style::ButtonStyle};

use std::{any::Any, rc::Rc};
use winit::event::VirtualKeyCode;

struct SetScrollPosition {
    vertical: bool,
    value: f32,
}

pub struct ScrollBar {
    handle: Id,
    scroll_view: Id,
    dragging: bool,
    drag_start: f32,
    mouse_pos: f32,
    curr_value: f32,
    vertical: bool,
    style: Rc<ButtonStyle>,
}
impl ScrollBar {
    pub fn new(handle: Id, scroll_view: Id, vertical: bool, style: Rc<ButtonStyle>) -> Self {
        Self {
            handle,
            scroll_view,
            dragging: false,
            drag_start: 0.0,
            mouse_pos: 0.0,
            curr_value: 0.0,
            vertical,
            style,
        }
    }
}
impl Behaviour for ScrollBar {
    fn on_active(&mut self, _this: Id, ctx: &mut Context) {
        ctx.set_graphic(self.handle, self.style.normal.clone());
    }

    fn on_mouse_event(&mut self, event: MouseEvent, _this: Id, ctx: &mut Context) -> bool {
        use MouseButton::*;
        match event {
            MouseEvent::Enter => {}
            MouseEvent::Exit => {
                ctx.set_graphic(self.handle, self.style.normal.clone());
            }
            MouseEvent::Down(Left) => {
                self.dragging = true;
                ctx.set_graphic(self.handle, self.style.pressed.clone());
                ctx.send_event(event::LockOver);
                let handle_rect = *ctx.get_rect(self.handle);
                let area = ctx
                    .get_parent(self.handle)
                    .expect("the handle of the scrollbar must have a parent");
                let area_rect = *ctx.get_rect(area);
                self.drag_start = self.mouse_pos;
                if !self.vertical {
                    let handle_size = handle_rect[2] - handle_rect[0];
                    let area_size = area_rect[2] - area_rect[0] - handle_size;
                    if self.mouse_pos < handle_rect[0] || self.mouse_pos > handle_rect[2] {
                        self.curr_value =
                            (self.mouse_pos - (area_rect[0] + handle_size / 2.0)) / area_size;
                        ctx.send_event_to(
                            self.scroll_view,
                            SetScrollPosition {
                                vertical: false,
                                value: self.curr_value,
                            },
                        )
                    } else {
                        self.curr_value = (handle_rect[0] - area_rect[0]) / area_size;
                    }
                } else {
                    let handle_size = handle_rect[3] - handle_rect[1];
                    let area_size = area_rect[3] - area_rect[1] - handle_size;
                    if self.mouse_pos < handle_rect[1] || self.mouse_pos > handle_rect[3] {
                        self.curr_value =
                            (self.mouse_pos - (area_rect[1] + handle_size / 2.0)) / area_size;
                        ctx.send_event_to(
                            self.scroll_view,
                            SetScrollPosition {
                                vertical: true,
                                value: self.curr_value,
                            },
                        )
                    } else {
                        self.curr_value = (handle_rect[1] - area_rect[1]) / area_size;
                    }
                }
            }
            MouseEvent::Up(Left) => {
                if self.dragging {
                    self.dragging = false;
                    ctx.send_event(event::UnlockOver);
                    ctx.set_graphic(self.handle, self.style.hover.clone());
                }
            }
            MouseEvent::Moved { x, y } => {
                self.mouse_pos = if self.vertical { y } else { x };
                if self.dragging {
                    let handle_rect = *ctx.get_rect(self.handle);
                    let area = ctx
                        .get_parent(self.handle)
                        .expect("handle must have a parent");
                    let area_rect = *ctx.get_rect(area);

                    let handle_size = if !self.vertical {
                        handle_rect[2] - handle_rect[0]
                    } else {
                        handle_rect[3] - handle_rect[1]
                    };
                    let area_size = if !self.vertical {
                        area_rect[2] - area_rect[0] - handle_size
                    } else {
                        area_rect[3] - area_rect[1] - handle_size
                    };

                    let value = if area_size != 0.0 {
                        self.curr_value + (self.mouse_pos - self.drag_start) / area_size
                    } else {
                        0.0
                    };

                    ctx.send_event_to(
                        self.scroll_view,
                        SetScrollPosition {
                            vertical: self.vertical,
                            value,
                        },
                    )
                } else {
                    let handle_rect = *ctx.get_rect(self.handle);
                    if self.mouse_pos < handle_rect[1] || self.mouse_pos > handle_rect[3] {
                        ctx.set_graphic(self.handle, self.style.normal.clone());
                    } else {
                        ctx.set_graphic(self.handle, self.style.hover.clone());
                    }
                }
            }
            MouseEvent::Up(_) => {}
            MouseEvent::Down(_) => {}
        }
        true
    }
}

pub struct NoneLayout;
impl Layout for NoneLayout {
    fn update_layouts(&mut self, _: Id, _: &mut LayoutContext) {}
}

pub struct ScrollView {
    pub delta_x: f32,
    pub delta_y: f32,
    view: Id,
    content: Id,
    h_scroll_bar: Id,
    h_scroll_bar_handle: Id,
    v_scroll_bar: Id,
    v_scroll_bar_handle: Id,
}
impl ScrollView {
    /// v_scroll must be a descendant of this
    pub fn new(
        view: Id,
        content: Id,
        h_scroll_bar: Id,
        h_scroll_bar_handle: Id,
        v_scroll_bar: Id,
        v_scroll_bar_handle: Id,
    ) -> Self {
        Self {
            delta_x: 0.0,
            delta_y: 0.0,
            view,
            content,
            h_scroll_bar,
            h_scroll_bar_handle,
            v_scroll_bar,
            v_scroll_bar_handle,
        }
    }
}
impl Behaviour for ScrollView {
    fn on_start(&mut self, _this: Id, ctx: &mut Context) {
        ctx.move_to_front(self.v_scroll_bar);
        ctx.move_to_front(self.h_scroll_bar);
    }

    fn on_active(&mut self, _: Id, ctx: &mut Context) {
        let content_size = ctx.get_min_size(self.content);

        let view_rect = ctx.get_rect(self.view);

        let view_width = view_rect[2] - view_rect[0];
        let view_height = view_rect[3] - view_rect[1];

        ctx.set_anchor_left(self.h_scroll_bar_handle, self.delta_x / content_size[0]);
        ctx.set_anchor_right(
            self.h_scroll_bar_handle,
            ((self.delta_x + view_width) / content_size[0]).min(1.0),
        );

        ctx.set_anchor_top(self.v_scroll_bar_handle, self.delta_y / content_size[1]);
        ctx.set_anchor_bottom(
            self.v_scroll_bar_handle,
            ((self.delta_y + view_height) / content_size[1]).min(1.0),
        );
    }

    fn on_event(&mut self, event: &dyn Any, _: Id, ctx: &mut Context) {
        if let Some(event) = event.downcast_ref::<SetScrollPosition>() {
            if !event.vertical {
                let total_size = ctx.get_size(self.content)[0] - ctx.get_size(self.view)[0];
                self.delta_x = event.value * total_size;
            } else {
                let total_size = ctx.get_size(self.content)[1] - ctx.get_size(self.view)[1];
                self.delta_y = event.value * total_size;
            }
            ctx.dirty_layout(self.view);
        }
    }

    fn on_scroll_event(&mut self, delta: [f32; 2], _: Id, ctx: &mut Context) -> bool {
        self.delta_x += delta[0];
        self.delta_y -= delta[1];

        ctx.dirty_layout(self.view);
        true
    }

    fn on_keyboard_event(&mut self, event: KeyboardEvent, _this: Id, ctx: &mut Context) -> bool {
        match event {
            KeyboardEvent::Pressed(key) => match key {
                VirtualKeyCode::Up => {
                    self.delta_y -= 30.0;
                    ctx.dirty_layout(self.view);
                    true
                }
                VirtualKeyCode::Down => {
                    self.delta_y += 30.0;
                    ctx.dirty_layout(self.view);
                    true
                }
                VirtualKeyCode::Right => {
                    self.delta_x += 30.0;
                    ctx.dirty_layout(self.view);
                    true
                }
                VirtualKeyCode::Left => {
                    self.delta_x -= 30.0;
                    ctx.dirty_layout(self.view);
                    true
                }
                VirtualKeyCode::Home => {
                    self.delta_y = 0.0;
                    ctx.dirty_layout(self.view);
                    true
                }
                VirtualKeyCode::End => {
                    self.delta_y = f32::INFINITY;
                    ctx.dirty_layout(self.view);
                    true
                }
                VirtualKeyCode::PageUp => {
                    let height = ctx.get_size(self.view)[1] - 40.0;
                    self.delta_y -= height;
                    ctx.dirty_layout(self.view);
                    true
                }
                VirtualKeyCode::PageDown => {
                    let height = ctx.get_size(self.view)[1] - 40.0;
                    self.delta_y += height;
                    ctx.dirty_layout(self.view);
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }
}

impl<T: Layout> Layout for std::rc::Rc<std::cell::RefCell<T>> {
    fn compute_min_size(&mut self, this: Id, ctx: &mut MinSizeContext) {
        self.as_ref().borrow_mut().compute_min_size(this, ctx)
    }

    fn update_layouts(&mut self, this: Id, ctx: &mut LayoutContext) {
        self.as_ref().borrow_mut().update_layouts(this, ctx)
    }
}
impl<T: Behaviour> Behaviour for std::rc::Rc<std::cell::RefCell<T>> {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        self.as_ref().borrow_mut().on_start(this, ctx)
    }

    fn on_active(&mut self, this: Id, ctx: &mut Context) {
        self.as_ref().borrow_mut().on_active(this, ctx)
    }

    fn on_deactive(&mut self, this: Id, ctx: &mut Context) {
        self.as_ref().borrow_mut().on_deactive(this, ctx)
    }

    fn on_event(&mut self, event: &dyn Any, this: Id, ctx: &mut Context) {
        self.as_ref().borrow_mut().on_event(event, this, ctx)
    }

    fn on_scroll_event(&mut self, delta: [f32; 2], this: Id, ctx: &mut Context) -> bool {
        self.as_ref().borrow_mut().on_scroll_event(delta, this, ctx)
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        self.as_ref().borrow_mut().on_mouse_event(event, this, ctx)
    }

    fn on_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {
        self.as_ref().borrow_mut().on_focus_change(focus, this, ctx)
    }

    fn on_keyboard_event(&mut self, event: KeyboardEvent, this: Id, ctx: &mut Context) -> bool {
        self.as_ref()
            .borrow_mut()
            .on_keyboard_event(event, this, ctx)
    }
}

impl Layout for ScrollView {
    fn compute_min_size(&mut self, _this: Id, ctx: &mut MinSizeContext) {
        let mut min_size = ctx.get_min_size(self.view);

        let h_scroll_bar_size = ctx.get_min_size(self.v_scroll_bar);
        let v_scroll_bar_size = ctx.get_min_size(self.v_scroll_bar);

        min_size[0] = min_size[0].max(h_scroll_bar_size[0]);
        min_size[1] = min_size[1].max(v_scroll_bar_size[1]);

        min_size[0] += v_scroll_bar_size[0];
        min_size[1] += h_scroll_bar_size[1];

        ctx.set_this_min_size(min_size);
    }

    fn update_layouts(&mut self, this: Id, ctx: &mut LayoutContext) {
        let this_rect = *ctx.get_rect(this);
        let content = ctx.get_children(self.view)[0];
        let content_size = ctx.get_min_size(content);
        let this_width = this_rect[2] - this_rect[0];
        let this_height = this_rect[3] - this_rect[1];

        let mut h_active = this_width < content_size[0];
        let mut h_scroll_bar_size = if h_active {
            ctx.get_min_size(self.h_scroll_bar)[1]
        } else {
            0.0
        };

        let v_active = this_height - h_scroll_bar_size < content_size[1];
        let v_scroll_bar_size = if v_active {
            ctx.get_min_size(self.v_scroll_bar)[0]
        } else {
            0.0
        };

        if !h_active && this_width - v_scroll_bar_size < content_size[0] {
            h_active = true;
            h_scroll_bar_size = ctx.get_min_size(self.h_scroll_bar)[1];
        }

        if ctx.is_active(self.h_scroll_bar) {
            if !h_active {
                ctx.deactive(self.h_scroll_bar);
            }
        } else if h_active {
            ctx.active(self.h_scroll_bar);
        }

        if ctx.is_active(self.v_scroll_bar) {
            if !v_active {
                ctx.deactive(self.v_scroll_bar);
            }
        } else if v_active {
            ctx.active(self.v_scroll_bar);
        }

        if h_active {
            ctx.set_designed_rect(
                self.h_scroll_bar,
                [
                    this_rect[0],
                    this_rect[3] - h_scroll_bar_size,
                    this_rect[2] - v_scroll_bar_size,
                    this_rect[3],
                ],
            );
        }
        if v_active {
            ctx.set_designed_rect(
                self.v_scroll_bar,
                [
                    this_rect[2] - v_scroll_bar_size,
                    this_rect[1],
                    this_rect[2],
                    this_rect[3] - h_scroll_bar_size,
                ],
            );
        }

        ctx.set_designed_rect(
            self.view,
            [
                this_rect[0],
                this_rect[1],
                this_rect[2] - v_scroll_bar_size,
                this_rect[3] - h_scroll_bar_size,
            ],
        );

        let mut content_rect = [0.0; 4];

        let view_width = this_rect[2] - this_rect[0] - v_scroll_bar_size;
        let view_height = this_rect[3] - this_rect[1] - h_scroll_bar_size;

        if self.delta_x < 0.0 || view_width > content_size[0] {
            self.delta_x = 0.0;
        } else if self.delta_x > content_size[0] - view_width {
            self.delta_x = content_size[0] - view_width;
        }
        if self.delta_y < 0.0 || view_height > content_size[1] {
            self.delta_y = 0.0;
        } else if self.delta_y > content_size[1] - view_height {
            self.delta_y = content_size[1] - view_height;
        }

        if content_size[0] < view_width {
            content_rect[0] = this_rect[0];
            content_rect[2] = this_rect[0] + view_width;
        } else {
            content_rect[0] = this_rect[0] - self.delta_x;
            content_rect[2] = this_rect[0] - self.delta_x + content_size[0];
        }

        if content_size[1] < view_height {
            content_rect[1] = this_rect[1];
            content_rect[3] = this_rect[1] + view_height;
        } else {
            content_rect[1] = this_rect[1] - self.delta_y;
            content_rect[3] = this_rect[1] - self.delta_y + content_size[1];
        }

        ctx.set_anchor_left(self.h_scroll_bar_handle, self.delta_x / content_size[0]);
        ctx.set_anchor_right(
            self.h_scroll_bar_handle,
            ((self.delta_x + this_width) / content_size[0]).min(1.0),
        );

        ctx.set_anchor_top(self.v_scroll_bar_handle, self.delta_y / content_size[1]);
        ctx.set_anchor_bottom(
            self.v_scroll_bar_handle,
            ((self.delta_y + this_height) / content_size[1]).min(1.0),
        );

        ctx.set_designed_rect(content, content_rect);
    }
}
