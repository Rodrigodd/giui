use std::cell::RefCell;
use std::{num::NonZeroU32, rc::Rc};

use crate::{graphics::Graphic, Behaviour, Id, Layout, Rect, RectFill};

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
        if let ControlEntry::Reserved {
            generation,
            children,
        } = &mut controls.controls[id.index()]
        {
            debug_assert_eq!(*generation, id.generation);
            let mut control = Control::new(*generation);
            control.active = true;
            control.children = std::mem::take(children);
            controls.controls[id.index()] = ControlEntry::Building { control };
        } else {
            panic!("Building Control that isn't in Reserved State")
        }
        Self {
            inner: Box::new(inner),
            id,
        }
    }

    pub fn reserve(&mut self) -> Id {
        self.inner.controls().reserve()
    }

    /// Return the Id of the control that this ControlBuilder is building
    pub fn id(&self) -> Id {
        self.id
    }

    fn control(&mut self, id: Id) -> &mut Control {
        self.inner.controls().get_mut(id).unwrap()
    }

    pub fn anchors(mut self, anchors: [f32; 4]) -> Self {
        self.control(self.id).rect.anchors = anchors;
        self
    }
    pub fn margins(mut self, margins: [f32; 4]) -> Self {
        self.control(self.id).rect.margins = margins;
        self
    }
    pub fn min_size(mut self, min_size: [f32; 2]) -> Self {
        self.control(self.id).rect.user_min_size = min_size;
        self.control(self.id).rect.min_size = min_size;
        self
    }
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.control(self.id).rect.min_size[0] = min_width;
        self
    }
    pub fn min_height(mut self, min_height: f32) -> Self {
        self.control(self.id).rect.min_size[1] = min_height;
        self
    }
    pub fn fill_x(mut self, fill: RectFill) -> Self {
        self.control(self.id).rect.set_fill_x(fill);
        self
    }
    pub fn fill_y(mut self, fill: RectFill) -> Self {
        self.control(self.id).rect.set_fill_y(fill);
        self
    }
    pub fn expand_x(mut self, expand: bool) -> Self {
        self.control(self.id).rect.expand_x = expand;
        self
    }
    pub fn expand_y(mut self, expand: bool) -> Self {
        self.control(self.id).rect.expand_y = expand;
        self
    }
    pub fn behaviour<T: Behaviour + 'static>(mut self, behaviour: T) -> Self {
        // TODO: remove this someday
        debug_assert!(self.control(self.id).behaviour.is_none());
        self.control(self.id).behaviour = Some(Box::new(behaviour));
        self
    }
    pub fn layout<T: Layout + 'static>(mut self, layout: T) -> Self {
        self.control(self.id).layout = Some(Box::new(layout));
        self
    }
    pub fn behaviour_and_layout<T: Layout + Behaviour + 'static>(
        self,
        behaviour_layout: T,
    ) -> Self {
        let x = Rc::new(RefCell::new(behaviour_layout));
        self.behaviour(x.clone()).layout(x)
    }
    pub fn graphic(mut self, graphic: Graphic) -> Self {
        self.control(self.id).graphic = graphic;
        self
    }
    pub fn parent(mut self, parent: Id) -> Self {
        self.control(self.id).parent = Some(parent);
        self
    }
    pub fn active(mut self, active: bool) -> Self {
        self.control(self.id).active = active;
        self
    }

    pub fn child<F>(mut self, create_child: F) -> Self
    where
        F: for<'b> FnOnce(ControlBuilder<'b>) -> ControlBuilder<'b>,
    {
        let id = self.inner.controls().reserve();
        self.child_reserved(id, create_child)
    }

    pub fn child_reserved<F>(mut self, id: Id, create_child: F) -> Self
    where
        F: for<'b> FnOnce(ControlBuilder<'b>) -> ControlBuilder<'b>,
    {
        // let id = self.inner.controls().reserve();
        let parent = self.id;

        // while creating a child, be sure that it see its parent as deactive
        let active = std::mem::replace(&mut self.control(parent).active, false);

        {
            struct ChildBuilderInner<'a>(&'a mut Controls);
            impl ControlBuilderInner for ChildBuilderInner<'_> {
                fn controls(&mut self) -> &mut Controls {
                    self.0
                }
                fn build(&mut self, _id: Id) {}
            }
            let child_builder = ControlBuilder::new(id, ChildBuilderInner(self.inner.controls()));
            (create_child)(child_builder).parent(parent).build();
        }

        // restore the parent active
        self.control(parent).active = active;

        self
    }

    pub fn build(mut self) -> Id {
        let id = self.id;

        if let Some(parent) = self.control(id).parent {
            match self.inner.controls().get_mut(parent) {
                Some(x) => x.add_child(id),
                // the parent could be in Reserved state
                None => match &mut self.inner.controls().controls[parent.index()] {
                    ControlEntry::Reserved { children, .. } => {
                        children.push(id);
                    }
                    _ => panic!("Control's parent state is invalid. It is Free or Take"),
                },
            }
        } else {
            self.control(id).parent = Some(Id::ROOT_ID);
            self.control(Id::ROOT_ID).add_child(id);
        }

        if self.control(id).active
            && self.control(id).parent.map_or(true, |x| {
                self.inner
                    .controls()
                    .get(x)
                    .map_or(false, |x| x.really_active)
            })
        {
            self.control(id).really_active = true;
        }

        let Self { mut inner, id } = self;
        inner.build(id);
        id
    }
}

