use crate::{event, Behaviour, EventHandler, Id, MouseEvent, Rect, Widgets};

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
    fn on_mouse_event(
        &mut self,
        event: MouseEvent,
        this: Id,
        mut widgets: Widgets,
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
    slide_area: Id,
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
    fn on_start(&mut self, this: Id, mut widgets: Widgets, event_handler: &mut EventHandler) {
        self.set_handle_pos(widgets.get_rect(self.handle), event_handler);
        let value = self.value;
        event_handler.send_event(event::ValueSet { id: this, value });
    }

    fn on_mouse_event(
        &mut self,
        event: MouseEvent,
        this: Id,
        mut widgets: Widgets,
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
    fn on_start(&mut self, this: Id, mut widgets: Widgets, event_handler: &mut EventHandler) {
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
        mut widgets: Widgets,
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
