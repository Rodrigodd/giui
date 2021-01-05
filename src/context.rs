use std::any::Any;

use ab_glyph::FontArc;
use winit::event::ModifiersState;

use crate::{event, render::Graphic, Behaviour, ControlBuilder, Controls, Id, Layout, Rect, GUI};

// contains a reference to all the controls, except the behaviour of one control
pub struct Context<'a> {
    gui: &'a mut GUI,
    // modifiers: ModifiersState,
    // controls: &'a mut Controls,
    fonts: &'a [FontArc],
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
    pub(crate) fn new(gui: &'a mut GUI) -> Self {
        let fonts = unsafe { std::mem::transmute(gui.fonts.as_slice()) };
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
        gui: &'a mut GUI,
    ) -> Option<(&'a mut dyn Behaviour, Self)> {
        let this_one = unsafe {
            &mut *(gui.controls[this].behaviour.as_mut()?.as_mut() as *mut dyn Behaviour)
        };
        let fonts = unsafe { std::mem::transmute(gui.fonts.as_slice()) };
        Some((
            this_one,
            Self {
                gui,
                fonts,
                events: Vec::new(),
                events_to: Vec::new(),
                dirtys: Vec::new(),
                render_dirty: false,
            },
        ))
    }

    pub fn create_control(&mut self) -> ControlBuilder {
        let id = self.gui.controls.reserve();
        ControlBuilder::new(Box::new(move |build| {
            self.send_event((id, build));
            id
        }))
    }

    pub fn modifiers(&self) -> ModifiersState {
        self.gui.modifiers
    }

    pub fn get_fonts(&self) -> &'a [FontArc] {
        self.fonts
    }

    pub fn send_event<T: 'static>(&mut self, event: T) {
        self.events.push(Box::new(event));
    }
    pub fn send_event_to<T: 'static>(&mut self, id: Id, event: T) {
        self.events_to.push((id, Box::new(event)));
    }

    pub fn get_layouting(&mut self, id: Id) -> &mut Rect {
        &mut self.gui.controls[id].rect
    }

    pub fn dirty_layout(&mut self, id: Id) {
        if !self.dirtys.iter().any(|x| *x == id) {
            self.dirtys.push(id);
        }
    }

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
            Some((&mut control.rect, &mut control.graphic))
        }
    }

    pub fn is_active(&self, id: Id) -> bool {
        self.gui.controls[id].active
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

    pub fn get_children(&self, id: Id) -> Vec<Id> {
        self.gui.controls.get_children(id)
    }
}

pub struct MinSizeContext<'a> {
    this: Id,
    controls: &'a mut Controls,
    fonts: &'a [FontArc],
}
impl<'a> MinSizeContext<'a> {
    pub(crate) fn new(
        this: Id,
        controls: &'a mut Controls,
        fonts: &'a [FontArc],
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

    pub fn get_fonts(&mut self) -> &'a [FontArc] {
        self.fonts
    }

    pub fn get_layouting(&self, id: Id) -> &Rect {
        &self.controls[id].rect
    }

    pub fn get_rect(&self, id: Id) -> &[f32; 4] {
        &self.controls[id].rect.rect
    }

    pub fn get_size(&mut self, id: Id) -> [f32; 2] {
        self.controls[id].rect.get_size()
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

    pub fn get_rect_and_graphic(&mut self, id: Id) -> Option<(&mut Rect, &mut Graphic)> {
        let control = &mut self.controls[id];
        if let Graphic::None = control.graphic {
            None
        } else {
            Some((&mut control.rect, &mut control.graphic))
        }
    }

    pub fn is_active(&self, id: Id) -> bool {
        self.controls[id].active
    }

    pub fn get_parent(&self, id: Id) -> Option<Id> {
        self.controls[id].parent
    }

    pub fn get_children(&self, id: Id) -> Vec<Id> {
        self.controls.get_children(id)
    }
}

pub struct LayoutContext<'a> {
    this: Id,
    controls: &'a mut Controls,
    pub(crate) dirtys: Vec<Id>,
    pub(crate) events: Vec<Box<dyn Any>>,
}
impl<'a> LayoutContext<'a> {
    pub(crate) fn new(this: Id, controls: &'a mut Controls) -> (&'a mut dyn Layout, Self) {
        let this_one = unsafe { &mut *(controls[this].layout.as_mut() as *mut dyn Layout) };
        (
            this_one,
            Self {
                this,
                controls,
                dirtys: Vec::new(),
                events: Vec::new(),
            },
        )
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

    pub fn move_to_front(&mut self, id: Id) {
        self.controls.move_to_front(id);
        self.dirty_layout(id);
    }

    pub fn get_parent(&self, id: Id) -> Option<Id> {
        self.controls[id].parent
    }

    pub fn get_children(&self, id: Id) -> Vec<Id> {
        self.controls.get_children(id)
    }
}
