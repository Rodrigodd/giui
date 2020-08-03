use crate::{render::Graphic, GUIRender};
use glyph_brush_layout::ab_glyph::FontArc;
use std::any::{Any, TypeId};
use std::collections::VecDeque;
use winit::event::{ElementState, Event, WindowEvent};

pub mod event {
    use super::Id;
    pub struct Redraw;
    pub struct InvalidadeLayout {
        pub id: Id,
    }
    pub struct LockOver;
    pub struct UnlockOver;
    pub struct ActiveWidget {
        pub id: Id,
    }
    pub struct DeactiveWidget {
        pub id: Id,
    }
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

    pub struct ToggleChanged {
        pub id: Id,
        pub value: bool,
    }
}

pub const ROOT_ID: Id = Id { index: 0 };

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Id {
    index: usize,
}
impl Id {
    /// Get the index of the widget in the widgets vector inside GUI<R>
    pub fn get_index(&self) -> usize {
        self.index
    }
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
    fn get_parent(&self, id: Id) -> Option<Id> {
        self.parents[id.index]
    }

    #[inline]
    fn get_children(&self, id: Id) -> Vec<Id> {
        self.childs[id.index]
            .iter()
            .filter(|x| self.active[x.index])
            .cloned()
            .collect::<Vec<Id>>()
    }

    #[inline]
    fn is_active(&mut self, id: Id) -> bool {
        self.active[id.index]
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
    input_flags: InputFlags,
    rect: Rect,
    graphic: Option<Graphic>,
    behaviours: Vec<Box<dyn Behaviour>>,
    layout: Option<Box<dyn Layout>>,
    parent: Option<Id>,
}
impl<'a, R: GUIRender> WidgetBuilder<'a, R> {
    fn new(gui: &'a mut GUI<R>) -> Self {
        Self {
            gui,
            input_flags: InputFlags::empty(),
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
        self.input_flags |= behaviour.input_flags();
        self.behaviours.push(behaviour);
        self
    }
    pub fn with_layout(mut self, layout: Box<dyn Layout>) -> Self {
        self.layout = Some(layout);
        self
    }
    pub fn with_graphic(mut self, graphic: Graphic) -> Self {
        self.graphic = Some(graphic);
        self
    }
    pub fn with_parent(mut self, parent: Id) -> Self {
        self.parent = Some(parent);
        self
    }
    pub fn build(self) -> Id {
        let Self {
            gui,
            input_flags,
            rect,
            graphic,
            behaviours,
            layout,
            parent,
        } = self;
        gui.add_widget(
            Widget {
                input_flags,
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
    input_flags: InputFlags,
    rect: Rect,
    graphic: Option<Graphic>,
    layout: Option<Box<dyn Layout>>,
    behaviours: Vec<Box<dyn Behaviour>>,
}
impl Widget {
    /// create a widget with no behaviour
    pub fn new(rect: Rect, graphic: Option<Graphic>) -> Self {
        Self {
            input_flags: InputFlags::empty(),
            rect,
            graphic,
            layout: None,
            behaviours: Vec::new(),
        }
    }
    /// add one more behaviour to the widget
    pub fn with_behaviour(mut self, behaviour: Box<dyn Behaviour>) -> Self {
        self.input_flags |= behaviour.input_flags();
        self.behaviours.push(behaviour);
        self
    }

    /// add one more behaviour to the widget
    pub fn add_behaviour(&mut self, behaviour: Box<dyn Behaviour>) {
        self.input_flags |= behaviour.input_flags();
        self.behaviours.push(behaviour);
    }

    /// set the layout of the widget
    pub fn with_layout(mut self, layout: Box<dyn Layout>) -> Self {
        self.layout = Some(layout);
        self
    }

    /// set the layout of the widget
    pub fn set_layout(&mut self, layout: Box<dyn Layout>) {
        self.layout = Some(layout);
    }
}

// contains a reference to all the widgets, except the behaviour of one widget
pub struct Widgets<'a> {
    widgets: &'a mut [Widget],
    hierarchy: &'a mut Hierarchy,
    fonts: &'a [FontArc],
    events: Vec<Box<dyn Any>>,
}
impl<'a> Widgets<'a> {
    pub fn new(
        widgets: &'a mut [Widget],
        hierarchy: &'a mut Hierarchy,
        fonts: &'a [FontArc],
    ) -> Self {
        Self {
            widgets,
            hierarchy,
            events: Vec::new(),
            fonts,
        }
    }

    pub fn new_with_mut_layout(
        this: Id,
        widgets: &'a mut [Widget],
        hierarchy: &'a mut Hierarchy,
        fonts: &'a [FontArc],
    ) -> Option<(&'a mut dyn Layout, Self)> {
        let this_one =
            unsafe { &mut *(widgets[this.index].layout.as_mut()?.as_mut() as *mut dyn Layout) };
        Some((
            this_one,
            Self {
                widgets,
                hierarchy,
                events: Vec::new(),
                fonts,
            },
        ))
    }

    pub fn new_with_mut_behaviour(
        this: Id,
        index: usize,
        widgets: &'a mut [Widget],
        hierarchy: &'a mut Hierarchy,
        fonts: &'a [FontArc],
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
                fonts,
            },
        ))
    }

