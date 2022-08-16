use std::collections::VecDeque;
use std::{any::Any, rc::Rc};

use instant::{Duration, Instant};
use winit::event::VirtualKeyCode;

use crate::{
    style::ButtonStyle, Behaviour, Context, Id, InputFlags, KeyboardEvent, Layout, LayoutContext,
    MinSizeContext, MouseButton, MouseEvent, MouseInfo,
};
use crate::{Animation, AnimationId};

pub struct SetScrollPosition {
    /// If true, it is setting the vertical scroll. Otherwise the horizontal.
    pub vertical: bool,
    /// A value between 0 and 1. 0 means at the top, 1 means at the bottom.
    pub value: f32,
}

pub struct ScrollDelta {
    pub delta: [f32; 2],
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

    pub fn set_anchors(
        ctx: &mut LayoutContext,
        handle: Id,
        vertical: bool,
        mut start: f32,
        mut end: f32,
        length: f32,
    ) {
        let handle_min_size = ctx.get_min_size(handle)[vertical as usize];

        let gap = handle_min_size - (end - start) * length;

        if gap > 0.0 {
            start *= 1.0 - gap / length;
            end *= 1.0 - gap / length;
        }

        if !vertical {
            ctx.set_anchor_left(handle, start);
            ctx.set_anchor_right(handle, end);
        } else {
            ctx.set_anchor_top(handle, start);
            ctx.set_anchor_bottom(handle, end);
        }
    }
}
impl Behaviour for ScrollBar {
    fn on_active(&mut self, _this: Id, ctx: &mut Context) {
        ctx.set_graphic(self.handle, self.style.normal.clone());
    }

    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE
    }

    fn on_mouse_event(&mut self, mouse: MouseInfo, _this: Id, ctx: &mut Context) {
        use MouseButton::*;
        match mouse.event {
            MouseEvent::Enter => {}
            MouseEvent::Exit => {
                ctx.set_graphic(self.handle, self.style.normal.clone());
            }
            MouseEvent::Down(Left) => {
                self.dragging = true;
                ctx.set_graphic(self.handle, self.style.pressed.clone());
                ctx.lock_cursor(true, mouse.id);
                let handle_rect = ctx.get_rect(self.handle);
                let area = ctx
                    .get_parent(self.handle)
                    .expect("the handle of the scrollbar must have a parent");
                let area_rect = ctx.get_rect(area);
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
                    ctx.lock_cursor(false, mouse.id);
                    ctx.set_graphic(self.handle, self.style.hover.clone());
                }
            }
            MouseEvent::Moved => {
                let [x, y] = mouse.pos;
                self.mouse_pos = if self.vertical { y } else { x };
                if self.dragging {
                    let handle_rect = ctx.get_rect(self.handle);
                    let area = ctx
                        .get_parent(self.handle)
                        .expect("handle must have a parent");
                    let area_rect = ctx.get_rect(area);

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
                    let handle_rect = ctx.get_rect(self.handle);
                    if self.mouse_pos < handle_rect[1] || self.mouse_pos > handle_rect[3] {
                        ctx.set_graphic(self.handle, self.style.normal.clone());
                    } else {
                        ctx.set_graphic(self.handle, self.style.hover.clone());
                    }
                }
            }
            MouseEvent::Up(_) => {}
            MouseEvent::Down(_) => {}
            MouseEvent::None => {}
        }
    }
}

#[derive(Default)]
pub struct ViewLayout {
    scroll_horz: bool,
    scroll_vert: bool,
}
impl ViewLayout {
    /// Create new ViewLayout.
    ///
    /// `scroll_horz` and `scroll_vert` tell, in its respective dimension, if the view should
    /// scroll the content if its exceed the view size, or if view min_size must be equal the
    /// min_size of the content.
    pub fn new(scroll_horz: bool, scroll_vert: bool) -> Self {
        Self {
            scroll_horz,
            scroll_vert,
        }
    }
}
impl Layout for ViewLayout {
    fn compute_min_size(&mut self, this: Id, ctx: &mut MinSizeContext) -> [f32; 2] {
        let content = match ctx.get_active_children(this).get(0) {
            Some(x) => *x,
            None => return [0.0; 2],
        };
        let mut min_size = [0.0, 0.0];
        let content_min_size = ctx.get_min_size(content);
        if !self.scroll_horz {
            min_size[0] = content_min_size[0];
        }
        if !self.scroll_vert {
            min_size[1] = content_min_size[1];
        }

        min_size
    }

    fn update_layouts(&mut self, _this: Id, _ctx: &mut LayoutContext) {}
}

const MEAN_SIZE: usize = 5;

pub struct FinishScrollMomentum;

