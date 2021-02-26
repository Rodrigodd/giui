use crate::{style::TabStyle, Behaviour, Context, Id, InputFlags, MouseButton, MouseEvent};

use std::rc::Rc;
use std::{any::Any, cell::RefCell};

struct Unselected;
pub struct Select;

struct ButtonGroupInner {
    selected: Option<Id>,
    on_change: Box<dyn Fn(Id, &mut Context)>,
}

#[derive(Clone)]
/// It is basically a Rc<RefCell<Option<Id>>>.
pub struct ButtonGroup(Rc<RefCell<ButtonGroupInner>>);
impl ButtonGroup {
    pub fn new<F: Fn(Id, &mut Context) + 'static>(on_change: F) -> Self {
        Self(Rc::new(RefCell::new(ButtonGroupInner {
            selected: None,
            on_change: Box::new(on_change),
        })))
    }
    pub fn selected(&self) -> Option<Id> {
        self.0.borrow().selected
    }
    pub fn set_selected(&mut self, selected: Option<Id>, ctx: &mut Context) {
        let mut this = self.0.borrow_mut();
        this.selected = selected;
        (this.on_change)(selected.expect("None selected is not implemented yet"), ctx);
    }
}

pub struct TabButton {
    tab_group: ButtonGroup,
    page: Id,
    selected: bool,
    click: bool,
    style: Rc<TabStyle>,
}
impl TabButton {
    pub fn new(tab_group: ButtonGroup, page: Id, selected: bool, style: Rc<TabStyle>) -> Self {
        Self {
            tab_group,
            page,
            selected,
            click: false,
            style,
        }
    }

    fn select(&mut self, this: Id, ctx: &mut Context) {
        if let Some(selected) = self.tab_group.selected() {
            if selected == this {
                return;
            }
            ctx.send_event_to(selected, Unselected);
        }
        self.selected = true;
        self.tab_group.set_selected(Some(this), ctx);
        ctx.set_graphic(this, self.style.selected.clone());
    }

    fn unselect(&mut self, this: Id, ctx: &mut Context) {
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

    fn on_event(&mut self, event: Box<dyn Any>, this: Id, ctx: &mut Context) {
        if event.is::<Unselected>() {
            self.unselect(this, ctx)
        } else if event.is::<Select>() {
            self.select(this, ctx);
        }
    }

    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) {
        use MouseButton::*;
        match event {
            MouseEvent::Enter => {
                self.click = false;
                if !self.selected {
                    ctx.set_graphic(this, self.style.hover.clone());
                }
            }
            MouseEvent::Exit => {
                if !self.selected {
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
    }
}