    pub fn get_fonts(&mut self) -> &'a [FontArc] {
        self.fonts
    }

    pub fn get_rect(&mut self, id: Id) -> &mut Rect {
        &mut self.widgets[id.index].rect
    }

    pub fn get_graphic(&mut self, id: Id) -> Option<&mut Graphic> {
        self.widgets[id.index].graphic.as_mut()
    }

    pub fn get_rect_and_graphic(&mut self, id: Id) -> Option<(&mut Rect, &mut Graphic)> {
        let widget = &mut self.widgets[id.index];
        Some((&mut widget.rect, widget.graphic.as_mut()?))
    }

    pub fn get_layout(&mut self, id: Id) -> Option<&mut dyn Layout> {
        //TODO: this is unsafe, when this is called from inside a Layout with the id 'this'
        Some(self.widgets[id.index].layout.as_mut()?.as_mut())
    }

    pub fn set_layout(&mut self, id: Id, layout: Box<dyn Layout>) {
        //TODO: this is unsafe, when this is called from inside a Layout with the id 'this'
        self.widgets[id.index].layout = Some(layout);
    }

    pub fn is_active(&mut self, id: Id) -> bool {
        self.hierarchy.is_active(id)
    }

    /// This only took effect when Widgets is dropped
    pub fn active(&mut self, id: Id) {
        self.events.push(Box::new(event::ActiveWidget { id }));
    }

    /// This only took effect when Widgets is dropped
    pub fn deactive(&mut self, id: Id) {
        self.events.push(Box::new(event::DeactiveWidget { id }));
    }

    pub fn move_to_front(&mut self, id: Id) {
        self.hierarchy.move_to_front(id);
        self.events.push(Box::new(event::InvalidadeLayout { id }));
        self.events.push(Box::new(event::Redraw));
    }

    pub fn get_parent(&mut self, id: Id) -> Option<Id> {
        self.hierarchy.get_parent(id)
    }

    pub fn get_children(&mut self, id: Id) -> Vec<Id> {
        self.hierarchy.get_children(id)
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
    current_scroll: Option<Id>,
    over_is_locked: bool,
    events: Vec<Box<dyn Any>>,
    fonts: Vec<FontArc>,
    render: R,
}
impl<R: GUIRender> GUI<R> {
    pub fn new(width: f32, height: f32, fonts: Vec<FontArc>, render: R) -> Self {
        Self {
            widgets: vec![Widget {
                input_flags: InputFlags::empty(),
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
                    dirty_flags: DirtyFlags::empty(),
                },
                graphic: None,
                layout: None,
                behaviours: Vec::new(),
            }],
            hierarchy: Hierarchy::default(),
            current_over: None,
            current_scroll: None,
            over_is_locked: false,
            events: Vec::new(),
            fonts,
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
        self.send_event(Box::new(event::InvalidadeLayout { id }));
        self.send_event(Box::new(event::Redraw));
        let mut parents = vec![id];
        while let Some(id) = parents.pop() {
            self.call_event(id, |this, id, widgets, event_handler| {
                this.on_active(id, widgets, event_handler)
            });
            parents.extend(self.hierarchy.get_children(id).iter().rev());
        }
    }

    pub fn deactive_widget(&mut self, id: Id) {
        self.hierarchy.deactive(id);
        self.send_event(Box::new(event::InvalidadeLayout { id }));
    }

    pub fn get_fonts(&mut self) -> Vec<FontArc> {
        self.fonts.clone()
    }

    #[inline]
    pub fn get_children(&mut self, id: Id) -> Vec<Id> {
        self.hierarchy.get_children(id)
    }

    #[inline]
    pub fn render(&mut self) -> &mut R {
        &mut self.render
    }