pub(crate) enum ControlEntry {
    /// The entry is free, and can be occupy by a new Control. There is no valid Id pointing to it.
    Free { free_next: Option<u32> },
    /// reserve() has been called, so there is Id pointing to it, but the control is not yet alive.
    /// Can already build controls with this as parent.
    Reserved {
        generation: NonZeroU32,
        children: Vec<Id>,
    },
    /// A ControlBuilder for this control has been created, but the control is not yet alive.
    Building { control: Control },
    /// The control is alive, and exist in the Gui Tree.
    Started { control: Control },
    /// The control has been temporarily taken, to allow transitioning beetween states.
    Take,
}

impl std::fmt::Debug for ControlEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlEntry::Free { free_next } => write!(f, "Free {{ free_next: {:?} }}", free_next),
            ControlEntry::Reserved {
                generation,
                children,
            } => write!(
                f,
                "Reserved {{ generation: {:?}, children: {:?} }}",
                generation, children
            ),
            ControlEntry::Building { control } => {
                write!(f, "Building {{ generation: {:?}, .. }}", control.generation)
            }
            ControlEntry::Started { control } => {
                write!(f, "Started {{ generation: {:?}, .. }}", control.generation)
            }
            ControlEntry::Take => write!(f, "Take"),
        }
    }
}
impl Default for ControlEntry {
    fn default() -> Self {
        Self::Take
    }
}
impl ControlEntry {
    fn get(&self, id: Id) -> Option<&Control> {
        match self {
            ControlEntry::Building { control } | ControlEntry::Started { control } => {
                if control.generation != id.generation {
                    return None;
                }
                Some(control)
            }
            ControlEntry::Free { .. } | ControlEntry::Reserved { .. } => None,
            ControlEntry::Take => {
                debug_assert!(false, "a entry should not being Take for much time");
                None
            }
        }
    }

    fn get_mut(&mut self, id: Id) -> Option<&mut Control> {
        match self {
            ControlEntry::Building { control } | ControlEntry::Started { control } => {
                if control.generation != id.generation {
                    return None;
                }
                Some(control)
            }
            ControlEntry::Free { .. } | ControlEntry::Reserved { .. } => None,
            ControlEntry::Take => {
                debug_assert!(false, "a entry should not being Take for much time");
                None
            }
        }
    }
}

/// Return the next avaliable generation. The next generation is a global value, which means that a
/// Id will be unique for every Instance of Controls. In case the value overflows it wraps to 1,
/// wich means that after 4_294_967_295 generations, invalid Id's are possible.
fn next_generation() -> NonZeroU32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static GENERATION: AtomicU32 = AtomicU32::new(1);
    let mut next = GENERATION.fetch_add(1, Ordering::Relaxed);
    if next == 0 {
        next = GENERATION.fetch_add(1, Ordering::Relaxed);
    }
    NonZeroU32::new(next).unwrap()
}

pub(crate) struct Controls {
    free_head: Option<u32>,
    pub(crate) controls: Vec<ControlEntry>,
    generation: NonZeroU32,
}
impl Controls {
    /// Create a new Controls, with a single ROOT Control with the given width and height.
    pub fn new(width: f32, height: f32) -> Self {
        let root = Control {
            rect: Rect {
                anchors: [0.0; 4],
                margins: [0.0; 4],
                min_size: [width, height],
                rect: [0.0, 0.0, width, height],
                ..Default::default()
            },
            active: true,
            really_active: true,
            ..Control::new(NonZeroU32::new(1).unwrap())
        };
        Self {
            free_head: None,
            controls: vec![ControlEntry::Started { control: root }],
            generation: next_generation(),
        }
    }

