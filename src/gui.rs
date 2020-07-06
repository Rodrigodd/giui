use crate::{render::GraphicId, GUIRender};
use std::any::{Any, TypeId};
use std::collections::VecDeque;
use winit::event::{ElementState, Event, WindowEvent};

pub mod event {
    use super::Id;
    pub struct Redraw;
    pub struct InvalidadeLayout;
    pub struct LockOver;
    pub struct UnlockOver;
    pub struct ButtonClicked {
        pub id: Id,
    }
    pub struct ValueChanged {
        pub id: Id,
        pub value: f32,
    }
    pub struct ValueSet {
        pub id: Id,
        pub value: f32,
    }

    pub struct ToogleChanged {
        pub id: Id,
        pub value: bool,
    }
}

pub const ROOT_ID: Id = Id { index: 0 };

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Id {
    index: usize,
}

#[derive(Clone, Copy, Debug)]
pub enum MouseEvent {
    Enter,
    Exit,
    Down,
    Up,
    Moved { x: f32, y: f32 },
}

#[derive(Default)]
pub struct Hierarchy {
    /// Map each Id to a vector of childs
    childs: Vec<Vec<Id>>,
    /// Map each Id to its parent
    parents: Vec<Option<Id>>,
    active: Vec<bool>,
}
impl Hierarchy {
    fn resize(&mut self, len: usize) {
        self.childs.resize_with(len, Default::default);
        self.parents.resize_with(len, Default::default);
        self.active.resize(len, true);
    }

    #[inline]
    fn get_childs(&self, id: Id) -> Vec<Id> {
        self.childs[id.index]
            .iter()
            .filter(|x| self.active[x.index])
            .cloned()
            .collect::<Vec<Id>>()
    }

    fn active(&mut self, id: Id) {
        self.active[id.index] = true;
    }

    fn deactive(&mut self, id: Id) {
        self.active[id.index] = false;
    }

    fn set_child(&mut self, parent: Id, child: Id) {
        self.childs[parent.index].push(child);
        if let Some(parent) = self.parents[child.index] {
            let pos = self.childs[parent.index]
                .iter()
                .position(|x| *x == child)
                .unwrap();
            self.childs[parent.index].remove(pos);
        }
        self.parents[child.index] = Some(parent);
    }
}

pub struct WidgetBuilder<'a, R: GUIRender> {
    gui: &'a mut GUI<R>,
    rect: Rect,
    graphic: Option<GraphicId>,
    behaviour: Option<Box<dyn Behaviour>>,
    parent: Option<Id>,
}
impl<'a, R: GUIRender> WidgetBuilder<'a, R> {
    fn new(gui: &'a mut GUI<R>) -> Self {
        Self {
            gui,
            rect: Rect::new([0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0]),
            graphic: None,
            behaviour: None,
            parent: None,
        }
    }
    pub fn with_anchors(mut self, anchors: [f32; 4]) -> Self {
        self.rect.anchors = anchors;
        self
    }
    pub fn with_margins(mut self, margins: [f32; 4]) -> Self {
        self.rect.margins = margins;
        self
    }
    pub fn with_behaviour(mut self, behaviour: Option<Box<dyn Behaviour>>) -> Self {
        self.behaviour = behaviour;
        self
    }
    pub fn with_graphic(mut self, graphic: Option<GraphicId>) -> Self {
        self.graphic = graphic;
        self
    }
    pub fn with_parent(mut self, parent: Option<Id>) -> Self {
        self.parent = parent;
        self
    }
    pub fn build(self) -> Id {
        let Self {
            gui,
            rect,
            graphic,
            behaviour,
            parent,
        } = self;
        gui.add_widget(Widget::new(rect, graphic, behaviour), parent)
    }
}

pub struct Widget {
    rect: Rect,
    graphic: Option<GraphicId>,
    behaviour: Option<Box<dyn Behaviour>>,
}
impl Widget {
    pub fn new(
        rect: Rect,
        graphic: Option<GraphicId>,
        behaviour: Option<Box<dyn Behaviour>>,
    ) -> Self {
        Self {
            rect,
            graphic,
            behaviour,
        }
    }
}

