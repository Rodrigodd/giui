use std::any::TypeId;
use std::{any::Any, time::Instant};

use winit::{event::ModifiersState, window::CursorIcon};

use crate::{
    control::ControlBuilderInner, event, font::Fonts, graphics::Graphic, Behaviour, ControlBuilder,
    Controls, Gui, Id, Layout, Rect,
};

// contains a reference to all the controls, except the behaviour of one control
pub struct Context<'a> {
    gui: &'a mut Gui,
    // modifiers: ModifiersState,
    // controls: &'a mut Controls,
    fonts: &'a Fonts,
    pub(crate) events: Vec<Box<dyn Any>>,
    pub(crate) events_to: Vec<(Id, Box<dyn Any>)>,
    pub(crate) dirtys: Vec<Id>,
    pub(crate) render_dirty: bool,
}
impl<'a> Drop for Context<'a> {
    fn drop(&mut self) {
        let Context {
            events,
            events_to,
            dirtys,
            render_dirty,
            ..
        } = self;
        self.gui
            .context_drop(events, events_to, dirtys, *render_dirty);
    }
}
impl<'a> Context<'a> {
    pub(crate) fn new(gui: &'a mut Gui) -> Self {
        let fonts = unsafe { std::mem::transmute(&gui.fonts) };
        Self {
            gui,
            fonts,
            events: Vec::new(),
            events_to: Vec::new(),
            dirtys: Vec::new(),
            render_dirty: false,
        }
    }

    pub(crate) fn new_with_mut_behaviour(
        this: Id,
        gui: &'a mut Gui,
    ) -> Option<(&'a mut dyn Behaviour, Self)> {
        let this_one = unsafe {
            &mut *(gui.controls.get_mut(this)?.behaviour.as_mut()?.as_mut() as *mut dyn Behaviour)
        };
        Some((this_one, Self::new(gui)))
    }

    /// Set the value of the type T that is owned by the Gui. Any value set before will be dropped
    /// and replaced.
    pub fn set<T: Any + 'static>(&mut self, value: T) {
        self.gui.resources.insert(TypeId::of::<T>(), Box::new(value));
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

        struct Builder<'a, 'b>(&'b mut Context<'a>);
        impl ControlBuilderInner for Builder<'_, '_> {
            fn controls(&mut self) -> &mut Controls {
                &mut self.0.gui.controls
            }
            fn build(&mut self, id: Id) {
                self.0.send_event(event::CreateControl { id });
            }
        }

