use crate::{
    context::{Context, LayoutContext, MinSizeContext},
    control::ControlBuilderInner,
    graphics::Graphic,
    util::WithPriority,
    Control, ControlBuilder, ControlState, Controls, LayoutDirtyFlags, Rect,
};
use ab_glyph::FontArc;
use keyed_priority_queue::KeyedPriorityQueue;
use std::{
    any::Any,
    collections::VecDeque,
    num::NonZeroU32,
    sync::atomic::AtomicU64,
    time::{Duration, Instant},
};
use winit::{
    event::{ElementState, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
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
    pub struct CreateControl {
        pub id: Id,
    }
    pub struct SetValue<T>(pub T);

    pub struct ToggleChanged {
        pub id: Id,
        pub value: bool,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Id {
    pub(crate) index: u32,
    pub(crate) generation: NonZeroU32,
}
impl Id {
    pub const ROOT_ID: Id = Id {
        index: 0,
        generation: unsafe { NonZeroU32::new_unchecked(1) },
    };
    /// Get the index of the control in the controls vector inside Gui<R>
    pub fn index(&self) -> usize {
        self.index as usize
    }
    /// Get the generation of the control it is refering
    pub fn generation(&self) -> u32 {
        self.generation.get()
    }
}
impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.generation, self.index)
    }
}

#[allow(clippy::clippy::enum_variant_names)]
#[derive(PartialEq, Eq, Debug)]
enum LazyEvent {
    OnStart(Id),
    OnRemove(Id),
    OnActive(Id),
    OnDeactive(Id),
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
    Other(u16),
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
    Moved,
    None,
}

// #[derive(Clone, Copy, Debug)]
// pub enum MouseAction {
//     Click,
//     DoubleClick,
//     Drag,
//     None,
// }

#[derive(Clone, Debug)]
pub struct MouseInfo {
    pub event: MouseEvent,
    // pub action: MouseAction,
    pub pos: [f32; 2],
    pub delta: Option<[f32; 2]>,
    /// number of consecutives MouseDown's
    pub click_count: u8,
}
impl MouseInfo {
    /// Returns `true` if the mouse_action is a click.
    pub fn click(&self) -> bool {
        self.click_count > 0 && matches!(self.event, MouseEvent::Up(MouseButton::Left))
    }
}
impl Default for MouseInfo {
    fn default() -> Self {
        Self {
            event: MouseEvent::None,
            // action: MouseAction::None,
            pos: [f32::NAN; 2],
            delta: None,
            click_count: 0,
        }
    }
}
#[derive(Copy, Clone)]
pub enum KeyboardEvent {
    Char(char),
    Pressed(VirtualKeyCode),
}

#[derive(Default)]
pub(crate) struct Input {
    pub mouse_pos: Option<[f32; 2]>,
    pub last_mouse_pos: Option<[f32; 2]>,
    /// number of consecutives MouseDown's
    pub click_count: u8,
    /// used to check for double clicks
    pub last_mouse_down: Option<Instant>,
}
impl Input {
    fn get_mouse_info(&self, event: MouseEvent) -> MouseInfo {
        let delta = self
            .mouse_pos
            .zip(self.last_mouse_pos)
            .map(|(a, b)| [a[0] - b[0], a[1] - b[1]]);
        MouseInfo {
            event,
            // action,
            pos: self.mouse_pos.unwrap(),
            delta,
            click_count: self.click_count,
        }
    }
}

type ScheduledEventTo = WithPriority<(Instant, u64), (Id, Box<dyn Any>)>;

