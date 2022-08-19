use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use winit::{event::ModifiersState, window::CursorIcon};

use crate::{
    control::BuilderContext, event, font::Fonts, graphics::Graphic, next_animation_id,
    time::Instant, Animation, AnimationId, Control, ControlBuilder, Controls, Gui, Id, Rect,
};

pub enum Event {
    Event(Box<dyn Any>),
    EventTo(Id, Box<dyn Any>),
    Dirty(Id),
    AddAnimation {
        id: AnimationId,
        length: f32,
        animation: Box<dyn Animation>,
    },
    RemoveAnimation {
        id: AnimationId,
    },
}

// contains a reference to all the controls, except the behaviour of one control
pub struct Context<'a> {
    gui: &'a mut Gui,
    // modifiers: ModifiersState,
    // controls: &'a mut Controls,
    fonts: &'a Fonts,
    pub(crate) events: Vec<Event>,
    pub(crate) render_dirty: bool,
}
impl BuilderContext for Context<'_> {
    fn get_from_type_id(&self, type_id: TypeId) -> &dyn Any {
        self.gui.get_from_type_id(type_id)
    }

    fn scale_factor(&self) -> f64 {
        self.gui.scale_factor()
    }

    fn get_graphic_mut(&mut self, id: Id) -> &mut Graphic {
        self.get_graphic_mut(id)
    }

    fn controls(&self) -> &Controls {
        &self.gui.controls
    }

    fn controls_mut(&mut self) -> &mut Controls {
        &mut self.gui.controls
    }

    fn build(&mut self, id: Id, mut control: Control) {
        let focus = control.focus;
        control.focus = false;

        self.gui.controls.add_builded_control(id, control);
        self.send_event(event::StartControl { id });
        if focus {
            self.send_event(event::RequestFocus { id });
        }
    }
}
impl<'a> Drop for Context<'a> {
    fn drop(&mut self) {
        let Context {
            events,
            render_dirty,
            ..
        } = self;
        self.gui.context_drop(events, *render_dirty);
    }
}
impl<'a> Context<'a> {
    pub(crate) fn new(gui: &'a mut Gui) -> Self {
        let fonts = unsafe { std::mem::transmute(&gui.fonts) };
        Self {
            gui,
            fonts,
            events: Vec::new(),
            render_dirty: false,
        }
    }

    /// Destructs the Context in its fields, without Dropping. Drop would automatically call
    /// Gui::context_drop, but I may need to call it manually.
    ///
    /// Returns `(events, , render_dirty)`.
    pub(crate) fn destructs(mut self) -> (Vec<Event>, bool) {
        use std::mem;
        let events = mem::take(&mut self.events);
        let render_dirty = self.render_dirty;

        // this will forget a Vec, but it doesn't have a allocation.
        mem::forget(self);

        (events, render_dirty)
    }

