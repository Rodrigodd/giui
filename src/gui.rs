use crate::{render::GraphicId, GUIRender};
use std::any::Any;
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

    #[inline]
    fn active(&mut self, id: Id) {
        self.active[id.index] = true;
    }

    #[inline]
    fn deactive(&mut self, id: Id) {
        self.active[id.index] = false;
    }

    fn move_to_front(&mut self, id: Id) {
        if let Some(parent) = self.parents[id.index] {
            let childs = &mut self.childs[parent.index];
            let i = childs.iter().position(|x| *x == id).unwrap();
            childs.remove(i);
            childs.push(id);
        }
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
    listen_mouse: bool,
    rect: Rect,
    graphic: Option<GraphicId>,
    behaviours: Vec<Box<dyn Behaviour>>,
    layout: Option<Box<dyn Layout>>,
    parent: Option<Id>,
}
impl<'a, R: GUIRender> WidgetBuilder<'a, R> {
    fn new(gui: &'a mut GUI<R>) -> Self {
        Self {
            gui,
            listen_mouse: false,
            rect: Rect::new([0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0]),
            graphic: None,
            behaviours: Vec::new(),
            layout: None,
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
    pub fn with_min_size(mut self, min_size: [f32; 2]) -> Self {
        self.rect.min_size = min_size;
        self
    }
    pub fn with_fill_x(mut self, fill: RectFill) -> Self {
        self.rect.set_fill_x(fill);
        self
    }
    pub fn with_fill_y(mut self, fill: RectFill) -> Self {
        self.rect.set_fill_y(fill);
        self
    }
    pub fn with_expand_x(mut self, expand: bool) -> Self {
        self.rect.expand_x = expand;
        self
    }
    pub fn with_expand_y(mut self, expand: bool) -> Self {
        self.rect.expand_y = expand;
        self
    }
    pub fn with_behaviour(mut self, behaviour: Box<dyn Behaviour>) -> Self {
        self.listen_mouse = self.listen_mouse || behaviour.listen_mouse();
        self.behaviours.push(behaviour);
        self
    }
    pub fn with_layout(mut self, layout: Box<dyn Layout>) -> Self {
        self.layout = Some(layout);
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
            listen_mouse,
            rect,
            graphic,
            behaviours,
            layout,
            parent,
        } = self;
        gui.add_widget(
            Widget {
                listen_mouse,
                rect,
                graphic,
                layout,
                behaviours,
            },
            parent,
        )
    }
}

pub struct Widget {
    listen_mouse: bool,
    rect: Rect,
    graphic: Option<GraphicId>,
    layout: Option<Box<dyn Layout>>,
    behaviours: Vec<Box<dyn Behaviour>>,
}
impl Widget {
    /// create a widget with no behaviour
    pub fn new(rect: Rect, graphic: Option<GraphicId>) -> Self {
        Self {
            listen_mouse: false,
            rect,
            graphic,
            layout: None,
            behaviours: Vec::new(),
        }
    }
    /// add one more behaviour to the widget
    pub fn with_behaviour(mut self, behaviour: Box<dyn Behaviour>) -> Self {
        self.listen_mouse = self.listen_mouse || behaviour.listen_mouse();
        self.behaviours.push(behaviour);
        self
    }

    /// add one more behaviour to the widget
    pub fn add_behaviour(&mut self, behaviour: Box<dyn Behaviour>) {
        self.listen_mouse = self.listen_mouse || behaviour.listen_mouse();
        self.behaviours.push(behaviour);
    }

    /// add one more behaviour to the widget
    pub fn with_layout(mut self, layout: Box<dyn Layout>) -> Self {
        self.layout = Some(layout);
        self
    }
}

// contains a reference to all the widgets, except the behaviour of one widget
pub struct Widgets<'a> {
    widgets: &'a mut [Widget],
    hierarchy: &'a mut Hierarchy,
    events: Vec<Box<dyn Any>>,
}
impl<'a> Widgets<'a> {
    pub fn new_with_mut_layout(
        this: Id,
        widgets: &'a mut [Widget],
        hierarchy: &'a mut Hierarchy,
    ) -> Option<(&'a mut dyn Layout, Self)> {
        let this_one =
            unsafe { &mut *(widgets[this.index].layout.as_mut()?.as_mut() as *mut dyn Layout) };
        Some((
            this_one,
            Self {
                widgets,
                hierarchy,
                events: Vec::new(),
            },
        ))
    }

