use crate::{
    context::{Context, LayoutContext, MinSizeContext, RenderContext},
    control::BuilderContext,
    font::Fonts,
    graphics::Graphic,
    util::WithPriority,
    Control, ControlBuilder, ControlEntry, Controls, LayoutDirtyFlags, Rect,
};
use instant::Instant;
use keyed_priority_queue::KeyedPriorityQueue;
use std::{
    any::{Any, TypeId},
    collections::{HashMap, VecDeque},
    num::NonZeroU32,
    sync::atomic::AtomicU64,
    time::Duration,
};
use winit::{
    dpi::LogicalPosition,
    event::{ElementState, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
    window::CursorIcon,
};

pub type MouseId = u64;
/// The default mouse Id for the default mouse.
const MOUSE_ID: MouseId = 0;

pub mod event {
    use super::{Id, MouseId};
    pub struct SetLockOver {
        pub lock: bool,
        pub mouse_id: MouseId,
    }
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
    pub struct StartControl {
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
    /// Get the index of the control in the controls vector inside Gui
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
    /// The id to be removed, and if it should dirty the layout of its parent
    OnRemove(Id, bool),
    OnActive(Id),
    OnDeactive(Id),
}

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

/// The state of a button, if is Pressed or Released.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonState {
    Released = 0,
    Pressed = 1,
}
impl ButtonState {
    /// Return `true` if the button state is Pressed.
    pub fn pressed(self) -> bool {
        self == Self::Pressed
    }
}
impl Default for ButtonState {
    fn default() -> Self {
        Self::Released
    }
}

/// The state of each button of the mouse.
#[derive(Default, Debug, Clone)]
pub struct MouseButtons {
    /// The button state of the left mouse button.
    pub left: ButtonState,
    /// The button state of the right mouse button.
    pub right: ButtonState,
    /// The button state of the middle mouse button (the scroll wheel button).
    pub middle: ButtonState,
}

#[derive(Clone, Debug)]
pub struct MouseInfo {
    /// The unique id of this mouse.
    ///
    /// Giui supports the use of multiple mouses.
    pub id: MouseId,
    pub event: MouseEvent,
    // pub action: MouseAction,
    /// The position of the mouse, in pixels, relative to the top-right corner of the window
    pub pos: [f32; 2],
    /// The state of each button of the mouse.
    pub buttons: MouseButtons,
    /// The different beetween this mouse position, and the position in the last event. The last
    /// position may be outside of this control.
    pub delta: Option<[f32; 2]>,
    /// Number of consecutives mouse click. A mouse click is mouse down followed by a mouse up,
    /// without a mouse exit between the two. Consecutive means that the click occurred within
    /// 500 ms after the previous one (without a mouse exit between).
    pub click_count: u8,
}
impl MouseInfo {
    /// Returns `true` if the event is a MouseEvent::Up(MouseButton::Left) and click_count > 0.
    pub fn click(&self) -> bool {
        self.click_count > 0 && matches!(self.event, MouseEvent::Up(MouseButton::Left))
    }
}
impl Default for MouseInfo {
    fn default() -> Self {
        Self {
            id: MOUSE_ID,
            event: MouseEvent::None,
            // action: MouseAction::None,
            pos: [f32::NAN; 2],
            buttons: MouseButtons::default(),
            delta: None,
            click_count: 0,
        }
    }
}
#[derive(Copy, Clone)]
pub enum KeyboardEvent {
    Char(char),
    Pressed(VirtualKeyCode),
    Release(VirtualKeyCode),
}

/// Store data related to mouse input.
#[derive(Default)]
pub(crate) struct MouseInput {
    /// The unique Id of this MouseInput.
    pub id: MouseId,
    pub position: Option<[f32; 2]>,
    pub last_position: Option<[f32; 2]>,
    pub buttons: MouseButtons,
    /// number of consecutives MouseDown's
    pub click_count: u8,
    /// used to check for double clicks
    pub last_down: Option<Instant>,
    /// Tells if current_mouse control is locked, and will not change when the mouse stop hovering
    /// it. Useful for drag widgets like a slider.
    over_is_locked: bool,
    /// The control currently hovered by the mouse. Has receive a MouseEvent::Enter, and
    /// will receive a MouseEvent::Exit when this value chances.
    current_mouse: Option<Id>,
    /// The control currently receiving on_scroll_event's.
    current_scroll: Option<Id>,
}
impl MouseInput {
    fn get_mouse_info(&self, event: MouseEvent) -> MouseInfo {
        let delta = self
            .position
            .zip(self.last_position)
            .map(|(a, b)| [a[0] - b[0], a[1] - b[1]]);
        MouseInfo {
            id: self.id,
            event,
            // action,
            pos: self.position.unwrap(),
            buttons: self.buttons.clone(),
            delta,
            click_count: self.click_count,
        }
    }
}

type ScheduledEventTo = WithPriority<(Instant, u64), (Id, Box<dyn Any>)>;

pub struct Gui {
    pub(crate) controls: Controls,
    pub(crate) fonts: Fonts,
    pub(crate) modifiers: ModifiersState,
    pub(crate) resources: HashMap<TypeId, Box<dyn Any>>,

    redraw: bool,
    // controls that need to update the layout
    dirty_layouts: Vec<Id>,
    // controls that 'on_start' need be called
    scheduled_events: KeyedPriorityQueue<u64, ScheduledEventTo>,
    lazy_events: VecDeque<LazyEvent>,

    pub(crate) inputs: Vec<MouseInput>,
    /// The control currently receiving on_keyboard_event's.
    pub(crate) current_focus: Option<Id>,

    change_cursor: Option<CursorIcon>,
    scale_factor: f64,
}
impl Gui {
    pub fn new(width: f32, height: f32, scale_factor: f64, fonts: Fonts) -> Self {
        Self {
            modifiers: ModifiersState::empty(),
            controls: Controls::new(width, height),
            resources: HashMap::new(),
            redraw: true,
            scheduled_events: KeyedPriorityQueue::default(),
            dirty_layouts: Vec::new(),
            lazy_events: VecDeque::new(),
            change_cursor: None,
            inputs: vec![MouseInput::default()],
            current_focus: None,
            fonts,
            scale_factor,
        }
    }

    /// Set the value of the type T that is owned by the Gui. Any value set before will be dropped
    /// and replaced.
    pub fn set<T: Any + 'static>(&mut self, value: T) {
        let v: Box<dyn Any + 'static> = Box::new(value);
        let type_id = TypeId::of::<T>();
        self.resources.insert(type_id, v);
    }

    /// Get a reference to the value of type T that is owned by the Gui.
    /// # Panics
    /// Panics if the value was not set before hand
    pub fn get<T: Any + 'static>(&self) -> &T {
        self.get_from_type_id(TypeId::of::<T>())
            .downcast_ref()
            .expect("The type for get<T> must be T")
    }

    /// Get a reference to the value of type T that is owned by the Gui.
    /// # Panics
    /// Panics if the value was not set beforehand
    pub fn get_from_type_id(&self, type_id: TypeId) -> &dyn Any {
        &**self
            .resources
            .get(&type_id)
            .expect("The type need to be added with Gui::set beforehand")
    }

    /// Get a mutable reference to the value of type T that is owned by the Gui.
    /// # Panics
    /// Panics if the value was not set beforehand
    pub fn get_mut<T: Any + 'static>(&mut self) -> &mut T {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .expect("The type need to be added with Gui::set before hand.")
            .downcast_mut()
            .expect("The type for get<T> must be T")
    }

    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }

    pub fn fonts_mut(&mut self) -> &mut Fonts {
        &mut self.fonts
    }

    fn get_parent(&self, id: Id) -> Option<Id> {
        self.controls.get(id).and_then(|x| x.parent)
    }

    fn get_active_children(&self, id: Id) -> Vec<Id> {
        self.controls.get_active_children(id).unwrap()
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
        impl BuilderContext for Gui {
            fn get_from_type_id(&self, type_id: TypeId) -> &dyn Any {
                self.get_from_type_id(type_id)
            }
            fn get_graphic_mut(&mut self, id: Id) -> &mut Graphic {
                self.get_graphic(id).unwrap()
            }
            fn controls(&self) -> &Controls {
                &self.controls
            }

            fn controls_mut(&mut self) -> &mut Controls {
                &mut self.controls
            }

            fn build(&mut self, id: Id, mut control: Control) {
                // I don't know if this is necessary, but I setting focus to false before doing any
                // operation with it, because it is breaking the invariant by being true when not
                // yet focused
                let focus = control.focus;
                control.focus = false;

                self.controls.add_builded_control(id, control);
                self.start_control(id);
                if focus {
                    self.set_focus(Some(id));
                }
            }
        }

        ControlBuilder::new(self, reserved_id)
    }

    fn start_control(&mut self, id: Id) -> Id {
        match std::mem::take(&mut self.controls.controls[id.index()]) {
            ControlEntry::Take => {
                panic!("Added a taken control?");
            }
            ControlEntry::Free { .. } | ControlEntry::Reserved { .. } => {
                panic!("A added control should be in building state")
            }
            ControlEntry::Started { control } => {
                // This happens when the child is builded before its parent, the parent is
                // a reserved id.

                //return the taken control
                self.controls.controls[id.index()] = ControlEntry::Started { control };

                log::trace!("double start {}", id)
            }
            ControlEntry::Builded { mut control } => {
                if self
                    .controls
                    .controls
                    .get(control.parent.unwrap().index())
                    .map_or(false, |x| !matches!(x, ControlEntry::Started { .. }))
                {
                    log::trace!("delayed start of {}, parent don't started yet", id);

                    //return the taken control
                    self.controls.controls[id.index()] = ControlEntry::Builded { control };

                    return id;
                }
                log::trace!(
                    "add control {:<10} {}",
                    id.to_string(),
                    control
                        .parent
                        .map(|x| format!("child of {}", x))
                        .unwrap_or_default()
                );
                self.dirty_layout(id);
                assert_eq!(control.generation, id.generation);
                let has_behaviour = control.behaviour.is_some();
                if has_behaviour {
                    self.lazy_events.push_back(LazyEvent::OnStart(id));
                }

                if control.active
                    && control.parent.map_or(true, |x| {
                        self.controls.get(x).map_or(false, |x| x.really_active)
                    })
                {
                    log::trace!("really active {}", id);
                    control.really_active = true;
                    debug_assert!(control.active);
                    self.lazy_events.push_back(LazyEvent::OnActive(id));
                }

                self.controls.controls[id.index()] = ControlEntry::Started { control };

                let children = self.controls.get(id).unwrap().children.clone();
                for child in children {
                    log::trace!("add child {}", child);
                    self.start_control(child);
                }
            }
        }

        id
    }

    pub fn active_control(&mut self, id: Id) {
        if let Some(mut control) = self.controls.get_mut(id) {
            if control.active {
                return;
            }
            control.active = true;
        } else {
            return;
        }

        let parent = self.controls.get(id).unwrap().parent;
        if let Some(parent) = parent {
            self.dirty_layout(parent);
        }

        if parent
            .map(|x| {
                self.controls
                    .get(x)
                    .expect("Parent/child desync")
                    .really_active
            })
            .unwrap_or(true)
        {
            log::trace!("really active {}", id);
            self.controls.get_mut(id).unwrap().really_active = true;
            let mut parents = vec![id];
            while let Some(id) = parents.pop() {
                parents.extend(self.get_active_children(id).iter().rev());
                log::trace!("really active {}", id);
                self.controls.get_mut(id).unwrap().really_active = true;
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
        if let Some(mut control) = self.controls.get_mut(id) {
            if !control.active {
                return;
            }
            control.active = false;
        } else {
            return;
        }

        let parent = self.controls.get(id).unwrap().parent;
        if let Some(parent) = parent {
            self.dirty_layout(parent);
        }

        if parent
            .map(|x| {
                self.controls
                    .get(x)
                    .expect("Parent/child desync")
                    .really_active
            })
            .unwrap_or(true)
        {
            let mut parents = vec![id];
            while let Some(id) = parents.pop() {
                parents.extend(self.get_active_children(id).iter().rev());

                // the Vec self.inputs is only mutate in self.mouse_enters and self.mouse_exit,
                // which are only called by the root caller self.handle_event, so this for will not
                // be invalidate by inner calls.
                for i in 0..self.inputs.len() {
                    if Some(id) == self.inputs[i].current_scroll {
                        self.inputs[i].current_scroll = None;
                    }
                    if Some(id) == self.inputs[i].current_mouse {
                        self.update_layout();
                        let mouse = self.inputs[i].get_mouse_info(MouseEvent::Exit);
                        self.call_event_no_lazy(id, |x, id, ctx| x.on_mouse_event(mouse, id, ctx));
                        self.inputs[i].current_mouse = None;
                    }
                }

                if Some(id) == self.current_focus {
                    self.set_focus(None);
                }
                log::trace!("really deactive {}", id);
                self.controls.get_mut(id).unwrap().really_active = false;
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
        self.lazy_events.push_back(LazyEvent::OnRemove(id, true));
    }

    /// Remove all control
    pub fn clear_controls(&mut self) {
        self.lazy_update();
        self.lazy_events
            .push_back(LazyEvent::OnRemove(Id::ROOT_ID, false));
        self.lazy_update();
    }

    pub fn render_is_dirty(&self) -> bool {
        if self.redraw {
            log::debug!("render is dirty");
        }
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
        Context::new(self)
    }

    #[inline]
    pub fn get_render_context(&mut self) -> RenderContext {
        self.lazy_update();
        self.redraw = false;
        RenderContext::new(self)
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

    pub fn get_graphic(&mut self, id: Id) -> Option<&mut Graphic> {
        Some(&mut self.controls.get_mut(id)?.graphic)
    }

    pub fn get_rect(&self, id: Id) -> Option<&Rect> {
        Some(&self.controls.get(id)?.rect)
    }

    /// Set the scale factor of the gui.
    ///
    /// This is used to scale the gui when rendering, allowing dpi awareness.
    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
    }

    /// Get the current scale factor of the gui.
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    /// Set the rect of the root control. Must be called when the window resize for example.
    ///
    /// The given rect must be in the format [x1, y1, x2, y2].
    pub fn set_root_rect(&mut self, rect: [f32; 4]) {
        self.controls
            .get_mut(Id::ROOT_ID)
            .unwrap()
            .rect
            .set_rect(rect);
        self.dirty_layout(Id::ROOT_ID);
    }

    pub fn send_event(&mut self, event: Box<dyn Any>) {
        log::trace!("send_event");
        if let Some(event::ActiveControl { id }) = event.downcast_ref() {
            self.active_control(*id);
        } else if let Some(event::DeactiveControl { id }) = event.downcast_ref() {
            self.deactive_control(*id);
        } else if let Some(event::RemoveControl { id }) = event.downcast_ref() {
            self.remove_control(*id);
        } else if let Some(&event::SetLockOver { lock, mouse_id }) = event.downcast_ref() {
            let input = Gui::get_mouse(&mut self.inputs, mouse_id);
            input.map(|x| x.over_is_locked = lock);
        } else if let Some(event::RequestFocus { id }) = event.downcast_ref() {
            self.set_focus(Some(*id));
        } else if let Some(event::StartControl { id }) = event.downcast_ref() {
            self.start_control(*id);
        } else if let Some(cursor) = event.downcast_ref::<CursorIcon>() {
            self.change_cursor = Some(*cursor);
        }
    }

    pub(crate) fn get_mouse(
        inputs: &mut Vec<MouseInput>,
        mouse_id: u64,
    ) -> Option<&mut MouseInput> {
        let mouse_id = mouse_id;
        inputs.iter_mut().find(|x| x.id == mouse_id)
    }

    // TODO: there should not be a public function which receive Box<...>
    // (specially when there are identical functions that are generic)
    pub fn send_event_to(&mut self, id: Id, event: Box<dyn Any>) {
        self.call_event(id, |this, id, ctx| this.on_event(event, id, ctx));
    }

    // TODO: there should not be a public function which receive Box<...>
    // (specially when there is identical function that is generic)
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

    fn call_event<F: FnOnce(&mut dyn Behaviour, Id, &mut Context)>(
        self: &mut Self,
        id: Id,
        event: F,
    ) {
        self.lazy_update();
        if self
            .controls
            .get(id)
            .map_or(false, |x| x.behaviour.is_some())
        {
            let control = self.controls.get_mut(id).unwrap();
            let mut this = control.behaviour.take().unwrap();
            let mut ctx = Context::new(self);
            event(this.as_mut(), id, &mut ctx);
            // The behaviour must be returned before doing the context drop.
            let (mut events, mut events_to, mut dirtys, render_dirty) = ctx.destructs();
            self.controls.get_mut(id).unwrap().behaviour = Some(this);
            self.context_drop(&mut events, &mut events_to, &mut dirtys, render_dirty);
        }
    }

    /// A version of call_event without lazy_update(). This is called from inside lazy_update(),
    /// for example, to avoid infinite recursive calls.
    fn call_event_no_lazy<F: FnOnce(&mut dyn Behaviour, Id, &mut Context)>(
        self: &mut Self,
        id: Id,
        event: F,
    ) {
        if self
            .controls
            .get(id)
            .map_or(false, |x| x.behaviour.is_some())
        {
            let control = self.controls.get_mut(id).unwrap();
            let mut this = control.behaviour.take().unwrap();
            let mut ctx = Context::new(self);
            event(this.as_mut(), id, &mut ctx);
            // The behaviour must be returned before doing the context drop.
            let (mut events, mut events_to, mut dirtys, render_dirty) = ctx.destructs();
            self.controls.get_mut(id).unwrap().behaviour = Some(this);
            self.context_drop(&mut events, &mut events_to, &mut dirtys, render_dirty);
        }
    }

    pub fn call_event_chain<F: Fn(&mut dyn Behaviour, Id, &mut Context) -> bool>(
        &mut self,
        id: Id,
        event: F,
    ) -> bool {
        let mut handled = false;
        self.call_event(id, |this, id, ctx| handled = event(this, id, ctx));
        if handled {
            return true;
        }
        let id = self.controls.get(id).unwrap().parent;
        if let Some(parent) = id {
            self.call_event_chain(parent, event)
        } else {
            false
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        self.lazy_update();
        match event {
            &WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.set_scale_factor(scale_factor);
            }
            &WindowEvent::CursorMoved { position, .. } => {
                let position = LogicalPosition::<f32>::from_physical(position, self.scale_factor);
                self.mouse_moved(MOUSE_ID, position.x, position.y);
            }
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    self.mouse_down(MOUSE_ID, (*button).into());
                }
                ElementState::Released => {
                    self.mouse_up(MOUSE_ID, (*button).into());
                }
            },
            &WindowEvent::Touch(winit::event::Touch {
                phase,
                location,
                id,
                ..
            }) => {
                // id 0 represents the default mouse, so add 1 to the id to avoid conflict.
                // (assumes that the id will never be u64::MAX)
                let id = (id + 1) as MouseId;
                if id == 0 {
                    log::error!("touch id conflicts with the default mouse");
                }

                let location = LogicalPosition::<f32>::from_physical(location, self.scale_factor);
                match phase {
                    winit::event::TouchPhase::Started => {
                        self.mouse_enter(id);
                        self.mouse_moved(id, location.x, location.y);
                        self.mouse_down(id, MouseButton::Left);
                    }
                    winit::event::TouchPhase::Ended => {
                        self.mouse_up(id, MouseButton::Left);
                        self.mouse_exit(id);
                    }
                    winit::event::TouchPhase::Moved => {
                        self.mouse_moved(id, location.x, location.y);
                    }
                    winit::event::TouchPhase::Cancelled => {
                        self.mouse_exit(id);
                    }
                }
            }
            &WindowEvent::MouseWheel { delta, .. } => {
                self.mouse_scroll(MOUSE_ID, delta);
            }
            WindowEvent::CursorEntered { .. } => {
                self.mouse_enter(MOUSE_ID);
            }
            WindowEvent::CursorLeft { .. } => {
                self.mouse_exit(MOUSE_ID);
            }
            WindowEvent::ReceivedCharacter(ch) => {
                log::debug!("received character {:?}", ch);
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
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                log::debug!("received key {:?}", keycode);
                if let Some(curr) = self.current_focus {
                    let event = if *state == ElementState::Pressed {
                        KeyboardEvent::Pressed(*keycode)
                    } else {
                        KeyboardEvent::Release(*keycode)
                    };
                    let handled = self.call_event_chain(curr, |this, id, ctx| {
                        this.on_keyboard_event(event, id, ctx)
                    });
                    // if the key press was not handled, use it for navigation. Tab go to next
                    // control, Shift+Tab go to previous.
                    if !handled && *state == ElementState::Pressed {
                        let shift = self.modifiers.shift();
                        let next = match *keycode {
                            VirtualKeyCode::Tab if !shift => {
                                let mut tree = self.controls.tree_starting_at(curr).unwrap();
                                tree.pop(); // pop 'this'
                                loop {
                                    let id = match tree.pop() {
                                        Some(id) => id,
                                        None => break None,
                                    };
                                    tree.extend(
                                        self.controls.get_active_children(id).unwrap().iter().rev(),
                                    );
                                    let is_focus = self
                                        .controls
                                        .get(id)
                                        .unwrap()
                                        .behaviour
                                        .as_ref()
                                        .map_or(false, |x| {
                                            x.input_flags().contains(InputFlags::FOCUS)
                                        });
                                    if is_focus {
                                        break Some(id);
                                    }
                                }
                            }
                            VirtualKeyCode::Tab => {
                                let mut tree = self.controls.rev_tree_starting_at(curr).unwrap();
                                tree.pop(); // pop 'this'
                                loop {
                                    let id = match tree.pop() {
                                        Some(id) => id,
                                        None => break None,
                                    };
                                    tree.extend(self.controls.get_active_children(id).unwrap());
                                    let is_focus = self
                                        .controls
                                        .get(id)
                                        .unwrap()
                                        .behaviour
                                        .as_ref()
                                        .map_or(false, |x| {
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
        self.lazy_update();
        log::trace!(
            "set focus to {}",
            id.map(|x| x.to_string())
                .unwrap_or_else(|| "None".to_string())
        );

        let id = if id.map_or(false, |id| {
            self.controls.get(id).map_or(true, |x| !x.really_active)
        }) {
            log::trace!(
                "{} is not active yet, focusing None",
                id.map(|x| x.to_string())
                    .unwrap_or_else(|| "None".to_string())
            );
            None
        } else {
            id
        };

        if id == self.current_focus {
            log::trace!(
                "{} is already focus",
                id.map(|x| x.to_string())
                    .unwrap_or_else(|| "None".to_string())
            );
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
                        log::trace!("unfocus {}", id.to_string());
                        self.call_event(id, |this, id, ctx| this.on_focus_change(false, id, ctx));
                        self.controls.get_mut(id).unwrap().focus = false;
                        curr = self.get_parent(id);
                        if curr == lca {
                            break;
                        }
                    }
                }

                // call on_focus_change(true, ...) for all control with focus
                let mut curr = Some(next);
                while let Some(id) = curr {
                    log::trace!("focus {}", id.to_string());
                    self.call_event(id, |this, id, ctx| this.on_focus_change(true, id, ctx));
                    self.controls.get_mut(id).unwrap().focus = true;
                    curr = self.get_parent(id);
                }
            }
            (Some(prev), None) => {
                self.current_focus = None;
                let mut curr = Some(prev);
                while let Some(id) = curr {
                    log::trace!("unfocus {}", id.to_string());
                    self.call_event(id, |this, id, ctx| this.on_focus_change(false, id, ctx));
                    self.controls.get_mut(id).unwrap().focus = false;
                    curr = self.get_parent(id);
                }
            }
            (None, Some(next)) => {
                self.current_focus = Some(next);
                let mut curr = self.current_focus;
                while let Some(id) = curr {
                    log::trace!("focus {}", id.to_string());
                    self.call_event(id, |this, id, ctx| this.on_focus_change(true, id, ctx));
                    self.controls.get_mut(id).unwrap().focus = true;
                    curr = self.get_parent(id);
                }
            }
            (None, None) => {}
        }
    }

    pub fn mouse_moved(&mut self, id: MouseId, mouse_x: f32, mouse_y: f32) {
        let input = match Gui::get_mouse(&mut self.inputs, id) {
            Some(x) => x,
            None => {
                log::error!("moved mouse with unkown id {}.", id);
                return;
            }
        };

        input.last_position = input.position;
        input.position = Some([mouse_x, mouse_y]);
        if input.current_mouse.is_some() && input.over_is_locked {
            let mouse = input.get_mouse_info(MouseEvent::Moved);
            let curr = input.current_mouse.unwrap();
            self.send_mouse_event_to(curr, mouse);
            return;
        }

        let mut curr = Id::ROOT_ID;
        let mut curr_scroll = None;
        let mut curr_mouse = None;
        self.update_layout();
        'l: loop {
            if let Some(flags) = self
                .controls
                .get(curr)
                .unwrap()
                .behaviour
                .as_ref()
                .map(|x| x.input_flags())
            {
                if flags.contains(InputFlags::SCROLL) {
                    curr_scroll = Some(curr);
                }
                if flags.contains(InputFlags::MOUSE) {
                    curr_mouse = Some(curr);
                }
            }
            // the interator is reversed because the last child block the previous ones
            for child in self.get_active_children(curr).iter().rev() {
                if self
                    .controls
                    .get(*child)
                    .unwrap()
                    .rect
                    .contains(mouse_x, mouse_y)
                {
                    curr = *child;
                    continue 'l;
                }
            }
            break;
        }

        let input = Gui::get_mouse(&mut self.inputs, id).unwrap();
        if input.current_scroll != curr_scroll {
            log::trace!(
                "set current_scroll from {:?} to {:?}",
                input.current_mouse,
                curr_mouse
            );
        }
        input.current_scroll = curr_scroll;
        if curr_mouse == input.current_mouse {
            if let Some(current_mouse) = input.current_mouse {
                let mouse = input.get_mouse_info(MouseEvent::Moved);
                self.send_mouse_event_to(current_mouse, mouse);
            }
        } else {
            if let Some(current_mouse) = input.current_mouse {
                let mouse = input.get_mouse_info(MouseEvent::Exit);
                self.send_mouse_event_to(current_mouse, mouse);
            }
            let input = Gui::get_mouse(&mut self.inputs, id).unwrap();
            log::trace!(
                "set current_mouse from {:?} to {:?}",
                input.current_mouse,
                curr_mouse
            );
            input.current_mouse = curr_mouse;
            if let Some(current_mouse) = input.current_mouse {
                input.click_count = 0;
                let mouse_enter = input.get_mouse_info(MouseEvent::Enter);
                let mouse_moved = input.get_mouse_info(MouseEvent::Moved);
                self.send_mouse_event_to(current_mouse, mouse_enter);
                self.send_mouse_event_to(current_mouse, mouse_moved);
            }
        }
    }

    pub fn mouse_down(&mut self, id: MouseId, button: MouseButton) {
        let input = match Gui::get_mouse(&mut self.inputs, id) {
            Some(x) => x,
            None => {
                log::error!("down mouse with unkown id {}.", id);
                return;
            }
        };

        match button {
            MouseButton::Left => input.buttons.left = ButtonState::Pressed,
            MouseButton::Right => input.buttons.right = ButtonState::Pressed,
            MouseButton::Middle => input.buttons.middle = ButtonState::Pressed,
            MouseButton::Other(_) => {}
        }

        log::info!(
            "down on {}",
            input
                .current_mouse
                .map_or("None".to_string(), |x| x.to_string())
        );
        let current_mouse = input.current_mouse;
        self.set_focus(current_mouse);

        let input = Gui::get_mouse(&mut self.inputs, id).unwrap();

        if let Some(curr) = input.current_mouse {
            if let MouseButton::Left = button {
                const DOUBLE_CLICK_TIME: Duration = Duration::from_millis(500);
                let time = if let Some(last_click) = input.last_down {
                    last_click.elapsed()
                } else {
                    Duration::from_millis(0)
                };
                input.last_down = Some(Instant::now());
                input.click_count = if time < DOUBLE_CLICK_TIME {
                    // with saturating the program don't will crash after 256 consective clicks
                    input.click_count.saturating_add(1)
                } else {
                    1
                }
            }
            let mouse = input.get_mouse_info(MouseEvent::Down(button));
            self.send_mouse_event_to(curr, mouse);
        }
    }

    pub fn mouse_up(&mut self, id: MouseId, button: MouseButton) {
        let input = match Gui::get_mouse(&mut self.inputs, id) {
            Some(x) => x,
            None => {
                log::error!("up mouse with unkown id {}.", id);
                return;
            }
        };

        match button {
            MouseButton::Left => input.buttons.left = ButtonState::Released,
            MouseButton::Right => input.buttons.right = ButtonState::Released,
            MouseButton::Middle => input.buttons.middle = ButtonState::Released,
            MouseButton::Other(_) => {}
        }
        if let Some(curr) = input.current_mouse {
            let mouse = input.get_mouse_info(MouseEvent::Up(button));
            self.send_mouse_event_to(curr, mouse);
        }
    }

    fn mouse_scroll(&mut self, id: MouseId, delta: winit::event::MouseScrollDelta) {
        let input = match Gui::get_mouse(&mut self.inputs, id) {
            Some(x) => x,
            None => {
                log::error!("up mouse with unkown id {}.", id);
                return;
            }
        };

        if let Some(curr) = input.current_scroll {
            //TODO: I should handle Line and Pixel Delta differences more wisely?
            let delta = match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    let line_scale = 100.0 / self.scale_factor as f32;
                    [x * line_scale, y * line_scale]
                }
                winit::event::MouseScrollDelta::PixelDelta(p) => {
                    let p = LogicalPosition::<f32>::from_physical(p, self.scale_factor);
                    [p.x, p.y]
                }
            };
            self.call_event(curr, |this, id, ctx| this.on_scroll_event(delta, id, ctx));
        }
    }

    /// Called when the mouse enters the Gui. Could have being outside of the window, for example.
    pub fn mouse_enter(&mut self, id: MouseId) {
        log::trace!("mouse {} enters", id);
        if id == MOUSE_ID {
            // the default mouse aways exists.
            return;
        }
        if Gui::get_mouse(&mut self.inputs, id).is_none() {
            let value = MouseInput {
                id,
                ..MouseInput::default()
            };
            self.inputs.push(value)
        } else {
            log::error!("enter mouse with repeating id {}.", id)
        }
    }

    /// Called when the mouse exit the Gui. Could have exit the window, for example.
    ///
    /// The mouse will emit a MouseEvent::Exit even if locked.
    pub fn mouse_exit(&mut self, id: MouseId) {
        log::trace!("mouse {} exit", id);

        let input = match Gui::get_mouse(&mut self.inputs, id) {
            Some(x) => x,
            None => {
                log::error!("exit mouse with unkown id {}.", id);
                return;
            }
        };

        input.buttons = MouseButtons::default();

        if let Some(curr) = input.current_mouse.take() {
            let mouse = input.get_mouse_info(MouseEvent::Exit);
            self.send_mouse_event_to(curr, mouse);
        }

        if id == MOUSE_ID {
            // don't remove the default mouse, because it is good to keep track of its position
            // even outside of the window.
            return;
        }

        let index = self.inputs.iter().position(|x| x.id == id).unwrap();
        self.inputs.swap_remove(index);
    }

    // TODO: think more carefully in what functions must be public
    pub fn send_mouse_event_to(&mut self, id: Id, mouse: MouseInfo) {
        self.call_event(id, move |this, id, ctx| this.on_mouse_event(mouse, id, ctx));
    }

    pub fn dirty_layout(&mut self, id: Id) {
        log::trace!("dirty layout of {}", id);
        self.dirty_layouts.push(id);
        self.redraw = true;
    }

    fn lazy_update(&mut self) {
        loop {
            while let Some(event) = self.lazy_events.pop_front() {
                match event {
                    LazyEvent::OnStart(id) => {
                        if self.controls.get(id).is_none() {
                            log::info!("starting {}, but already removed", id);
                            continue;
                        }
                        // TODO: on_start must receive a context that do not exposure the broke layout
                        log::trace!("starting {}", id);
                        self.call_event_no_lazy(id, |this, id, ctx| this.on_start(id, ctx));
                    }
                    LazyEvent::OnRemove(id, dirty_parent_layout) => {
                        if self.controls.get(id).is_none() {
                            log::info!("removing {}, but already removed", id);
                            continue;
                        }
                        log::trace!("removing {}", id);

                        if self.controls.get(id).unwrap().active {
                            // only deactive if it is not the ROOT_ID
                            self.controls.get_mut(id).unwrap().active = id == Id::ROOT_ID;
                            if dirty_parent_layout {
                                if let Some(parent) = self.get_parent(id) {
                                    self.dirty_layout(parent);
                                }
                            }
                        }

                        let parent = self.controls.get(id).unwrap().parent;
                        if let Some(parent) = parent {
                            let children = &mut self.controls.get_mut(parent).unwrap().children;
                            let pos = children
                                .iter()
                                .position(|x| *x == id)
                                .expect("parent/child desync");
                            children.remove(pos);
                        }

                        // if the id is the ROOT_ID, it should not be removed
                        let mut parents = if id == Id::ROOT_ID {
                            self.controls.get(id).unwrap().children.clone()
                        } else {
                            vec![id]
                        };
                        while let Some(id) = parents.pop() {
                            parents.extend(self.controls.get(id).unwrap().children.iter().rev());

                            for (_, input) in self.inputs.iter_mut().enumerate() {
                                if input.current_mouse == Some(id) {
                                    // TODO: should call mouse exit here? (but would recurse)
                                    input.current_mouse = None;
                                }
                                if input.current_scroll == Some(id) {
                                    input.current_scroll = None;
                                }
                            }
                            if self.current_focus == Some(id) {
                                let mut curr = Some(id);
                                while let Some(id) = curr {
                                    // self.call_event(id, |this, id, ctx| this.on_focus_change(false, id, ctx));
                                    self.controls.get_mut(id).unwrap().focus = false;
                                    curr = self.get_parent(id);
                                }
                                log::trace!("set focus to None, on remove");
                                self.current_focus = None;
                            }

                            log::trace!("remove {}", id);
                            if self.controls.get(id).unwrap().really_active {
                                self.update_layout(); // TODO: remotion is quadradic?
                                self.call_event_no_lazy(id, |this, id, ctx| {
                                    this.on_remove(id, ctx)
                                });
                            }
                        }
                        // if the id is the ROOT_ID, it should not be removed
                        if id == Id::ROOT_ID {
                            parents.clone_from(&self.controls.get(id).unwrap().children);
                            self.controls.get_mut(id).unwrap().children.clear();
                        } else {
                            parents.clear();
                            parents.push(id);
                        };
                        while let Some(id) = parents.pop() {
                            parents.extend(self.controls.get(id).unwrap().children.iter().rev());
                            self.controls.remove(id);
                        }
                        // uncommenting the line below allow infinity recursion to happen
                        // self.mouse_moved(self.input.mouse_x, self.input.mouse_y);
                    }
                    LazyEvent::OnActive(id) => {
                        if self.controls.get(id).is_none() {
                            log::info!("activing {}, but already removed", id);
                            continue;
                        }
                        debug_assert!(
                            self.controls.get(id).unwrap().active,
                            "OnDeactive on deactive: {}",
                            id
                        );
                        debug_assert!(
                            self.controls.get(id).unwrap().really_active,
                            "OnDeactive on really_deactive: {}",
                            id
                        );
                        self.update_layout();
                        // The update_layout could have deactivated this control
                        if !self.controls.get(id).unwrap().really_active {
                            continue;
                        }

                        log::trace!("activing {}", id);
                        self.call_event_no_lazy(id, |this, id, ctx| this.on_active(id, ctx));

                        let mut tree = self.controls.get_active_children(id).unwrap();
                        tree.reverse();
                        while let Some(id) = tree.pop() {
                            if !self.controls.get(id).unwrap().really_active {
                                log::trace!("active {}", id);
                                tree.extend(
                                    self.controls.get_active_children(id).unwrap().iter().rev(),
                                );
                                log::trace!("really active {}", id);
                                self.controls.get_mut(id).unwrap().really_active = true;
                                self.call_event_no_lazy(id, |this, id, ctx| {
                                    this.on_active(id, ctx)
                                });
                            }
                        }
                    }
                    LazyEvent::OnDeactive(id) => {
                        if self.controls.get(id).is_none() {
                            log::info!("deactiving {}, but already removed", id);
                            continue;
                        }
                        debug_assert!(
                            !self.controls.get(id).unwrap().really_active,
                            "OnDeactive on really_deactive: {}",
                            id
                        );
                        self.update_layout();
                        // The update_layout could have deactivated this control
                        if !self.controls.get(id).unwrap().really_active {
                            continue;
                        }
                        log::trace!("deactiving {}", id);
                        self.call_event_no_lazy(id, |this, id, ctx| this.on_deactive(id, ctx));
                    }
                }
            }

            self.update_layout();

            if self.lazy_events.is_empty() {
                break;
            }
            log::trace!("lazy update is looping");
        }
    }

    pub fn update_layout(&mut self) {
        if !self.dirty_layouts.is_empty() {
            log::trace!("updating layout for {}", self.dirty_layouts.len());
            self.dirty_layouts.clear();
            self.update_all_layouts();
        }
    }

    pub fn update_one_layout(&mut self, mut id: Id) {
        // if min_size is dirty and parent has layout, update parent min_size, and recurse it
        // from the highter parent, update layout of its children. For each dirty chldren, update them, recursivily

        {
            let mut layout = self.controls.get_mut(id).unwrap().layout.take().unwrap();
            let mut ctx = MinSizeContext::new(id, &mut self.controls, &self.fonts);
            let mut min_size = layout.compute_min_size(id, &mut ctx);
            self.controls.get_mut(id).unwrap().layout = Some(layout);
            let user_min_size = self.controls.get(id).unwrap().rect.user_min_size;
            min_size[0] = min_size[0].max(user_min_size[0]);
            min_size[1] = min_size[1].max(user_min_size[1]);
            self.controls.get_mut(id).unwrap().rect.min_size = min_size;
        }
        while let Some(parent) = self.get_parent(id) {
            let flags = &mut self.controls.get_mut(id).unwrap().rect.layout_dirty_flags;
            flags.insert(LayoutDirtyFlags::DIRTY);
            if flags.intersects(LayoutDirtyFlags::MIN_WIDTH | LayoutDirtyFlags::MIN_HEIGHT) {
                {
                    let mut layout = self
                        .controls
                        .get_mut(parent)
                        .unwrap()
                        .layout
                        .take()
                        .unwrap();
                    let mut ctx = MinSizeContext::new(parent, &mut self.controls, &self.fonts);
                    let mut min_size = layout.compute_min_size(parent, &mut ctx);
                    self.controls.get_mut(parent).unwrap().layout = Some(layout);
                    let user_min_size = self.controls.get(parent).unwrap().rect.user_min_size;
                    min_size[0] = min_size[0].max(user_min_size[0]);
                    min_size[1] = min_size[1].max(user_min_size[1]);
                    self.controls.get_mut(parent).unwrap().rect.min_size = min_size;
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
                let (events, dirtys) = {
                    let mut layout = self.controls.get_mut(id).unwrap().layout.take().unwrap();
                    let mut ctx = LayoutContext::new(
                        id,
                        &mut self.controls,
                        &mut self.resources,
                        &self.fonts,
                    );
                    layout.update_layouts(id, &mut ctx);
                    let LayoutContext { events, dirtys, .. } = ctx;
                    self.controls.get_mut(id).unwrap().layout = Some(layout);
                    (events, dirtys)
                };
                for event in events {
                    //TODO: think carefully about this deactives
                    if let Some(event::DeactiveControl { id }) = event.downcast_ref() {
                        // self.deactive_control(*id)
                        self.controls.get_mut(*id).unwrap().active = false;
                    } else if let Some(event::ActiveControl { id }) = event.downcast_ref() {
                        // self.active_control(*id)
                        self.controls.get_mut(*id).unwrap().active = true;
                    } else if let Some(event::RemoveControl { id }) = event.downcast_ref() {
                        // TODO: this need more thinking
                        let id = *id;
                        self.controls.get_mut(id).unwrap().active = false;
                        if let Some(parent) = self.controls.get(id).unwrap().parent {
                            let children = &mut self.controls.get_mut(parent).unwrap().children;
                            if let Some(pos) = children.iter().position(|x| *x == id) {
                                children.remove(pos);
                            }
                        }

                        let mut parents = vec![id];
                        while let Some(id) = parents.pop() {
                            parents.extend(self.get_active_children(id).iter().rev());

                            for (_, input) in self.inputs.iter_mut().enumerate() {
                                // TODO: this comment-out's are probaly buggy
                                if Some(id) == input.current_mouse {
                                    // self.send_mouse_event_to(id, MouseEvent::Exit);
                                    input.current_mouse = None;
                                }
                                if Some(id) == input.current_scroll {
                                    input.current_scroll = None;
                                }
                            }
                            if Some(id) == self.current_focus {
                                // self.set_focus(None);
                            }
                            if self.controls.get(id).unwrap().really_active {
                                // self.call_event(id, |this, id, ctx| this.on_deactive(id, ctx));
                            }
                        }
                        let mut parents = vec![id];
                        while let Some(id) = parents.pop() {
                            parents.extend(self.get_active_children(id).iter().rev());
                            self.controls.remove(id);
                        }
                    } else if let Some(event::StartControl { id }) = event.downcast_ref() {
                        self.start_control(*id);
                    }
                }
                for dirty in dirtys {
                    // if dirty == id {
                    //     panic!("Layout cannot modify its own control");
                    // } else {
                    //     self.update_layout(dirty);
                    // }
                    // TODO: rethink this!!!!
                    if let Some(dirty_parent) = self.get_parent(dirty) {
                        assert!(dirty_parent != id, "A layout cannot dirty its own child!");
                        parents.push(dirty_parent);
                    }
                }
            }

            for child in self.get_active_children(id).iter().rev() {
                if !self
                    .controls
                    .get_mut(*child)
                    .unwrap()
                    .rect
                    .get_layout_dirty_flags()
                    .is_empty()
                {
                    parents.push(*child);
                    self.controls
                        .get_mut(*child)
                        .unwrap()
                        .rect
                        .clear_layout_dirty_flags();
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
            let mut layout = self
                .controls
                .get_mut(parent)
                .unwrap()
                .layout
                .take()
                .unwrap();
            let mut ctx = MinSizeContext::new(parent, &mut self.controls, &self.fonts);
            let mut min_size = layout.compute_min_size(parent, &mut ctx);
            self.controls.get_mut(parent).unwrap().layout = Some(layout);
            let user_min_size = self.controls.get(parent).unwrap().rect.user_min_size;
            min_size[0] = min_size[0].max(user_min_size[0]);
            min_size[1] = min_size[1].max(user_min_size[1]);
            self.controls.get_mut(parent).unwrap().rect.min_size = min_size;
        }

        // parents is empty now

        // inorder traversal
        parents.push(Id::ROOT_ID);
        while let Some(parent) = parents.pop() {
            {
                let (events, _dirtys) = {
                    let mut layout = self
                        .controls
                        .get_mut(parent)
                        .unwrap()
                        .layout
                        .take()
                        .unwrap();
                    let mut ctx = LayoutContext::new(
                        parent,
                        &mut self.controls,
                        &mut self.resources,
                        &self.fonts,
                    );
                    layout.update_layouts(parent, &mut ctx);
                    let LayoutContext { events, dirtys, .. } = ctx;
                    self.controls.get_mut(parent).unwrap().layout = Some(layout);
                    (events, dirtys)
                };
                for event in events {
                    if let Some(event::DeactiveControl { id }) = event.downcast_ref() {
                        self.deactive_control(*id)
                    } else if let Some(event::ActiveControl { id }) = event.downcast_ref() {
                        self.active_control(*id)
                    } else if let Some(event::RemoveControl { id }) = event.downcast_ref() {
                        // self.remove_control(*id);
                        self.lazy_events.push_back(LazyEvent::OnRemove(*id, false));
                    } else if let Some(event::StartControl { id }) = event.downcast_ref() {
                        self.start_control(*id);
                    }
                }
            }
            parents.extend(self.get_active_children(parent).iter().rev());
        }
        // self.start_control(id) calls dirty the id, but because all layouts are updated, this
        // dirties layouts can be clear
        self.dirty_layouts.clear();
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
        // default implementation use anchor and margins for layouting
        let rect = ctx.get_rect(this);
        let size = [rect[2] - rect[0], rect[3] - rect[1]];
        let pos: [f32; 2] = [rect[0], rect[1]];
        for child in ctx.get_active_children(this) {
            let rect = &mut ctx.get_layouting(child);
            let mut new_rect = [0.0; 4];
            for i in 0..4 {
                new_rect[i] = pos[i % 2] + size[i % 2] * rect.anchors[i] + rect.margins[i];
            }
            ctx.set_designed_rect(child, new_rect);
        }
    }
}
impl Layout for () {}

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