pub struct Gui {
    pub(crate) controls: Controls,
    pub(crate) fonts: Vec<FontArc>,
    pub(crate) modifiers: ModifiersState,
    redraw: bool,
    // controls that need to update the layout
    dirty_layouts: Vec<Id>,
    // controls that 'on_start' need be called
    scheduled_events: KeyedPriorityQueue<u64, ScheduledEventTo>,
    lazy_events: VecDeque<LazyEvent>,
    change_cursor: Option<CursorIcon>,
    pub(crate) input: Input,
    current_mouse: Option<Id>,
    current_scroll: Option<Id>,
    pub(crate) current_focus: Option<Id>,
    over_is_locked: bool,
}
impl Gui {
    pub fn new(width: f32, height: f32, fonts: Vec<FontArc>) -> Self {
        Self {
            modifiers: ModifiersState::empty(),
            controls: vec![Control {
                generation: NonZeroU32::new(1).unwrap(),
                rect: Rect {
                    anchors: [0.0; 4],
                    margins: [0.0; 4],
                    min_size: [width, height],
                    rect: [0.0, 0.0, width, height],
                    ..Default::default()
                },
                active: true,
                really_active: true,
                ..Default::default()
            }]
            .into(),
            redraw: true,
            scheduled_events: KeyedPriorityQueue::default(),
            dirty_layouts: Vec::new(),
            lazy_events: VecDeque::new(),
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

    fn get_active_children(&self, id: Id) -> Vec<Id> {
        self.controls.get_active_children(id)
    }

    pub fn reserve_id(&mut self) -> Id {
        self.controls.reserve()
    }

    pub fn create_control(&mut self) -> ControlBuilder {
        let id = self.reserve_id();
        self.create_control_reserved(id)
    }

    /// Create a control with a predetermined id, id that can be obtained by the method reserve_id().
    pub fn create_control_reserved(&mut self, reserved_id: Id) -> ControlBuilder {
        struct Builder<'a>(&'a mut Gui);
        impl ControlBuilderInner for Builder<'_> {
            fn controls(&mut self) -> &mut Controls {
                &mut self.0.controls
            }
            fn build(&mut self, id: Id) {
                self.0.add_control(id);
            }
        }

        ControlBuilder::new(reserved_id, Builder(self))
    }

    fn add_control(&mut self, id: Id) -> Id {
        if let ControlState::Building = self.controls[id].state {
            println!(
                "add control {:<10} {}",
                id.to_string(),
                self.controls[id]
                    .parent
                    .map(|x| format!("child of {}", x))
                    .unwrap_or_default()
            );
            self.dirty_layout(id);
            assert_eq!(self.controls[id].generation, id.generation);
            let has_behaviour = self.controls[id].behaviour.is_some();
            if has_behaviour {
                self.lazy_events.push_back(LazyEvent::OnStart(id));
            }

            if self.controls[id].really_active {
                debug_assert!(self.controls[id].active);
                self.lazy_events.push_back(LazyEvent::OnActive(id));
            }

            self.controls[id].state = ControlState::Started;

            for child in self.controls[id].children.clone() {
                println!("add child {}", child);
                self.add_control(child);
            }
        } else {
            println!("double add {}", id);
        }

        id
    }

    pub fn active_control(&mut self, id: Id) {
        if self.controls[id].active {
            return;
        }
        self.controls[id].active = true;

        if let Some(parent) = self.get_parent(id) {
            self.dirty_layout(parent);
        }

        if self
            .get_parent(id)
            .map(|x| self.controls[x].really_active)
            .unwrap_or(true)
        {
            self.controls[id].really_active = true;
            let mut parents = vec![id];
            while let Some(id) = parents.pop() {
                parents.extend(self.get_active_children(id).iter().rev());
                self.controls[id].really_active = true;
                println!("really_active = true for {}", id);
                // If there was already a deactive event queued, we cancel it
                if let Some(i) = self
                    .lazy_events
                    .iter()
                    .position(|x| *x == LazyEvent::OnDeactive(id))
                {
                    self.lazy_events.remove(i);
                }
                self.lazy_events.push_back(LazyEvent::OnActive(id));
            }
        }
        // TODO: uncommenting the line below allow infinity recursion to happen
        // self.mouse_moved(self.input.mouse_x, self.input.mouse_y);
    }

