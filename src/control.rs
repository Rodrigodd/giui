use std::{
    any::{Any, TypeId},
    cell::RefCell,
    num::NonZeroU32,
    rc::Rc,
};

use crate::{graphics::Graphic, Behaviour, Id, Layout, Rect, RectFill};

pub trait BuilderContext {
    /// Get a reference to the value of type T that is owned by the Gui.
    /// # Panics
    /// Panics if the value was not set before hand
    fn get_from_type_id(&self, type_id: TypeId) -> &dyn Any;

    fn create_control(&mut self) -> ControlBuilder {
        let id = self.controls_mut().reserve();
        self.create_control_reserved(id)
    }

    /// Create a new ControlBuilder, that build a Control with the given Id. The given Id must be a
    /// reserved Id, with no other control already build with this Id.
    fn create_control_reserved(&mut self, id: Id) -> ControlBuilder {
        ControlBuilder::new(self, id)
    }

    fn reserve(&mut self) -> Id {
        self.controls_mut().reserve()
    }

    fn get_graphic_mut(&mut self, id: Id) -> &mut Graphic;

    fn get_all_children(&self, id: Id) -> &[Id] {
        self.controls().get_all_children(id).unwrap()
    }

    fn get_active_children(&self, id: Id) -> Vec<Id> {
        self.controls().get_active_children(id).unwrap()
    }

    #[doc(hidden)]
    fn controls(&self) -> &Controls;
    #[doc(hidden)]
    fn controls_mut(&mut self) -> &mut Controls;
    #[doc(hidden)]
    /// This need to add to control to Controls, send a Start event, and a FocusRequest event when
    /// necessary
    fn build(&mut self, id: Id, control: Control);
}
impl<'a> dyn BuilderContext + 'a {
    /// Get a reference to the value of type T that is owned by the Gui.
    /// # Panics
    /// Panics if the value was not set before hand
    pub fn get<T: Any + 'static>(&self) -> &T {
        let value = self.get_from_type_id(TypeId::of::<T>());
        value.downcast_ref::<T>().unwrap()
    }
}

pub struct ControlBuilder {
    id: Id,
    control: Control,
}
impl ControlBuilder {
    /// Create a new ControlBuilder, that build a Control with the given Id. The given Id must be a
    /// reserved Id, with no other control already build with this Id.
    pub(crate) fn new(ctx: &mut (impl BuilderContext + ?Sized), id: Id) -> Self {
        if let ControlEntry::Reserved { .. } = &mut ctx.controls_mut().controls[id.index()] {
        } else {
            panic!("Tried to create a control without a reserved Id")
        }
        let mut control = Control::new(id.generation);
        control.active = true;
        Self { id, control }
    }

    /// Return the Id of the control that this ControlBuilder is building
    pub fn id(&self) -> Id {
        self.id
    }