/// Encapsulate the behaviour of dragging a scroll container that preserves the drag momentum.
///
/// Is used by composition. The owner Behaviour must have `InputFlags::DRAG`, delegate the
/// `on_mouse_event` to this struct, and responde to [`ScrollDelta`] event.
#[derive(Default)]
pub struct ScrollMomentum {
    drag_anim: Option<AnimationId>,
    position: VecDeque<([f32; 2], Instant)>,
    /// true while a animation is running.
    pub is_scrolling: bool,
}
impl ScrollMomentum {
    fn get_mean_velocity(&mut self) -> Option<[f32; 2]> {
        if self.position.len() < 2 {
            return None;
        }

        let first = self.position.front().unwrap();
        let last = self.position.back().unwrap();

        let (lpos, lt) = first;
        let (pos, t) = last;

        // log::trace!("positions: {:?}", {
        //     let now = Instant::now();
        //     self.position
        //         .iter()
        //         .map(|x| (x.0[1], now.duration_since(x.1)))
        //         .collect::<Vec<_>>()
        // });

        let dpos = [pos[0] - lpos[0], pos[1] - lpos[1]];
        let dt = t
            .checked_duration_since(*lt)
            .unwrap_or(Duration::ZERO)
            .as_secs_f32();

        Some([dpos[0] / dt, dpos[1] / dt])
    }

    pub fn cancel_scroll(&mut self, ctx: &mut Context) {
        self.drag_anim.take().map(|id| ctx.remove_animation(id));
        self.is_scrolling = false;
    }

    fn add_position(&mut self, pos: [f32; 2]) {
        let now = Instant::now();
        if self.position.len() > MEAN_SIZE {
            self.position.pop_front();
        }

        // remove old positions
        while let Some((_, time)) = self.position.front() {
            if now.duration_since(*time) < Duration::from_millis(50) {
                break;
            }
            self.position.pop_front();
        }

        self.position.push_back((pos, now));
    }

    pub fn on_mouse_event(&mut self, mouse: MouseInfo, this: Id, ctx: &mut Context) {
        if MouseEvent::Moved == mouse.event {
            self.add_position(mouse.pos);
        }
        match mouse.event {
            MouseEvent::Moved if mouse.is_dragging() => {
                self.cancel_scroll(ctx);

                ctx.send_event_to(
                    this,
                    ScrollDelta {
                        delta: mouse.drag_delta(),
                    },
                );
            }
            MouseEvent::Down(MouseButton::Left) => {
                self.cancel_scroll(ctx);
                self.position.clear();
            }
            MouseEvent::Enter if mouse.is_dragging() => {
                self.cancel_scroll(ctx);
                self.position.clear();
            }
            MouseEvent::Up(MouseButton::Left) if mouse.is_dragging() => {
                struct ScrollAnim {
                    id: Id,
                    speed: [f32; 2],
                }
                impl Animation for ScrollAnim {
                    fn on_update(&self, t: f32, dt: f32, _: f32, ctx: &mut Context) {
                        if t == 1.0 {
                            ctx.send_event_to(self.id, FinishScrollMomentum);
                        }
                        ctx.send_event_to(
                            self.id,
                            ScrollDelta {
                                delta: [
                                    dt * self.speed[0] * (1.0 - t),
                                    dt * self.speed[1] * (1.0 - t),
                                ],
                            },
                        )
                    }
                }

                self.add_position(mouse.pos);

                if let Some(mut speed) = self.get_mean_velocity() {
                    if !mouse.is_dragging_x {
                        speed[0] = 0.0;
                    }
                    if !mouse.is_dragging_y {
                        speed[1] = 0.0;
                    }

                    let drag_acel = 1000.0; // px/s^2
                    let mag = (speed[0] * speed[0] + speed[1] * speed[1]).sqrt();
                    let length = mag / drag_acel; // s
                    let id = ctx.add_animation(length, ScrollAnim { id: this, speed });
                    self.drag_anim = Some(id);
                    self.is_scrolling = true;
                }
            }
            _ => {}
        }
    }
}

pub struct ScrollView {
    pub delta_x: f32,
    pub delta_y: f32,
    view: Id,
    content: Id,
    h_scroll_bar_and_handle: Option<(Id, Id)>,
    v_scroll_bar_and_handle: Option<(Id, Id)>,