    /// Set the value of the type T that is owned by the Gui. Any value set before will be dropped
    /// and replaced.
    pub fn set<T: Any + 'static>(&mut self, value: T) {
        self.gui
            .resources
            .insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Get a reference to the value of type T that is owned by the Gui. If the value was not set
    /// by Gui::set, this returns None.
    pub fn get<T: Any + 'static>(&self) -> &T {
        self.gui.get()
    }

    /// Get a mutable reference to the value of type T that is owned by the Gui. If the value was
    /// not set by Gui::set, this returns None.
    pub fn get_mut<T: Any + 'static>(&mut self) -> &mut T {
        self.gui.get_mut()
    }

    pub fn create_control(&mut self) -> ControlBuilder {
        let id = self.gui.controls.reserve();
        ControlBuilder::new(self, id)
    }

    pub fn modifiers(&self) -> ModifiersState {
        self.gui.modifiers
    }

    pub fn get_fonts(&self) -> &'a Fonts {
        self.fonts
    }

    pub fn send_event<T: 'static>(&mut self, event: T) {
        self.events.push(Event::Event(Box::new(event)));
    }
    pub fn send_event_to<T: 'static>(&mut self, id: Id, event: T) {
        self.events.push(Event::EventTo(id, Box::new(event)));
    }

    pub fn send_event_to_scheduled<T: 'static>(
        &mut self,
        id: Id,
        event: T,
        instant: Instant,
    ) -> u64 {
        self.gui
            .send_event_to_scheduled(id, Box::new(event), instant)
    }

    pub fn cancel_scheduled_event(&mut self, event_id: u64) {
        self.gui.cancel_scheduled_event(event_id);
    }

    /// Add a new animation.
    ///
    /// The returned `AnimationId` can be used to remove the added animation with
    /// [`Context::remove_animation`]. This id is unique.
    pub fn add_animation<A: 'static + Animation>(
        &mut self,
        length: f32,
        animation: A,
    ) -> AnimationId {
        let id = next_animation_id();
        self.events.push(Event::AddAnimation {
            id,
            length,
            animation: Box::new(animation),
        });
        id
    }

    /// Remove the animation with the given `id`.
    ///
    /// The id is the one returned by [`Context::add_animation`] when the animation to be removed
    /// was added. If the animation doesn't exist (already finished or id is invalid), this will do
    /// nothing.
    pub fn remove_animation(&mut self, id: AnimationId) {
        self.events.push(Event::RemoveAnimation { id });
    }

    pub fn set_cursor(&mut self, cursor: CursorIcon) {
        self.send_event(cursor);
    }

    /// If lock is true, locks the cursor over the current control that is receiving mouse events.
    /// This means that even if the mouse position go out of the area of the control, the control
    /// will continue receiving mouse events, and MouseExit will not be emitted. This is useful
    /// for dragging behavior.
    pub fn lock_cursor(&mut self, lock: bool, mouse_id: crate::MouseId) {
        if lock {
            self.send_event(event::SetLockOver {
                lock: true,
                mouse_id,
            });
        } else {
            self.send_event(event::SetLockOver {
                lock: false,
                mouse_id,
            });
        }
    }

    pub fn get_layouting(&mut self, id: Id) -> &mut Rect {
        &mut self.gui.controls.get_mut(id).unwrap().rect
    }

    pub fn dirty_layout(&mut self, id: Id) {
        self.events.push(Event::Dirty(id));
    }

    pub fn get_rect(&self, id: Id) -> [f32; 4] {
        self.gui.controls.get(id).unwrap().rect.rect
    }

    pub fn get_size(&mut self, id: Id) -> [f32; 2] {
        self.gui.controls.get(id).unwrap().rect.get_size()
    }

    pub fn get_margins(&self, id: Id) -> [f32; 4] {
        self.gui.controls.get(id).unwrap().rect.margins
    }

    pub fn set_margins(&mut self, id: Id, margins: [f32; 4]) {
        self.gui.controls.get_mut(id).unwrap().rect.margins = margins;
        self.dirty_layout(id);
    }

    pub fn get_anchors(&self, id: Id) -> [f32; 4] {
        self.gui.controls.get(id).unwrap().rect.anchors
    }

    pub fn set_margin_left(&mut self, id: Id, margin: f32) {
        self.gui.controls.get_mut(id).unwrap().rect.margins[0] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_top(&mut self, id: Id, margin: f32) {
        self.gui.controls.get_mut(id).unwrap().rect.margins[1] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_right(&mut self, id: Id, margin: f32) {
        self.gui.controls.get_mut(id).unwrap().rect.margins[2] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_bottom(&mut self, id: Id, margin: f32) {
        self.gui.controls.get_mut(id).unwrap().rect.margins[3] = margin;
        self.dirty_layout(id);
    }

    pub fn set_anchors(&mut self, id: Id, anchors: [f32; 4]) {
        self.gui.controls.get_mut(id).unwrap().rect.anchors = anchors;
        self.dirty_layout(id);
    }

    pub fn set_anchor_left(&mut self, id: Id, anchor: f32) {
        self.gui.controls.get_mut(id).unwrap().rect.anchors[0] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_top(&mut self, id: Id, anchor: f32) {
        self.gui.controls.get_mut(id).unwrap().rect.anchors[1] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_right(&mut self, id: Id, anchor: f32) {
        self.gui.controls.get_mut(id).unwrap().rect.anchors[2] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_bottom(&mut self, id: Id, anchor: f32) {
        self.gui.controls.get_mut(id).unwrap().rect.anchors[3] = anchor;
        self.dirty_layout(id);
    }

    pub fn get_min_size(&self, id: Id) -> [f32; 2] {
        self.gui.controls.get(id).unwrap().rect.get_min_size()
    }

    pub fn set_min_size(&mut self, id: Id, min_size: [f32; 2]) {
        self.gui
            .controls
            .get_mut(id)
            .unwrap()
            .rect
            .set_min_size(min_size);
        self.dirty_layout(id);
    }

    pub fn get_graphic_mut(&mut self, id: Id) -> &mut Graphic {
        self.render_dirty = true;
        let control = self.gui.controls.get_mut(id).unwrap();
        control.rect.dirty_render_dirty_flags();
        &mut control.graphic
    }

    pub fn set_graphic(&mut self, id: Id, graphic: Graphic) {
        let control = self.gui.controls.get_mut(id).unwrap();
        control.graphic = graphic;
        control.rect.dirty_render_dirty_flags();
        self.render_dirty = true;
    }

    pub fn get_rect_and_graphic(&mut self, id: Id) -> (&mut Rect, &mut Graphic) {
        let control = self.gui.controls.get_mut(id).unwrap();
        self.render_dirty = true;
        control.rect.dirty_render_dirty_flags();
        (&mut control.rect, &mut control.graphic)
    }

    /// Return if the control with the given Id is active.
    ///
    /// If the control was removed,  or the Id is invalid, returns false.
    pub fn is_active(&self, id: Id) -> bool {
        self.gui.controls.get(id).map_or(false, |x| x.active)
    }

    pub fn set_focus(&mut self, id: Id) {
        self.send_event(event::RequestFocus { id });
    }

    pub fn get_focus(&mut self) -> Option<Id> {
        self.gui.current_focus
    }

    pub fn is_focus(&self, id: Id) -> bool {
        self.gui.controls.get(id).unwrap().focus
    }

    /// Set MouseInfo::click_count to 1, wich keep track of consecutives clicks.
    /// This means that, if called, if the next click is consecutive,
    /// it will have a click count of 2.
    pub fn reset_click_count_to_one(&mut self, mouse_id: crate::MouseId) {
        let input = match self.gui.inputs.get_mouse(mouse_id) {
            Some(x) => x,
            None => {
                log::error!(
                    "reset clock count to one mouse with unkown id {}.",
                    mouse_id
                );
                return;
            }
        };
        input.click_count = 1;
    }

    /// This only took effect when Controls is dropped
    pub fn active(&mut self, id: Id) {
        self.send_event(event::ActiveControl { id });
    }

    /// This only took effect when Controls is dropped
    pub fn deactive(&mut self, id: Id) {
        self.send_event(event::DeactiveControl { id });
    }

    /// Destroy the control, drop it, invalidating all of its referencing Id's
    /// This only took effect when Controls is dropped
    pub fn remove(&mut self, id: Id) {
        self.send_event(event::RemoveControl { id });
    }

    pub fn move_to_front(&mut self, id: Id) {
        self.gui.controls.move_to_front(id);
        self.dirty_layout(id);
    }

    pub fn get_parent(&self, id: Id) -> Option<Id> {
        self.gui.controls.get(id).unwrap().parent
    }

    /// Get both active and deactive children
    pub fn get_all_children(&self, id: Id) -> &[Id] {
        self.gui.controls.get_all_children(id).unwrap()
    }

    pub fn get_active_children(&self, id: Id) -> Vec<Id> {
        self.gui.controls.get_active_children(id).unwrap()
    }
}

pub struct MinSizeContext<'a> {
    this: Id,
    controls: &'a mut Controls,
    fonts: &'a Fonts,
}
impl<'a> MinSizeContext<'a> {
    pub(crate) fn new(this: Id, controls: &'a mut Controls, fonts: &'a Fonts) -> Self {
        Self {
            this,
            controls,
            fonts,
        }
    }

    pub fn get_fonts(&mut self) -> &'a Fonts {
        self.fonts
    }

    pub fn get_layouting(&self, id: Id) -> Option<&Rect> {
        Some(&self.controls.get(id)?.rect)
    }

    pub fn get_margins(&self, id: Id) -> [f32; 4] {
        self.controls.get(id).unwrap().rect.margins
    }

    pub fn get_anchors(&self, id: Id) -> [f32; 4] {
        self.controls.get(id).unwrap().rect.anchors
    }

    pub fn get_min_size(&self, id: Id) -> [f32; 2] {
        self.controls.get(id).unwrap().rect.get_min_size()
    }

    pub fn set_this_min_size(&mut self, min_size: [f32; 2]) {
        self.controls
            .get_mut(self.this)
            .unwrap()
            .rect
            .set_min_size(min_size);
    }

    pub fn get_graphic(&mut self, id: Id) -> Option<&mut Graphic> {
        Some(&mut self.controls.get_mut(id)?.graphic)
    }

    // pub fn get_rect_and_graphic(&mut self, id: Id) -> Option<(&mut Rect, &mut Graphic)> {
    //     let control = &mut self.controls.get(id).unwrap();
    //     if let Graphic::None = control.graphic {
    //         None
    //     } else {
    //         Some((&mut control.rect, &mut control.graphic))
    //     }
    // }

    pub fn is_active(&self, id: Id) -> bool {
        self.controls.get(id).unwrap().active
    }

    // pub fn get_parent(&self, id: Id) -> Option<Id> {
    //     self.controls.get(id).unwrap().parent
    // }

    pub fn get_all_children(&self, id: Id) -> &[Id] {
        self.controls.get_all_children(id).unwrap()
    }

    pub fn get_active_children(&self, id: Id) -> Vec<Id> {
        self.controls.get_active_children(id).unwrap()
    }
}

pub struct LayoutContext<'a> {
    this: Id,
    controls: &'a mut Controls,
    resources: &'a mut HashMap<TypeId, Box<dyn Any>>,
    fonts: &'a Fonts,
    scale_factor: f64,
    pub(crate) dirtys: Vec<Id>,
    pub(crate) events: Vec<Box<dyn Any>>,
}
impl BuilderContext for LayoutContext<'_> {
    fn get_from_type_id(&self, type_id: TypeId) -> &dyn Any {
        let value = self
            .resources
            .get(&type_id)
            .expect("The type need to be added with Gui::set before hand.");
        &**value
    }

    fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    fn get_graphic_mut(&mut self, id: Id) -> &mut Graphic {
        let control = self.controls.get_mut(id).unwrap();
        control.rect.dirty_render_dirty_flags();
        &mut self.controls.get_mut(id).unwrap().graphic
    }

    fn controls(&self) -> &Controls {
        self.controls
    }

    fn controls_mut(&mut self) -> &mut Controls {
        self.controls
    }

    fn build(&mut self, id: Id, mut control: Control) {
        let focus = control.focus;
        control.focus = false;

        self.controls.add_builded_control(id, control);
        self.events.push(Box::new(event::StartControl { id }));
        if focus {
            self.events.push(Box::new(event::RequestFocus { id }));
        }

        // when a control is created during layout, the min_size need to be immediately
        // computed
        if self
            .controls
            .get(id)
            .and_then(|x| x.parent)
            .map_or(false, |x| {
                self.controls.get(x).map_or(false, |x| x.really_active)
            })
        {
            self.recompute_min_size(id);
        }
    }
}
impl<'a> LayoutContext<'a> {
    pub(crate) fn new(
        this: Id,
        controls: &'a mut Controls,
        resources: &'a mut HashMap<TypeId, Box<dyn Any>>,
        fonts: &'a Fonts,
        scale_factor: f64,
    ) -> Self {
        Self {
            this,
            controls,
            resources,
            fonts,
            scale_factor,
            dirtys: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Set the value of the type T that is owned by the Gui. Any value set before will be dropped
    /// and replaced.
    pub fn set<T: Any + 'static>(&mut self, value: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Get a reference to the value of type T that is owned by the Gui.
    /// # Panics
    /// Panics if the value was not set before hand
    pub fn get<T: Any + 'static>(&self) -> &T {
        self.resources
            .get(&TypeId::of::<T>())
            .expect("The type need to be added with Gui::set before hand.")
            .downcast_ref()
            .expect("The type for get<T> must be T")
    }

    /// Get a mutable reference to the value of type T that is owned by the Gui.
    /// # Panics
    /// Panics if the value was not set before hand
    pub fn get_mut<T: Any + 'static>(&mut self) -> &mut T {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .expect("The type need to be added with Gui::set before hand.")
            .downcast_mut()
            .expect("The type for get<T> must be T")
    }

    pub fn create_control(&mut self) -> ControlBuilder {
        let id = self.controls.reserve();

        ControlBuilder::new(self, id)
    }

    /// Recompute the layout of a control, and all of its children. This is need when modifing a
    /// control during layout.
    pub fn recompute_min_size(&mut self, id: Id) {
        let mut parents = vec![id];
        // post order traversal
        let mut i = 0;
        while i != parents.len() {
            parents.extend(self.get_active_children(parents[i]).iter().rev());
            i += 1;
        }
        while let Some(parent) = parents.pop() {
            let mut min_size = {
                let mut layout = self
                    .controls_mut()
                    .get_mut(parent)
                    .unwrap()
                    .layout
                    .take()
                    .unwrap();
                let mut ctx = MinSizeContext::new(parent, &mut self.controls, &self.fonts);
                let min_size = layout.compute_min_size(parent, &mut ctx);
                self.controls_mut().get_mut(parent).unwrap().layout = Some(layout);
                min_size
            };
            let parent = self.controls.get_mut(parent).unwrap();
            let user_min_size = parent.rect.user_min_size;
            min_size[0] = min_size[0].max(user_min_size[0]);
            min_size[1] = min_size[1].max(user_min_size[1]);
            parent.rect.min_size = min_size;
        }
    }

    pub fn set_rect(&mut self, id: Id, rect: [f32; 4]) {
        self.controls.get_mut(id).unwrap().rect.set_rect(rect);
    }

    pub fn set_designed_rect(&mut self, id: Id, rect: [f32; 4]) {
        self.controls
            .get_mut(id)
            .unwrap()
            .rect
            .set_designed_rect(rect);
    }

    pub fn get_layouting(&self, id: Id) -> &Rect {
        &self.controls.get(id).unwrap().rect
    }

    pub fn dirty_layout(&mut self, id: Id) {
        debug_assert!(
            !self.controls.is_child(self.this, id),
            "It is only allowed to modify a child using set_rect, or set_designed_rect."
        );
        debug_assert!(
            self.controls.is_descendant(self.this, id),
            "It is only allowed to modify descendant controls."
        );
        if !self.dirtys.iter().any(|x| *x == id) {
            self.dirtys.push(id);
        }
    }

    pub fn get_rect(&self, id: Id) -> [f32; 4] {
        self.controls.get(id).unwrap().rect.rect
    }

    pub fn get_size(&mut self, id: Id) -> [f32; 2] {
        self.controls.get(id).unwrap().rect.get_size()
    }

    pub fn get_margins(&self, id: Id) -> [f32; 4] {
        self.controls.get(id).unwrap().rect.margins
    }

    pub fn set_margins(&mut self, id: Id, margins: [f32; 4]) {
        self.controls.get_mut(id).unwrap().rect.margins = margins;
        self.dirty_layout(id);
    }

    pub fn get_anchors(&self, id: Id) -> [f32; 4] {
        self.controls.get(id).unwrap().rect.anchors
    }

    pub fn set_margin_left(&mut self, id: Id, margin: f32) {
        self.controls.get_mut(id).unwrap().rect.margins[0] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_top(&mut self, id: Id, margin: f32) {
        self.controls.get_mut(id).unwrap().rect.margins[1] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_right(&mut self, id: Id, margin: f32) {
        self.controls.get_mut(id).unwrap().rect.margins[2] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_bottom(&mut self, id: Id, margin: f32) {
        self.controls.get_mut(id).unwrap().rect.margins[3] = margin;
        self.dirty_layout(id);
    }

    pub fn set_anchors(&mut self, id: Id, anchors: [f32; 4]) {
        self.controls.get_mut(id).unwrap().rect.anchors = anchors;
        self.dirty_layout(id);
    }

    pub fn set_anchor_left(&mut self, id: Id, anchor: f32) {
        self.controls.get_mut(id).unwrap().rect.anchors[0] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_top(&mut self, id: Id, anchor: f32) {
        self.controls.get_mut(id).unwrap().rect.anchors[1] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_right(&mut self, id: Id, anchor: f32) {
        self.controls.get_mut(id).unwrap().rect.anchors[2] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_bottom(&mut self, id: Id, anchor: f32) {
        self.controls.get_mut(id).unwrap().rect.anchors[3] = anchor;
        self.dirty_layout(id);
    }

    pub fn get_min_size(&self, id: Id) -> [f32; 2] {
        self.controls.get(id).unwrap().rect.get_min_size()
    }

    pub fn set_min_size(&mut self, id: Id, min_size: [f32; 2]) {
        self.controls
            .get_mut(id)
            .unwrap()
            .rect
            .set_min_size(min_size);
        self.dirty_layout(id);
    }

    pub fn is_active(&self, id: Id) -> bool {
        self.controls.get(id).unwrap().active
    }

    /// This only took effect when Controls is dropped
    pub fn active(&mut self, id: Id) {
        self.events.push(Box::new(event::ActiveControl { id }));
    }

    /// This only took effect when Controls is dropped
    pub fn deactive(&mut self, id: Id) {
        self.events.push(Box::new(event::DeactiveControl { id }));
    }

    pub fn remove(&mut self, id: Id) {
        self.events.push(Box::new(event::RemoveControl { id }));
    }

    pub fn move_to_front(&mut self, id: Id) {
        self.controls.move_to_front(id);
        self.dirty_layout(id);
    }

    pub fn move_to_back(&mut self, id: Id) {
        self.controls.move_to_back(id);
        self.dirty_layout(id);
    }

    pub fn get_parent(&self, id: Id) -> Option<Id> {
        self.controls.get(id).unwrap().parent
    }

    pub fn get_all_children(&self, id: Id) -> &[Id] {
        self.controls.get_all_children(id).unwrap()
    }

    pub fn get_active_children(&self, id: Id) -> Vec<Id> {
        self.controls.get_active_children(id).unwrap()
    }
}

pub struct RenderContext<'a> {
    gui: &'a mut Gui,
    fonts: &'a Fonts,
    render_dirty: bool,
}
impl<'a> RenderContext<'a> {
    pub(crate) fn new(gui: &'a mut Gui) -> Self {
        let fonts = unsafe { std::mem::transmute(&gui.fonts) };
        Self {
            gui,
            fonts,
            render_dirty: false,
        }
    }

    /// Returns true if there is currently a animation playing in the Gui.
    pub fn is_animating(&self) -> bool {
        self.gui.animation_count() > 0
    }

    pub fn scale_factor(&self) -> f64 {
        self.gui.scale_factor()
    }

    pub fn get_fonts(&self) -> &'a Fonts {
        self.fonts
    }

    pub fn get_layouting(&mut self, id: Id) -> &mut Rect {
        &mut self.gui.controls.get_mut(id).unwrap().rect
    }

    pub fn get_rect(&self, id: Id) -> [f32; 4] {
        self.gui.controls.get(id).unwrap().rect.rect
    }

    pub fn get_size(&mut self, id: Id) -> [f32; 2] {
        self.gui.controls.get(id).unwrap().rect.get_size()
    }

    pub fn get_graphic_mut(&mut self, id: Id) -> &mut Graphic {
        self.render_dirty = true;
        let control = self.gui.controls.get_mut(id).unwrap();
        control.rect.dirty_render_dirty_flags();
        &mut control.graphic
    }

    pub fn get_rect_and_graphic(&mut self, id: Id) -> (&mut Rect, &mut Graphic) {
        let control = self.gui.controls.get_mut(id).unwrap();
        self.render_dirty = true;
        control.rect.dirty_render_dirty_flags();
        (&mut control.rect, &mut control.graphic)
    }

    pub fn is_active(&self, id: Id) -> bool {
        self.gui.controls.get(id).map_or(false, |x| x.active)
    }

    pub fn get_parent(&self, id: Id) -> Option<Id> {
        self.gui.controls.get(id).unwrap().parent
    }

    /// Get both active and deactive children.
    pub fn get_all_children(&self, id: Id) -> &[Id] {
        self.gui.controls.get_all_children(id).unwrap()
    }

    pub fn get_active_children(&self, id: Id) -> Vec<Id> {
        self.gui.controls.get_active_children(id).unwrap()
    }
}