// contains a reference to all the widgets, except the behaviour of one widget
pub struct Widgets<'a> {
    this: Id,
    widgets: &'a mut [Widget],
    hierarchy: &'a mut Hierarchy,
    events: Vec<Box<dyn Any>>,
}
impl<'a> Widgets<'a> {
    pub fn new(
        this: Id,
        widgets: &'a mut [Widget],
        hierarchy: &'a mut Hierarchy,
    ) -> Option<(&'a mut dyn Behaviour, Self)> {
        let this_one = unsafe {
            &mut *(widgets[this.index].behaviour.as_mut()?.as_mut() as *mut dyn Behaviour)
        };
        Some((
            this_one,
            Self {
                this,
                widgets,
                hierarchy,
                events: Vec::new(),
            },
        ))
    }

    pub fn get_behaviour<T: Behaviour>(&mut self, id: Id) -> Option<&mut T> {
        if id == self.this {
            panic!("Attempt to get a second mutable reference to Behaviour")
        }
        self.widgets[id.index].behaviour.as_mut()?.downcast_mut()
    }

    pub fn get_graphic(&mut self, id: Id) -> Option<&mut GraphicId> {
        self.widgets[id.index].graphic.as_mut()
    }

    pub fn get_rect(&mut self, id: Id) -> &mut Rect {
        &mut self.widgets[id.index].rect
    }

    pub fn active(&mut self, id: Id) {
        self.hierarchy.active(id);
        self.events.push(Box::new(event::InvalidadeLayout));
        self.events.push(Box::new(event::Redraw));
    }

    pub fn deactive(&mut self, id: Id) {
        self.hierarchy.deactive(id);
        self.events.push(Box::new(event::InvalidadeLayout));
        self.events.push(Box::new(event::Redraw));
    }
}

#[derive(Default)]
pub struct EventHandler {
    events: Vec<Box<dyn Any>>,
    events_to: Vec<(Id, Box<dyn Any>)>,
}
impl EventHandler {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            events_to: Vec::new(),
        }
    }
    pub fn send_event<T: 'static>(&mut self, event: T) {
        self.events.push(Box::new(event));
    }
    pub fn send_event_to<T: 'static>(&mut self, id: Id, event: T) {
        self.events_to.push((id, Box::new(event)));
    }
}

pub struct GUI<R: GUIRender> {
    widgets: Vec<Widget>,
    hierarchy: Hierarchy,
    current_over: Option<Id>,
    over_is_locked: bool,
    events: Vec<Box<dyn Any>>,
    render: R,
}
impl<R: GUIRender> GUI<R> {
    pub fn new(width: f32, height: f32, render: R) -> Self {
        Self {
            widgets: vec![Widget {
                rect: Rect {
                    anchors: [0.0; 4],
                    margins: [0.0; 4],
                    rect: [0.0, 0.0, width, height],
                },
                graphic: None,
                behaviour: None,
            }],
            hierarchy: Hierarchy::default(),
            current_over: None,
            over_is_locked: false,
            events: Vec::new(),
            render,
        }
    }

    pub fn create_widget(&mut self) -> WidgetBuilder<R> {
        WidgetBuilder::new(self)
    }

    pub fn add_widget(&mut self, widget: Widget, parent: Option<Id>) -> Id {
        let parent = parent.unwrap_or(ROOT_ID);
        self.widgets.push(widget);
        let new = Id {
            index: self.widgets.len() - 1,
        };
        self.hierarchy.resize(self.widgets.len());
        self.hierarchy.set_child(parent, new);
        new
    }

    pub fn active_widget(&mut self, id: Id) {
        self.hierarchy.active(id);
        self.send_event(Box::new(event::InvalidadeLayout));
    }

    pub fn deactive_widget(&mut self, id: Id) {
        self.hierarchy.deactive(id);
        self.send_event(Box::new(event::InvalidadeLayout));
    }

    #[inline]
    pub fn get_childs(&mut self, id: Id) -> Vec<Id> {
        self.hierarchy.get_childs(id)
    }

    #[inline]
    pub fn render(&mut self) -> &mut R {
        &mut self.render
    }

