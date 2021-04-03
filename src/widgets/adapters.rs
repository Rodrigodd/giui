use crate::{Behaviour, Context, Id, KeyboardEvent};

pub struct OnKeyboardEvent<F>(pub F)
where
    F: FnMut(KeyboardEvent, Id, &mut Context) -> bool;
 
impl<F> Behaviour for OnKeyboardEvent<F>
where
    F: FnMut(KeyboardEvent, Id, &mut Context) -> bool,
{
    fn on_keyboard_event(&mut self, event: KeyboardEvent, this: Id, ctx: &mut Context) -> bool {
        (self.0)(event, this, ctx)
    }
}
