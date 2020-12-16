use crate::{render::Graphic, util::cmp_float};
use ab_glyph::FontArc;
use std::any::Any;
use std::collections::VecDeque;
use winit::event::{
    ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent,
};

pub mod event {
    use super::Id;
    pub struct LockOver;
    pub struct UnlockOver;
    pub struct RequestFocus {
        pub id: Id,
    }
    pub struct ActiveControl {
        pub id: Id,
    }
    pub struct DeactiveControl {
        pub id: Id,
    }
    pub struct RemoveControl {
        pub id: Id,
    }

    pub struct SubmitText {
        pub id: Id,
        pub text: String,
    }
    pub struct ClearText;
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

pub const ROOT_ID: Id = Id {
    index: 0,
    generation: 0,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Id {
    index: u32,
    generation: u32,
}
impl Id {
    /// Get the index of the control in the controls vector inside GUI<R>
    pub fn get_index(&self) -> usize {
        self.index as usize
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

#[derive(Copy, Clone)]
pub enum KeyboardEvent {
    Char(char),
    Pressed(VirtualKeyCode),
}

fn move_to_front(controls: &mut Controls, id: Id) {
    debug_assert!(
        controls[id.get_index()].generation == id.generation,
        "The Control with this Id is not alive anymore"
    );
    if let Some(parent) = controls[id.get_index()].parent {
        let children = &mut controls[parent.get_index()].children;
        let i = children.iter().position(|x| *x == id).unwrap();
        children.remove(i);
        children.push(id);
    }
}

fn is_child(controls: &mut Controls, parent: Id, child: Id) -> bool {
    debug_assert!(
        controls[child.get_index()].generation == child.generation,
        "The Control with this Id is not alive anymore"
    );
    debug_assert!(
        controls[parent.get_index()].generation == parent.generation,
        "The Control with this Id is not alive anymore"
    );
    Some(parent) == controls[child.get_index()].parent
}

fn is_descendant(controls: &mut Controls, ascendant: Id, descendant: Id) -> bool {
    debug_assert!(
        controls[ascendant.get_index()].generation == ascendant.generation,
        "The Control with this Id is not alive anymore"
    );
    debug_assert!(
        controls[descendant.get_index()].generation == descendant.generation,
        "The Control with this Id is not alive anymore"
    );
    let mut curr = descendant;
    while let Some(parent) = controls[curr.get_index()].parent {
        if parent == ascendant {
            return true;
        }
        curr = parent;
    }
    false
}

fn get_children(controls: &Controls, id: Id) -> Vec<Id> {
    debug_assert!(
        controls[id.get_index()].generation == id.generation,
        "The Control with this Id is not alive anymore"
    );
    controls[id.get_index()]
        .children
        .iter()
        .cloned()
        .filter(|x| controls[x.get_index()].active)
        .collect::<Vec<Id>>()
}

fn get_control_stack(controls: &Controls, id: Id) -> Vec<Id> {
    debug_assert!(
        controls[id.get_index()].generation == id.generation,
        "The Control with this Id is not alive anymore"
    );
    let mut curr = id;
    let mut stack = vec![curr];
    while let Some(parent) = controls[curr.get_index()].parent {
        curr = parent;
        stack.push(curr);
    }
    stack
}

fn lowest_common_ancestor(controls: &Controls, a: Id, b: Id) -> Option<Id> {
    debug_assert!(
        controls[a.get_index()].generation == a.generation,
        "The Control with this Id is not alive anymore"
    );
    debug_assert!(
        controls[b.get_index()].generation == b.generation,
        "The Control with this Id is not alive anymore"
    );
    let a_stack = get_control_stack(controls, a);
    let b_stack = get_control_stack(controls, b);
    // lowest common anscertor
    a_stack
        .iter()
        .rev()
        .zip(b_stack.iter().rev())
        .take_while(|(a, b)| *a == *b)
        .last()
        .map(|(a, _)| *a)
}

pub struct ControlBuilder<'a> {
    builder: Box<dyn FnOnce(ControlBuild) -> Id + 'a>,
    build: ControlBuild,
}
impl<'a> ControlBuilder<'a> {
    fn new(builder: Box<dyn FnOnce(ControlBuild) -> Id + 'a>) -> Self {
        Self {
            builder,
            build: ControlBuild::default(),
        }
    }
    pub fn with_anchors(mut self, anchors: [f32; 4]) -> Self {
        self.build.rect.anchors = anchors;
        self
    }
    pub fn with_margins(mut self, margins: [f32; 4]) -> Self {
        self.build.rect.margins = margins;
        self
    }
    pub fn with_min_size(mut self, min_size: [f32; 2]) -> Self {
        self.build.rect.min_size = min_size;
        self
    }
    pub fn with_min_width(mut self, min_width: f32) -> Self {
        self.build.rect.min_size[0] = min_width;
        self
    }
    pub fn with_min_height(mut self, min_height: f32) -> Self {
        self.build.rect.min_size[1] = min_height;
        self
    }
    pub fn with_fill_x(mut self, fill: RectFill) -> Self {
        self.build.rect.set_fill_x(fill);
        self
    }
    pub fn with_fill_y(mut self, fill: RectFill) -> Self {
        self.build.rect.set_fill_y(fill);
        self
    }
    pub fn with_expand_x(mut self, expand: bool) -> Self {
        self.build.rect.expand_x = expand;
        self
    }
    pub fn with_expand_y(mut self, expand: bool) -> Self {
        self.build.rect.expand_y = expand;
        self
    }
    pub fn with_behaviour(mut self, behaviour: Box<dyn Behaviour>) -> Self {
        // TODO: remove this in production!!
        debug_assert!(self.build.behaviour.is_none());
        self.build.behaviour = Some(behaviour);
        self
    }
    pub fn with_layout(mut self, layout: Box<dyn Layout>) -> Self {
        self.build.layout = layout;
        self
    }
    pub fn with_graphic(mut self, graphic: Graphic) -> Self {
        self.build.graphic = graphic;
        self
    }
    pub fn with_parent(mut self, parent: Id) -> Self {
        self.build.parent = Some(parent);
        self
    }
    pub fn with_active(mut self, active: bool) -> Self {
        self.build.active = active;
        self
    }
    pub fn build(self) -> Id {
        let Self { build, builder } = self;
        (builder)(build)
    }
}

#[derive(Default)]
pub struct Control {
    generation: u32,
    rect: Rect,
    graphic: Graphic,
    behaviour: Option<Box<dyn Behaviour>>,
    layout: Box<dyn Layout>,
    parent: Option<Id>,
    children: Vec<Id>,
    active: bool,
}
impl Control {
    /// add one more behaviour to the control
    pub fn with_behaviour(mut self, behaviour: Box<dyn Behaviour>) -> Self {
        self.behaviour = Some(behaviour);
        self
    }

    /// add one more behaviour to the control
    pub fn set_behaviour(&mut self, behaviour: Box<dyn Behaviour>) {
        self.behaviour = Some(behaviour);
    }

    pub fn set_layout(&mut self, layout: Box<dyn Layout>) {
        self.layout = layout;
    }

    fn add_children(&mut self, child: Id) {
        if !self.children.iter().any(|x| *x == child) {
            self.children.push(child)
        }
    }

    /// Set the widget with that id to active = true.
    /// Return true if the active was false.
    fn active(&mut self) -> bool {
        if self.active {
            false
        } else {
            self.active = true;
            true
        }
    }

    #[inline]
    /// Set the widget with that id to active = false.
    /// Return true if the active was true.
    fn deactive(&mut self) -> bool {
        if self.active {
            self.active = false;
            true
        } else {
            false
        }
    }
}

pub struct ControlBuild {
    rect: Rect,
    graphic: Graphic,
    behaviour: Option<Box<dyn Behaviour>>,
    layout: Box<dyn Layout>,
    parent: Option<Id>,
    active: bool,
}
impl Default for ControlBuild {
    fn default() -> Self {
        Self {
            rect: Rect::default(),
            graphic: Graphic::None,
            layout: Default::default(),
            behaviour: None,
            parent: None,
            active: true,
        }
    }
}

// contains a reference to all the controls, except the behaviour of one control
pub struct Context<'a> {
    modifiers: ModifiersState,
    controls: &'a mut Controls,
    fonts: &'a [FontArc],
    events: Vec<Box<dyn Any>>,
    events_to: Vec<(Id, Box<dyn Any>)>,
    dirtys: Vec<Id>,
    render_dirty: bool,
}
impl<'a> Context<'a> {
    fn new(controls: &'a mut Controls, fonts: &'a [FontArc], modifiers: ModifiersState) -> Self {
        Self {
            modifiers,
            controls,
            events: Vec::new(),
            events_to: Vec::new(),
            fonts,
            dirtys: Vec::new(),
            render_dirty: false,
        }
    }

    fn new_with_mut_behaviour(
        this: Id,
        controls: &'a mut Controls,
        fonts: &'a [FontArc],
        modifiers: ModifiersState,
    ) -> Option<(&'a mut dyn Behaviour, Self)> {
        let this_one = unsafe {
            &mut *(controls[this.get_index()].behaviour.as_mut()?.as_mut() as *mut dyn Behaviour)
        };
        Some((
            this_one,
            Self {
                modifiers,
                controls,
                events: Vec::new(),
                events_to: Vec::new(),
                fonts,
                dirtys: Vec::new(),
                render_dirty: false,
            },
        ))
    }

    pub fn create_control(&mut self) -> ControlBuilder {
        let id = self.controls.reserve();
        ControlBuilder::new(Box::new(move |build| {
            self.send_event((id, build));
            id
        }))
    }

    fn get_control(&self, id: Id) -> &Control {
        debug_assert!(
            self.controls[id.get_index()].generation == id.generation,
            "The Control with this Id is not alive anymore"
        );
        &self.controls[id.get_index()]
    }

    fn get_control_mut(&mut self, id: Id) -> &mut Control {
        debug_assert!(
            self.controls[id.get_index()].generation == id.generation,
            "The Control with this Id is not alive anymore"
        );
        &mut self.controls[id.get_index()]
    }

    pub fn modifiers(&self) -> ModifiersState {
        self.modifiers
    }

    pub fn get_fonts(&mut self) -> &'a [FontArc] {
        self.fonts
    }

    pub fn send_event<T: 'static>(&mut self, event: T) {
        self.events.push(Box::new(event));
    }
    pub fn send_event_to<T: 'static>(&mut self, id: Id, event: T) {
        self.events_to.push((id, Box::new(event)));
    }

    pub fn get_layouting(&mut self, id: Id) -> &mut Rect {
        &mut self.get_control_mut(id).rect
    }

    pub fn dirty_layout(&mut self, id: Id) {
        if !self.dirtys.iter().any(|x| *x == id) {
            self.dirtys.push(id);
        }
    }

    pub fn get_rect(&self, id: Id) -> &[f32; 4] {
        &self.get_control(id).rect.rect
    }

    pub fn get_size(&mut self, id: Id) -> [f32; 2] {
        self.get_control(id).rect.get_size()
    }

    pub fn get_margins(&self, id: Id) -> &[f32; 4] {
        &self.get_control(id).rect.margins
    }

    pub fn set_margins(&mut self, id: Id, margins: [f32; 4]) {
        self.get_control_mut(id).rect.margins = margins;
        self.dirty_layout(id);
    }

    pub fn get_anchors(&self, id: Id) -> &[f32; 4] {
        &self.get_control(id).rect.anchors
    }

    pub fn set_margin_left(&mut self, id: Id, margin: f32) {
        self.get_control_mut(id).rect.margins[0] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_top(&mut self, id: Id, margin: f32) {
        self.get_control_mut(id).rect.margins[1] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_right(&mut self, id: Id, margin: f32) {
        self.get_control_mut(id).rect.margins[2] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_bottom(&mut self, id: Id, margin: f32) {
        self.get_control_mut(id).rect.margins[3] = margin;
        self.dirty_layout(id);
    }

    pub fn set_anchors(&mut self, id: Id, anchors: [f32; 4]) {
        self.get_control_mut(id).rect.anchors = anchors;
        self.dirty_layout(id);
    }

    pub fn set_anchor_left(&mut self, id: Id, anchor: f32) {
        self.get_control_mut(id).rect.anchors[0] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_top(&mut self, id: Id, anchor: f32) {
        self.get_control_mut(id).rect.anchors[1] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_right(&mut self, id: Id, anchor: f32) {
        self.get_control_mut(id).rect.anchors[2] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_bottom(&mut self, id: Id, anchor: f32) {
        self.get_control_mut(id).rect.anchors[3] = anchor;
        self.dirty_layout(id);
    }

    pub fn get_min_size(&self, id: Id) -> [f32; 2] {
        self.get_control(id).rect.get_min_size()
    }

    pub fn set_min_size(&mut self, id: Id, min_size: [f32; 2]) {
        self.get_control_mut(id).rect.set_min_size(min_size);
        self.dirty_layout(id);
    }

    pub fn get_graphic_mut(&mut self, id: Id) -> &mut Graphic {
        self.render_dirty = true;
        &mut self.get_control_mut(id).graphic
    }

    pub fn set_graphic(&mut self, id: Id, graphic: Graphic) {
        let control = self.get_control_mut(id);
        control.graphic = graphic;
        control.rect.dirty_render_dirty_flags();
        self.render_dirty = true;
    }

    pub fn get_rect_and_graphic(&mut self, id: Id) -> Option<(&mut Rect, &mut Graphic)> {
        let control = self.get_control_mut(id);
        if let Graphic::None = control.graphic {
            None
        } else {
            Some((&mut control.rect, &mut control.graphic))
        }
    }

    pub fn is_active(&self, id: Id) -> bool {
        self.get_control(id).active
    }

    /// This only took effect when Controls is dropped
    pub fn active(&mut self, id: Id) {
        self.events.push(Box::new(event::ActiveControl { id }));
    }

    /// This only took effect when Controls is dropped
    pub fn deactive(&mut self, id: Id) {
        self.events.push(Box::new(event::DeactiveControl { id }));
    }

    /// Destroy the control, drop it, invalidating all of its referencing Id's
    /// This only took effect when Controls is dropped
    pub fn remove(&mut self, id: Id) {
        self.events.push(Box::new(event::RemoveControl { id }));
    }

    pub fn move_to_front(&mut self, id: Id) {
        move_to_front(self.controls, id);
        self.dirty_layout(id);
    }

    pub fn get_parent(&self, id: Id) -> Option<Id> {
        self.get_control(id).parent
    }

    pub fn get_children(&self, id: Id) -> Vec<Id> {
        get_children(&self.controls, id)
    }
}

pub struct MinSizeContext<'a> {
    this: Id,
    controls: &'a mut Controls,
    fonts: &'a [FontArc],
}
impl<'a> MinSizeContext<'a> {
    fn new(
        this: Id,
        controls: &'a mut Controls,
        fonts: &'a [FontArc],
    ) -> (&'a mut dyn Layout, Self) {
        let this_one =
            unsafe { &mut *(controls[this.get_index()].layout.as_mut() as *mut dyn Layout) };
        (
            this_one,
            Self {
                this,
                controls,
                fonts,
            },
        )
    }

    fn get_control(&self, id: Id) -> &Control {
        debug_assert!(
            self.controls[id.get_index()].generation == id.generation,
            "The Control with this Id is not alive anymore"
        );
        &self.controls[id.get_index()]
    }

    fn get_control_mut(&mut self, id: Id) -> &mut Control {
        debug_assert!(
            self.controls[id.get_index()].generation == id.generation,
            "The Control with this Id is not alive anymore"
        );
        &mut self.controls[id.get_index()]
    }

    pub fn get_fonts(&mut self) -> &'a [FontArc] {
        self.fonts
    }

    pub fn get_layouting(&self, id: Id) -> &Rect {
        &self.get_control(id).rect
    }

    pub fn get_rect(&self, id: Id) -> &[f32; 4] {
        &self.get_control(id).rect.rect
    }

    pub fn get_size(&mut self, id: Id) -> [f32; 2] {
        self.get_control(id).rect.get_size()
    }

    pub fn get_margins(&self, id: Id) -> &[f32; 4] {
        &self.get_control(id).rect.margins
    }

    pub fn get_anchors(&self, id: Id) -> &[f32; 4] {
        &self.get_control(id).rect.anchors
    }

    pub fn get_min_size(&self, id: Id) -> [f32; 2] {
        self.get_control(id).rect.get_min_size()
    }

    pub fn set_this_min_size(&mut self, min_size: [f32; 2]) {
        self.get_control_mut(self.this).rect.set_min_size(min_size);
    }

    pub fn get_graphic(&mut self, id: Id) -> &mut Graphic {
        &mut self.get_control_mut(id).graphic
    }

    pub fn get_rect_and_graphic(&mut self, id: Id) -> Option<(&mut Rect, &mut Graphic)> {
        let control = self.get_control_mut(id);
        if let Graphic::None = control.graphic {
            None
        } else {
            Some((&mut control.rect, &mut control.graphic))
        }
    }

    pub fn is_active(&self, id: Id) -> bool {
        self.get_control(id).active
    }

    pub fn get_parent(&self, id: Id) -> Option<Id> {
        self.get_control(id).parent
    }

    pub fn get_children(&self, id: Id) -> Vec<Id> {
        get_children(&self.controls, id)
    }
}

pub struct LayoutContext<'a> {
    this: Id,
    controls: &'a mut Controls,
    dirtys: Vec<Id>,
    events: Vec<Box<dyn Any>>,
}
impl<'a> LayoutContext<'a> {
    fn new(this: Id, controls: &'a mut Controls) -> (&'a mut dyn Layout, Self) {
        let this_one =
            unsafe { &mut *(controls[this.get_index()].layout.as_mut() as *mut dyn Layout) };
        (
            this_one,
            Self {
                this,
                controls,
                dirtys: Vec::new(),
                events: Vec::new(),
            },
        )
    }

    fn get_control(&self, id: Id) -> &Control {
        debug_assert!(
            self.controls[id.get_index()].generation == id.generation,
            "The Control with this Id is not alive anymore"
        );
        &self.controls[id.get_index()]
    }

    fn get_control_mut(&mut self, id: Id) -> &mut Control {
        debug_assert!(
            self.controls[id.get_index()].generation == id.generation,
            "The Control with this Id is not alive anymore"
        );
        &mut self.controls[id.get_index()]
    }

    pub fn set_rect(&mut self, id: Id, rect: [f32; 4]) {
        self.get_control_mut(id).rect.set_rect(rect);
    }

    pub fn set_designed_rect(&mut self, id: Id, rect: [f32; 4]) {
        self.get_control_mut(id).rect.set_designed_rect(rect);
    }

    pub fn get_layouting(&self, id: Id) -> &Rect {
        &self.get_control(id).rect
    }

    pub fn dirty_layout(&mut self, id: Id) {
        debug_assert!(
            !is_child(self.controls, self.this, id),
            "It is only allowed to modify a child using set_rect, or set_designed_rect."
        );
        debug_assert!(
            is_descendant(self.controls, self.this, id),
            "It is only allowed to modify descendant controls."
        );
        if !self.dirtys.iter().any(|x| *x == id) {
            self.dirtys.push(id);
        }
    }

    pub fn get_rect(&self, id: Id) -> &[f32; 4] {
        &self.get_control(id).rect.rect
    }

    pub fn get_size(&mut self, id: Id) -> [f32; 2] {
        self.get_control_mut(id).rect.get_size()
    }

    pub fn get_margins(&self, id: Id) -> &[f32; 4] {
        &self.get_control(id).rect.margins
    }

    pub fn set_margins(&mut self, id: Id, margins: [f32; 4]) {
        self.get_control_mut(id).rect.margins = margins;
        self.dirty_layout(id);
    }

    pub fn get_anchors(&self, id: Id) -> &[f32; 4] {
        &self.get_control(id).rect.anchors
    }

    pub fn set_margin_left(&mut self, id: Id, margin: f32) {
        self.get_control_mut(id).rect.margins[0] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_top(&mut self, id: Id, margin: f32) {
        self.get_control_mut(id).rect.margins[1] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_right(&mut self, id: Id, margin: f32) {
        self.get_control_mut(id).rect.margins[2] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_bottom(&mut self, id: Id, margin: f32) {
        self.get_control_mut(id).rect.margins[3] = margin;
        self.dirty_layout(id);
    }

    pub fn set_anchors(&mut self, id: Id, anchors: [f32; 4]) {
        self.get_control_mut(id).rect.anchors = anchors;
        self.dirty_layout(id);
    }

    pub fn set_anchor_left(&mut self, id: Id, anchor: f32) {
        self.get_control_mut(id).rect.anchors[0] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_top(&mut self, id: Id, anchor: f32) {
        self.get_control_mut(id).rect.anchors[1] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_right(&mut self, id: Id, anchor: f32) {
        self.get_control_mut(id).rect.anchors[2] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_bottom(&mut self, id: Id, anchor: f32) {
        self.get_control_mut(id).rect.anchors[3] = anchor;
        self.dirty_layout(id);
    }

    pub fn get_min_size(&self, id: Id) -> [f32; 2] {
        self.get_control(id).rect.get_min_size()
    }

    pub fn set_min_size(&mut self, id: Id, min_size: [f32; 2]) {
        self.get_control_mut(id).rect.set_min_size(min_size);
        self.dirty_layout(id);
    }

    pub fn is_active(&self, id: Id) -> bool {
        self.get_control(id).active
    }

    /// This only took effect when Controls is dropped
    pub fn active(&mut self, id: Id) {
        self.events.push(Box::new(event::ActiveControl { id }));
    }

    /// This only took effect when Controls is dropped
    pub fn deactive(&mut self, id: Id) {
        self.events.push(Box::new(event::DeactiveControl { id }));
    }

    pub fn move_to_front(&mut self, id: Id) {
        move_to_front(self.controls, id);
        self.dirty_layout(id);
    }

    pub fn get_parent(&self, id: Id) -> Option<Id> {
        self.get_control(id).parent
    }

    pub fn get_children(&self, id: Id) -> Vec<Id> {
        get_children(&self.controls, id)
    }
}

#[derive(Default)]
struct Input {
    mouse_x: f32,
    mouse_y: f32,
}

struct Controls {
    dead_controls: Vec<u32>,
    controls: Vec<Control>,
}
impl Controls {
    fn reserve(&mut self) -> Id {
        if let Some(index) = self.dead_controls.pop() {
            Id {
                generation: self.controls[index as usize].generation,
                index,
            }
        } else {
            let control = Control {
                generation: 0,
                ..Control::default()
            };
            self.controls.push(control);
            Id {
                generation: 0,
                index: self.controls.len() as u32 - 1,
            }
        }
    }
}
impl std::ops::Index<usize> for Controls {
    type Output = Control;
    fn index(&self, index: usize) -> &Self::Output {
        &self.controls[index]
    }
}
impl std::ops::IndexMut<usize> for Controls {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.controls[index]
    }
}
impl From<Vec<Control>> for Controls {
    fn from(controls: Vec<Control>) -> Self {
        Self {
            dead_controls: Vec::new(),
            controls,
        }
    }
}

pub struct GUI {
    controls: Controls,
    events: Vec<Box<dyn Any>>,
    redraw: bool,
    fonts: Vec<FontArc>,
    modifiers: ModifiersState,
    input: Input,
    current_over: Option<Id>,
    current_focus: Option<Id>,
    over_is_locked: bool,
}
impl GUI {
    pub fn new(width: f32, height: f32, fonts: Vec<FontArc>) -> Self {
        Self {
            modifiers: ModifiersState::empty(),
            controls: vec![Control {
                generation: 0,
                rect: Rect {
                    anchors: [0.0; 4],
                    margins: [0.0; 4],
                    min_size: [width, height],
                    rect: [0.0, 0.0, width, height],
                    ..Default::default()
                },
                graphic: Graphic::None,
                behaviour: None,
                layout: Default::default(),
                parent: None,
                children: Vec::new(),
                active: true,
            }]
            .into(),
            input: Input::default(),
            current_over: None,
            current_focus: None,
            over_is_locked: false,
            events: Vec::new(),
            fonts,
            redraw: true,
        }
    }

    fn get_control(&self, id: Id) -> &Control {
        debug_assert!(
            self.controls[id.get_index()].generation == id.generation,
            "The Control with this Id is not alive anymore"
        );
        &self.controls[id.get_index()]
    }

    fn get_control_mut(&mut self, id: Id) -> &mut Control {
        debug_assert!(
            self.controls[id.get_index()].generation == id.generation,
            "The Control with this Id is not alive anymore"
        );
        &mut self.controls[id.get_index()]
    }

    fn get_parent(&self, id: Id) -> Option<Id> {
        self.get_control(id).parent
    }

    fn get_children(&self, id: Id) -> Vec<Id> {
        get_children(&self.controls, id)
    }

    pub fn reserve_id(&mut self) -> Id {
        self.controls.reserve()
    }

    pub fn create_control(&mut self) -> ControlBuilder {
        ControlBuilder::new(Box::new(move |build| self.add_control(build)))
    }

    /// Create a control with a predetermined id, id that can be obtained by the method reserve_id().
    pub fn create_control_reserved(&mut self, reserved_id: Id) -> ControlBuilder {
        ControlBuilder::new(Box::new(move |build| {
            self.add_control_reserved(build, reserved_id)
        }))
    }

    pub fn add_control(&mut self, build: ControlBuild) -> Id {
        let reserve = self.controls.reserve();
        self.add_control_reserved(build, reserve)
    }

    pub fn add_control_reserved(&mut self, build: ControlBuild, reserve: Id) -> Id {
        let ControlBuild {
            rect,
            graphic,
            behaviour,
            layout,
            parent,
            active,
        } = build;
        let new = reserve;
        
        let mut control = &mut self.controls[new.get_index() as usize];
        control.rect = rect;
        control.graphic = graphic;
        control.behaviour = behaviour;
        control.layout = layout;
        control.parent = parent;
        control.active = active;

        assert_eq!(self.controls[new.get_index() as usize].generation, new.generation);
        // self.controls[new.get_index() as usize].generation = new.generation;

        let control = self.get_control_mut(new);

        if let Some(parent) = control.parent {
            self.get_control_mut(parent).add_children(new);
        } else {
            control.parent = Some(ROOT_ID);
            self.get_control_mut(ROOT_ID).add_children(new);
        }

        if active {
            let mut parents = vec![new];
            while let Some(id) = parents.pop() {
                self.call_event(id, |this, id, ctx| this.on_active(id, ctx));
                parents.extend(self.get_children(id).iter().rev());
            }
        }

        self.update_layout(new);
        new
    }

    pub fn active_control(&mut self, id: Id) {
        if !self.get_control_mut(id).active() {
            return;
        }
        if let Some(parent) = self.get_parent(id) {
            self.update_layout(parent);
        }
        let mut parents = vec![id];
        while let Some(id) = parents.pop() {
            parents.extend(self.get_children(id).iter().rev());
            self.call_event(id, |this, id, ctx| this.on_active(id, ctx));
        }
        // uncommenting the line below allow infinity recursion to happen
        // self.mouse_moved(self.input.mouse_x, self.input.mouse_y);
    }

    pub fn deactive_control(&mut self, id: Id) {
        if !self.get_control_mut(id).deactive() {
            return;
        }
        if let Some(parent) = self.get_parent(id) {
            self.update_layout(parent);
        }
        let mut parents = vec![id];
        while let Some(id) = parents.pop() {
            parents.extend(self.get_children(id).iter().rev());
            if Some(id) == self.current_over {
                self.send_mouse_event_to(id, MouseEvent::Exit);
                self.current_over = None;
            }
            self.call_event(id, |this, id, ctx| this.on_deactive(id, ctx));
        }
        // uncommenting the line below allow infinity recursion to happen
        // self.mouse_moved(self.input.mouse_x, self.input.mouse_y);
    }

    /// Remove a control and all of its children
    pub fn remove_control(&mut self, id: Id) {
        if self.controls[id.get_index()].deactive() {
            if let Some(parent) = self.get_parent(id) {
                self.update_layout(parent);
            }
        }
        if let Some(parent) = self.controls[id.get_index()].parent {
            let children = &mut self.get_control_mut(parent).children;
            if let Some(pos) = children.iter().position(|x| *x == id) {
                children.remove(pos);
            }
        }

        let mut parents = vec![id];
        while let Some(id) = parents.pop() {
            parents.extend(self.get_children(id).iter().rev());

            if Some(id) == self.current_over {
                self.send_mouse_event_to(id, MouseEvent::Exit);
                self.current_over = None;
            }
            self.call_event(id, |this, id, ctx| this.on_deactive(id, ctx));

            self.controls[id.get_index()] = Control {
                generation: self.controls[id.get_index()].generation + 1,
                ..Control::default()
            };
            self.controls.dead_controls.push(id.index);
        }
        // uncommenting the line below allow infinity recursion to happen
        // self.mouse_moved(self.input.mouse_x, self.input.mouse_y);
    }

    pub fn get_fonts(&mut self) -> Vec<FontArc> {
        self.fonts.clone()
    }

    pub fn render_is_dirty(&self) -> bool {
        self.redraw
    }

    #[inline]
    pub fn get_render_context(&mut self) -> Context {
        //TODO: Context -> RenderContext
        self.redraw = false;
        Context::new(&mut self.controls, &self.fonts, self.modifiers)
    }

    pub fn set_behaviour(&mut self, id: Id, behaviour: Box<dyn Behaviour>) {
        self.get_control_mut(id).set_behaviour(behaviour);
    }

    pub fn set_layout(&mut self, id: Id, layout: Box<dyn Layout>) {
        self.get_control_mut(id).set_layout(layout);
    }

    pub fn get_graphic(&mut self, id: Id) -> &mut Graphic {
        &mut self.get_control_mut(id).graphic
    }

    pub fn get_rect(&self, id: Id) -> &Rect {
        &self.get_control(id).rect
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.get_control_mut(ROOT_ID)
            .rect
            .set_rect([0.0, 0.0, width, height]);
        self.update_layout(ROOT_ID);
    }

    pub fn get_events(&mut self) -> std::vec::Drain<'_, Box<dyn Any>> {
        self.events.drain(..)
    }

    pub fn send_event(&mut self, event: Box<dyn Any>) {
        if let Some(event::ActiveControl { id }) = event.downcast_ref() {
            self.active_control(*id);
        } else if let Some(event::DeactiveControl { id }) = event.downcast_ref() {
            self.deactive_control(*id);
        } else if let Some(event::RemoveControl { id }) = event.downcast_ref() {
            self.remove_control(*id);
        } else if event.is::<event::LockOver>() {
            self.over_is_locked = true;
        } else if event.is::<event::UnlockOver>() {
            self.over_is_locked = false;
        } else if let Some(event::RequestFocus { id }) = event.downcast_ref() {
            self.set_focus(Some(*id));
        } else if event.is::<(Id, ControlBuild)>() {
            let (id, build) = *event.downcast::<(Id, ControlBuild)>().unwrap();
            self.add_control_reserved(build, id);
        } else {
            self.events.push(event);
        }
    }

    pub fn send_event_to(&mut self, id: Id, event: Box<dyn Any>) {
        if let Some((this, mut ctx)) =
            Context::new_with_mut_behaviour(id, &mut self.controls, &self.fonts, self.modifiers)
        {
            this.on_event(event.as_ref(), id, &mut ctx);
            let Context {
                events,
                events_to,
                dirtys,
                render_dirty,
                ..
            } = ctx;
            if render_dirty {
                self.redraw = true;
            }
            for event in events {
                self.send_event(event);
            }
            for (id, event) in events_to {
                self.send_event_to(id, event);
            }
            for dirty in dirtys {
                self.update_layout(dirty);
            }
        }
    }

    pub fn call_event<F: Fn(&mut dyn Behaviour, Id, &mut Context)>(&mut self, id: Id, event: F) {
        if let Some((this, mut ctx)) =
            Context::new_with_mut_behaviour(id, &mut self.controls, &self.fonts, self.modifiers)
        {
            event(this, id, &mut ctx);
            let Context {
                events,
                events_to,
                dirtys,
                render_dirty,
                ..
            } = ctx;
            if render_dirty {
                self.redraw = true;
            }
            for event in events {
                self.send_event(event);
            }
            let mut event_queue = VecDeque::from(events_to);
            while let Some((id, event)) = event_queue.pop_back() {
                self.send_event_to(id, event);
            }
            for dirty in dirtys {
                self.update_layout(dirty);
            }
        }
    }

    pub fn call_event_chain<F: Fn(&mut dyn Behaviour, Id, &mut Context) -> bool>(
        &mut self,
        id: Id,
        event: F,
    ) {
        let mut handled = false;
        if let Some((this, mut ctx)) =
            Context::new_with_mut_behaviour(id, &mut self.controls, &self.fonts, self.modifiers)
        {
            handled = event(this, id, &mut ctx);
            let Context {
                events,
                events_to,
                dirtys,
                render_dirty,
                ..
            } = ctx;
            if render_dirty {
                self.redraw = true;
            }
            for event in events {
                self.send_event(event);
            }
            for (id, event) in events_to {
                self.send_event_to(id, event);
            }
            for dirty in dirtys {
                self.update_layout(dirty);
            }
        }
        if !handled {
            if let Some(parent) = self.get_control_mut(id).parent {
                self.call_event_chain(parent, event);
            }
        }
    }

    pub fn start(&mut self) {
        self.update_all_layouts();
        let mut parents = vec![ROOT_ID];
        while let Some(id) = parents.pop() {
            self.call_event(id, |this, id, ctx| this.on_start(id, ctx));
            // when acessing childs directly, the inactive controls is also picked.
            parents.extend(self.get_control_mut(id).children.iter().rev());
        }
        parents.clear();
        parents.push(ROOT_ID);
        while let Some(id) = parents.pop() {
            self.call_event(id, |this, id, ctx| this.on_active(id, ctx));
            parents.extend(self.get_children(id));
        }
        fn print_tree(deep: usize, id: Id, gui: &mut GUI) {
            let childs = gui.get_control(id).children.clone();
            let len = childs.len();
            for (i, child) in childs.iter().enumerate() {
                println!(
                    "{}{}━━{:?}",
                    "┃  ".repeat(deep),
                    if i + 1 == len { "┗" } else { "┣" },
                    child
                );
                print_tree(deep + 1, *child, gui)
            }
        }
        println!("{:?}", ROOT_ID);
        print_tree(0, ROOT_ID, self);
    }

    pub fn handle_event<T>(&mut self, event: &Event<T>) {
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.input.mouse_x = position.x as f32;
                    self.input.mouse_y = position.y as f32;
                    self.mouse_moved(position.x as f32, position.y as f32);
                }
                WindowEvent::MouseInput { state, .. } => {
                    if let ElementState::Pressed = state {
                        self.set_focus(self.current_over);
                    }
                    if let Some(curr) = self.current_over {
                        match state {
                            ElementState::Pressed => {
                                self.send_mouse_event_to(curr, MouseEvent::Down);
                            }
                            ElementState::Released => {
                                self.send_mouse_event_to(curr, MouseEvent::Up);
                            }
                        };
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    if let Some(curr) = self.current_over {
                        //TODO: I should handle Line and Pixel Delta differences more wisely?
                        let delta = match delta {
                            winit::event::MouseScrollDelta::LineDelta(x, y) => {
                                [*x * 100.0, *y * 100.0]
                            }
                            winit::event::MouseScrollDelta::PixelDelta(p) => {
                                [p.x as f32, p.y as f32]
                            }
                        };
                        self.call_event_chain(curr, |this, id, ctx| {
                            this.on_scroll_event(delta, id, ctx)
                        });
                    }
                }
                WindowEvent::CursorLeft { .. } => {
                    if let Some(curr) = self.current_over.take() {
                        if !self.over_is_locked {
                            self.send_mouse_event_to(curr, MouseEvent::Exit);
                        }
                    }
                }
                WindowEvent::ReceivedCharacter(ch) => {
                    if let Some(curr) = self.current_focus {
                        if ch.is_control() {
                            return;
                        }
                        self.call_event_chain(curr, move |this, id, ctx| {
                            this.on_keyboard_event(KeyboardEvent::Char(*ch), id, ctx)
                        });
                    }
                }
                WindowEvent::ModifiersChanged(modifiers) => self.modifiers = *modifiers,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(keycode),
                            ..
                        },
                    ..
                } => {
                    if let Some(curr) = self.current_focus {
                        self.call_event_chain(curr, |this, id, ctx| {
                            this.on_keyboard_event(KeyboardEvent::Pressed(*keycode), id, ctx)
                        });
                    }
                }
                _ => {}
            }
        }
    }

    pub fn set_focus(&mut self, id: Option<Id>) {
        if id == self.current_focus {
            return;
        }

        if let (Some(prev), Some(next)) = (self.current_focus, id) {
            let lca = lowest_common_ancestor(&self.controls, prev, next);

            let mut curr = Some(prev);
            if curr != lca {
                while let Some(id) = curr {
                    self.call_event(id, |this, id, ctx| this.on_focus_change(false, id, ctx));
                    curr = self.get_parent(id);
                    if curr == lca {
                        break;
                    }
                }
            }

            self.current_focus = Some(next);

            let mut curr = Some(next);
            if curr != lca {
                while let Some(id) = curr {
                    self.call_event(id, |this, id, ctx| this.on_focus_change(true, id, ctx));
                    curr = self.get_parent(id);
                    if curr == lca {
                        break;
                    }
                }
            }
        } else if let Some(current_keyboard) = self.current_focus {
            self.current_focus = None;
            self.call_event_chain(current_keyboard, |this, id, ctx| {
                this.on_focus_change(false, id, ctx);
                true
            });
        } else if let Some(current_keyboard) = id {
            self.current_focus = Some(current_keyboard);
            self.call_event_chain(current_keyboard, |this, id, ctx| {
                this.on_focus_change(true, id, ctx);
                true
            });
        }
    }

    pub fn mouse_moved(&mut self, mouse_x: f32, mouse_y: f32) {
        if self.current_over.is_some() && self.over_is_locked {
            self.send_mouse_event_to(
                self.current_over.unwrap(),
                MouseEvent::Moved {
                    x: mouse_x,
                    y: mouse_y,
                },
            );
            return;
        }

        let mut curr = ROOT_ID;
        'l: loop {
            // the interator is reversed because the last child block the previous ones
            for child in self.get_children(curr).iter().rev() {
                if self.get_control_mut(*child).rect.contains(mouse_x, mouse_y) {
                    curr = *child;
                    continue 'l;
                }
            }
            break;
        }

        if Some(curr) == self.current_over {
            self.send_mouse_event_to(
                curr,
                MouseEvent::Moved {
                    x: mouse_x,
                    y: mouse_y,
                },
            );
        } else {
            if let Some(current_over) = self.current_over {
                self.send_mouse_event_to(current_over, MouseEvent::Exit);
            }
            self.current_over = Some(curr);
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

    pub fn send_mouse_event_to(&mut self, id: Id, event: MouseEvent) {
        // TODO: This need more thought, because call_event_chain implys that a widget
        // can receive MouseMoved withou receving MouseEnter!
        self.call_event_chain(id, |this, id, ctx| this.on_mouse_event(event, id, ctx));
    }

    pub fn update_layout(&mut self, mut id: Id) {
        self.redraw = true;
        // if min_size is dirty and parent has layout, update parent min_size, and recurse it
        // from the highter parent, update layout of its children. For each dirty chldren, update them, recursivily

        {
            let (layout, mut ctx) = MinSizeContext::new(id, &mut self.controls, &self.fonts);
            layout.compute_min_size(id, &mut ctx);
        }
        while let Some(parent) = self.get_parent(id) {
            self.get_control_mut(id)
                .rect
                .layout_dirty_flags
                .insert(LayoutDirtyFlags::DIRTY);
            if self
                .get_control_mut(id)
                .rect
                .get_layout_dirty_flags()
                .intersects(LayoutDirtyFlags::MIN_WIDTH | LayoutDirtyFlags::MIN_HEIGHT)
            {
                {
                    let (layout, mut ctx) =
                        MinSizeContext::new(parent, &mut self.controls, &self.fonts);
                    layout.compute_min_size(parent, &mut ctx);
                }
                id = parent;
            } else {
                id = parent;
                break;
            }
        }

        // inorder traversal
        let mut parents = vec![id];
        while let Some(id) = parents.pop() {
            {
                let (layout, mut ctx) = LayoutContext::new(id, &mut self.controls);
                layout.update_layouts(id, &mut ctx);
                let LayoutContext { events, dirtys, .. } = ctx;
                for event in events {
                    //TODO: think carefully about this deactives
                    if let Some(event::DeactiveControl { id }) = event.downcast_ref() {
                        // self.deactive_control(*id)
                        self.get_control_mut(*id).active = false;
                    } else if let Some(event::ActiveControl { id }) = event.downcast_ref() {
                        // self.active_control(*id)
                        self.get_control_mut(*id).active = true;
                    }
                }
                for dirty in dirtys {
                    // if dirty == id {
                    //     panic!("Layout cannot modify its own control");
                    // } else {
                    //     self.update_layout(dirty);
                    // }
                    // TODO: RETHINK THIS!!!!
                    if let Some(dirty_parent) = self.get_parent(dirty) {
                        assert!(dirty_parent != id, "A layout cannot dirty its own child!");
                        parents.push(dirty_parent);
                    }
                }
            }

            for child in self.get_children(id).iter().rev() {
                if !self
                    .get_control_mut(*child)
                    .rect
                    .get_layout_dirty_flags()
                    .is_empty()
                {
                    parents.push(*child);
                    self.get_control_mut(*child).rect.clear_layout_dirty_flags();
                }
            }
        }
    }

    pub fn update_all_layouts(&mut self) {
        self.redraw = true;
        let mut parents = vec![ROOT_ID];

        // post order traversal
        let mut i = 0;
        while i != parents.len() {
            parents.extend(self.get_children(parents[i]).iter().rev());
            i += 1;
        }
        while let Some(parent) = parents.pop() {
            {
                let (layout, mut ctx) =
                    MinSizeContext::new(parent, &mut self.controls, &self.fonts);
                layout.compute_min_size(parent, &mut ctx);
            }
        }

        // parents is empty now

        // inorder traversal
        parents.push(ROOT_ID);
        while let Some(parent) = parents.pop() {
            {
                let (layout, mut ctx) = LayoutContext::new(parent, &mut self.controls);
                layout.update_layouts(parent, &mut ctx);
                for event in ctx.events {
                    //TODO: think carefully about this deactives
                    if let Some(event::DeactiveControl { id }) = event.downcast_ref() {
                        // self.deactive_control(*id)
                        self.get_control_mut(*id).active = false;
                    } else if let Some(event::ActiveControl { id }) = event.downcast_ref() {
                        // self.active_control(*id)
                        self.get_control_mut(*id).active = true;
                    }
                }
            }
            parents.extend(self.get_children(parent).iter().rev());
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
impl Default for RectFill {
    fn default() -> Self {
        RectFill::Fill
    }
}

bitflags! {
    pub struct RenderDirtyFlags: u8 {
        /// The width of the rect has changed
        const WIDTH = 0x1;
        /// The height of the rect has changed
        const HEIGHT = 0x2;
        /// The rect of the rect has changed
        const RECT = 0x4;
    }
}
impl Default for RenderDirtyFlags {
    fn default() -> Self {
        RenderDirtyFlags::all()
    }
}

bitflags! {
    pub struct LayoutDirtyFlags: u16 {
        /// The width of the rect has changed
        const WIDTH = 0x01;
        /// The height of the rect has changed
        const HEIGHT = 0x02;
        /// The rect of the rect has changed
        const RECT = 0x04;

        const MIN_WIDTH = 0x08;
        const MIN_HEIGHT = 0x10;

        const DIRTY = 0x20;
    }
}
impl Default for LayoutDirtyFlags {
    fn default() -> Self {
        LayoutDirtyFlags::all()
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
    min_size: [f32; 2],
    rect: [f32; 4],
    expand_x: bool,
    expand_y: bool,
    fill_x: RectFill,
    fill_y: RectFill,
    pub ratio_x: f32,
    pub ratio_y: f32,
    render_dirty_flags: RenderDirtyFlags,
    layout_dirty_flags: LayoutDirtyFlags,
}
impl Default for Rect {
    fn default() -> Self {
        Self {
            anchors: [0.0, 0.0, 1.0, 1.0],
            margins: [0.0, 0.0, 0.0, 0.0],
            min_size: [0.0; 2],
            rect: [0.0; 4],
            expand_x: false,
            expand_y: false,
            fill_x: RectFill::default(),
            fill_y: RectFill::default(),
            ratio_x: 1.0,
            ratio_y: 1.0,
            render_dirty_flags: RenderDirtyFlags::default(),
            layout_dirty_flags: LayoutDirtyFlags::default(),
        }
    }
}
impl Rect {
    pub fn new(anchors: [f32; 4], margins: [f32; 4]) -> Self {
        Self {
            anchors,
            margins,
            ..Default::default()
        }
    }

    /// Get the dirty flags. The dirty flags keep track if some values have changed
    /// since last call to clear_dirty_flags.
    pub fn get_render_dirty_flags(&mut self) -> RenderDirtyFlags {
        self.render_dirty_flags
    }

    pub fn clear_render_dirty_flags(&mut self) {
        self.render_dirty_flags = RenderDirtyFlags::empty();
    }

    pub fn dirty_render_dirty_flags(&mut self) {
        self.render_dirty_flags = RenderDirtyFlags::all();
    }

    pub fn get_layout_dirty_flags(&mut self) -> LayoutDirtyFlags {
        self.layout_dirty_flags
    }

    pub fn clear_layout_dirty_flags(&mut self) {
        self.layout_dirty_flags = LayoutDirtyFlags::empty();
    }

    pub fn dirty_layout_dirty_flags(&mut self) {
        self.layout_dirty_flags = LayoutDirtyFlags::all();
    }

    pub fn set_rect(&mut self, rect: [f32; 4]) {
        #[allow(clippy::float_cmp)]
        if rect == self.rect {
            return;
        }
        self.render_dirty_flags.insert(RenderDirtyFlags::RECT);
        self.layout_dirty_flags.insert(LayoutDirtyFlags::RECT);
        if !cmp_float(self.get_width(), rect[2] - rect[0]) {
            self.render_dirty_flags.insert(RenderDirtyFlags::WIDTH);
            self.layout_dirty_flags.insert(LayoutDirtyFlags::WIDTH);
        }
        if !cmp_float(self.get_height(), rect[3] - rect[1]) {
            self.render_dirty_flags.insert(RenderDirtyFlags::HEIGHT);
            self.layout_dirty_flags.insert(LayoutDirtyFlags::HEIGHT);
        }
        self.rect = rect;
    }

    /// Set the designed area for this rect. This rect will decide its own size,
    /// based on its size flags and the designed area.
    pub fn set_designed_rect(&mut self, rect: [f32; 4]) {
        let mut new_rect = [0.0; 4];
        if rect[2] - rect[0] <= self.get_min_size()[0] {
            new_rect[0] = rect[0];
            new_rect[2] = rect[0] + self.get_min_size()[0];
        } else {
            match self.fill_x {
                RectFill::Fill => {
                    new_rect[0] = rect[0];
                    new_rect[2] = rect[2];
                }
                RectFill::ShrinkStart => {
                    new_rect[0] = rect[0];
                    new_rect[2] = rect[0] + self.get_min_size()[0];
                }
                RectFill::ShrinkCenter => {
                    let x = (rect[2] - rect[0] - self.get_min_size()[0]) / 2.0;
                    new_rect[0] = rect[0] + x;
                    new_rect[2] = rect[2] - x;
                }
                RectFill::ShrinkEnd => {
                    new_rect[0] = rect[2] - self.get_min_size()[0];
                    new_rect[2] = rect[2];
                }
            }
        }

        if rect[3] - rect[1] <= self.get_min_size()[1] {
            new_rect[1] = rect[1];
            new_rect[3] = rect[1] + self.get_min_size()[1];
        } else {
            match self.fill_y {
                RectFill::Fill => {
                    new_rect[1] = rect[1];
                    new_rect[3] = rect[3];
                }
                RectFill::ShrinkStart => {
                    new_rect[1] = rect[1];
                    new_rect[3] = rect[1] + self.get_min_size()[1];
                }
                RectFill::ShrinkCenter => {
                    let x = (rect[3] - rect[1] - self.get_min_size()[1]) / 2.0;
                    new_rect[1] = rect[1] + x;
                    new_rect[3] = rect[3] - x;
                }
                RectFill::ShrinkEnd => {
                    new_rect[1] = rect[3] - self.get_min_size()[1];
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
    pub fn get_min_size(&self) -> [f32; 2] {
        self.min_size
    }

    #[inline]
    pub fn set_min_size(&mut self, min_size: [f32; 2]) {
        if !cmp_float(self.min_size[0], min_size[0]) {
            self.layout_dirty_flags.insert(LayoutDirtyFlags::MIN_WIDTH);
            self.min_size[0] = min_size[0];
        }
        if !cmp_float(self.min_size[1], min_size[1]) {
            self.layout_dirty_flags.insert(LayoutDirtyFlags::MIN_HEIGHT);
            self.min_size[1] = min_size[1];
        }

        if self.get_width() < self.get_min_size()[0] {
            self.set_width(min_size[0]);
        }
        if self.get_height() < self.get_min_size()[1] {
            self.set_height(min_size[1]);
        }
    }

    /// Return true if this have the size_flag::EXPAND_X flag.
    #[inline]
    pub fn is_expand_x(&self) -> bool {
        self.expand_x
    }

    /// Return true if this have the size_flag::EXPAND_Y flag.
    #[inline]
    pub fn is_expand_y(&self) -> bool {
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
        if !cmp_float(self.get_width(), width) {
            self.render_dirty_flags.insert(RenderDirtyFlags::WIDTH);
            self.layout_dirty_flags.insert(LayoutDirtyFlags::WIDTH);
        }
        self.rect[2] = self.rect[0] + width;
    }

    #[inline]
    pub fn get_height(&self) -> f32 {
        self.rect[3] - self.rect[1]
    }

    #[inline]
    pub fn set_height(&mut self, height: f32) {
        if !cmp_float(self.get_height(), height) {
            self.render_dirty_flags.insert(RenderDirtyFlags::HEIGHT);
            self.layout_dirty_flags.insert(LayoutDirtyFlags::HEIGHT);
        }
        self.rect[3] = self.rect[1] + height;
    }

    #[inline]
    pub fn get_size(&self) -> [f32; 2] {
        [self.rect[2] - self.rect[0], self.rect[3] - self.rect[1]]
    }

    #[inline]
    pub fn get_relative_x(&self, x: f32) -> f32 {
        (x - self.rect[0]) / self.get_width()
    }

    #[inline]
    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.rect[0] < x && x < self.rect[2] && self.rect[1] < y && y < self.rect[3]
    }
}

#[allow(unused_variables)]
pub trait Behaviour {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {}
    fn on_active(&mut self, this: Id, ctx: &mut Context) {}
    fn on_deactive(&mut self, this: Id, ctx: &mut Context) {}

    fn on_event(&mut self, event: &dyn Any, this: Id, ctx: &mut Context) {}

    fn on_scroll_event(&mut self, delta: [f32; 2], this: Id, ctx: &mut Context) -> bool {
        false
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        false
    }

    fn on_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {}

    fn on_keyboard_event(&mut self, event: KeyboardEvent, this: Id, ctx: &mut Context) -> bool {
        false
    }
}
impl Behaviour for () {}

#[allow(unused_variables)]
pub trait Layout {
    /// Compute its own min size, based on the min size of its children.
    fn compute_min_size(&mut self, this: Id, ctx: &mut MinSizeContext) {}
    /// Update the position and size of its children.
    fn update_layouts(&mut self, this: Id, ctx: &mut LayoutContext) {
        let rect = ctx.get_rect(this);
        let size = [rect[2] - rect[0], rect[3] - rect[1]];
        let pos: [f32; 2] = [rect[0], rect[1]];
        for child in ctx.get_children(this) {
            let rect = &mut ctx.get_layouting(child);
            let mut new_rect = [0.0; 4];
            for i in 0..4 {
                new_rect[i] = pos[i % 2] + size[i % 2] * rect.anchors[i] + rect.margins[i];
            }
            if new_rect[2] - new_rect[0] < rect.get_min_size()[0] {
                new_rect[2] = new_rect[0] + rect.get_min_size()[0];
            }
            if new_rect[3] - new_rect[1] < rect.get_min_size()[1] {
                new_rect[3] = new_rect[1] + rect.get_min_size()[1];
            }
            ctx.set_designed_rect(child, new_rect);
        }
    }
}
impl Layout for () {}
impl Default for Box<dyn Layout> {
    fn default() -> Self {
        Box::new(())
    }
}