    pub fn deactive_control(&mut self, id: Id) {
        if !self.controls[id].active {
            return;
        }
        self.controls[id].active = false;

        if let Some(parent) = self.get_parent(id) {
            self.dirty_layout(parent);
        }
        if self
            .get_parent(id)
            .map(|x| self.controls[x].really_active)
            .unwrap_or(true)
        {
            let mut parents = vec![id];
            while let Some(id) = parents.pop() {
                parents.extend(self.get_active_children(id).iter().rev());
                if Some(id) == self.current_mouse {
                    self.update_layout();
                    let mouse = self.input.get_mouse_info(MouseEvent::Exit);
                    if let Some((this, mut ctx)) = Context::new_with_mut_behaviour(id, self) {
                        this.on_mouse_event(mouse, id, &mut ctx);
                    }
                    self.current_mouse = None;
                }
                if Some(id) == self.current_scroll {
                    self.current_scroll = None;
                }
                if Some(id) == self.current_focus {
                    self.set_focus(None);
                }
                self.controls[id].really_active = false;
                println!("really_active = false for {}", id);
                // If there was already a active event queued, we cancel it
                if let Some(i) = self
                    .lazy_events
                    .iter()
                    .position(|x| *x == LazyEvent::OnActive(id))
                {
                    self.lazy_events.remove(i);
                }
                self.lazy_events.push_back(LazyEvent::OnDeactive(id));
            }
        }
        // uncommenting the line below allow infinity recursion to happen
        // self.mouse_moved(self.input.mouse_x, self.input.mouse_y);
    }

    /// Remove a control and all of its children
    pub fn remove_control(&mut self, id: Id) {
        self.lazy_events.push_back(LazyEvent::OnRemove(id));
    }

    /// Remove all control
    pub fn clear_controls(&mut self) {
        self.lazy_update();
        self.lazy_events.push_back(LazyEvent::OnRemove(Id::ROOT_ID));
        self.lazy_update();
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

    /// Handle if there is some scheduled event to be adressed, and
    /// return the instant for the next scheduled event
    pub fn handle_scheduled_event(&mut self) -> Option<Instant> {
        loop {
            let now = Instant::now();
            match self.scheduled_events.peek().map(|x| x.1.priority().0) {
                Some(time) => {
                    if now >= time {
                        let (id, event) = self.scheduled_events.pop().unwrap().1.item;
                        self.send_event_to(id, event);
                        continue;
                    }
                    return self.scheduled_events.peek().map(|x| x.1.priority().0);
                }
                None => return None,
            }
        }
    }

    #[inline]
    pub fn get_context(&mut self) -> Context {
        self.lazy_update();
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
            self.dirty_layout(dirty);
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
        self.controls[Id::ROOT_ID]
            .rect
            .set_rect([0.0, 0.0, width, height]);
        self.dirty_layout(Id::ROOT_ID);
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
        } else if let Some(event::CreateControl { id }) = event.downcast_ref() {
            self.add_control(*id);
        } else if let Some(cursor) = event.downcast_ref::<CursorIcon>() {
            self.change_cursor = Some(*cursor);
        }
    }

    // TODO: there should not be a public function which receive Box<...>
    // (specially when there is identical funtcion that is generic)
    pub fn send_event_to(&mut self, id: Id, event: Box<dyn Any>) {
        self.lazy_update();
        if let Some((this, mut ctx)) = Context::new_with_mut_behaviour(id, self) {
            this.on_event(event, id, &mut ctx);
        }
    }

    // TODO: there should not be a public function which receive Box<...>
    // (specially when there is identical funtcion that is generic)
    pub fn send_event_to_scheduled(
        &mut self,
        id: Id,
        event: Box<dyn Any>,
        instant: Instant,
    ) -> u64 {
        static ORDER_OF_INSERTION: AtomicU64 = AtomicU64::new(0);
        let event_id = ORDER_OF_INSERTION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let event = WithPriority::new((instant, event_id), (id, event));
        self.scheduled_events.push(event_id, event);
        event_id
    }

    pub fn cancel_scheduled_event(&mut self, event_id: u64) {
        self.scheduled_events.remove(&event_id);
    }

    pub fn call_event<F: FnOnce(&mut dyn Behaviour, Id, &mut Context)>(
        &mut self,
        id: Id,
        event: F,
    ) {
        self.lazy_update();
        if let Some((this, mut ctx)) = Context::new_with_mut_behaviour(id, self) {
            event(this, id, &mut ctx);
        }
    }

