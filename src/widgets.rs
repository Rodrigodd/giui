use crate::{
    event, render::Graphic, text::TextInfo, Behaviour, Context, Id, KeyboardEvent, LayoutContext,
    MinSizeContext, MouseEvent,
};
use copypasta::{ClipboardContext, ClipboardProvider};
use std::any::Any;
use winit::event::VirtualKeyCode;

#[derive(Default)]
pub struct Button {
    pub click: bool,
}
impl Button {
    pub fn new() -> Self {
        Self { click: false }
    }
}
impl Behaviour for Button {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        let graphic = ctx.get_graphic(this).unwrap();
        graphic.set_color([200, 200, 200, 255]);
        ctx.send_event(event::Redraw);
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        match event {
            MouseEvent::Enter => {
                let graphic = ctx.get_graphic(this).unwrap();
                graphic.set_color([180, 180, 180, 255]);
                ctx.send_event(event::Redraw);
            }
            MouseEvent::Exit => {
                self.click = false;
                let graphic = ctx.get_graphic(this).unwrap();
                graphic.set_color([200, 200, 200, 255]);
                ctx.send_event(event::Redraw);
            }
            MouseEvent::Down => {
                self.click = true;
                let graphic = ctx.get_graphic(this).unwrap();
                graphic.set_color([128, 128, 128, 255]);
                ctx.send_event(event::Redraw);
            }
            MouseEvent::Up => {
                let graphic = ctx.get_graphic(this).unwrap();
                graphic.set_color([180, 180, 180, 255]);
                ctx.send_event(event::Redraw);
                if self.click {
                    ctx.send_event(event::ButtonClicked { id: this });
                }
            }
            MouseEvent::Moved { .. } => {}
        }
        true
    }
}

pub struct Slider {
    handle: Id,
    slide_area: Id, //TODO: I should remove this slide_area
    dragging: bool,
    mouse_x: f32,
    min_value: f32,
    max_value: f32,
    value: f32,
}
impl Slider {
    pub fn new(
        handle: Id,
        slide_area: Id,
        min_value: f32,
        max_value: f32,
        start_value: f32,
    ) -> Self {
        Self {
            handle,
            slide_area,
            dragging: false,
            mouse_x: 0.0,
            max_value,
            min_value,
            value: start_value,
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
        ctx.send_event(event::Redraw);
    }
}
impl Behaviour for Slider {
    fn on_active(&mut self, this: Id, ctx: &mut Context) {
        self.set_handle_pos(this, ctx);
        let value = self.value;
        ctx.send_event(event::ValueSet { id: this, value });
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

pub struct Toggle {
    click: bool,
    enable: bool,
    background: Id,
    marker: Id,
}
impl Toggle {
    pub fn new(background: Id, marker: Id) -> Self {
        Self {
            click: false,
            enable: false,
            background,
            marker,
        }
    }
}
impl Behaviour for Toggle {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        ctx.send_event(event::ToggleChanged {
            id: this,
            value: self.enable,
        });
        if self.enable {
            ctx.get_graphic(self.marker).unwrap().set_alpha(255)
        } else {
            ctx.get_graphic(self.marker).unwrap().set_alpha(0)
        }
    }
    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        match event {
            MouseEvent::Enter => {
                let graphic = ctx.get_graphic(self.background).unwrap();
                graphic.set_color([190, 190, 190, 255]);
                ctx.send_event(event::Redraw);
            }
            MouseEvent::Exit => {
                self.click = false;
                let graphic = ctx.get_graphic(self.background).unwrap();
                graphic.set_color([200, 200, 200, 255]);
                ctx.send_event(event::Redraw);
            }
            MouseEvent::Down => {
                self.click = true;
                let graphic = ctx.get_graphic(self.background).unwrap();
                graphic.set_color([170, 170, 170, 255]);
                ctx.send_event(event::Redraw);
            }
            MouseEvent::Up => {
                let graphic = ctx.get_graphic(self.background).unwrap();
                graphic.set_color([190, 190, 190, 255]);
                ctx.send_event(event::Redraw);
                if self.click {
                    self.enable = !self.enable;
                    ctx.send_event(event::ToggleChanged {
                        id: this,
                        value: self.enable,
                    });
                    if self.enable {
                        ctx.get_graphic(self.marker).unwrap().set_alpha(255)
                    } else {
                        ctx.get_graphic(self.marker).unwrap().set_alpha(0)
                    }
                }
            }
            MouseEvent::Moved { .. } => {}
        }
        true
    }
}

pub struct Unselected;
pub struct Selected;
pub struct Select(Id);

#[derive(Default, Clone)]
/// It is basically a Rc<RefCell<Option<Id>>>.
pub struct ButtonGroup(std::rc::Rc<std::cell::RefCell<Option<Id>>>);
impl ButtonGroup {
    pub fn new() -> ButtonGroup {
        ButtonGroup::default()
    }
    pub fn selected(&self) -> Option<Id> {
        *self.0.borrow()
    }
    pub fn set_selected(&mut self, selected: Option<Id>) {
        *self.0.borrow_mut() = selected;
    }
}

pub struct TabButton {
    tab_group: ButtonGroup,
    page: Id,
    selected: bool,
    click: bool,
}
impl TabButton {
    pub fn new(tab_group: ButtonGroup, page: Id, selected: bool) -> Self {
        Self {
            tab_group,
            page,
            selected,
            click: false,
        }
    }

