use crate::{event, Behaviour, EventHandler, Id, MouseEvent, Rect, Widgets};
use std::any::Any;

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
    fn listen_mouse(&self) -> bool {
        true
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

    fn set_handle_pos(&mut self, rect: &mut Rect, event_handler: &mut EventHandler) {
        let mut rel_x = (self.value - self.min_value) / (self.max_value - self.min_value);
        rel_x = rel_x.max(0.0).min(1.0);
        rect.anchors[0] = rel_x;
        rect.anchors[2] = rel_x;
        event_handler.send_event(event::Redraw);
        event_handler.send_event(event::InvalidadeLayout);
    }
}
impl Behaviour for Slider {
    fn listen_mouse(&self) -> bool {
        true
    }

    fn on_start(&mut self, this: Id, widgets: &mut Widgets, event_handler: &mut EventHandler) {
        self.set_handle_pos(widgets.get_rect(self.handle), event_handler);
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
                self.set_handle_pos(widgets.get_rect(self.handle), event_handler);
                let value = self.value;
                event_handler.send_event(event::ValueChanged { id: this, value });
            }
            MouseEvent::Up => {
                self.dragging = false;
                self.set_handle_pos(widgets.get_rect(self.handle), event_handler);
                let value = self.value;
                event_handler.send_event(event::ValueSet { id: this, value });
                event_handler.send_event(event::UnlockOver);
            }
            MouseEvent::Moved { x, .. } => {
                self.mouse_x = x;
                if self.dragging {
                    self.compute_value(widgets.get_rect(self.slide_area));
                    self.set_handle_pos(widgets.get_rect(self.handle), event_handler);
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
    fn listen_mouse(&self) -> bool {
        true
    }

    fn on_start(&mut self, this: Id, widgets: &mut Widgets, event_handler: &mut EventHandler) {
        event_handler.send_event(event::ToogleChanged {
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
                    event_handler.send_event(event::ToogleChanged {
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
    fn listen_mouse(&self) -> bool {
        false
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
    fn listen_mouse(&self) -> bool {
        true
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
    fn listen_mouse(&self) -> bool {
        true
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
                widgets.get_graphic(self.label).unwrap().set_text(&self.text);
                widgets.move_to_front(self.hover);
                self.is_over = true;
            }
            MouseEvent::Exit => {
                widgets.deactive(self.hover);
                self.is_over = false;
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
                    event_handler.send_event(event::InvalidadeLayout);
                    event_handler.send_event(event::Redraw);
                }
            }
        }
    }
}