    pub fn get(&self, id: Id) -> Option<&Control> {
        if let Some(control) = self.controls.get(id.index()) {
            control.get(id)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut Control> {
        if let Some(control) = self.controls.get_mut(id.index()) {
            control.get_mut(id)
        } else {
            None
        }
    }

    pub fn reserve(&mut self) -> Id {
        if let Some(index) = self.free_head {
            match self.controls[index as usize] {
                ControlEntry::Free {
                    free_next: next_free,
                } => {
                    self.free_head = next_free;
                    self.controls[index as usize] = ControlEntry::Reserved {
                        generation: self.generation,
                        children: Vec::new(),
                    };
                    Id {
                        generation: self.generation,
                        index,
                    }
                }
                _ => panic!("Controls is corrupted. Entry in free list is not free"),
            }
        } else {
            self.controls.push(ControlEntry::Reserved {
                generation: self.generation,
                children: Vec::new(),
            });
            Id {
                generation: self.generation,
                index: self.controls.len() as u32 - 1,
            }
        }
    }

    #[allow(clippy::or_fun_call)]
    pub fn remove(&mut self, id: Id) {
        self.generation = next_generation();
        self.controls[id.index()] = ControlEntry::Free {
            free_next: self.free_head,
        };
        self.free_head = Some(id.index);
    }

    pub fn move_to_front(&mut self, id: Id) {
        if let Some(parent) = self.get(id).and_then(|x| x.parent) {
            let children = &mut self
                .get_mut(parent)
                .expect("Control's parent is unintialized")
                .children;
            let i = children.iter().position(|x| *x == id).unwrap();
            children.remove(i);
            children.push(id);
        }
    }

    pub fn move_to_back(&mut self, id: Id) {
        if let Some(parent) = self.get(id).and_then(|x| x.parent) {
            let children = &mut self
                .get_mut(parent)
                .expect("Control's parent is unintialized")
                .children;
            let i = children.iter().position(|x| *x == id).unwrap();
            children.remove(i);
            children.insert(0, id);
        }
    }

    pub fn is_child(&mut self, parent: Id, child: Id) -> bool {
        Some(parent) == self.get(child).and_then(|x| x.parent)
    }

    pub fn is_descendant(&mut self, ascendant: Id, descendant: Id) -> bool {
        let mut curr = descendant;
        while let Some(parent) = self.get(curr).and_then(|x| x.parent) {
            if parent == ascendant {
                return true;
            }
            curr = parent;
        }
        false
    }

    pub fn get_active_children(&self, id: Id) -> Option<Vec<Id>> {
        Some(
            self.get(id)?
                .children
                .iter()
                .filter(|x| self.get(**x).expect("Parent-child desync").active)
                .cloned()
                .collect::<Vec<Id>>(),
        )
    }

    pub fn get_control_stack(&self, id: Id) -> Option<Vec<Id>> {
        if self.get(id).is_none() {
            return None;
        }
        let mut curr = id;
        let mut stack = vec![curr];
        while let Some(parent) = self.get(curr).expect("Parent-child desync").parent {
            curr = parent;
            stack.push(curr);
        }
        Some(stack)
    }

    /// Return the id of the lowest common ancestor of both controls. This is used to only update
    /// the focus flag of the controls that changed.
    pub fn lowest_common_ancestor(&self, a: Id, b: Id) -> Option<Id> {
        let a_stack = self.get_control_stack(a)?;
        let b_stack = self.get_control_stack(b)?;
        // lowest common anscertor
        a_stack
            .iter()
            .rev()
            .zip(b_stack.iter().rev())
            .take_while(|(a, b)| *a == *b)
            .last()
            .map(|(a, _)| *a)
    }

    /// Return the state of the vector used to traverse a tree in order, when the next control is
    /// the given one.
    pub fn tree_starting_at(&self, id: Id) -> Option<Vec<Id>> {
        debug_assert!(self.get(id).unwrap().active);
        if let Some(parent) = self.get(id)?.parent {
            let mut up = self.tree_starting_at(parent).unwrap();
            up.pop();
            let children = self.get_active_children(parent).unwrap();
            let i = children
                .iter()
                .position(|x| *x == id)
                .expect("Parent/children desync");
            up.extend(children[i..].iter().rev());
            Some(up)
        } else {
            Some(vec![id])
        }
    }
    /// Return the state of the vector used to traverse a tree in reverse order, when the next
    /// control is the given one.
    pub fn rev_tree_starting_at(&self, id: Id) -> Option<Vec<Id>> {
        debug_assert!(self.get(id).unwrap().active);
        if let Some(parent) = self.get(id)?.parent {
            let mut up = self.rev_tree_starting_at(parent).unwrap();
            up.pop();
            let children = self.get_active_children(parent).unwrap();
            let i = children
                .iter()
                .position(|x| *x == id)
                .expect("Parent/children desync");
            up.extend(children[..=i].iter());
            Some(up)
        } else {
            Some(vec![id])
        }
    }
}

pub(crate) struct Control {
    pub(crate) generation: NonZeroU32,
    pub(crate) rect: Rect,
    pub(crate) graphic: Graphic,
    pub(crate) behaviour: Option<Box<dyn Behaviour>>,
    // Every control has a layout. This is a Option only to allow to temporarily take owership of it.
    pub(crate) layout: Option<Box<dyn Layout>>,
    pub(crate) parent: Option<Id>,
    pub(crate) children: Vec<Id>,
    pub(crate) active: bool,
    pub(crate) focus: bool,
    pub(crate) really_active: bool,
}
impl Control {
    fn new(generation: NonZeroU32) -> Self {
        Self {
            generation,
            rect: Default::default(),
            graphic: Default::default(),
            behaviour: Default::default(),
            layout: Some(Box::new(())),
            parent: Default::default(),
            children: Default::default(),
            focus: Default::default(),
            active: Default::default(),
            really_active: Default::default(),
        }
    }
}
impl Control {
    pub fn add_child(&mut self, child: Id) {
        if !self.children.iter().any(|x| *x == child) {
            self.children.push(child)
        }
    }
}
