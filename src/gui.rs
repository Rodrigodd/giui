use crate::{
    context::{Context, LayoutContext, MinSizeContext},
    render::Graphic,
    Control, ControlBuild, ControlBuilder, Controls, LayoutDirtyFlags, Rect,
};
use ab_glyph::FontArc;
use std::any::Any;
use winit::{
    event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
    window::CursorIcon,
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
    pub struct SetValue<T>(pub T);

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
    pub(crate) index: u32,
    pub(crate) generation: u32,
}
impl Id {
    /// Get the index of the control in the controls vector inside GUI<R>
    pub fn index(&self) -> usize {
        self.index as usize
    }
    /// Get the generation of the control it is refering
    pub fn generation(&self) -> u32 {
        self.generation as u32
    }
}

// pub struct Mouse {
//     pos: [f32; 2],
//     event: MouseEvent,
// }

#[derive(Clone, Copy, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}
impl From<winit::event::MouseButton> for MouseButton {
    fn from(x: winit::event::MouseButton) -> MouseButton {
        match x {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Other(x) => MouseButton::Other(x),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MouseEvent {
    Enter,
    Exit,
    Down(MouseButton),
    Up(MouseButton),
    Moved { x: f32, y: f32 },
}

#[derive(Copy, Clone)]
pub enum KeyboardEvent {
    Char(char),
    Pressed(VirtualKeyCode),
}

#[derive(Default)]
struct Input {
    mouse_x: f32,
    mouse_y: f32,
    mouse_invalid: bool,
}

pub struct GUI {
    pub(crate) controls: Controls,
    pub(crate) fonts: Vec<FontArc>,
    pub(crate) modifiers: ModifiersState,
    redraw: bool,
    change_cursor: Option<CursorIcon>,
    input: Input,
    current_mouse: Option<Id>,
    current_scroll: Option<Id>,
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
                really_active: true,
            }]
            .into(),
            redraw: true,
            change_cursor: None,
            input: Input::default(),
            current_mouse: None,
            current_scroll: None,
            current_focus: None,
            over_is_locked: false,
            fonts,
        }
    }

    fn get_parent(&self, id: Id) -> Option<Id> {
        self.controls[id].parent
    }

    fn get_children(&self, id: Id) -> Vec<Id> {
        self.controls.get_children(id)
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

    fn add_control(&mut self, build: ControlBuild) -> Id {
        let reserve = self.controls.reserve();
        self.add_control_reserved(build, reserve)
    }

    fn add_control_reserved(&mut self, build: ControlBuild, reserve: Id) -> Id {
        let ControlBuild {
            rect,
            graphic,
            behaviour,
            layout,
            parent,
            active,
        } = build;
        let this = reserve;
        let has_behaviour = behaviour.is_some();

        let mut control = &mut self.controls[this];
        control.rect = rect;
        control.graphic = graphic;
        control.behaviour = behaviour;
        control.layout = layout;
        control.parent = parent;
        // control.active = active;

        assert_eq!(control.generation, this.generation);

        if let Some(parent) = control.parent {
            self.controls[parent].add_child(this);
        } else {
            control.parent = Some(ROOT_ID);
            self.controls[ROOT_ID].add_child(this);
        }

        if has_behaviour {
            self.call_event(this, |this, id, ctx| this.on_start(id, ctx));
        }

        if active {
            self.active_control(this);
        }

        self.update_layout(this);
        this
    }

    pub fn active_control(&mut self, id: Id) {
        if self.controls[id].active {
            return;
        }
        self.controls[id].active = true;

        if let Some(parent) = self.get_parent(id) {
            self.update_layout(parent);
        }
        if self
            .get_parent(id)
            .map(|x| self.controls[x].really_active)
            .unwrap_or(true)
        {
            self.controls[id].really_active = true;
            let mut parents = vec![id];
            while let Some(id) = parents.pop() {
                parents.extend(self.get_children(id).iter().rev());
                self.controls[id].really_active = true;
                self.call_event(id, |this, id, ctx| this.on_active(id, ctx));
            }
        }
        // uncommenting the line below allow infinity recursion to happen
        // self.mouse_moved(self.input.mouse_x, self.input.mouse_y);
    }

    pub fn deactive_control(&mut self, id: Id) {
        if !self.controls[id].active {
            return;
        }
        self.controls[id].active = false;

        if let Some(parent) = self.get_parent(id) {
            self.update_layout(parent);
        }
        if self
            .get_parent(id)
            .map(|x| self.controls[x].really_active)
            .unwrap_or(true)
        {
            self.controls[id].really_active = false;
            let mut parents = vec![id];
            while let Some(id) = parents.pop() {
                parents.extend(self.get_children(id).iter().rev());
                if Some(id) == self.current_mouse {
                    self.send_mouse_event_to(id, MouseEvent::Exit);
                    self.current_mouse = None;
                    self.input.mouse_invalid = true;
                }
                if Some(id) == self.current_scroll {
                    self.current_scroll = None;
                }
                if Some(id) == self.current_focus {
                    self.set_focus(None);
                }
                self.controls[id].really_active = false;
                self.call_event(id, |this, id, ctx| this.on_deactive(id, ctx));
            }
        }
        // uncommenting the line below allow infinity recursion to happen
        // self.mouse_moved(self.input.mouse_x, self.input.mouse_y);
    }

    /// Remove a control and all of its children
    pub fn remove_control(&mut self, id: Id) {
        if self.controls[id].active {
            self.controls[id].active = false;
            if let Some(parent) = self.get_parent(id) {
                self.update_layout(parent);
            }
        }
        if let Some(parent) = self.controls[id].parent {
            let children = &mut self.controls[parent].children;
            if let Some(pos) = children.iter().position(|x| *x == id) {
                children.remove(pos);
            }
        }

        let mut parents = vec![id];
        while let Some(id) = parents.pop() {
            parents.extend(self.get_children(id).iter().rev());

            if Some(id) == self.current_mouse {
                self.send_mouse_event_to(id, MouseEvent::Exit);
                self.current_mouse = None;
                self.input.mouse_invalid = true;
            }
            if Some(id) == self.current_scroll {
                self.current_scroll = None;
            }
            if Some(id) == self.current_focus {
                self.set_focus(None);
            }
            if self.controls[id].really_active {
                self.call_event(id, |this, id, ctx| this.on_deactive(id, ctx));
            }
        }
        let mut parents = vec![id];
        while let Some(id) = parents.pop() {
            parents.extend(self.get_children(id).iter().rev());
            self.controls.remove(id);
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

    pub fn cursor_change(&mut self) -> Option<CursorIcon> {
        self.change_cursor.take()
    }

    #[inline]
    pub fn get_context(&mut self) -> Context {
        //TODO: Context -> RenderContext
        self.redraw = false;
        Context::new(self)
    }

    pub(crate) fn context_drop(
        &mut self,
        events: &mut Vec<Box<dyn Any>>,
        events_to: &mut Vec<(Id, Box<dyn Any>)>,
        dirtys: &mut Vec<Id>,
        render_dirty: bool,
    ) {
        if render_dirty {
            self.redraw = true;
        }
        for event in events.drain(..) {
            self.send_event(event);
        }
        for (id, event) in events_to.drain(..) {
            self.send_event_to(id, event);
        }
        for dirty in dirtys.drain(..) {
            self.update_layout(dirty);
        }
    }

    pub fn set_behaviour<T: Behaviour + 'static>(&mut self, id: Id, behaviour: T) {
        if self.controls[id].really_active {
            self.call_event(id, |this, id, ctx| this.on_deactive(id, ctx));
        }
        self.controls[id].set_behaviour(Box::new(behaviour));
        self.call_event(id, |this, id, ctx| this.on_start(id, ctx));
        if self.controls[id].really_active {
            self.call_event(id, |this, id, ctx| this.on_active(id, ctx));
        }
    }

    pub fn set_layout<T: Layout + 'static>(&mut self, id: Id, layout: T) {
        self.controls[id].set_layout(Box::new(layout));
    }

    pub fn get_graphic(&mut self, id: Id) -> &mut Graphic {
        &mut self.controls[id].graphic
    }

    pub fn get_rect(&self, id: Id) -> &Rect {
        &self.controls[id].rect
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.controls[ROOT_ID]
            .rect
            .set_rect([0.0, 0.0, width, height]);
        self.update_layout(ROOT_ID);
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
        } else if let Some(cursor) = event.downcast_ref::<CursorIcon>() {
            self.change_cursor = Some(*cursor);
        }
    }

    pub fn send_event_to(&mut self, id: Id, event: Box<dyn Any>) {
        if let Some((this, mut ctx)) = Context::new_with_mut_behaviour(id, self) {
            this.on_event(event.as_ref(), id, &mut ctx);
        }
    }

    pub fn call_event<F: Fn(&mut dyn Behaviour, Id, &mut Context)>(&mut self, id: Id, event: F) {
        if let Some((this, mut ctx)) = Context::new_with_mut_behaviour(id, self) {
            event(this, id, &mut ctx);
        }
    }

    pub fn call_event_chain<F: Fn(&mut dyn Behaviour, Id, &mut Context) -> bool>(
        &mut self,
        id: Id,
        event: F,
    ) {
        let mut handled = false;
        if let Some((this, mut ctx)) = Context::new_with_mut_behaviour(id, self) {
            handled = event(this, id, &mut ctx);
        }
        if !handled {
            if let Some(parent) = self.controls[id].parent {
                self.call_event_chain(parent, event);
            }
        }
    }

    pub fn start(&mut self) {
        self.update_all_layouts();
        fn print_tree(deep: usize, id: Id, gui: &mut GUI) {
            let childs = gui.controls[id].children.clone();
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
        match event {
            Event::MainEventsCleared => {
                if self.input.mouse_invalid {
                    self.input.mouse_invalid = false;
                    self.mouse_moved(self.input.mouse_x, self.input.mouse_y);
                }
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.input.mouse_x = position.x as f32;
                    self.input.mouse_y = position.y as f32;
                    self.mouse_moved(position.x as f32, position.y as f32);
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if let ElementState::Pressed = state {
                        self.set_focus(self.current_mouse);
                    }
                    if let Some(curr) = self.current_mouse {
                        match state {
                            ElementState::Pressed => {
                                self.send_mouse_event_to(curr, MouseEvent::Down((*button).into()));
                            }
                            ElementState::Released => {
                                self.send_mouse_event_to(curr, MouseEvent::Up((*button).into()));
                            }
                        };
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    if let Some(curr) = self.current_mouse {
                        //TODO: I should handle Line and Pixel Delta differences more wisely?
                        let delta = match delta {
                            winit::event::MouseScrollDelta::LineDelta(x, y) => {
                                [*x * 100.0, *y * 100.0]
                            }
                            winit::event::MouseScrollDelta::PixelDelta(p) => {
                                [p.x as f32, p.y as f32]
                            }
                        };
                        self.call_event(curr, |this, id, ctx| this.on_scroll_event(delta, id, ctx));
                    }
                }
                WindowEvent::CursorLeft { .. } => {
                    if !self.over_is_locked {
                        if let Some(curr) = self.current_mouse.take() {
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
            },
            _ => {}
        }
    }

    pub fn set_focus(&mut self, id: Option<Id>) {
        if id == self.current_focus {
            return;
        }

        if let (Some(prev), Some(next)) = (self.current_focus, id) {
            let lca = self.controls.lowest_common_ancestor(prev, next);

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
        if self.current_mouse.is_some() && self.over_is_locked {
            self.send_mouse_event_to(
                self.current_mouse.unwrap(),
                MouseEvent::Moved {
                    x: mouse_x,
                    y: mouse_y,
                },
            );
            return;
        }

        let mut curr = ROOT_ID;
        let mut curr_mouse = None;
        'l: loop {
            if let Some(flags) = self.controls[curr]
                .behaviour
                .as_ref()
                .map(|x| x.input_flags())
            {
                if flags.contains(InputFlags::MOUSE) {
                    curr_mouse = Some(curr);
                }
                if flags.contains(InputFlags::SCROLL) {
                    self.current_scroll = Some(curr);
                }
            }
            // the interator is reversed because the last child block the previous ones
            for child in self.get_children(curr).iter().rev() {
                if self.controls[*child].rect.contains(mouse_x, mouse_y) {
                    curr = *child;
                    continue 'l;
                }
            }
            break;
        }

        if curr_mouse == self.current_mouse {
            if let Some(current_mouse) = self.current_mouse {
                self.send_mouse_event_to(
                    current_mouse,
                    MouseEvent::Moved {
                        x: mouse_x,
                        y: mouse_y,
                    },
                );
            }
        } else {
            if let Some(current_mouse) = self.current_mouse {
                self.send_mouse_event_to(current_mouse, MouseEvent::Exit);
            }
            self.current_mouse = curr_mouse;
            if let Some(current_mouse) = self.current_mouse {
                self.send_mouse_event_to(current_mouse, MouseEvent::Enter);
                self.send_mouse_event_to(
                    current_mouse,
                    MouseEvent::Moved {
                        x: mouse_x,
                        y: mouse_y,
                    },
                );
            }
        }
    }

    pub fn send_mouse_event_to(&mut self, id: Id, event: MouseEvent) {
        self.call_event(id, |this, id, ctx| this.on_mouse_event(event, id, ctx));
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
            self.controls[id]
                .rect
                .layout_dirty_flags
                .insert(LayoutDirtyFlags::DIRTY);
            if self.controls[id]
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
                        self.controls[*id].active = false;
                    } else if let Some(event::ActiveControl { id }) = event.downcast_ref() {
                        // self.active_control(*id)
                        self.controls[*id].active = true;
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
                if !self.controls[*child]
                    .rect
                    .get_layout_dirty_flags()
                    .is_empty()
                {
                    parents.push(*child);
                    self.controls[*child].rect.clear_layout_dirty_flags();
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
                        self.controls[*id].active = false;
                    } else if let Some(event::ActiveControl { id }) = event.downcast_ref() {
                        // self.active_control(*id)
                        self.controls[*id].active = true;
                    }
                }
            }
            parents.extend(self.get_children(parent).iter().rev());
        }
    }
}

bitflags! {
    pub struct InputFlags: u8 {
        const MOUSE = 0x1;
        const SCROLL = 0x2;
    }
}

#[allow(unused_variables)]
pub trait Behaviour {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {}
    fn on_active(&mut self, this: Id, ctx: &mut Context) {}
    fn on_deactive(&mut self, this: Id, ctx: &mut Context) {}

    fn input_flags(&self) -> InputFlags {
        InputFlags::empty()
    }

    fn on_event(&mut self, event: &dyn Any, this: Id, ctx: &mut Context) {}

    fn on_scroll_event(&mut self, delta: [f32; 2], this: Id, ctx: &mut Context) {}

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) {}

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
