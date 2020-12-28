use crate::{render::Graphic, Behaviour, Id, Layout, Rect, RectFill};

pub(crate) struct ControlBuild {
    pub rect: Rect,
    pub graphic: Graphic,
    pub behaviour: Option<Box<dyn Behaviour>>,
    pub layout: Box<dyn Layout>,
    pub parent: Option<Id>,
    pub active: bool,
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
pub struct ControlBuilder<'a> {
    builder: Box<dyn FnOnce(ControlBuild) -> Id + 'a>,
    build: ControlBuild,
}
impl<'a> ControlBuilder<'a> {
    pub(crate) fn new(builder: Box<dyn FnOnce(ControlBuild) -> Id + 'a>) -> Self {
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
    pub fn with_behaviour<T: Behaviour + 'static>(mut self, behaviour: T) -> Self {
        // TODO: remove this in production!!
        debug_assert!(self.build.behaviour.is_none());
        self.build.behaviour = Some(Box::new(behaviour));
        self
    }
    pub fn with_layout<T: Layout + 'static>(mut self, layout: T) -> Self {
        self.build.layout = Box::new(layout);
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
pub(crate) struct Controls {
    dead_controls: Vec<u32>,
    controls: Vec<Control>,
}
impl Controls {
    pub fn reserve(&mut self) -> Id {
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

    pub fn remove(&mut self, id: Id) {
        self[id] = Control {
            generation: self[id].generation + 1,
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

#[derive(Default)]
pub(crate) struct Control {
    pub(crate) generation: u32,
    pub(crate) rect: Rect,
    pub(crate) graphic: Graphic,
    pub(crate) behaviour: Option<Box<dyn Behaviour>>,
    pub(crate) layout: Box<dyn Layout>,
    pub(crate) parent: Option<Id>,
    pub(crate) children: Vec<Id>,
    pub(crate) active: bool,
}
impl Control {
    /// add one more behaviour to the control
    pub fn set_behaviour(&mut self, behaviour: Box<dyn Behaviour>) {
        self.behaviour = Some(behaviour);
    }

    pub fn set_layout(&mut self, layout: Box<dyn Layout>) {
        self.layout = layout;
    }

    pub fn add_children(&mut self, child: Id) {
        if !self.children.iter().any(|x| *x == child) {
            self.children.push(child)
        }
    }

    /// Set the widget with that id to active = true.
    /// Return true if the active was false.
    pub fn active(&mut self) -> bool {
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
    pub fn deactive(&mut self) -> bool {
        if self.active {
            self.active = false;
            true
        } else {
            false
        }
    }
}