    momentum_scroll: ScrollMomentum,
}
impl ScrollView {
    /// Create a new ScrollView.
    ///
    /// The hiearchy of controls must be the following:
    ///
    /// ```text
    /// scroll_view : ScrollView
    /// ├─ view : ViewLayout
    /// │  └─ content
    /// ├─ h_bar : ScrollBar
    /// │  └─ h_bar_handle
    /// └─ v_bar : ScrollBar
    ///    └─ v_bar_handle
    /// ```
    ///
    /// The scrollview will layout the `view`, `h_bar` and `v_bar` in a 4x4 grid, with the
    /// bottom-right corner empty. `h_bar` and `v_bar` must have a non-zero min_size to be visible.
    ///
    /// `view` will layout `content` to be scrollable. If `content` min_size is smaller than
    /// `view` size in a dimension, the `content` will occupy the entire `view`, in that dimension.
    ///
    /// `h_bar` and `v_bar` will only be active if the min_size of `content` is greater than
    /// `view` size in its respective dimension.
    ///
    /// If `h_scroll_bar_and_handle` or `v_scroll_bar_and_handle` are None, ScrollView will not
    /// scroll the content in its respective dimension, and will instead inherit its min_size.
    pub fn new(
        view: Id,
        content: Id,
        h_scroll_bar_and_handle: Option<(Id, Id)>,
        v_scroll_bar_and_handle: Option<(Id, Id)>,
    ) -> Self {
        Self {
            delta_x: 0.0,
            delta_y: 0.0,
            view,
            content,
            h_scroll_bar_and_handle,
            v_scroll_bar_and_handle,
            momentum_scroll: ScrollMomentum::default(),
        }
    }

    fn add_delta(&mut self, delta: [f32; 2], ctx: &mut Context) {
        self.delta_x -= delta[0];
        self.delta_y -= delta[1];
        ctx.dirty_layout(self.view);
    }
}

impl Behaviour for ScrollView {
    fn on_start(&mut self, _this: Id, ctx: &mut Context) {
        debug_assert!({
            let children = ctx.get_all_children(_this);
            debug_assert!(
                children.contains(&self.view),
                "self.view should be a child of ScrollView"
            );
            debug_assert!(
                self.h_scroll_bar_and_handle
                    .as_ref()
                    .map(|x| children.contains(&x.0))
                    .unwrap_or(true),
                "self.h_scroll_bar should be a child of ScrollView"
            );
            debug_assert!(
                self.v_scroll_bar_and_handle
                    .as_ref()
                    .map(|x| children.contains(&x.0))
                    .unwrap_or(true),
                "self.v_scroll_bar should be a child of ScrollView"
            );
            debug_assert!(
                ctx.get_all_children(self.view).contains(&self.content),
                "self.content should be a child of self.view"
            );
            true
        });
        if let Some((h_scroll_bar, _)) = self.h_scroll_bar_and_handle {
            ctx.move_to_front(h_scroll_bar);
        }
        if let Some((v_scroll_bar, _)) = self.v_scroll_bar_and_handle {
            ctx.move_to_front(v_scroll_bar);
        }
    }

    fn on_active(&mut self, _: Id, ctx: &mut Context) {
        let content_size = ctx.get_min_size(self.content);

        let view_rect = ctx.get_rect(self.view);

        let view_width = view_rect[2] - view_rect[0];
        let view_height = view_rect[3] - view_rect[1];

        if let Some((_, h_scroll_bar_handle)) = self.h_scroll_bar_and_handle {
            ctx.set_anchor_left(h_scroll_bar_handle, self.delta_x / content_size[0]);
            ctx.set_anchor_right(
                h_scroll_bar_handle,
                ((self.delta_x + view_width) / content_size[0]).min(1.0),
            );
        }

        if let Some((_, v_scroll_bar_handle)) = self.v_scroll_bar_and_handle {
            ctx.set_anchor_top(v_scroll_bar_handle, self.delta_y / content_size[1]);
            ctx.set_anchor_bottom(
                v_scroll_bar_handle,
                ((self.delta_y + view_height) / content_size[1]).min(1.0),
            );
        }
    }

    fn on_event(&mut self, event: Box<dyn Any>, _: Id, ctx: &mut Context) {
        if let Some(event) = event.downcast_ref::<SetScrollPosition>() {
            self.momentum_scroll.cancel_scroll(ctx);
            if !event.vertical {
                let total_size = ctx.get_size(self.content)[0] - ctx.get_size(self.view)[0];
                self.delta_x = event.value * total_size;
            } else {
                let total_size = ctx.get_size(self.content)[1] - ctx.get_size(self.view)[1];
                self.delta_y = event.value * total_size;
            }
            ctx.dirty_layout(self.view);
        } else if let Some(event) = event.downcast_ref::<ScrollDelta>() {
            self.add_delta(event.delta, ctx);
        } else if event.is::<FinishScrollMomentum>() {
            self.momentum_scroll.is_scrolling = false;
        }
    }

    fn input_flags(&self) -> InputFlags {
        let mut flags = InputFlags::MOUSE | InputFlags::SCROLL | InputFlags::DRAG;
        if self.momentum_scroll.is_scrolling {
            flags |= InputFlags::BLOCK_MOUSE
        }
        flags
    }