    #[inline]
    pub fn get_render_and_widgets(&mut self) -> (&mut R, Widgets) {
        (
            &mut self.render,
            Widgets::new(&mut self.widgets, &mut self.hierarchy, &self.fonts),
        )
    }

    pub fn add_behaviour(&mut self, id: Id, behaviour: Box<dyn Behaviour>) {
        self.widgets[id.index].add_behaviour(behaviour);
    }

    pub fn add_layout(&mut self, id: Id, layout: Box<dyn Layout>) {
        self.widgets[id.index].set_layout(layout);
    }

    pub fn get_graphic(&mut self, id: Id) -> Option<&mut Graphic> {
        self.widgets[id.index].graphic.as_mut()
    }
    pub fn get_rect(&self, id: Id) -> &Rect {
        &self.widgets[id.index].rect
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.widgets[ROOT_ID.index].rect.rect = [0.0, 0.0, width, height];
        self.widgets[ROOT_ID.index].rect.dirty_flags.insert(DirtyFlags::all());
        self.update_layouts(ROOT_ID);
    }

    pub fn get_events(&mut self) -> std::vec::Drain<'_, Box<dyn Any>> {
        self.events.drain(..)
    }

    pub fn send_event(&mut self, event: Box<dyn Any>) {
        if let Some(event::InvalidadeLayout { id }) = event.downcast_ref() {
            self.update_layouts(*id);
        } else if let Some(event::ActiveWidget { id }) = event.downcast_ref() {
            self.active_widget(*id);
        } else if let Some(event::DeactiveWidget { id }) = event.downcast_ref() {
            self.deactive_widget(*id);
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
            if let Some((this, mut widgets)) = Widgets::new_with_mut_behaviour(
                id,
                index,
                &mut self.widgets,
                &mut self.hierarchy,
                &self.fonts,
            ) {
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
                            &self.fonts,
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
        self.update_layouts(ROOT_ID);
        let mut parents = vec![ROOT_ID];
        while let Some(id) = parents.pop() {
            self.call_event(id, |this, id, widgets, event_handler| {
                this.on_start(id, widgets, event_handler)
            });
            // when acessing childs directly, the inactive widgets is also picked.
            parents.extend(self.hierarchy.childs[id.index].iter().rev());
        }
        parents.clear();
        parents.push(ROOT_ID);
        while let Some(id) = parents.pop() {
            self.call_event(id, |this, id, widgets, event_handler| {
                this.on_active(id, widgets, event_handler)
            });
            parents.extend(self.hierarchy.get_children(id));
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
                        let event = match state {
                            ElementState::Pressed => MouseEvent::Down,
                            ElementState::Released => MouseEvent::Up,
                        };
                        self.send_mouse_event_to(curr, event);
                        return true;
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    if let Some(curr) = self.current_scroll {
                        //TODO: I should handle Line and Pixel Delta differences more wisely?
                        let delta = match delta {
                            winit::event::MouseScrollDelta::LineDelta(x, y) => {
                                [*x * 100.0, *y * 100.0]
                            }
                            winit::event::MouseScrollDelta::PixelDelta(p) => {
                                [p.x as f32, p.y as f32]
                            }
                        };
                        self.call_event(curr, |this, id, widgets, event_handler| {
                            this.on_scroll_event(delta, id, widgets, event_handler)
                        });
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
                self.current_over = Some(curr);
            }
            if self.listen_scroll(curr) {
                self.current_scroll = Some(curr);
            }
            // the interator is reversed because the last childs block the previous ones
            for child in self.hierarchy.get_children(curr).iter().rev() {
                if self.widgets[child.index].rect.contains(mouse_x, mouse_y) {
                    curr = *child;
                    continue 'l;
                }
            }
            break;
        }

        if let Some(curr) = self.current_over {
            self.send_mouse_event_to(curr, MouseEvent::Enter);
            self.send_mouse_event_to(
                curr,
                MouseEvent::Moved {
                    x: mouse_x,
                    y: mouse_y,
                },
            );
        }
    }

    pub fn listen_mouse(&self, id: Id) -> bool {
        self.widgets[id.index]
            .input_flags
            .contains(InputFlags::POINTER)
    }
    pub fn listen_scroll(&self, id: Id) -> bool {
        self.widgets[id.index]
            .input_flags
            .contains(InputFlags::SCROLL)
    }

    pub fn send_mouse_event_to(&mut self, id: Id, event: MouseEvent) {
        self.call_event(id, |this, id, widgets, event_handler| {
            this.on_mouse_event(event, id, widgets, event_handler)
        });
    }

    pub fn update_layouts(&mut self, mut id: Id) {
        if let Some(parent) = self.hierarchy.get_parent(id) {
            if self.widgets[parent.index].layout.is_some() {
                id = parent;
            } else {
                let size = self.widgets[parent.index].rect.get_size();
                let size = [size.0, size.1];
                let pos: [f32; 2] = [
                    self.widgets[parent.index].rect.rect[0],
                    self.widgets[parent.index].rect.rect[1],
                ];
                let rect = &mut self.widgets[id.index].rect;
                let mut new_rect = [0.0; 4];
                for i in 0..4 {
                    new_rect[i] = pos[i % 2] + size[i % 2] * rect.anchors[i] + rect.margins[i];
                }
                rect.set_rect(new_rect);
                if rect.get_width() < rect.min_size[0] {
                    rect.set_width(rect.min_size[0]);
                }
                if rect.get_height() < rect.min_size[1] {
                    rect.set_height(rect.min_size[1]);
                }
            }
        }

        let mut parents = vec![id];
        let mut i = 0;
        // post order traversal
        while i != parents.len() {
            parents.extend(self.hierarchy.get_children(parents[i]).iter().rev());
            i += 1;
        }
        while let Some(parent) = parents.pop() {
            if let Some((layout, mut widgets)) = Widgets::new_with_mut_layout(
                parent,
                &mut self.widgets,
                &mut self.hierarchy,
                &self.fonts,
            ) {
                layout.compute_min_size(parent, &mut widgets);
            }
        }

        // inorder traversal
        parents.push(id);
        while let Some(parent) = parents.pop() {
            if let Some((layout, mut widgets)) = Widgets::new_with_mut_layout(
                parent,
                &mut self.widgets,
                &mut self.hierarchy,
                &self.fonts,
            ) {
                layout.update_layouts(parent, &mut widgets);
                for event in widgets.events {
                    if let Some(event::DeactiveWidget { id }) = event.downcast_ref() {
                        self.hierarchy.deactive(*id);
                    } else if let Some(event::ActiveWidget { id }) = event.downcast_ref() {
                        self.hierarchy.active(*id);
                    }
                }
            } else {
                for child in &self.hierarchy.get_children(parent) {
                    let size = self.widgets[parent.index].rect.get_size();
                    let size = [size.0, size.1];
                    let pos: [f32; 2] = [
                        self.widgets[parent.index].rect.rect[0],
                        self.widgets[parent.index].rect.rect[1],
                    ];
                    let rect = &mut self.widgets[child.index].rect;
                    let mut new_rect = [0.0; 4];
                    for i in 0..4 {
                        new_rect[i] = pos[i % 2] + size[i % 2] * rect.anchors[i] + rect.margins[i];
                    }
                    rect.set_rect(new_rect);
                    if rect.get_width() < rect.min_size[0] {
                        rect.set_width(rect.min_size[0]);
                    }
                    if rect.get_height() < rect.min_size[1] {
                        rect.set_height(rect.min_size[1]);
                    }
                }
            }
            parents.extend(self.hierarchy.get_children(parent).iter().rev());
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RectFill {
    Fill,
    ShrinkStart,
    ShrinkCenter,
    ShrinkEnd,
}

bitflags! {
    pub struct DirtyFlags: u32 {
        /// The width of the rect has changed
        const WIDTH = 0x01;
        /// The height of the rect has changed
        const HEIGHT = 0x02;
        /// The rect of the rect has changed
        const RECT = 0x04;
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
    pub min_size: [f32; 2],
    rect: [f32; 4],
    expand_x: bool,
    expand_y: bool,
    fill_x: RectFill,
    fill_y: RectFill,
    pub ratio_x: f32,
    pub ratio_y: f32,
    dirty_flags: DirtyFlags,
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
            dirty_flags: DirtyFlags::all(),
        }
    }

    /// Get the dirty flags. The dirty flags keep track if some values have changed
    /// since last call to clear_dirty_flags.
    pub fn get_dirty_flags(&mut self) -> DirtyFlags {
        self.dirty_flags
    }

    pub fn clear_dirty_flags(&mut self) {
        self.dirty_flags = DirtyFlags::empty();
    }

    pub fn set_rect(&mut self, rect: [f32; 4]) {
        #[allow(clippy::float_cmp)]
        if rect == self.rect {
            return;
        }
        self.dirty_flags.insert(DirtyFlags::RECT);
        if (self.get_width() - (rect[2] - rect[0])).abs() > f32::EPSILON {
            self.dirty_flags.insert(DirtyFlags::WIDTH);
        }
        if (self.get_height() - (rect[3] - rect[1])).abs() > f32::EPSILON {
            self.dirty_flags.insert(DirtyFlags::HEIGHT);
        }
        self.rect = rect;
    }

    /// Set the designed area for this rect. This rect will decide its own size,
    /// based on its size flags and the designed area.
    pub fn set_designed_rect(&mut self, rect: [f32; 4]) {
        let mut new_rect = [0.0; 4];
        if rect[2] - rect[0] <= self.min_size[0] {
            new_rect[0] = rect[0];
            new_rect[2] = rect[0] + self.min_size[0];
        } else {
            match self.fill_x {
                RectFill::Fill => {
                    new_rect[0] = rect[0];
                    new_rect[2] = rect[2];
                }
                RectFill::ShrinkStart => {
                    new_rect[0] = rect[0];
                    new_rect[2] = rect[0] + self.min_size[0];
                }
                RectFill::ShrinkCenter => {
                    let x = (rect[2] - rect[0] - self.min_size[0]) / 2.0;
                    new_rect[0] = rect[0] + x;
                    new_rect[2] = rect[2] - x;
                }
                RectFill::ShrinkEnd => {
                    new_rect[0] = rect[2] - self.min_size[0];
                    new_rect[2] = rect[2];
                }
            }
        }

        if rect[3] - rect[1] <= self.min_size[1] {
            new_rect[1] = rect[1];
            new_rect[3] = rect[1] + self.min_size[1];
        } else {
            match self.fill_y {
                RectFill::Fill => {
                    new_rect[1] = rect[1];
                    new_rect[3] = rect[3];
                }
                RectFill::ShrinkStart => {
                    new_rect[1] = rect[1];
                    new_rect[3] = rect[1] + self.min_size[1];
                }
                RectFill::ShrinkCenter => {
                    let x = (rect[3] - rect[1] - self.min_size[1]) / 2.0;
                    new_rect[1] = rect[1] + x;
                    new_rect[3] = rect[3] - x;
                }
                RectFill::ShrinkEnd => {
                    new_rect[1] = rect[3] - self.min_size[1];
                    new_rect[3] = rect[3];
                }
            }
        }
        self.set_rect(new_rect);
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
    pub fn set_width(&mut self, width: f32) {
        if (self.get_width() - width).abs() > f32::EPSILON {
            self.dirty_flags.insert(DirtyFlags::WIDTH);
        }
        self.rect[2] = self.rect[0] + width;
    }

    #[inline]
    pub fn get_height(&self) -> f32 {
        self.rect[3] - self.rect[1]
    }

    #[inline]
    pub fn set_height(&mut self, height: f32) {
        if (self.get_height() - height).abs() > f32::EPSILON {
            self.dirty_flags.insert(DirtyFlags::HEIGHT);
        }
        self.rect[3] = self.rect[1] + height;
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

bitflags! {
    pub struct InputFlags: u32 {
        const POINTER = 0x01;
        const SCROLL = 0x02;
    }
}

#[allow(unused_variables)]
pub trait Behaviour {
    fn input_flags(&self) -> InputFlags;

    fn on_start(&mut self, this: Id, widgets: &mut Widgets, event_handler: &mut EventHandler) {}
    fn on_active(&mut self, this: Id, widgets: &mut Widgets, event_handler: &mut EventHandler) {}

    fn on_event(
        &mut self,
        event: &dyn Any,
        this: Id,
        widgets: &mut Widgets,
        event_handler: &mut EventHandler,
    ) {
    }

    fn on_scroll_event(
        &mut self,
        delta: [f32; 2],
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
pub trait Layout: 'static {
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    /// Cmpute its own min size, based on the min size of its children.
    fn compute_min_size(&mut self, this: Id, widgets: &mut Widgets);

    /// Update the position and size of its children.
    fn update_layouts(&mut self, this: Id, widgets: &mut Widgets);
}
impl dyn Layout {
    #[inline]
    pub fn is<T: Any>(&self) -> bool {
        let t = TypeId::of::<T>();
        let concrete = self.type_id();
        t == concrete
    }

    #[inline]
    pub fn downcast_ref<T: Layout>(&self) -> Option<&T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(&*(self as *const dyn Layout as *const T)) }
        } else {
            None
        }
    }

    #[inline]
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(&mut *(self as *mut dyn Layout as *mut T)) }
        } else {
            None
        }
    }
}