        ControlBuilder::new(id, Builder(self))
    }

    pub fn modifiers(&self) -> ModifiersState {
        self.gui.modifiers
    }

    pub fn get_fonts(&self) -> &'a Fonts {
        self.fonts
    }

    pub fn send_event<T: 'static>(&mut self, event: T) {
        self.events.push(Box::new(event));
    }
    pub fn send_event_to<T: 'static>(&mut self, id: Id, event: T) {
        self.events_to.push((id, Box::new(event)));
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

    pub fn set_cursor(&mut self, cursor: CursorIcon) {
        self.send_event(cursor);
    }

    /// If lock is true, locks the cursor over the current control that is receiving mouse events.
    /// This means that even if the mouse position go out of the area of the control, the control
    /// will continue receiving mouse events, and MouseExit will not be emitted. This is useful
    /// for dragging behavior.
    pub fn lock_cursor(&mut self, lock: bool) {
        if lock {
            self.send_event(event::LockOver);
        } else {
            self.send_event(event::UnlockOver);
        }
    }

    pub fn get_layouting(&mut self, id: Id) -> &mut Rect {
        &mut self.gui.controls[id].rect
    }

    pub fn dirty_layout(&mut self, id: Id) {
        if !self.dirtys.iter().any(|x| *x == id) {
            self.dirtys.push(id);
        }
    }

    // TODO: this should not return a reference?
    pub fn get_rect(&self, id: Id) -> &[f32; 4] {
        &self.gui.controls[id].rect.rect
    }

    pub fn get_size(&mut self, id: Id) -> [f32; 2] {
        self.gui.controls[id].rect.get_size()
    }

    pub fn get_margins(&self, id: Id) -> &[f32; 4] {
        &self.gui.controls[id].rect.margins
    }

    pub fn set_margins(&mut self, id: Id, margins: [f32; 4]) {
        self.gui.controls[id].rect.margins = margins;
        self.dirty_layout(id);
    }

    pub fn get_anchors(&self, id: Id) -> &[f32; 4] {
        &self.gui.controls[id].rect.anchors
    }

    pub fn set_margin_left(&mut self, id: Id, margin: f32) {
        self.gui.controls[id].rect.margins[0] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_top(&mut self, id: Id, margin: f32) {
        self.gui.controls[id].rect.margins[1] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_right(&mut self, id: Id, margin: f32) {
        self.gui.controls[id].rect.margins[2] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_bottom(&mut self, id: Id, margin: f32) {
        self.gui.controls[id].rect.margins[3] = margin;
        self.dirty_layout(id);
    }

    pub fn set_anchors(&mut self, id: Id, anchors: [f32; 4]) {
        self.gui.controls[id].rect.anchors = anchors;
        self.dirty_layout(id);
    }

    pub fn set_anchor_left(&mut self, id: Id, anchor: f32) {
        self.gui.controls[id].rect.anchors[0] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_top(&mut self, id: Id, anchor: f32) {
        self.gui.controls[id].rect.anchors[1] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_right(&mut self, id: Id, anchor: f32) {
        self.gui.controls[id].rect.anchors[2] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_bottom(&mut self, id: Id, anchor: f32) {
        self.gui.controls[id].rect.anchors[3] = anchor;
        self.dirty_layout(id);
    }

    pub fn get_min_size(&self, id: Id) -> [f32; 2] {
        self.gui.controls[id].rect.get_min_size()
    }

    pub fn set_min_size(&mut self, id: Id, min_size: [f32; 2]) {
        self.gui.controls[id].rect.set_min_size(min_size);
        self.dirty_layout(id);
    }

    pub fn get_graphic_mut(&mut self, id: Id) -> &mut Graphic {
        self.render_dirty = true;
        &mut self.gui.controls[id].graphic
    }

    pub fn set_graphic(&mut self, id: Id, graphic: Graphic) {
        let control = &mut self.gui.controls[id];
        control.graphic = graphic;
        control.rect.dirty_render_dirty_flags();
        self.render_dirty = true;
    }

    pub fn get_rect_and_graphic(&mut self, id: Id) -> Option<(&mut Rect, &mut Graphic)> {
        let control = &mut self.gui.controls[id];
        if let Graphic::None = control.graphic {
            None
        } else {
            self.render_dirty = true;
            Some((&mut control.rect, &mut control.graphic))
        }
    }

    pub fn is_active(&self, id: Id) -> bool {
        self.gui.controls.get(id).map_or(false, |x| x.active)
    }

    pub fn set_focus(&mut self, id: Id) {
        self.events.push(Box::new(event::RequestFocus { id }));
    }

    pub fn get_focus(&mut self) -> Option<Id> {
        self.gui.current_focus
    }

    pub fn is_focus(&self, id: Id) -> bool {
        self.gui.controls[id].focus
    }

    /// Set MouseInfo::click_count to 1, wich keep track of consecutives clicks.
    /// This means that, if called, if the next click is consecutive,
    /// it will have a click count of 2.
    pub fn reset_click_count_to_one(&mut self) {
        self.gui.input.click_count = 1;
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
        self.gui.controls.move_to_front(id);
        self.dirty_layout(id);
    }

    pub fn get_parent(&self, id: Id) -> Option<Id> {
        self.gui.controls[id].parent
    }

    pub fn get_active_children(&self, id: Id) -> Vec<Id> {
        self.gui.controls.get_active_children(id)
    }
}

pub struct MinSizeContext<'a> {
    this: Id,
    controls: &'a mut Controls,
    fonts: &'a Fonts,
}
impl<'a> MinSizeContext<'a> {
    pub(crate) fn new(
        this: Id,
        controls: &'a mut Controls,
        fonts: &'a Fonts,
    ) -> (&'a mut dyn Layout, Self) {
        let this_one = unsafe { &mut *(controls[this].layout.as_mut() as *mut dyn Layout) };
        (
            this_one,
            Self {
                this,
                controls,
                fonts,
            },
        )
    }

    pub fn get_fonts(&mut self) -> &'a Fonts {
        self.fonts
    }

    pub fn get_layouting(&self, id: Id) -> &Rect {
        &self.controls[id].rect
    }

    pub fn get_margins(&self, id: Id) -> &[f32; 4] {
        &self.controls[id].rect.margins
    }

    pub fn get_anchors(&self, id: Id) -> &[f32; 4] {
        &self.controls[id].rect.anchors
    }

    pub fn get_min_size(&self, id: Id) -> [f32; 2] {
        self.controls[id].rect.get_min_size()
    }

    pub fn set_this_min_size(&mut self, min_size: [f32; 2]) {
        self.controls[self.this].rect.set_min_size(min_size);
    }

    pub fn get_graphic(&mut self, id: Id) -> &mut Graphic {
        &mut self.controls[id].graphic
    }

    // pub fn get_rect_and_graphic(&mut self, id: Id) -> Option<(&mut Rect, &mut Graphic)> {
    //     let control = &mut self.controls[id];
    //     if let Graphic::None = control.graphic {
    //         None
    //     } else {
    //         Some((&mut control.rect, &mut control.graphic))
    //     }
    // }

    pub fn is_active(&self, id: Id) -> bool {
        self.controls[id].active
    }

    // pub fn get_parent(&self, id: Id) -> Option<Id> {
    //     self.controls[id].parent
    // }

    pub fn get_active_children(&self, id: Id) -> Vec<Id> {
        self.controls.get_active_children(id)
    }
}

pub struct LayoutContext<'a> {
    this: Id,
    controls: &'a mut Controls,
    fonts: &'a Fonts,
    pub(crate) dirtys: Vec<Id>,
    pub(crate) events: Vec<Box<dyn Any>>,
}
impl<'a> LayoutContext<'a> {
    pub(crate) fn new(
        this: Id,
        controls: &'a mut Controls,
        fonts: &'a Fonts,
    ) -> (&'a mut dyn Layout, Self) {
        let this_one = unsafe { &mut *(controls[this].layout.as_mut() as *mut dyn Layout) };
        (
            this_one,
            Self {
                this,
                controls,
                fonts,
                dirtys: Vec::new(),
                events: Vec::new(),
            },
        )
    }

    pub fn create_control(&mut self) -> ControlBuilder {
        let id = self.controls.reserve();

        struct Builder<'a, 'b>(&'b mut LayoutContext<'a>);
        impl ControlBuilderInner for Builder<'_, '_> {
            fn controls(&mut self) -> &mut Controls {
                &mut self.0.controls
            }
            fn build(&mut self, id: Id) {
                self.0.events.push(Box::new(event::CreateControl { id }));

                let mut parents = vec![id];
                // post order traversal
                let mut i = 0;
                while i != parents.len() {
                    parents.extend(self.0.get_active_children(parents[i]).iter().rev());
                    i += 1;
                }
                while let Some(parent) = parents.pop() {
                    let (layout, mut ctx) =
                        MinSizeContext::new(parent, &mut self.0.controls, &self.0.fonts);
                    let mut min_size = layout.compute_min_size(parent, &mut ctx);
                    let user_min_size = self.0.controls[parent].rect.user_min_size;
                    min_size[0] = min_size[0].max(user_min_size[0]);
                    min_size[1] = min_size[1].max(user_min_size[1]);
                    self.0.controls[parent].rect.min_size = min_size;
                }
            }
        }

        ControlBuilder::new(id, Builder(self))
    }

    pub fn set_rect(&mut self, id: Id, rect: [f32; 4]) {
        self.controls[id].rect.set_rect(rect);
    }

    pub fn set_designed_rect(&mut self, id: Id, rect: [f32; 4]) {
        self.controls[id].rect.set_designed_rect(rect);
    }

    pub fn get_layouting(&self, id: Id) -> &Rect {
        &self.controls[id].rect
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

    // TODO: Return a reference?
    pub fn get_rect(&self, id: Id) -> &[f32; 4] {
        &self.controls[id].rect.rect
    }

    pub fn get_size(&mut self, id: Id) -> [f32; 2] {
        self.controls[id].rect.get_size()
    }

    pub fn get_margins(&self, id: Id) -> &[f32; 4] {
        &self.controls[id].rect.margins
    }

    pub fn set_margins(&mut self, id: Id, margins: [f32; 4]) {
        self.controls[id].rect.margins = margins;
        self.dirty_layout(id);
    }

    pub fn get_anchors(&self, id: Id) -> &[f32; 4] {
        &self.controls[id].rect.anchors
    }

    pub fn set_margin_left(&mut self, id: Id, margin: f32) {
        self.controls[id].rect.margins[0] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_top(&mut self, id: Id, margin: f32) {
        self.controls[id].rect.margins[1] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_right(&mut self, id: Id, margin: f32) {
        self.controls[id].rect.margins[2] = margin;
        self.dirty_layout(id);
    }

    pub fn set_margin_bottom(&mut self, id: Id, margin: f32) {
        self.controls[id].rect.margins[3] = margin;
        self.dirty_layout(id);
    }

    pub fn set_anchors(&mut self, id: Id, anchors: [f32; 4]) {
        self.controls[id].rect.anchors = anchors;
        self.dirty_layout(id);
    }

    pub fn set_anchor_left(&mut self, id: Id, anchor: f32) {
        self.controls[id].rect.anchors[0] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_top(&mut self, id: Id, anchor: f32) {
        self.controls[id].rect.anchors[1] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_right(&mut self, id: Id, anchor: f32) {
        self.controls[id].rect.anchors[2] = anchor;
        self.dirty_layout(id);
    }

    pub fn set_anchor_bottom(&mut self, id: Id, anchor: f32) {
        self.controls[id].rect.anchors[3] = anchor;
        self.dirty_layout(id);
    }

    pub fn get_min_size(&self, id: Id) -> [f32; 2] {
        self.controls[id].rect.get_min_size()
    }

    pub fn set_min_size(&mut self, id: Id, min_size: [f32; 2]) {
        self.controls[id].rect.set_min_size(min_size);
        self.dirty_layout(id);
    }

    pub fn is_active(&self, id: Id) -> bool {
        self.controls[id].active
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
        self.controls[id].parent
    }

    pub fn get_active_children(&self, id: Id) -> Vec<Id> {
        self.controls.get_active_children(id)
    }
}