    pub fn call_event_chain<F: Fn(&mut dyn Behaviour, Id, &mut Context) -> bool>(
        &mut self,
        id: Id,
        event: F,
    ) -> bool {
        self.lazy_update();
        let mut handled = false;
        if let Some((this, mut ctx)) = Context::new_with_mut_behaviour(id, self) {
            handled = event(this, id, &mut ctx);
        }
        if handled {
            return true;
        }
        if let Some(parent) = self.controls[id].parent {
            self.call_event_chain(parent, event)
        } else {
            false
        }
    }

    pub fn start(&mut self) {
        self.update_all_layouts();
        fn print_tree(branchs: String, id: Id, gui: &mut Gui) {
            let childs = gui.controls.get_active_children(id); //.clone();
            let len = childs.len();
            for (i, child) in childs.iter().enumerate() {
                println!(
                    "{}{}━━{}",
                    branchs,
                    if i + 1 == len { "┗" } else { "┣" },
                    child
                );
                if i + 1 == len {
                    print_tree(branchs.clone() + "   ", *child, gui)
                } else {
                    print_tree(branchs.clone() + "┃  ", *child, gui)
                };
            }
        }
        println!("{:?}", Id::ROOT_ID);
        print_tree("".into(), Id::ROOT_ID, self);
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        self.lazy_update();
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_moved(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    self.mouse_down((*button).into());
                }
                ElementState::Released => {
                    self.mouse_up((*button).into());
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(curr) = self.current_scroll {
                    //TODO: I should handle Line and Pixel Delta differences more wisely?
                    let delta = match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => [*x * 100.0, *y * 100.0],
                        winit::event::MouseScrollDelta::PixelDelta(p) => [p.x as f32, p.y as f32],
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
                    let handled = self.call_event_chain(curr, |this, id, ctx| {
                        this.on_keyboard_event(KeyboardEvent::Pressed(*keycode), id, ctx)
                    });
                    if !handled {
                        let shift = self.modifiers.shift();
                        let next = match *keycode {
                            VirtualKeyCode::Tab if !shift => {
                                let mut tree = self.controls.tree_starting_at(curr);
                                tree.pop(); // pop 'this'
                                loop {
                                    let id = match tree.pop() {
                                        Some(id) => id,
                                        None => break None,
                                    };
                                    tree.extend(self.controls.get_active_children(id).iter().rev());
                                    let is_focus =
                                        self.controls[id].behaviour.as_ref().map_or(false, |x| {
                                            x.input_flags().contains(InputFlags::FOCUS)
                                        });
                                    if is_focus {
                                        break Some(id);
                                    }
                                }
                            }
                            VirtualKeyCode::Tab => {
                                let mut tree = self.controls.rev_tree_starting_at(curr);
                                tree.pop(); // pop 'this'
                                loop {
                                    let id = match tree.pop() {
                                        Some(id) => id,
                                        None => break None,
                                    };
                                    tree.extend(self.controls.get_active_children(id));
                                    let is_focus =
                                        self.controls[id].behaviour.as_ref().map_or(false, |x| {
                                            x.input_flags().contains(InputFlags::FOCUS)
                                        });
                                    if is_focus {
                                        break Some(id);
                                    }
                                }
                            }
                            _ => None,
                        };
                        if next.is_some() {
                            self.set_focus(next);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn set_focus(&mut self, id: Option<Id>) {
        println!(
            "set focus to {}",
            id.map(|x| x.to_string())
                .unwrap_or_else(|| "None".to_string())
        );
        if id == self.current_focus {
            return;
        }

        match (self.current_focus, id) {
            (Some(prev), Some(next)) => {
                self.current_focus = Some(next);
                let lca = self.controls.lowest_common_ancestor(prev, next);

                // call on_focus_change(false, ...) only for the controls that lost focus
                let mut curr = Some(prev);
                if curr != lca {
                    while let Some(id) = curr {
                        self.call_event(id, |this, id, ctx| this.on_focus_change(false, id, ctx));
                        self.controls[id].focus = false;
                        curr = self.get_parent(id);
                        if curr == lca {
                            break;
                        }
                    }
                }

                // call on_focus_change(true, ...) for all control with focus
                let mut curr = Some(next);
                while let Some(id) = curr {
                    self.call_event(id, |this, id, ctx| this.on_focus_change(true, id, ctx));
                    self.controls[id].focus = true;
                    curr = self.get_parent(id);
                }
            }
            (Some(prev), None) => {
                self.current_focus = None;
                let mut curr = Some(prev);
                while let Some(id) = curr {
                    self.call_event(id, |this, id, ctx| this.on_focus_change(false, id, ctx));
                    self.controls[id].focus = false;
                    curr = self.get_parent(id);
                }
            }
            (None, Some(next)) => {
                self.current_focus = Some(next);
                let mut curr = self.current_focus;
                while let Some(id) = curr {
                    self.call_event(id, |this, id, ctx| this.on_focus_change(true, id, ctx));
                    self.controls[id].focus = true;
                    curr = self.get_parent(id);
                }
            }
            (None, None) => {}
        }
    }

    pub fn mouse_moved(&mut self, mouse_x: f32, mouse_y: f32) {
        self.input.last_mouse_pos = self.input.mouse_pos;
        self.input.mouse_pos = Some([mouse_x, mouse_y]);
        if self.current_mouse.is_some() && self.over_is_locked {
            self.send_mouse_event_to(self.current_mouse.unwrap(), MouseEvent::Moved);
            return;
        }

        let mut curr = Id::ROOT_ID;
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
            for child in self.get_active_children(curr).iter().rev() {
                if self.controls[*child].rect.contains(mouse_x, mouse_y) {
                    curr = *child;
                    continue 'l;
                }
            }
            break;
        }

        if curr_mouse == self.current_mouse {
            if let Some(current_mouse) = self.current_mouse {
                self.send_mouse_event_to(current_mouse, MouseEvent::Moved);
            }
        } else {
            if let Some(current_mouse) = self.current_mouse {
                self.send_mouse_event_to(current_mouse, MouseEvent::Exit);
            }
            self.current_mouse = curr_mouse;
            if let Some(current_mouse) = self.current_mouse {
                self.input.click_count = 0;
                self.send_mouse_event_to(current_mouse, MouseEvent::Enter);
                self.send_mouse_event_to(current_mouse, MouseEvent::Moved);
            }
        }
    }

    pub fn mouse_down(&mut self, button: MouseButton) {
        self.set_focus(self.current_mouse);
        if let Some(curr) = self.current_mouse {
            if let MouseButton::Left = button {
                const DOUBLE_CLICK_TIME: Duration = Duration::from_millis(500);
                let time = if let Some(last_click) = self.input.last_mouse_down {
                    last_click.elapsed()
                } else {
                    Duration::from_millis(0)
                };
                self.input.last_mouse_down = Some(Instant::now());
                self.input.click_count = if time < DOUBLE_CLICK_TIME {
                    // with saturating the program don't will crash after 256 consective clicks
                    self.input.click_count.saturating_add(1)
                } else {
                    1
                }
            }
            self.send_mouse_event_to(curr, MouseEvent::Down(button));
        }
    }

    pub fn mouse_up(&mut self, button: MouseButton) {
        if let Some(curr) = self.current_mouse {
            self.send_mouse_event_to(curr, MouseEvent::Up(button));
        }
    }

    // TODO: think more carefully in what functions must be public
    pub fn send_mouse_event_to(&mut self, id: Id, event: MouseEvent) {
        let mouse = self.input.get_mouse_info(event);
        self.call_event(id, move |this, id, ctx| this.on_mouse_event(mouse, id, ctx));
    }

    pub fn dirty_layout(&mut self, id: Id) {
        self.dirty_layouts.push(id);
        self.redraw = true;
    }

    fn lazy_update(&mut self) {
        loop {
            while let Some(event) = self.lazy_events.pop_front() {
                match event {
                    LazyEvent::OnStart(id) => {
                        if self.controls.get(id).is_none() {
                            println!("starting {}, but already removed", id);
                            continue;
                        }
                        // TODO: on_start must receive a context that do not exposure the broke layout
                        println!("starting {}", id);
                        if let Some((this, mut ctx)) = Context::new_with_mut_behaviour(id, self) {
                            this.on_start(id, &mut ctx);
                        }
                    }
                    LazyEvent::OnRemove(id) => {
                        if self.controls.get(id).is_none() {
                            println!("removing {}, but already removed", id);
                            continue;
                        }
                        println!("removing {}", id);

                        if self.controls[id].active {
                            // only deactive if it is not the ROOT_ID
                            self.controls[id].active = id == Id::ROOT_ID;
                            if let Some(parent) = self.get_parent(id) {
                                self.dirty_layout(parent);
                            }
                        }

                        if let Some(parent) = self.controls[id].parent {
                            let children = &mut self.controls[parent].children;
                            let pos = children
                                .iter()
                                .position(|x| *x == id)
                                .expect("parent/child desync");
                            children.remove(pos);
                        }

                        // if the id is the ROOT_ID, it should not be removed
                        let mut parents = if id == Id::ROOT_ID {
                            self.controls[id].children.clone()
                        } else {
                            vec![id]
                        };
                        while let Some(id) = parents.pop() {
                            parents.extend(self.controls[id].children.iter().rev());

                            if self.current_mouse == Some(id) {
                                // self.update_layout();
                                // if let Some((this, mut ctx)) =
                                //     Context::new_with_mut_behaviour(id, self)
                                // {
                                //     this.on_mouse_event(MouseEvent::Exit, id, &mut ctx);
                                // }
                                self.current_mouse = None;
                            }
                            if self.current_scroll == Some(id) {
                                self.current_scroll = None;
                            }
                            if self.current_focus == Some(id) {
                                let mut curr = Some(id);
                                while let Some(id) = curr {
                                    // self.call_event(id, |this, id, ctx| this.on_focus_change(false, id, ctx));
                                    self.controls[id].focus = false;
                                    curr = self.get_parent(id);
                                }
                                self.current_focus = None;
                            }

                            println!("remove {}", id);
                            if self.controls[id].really_active {
                                self.update_layout(); // TODO: remotion is quadradic?
                                if let Some((this, mut ctx)) =
                                    Context::new_with_mut_behaviour(id, self)
                                {
                                    this.on_remove(id, &mut ctx);
                                }
                            }
                        }
                        // if the id is the ROOT_ID, it should not be removed
                        if id == Id::ROOT_ID {
                            parents.clone_from(&self.controls[id].children);
                            self.controls[id].children.clear();
                        } else {
                            parents.clear();
                            parents.push(id);
                        };
                        while let Some(id) = parents.pop() {
                            parents.extend(self.controls[id].children.iter().rev());
                            self.controls.remove(id);
                        }
                        // uncommenting the line below allow infinity recursion to happen
                        // self.mouse_moved(self.input.mouse_x, self.input.mouse_y);
                    }
                    LazyEvent::OnActive(id) => {
                        if self.controls.get(id).is_none() {
                            println!("activing {}, but already removed", id);
                            continue;
                        }
                        debug_assert!(self.controls[id].active, "OnDeactive on deactive: {}", id);
                        debug_assert!(
                            self.controls[id].really_active,
                            "OnDeactive on really_deactive: {}",
                            id
                        );
                        self.update_layout();
                        // The update_layout could have deactivated this control
                        if !self.controls[id].really_active {
                            continue;
                        }

                        println!("activing {}", id);
                        if let Some((this, mut ctx)) = Context::new_with_mut_behaviour(id, self) {
                            this.on_active(id, &mut ctx);
                        }

                        let mut tree = self.controls.get_active_children(id);
                        tree.reverse();
                        while let Some(id) = tree.pop() {
                            if !self.controls[id].really_active {
                                println!("active {}", id);
                                tree.extend(self.controls.get_active_children(id).iter().rev());
                                self.controls[id].really_active = true;
                                if let Some((this, mut ctx)) =
                                    Context::new_with_mut_behaviour(id, self)
                                {
                                    this.on_active(id, &mut ctx);
                                }
                            }
                        }
                    }
                    LazyEvent::OnDeactive(id) => {
                        if self.controls.get(id).is_none() {
                            println!("deactiving {}, but already removed", id);
                            continue;
                        }

                        debug_assert!(
                            !self.controls[id].really_active,
                            "OnDeactive on really_deactive: {}",
                            id
                        );
                        self.update_layout();
                        // The update_layout could have deactivated this control
                        if !self.controls[id].really_active {
                            continue;
                        }
                        println!("deactiving {}", id);
                        if let Some((this, mut ctx)) = Context::new_with_mut_behaviour(id, self) {
                            this.on_deactive(id, &mut ctx);
                        }
                    }
                }
            }

            self.update_layout();

            if self.lazy_events.is_empty() {
                break;
            }
            println!("lopping!");
        }
    }

    pub fn update_layout(&mut self) {
        if !self.dirty_layouts.is_empty() {
            println!("updating layout for {}", self.dirty_layouts.len());
            self.dirty_layouts.clear();
            self.update_all_layouts();
        }
    }

    pub fn update_one_layout(&mut self, mut id: Id) {
        // if min_size is dirty and parent has layout, update parent min_size, and recurse it
        // from the highter parent, update layout of its children. For each dirty chldren, update them, recursivily

        {
            let (layout, mut ctx) = MinSizeContext::new(id, &mut self.controls, &self.fonts);
            let mut min_size = layout.compute_min_size(id, &mut ctx);
            let user_min_size = self.controls[id].rect.user_min_size;
            min_size[0] = min_size[0].max(user_min_size[0]);
            min_size[1] = min_size[1].max(user_min_size[1]);
            self.controls[id].rect.min_size = min_size;
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
                    let mut min_size = layout.compute_min_size(id, &mut ctx);
                    let user_min_size = self.controls[id].rect.user_min_size;
                    min_size[0] = min_size[0].max(user_min_size[0]);
                    min_size[1] = min_size[1].max(user_min_size[1]);
                    self.controls[id].rect.min_size = min_size;
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
                let (layout, mut ctx) = LayoutContext::new(id, &mut self.controls, &self.fonts);
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
                    } else if let Some(event::RemoveControl { id }) = event.downcast_ref() {
                        // TODO: this need more thinking
                        let id = *id;
                        self.controls[id].active = false;
                        if let Some(parent) = self.controls[id].parent {
                            let children = &mut self.controls[parent].children;
                            if let Some(pos) = children.iter().position(|x| *x == id) {
                                children.remove(pos);
                            }
                        }

                        let mut parents = vec![id];
                        while let Some(id) = parents.pop() {
                            parents.extend(self.get_active_children(id).iter().rev());

                            // TODO: this comment-out's are probaly buggy
                            if Some(id) == self.current_mouse {
                                // self.send_mouse_event_to(id, MouseEvent::Exit);
                                self.current_mouse = None;
                            }
                            if Some(id) == self.current_scroll {
                                self.current_scroll = None;
                            }
                            if Some(id) == self.current_focus {
                                // self.set_focus(None);
                            }
                            if self.controls[id].really_active {
                                // self.call_event(id, |this, id, ctx| this.on_deactive(id, ctx));
                            }
                        }
                        let mut parents = vec![id];
                        while let Some(id) = parents.pop() {
                            parents.extend(self.get_active_children(id).iter().rev());
                            self.controls.remove(id);
                        }
                    } else if let Some(event::CreateControl { id }) = event.downcast_ref() {
                        self.add_control(*id);
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

            for child in self.get_active_children(id).iter().rev() {
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
        let mut parents = vec![Id::ROOT_ID];

        // post order traversal
        let mut i = 0;
        while i != parents.len() {
            parents.extend(self.get_active_children(parents[i]).iter().rev());
            i += 1;
        }
        while let Some(parent) = parents.pop() {
            let (layout, mut ctx) = MinSizeContext::new(parent, &mut self.controls, &self.fonts);
            let mut min_size = layout.compute_min_size(parent, &mut ctx);
            let user_min_size = self.controls[parent].rect.user_min_size;
            min_size[0] = min_size[0].max(user_min_size[0]);
            min_size[1] = min_size[1].max(user_min_size[1]);
            self.controls[parent].rect.min_size = min_size;
        }

        // parents is empty now

        // inorder traversal
        parents.push(Id::ROOT_ID);
        while let Some(parent) = parents.pop() {
            {
                let (layout, mut ctx) = LayoutContext::new(parent, &mut self.controls, &self.fonts);
                layout.update_layouts(parent, &mut ctx);
                for event in ctx.events {
                    //TODO: think carefully about this deactives
                    if let Some(event::DeactiveControl { id }) = event.downcast_ref() {
                        self.deactive_control(*id)
                    } else if let Some(event::ActiveControl { id }) = event.downcast_ref() {
                        self.active_control(*id)
                    } else if let Some(event::RemoveControl { id }) = event.downcast_ref() {
                        self.remove_control(*id);
                    } else if let Some(event::CreateControl { id }) = event.downcast_ref() {
                        self.add_control(*id);
                    }
                }
            }
            parents.extend(self.get_active_children(parent).iter().rev());
        }
    }
}

bitflags! {
    pub struct InputFlags: u8 {
        const MOUSE = 0x1;
        const SCROLL = 0x2;
        const FOCUS = 0x4;
    }
}

#[allow(unused_variables)]
pub trait Behaviour {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {}
    fn on_active(&mut self, this: Id, ctx: &mut Context) {}
    fn on_deactive(&mut self, this: Id, ctx: &mut Context) {}
    fn on_remove(&mut self, this: Id, ctx: &mut Context) {}

    fn input_flags(&self) -> InputFlags {
        InputFlags::empty()
    }

    fn on_event(&mut self, event: Box<dyn Any>, this: Id, ctx: &mut Context) {}

    fn on_scroll_event(&mut self, delta: [f32; 2], this: Id, ctx: &mut Context) {}

    fn on_mouse_event(&mut self, mouse: MouseInfo, this: Id, ctx: &mut Context) {}

    fn on_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {}

    fn on_keyboard_event(&mut self, event: KeyboardEvent, this: Id, ctx: &mut Context) -> bool {
        false
    }
}
impl Behaviour for () {}

#[allow(unused_variables)]
pub trait Layout {
    /// Compute its own min size, based on the min size of its children.
    fn compute_min_size(&mut self, this: Id, ctx: &mut MinSizeContext) -> [f32; 2] {
        [0.0; 2]
    }
    /// Update the position and size of its children.
    fn update_layouts(&mut self, this: Id, ctx: &mut LayoutContext) {
        let rect = *ctx.get_rect(this);
        let size = [rect[2] - rect[0], rect[3] - rect[1]];
        let pos: [f32; 2] = [rect[0], rect[1]];
        for child in ctx.get_active_children(this) {
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

impl<T: Layout> Layout for std::rc::Rc<std::cell::RefCell<T>> {
    fn compute_min_size(&mut self, this: Id, ctx: &mut MinSizeContext) -> [f32; 2] {
        self.as_ref().borrow_mut().compute_min_size(this, ctx)
    }

    fn update_layouts(&mut self, this: Id, ctx: &mut LayoutContext) {
        self.as_ref().borrow_mut().update_layouts(this, ctx)
    }
}
impl<T: Behaviour> Behaviour for std::rc::Rc<std::cell::RefCell<T>> {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        self.as_ref().borrow_mut().on_start(this, ctx)
    }

    fn on_active(&mut self, this: Id, ctx: &mut Context) {
        self.as_ref().borrow_mut().on_active(this, ctx)
    }

    fn on_deactive(&mut self, this: Id, ctx: &mut Context) {
        self.as_ref().borrow_mut().on_deactive(this, ctx)
    }

    fn on_event(&mut self, event: Box<dyn Any>, this: Id, ctx: &mut Context) {
        self.as_ref().borrow_mut().on_event(event, this, ctx)
    }

    fn input_flags(&self) -> InputFlags {
        self.as_ref().borrow_mut().input_flags()
    }

    fn on_scroll_event(&mut self, delta: [f32; 2], this: Id, ctx: &mut Context) {
        self.as_ref().borrow_mut().on_scroll_event(delta, this, ctx)
    }

    fn on_mouse_event(&mut self, mouse: MouseInfo, this: Id, ctx: &mut Context) {
        self.as_ref().borrow_mut().on_mouse_event(mouse, this, ctx)
    }

    fn on_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {
        self.as_ref().borrow_mut().on_focus_change(focus, this, ctx)
    }

    fn on_keyboard_event(&mut self, event: KeyboardEvent, this: Id, ctx: &mut Context) -> bool {
        self.as_ref()
            .borrow_mut()
            .on_keyboard_event(event, this, ctx)
    }
}