    pub fn new_with_mut_behaviour(
        this: Id,
        index: usize,
        widgets: &'a mut [Widget],
        hierarchy: &'a mut Hierarchy,
    ) -> Option<(&'a mut dyn Behaviour, Self)> {
        let this_one = unsafe {
            &mut *(widgets[this.index].behaviours.get_mut(index)?.as_mut() as *mut dyn Behaviour)
        };
        Some((
            this_one,
            Self {
                widgets,
                hierarchy,
                events: Vec::new(),
            },
        ))
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

    pub fn move_to_front(&mut self, id: Id) {
        self.hierarchy.move_to_front(id);
        self.events.push(Box::new(event::InvalidadeLayout));
        self.events.push(Box::new(event::Redraw));
    }

    pub fn get_children(&mut self, id: Id) -> Vec<Id> {
        self.hierarchy.get_childs(id)
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
                listen_mouse: false,
                rect: Rect {
                    anchors: [0.0; 4],
                    margins: [0.0; 4],
                    min_size: [width, height],
                    rect: [0.0, 0.0, width, height],
                    expand_x: false,
                    expand_y: false,
                    fill_x: RectFill::Fill,
                    fill_y: RectFill::Fill,
                    ratio_x: 0.0,
                    ratio_y: 0.0,
                },
                graphic: None,
                layout: None,
                behaviours: Vec::new(),
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

    pub fn add_behaviour(&mut self, id: Id, behaviour: Box<dyn Behaviour>) {
        self.widgets[id.index].add_behaviour(behaviour);
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

    pub fn call_event<F: Fn(&mut dyn Behaviour, Id, &mut Widgets, &mut EventHandler)>(
        &mut self,
        id: Id,
        event: F,
    ) {
        for index in 0..self.widgets[id.index].behaviours.len() {
            if let Some((this, mut widgets)) =
                Widgets::new_with_mut_behaviour(id, index, &mut self.widgets, &mut self.hierarchy)
            {
                let mut event_handler = EventHandler::new();
                event(this, id, &mut widgets, &mut event_handler);
                let EventHandler { events, events_to } = event_handler;
                for event in events.into_iter().chain(widgets.events.into_iter()) {
                    self.send_event(event);
                }
                let mut event_queue = VecDeque::from(events_to);
                while let Some((id, event)) = event_queue.pop_back() {
                    for index in 0..self.widgets[id.index].behaviours.len() {
                        if let Some((this, mut widgets)) = Widgets::new_with_mut_behaviour(
                            id,
                            index,
                            &mut self.widgets,
                            &mut self.hierarchy,
                        ) {
                            let mut event_handler = EventHandler::new();
                            this.on_event(event.as_ref(), id, &mut widgets, &mut event_handler);
                            let EventHandler { events, events_to } = event_handler;
                            for event in events.into_iter().chain(widgets.events.into_iter()) {
                                self.send_event(event);
                            }
                            event_queue.extend(events_to.into_iter().rev());
                        }
                    }
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
            parents.extend(self.hierarchy.childs[id.index].iter().rev());
        }
        fn print_tree(deep: usize, id: Id, hierarchy: &Hierarchy) {
            let childs = hierarchy.childs[id.index].clone();
            let len = childs.len();
            for (i, child) in childs.iter().enumerate() {
                println!(
                    "{}{}━━{:?}",
                    "┃  ".repeat(deep),
                    if i + 1 == len { "┗" } else { "┣" },
                    child
                );
                print_tree(deep + 1, *child, hierarchy)
            }
        }
        println!("{:?}", ROOT_ID);
        print_tree(0, ROOT_ID, &self.hierarchy);
    }

    pub fn handle_event<T>(&mut self, event: &Event<T>) -> bool {
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.mouse_moved(position.x as f32, position.y as f32);
                    return true;
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
                WindowEvent::CursorLeft { .. } => {
                    if let Some(curr) = self.current_over.take() {
                        if self.listen_mouse(curr) && !self.over_is_locked {
                            self.send_mouse_event_to(curr, MouseEvent::Exit);
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    pub fn mouse_moved(&mut self, mouse_x: f32, mouse_y: f32) {
        if let Some(curr) = self.current_over {
            if self.over_is_locked || self.widgets[curr.index].rect.contains(mouse_x, mouse_y) {
                self.send_mouse_event_to(
                    curr,
                    MouseEvent::Moved {
                        x: mouse_x,
                        y: mouse_y,
                    },
                );
                return;
            } else {
                self.send_mouse_event_to(curr, MouseEvent::Exit);
                self.current_over = None;
            }
        }
        let mut curr = ROOT_ID;
        'l: loop {
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
            }
            // the interator is reversed because the last childs block the previous ones
            for child in self.hierarchy.get_childs(curr).iter().rev() {
                if self.widgets[child.index].rect.contains(mouse_x, mouse_y) {
                    curr = *child;
                    continue 'l;
                }
            }
            break;
        }
    }

    pub fn listen_mouse(&self, id: Id) -> bool {
        self.widgets[id.index].listen_mouse
    }

    pub fn send_mouse_event_to(&mut self, id: Id, event: MouseEvent) {
        self.call_event(id, |this, id, widgets, event_handler| {
            this.on_mouse_event(event, id, widgets, event_handler)
        });
    }

    pub fn update_layouts(&mut self, id: Id) {
        let mut parents = vec![id];
        let mut i = 0;
        // post order traversal
        while i != parents.len() {
            parents.extend(self.hierarchy.get_childs(parents[i]).iter().rev());
            i += 1;
        }
        while let Some(parent) = parents.pop() {
            if let Some((layout, mut widgets)) =
                Widgets::new_with_mut_layout(parent, &mut self.widgets, &mut self.hierarchy)
            {
                layout.compute_min_size(parent, &mut widgets);
            }
        }

        // inorder traversal
        parents.push(id);
        while let Some(parent) = parents.pop() {
            if let Some((layout, mut widgets)) =
                Widgets::new_with_mut_layout(parent, &mut self.widgets, &mut self.hierarchy)
            {
                layout.update_layouts(parent, &mut widgets);
            } else {
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
                }
            }
            parents.extend(self.hierarchy.get_childs(parent).iter().rev());
        }
    }
}

#[derive(Copy, Clone)]
pub enum RectFill {
    Fill,
    ShrinkStart,
    ShrinkCenter,
    ShrinkEnd,
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
    pub min_size: [f32; 2],
    pub rect: [f32; 4],
    expand_x: bool,
    expand_y: bool,
    fill_x: RectFill,
    fill_y: RectFill,
    pub ratio_x: f32,
    pub ratio_y: f32,
}
impl Rect {
    pub fn new(anchors: [f32; 4], margins: [f32; 4]) -> Self {
        Self {
            anchors,
            margins,
            min_size: [0.0; 2],
            rect: [0.0; 4],
            expand_x: false,
            expand_y: false,
            fill_x: RectFill::Fill,
            fill_y: RectFill::Fill,
            ratio_x: 1.0,
            ratio_y: 1.0,
        }
    }

    /// Set the designed area for this rect. This rect will decide its own size,
    /// based on its size flags and the designed area.
    pub fn set_designed_rect(&mut self, rect: [f32; 4]) {
        if rect[2] - rect[0] <= self.min_size[0] {
            self.rect[0] = rect[0];
            self.rect[2] = rect[0] + self.min_size[0];
        } else {
            match self.fill_x {
                RectFill::Fill => {
                    self.rect[0] = rect[0];
                    self.rect[2] = rect[2];
                }
                RectFill::ShrinkStart => {
                    self.rect[0] = rect[0];
                    self.rect[2] = rect[0] + self.min_size[0];
                }
                RectFill::ShrinkCenter => {
                    let x = (rect[2] - rect[0] - self.min_size[0]) / 2.0;
                    self.rect[0] = rect[0] + x;
                    self.rect[2] = rect[2] - x;
                }
                RectFill::ShrinkEnd => {
                    self.rect[0] = rect[2] - self.min_size[0];
                    self.rect[2] = rect[2];
                }
            }
        }

        if rect[3] - rect[1] <= self.min_size[1] {
            self.rect[1] = rect[1];
            self.rect[3] = rect[1] + self.min_size[1];
        } else {
            match self.fill_y {
                RectFill::Fill => {
                    self.rect[1] = rect[1];
                    self.rect[3] = rect[3];
                }
                RectFill::ShrinkStart => {
                    self.rect[1] = rect[1];
                    self.rect[3] = rect[1] + self.min_size[1];
                }
                RectFill::ShrinkCenter => {
                    let x = (rect[3] - rect[1] - self.min_size[1]) / 2.0;
                    self.rect[1] = rect[1] + x;
                    self.rect[3] = rect[3] - x;
                }
                RectFill::ShrinkEnd => {
                    self.rect[1] = rect[3] - self.min_size[1];
                    self.rect[3] = rect[3];
                }
            }
        }
    }

    pub fn set_fill_x(&mut self, fill: RectFill) {
        self.fill_x = fill;
    }

    pub fn set_fill_y(&mut self, fill: RectFill) {
        self.fill_y = fill;
    }

    #[inline]
    pub fn set_min_size(&mut self, min_size: [f32; 2]) {
        self.min_size = min_size;
    }

    /// Return true if this have the size_flag::EXPAND_X flag.
    #[inline]
    pub fn is_expand_x(&mut self) -> bool {
        self.expand_x
    }

    /// Return true if this have the size_flag::EXPAND_Y flag.
    #[inline]
    pub fn is_expand_y(&mut self) -> bool {
        self.expand_y
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
    pub fn get_height(&self) -> f32 {
        self.rect[3] - self.rect[1]
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
pub trait Behaviour {
    fn listen_mouse(&self) -> bool;

    fn on_start(&mut self, this: Id, widgets: &mut Widgets, event_handler: &mut EventHandler) {}

    fn on_event(
        &mut self,
        event: &dyn Any,
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

#[allow(unused_variables)]
pub trait Layout {
    /// Cmpute its own min size, based on the min size of its children.
    fn compute_min_size(&mut self, this: Id, widgets: &mut Widgets);

    /// Update the position and size of its children.
    fn update_layouts(&mut self, this: Id, widgets: &mut Widgets);
}