    pub fn anchors(mut self, anchors: [f32; 4]) -> Self {
        self.control.rect.anchors = anchors;
        self
    }
    pub fn margins(mut self, margins: [f32; 4]) -> Self {
        self.control.rect.margins = margins;
        self
    }
    pub fn min_size(mut self, min_size: [f32; 2]) -> Self {
        self.control.rect.user_min_size = min_size;
        self.control.rect.min_size = min_size;
        self
    }
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.control.rect.user_min_size[0] = min_width;
        self.control.rect.min_size[0] = min_width;
        self
    }
    pub fn min_height(mut self, min_height: f32) -> Self {
        self.control.rect.user_min_size[1] = min_height;
        self.control.rect.min_size[1] = min_height;
        self
    }
    pub fn fill_x(mut self, fill: RectFill) -> Self {
        self.control.rect.set_fill_x(fill);
        self
    }
    pub fn fill_y(mut self, fill: RectFill) -> Self {
        self.control.rect.set_fill_y(fill);
        self
    }
    pub fn expand_x(mut self, expand: bool) -> Self {
        self.control.rect.expand_x = expand;
        self
    }
    pub fn expand_y(mut self, expand: bool) -> Self {
        self.control.rect.expand_y = expand;
        self
    }
    pub fn behaviour<T: Behaviour + 'static>(mut self, behaviour: T) -> Self {
        // TODO: remove this someday
        debug_assert!(self.control.behaviour.is_none());
        self.control.behaviour = Some(Box::new(behaviour));
        self
    }
    pub fn layout<T: Layout + 'static>(mut self, layout: T) -> Self {
        self.control.layout = Some(Box::new(layout));
        self
    }
    pub fn behaviour_and_layout<T: Layout + Behaviour + 'static>(
        self,
        behaviour_layout: T,
    ) -> Self {
        let x = Rc::new(RefCell::new(behaviour_layout));
        self.behaviour(x.clone()).layout(x)
    }
    pub fn graphic(mut self, graphic: impl Into<Graphic>) -> Self {
        self.control.graphic = graphic.into();
        self
    }
    pub fn parent(mut self, parent: Id) -> Self {
        self.control.parent = Some(parent);
        self
    }
    pub fn active(mut self, active: bool) -> Self {
        self.control.active = active;
        self
    }

    /// If it is true, the focus will change to this control when builded.
    pub fn focus(mut self, focus: bool) -> Self {
        self.control.focus = focus;
        self
    }

    pub fn child<F>(self, ctx: &mut dyn BuilderContext, create_child: F) -> Self
    where
        F: FnOnce(ControlBuilder, &mut dyn BuilderContext) -> ControlBuilder,
    {
        let id = ctx.controls_mut().reserve();
        self.child_reserved(id, ctx, create_child)
    }

    pub fn child_reserved<F>(self, id: Id, ctx: &mut dyn BuilderContext, create_child: F) -> Self
    where
        F: FnOnce(ControlBuilder, &mut dyn BuilderContext) -> ControlBuilder,
    {
        let parent = self.id;

        {
            struct ChildBuilderContext<'a>(&'a mut dyn BuilderContext);
            impl BuilderContext for ChildBuilderContext<'_> {
                fn controls(&self) -> &Controls {
                    self.0.controls()
                }
                fn controls_mut(&mut self) -> &mut Controls {
                    self.0.controls_mut()
                }
                fn build(&mut self, id: Id, control: Control) {
                    self.0.build(id, control)
                }

                fn get_graphic_mut(&mut self, _: Id) -> &mut Graphic {
                    unimplemented!()
                }

                fn get_from_type_id(&self, _: TypeId) -> &dyn Any {
                    unimplemented!()
                }
            }
            let child_builder = ControlBuilder::new(ctx, id);
            (create_child)(child_builder, ctx)
                .parent(parent)
                .build(&mut ChildBuilderContext(ctx));
        }

        self
    }

    pub fn build(mut self, ctx: &mut (impl BuilderContext + ?Sized)) -> Id {
        // append childs that was created with this control as parent, by using the reserved Id.
        match &mut ctx.controls_mut().controls[self.id.index()] {
            ControlEntry::Reserved { children, .. } => self.control.children.append(children),
            _ => unreachable!(),
        }

        if let Some(parent) = self.control.parent {
            match &mut ctx.controls_mut().controls[parent.index()] {
                ControlEntry::Reserved { children, .. } => {
                    children.push(self.id);
                }
                ControlEntry::Builded { control, .. } | ControlEntry::Started { control, .. } => {
                    control.add_child(self.id)
                }
                _ => panic!("Control's parent state is invalid. It is Free or Take"),
            }
        } else {
            // The ROOT_ID is always in Started state.
            self.control.parent = Some(Id::ROOT_ID);
            ctx.controls_mut()
                .get_mut(Id::ROOT_ID)
                .unwrap()
                .add_child(self.id);
        }

        let Self { id, control } = self;
        ctx.build(id, control);
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
    /// This control has been builded, but the control is not yet alive. The control already exist
    /// in the Gui Tree.
    Builded { control: Control },
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
            ControlEntry::Builded { control } => {
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
            ControlEntry::Builded { control } | ControlEntry::Started { control } => {
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
            ControlEntry::Builded { control } | ControlEntry::Started { control } => {
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

#[doc(hidden)]
pub struct Controls {
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

    /// Change a control from reserved state to builded state
    pub fn add_builded_control(&mut self, id: Id, control: Control) {
        match &mut self.controls[id.index()] {
            ControlEntry::Free { .. } | ControlEntry::Take | ControlEntry::Builded { .. } => {
                panic!("added a control while not in reserved state")
            }
            ControlEntry::Reserved { generation, .. } => {
                assert_eq!(control.generation, *generation);
                self.controls[id.index()] = ControlEntry::Builded { control };
            }
            ControlEntry::Started { .. } => panic!("Control already started"),
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

    /// Move the Control with the given Id, to the last position in the children vector of its
    /// parent, making it render in front of all of its siblings.
    pub fn move_to_front(&mut self, id: Id) {
        if let Some(parent) = self.get(id).and_then(|x| x.parent) {
            let children = &mut self
                .get_mut(parent)
                .expect("Control's parent is unintialized")
                .children;
            let i = children.iter().position(|x| *x == id).unwrap();
            children[i..].rotate_left(1);
        }
    }

    /// Move the Control with the given Id, to the first position in the children vector of its
    /// parent, making it render behind all of its siblings.
    pub fn move_to_back(&mut self, id: Id) {
        if let Some(parent) = self.get(id).and_then(|x| x.parent) {
            let children = &mut self
                .get_mut(parent)
                .expect("Control's parent is unintialized")
                .children;
            let i = children.iter().position(|x| *x == id).unwrap();
            children[..=i].rotate_right(1);
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

    pub fn get_all_children(&self, id: Id) -> Option<&[Id]> {
        Some(&self.get(id)?.children)
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

#[doc(hidden)]
pub struct Control {
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
