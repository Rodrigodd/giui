use crate::{style::TabStyle, Behaviour, Context, Id, MouseEvent, MouseButton};

use std::any::Any;

struct Unselected;
struct Selected;

#[derive(Default, Clone)]
/// It is basically a Rc<RefCell<Option<Id>>>.
pub struct ButtonGroup(std::rc::Rc<std::cell::RefCell<Option<Id>>>);
impl ButtonGroup {
    pub fn new() -> ButtonGroup {
        ButtonGroup::default()
    }
    pub fn selected(&self) -> Option<Id> {
        *self.0.borrow()
    }
    pub fn set_selected(&mut self, selected: Option<Id>) {
        *self.0.borrow_mut() = selected;
    }
}

pub struct TabButton {
    tab_group: ButtonGroup,
    page: Id,
    selected: bool,
    click: bool,
    style: TabStyle,
}
impl TabButton {
    pub fn new(tab_group: ButtonGroup, page: Id, selected: bool, style: TabStyle) -> Self {
        Self {
            tab_group,
            page,
            selected,
            click: false,
            style,
        }
    }

    pub fn select(&mut self, this: Id, ctx: &mut Context) {
        if let Some(selected) = self.tab_group.selected() {
            ctx.send_event_to(selected, Unselected);
        }
        ctx.active(self.page);
        self.selected = true;
        self.tab_group.set_selected(Some(this));
        ctx.set_graphic(this, self.style.selected.clone());
    }

    pub fn unselect(&mut self, this: Id, ctx: &mut Context) {
        ctx.deactive(self.page);
        self.selected = false;
        ctx.set_graphic(this, self.style.unselected.clone());
    }
}
impl Behaviour for TabButton {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        if self.selected {
            self.select(this, ctx);
        } else {
            self.unselect(this, ctx);
        }
    }

    fn on_event(&mut self, event: &dyn Any, this: Id, ctx: &mut Context) {
        if event.is::<Unselected>() {
            self.unselect(this, ctx)
        } else if event.is::<Selected>() {
            self.select(this, ctx);
        }
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        use MouseButton::*;
        match event {
            MouseEvent::Enter => {
                if !self.selected {
                    ctx.set_graphic(this, self.style.hover.clone());
                }
            }
            MouseEvent::Exit => {
                if !self.selected {
                    self.click = false;
                    ctx.set_graphic(this, self.style.unselected.clone());
                }
            }
            MouseEvent::Down(Left) => {
                if !self.selected {
                    self.click = true;
                    ctx.set_graphic(this, self.style.pressed.clone());
                }
            }
            MouseEvent::Up(Left) => {
                if !self.selected {
                    if self.click {
                        self.select(this, ctx);
                    } else {
                        ctx.set_graphic(this, self.style.unselected.clone());
                    }
                }
            }
            MouseEvent::Up(_) => {}
            MouseEvent::Down(_) => {}
            MouseEvent::Moved { .. } => {}
        }
        true
    }
}
