use crate::{Behaviour, Context, Id, KeyboardEvent};

pub struct OnKeyboardEvent<F, B: Behaviour>
where
    F: FnMut(KeyboardEvent, Id, &mut Context) -> bool,
{
    on_keyboard: F,
    extends: B,
}

impl<F> OnKeyboardEvent<F, ()>
where
    F: FnMut(KeyboardEvent, Id, &mut Context) -> bool,
{
    pub fn new(on_keyboard: F) -> Self {
        Self {
            on_keyboard,
            extends: (),
        }
    }

    pub fn extends<B: Behaviour>(self, behaviour: B) -> OnKeyboardEvent<F, B> {
        OnKeyboardEvent {
            on_keyboard: self.on_keyboard,
            extends: behaviour,
        }
    }
}

impl<F, B: Behaviour> Behaviour for OnKeyboardEvent<F, B>
where
    F: FnMut(KeyboardEvent, Id, &mut Context) -> bool,
{
    fn input_flags(&self) -> crate::InputFlags {
        crate::InputFlags::FOCUS | crate::InputFlags::MOUSE | self.extends.input_flags()
    }

    fn on_keyboard_event(&mut self, event: KeyboardEvent, this: Id, ctx: &mut Context) -> bool {
        self.extends.on_keyboard_event(event, this, ctx) || (self.on_keyboard)(event, this, ctx)
    }

    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        self.extends.on_start(this, ctx)
    }

    fn on_active(&mut self, this: Id, ctx: &mut Context) {
        self.extends.on_active(this, ctx)
    }

    fn on_deactive(&mut self, this: Id, ctx: &mut Context) {
        self.extends.on_deactive(this, ctx)
    }

    fn on_remove(&mut self, this: Id, ctx: &mut Context) {
        self.extends.on_remove(this, ctx)
    }

    fn on_event(&mut self, event: Box<dyn std::any::Any>, this: Id, ctx: &mut Context) {
        self.extends.on_event(event, this, ctx)
    }

    fn on_scroll_event(&mut self, delta: [f32; 2], this: Id, ctx: &mut Context) {
        self.extends.on_scroll_event(delta, this, ctx)
    }

    fn on_mouse_event(&mut self, mouse: crate::MouseInfo, this: Id, ctx: &mut Context) {
        self.extends.on_mouse_event(mouse, this, ctx)
    }

    fn on_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {
        self.extends.on_focus_change(focus, this, ctx)
    }
}
