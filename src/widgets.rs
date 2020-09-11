use crate::{
    event, render::Graphic, text::TextInfo, Behaviour, EventHandler, Id, InputFlags, KeyboardEvent,
    Layout, MouseEvent, Rect, Widgets,
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
    fn input_flags(&self) -> InputFlags {
        InputFlags::POINTER
    }

    fn on_start(&mut self, this: Id, widgets: &mut Widgets, event_handler: &mut EventHandler) {
        let graphic = widgets.get_graphic(this).unwrap();
        graphic.set_color([200, 200, 200, 255]);
        event_handler.send_event(event::Redraw);
    }

    fn on_mouse_event(
        &mut self,
        event: MouseEvent,
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        match event {
            MouseEvent::Enter => {
                let graphic = widgets.get_graphic(this).unwrap();
                graphic.set_color([180, 180, 180, 255]);
                event_handler.send_event(event::Redraw);
            }
            MouseEvent::Exit => {
                self.click = false;
                let graphic = widgets.get_graphic(this).unwrap();
                graphic.set_color([200, 200, 200, 255]);
                event_handler.send_event(event::Redraw);
            }
            MouseEvent::Down => {
                self.click = true;
                let graphic = widgets.get_graphic(this).unwrap();
                graphic.set_color([128, 128, 128, 255]);
                event_handler.send_event(event::Redraw);
            }
            MouseEvent::Up => {
                let graphic = widgets.get_graphic(this).unwrap();
                graphic.set_color([180, 180, 180, 255]);
                event_handler.send_event(event::Redraw);
                if self.click {
                    event_handler.send_event(event::ButtonClicked { id: this });
                }
            }
            MouseEvent::Moved { .. } => {}
        }
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

    fn compute_value(&mut self, rect: &Rect) {
        let mut rel_x = rect.get_relative_x(self.mouse_x);
        rel_x = rel_x.max(0.0).min(1.0);
        let value = rel_x * (self.max_value - self.min_value) + self.min_value;
        self.value = value;
    }

    fn set_handle_pos(
        &mut self,
        handle: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        let rect = widgets.get_rect(handle);
        let mut rel_x = (self.value - self.min_value) / (self.max_value - self.min_value);
        rel_x = rel_x.max(0.0).min(1.0);
        rect.anchors[0] = rel_x;
        rect.anchors[2] = rel_x;
        event_handler.send_event(event::Redraw);
        event_handler.send_event(event::InvalidadeLayout { id: handle });
    }
}
impl Behaviour for Slider {
    fn input_flags(&self) -> InputFlags {
        InputFlags::POINTER
    }

    fn on_start(&mut self, this: Id, widgets: &mut Widgets, event_handler: &mut EventHandler) {
        self.set_handle_pos(self.handle, widgets, event_handler);
        let value = self.value;
        event_handler.send_event(event::ValueSet { id: this, value });
    }

    fn on_mouse_event(
        &mut self,
        event: MouseEvent,
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        match event {
            MouseEvent::Enter => {}
            MouseEvent::Exit => {}
            MouseEvent::Down => {
                self.dragging = true;
                event_handler.send_event(event::LockOver);
                self.compute_value(widgets.get_rect(self.slide_area));
                self.set_handle_pos(self.handle, widgets, event_handler);
                let value = self.value;
                event_handler.send_event(event::ValueChanged { id: this, value });
            }
            MouseEvent::Up => {
                self.dragging = false;
                self.set_handle_pos(self.handle, widgets, event_handler);
                let value = self.value;
                event_handler.send_event(event::ValueSet { id: this, value });
                event_handler.send_event(event::UnlockOver);
            }
            MouseEvent::Moved { x, .. } => {
                self.mouse_x = x;
                if self.dragging {
                    self.compute_value(widgets.get_rect(self.slide_area));
                    self.set_handle_pos(self.handle, widgets, event_handler);
                    let value = self.value;
                    event_handler.send_event(event::ValueChanged { id: this, value });
                }
            }
        }
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
    fn input_flags(&self) -> InputFlags {
        InputFlags::POINTER
    }

    fn on_start(&mut self, this: Id, widgets: &mut Widgets, event_handler: &mut EventHandler) {
        event_handler.send_event(event::ToggleChanged {
            id: this,
            value: self.enable,
        });
        if self.enable {
            widgets.get_graphic(self.marker).unwrap().set_alpha(255)
        } else {
            widgets.get_graphic(self.marker).unwrap().set_alpha(0)
        }
    }
    fn on_mouse_event(
        &mut self,
        event: MouseEvent,
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        match event {
            MouseEvent::Enter => {
                let graphic = widgets.get_graphic(self.background).unwrap();
                graphic.set_color([190, 190, 190, 255]);
                event_handler.send_event(event::Redraw);
            }
            MouseEvent::Exit => {
                self.click = false;
                let graphic = widgets.get_graphic(self.background).unwrap();
                graphic.set_color([200, 200, 200, 255]);
                event_handler.send_event(event::Redraw);
            }
            MouseEvent::Down => {
                self.click = true;
                let graphic = widgets.get_graphic(self.background).unwrap();
                graphic.set_color([170, 170, 170, 255]);
                event_handler.send_event(event::Redraw);
            }
            MouseEvent::Up => {
                let graphic = widgets.get_graphic(self.background).unwrap();
                graphic.set_color([190, 190, 190, 255]);
                event_handler.send_event(event::Redraw);
                if self.click {
                    self.enable = !self.enable;
                    event_handler.send_event(event::ToggleChanged {
                        id: this,
                        value: self.enable,
                    });
                    if self.enable {
                        widgets.get_graphic(self.marker).unwrap().set_alpha(255)
                    } else {
                        widgets.get_graphic(self.marker).unwrap().set_alpha(0)
                    }
                }
            }
            MouseEvent::Moved { .. } => {}
        }
    }
}

pub struct Unselected;
pub struct Selected;
pub struct Select(Id);

pub struct TabGroup {
    buttons: Vec<Id>,
    pages: Vec<Id>,
    selected: usize,
}
impl TabGroup {
    pub fn new(buttons: Vec<Id>, pages: Vec<Id>) -> Self {
        assert_eq!(
            buttons.len(),
            pages.len(),
            "buttons len need be equal to pages len"
        );
        Self {
            buttons,
            pages,
            selected: 0,
        }
    }
}
impl Behaviour for TabGroup {
    fn input_flags(&self) -> InputFlags {
        InputFlags::empty()
    }

    fn on_event(
        &mut self,
        event: &dyn Any,
        _this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        if let Some(Select(id)) = event.downcast_ref::<Select>() {
            widgets.deactive(self.pages[self.selected]);
            event_handler.send_event_to(self.buttons[self.selected], Unselected);
            if let Some(i) = self.buttons.iter().position(|x| x == id) {
                self.selected = i;
                widgets.active(self.pages[self.selected]);
                event_handler.send_event_to(self.buttons[self.selected], Selected);
            }
        }
    }

    fn on_start(&mut self, _this: Id, widgets: &mut Widgets, event_handler: &mut EventHandler) {
        for i in 0..self.pages.len() {
            if i == self.selected {
                widgets.active(self.pages[i]);
                event_handler.send_event_to(self.buttons[i], Selected);
            } else {
                widgets.deactive(self.pages[i]);
                event_handler.send_event_to(self.buttons[i], Unselected);
            }
        }
    }
}

pub struct TabButton {
    tab_group: Id,
    selected: bool,
    click: bool,
}
impl TabButton {
    pub fn new(tab_group: Id) -> Self {
        Self {
            tab_group,
            selected: false,
            click: false,
        }
    }

    pub fn unselect(&mut self, this: Id, widgets: &mut Widgets, event_handler: &mut EventHandler) {
        self.selected = false;
        let graphic = widgets.get_graphic(this).unwrap();
        graphic.set_color([200, 200, 200, 255]);
        event_handler.send_event(event::Redraw);
    }
}
impl Behaviour for TabButton {
    fn input_flags(&self) -> InputFlags {
        InputFlags::POINTER
    }

    fn on_event(
        &mut self,
        event: &dyn Any,
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        if event.is::<Unselected>() {
            let graphic = widgets.get_graphic(this).unwrap();
            graphic.set_color([200, 200, 200, 255]);
            event_handler.send_event(event::Redraw);
            self.selected = false;
        } else if event.is::<Selected>() {
            let graphic = widgets.get_graphic(this).unwrap();
            graphic.set_color([255, 255, 255, 255]);
            event_handler.send_event(event::Redraw);
            self.selected = true;
        }
    }

    fn on_mouse_event(
        &mut self,
        event: MouseEvent,
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        match event {
            MouseEvent::Enter => {
                if !self.selected {
                    let graphic = widgets.get_graphic(this).unwrap();
                    graphic.set_color([180, 180, 180, 255]);
                    event_handler.send_event(event::Redraw);
                }
            }
            MouseEvent::Exit => {
                if !self.selected {
                    self.click = false;
                    let graphic = widgets.get_graphic(this).unwrap();
                    graphic.set_color([200, 200, 200, 255]);
                    event_handler.send_event(event::Redraw);
                }
            }
            MouseEvent::Down => {
                if !self.selected {
                    self.click = true;
                    let graphic = widgets.get_graphic(this).unwrap();
                    graphic.set_color([128, 128, 128, 255]);
                    event_handler.send_event(event::Redraw);
                }
            }
            MouseEvent::Up => {
                if !self.selected {
                    let graphic = widgets.get_graphic(this).unwrap();

                    if self.click {
                        event_handler.send_event_to(self.tab_group, Select(this));
                    } else {
                        graphic.set_color([180, 180, 180, 255]);
                    }
                    event_handler.send_event(event::Redraw);
                }
            }
            MouseEvent::Moved { .. } => {}
        }
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
    fn input_flags(&self) -> InputFlags {
        InputFlags::POINTER
    }

    fn on_start(&mut self, _this: Id, widgets: &mut Widgets, _event_handler: &mut EventHandler) {
        widgets.deactive(self.hover);
    }

    fn on_mouse_event(
        &mut self,
        event: MouseEvent,
        _this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        match event {
            MouseEvent::Enter => {
                widgets.active(self.hover);
                widgets
                    .get_graphic(self.label)
                    .unwrap()
                    .set_text(&self.text);
                widgets.move_to_front(self.hover);
                self.is_over = true;
                event_handler.send_event(event::InvalidadeLayout { id: self.hover });
                event_handler.send_event(event::Redraw);
            }
            MouseEvent::Exit => {
                widgets.deactive(self.hover);
                self.is_over = false;
                event_handler.send_event(event::Redraw);
            }
            MouseEvent::Down => {}
            MouseEvent::Up => {}
            MouseEvent::Moved { x, y } => {
                if self.is_over {
                    // TODO: this may be buggy, if the layout has not updated yet
                    let (width, heigth) = widgets.get_rect(crate::ROOT_ID).get_size();
                    let rect = widgets.get_rect(self.hover);
                    let x = x / width;
                    let y = y / heigth;
                    rect.anchors = [x, y, x, y];
                    event_handler.send_event(event::InvalidadeLayout { id: self.hover });
                    event_handler.send_event(event::Redraw);
                }
            }
        }
    }
}

struct ScrollViewLayout {
    view: Id,
    h_scroll_bar: Id,
    v_scroll_bar: Id,
}
impl Layout for ScrollViewLayout {
    fn compute_min_size(&mut self, this: Id, widgets: &mut Widgets) {
        let mut min_size = widgets.get_rect(self.view).min_size;

        let h_scroll_bar_size = widgets.get_rect(self.v_scroll_bar).min_size;
        let v_scroll_bar_size = widgets.get_rect(self.v_scroll_bar).min_size;

        min_size[0] = min_size[0].max(h_scroll_bar_size[0]);
        min_size[1] = min_size[1].max(v_scroll_bar_size[1]);

        min_size[0] += v_scroll_bar_size[0];
        min_size[1] += h_scroll_bar_size[1];

        widgets.get_rect(this).min_size = min_size;
    }

    fn update_layouts(&mut self, this: Id, widgets: &mut Widgets) {
        let this_rect = *widgets.get_rect(this).get_rect();
        let content = widgets.get_children(self.view)[0];
        let content_size = widgets.get_rect(content).min_size;
        let view_rect = *widgets.get_rect(this).get_rect();
        let view_width = view_rect[2] - view_rect[0];
        let view_height = view_rect[3] - view_rect[1];

        let mut h_active = view_width < content_size[0];
        let mut h_scroll_bar_size = if h_active {
            widgets.get_rect(self.h_scroll_bar).min_size[1]
        } else {
            0.0
        };

        let v_active = view_height - h_scroll_bar_size < content_size[1];
        let v_scroll_bar_size = if v_active {
            widgets.get_rect(self.v_scroll_bar).min_size[0]
        } else {
            0.0
        };

        if !h_active && view_width - v_scroll_bar_size < content_size[0] {
            h_active = true;
            h_scroll_bar_size = widgets.get_rect(self.h_scroll_bar).min_size[1];
        }

        if widgets.is_active(self.h_scroll_bar) {
            if !h_active {
                widgets.deactive(self.h_scroll_bar);
            }
        } else if h_active {
            widgets.active(self.h_scroll_bar);
        }

        if widgets.is_active(self.v_scroll_bar) {
            if !v_active {
                widgets.deactive(self.v_scroll_bar);
            }
        } else if v_active {
            widgets.active(self.v_scroll_bar);
        }

        if h_active {
            let h_scroll_bar_rect = widgets.get_rect(self.h_scroll_bar);
            h_scroll_bar_rect.set_designed_rect([
                this_rect[0],
                this_rect[3] - h_scroll_bar_size,
                this_rect[2] - v_scroll_bar_size,
                this_rect[3],
            ]);
        }
        if v_active {
            let v_scroll_bar_rect = widgets.get_rect(self.v_scroll_bar);
            v_scroll_bar_rect.set_designed_rect([
                this_rect[2] - v_scroll_bar_size,
                this_rect[1],
                this_rect[2],
                this_rect[3] - h_scroll_bar_size,
            ]);
        }

        widgets.get_rect(self.view).set_designed_rect([
            this_rect[0],
            this_rect[1],
            this_rect[2] - v_scroll_bar_size,
            this_rect[3] - h_scroll_bar_size,
        ])
    }
}

struct ScrollContentLayout {
    delta_x: f32,
    delta_y: f32,
    h_scroll_bar_handle: Id,
    v_scroll_bar_handle: Id,
}
impl Layout for ScrollContentLayout {
    fn compute_min_size(&mut self, _: Id, _: &mut Widgets) {}

    fn update_layouts(&mut self, this: Id, widgets: &mut Widgets) {
        debug_assert!(
            widgets.get_children(this).len() == 1,
            "The view of the scroll view must have only one child, wich is the content."
        );

        let content = widgets.get_children(this)[0];
        let content_size = widgets.get_rect(content).min_size;
        let view_rect = *widgets.get_rect(this).get_rect();
        let view_width = view_rect[2] - view_rect[0];
        let view_height = view_rect[3] - view_rect[1];

        let mut content_rect = [0.0; 4];

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
            content_rect[0] = view_rect[0];
            content_rect[2] = view_rect[0] + view_width;
        } else {
            content_rect[0] = view_rect[0] - self.delta_x;
            content_rect[2] = view_rect[0] - self.delta_x + content_size[0];
        }

        if content_size[1] < view_height {
            content_rect[1] = view_rect[1];
            content_rect[3] = view_rect[3] + view_height;
        } else {
            content_rect[1] = view_rect[1] - self.delta_y;
            content_rect[3] = view_rect[1] - self.delta_y + content_size[1];
        }

        let h_scroll_bar_handle = widgets.get_rect(self.h_scroll_bar_handle);
        h_scroll_bar_handle.anchors[0] = self.delta_x / content_size[0];
        h_scroll_bar_handle.anchors[2] = ((self.delta_x + view_width) / content_size[0]).min(1.0);

        let v_scroll_bar_handle = widgets.get_rect(self.v_scroll_bar_handle);
        v_scroll_bar_handle.anchors[1] = self.delta_y / content_size[1];
        v_scroll_bar_handle.anchors[3] = ((self.delta_y + view_height) / content_size[1]).min(1.0);

        widgets.get_rect(content).set_rect(content_rect);
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
    fn input_flags(&self) -> InputFlags {
        InputFlags::POINTER
    }

    fn on_mouse_event(
        &mut self,
        event: MouseEvent,
        _this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        match event {
            MouseEvent::Enter => {}
            MouseEvent::Exit => {
                if let Some(handle) = widgets.get_graphic(self.handle) {
                    handle.set_color([220, 220, 220, 255]);
                    event_handler.send_event(event::Redraw);
                }
            }
            MouseEvent::Down => {
                self.dragging = true;
                if let Some(handle) = widgets.get_graphic(self.handle) {
                    handle.set_color([180, 180, 180, 255]);
                    event_handler.send_event(event::Redraw);
                }
                event_handler.send_event(event::LockOver);
                let handle_rect = *widgets.get_rect(self.handle).get_rect();
                let area = widgets
                    .get_parent(self.handle)
                    .expect("the handle of the scrollbar must have a parent");
                let area_rect = *widgets.get_rect(area).get_rect();
                self.drag_start = self.mouse_pos;
                if !self.vertical {
                    let handle_size = handle_rect[2] - handle_rect[0];
                    let area_size = area_rect[2] - area_rect[0] - handle_size;
                    if self.mouse_pos < handle_rect[0] || self.mouse_pos > handle_rect[2] {
                        self.curr_value =
                            (self.mouse_pos - (area_rect[0] + handle_size / 2.0)) / area_size;
                        event_handler.send_event_to(
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
                        event_handler.send_event_to(
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
                    event_handler.send_event(event::UnlockOver);
                    if let Some(graphic) = widgets.get_graphic(self.handle) {
                        graphic.set_color([200, 200, 200, 255]);
                        event_handler.send_event(event::Redraw);
                    }
                }
            }
            MouseEvent::Moved { x, y } => {
                self.mouse_pos = if self.vertical { y } else { x };
                if self.dragging {
                    let handle_rect = *widgets.get_rect(self.handle).get_rect();
                    let area = widgets
                        .get_parent(self.handle)
                        .expect("handle must have a parent");
                    let area_rect = *widgets.get_rect(area).get_rect();

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

                    event_handler.send_event_to(
                        self.scroll_view,
                        SetScrollPosition {
                            vertical: self.vertical,
                            value,
                        },
                    )
                } else if widgets.get_graphic(self.handle).is_some() {
                    let handle_rect = *widgets.get_rect(self.handle).get_rect();
                    let graphic = widgets.get_graphic(self.handle).unwrap();
                    if self.mouse_pos < handle_rect[1] || self.mouse_pos > handle_rect[3] {
                        graphic.set_color([220, 220, 220, 255]);
                    } else {
                        graphic.set_color([200, 200, 200, 255]);
                    }
                    event_handler.send_event(event::Redraw);
                }
            }
        }
    }
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
    fn input_flags(&self) -> InputFlags {
        InputFlags::SCROLL
    }

    fn on_start(&mut self, this: Id, widgets: &mut Widgets, _: &mut EventHandler) {
        let scroll_content_layout = ScrollContentLayout {
            delta_x: self.delta_x,
            delta_y: self.delta_y,
            v_scroll_bar_handle: self.v_scroll_bar_handle,
            h_scroll_bar_handle: self.h_scroll_bar_handle,
        };
        widgets.move_to_front(self.v_scroll_bar);
        widgets.move_to_front(self.h_scroll_bar);
        widgets.set_layout(self.view, Box::new(scroll_content_layout));

        let scroll_view_layout = ScrollViewLayout {
            view: self.view,
            h_scroll_bar: self.h_scroll_bar,
            v_scroll_bar: self.v_scroll_bar,
        };
        widgets.set_layout(this, Box::new(scroll_view_layout));
    }

    fn on_active(&mut self, _: Id, widgets: &mut Widgets, event_handler: &mut EventHandler) {
        let content_size = widgets.get_rect(self.content).min_size;

        let view = widgets.get_rect(self.view);
        let view_rect = view.get_rect();

        let width = view_rect[2] - view_rect[0];
        let height = view_rect[3] - view_rect[1];

        let view_size = [content_size[0].max(width), content_size[1].max(height)];

        //TODO: this can be removed i guess
        let v_scroll_bar_handle = widgets.get_rect(self.v_scroll_bar_handle);
        v_scroll_bar_handle.anchors[1] = self.delta_y / view_size[1];
        v_scroll_bar_handle.anchors[3] = (self.delta_y + height) / view_size[1];

        event_handler.send_event(event::InvalidadeLayout {
            id: self.h_scroll_bar_handle,
        });
    }

    fn on_scroll_event(
        &mut self,
        delta: [f32; 2],
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        let scroll_view_layout = widgets
            .get_layout(self.view)
            .unwrap()
            .downcast_mut::<ScrollContentLayout>()
            .unwrap();
        scroll_view_layout.delta_x += delta[0];
        scroll_view_layout.delta_y -= delta[1];

        event_handler.send_event(event::InvalidadeLayout { id: this });
        event_handler.send_event(event::Redraw);
    }

    fn on_event(
        &mut self,
        event: &dyn Any,
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        if let Some(event) = event.downcast_ref::<SetScrollPosition>() {
            if !event.vertical {
                let total_size = widgets.get_rect(self.content).get_width()
                    - widgets.get_rect(self.view).get_width();
                let scroll_view_layout = widgets
                    .get_layout(self.view)
                    .unwrap()
                    .downcast_mut::<ScrollContentLayout>()
                    .unwrap();
                scroll_view_layout.delta_x = event.value * total_size;
            } else {
                let total_size = widgets.get_rect(self.content).get_height()
                    - widgets.get_rect(self.view).get_height();
                let scroll_view_layout = widgets
                    .get_layout(self.view)
                    .unwrap()
                    .downcast_mut::<ScrollContentLayout>()
                    .unwrap();
                scroll_view_layout.delta_y = event.value * total_size;
            }

            event_handler.send_event(event::InvalidadeLayout { id: this });
            event_handler.send_event(event::Redraw);
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
    fn update_text(&mut self, this: Id, widgets: &mut Widgets, event_handler: &mut EventHandler) {
        let fonts = widgets.get_fonts();
        if let Some((ref mut rect, Graphic::Text(text))) = widgets.get_rect_and_graphic(self.label)
        {
            let display_text = self.text.clone();
            text.set_text(&display_text);
            let min_size = text.compute_min_size(fonts).unwrap_or([0.0, 0.0]);
            self.text_width = min_size[0];
            rect.set_min_size(min_size);
            self.text_info = text.get_text_info(fonts, rect).clone();
            self.update_carret(this, widgets, event_handler, true);
        }
    }

    fn update_carret(
        &mut self,
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
        focus_caret: bool,
    ) {
        let this_rect = *widgets.get_rect(this).get_rect();

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

        widgets.get_rect(self.label).margins[0] = -self.x_scroll;

        caret_pos[0] -= self.x_scroll;

        if let Some(selection_index) = self.selection_index {
            widgets
                .get_graphic(self.caret)
                .unwrap()
                .set_color([51, 153, 255, 255]);
            let mut selection_pos = self.text_info.get_caret_pos(selection_index);
            selection_pos[0] -= self.x_scroll;
            let margins = &mut widgets.get_rect(self.caret).margins;
            *margins = [
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
        } else {
            widgets
                .get_graphic(self.caret)
                .unwrap()
                .set_color([0, 0, 0, 255]);
            if self.on_focus {
                widgets.get_rect(self.caret).margins = [
                    caret_pos[0],
                    caret_pos[1] - self.text_info.get_line_heigth(),
                    caret_pos[0] + 1.0,
                    caret_pos[1],
                ];
            } else {
                widgets.get_rect(self.caret).margins = [0.0, 0.0, 0.0, 0.0];
            }
        }
        event_handler.send_event(event::Redraw);
        event_handler.send_event(event::InvalidadeLayout { id: this });
    }

    fn move_caret(&mut self, caret: usize, widgets: &mut Widgets) {
        if widgets.modifiers().shift() {
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

    fn delete_selection(
        &mut self,
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        let selection_index = self.selection_index.unwrap();
        let a = self.text_info.get_indice(self.caret_index);
        let b = self.text_info.get_indice(selection_index);
        let range = if a > b { b..a } else { a..b };
        if self.caret_index > selection_index {
            self.caret_index = selection_index;
        }
        self.selection_index = None;
        self.text.replace_range(range, "");
        self.update_text(this, widgets, event_handler);
    }

    fn insert_char(
        &mut self,
        ch: char,
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        self.text
            .insert(self.text_info.get_indice(self.caret_index), ch);
        self.caret_index += 1;
        self.update_text(this, widgets, event_handler);
    }
}
impl Behaviour for TextField {
    fn input_flags(&self) -> InputFlags {
        InputFlags::KEYBOARD | InputFlags::POINTER | InputFlags::SCROLL
    }

    fn on_start(&mut self, this: Id, widgets: &mut Widgets, event_handler: &mut EventHandler) {
        self.update_text(this, widgets, event_handler);
        widgets.move_to_front(self.label);
    }

    fn on_event(
        &mut self,
        event: &dyn Any,
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        if event.is::<event::ClearText>() {
            self.text.clear();
            self.caret_index = 0;
            self.selection_index = None;
            self.update_text(this, widgets, event_handler);
        }
    }

    fn on_scroll_event(
        &mut self,
        delta: [f32; 2],
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        let delta = if delta[0].abs() > delta[1].abs() {
            delta[0]
        } else {
            delta[1]
        };
        self.x_scroll -= delta;
        self.update_carret(this, widgets, event_handler, false);
    }

    fn on_mouse_event(
        &mut self,
        event: MouseEvent,
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        match event {
            MouseEvent::Enter => {}
            MouseEvent::Exit => {}
            MouseEvent::Down => {
                if !self.on_focus {
                    event_handler.send_event(event::RequestKeyboardFocus { id: this });
                }
                let left = widgets.get_rect(this).get_rect()[0] - self.x_scroll;
                let x = self.mouse_x - left;
                self.caret_index = self.text_info.get_caret_index_at_pos(0, x);
                self.mouse_down = true;
                self.selection_index = None;
                self.update_carret(this, widgets, event_handler, true);
                event_handler.send_event(event::LockOver);
            }
            MouseEvent::Up => {
                self.mouse_down = false;
                event_handler.send_event(event::UnlockOver);
            }
            MouseEvent::Moved { x, .. } => {
                self.mouse_x = x;
                if self.mouse_down {
                    let left = widgets.get_rect(this).get_rect()[0] - self.x_scroll;
                    let x = self.mouse_x - left;
                    let caret_index = self.text_info.get_caret_index_at_pos(0, x);
                    if caret_index == self.caret_index {
                        return;
                    }
                    if let Some(selection_index) = self.selection_index {
                        if caret_index == selection_index {
                            self.selection_index = None;
                        }
                    } else {
                        self.selection_index = Some(self.caret_index);
                    }
                    self.caret_index = caret_index;
                    self.update_carret(this, widgets, event_handler, true);
                }
            }
        }
    }

    fn on_keyboard_focus_change(
        &mut self,
        focus: bool,
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        self.on_focus = focus;
        self.update_carret(this, widgets, event_handler, true);
        event_handler.send_event(event::InvalidadeLayout { id: this });
        event_handler.send_event(event::Redraw);
    }

    fn on_keyboard_event(
        &mut self,
        event: KeyboardEvent,
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
        match event {
            KeyboardEvent::Char(ch) => {
                if self.selection_index.is_some() {
                    self.delete_selection(this, widgets, event_handler);
                }
                self.insert_char(ch, this, widgets, event_handler);
                println!("receive char {:?}", ch);
            }
            KeyboardEvent::Pressed(key_code) => match key_code {
                VirtualKeyCode::C | VirtualKeyCode::X => {
                    if widgets.modifiers().ctrl() {
                        if let Some(selection_index) = self.selection_index {
                            let a = self.text_info.get_indice(selection_index);
                            let b = self.text_info.get_indice(self.caret_index);
                            let range = if a < b { a..b } else { b..a };
                            let mut ctx = ClipboardContext::new().unwrap();
                            let _ = ctx.set_contents(self.text[range].to_owned());
                            if key_code == VirtualKeyCode::X {
                                self.delete_selection(this, widgets, event_handler);
                            }
                        }
                    }
                }
                VirtualKeyCode::V => {
                    if widgets.modifiers().ctrl() {
                        let mut ctx = ClipboardContext::new().unwrap();
                        if let Ok(text) = ctx.get_contents() {
                            let text = text.replace(|x: char| x.is_control(), "");
                            let indice = self.text_info.get_indice(self.caret_index);
                            if let Some(selection_index) = self.selection_index {
                                let a = self.text_info.get_indice(selection_index);
                                let b = indice;
                                let range = if a < b { a..b } else { b..a };
                                self.text.replace_range(range.clone(), &text);
                                self.selection_index = None;
                                self.update_text(this, widgets, event_handler); // TODO: is is not working?
                                self.caret_index =
                                    self.text_info.get_caret_index(range.start + text.len());
                            } else {
                                self.text.insert_str(indice, &text);
                                self.update_text(this, widgets, event_handler);
                                self.caret_index =
                                    self.text_info.get_caret_index(indice + text.len());
                            }
                            self.update_carret(this, widgets, event_handler, true);
                        }
                    }
                }
                VirtualKeyCode::A => {
                    if widgets.modifiers().ctrl() {
                        let start = 0;
                        let end = self
                            .text_info
                            .get_line_range(self.caret_index)
                            .map_or(0, |x| x.end.saturating_sub(1));
                        self.selection_index = Some(start);
                        self.caret_index = end;
                        self.update_carret(this, widgets, event_handler, false);
                    }
                }
                VirtualKeyCode::Return => {
                    event_handler.send_event(event::SubmitText {
                        id: this,
                        text: self.text.clone(),
                    });
                }
                VirtualKeyCode::Back | VirtualKeyCode::Delete if self.selection_index.is_some() => {
                    self.delete_selection(this, widgets, event_handler);
                }
                VirtualKeyCode::Back => {
                    if self.caret_index == 0 {
                        return;
                    }
                    self.caret_index -= 1;
                    self.text
                        .remove(self.text_info.get_indice(self.caret_index));
                    self.update_text(this, widgets, event_handler);
                }
                VirtualKeyCode::Delete => {
                    if self.caret_index + 1 < self.text_info.len() {
                        self.text
                            .remove(self.text_info.get_indice(self.caret_index));
                        self.update_text(this, widgets, event_handler);
                    }
                }
                VirtualKeyCode::Left => {
                    if self.caret_index == 0 {
                        self.move_caret(0, widgets);
                    } else if widgets.modifiers().ctrl() {
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
                        self.move_caret(caret, widgets);
                    } else {
                        self.move_caret(self.caret_index - 1, widgets);
                    }
                    self.update_carret(this, widgets, event_handler, true);
                }
                VirtualKeyCode::Right => {
                    if self.caret_index + 1 >= self.text_info.len() {
                        self.move_caret(self.caret_index, widgets);
                    } else if widgets.modifiers().ctrl() {
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
                        self.move_caret(caret, widgets);
                    } else {
                        self.move_caret(self.caret_index + 1, widgets);
                    }
                    self.update_carret(this, widgets, event_handler, true);
                }
                VirtualKeyCode::Home => {
                    self.move_caret(0, widgets);
                    self.update_carret(this, widgets, event_handler, true);
                }
                VirtualKeyCode::End => {
                    self.move_caret(
                        self.text_info
                            .get_line_range(self.caret_index)
                            .map_or(0, |x| x.end.saturating_sub(1)),
                        widgets,
                    );
                    self.update_carret(this, widgets, event_handler, true);
                }
                _ => {}
            },
        }
    }
}