    pub fn set_behaviour_of(&mut self, id: Id, behaviour: Option<Box<dyn Behaviour>>) {
        self.widgets[id.index].behaviour = behaviour;
    }
    pub fn get_graphic(&mut self, id: Id) -> Option<&mut GraphicId> {
        self.widgets[id.index].graphic.as_mut()
    }
    pub fn get_rect(&self, id: Id) -> &Rect {
        &self.widgets[id.index].rect
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.widgets[ROOT_ID.index].rect.rect = [0.0, 0.0, width, height];
        self.update_layouts(ROOT_ID);
    }

    pub fn get_events(&mut self) -> std::vec::Drain<'_, Box<dyn Any>> {
        self.events.drain(..)
    }

    pub fn send_event(&mut self, event: Box<dyn Any>) {
        if event.is::<event::InvalidadeLayout>() {
            self.update_layouts(ROOT_ID);
        } else if event.is::<event::LockOver>() {
            self.over_is_locked = true;
        } else if event.is::<event::UnlockOver>() {
            self.over_is_locked = false;
        } else {
            self.events.push(event);
        }
    }

    pub fn call_event<F: FnOnce(&mut dyn Behaviour, Id, &mut Widgets, &mut EventHandler)>(
        &mut self,
        id: Id,
        event: F,
    ) {
        if let Some((this, mut widgets)) = Widgets::new(id, &mut self.widgets, &mut self.hierarchy)
        {
            let mut event_handler = EventHandler::new();
            event(this, id, &mut widgets, &mut event_handler);
            let EventHandler { events, events_to } = event_handler;
            for event in events.into_iter().chain(widgets.events.into_iter()) {
                self.send_event(event);
            }
            //TODO: this keep a non-intuitive order of event calls
            let mut event_queue = VecDeque::from(events_to);
            while let Some((id, event)) = event_queue.pop_back() {
                if let Some((this, mut widgets)) =
                    Widgets::new(id, &mut self.widgets, &mut self.hierarchy)
                {
                    let mut event_handler = EventHandler::new();
                    this.on_event(event, id, &mut widgets, &mut event_handler);
                    let EventHandler { events, events_to } = event_handler;
                    for event in events.into_iter().chain(widgets.events.into_iter()) {
                        self.send_event(event);
                    }
                    event_queue.extend(events_to.into_iter());
                }
            }
        }
    }

    pub fn start(&mut self) {
        let mut parents = vec![ROOT_ID];
        while let Some(id) = parents.pop() {
            self.call_event(id, |this, id, widgets, event_handler| {
                this.on_start(id, widgets, event_handler)
            });
            for child in &self.hierarchy.get_childs(id) {
                parents.push(*child);
            }
        }
    }

    pub fn handle_event<T>(&mut self, event: &Event<T>) -> bool {
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CursorMoved { position, .. } => {
                    return self.mouse_moved(position.x as f32, position.y as f32);
                }
                WindowEvent::MouseInput { state, .. } => {
                    if let Some(curr) = self.current_over {
                        if self.listen_mouse(curr) {
                            let event = match state {
                                ElementState::Pressed => MouseEvent::Down,
                                ElementState::Released => MouseEvent::Up,
                            };
                            self.send_mouse_event_to(curr, event);
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    pub fn mouse_moved(&mut self, mouse_x: f32, mouse_y: f32) -> bool {
        let mut ret = false;
        if let Some(curr) = self.current_over {
            if self.over_is_locked || self.widgets[curr.index].rect.contains(mouse_x, mouse_y) {
                self.send_mouse_event_to(
                    curr,
                    MouseEvent::Moved {
                        x: mouse_x,
                        y: mouse_y,
                    },
                );
                return false;
            } else {
                self.send_mouse_event_to(curr, MouseEvent::Exit);
                self.current_over = None;
                ret = true;
            }
        }
        let mut curr = ROOT_ID;
        ret | 'l: loop {
            if self.listen_mouse(curr) {
                self.send_mouse_event_to(curr, MouseEvent::Enter);
                self.send_mouse_event_to(
                    curr,
                    MouseEvent::Moved {
                        x: mouse_x,
                        y: mouse_y,
                    },
                );
                self.current_over = Some(curr);
                break true;
            }
            // the interator is reverse because the last childs block the previous ones
            for child in self.hierarchy.get_childs(curr).iter().rev() {
                if self.widgets[child.index].rect.contains(mouse_x, mouse_y) {
                    curr = *child;
                    continue 'l;
                }
            }
            break false;
        }
    }

    pub fn listen_mouse(&self, id: Id) -> bool {
        self.widgets[id.index]
            .behaviour
            .as_ref()
            .map(|x| x.listen_mouse())
            .unwrap_or(false)
    }

    pub fn send_mouse_event_to(&mut self, id: Id, event: MouseEvent) {
        self.call_event(id, |this, id, widgets, event_handler| {
            this.on_mouse_event(event, id, widgets, event_handler)
        });
    }

    pub fn update_layouts(&mut self, id: Id) {
        let mut parents = vec![id];
        while let Some(parent) = parents.pop() {
            for child in &self.hierarchy.get_childs(parent) {
                let size = self.widgets[parent.index].rect.get_size();
                let size = [size.0, size.1];
                let pos: [f32; 2] = [
                    self.widgets[parent.index].rect.rect[0],
                    self.widgets[parent.index].rect.rect[1],
                ];
                for i in 0..4 {
                    self.widgets[child.index].rect.rect[i] = pos[i % 2]
                        + size[i % 2] * self.widgets[child.index].rect.anchors[i]
                        + self.widgets[child.index].rect.margins[i];
                }
                parents.push(*child);
            }
        }
    }
}

/// The basic component of a UI element.
/// The final rect of the element is calculate
/// by the fomula ```anchor*parend_size + margins```.
///
/// For example, a Rect with margins 0 0 0 0, and
/// margins 10 10 40 40, will be a rect always located
/// in the top left corner of the screen, in the position
/// 10 10, and with width and height 30 30.
pub struct Rect {
    pub anchors: [f32; 4],
    pub margins: [f32; 4],
    rect: [f32; 4],
}
impl Rect {
    pub fn new(anchors: [f32; 4], margins: [f32; 4]) -> Self {
        Self {
            anchors,
            margins,
            rect: [0.0; 4],
        }
    }

    #[inline]
    pub fn get_top_left(&self) -> (f32, f32) {
        (self.rect[0], self.rect[1])
    }

    #[inline]
    pub fn get_rect(&self) -> &[f32; 4] {
        &self.rect
    }

    #[inline]
    pub fn get_center(&self) -> (f32, f32) {
        (
            (self.rect[0] + self.rect[2]) / 2.0,
            (self.rect[1] + self.rect[3]) / 2.0,
        )
    }

    #[inline]
    pub fn get_width(&self) -> f32 {
        self.rect[2] - self.rect[0]
    }

    #[inline]
    pub fn get_size(&self) -> (f32, f32) {
        (self.rect[2] - self.rect[0], self.rect[3] - self.rect[1])
    }

    #[inline]
    pub fn get_relative_x(&self, x: f32) -> f32 {
        (x - self.rect[0]) / self.get_width()
    }

    #[inline]
    pub fn contains(&mut self, x: f32, y: f32) -> bool {
        self.rect[0] < x && x < self.rect[2] && self.rect[1] < y && y < self.rect[3]
    }
}

#[allow(unused_variables)]
pub trait Behaviour: 'static {
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn listen_mouse(&self) -> bool;

    fn on_start(&mut self, this: Id, widgets: &mut Widgets, event_handler: &mut EventHandler) {}

    fn on_event(
        &mut self,
        event: Box<dyn Any>,
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
    }

    fn on_mouse_event(
        &mut self,
        event: MouseEvent,
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
    }
}
impl dyn Behaviour {
    fn downcast_mut<T: Behaviour>(&mut self) -> Option<&mut T> {
        if <dyn Behaviour>::type_id(self) == TypeId::of::<T>() {
            Some(unsafe { &mut *(self as *mut dyn Behaviour as *mut T) })
        } else {
            None
        }
    }
}