    fn on_mouse_event(&mut self, mouse: MouseInfo, this: Id, ctx: &mut Context) {
        self.momentum_scroll.on_mouse_event(mouse, this, ctx)
    }

    fn on_scroll_event(&mut self, delta: [f32; 2], _: Id, ctx: &mut Context) {
        self.momentum_scroll.cancel_scroll(ctx);

        self.add_delta(delta, ctx);
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
impl Layout for ScrollView {
    fn compute_min_size(&mut self, _this: Id, ctx: &mut MinSizeContext) -> [f32; 2] {
        let mut min_size = ctx.get_min_size(self.view);
        let content_min_size = ctx.get_min_size(self.content);

        let h_scroll_bar_size = if let Some((h_scroll_bar, _)) = self.h_scroll_bar_and_handle {
            ctx.get_min_size(h_scroll_bar)
        } else {
            min_size[0] = content_min_size[0];
            [0.0, 0.0]
        };
        let v_scroll_bar_size = if let Some((v_scroll_bar, _)) = self.v_scroll_bar_and_handle {
            ctx.get_min_size(v_scroll_bar)
        } else {
            min_size[1] = content_min_size[1];
            [0.0, 0.0]
        };

        min_size[0] = min_size[0].max(h_scroll_bar_size[0]);
        min_size[1] = min_size[1].max(v_scroll_bar_size[1]);

        min_size[0] += v_scroll_bar_size[0];
        min_size[1] += h_scroll_bar_size[1];

        min_size
    }

    fn update_layouts(&mut self, this: Id, ctx: &mut LayoutContext) {
        let this_rect = ctx.get_rect(this);
        let content_size = ctx.get_min_size(self.content);
        let this_width = this_rect[2] - this_rect[0];
        let this_height = this_rect[3] - this_rect[1];

        let mut h_active;
        let mut h_scroll_bar_size;
        let mut h_scroll_bar;
        if let Some((_h_scroll_bar, _)) = self.h_scroll_bar_and_handle {
            h_scroll_bar = _h_scroll_bar;
            h_active = this_width < content_size[0];
            h_scroll_bar_size = if h_active {
                ctx.get_min_size(h_scroll_bar)[1]
            } else {
                0.0
            };
        } else {
            h_active = false;
            h_scroll_bar_size = 0.0;
            h_scroll_bar = Id::ROOT_ID; // dumb value
        }

        let v_active;
        let v_scroll_bar_size;
        let v_scroll_bar;
        if let Some((_v_scroll_bar, _)) = self.v_scroll_bar_and_handle {
            v_scroll_bar = _v_scroll_bar;
            v_active = this_height - h_scroll_bar_size < content_size[1];
            v_scroll_bar_size = if v_active {
                ctx.get_min_size(v_scroll_bar)[0]
            } else {
                0.0
            };
        } else {
            v_active = false;
            v_scroll_bar_size = 0.0;
            v_scroll_bar = Id::ROOT_ID; // dumb value
        }

        if let Some((_h_scroll_bar, _)) = self.h_scroll_bar_and_handle {
            if !h_active && this_width - v_scroll_bar_size < content_size[0] {
                h_active = true;
                h_scroll_bar = _h_scroll_bar;
                h_scroll_bar_size = ctx.get_min_size(h_scroll_bar)[1];
            }
        }

        if let Some((h_scroll_bar, _)) = self.h_scroll_bar_and_handle {
            if ctx.is_active(h_scroll_bar) {
                if !h_active {
                    ctx.deactive(h_scroll_bar);
                }
            } else if h_active {
                ctx.active(h_scroll_bar);
            }
        }

        if let Some((v_scroll_bar, _)) = self.v_scroll_bar_and_handle {
            if ctx.is_active(v_scroll_bar) {
                if !v_active {
                    ctx.deactive(v_scroll_bar);
                }
            } else if v_active {
                ctx.active(v_scroll_bar);
            }
        }

        if h_active {
            ctx.set_designed_rect(
                h_scroll_bar,
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
                v_scroll_bar,
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

        if h_active {
            if let Some((_, h_scroll_bar_handle)) = self.h_scroll_bar_and_handle {
                let start = self.delta_x / content_size[0];
                let end = ((self.delta_x + view_width) / content_size[0]).min(1.0);
                ScrollBar::set_anchors(ctx, h_scroll_bar_handle, false, start, end, view_width);
            }
        }

        if v_active {
            if let Some((_, v_scroll_bar_handle)) = self.v_scroll_bar_and_handle {
                let start = self.delta_y / content_size[1];
                let end = ((self.delta_y + this_height) / content_size[1]).min(1.0);
                ScrollBar::set_anchors(ctx, v_scroll_bar_handle, true, start, end, view_height);
            }
        }

        ctx.set_designed_rect(self.content, content_rect);
    }
}
