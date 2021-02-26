use std::{cell::RefCell, num::NonZeroU32, rc::Rc};

use crate::{render::Graphic, Behaviour, Id, Layout, Rect, RectFill};

pub(crate) trait ControlBuilderInner {
    fn controls(&mut self) -> &mut Controls;
    fn build(&mut self, id: Id);
}
pub struct ControlBuilder<'a> {
    id: Id,
    inner: Box<dyn ControlBuilderInner + 'a>,
}
impl<'a> ControlBuilder<'a> {
    pub(crate) fn new<T: ControlBuilderInner + 'a>(id: Id, mut inner: T) -> Self {
        let controls = inner.controls();
        assert!(
            controls[id].state == ControlState::Reserved,
            "The state of the Control referenced by the Id {} is '{}'. Should have been '{}'",
            id,
            controls[id].state,
            ControlState::Reserved
        );
        controls[id].state = ControlState::Building;
        controls[id].active = true;
        Self {
            inner: Box::new(inner),
            id,
        }
    }
    pub fn with_anchors(mut self, anchors: [f32; 4]) -> Self {
        self.inner.controls()[self.id].rect.anchors = anchors;
        self
    }
    pub fn with_margins(mut self, margins: [f32; 4]) -> Self {
        self.inner.controls()[self.id].rect.margins = margins;
        self
    }
    pub fn with_min_size(mut self, min_size: [f32; 2]) -> Self {
        self.inner.controls()[self.id].rect.user_min_size = min_size;
        self.inner.controls()[self.id].rect.min_size = min_size;
        self
    }
    pub fn with_min_width(mut self, min_width: f32) -> Self {
        self.inner.controls()[self.id].rect.min_size[0] = min_width;
        self
    }
    pub fn with_min_height(mut self, min_height: f32) -> Self {
        self.inner.controls()[self.id].rect.min_size[1] = min_height;
        self
    }
    pub fn with_fill_x(mut self, fill: RectFill) -> Self {
        self.inner.controls()[self.id].rect.set_fill_x(fill);
        self
    }
    pub fn with_fill_y(mut self, fill: RectFill) -> Self {
        self.inner.controls()[self.id].rect.set_fill_y(fill);
        self
    }
    pub fn with_expand_x(mut self, expand: bool) -> Self {
        self.inner.controls()[self.id].rect.expand_x = expand;
        self
    }
    pub fn with_expand_y(mut self, expand: bool) -> Self {
        self.inner.controls()[self.id].rect.expand_y = expand;
        self
    }
    pub fn with_behaviour<T: Behaviour + 'static>(mut self, behaviour: T) -> Self {
        // TODO: remove this someday
        debug_assert!(self.inner.controls()[self.id].behaviour.is_none());
        self.inner.controls()[self.id].behaviour = Some(Box::new(behaviour));
        self
    }
    pub fn with_layout<T: Layout + 'static>(mut self, layout: T) -> Self {
        self.inner.controls()[self.id].layout = Box::new(layout);
        self
    }
    pub fn with_behaviour_and_layout<T: Layout + Behaviour + 'static>(
        self,
        behaviour_layout: T,
    ) -> Self {
        let x = Rc::new(RefCell::new(behaviour_layout));
        self.with_behaviour(x.clone()).with_layout(x)
    }
    pub fn with_graphic(mut self, graphic: Graphic) -> Self {
        self.inner.controls()[self.id].graphic = graphic;
        self
    }
    pub fn with_parent(mut self, parent: Id) -> Self {
        self.inner.controls()[self.id].parent = Some(parent);
        self
    }
    pub fn with_active(mut self, active: bool) -> Self {
        self.inner.controls()[self.id].active = active;
        self
    }

    pub fn reserve(&mut self) -> Id {
        self.inner.controls().reserve()
    }

    pub fn with_child<F>(mut self, create_child: F) -> Self
    where
        F: FnOnce(ControlBuilder) -> ControlBuilder,
    {
        let id = self.inner.controls().reserve();
        self.with_child_reserved(id, create_child)
    }

    pub fn with_child_reserved<F>(mut self, id: Id, create_child: F) -> Self
    where
        F: FnOnce(ControlBuilder) -> ControlBuilder,
    {
        // let id = self.inner.controls().reserve();
        let parent = self.id;

        // while creating a child, be sure that it see its parent as deactive
        let active = self.inner.controls()[parent].active;
        self.inner.controls()[parent].active = false;

        {
            struct ChildBuilderInner<'a>(&'a mut Controls);
            impl ControlBuilderInner for ChildBuilderInner<'_> {
                fn controls(&mut self) -> &mut Controls {
                    self.0
                }
                fn build(&mut self, _id: Id) {}
            }
            let child_builder = ControlBuilder::new(id, ChildBuilderInner(self.inner.controls()));
            (create_child)(child_builder).with_parent(parent).build();
        }

        // restore the parent active
        self.inner.controls()[parent].active = active;

        self
    }

    pub fn build(self) -> Id {
        let Self { mut inner, id } = self;

        let controls = inner.controls();

        if let Some(parent) = controls[id].parent {
            controls[parent].add_child(id);
        } else {
            controls[id].parent = Some(Id::ROOT_ID);
            controls[Id::ROOT_ID].add_child(id);
        }

        if controls[id].active
            && controls[id]
                .parent
                .map(|x| controls[x].really_active)
                .unwrap_or(true)
        {
            let mut parents = vec![id];
            while let Some(id) = parents.pop() {
                parents.extend(controls.get_children(id).iter().rev());
                controls[id].really_active = true;
            }
        }

        inner.build(id);
        id
    }
}
pub(crate) struct Controls {
    dead_controls: Vec<u32>,
    controls: Vec<Control>,
}
impl Controls {
    pub fn get(&self, id: Id) -> Option<&Control> {
        if let Some(control) = self.controls.get(id.index()) {
            if self.controls[id.index()].generation == id.generation {
                Some(control)
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn reserve(&mut self) -> Id {
        if let Some(index) = self.dead_controls.pop() {
            debug_assert!(self.controls[index as usize].state == ControlState::Free);
            self.controls[index as usize].state = ControlState::Reserved;
            Id {
                generation: self.controls[index as usize].generation,
                index,
            }
        } else {
            let control = Control {
                generation: NonZeroU32::new(1).unwrap(),
                state: ControlState::Reserved,
                ..Control::default()
            };
            self.controls.push(control);
            Id {
                generation: NonZeroU32::new(1).unwrap(),
                index: self.controls.len() as u32 - 1,
            }
        }
    }

    #[allow(clippy::or_fun_call)]
    pub fn remove(&mut self, id: Id) {
        self[id] = Control {
            generation: NonZeroU32::new(self[id].generation.get() + 1)
                .unwrap_or(NonZeroU32::new(1).unwrap()),
            ..Control::default()
        };
        self.dead_controls.push(id.index);
    }

    pub fn move_to_front(&mut self, id: Id) {
        if let Some(parent) = self[id].parent {
            let children = &mut self[parent].children;
            let i = children.iter().position(|x| *x == id).unwrap();
            children.remove(i);
            children.push(id);
        }
    }

    pub fn is_child(&mut self, parent: Id, child: Id) -> bool {
        Some(parent) == self[child].parent
    }

    pub fn is_descendant(&mut self, ascendant: Id, descendant: Id) -> bool {
        let mut curr = descendant;
        while let Some(parent) = self[curr].parent {
            if parent == ascendant {
                return true;
            }
            curr = parent;
        }
        false
    }

    pub fn get_children(&self, id: Id) -> Vec<Id> {
        self[id]
            .children
            .iter()
            .filter(|x| self[**x].active)
            .cloned()
            .collect::<Vec<Id>>()
    }

    pub fn get_control_stack(&self, id: Id) -> Vec<Id> {
        let mut curr = id;
        let mut stack = vec![curr];
        while let Some(parent) = self[curr].parent {
            curr = parent;
            stack.push(curr);
        }
        stack
    }

    pub fn lowest_common_ancestor(&self, a: Id, b: Id) -> Option<Id> {
        let a_stack = self.get_control_stack(a);
        let b_stack = self.get_control_stack(b);
        // lowest common anscertor
        a_stack
            .iter()
            .rev()
            .zip(b_stack.iter().rev())
            .take_while(|(a, b)| *a == *b)
            .last()
            .map(|(a, _)| *a)
    }

    pub fn tree_starting_at(&self, id: Id) -> Vec<Id> {
        debug_assert!(self[id].active);
        if let Some(parent) = self[id].parent {
            let mut up = self.tree_starting_at(parent);
            up.pop();
            let children = self.get_children(parent);
            let i = children
                .iter()
                .position(|x| *x == id)
                .expect("Parent/children desync");
            up.extend(children[i..].iter().rev());
            up
        } else {
            vec![id]
        }
    }

    pub fn rev_tree_starting_at(&self, id: Id) -> Vec<Id> {
        debug_assert!(self[id].active);
        if let Some(parent) = self[id].parent {
            let mut up = self.rev_tree_starting_at(parent);
            up.pop();
            let i = self
                .get_children(parent)
                .iter()
                .position(|x| *x == id)
                .expect("Parent/children desync");
            up.extend(self[parent].children[..=i].iter());
            up
        } else {
            vec![id]
        }
    }
}
impl std::ops::Index<Id> for Controls {
    type Output = Control;
    fn index(&self, id: Id) -> &Self::Output {
        debug_assert!(
            self.controls[id.index()].generation == id.generation, "The Control in index {} and generation {} is not alive anymore. Current generation is {}", id.index(), id.generation(), self.controls[id.index()].generation);
        &self.controls[id.index()]
    }
}
impl std::ops::IndexMut<Id> for Controls {
    fn index_mut(&mut self, id: Id) -> &mut Self::Output {
        debug_assert!(
            self.controls[id.index()].generation == id.generation, "The Control in index {} and generation {} is not alive anymore. Current generation is {}", id.index(), id.generation(), self.controls[id.index()].generation);
        &mut self.controls[id.index()]
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

#[derive(PartialEq, Eq)]
pub(crate) enum ControlState {
    /// It is free to be created, there is no valid Id pointing to It.
    Free,
    /// reserve() was called, so there is a Id refering it, but the control is not yet alive
    Reserved,
    /// A ControlBuilder has been created, refering this Control, but the control is not yet alive
    Building,
    /// The control is alive, and exist in the GUI tree.
    Started,
}
impl std::fmt::Display for ControlState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlState::Free => write!(f, "Free"),
            ControlState::Reserved => write!(f, "Reserved"),
            ControlState::Building => write!(f, "Building"),
            ControlState::Started => write!(f, "Started"),
        }
    }
}

pub(crate) struct Control {
    pub(crate) generation: NonZeroU32,
    pub(crate) rect: Rect,
    pub(crate) graphic: Graphic,
    pub(crate) behaviour: Option<Box<dyn Behaviour>>,
    pub(crate) layout: Box<dyn Layout>,
    pub(crate) parent: Option<Id>,
    pub(crate) children: Vec<Id>,
    pub(crate) active: bool,
    pub(crate) focus: bool,
    pub(crate) really_active: bool,
    pub(crate) state: ControlState,
}
impl Default for Control {
    fn default() -> Self {
        Self {
            generation: NonZeroU32::new(1).unwrap(),
            rect: Default::default(),
            graphic: Default::default(),
            behaviour: Default::default(),
            layout: Default::default(),
            parent: Default::default(),
            children: Default::default(),
            focus: Default::default(),
            active: Default::default(),
            really_active: Default::default(),
            state: ControlState::Free,
        }
    }
}
impl Control {

    pub fn set_layout(&mut self, layout: Box<dyn Layout>) {
        self.layout = layout;
    }

    pub fn add_child(&mut self, child: Id) {
        if !self.children.iter().any(|x| *x == child) {
            self.children.push(child)
        }
    }
}