    pub fn select(&mut self, this: Id, ctx: &mut Context) {
        if let Some(selected) = self.tab_group.selected() {
            ctx.send_event_to(selected, Unselected);
        }
        ctx.active(self.page);
        self.selected = true;
        self.tab_group.set_selected(Some(this));
        let graphic = ctx.get_graphic(this).unwrap();
        graphic.set_color([255, 255, 255, 255]);
        ctx.send_event(event::Redraw);
    }

    pub fn unselect(&mut self, this: Id, ctx: &mut Context) {
        ctx.deactive(self.page);
        self.selected = false;
        let graphic = ctx.get_graphic(this).unwrap();
        graphic.set_color([200, 200, 200, 255]);
        ctx.send_event(event::Redraw);
    }
}
impl Behaviour for TabButton {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        if self.selected {
            self.select(this, ctx);
        } else {
            self.unselect(this, ctx);
        }
    }

    fn on_event(&mut self, event: &dyn Any, this: Id, ctx: &mut Context) {
        if event.is::<Unselected>() {
            self.unselect(this, ctx)
        } else if event.is::<Selected>() {
            self.select(this, ctx);
        }
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        match event {
            MouseEvent::Enter => {
                if !self.selected {
                    let graphic = ctx.get_graphic(this).unwrap();
                    graphic.set_color([180, 180, 180, 255]);
                    ctx.send_event(event::Redraw);
                }
            }
            MouseEvent::Exit => {
                if !self.selected {
                    self.click = false;
                    let graphic = ctx.get_graphic(this).unwrap();
                    graphic.set_color([200, 200, 200, 255]);
                    ctx.send_event(event::Redraw);
                }
            }
            MouseEvent::Down => {
                if !self.selected {
                    self.click = true;
                    let graphic = ctx.get_graphic(this).unwrap();
                    graphic.set_color([128, 128, 128, 255]);
                    ctx.send_event(event::Redraw);
                }
            }
            MouseEvent::Up => {
                if !self.selected {
                    let graphic = ctx.get_graphic(this).unwrap();

                    if self.click {
                        self.select(this, ctx);
                    } else {
                        graphic.set_color([180, 180, 180, 255]);
                    }
                    ctx.send_event(event::Redraw);
                }
            }
            MouseEvent::Moved { .. } => {}
        }
        true
    }
}

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
                ctx.get_graphic(self.label).unwrap().set_text(&self.text);
                ctx.dirty_layout(self.label);
                ctx.move_to_front(self.hover);
                self.is_over = true;
                ctx.send_event(event::Redraw);
            }
            MouseEvent::Exit => {
                ctx.deactive(self.hover);
                self.is_over = false;
                ctx.send_event(event::Redraw);
            }
            MouseEvent::Down => {}
            MouseEvent::Up => {}
            MouseEvent::Moved { x, y } => {
                if self.is_over {
                    let [width, heigth] = ctx.get_size(crate::ROOT_ID);
                    let x = x / width;
                    let y = y / heigth;
                    ctx.set_anchors(self.hover, [x, y, x, y]);
                    ctx.send_event(event::Redraw);
                }
            }
        }
        true
    }
}

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
}
impl ScrollBar {
    pub fn new(handle: Id, scroll_view: Id, vertical: bool) -> Self {
        Self {
            handle,
            scroll_view,
            dragging: false,
            drag_start: 0.0,
            mouse_pos: 0.0,
            curr_value: 0.0,
            vertical,
        }
    }
}
impl Behaviour for ScrollBar {
    fn on_mouse_event(&mut self, event: MouseEvent, _this: Id, ctx: &mut Context) -> bool {
        match event {
            MouseEvent::Enter => {}
            MouseEvent::Exit => {
                if let Some(handle) = ctx.get_graphic(self.handle) {
                    handle.set_color([220, 220, 220, 255]);
                    ctx.send_event(event::Redraw);
                }
            }
            MouseEvent::Down => {
                self.dragging = true;
                if let Some(handle) = ctx.get_graphic(self.handle) {
                    handle.set_color([180, 180, 180, 255]);
                    ctx.send_event(event::Redraw);
                }
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
            MouseEvent::Up => {
                if self.dragging {
                    self.dragging = false;
                    ctx.send_event(event::UnlockOver);
                    if let Some(graphic) = ctx.get_graphic(self.handle) {
                        graphic.set_color([200, 200, 200, 255]);
                        ctx.send_event(event::Redraw);
                    }
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
                } else if ctx.get_graphic(self.handle).is_some() {
                    let handle_rect = *ctx.get_rect(self.handle);
                    let graphic = ctx.get_graphic(self.handle).unwrap();
                    if self.mouse_pos < handle_rect[1] || self.mouse_pos > handle_rect[3] {
                        graphic.set_color([220, 220, 220, 255]);
                    } else {
                        graphic.set_color([200, 200, 200, 255]);
                    }
                    ctx.send_event(event::Redraw);
                }
            }
        }
        true
    }
}

pub struct NoneLayout;
impl Behaviour for NoneLayout {
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
            ctx.send_event(event::Redraw);
        }
    }

    fn on_scroll_event(&mut self, delta: [f32; 2], _: Id, ctx: &mut Context) -> bool {
        self.delta_x += delta[0];
        self.delta_y -= delta[1];

        ctx.dirty_layout(self.view);
        ctx.send_event(event::Redraw);
        true
    }

    fn on_keyboard_event(&mut self, event: KeyboardEvent, _this: Id, ctx: &mut Context) -> bool {
        match event {
            KeyboardEvent::Pressed(key) => match key {
                VirtualKeyCode::Up => {
                    self.delta_y -= 30.0;
                    ctx.dirty_layout(self.view);
                    ctx.send_event(event::Redraw);
                    true
                }
                VirtualKeyCode::Down => {
                    self.delta_y += 30.0;
                    ctx.dirty_layout(self.view);
                    ctx.send_event(event::Redraw);
                    true
                }
                VirtualKeyCode::Right => {
                    self.delta_x += 30.0;
                    ctx.dirty_layout(self.view);
                    ctx.send_event(event::Redraw);
                    true
                }
                VirtualKeyCode::Left => {
                    self.delta_x -= 30.0;
                    ctx.dirty_layout(self.view);
                    ctx.send_event(event::Redraw);
                    true
                }
                VirtualKeyCode::Home => {
                    self.delta_y = 0.0;
                    ctx.dirty_layout(self.view);
                    ctx.send_event(event::Redraw);
                    true
                }
                VirtualKeyCode::End => {
                    self.delta_y = f32::INFINITY;
                    ctx.dirty_layout(self.view);
                    ctx.send_event(event::Redraw);
                    true
                }
                VirtualKeyCode::PageUp => {
                    let height = ctx.get_size(self.view)[1] - 40.0;
                    self.delta_y -= height;
                    ctx.dirty_layout(self.view);
                    ctx.send_event(event::Redraw);
                    true
                }
                VirtualKeyCode::PageDown => {
                    let height = ctx.get_size(self.view)[1] - 40.0;
                    self.delta_y += height;
                    ctx.dirty_layout(self.view);
                    ctx.send_event(event::Redraw);
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }
}

pub struct TextField {
    caret: Id,
    label: Id,
    text: String,
    caret_index: usize,
    selection_index: Option<usize>,
    text_info: TextInfo,
    text_width: f32,
    x_scroll: f32,
    on_focus: bool,
    mouse_x: f32,
    mouse_down: bool,
}
impl TextField {
    pub fn new(caret: Id, label: Id) -> Self {
        Self {
            caret,
            label,
            text: String::new(),
            caret_index: 0,
            selection_index: None,
            text_info: TextInfo::default(),
            text_width: 0.0,
            x_scroll: 0.0,
            on_focus: false,
            mouse_x: 0.0,
            mouse_down: false,
        }
    }
}
impl TextField {
    fn update_text(&mut self, this: Id, ctx: &mut Context) {
        let fonts = ctx.get_fonts();
        if let Some((ref mut rect, Graphic::Text(text))) = ctx.get_rect_and_graphic(self.label) {
            let display_text = self.text.clone();
            text.set_text(&display_text);
            let min_size = text.compute_min_size(fonts).unwrap_or([0.0, 0.0]);
            self.text_width = min_size[0];
            rect.set_min_size(min_size);
            self.text_info = text.get_text_info(fonts, rect).clone();
            self.update_carret(this, ctx, true);
        }
    }

    fn update_carret(&mut self, this: Id, ctx: &mut Context, focus_caret: bool) {
        let this_rect = *ctx.get_rect(this);

        let mut caret_pos = self.text_info.get_caret_pos(self.caret_index);

        const MARGIN: f32 = 5.0;

        let this_width = this_rect[2] - this_rect[0];
        if this_width > self.text_width {
            self.x_scroll = -MARGIN;
        } else if focus_caret {
            if caret_pos[0] - self.x_scroll > this_width - MARGIN {
                self.x_scroll = caret_pos[0] - (this_width - MARGIN);
            }
            if caret_pos[0] - self.x_scroll < MARGIN {
                self.x_scroll = caret_pos[0] - MARGIN;
            }
        } else {
            if self.text_width - self.x_scroll < this_width - MARGIN {
                self.x_scroll = self.text_width - (this_width - MARGIN);
            }
            if self.x_scroll < -MARGIN {
                self.x_scroll = -MARGIN;
            }
        }

        ctx.set_margin_left(self.label, -self.x_scroll);

        caret_pos[0] -= self.x_scroll;

        if let Some(selection_index) = self.selection_index {
            ctx.get_graphic(self.caret)
                .unwrap()
                .set_color([51, 153, 255, 255]);
            let mut selection_pos = self.text_info.get_caret_pos(selection_index);
            selection_pos[0] -= self.x_scroll;
            let mut margins = [
                caret_pos[0],
                caret_pos[1] - self.text_info.get_line_heigth(),
                selection_pos[0],
                caret_pos[1],
            ];
            if margins[0] > margins[2] {
                margins.swap(0, 2);
            }
            if margins[1] > margins[3] {
                margins.swap(1, 3);
            }
            ctx.set_margins(self.caret, margins);
        } else {
            ctx.get_graphic(self.caret)
                .unwrap()
                .set_color([0, 0, 0, 255]);
            if self.on_focus {
                ctx.set_margins(
                    self.caret,
                    [
                        caret_pos[0],
                        caret_pos[1] - self.text_info.get_line_heigth(),
                        caret_pos[0] + 1.0,
                        caret_pos[1],
                    ],
                );
            } else {
                ctx.set_margins(self.caret, [0.0, 0.0, 0.0, 0.0]);
            }
        }
        ctx.send_event(event::Redraw);
    }

    fn move_caret(&mut self, caret: usize, ctx: &mut Context) {
        if ctx.modifiers().shift() {
            if let Some(selection_index) = self.selection_index {
                if selection_index == caret {
                    self.selection_index = None;
                }
            } else {
                self.selection_index = Some(self.caret_index);
            }
        } else if let Some(selection_index) = self.selection_index {
            let start = selection_index;
            let end = self.caret_index;
            if (caret < self.caret_index) ^ (start > end) {
                self.caret_index = start;
            } else {
                self.caret_index = end;
            }
            self.selection_index = None;
            return;
        }
        self.caret_index = caret;
    }

    fn delete_selection(&mut self, this: Id, ctx: &mut Context) {
        let selection_index = self.selection_index.unwrap();
        let a = self.text_info.get_indice(self.caret_index);
        let b = self.text_info.get_indice(selection_index);
        let range = if a > b { b..a } else { a..b };
        if self.caret_index > selection_index {
            self.caret_index = selection_index;
        }
        self.selection_index = None;
        self.text.replace_range(range, "");
        self.update_text(this, ctx);
    }

    fn insert_char(&mut self, ch: char, this: Id, ctx: &mut Context) {
        self.text
            .insert(self.text_info.get_indice(self.caret_index), ch);
        self.caret_index += 1;
        self.update_text(this, ctx);
    }
}
impl Behaviour for TextField {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        self.update_text(this, ctx);
        ctx.move_to_front(self.label);
    }

    fn on_event(&mut self, event: &dyn Any, this: Id, ctx: &mut Context) {
        if event.is::<event::ClearText>() {
            self.text.clear();
            self.caret_index = 0;
            self.selection_index = None;
            self.update_text(this, ctx);
        }
    }

    fn on_scroll_event(&mut self, delta: [f32; 2], this: Id, ctx: &mut Context) -> bool {
        let delta = if delta[0].abs() > delta[1].abs() {
            delta[0]
        } else {
            delta[1]
        };
        self.x_scroll -= delta;
        self.update_carret(this, ctx, false);

        true
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        match event {
            MouseEvent::Enter => {}
            MouseEvent::Exit => {}
            MouseEvent::Down => {
                if !self.on_focus {
                    ctx.send_event(event::RequestKeyboardFocus { id: this });
                }
                let left = ctx.get_rect(this)[0] - self.x_scroll;
                let x = self.mouse_x - left;
                self.caret_index = self.text_info.get_caret_index_at_pos(0, x);
                self.mouse_down = true;
                self.selection_index = None;
                self.update_carret(this, ctx, true);
                ctx.send_event(event::LockOver);
            }
            MouseEvent::Up => {
                self.mouse_down = false;
                ctx.send_event(event::UnlockOver);
            }
            MouseEvent::Moved { x, .. } => {
                self.mouse_x = x;
                if self.mouse_down {
                    let left = ctx.get_rect(this)[0] - self.x_scroll;
                    let x = self.mouse_x - left;
                    let caret_index = self.text_info.get_caret_index_at_pos(0, x);
                    if caret_index == self.caret_index {
                        return true;
                    }
                    if let Some(selection_index) = self.selection_index {
                        if caret_index == selection_index {
                            self.selection_index = None;
                        }
                    } else {
                        self.selection_index = Some(self.caret_index);
                    }
                    self.caret_index = caret_index;
                    self.update_carret(this, ctx, true);
                }
            }
        }
        true
    }

    fn on_keyboard_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {
        self.on_focus = focus;
        self.update_carret(this, ctx, true);
        ctx.send_event(event::Redraw);
    }

    fn on_keyboard_event(&mut self, event: KeyboardEvent, this: Id, ctx: &mut Context) -> bool {
        match event {
            KeyboardEvent::Char(ch) => {
                if self.selection_index.is_some() {
                    self.delete_selection(this, ctx);
                }
                self.insert_char(ch, this, ctx);
            }
            KeyboardEvent::Pressed(key_code) => match key_code {
                VirtualKeyCode::C | VirtualKeyCode::X => {
                    if ctx.modifiers().ctrl() {
                        if let Some(selection_index) = self.selection_index {
                            let a = self.text_info.get_indice(selection_index);
                            let b = self.text_info.get_indice(self.caret_index);
                            let range = if a < b { a..b } else { b..a };
                            let mut cliptobard = ClipboardContext::new().unwrap();
                            let _ = cliptobard.set_contents(self.text[range].to_owned());
                            if key_code == VirtualKeyCode::X {
                                self.delete_selection(this, ctx);
                            }
                        }
                    }
                }
                VirtualKeyCode::V => {
                    if ctx.modifiers().ctrl() {
                        let mut clipboard = ClipboardContext::new().unwrap();
                        if let Ok(text) = clipboard.get_contents() {
                            let text = text.replace(|x: char| x.is_control(), "");
                            let indice = self.text_info.get_indice(self.caret_index);
                            if let Some(selection_index) = self.selection_index {
                                let a = self.text_info.get_indice(selection_index);
                                let b = indice;
                                let range = if a < b { a..b } else { b..a };
                                self.text.replace_range(range.clone(), &text);
                                self.selection_index = None;
                                self.update_text(this, ctx); // TODO: is is not working?
                                self.caret_index =
                                    self.text_info.get_caret_index(range.start + text.len());
                            } else {
                                self.text.insert_str(indice, &text);
                                self.update_text(this, ctx);
                                self.caret_index =
                                    self.text_info.get_caret_index(indice + text.len());
                            }
                            self.update_carret(this, ctx, true);
                        }
                    }
                }
                VirtualKeyCode::A => {
                    if ctx.modifiers().ctrl() {
                        let start = 0;
                        let end = self
                            .text_info
                            .get_line_range(self.caret_index)
                            .map_or(0, |x| x.end.saturating_sub(1));
                        self.selection_index = Some(start);
                        self.caret_index = end;
                        self.update_carret(this, ctx, false);
                    }
                }
                VirtualKeyCode::Return => {
                    ctx.send_event(event::SubmitText {
                        id: this,
                        text: self.text.clone(),
                    });
                }
                VirtualKeyCode::Back | VirtualKeyCode::Delete if self.selection_index.is_some() => {
                    self.delete_selection(this, ctx);
                }
                VirtualKeyCode::Back => {
                    if self.caret_index == 0 {
                        return true;
                    }
                    self.caret_index -= 1;
                    self.text
                        .remove(self.text_info.get_indice(self.caret_index));
                    self.update_text(this, ctx);
                }
                VirtualKeyCode::Delete => {
                    if self.caret_index + 1 < self.text_info.len() {
                        self.text
                            .remove(self.text_info.get_indice(self.caret_index));
                        self.update_text(this, ctx);
                    }
                }
                VirtualKeyCode::Left => {
                    if self.caret_index == 0 {
                        self.move_caret(0, ctx);
                    } else if ctx.modifiers().ctrl() {
                        let mut caret = self.caret_index - 1;
                        let mut s = false;
                        while caret != 0 {
                            let whitespace = match self.text[self.text_info.get_indice(caret)..]
                                .chars()
                                .next()
                            {
                                Some(x) => x.is_whitespace(),
                                None => false,
                            };
                            if !whitespace {
                                s = true;
                            } else if s {
                                caret += 1;
                                break;
                            }
                            caret -= 1;
                        }
                        self.move_caret(caret, ctx);
                    } else {
                        self.move_caret(self.caret_index - 1, ctx);
                    }
                    self.update_carret(this, ctx, true);
                }
                VirtualKeyCode::Right => {
                    if self.caret_index + 1 >= self.text_info.len() {
                        self.move_caret(self.caret_index, ctx);
                    } else if ctx.modifiers().ctrl() {
                        let mut caret = self.caret_index;
                        let mut s = false;
                        loop {
                            let whitespace = match self.text[self.text_info.get_indice(caret)..]
                                .chars()
                                .next()
                            {
                                Some(x) => x.is_whitespace(),
                                None => {
                                    caret = self.text_info.len() - 1;
                                    break;
                                }
                            };
                            if whitespace {
                                s = true;
                            } else if s {
                                break;
                            }
                            caret += 1;
                        }
                        self.move_caret(caret, ctx);
                    } else {
                        self.move_caret(self.caret_index + 1, ctx);
                    }
                    self.update_carret(this, ctx, true);
                }
                VirtualKeyCode::Home => {
                    self.move_caret(0, ctx);
                    self.update_carret(this, ctx, true);
                }
                VirtualKeyCode::End => {
                    self.move_caret(
                        self.text_info
                            .get_line_range(self.caret_index)
                            .map_or(0, |x| x.end.saturating_sub(1)),
                        ctx,
                    );
                    self.update_carret(this, ctx, true);
                }
                _ => {}
            },
        }
        true
    }
}
