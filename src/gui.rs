use crate::{
    context::{Context, LayoutContext, MinSizeContext},
    render::Graphic,
    Control, ControlBuild, ControlBuilder, Controls, LayoutDirtyFlags, Rect,
};
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
        let new = reserve;

        let mut control = &mut self.controls[new];
        control.rect = rect;
        control.graphic = graphic;
        control.behaviour = behaviour;
        control.layout = layout;
        control.parent = parent;
        control.active = active;

        assert_eq!(self.controls[new].generation, new.generation);
        // self.controls[new.get_index() as usize].generation = new.generation;

        let control = &mut self.controls[new];

        if let Some(parent) = control.parent {
            self.controls[parent].add_children(new);
        } else {
            control.parent = Some(ROOT_ID);
            self.controls[ROOT_ID].add_children(new);
        }

        if active {
            let mut parents = vec![new];
            while let Some(id) = parents.pop() {
                self.call_event(id, |this, id, ctx| this.on_start(id, ctx));
                self.call_event(id, |this, id, ctx| this.on_active(id, ctx));
                parents.extend(self.get_children(id).iter().rev());
            }
        }

        self.update_layout(new);
        new
    }

    pub fn active_control(&mut self, id: Id) {
        if !self.controls[id].active() {
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
        if !self.controls[id].deactive() {
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
                self.input.mouse_invalid = true;
            }
            if Some(id) == self.current_focus {
                self.set_focus(None);
            }
            self.call_event(id, |this, id, ctx| this.on_deactive(id, ctx));
        }
        // uncommenting the line below allow infinity recursion to happen
        // self.mouse_moved(self.input.mouse_x, self.input.mouse_y);
    }

    /// Remove a control and all of its children
    pub fn remove_control(&mut self, id: Id) {
        if self.controls[id].deactive() {
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

            if Some(id) == self.current_over {
                self.send_mouse_event_to(id, MouseEvent::Exit);
                self.current_over = None;
                self.input.mouse_invalid = true;
            }
            if Some(id) == self.current_focus {
                self.set_focus(None);
            }
            self.call_event(id, |this, id, ctx| this.on_deactive(id, ctx));
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

    #[inline]
    pub fn get_render_context(&mut self) -> Context {
        //TODO: Context -> RenderContext
        self.redraw = false;
        Context::new(&mut self.controls, &self.fonts, self.modifiers)
    }

    pub fn set_behaviour<T: Behaviour + 'static>(&mut self, id: Id, behaviour: T) {
        self.controls[id].set_behaviour(Box::new(behaviour));
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
            if let Some(parent) = self.controls[id].parent {
                self.call_event_chain(parent, event);
            }
        }
    }

    pub fn start(&mut self) {
        self.update_all_layouts();
        // TODO: maybe on_start and on_active must be called in-order in NewEvents, instead of immediatily after creation
        // let mut parents = vec![ROOT_ID];
        // while let Some(id) = parents.pop() {
        //     self.call_event(id, |this, id, ctx| this.on_start(id, ctx));
        //     // when acessing childs directly, instead of get_children(), the inactive controls is also picked.
        //     parents.extend(self.controls[id].children.iter().rev());
        // }
        // parents.clear();
        // parents.push(ROOT_ID);
        // while let Some(id) = parents.pop() {
        //     self.call_event(id, |this, id, ctx| this.on_active(id, ctx));
        //     parents.extend(self.get_children(id));
        // }
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
                        self.set_focus(self.current_over);
                    }
                    if let Some(curr) = self.current_over {
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
                    if !self.over_is_locked {
                        if let Some(curr) = self.current_over.take() {
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
                if self.controls[*child].rect.contains(mouse_x, mouse_y) {
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
