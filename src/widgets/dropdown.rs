use crate::{event, style::ButtonStyle, Behaviour, Context, Id, MouseEvent, MouseButton};

use std::any::Any;

struct SetIndex(usize);
// struct SetOwner(Id);
// struct SetItens<T: 'static + Clone>(Vec<T>);
// struct SetFocus(usize);
pub struct ShowMenu<T: 'static + Clone>(pub Id, pub Option<usize>, pub Vec<T>);
pub struct CloseMenu;
#[derive(Clone, Copy)]
pub struct ItemClicked {
    pub index: usize,
}
struct MenuClosed;

pub struct MenuItem {
    index: usize,
    state: u8,
    menu: Id,
    style: ButtonStyle,
    focus: bool,
}
impl MenuItem {
    pub fn new(menu: Id, style: ButtonStyle) -> Self {
        Self {
            index: 0,
            state: 0,
            menu,
            style,
            focus: false,
        }
    }
}
impl Behaviour for MenuItem {
    fn on_active(&mut self, this: Id, ctx: &mut Context) {
        ctx.set_graphic(this, self.style.normal.clone());
    }

    fn on_event(&mut self, event: &dyn Any, _this: Id, _ctx: &mut Context) {
        if let Some(SetIndex(index)) = event.downcast_ref() {
            self.index = *index;
        }
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        use MouseButton::*;
        match event {
            MouseEvent::Enter => {
                self.state = 1;
                ctx.set_graphic(this, self.style.hover.clone());
            }
            MouseEvent::Exit => {
                self.state = 0;
                if self.focus {
                    ctx.set_graphic(this, self.style.focus.clone());
                } else {
                    ctx.set_graphic(this, self.style.normal.clone());
                }
            }
            MouseEvent::Down(Left) => {
                self.state = 2;
                ctx.set_graphic(this, self.style.pressed.clone());
            }
            MouseEvent::Up(Left) => {
                if self.state == 2 {
                    ctx.send_event_to(self.menu, ItemClicked { index: self.index });
                }
                self.state = 1;
                ctx.set_graphic(this, self.style.hover.clone());
            }
            _ => {}
        }
        true
    }

    fn on_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {
        self.focus = focus;
        if self.state == 0 {
            if focus {
                ctx.set_graphic(this, self.style.focus.clone());
            } else {
                ctx.set_graphic(this, self.style.normal.clone());
            }
        }
    }
}

pub struct Blocker<F: Fn(Id, &mut Context)> {
    on_down: F,
}
impl<F: Fn(Id, &mut Context)> Blocker<F> {
    pub fn new(on_down: F) -> Self {
        Self { on_down }
    }
}
impl<F: Fn(Id, &mut Context)> Behaviour for Blocker<F> {
    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        if let MouseEvent::Down(_) = event {
            (self.on_down)(this, ctx);
        }
        true
    }
}

pub struct Menu<T, F>
where
    T: 'static + Clone,
    F: Fn(&T, Id, &mut Context) -> Id,
{
    blocker: Id,
    list: Vec<T>,
    create_item: F,
    owner: Id,
}
impl<T, F> Menu<T, F>
where
    T: 'static + Clone,
    F: Fn(&T, Id, &mut Context) -> Id,
{
    pub fn new(blocker: Id, create_item: F) -> Self {
        Self {
            blocker,
            list: Vec::new(),
            create_item,
            owner: crate::ROOT_ID,
        }
    }

    pub fn close(&self, this: Id, ctx: &mut Context) {
        ctx.deactive(this);
        ctx.deactive(self.blocker);
        ctx.send_event_to(self.owner, MenuClosed);
    }
}
impl<T, F> Behaviour for Menu<T, F>
where
    T: 'static + Clone + std::fmt::Debug,
    F: Fn(&T, Id, &mut Context) -> Id,
{
    fn on_event(&mut self, event: &dyn Any, this: Id, ctx: &mut Context) {
        if let Some(ShowMenu(owner, selected, itens)) = event.downcast_ref() {
            // set owner
            self.owner = *owner;
            // set focus
            if selected.is_none() {
                ctx.send_event(event::RequestFocus { id: this });
            }
            // set itens
            for child in ctx.get_children(this) {
                ctx.remove(child);
            }
            for (i, item) in itens.iter().enumerate() {
                let id = (self.create_item)(item, this, ctx);
                ctx.send_event_to(id, SetIndex(i));
                // set focus
                if let Some(index) = selected {
                    if *index == i {
                        ctx.send_event(event::RequestFocus { id });
                    }
                }
            }
            self.list = itens.clone();
            // active blocker
            ctx.active(this);
            ctx.active(self.blocker);
            ctx.move_to_front(self.blocker);
            ctx.move_to_front(this);
        } else if event.is::<CloseMenu>() {
            self.close(this, ctx);
        } else if let Some(x) = event.downcast_ref::<ItemClicked>() {
            ctx.send_event_to(self.owner, *x);
            self.close(this, ctx);
        } else {
        }
    }
}

pub struct Dropdown<T, F>
where
    T: 'static + Clone,
    F: Fn(T, Id, &mut Context),
{
    itens: Vec<T>,
    selected: Option<usize>,
    menu: Id,
    state: u8,
    style: ButtonStyle,
    focus: bool,
    on_select: F,
    opened: bool,
}
impl<T, F> Dropdown<T, F>
where
    T: 'static + Clone,
    F: Fn(T, Id, &mut Context),
{
    pub fn new(itens: Vec<T>, menu: Id, on_select: F, style: ButtonStyle) -> Self {
        Self {
            itens,
            selected: None,
            menu,
            state: 0,
            style,
            focus: false,
            on_select,
            opened: false,
        }
    }
}
impl<T, F> Behaviour for Dropdown<T, F>
where
    T: 'static + Clone,
    F: Fn(T, Id, &mut Context),
{
    fn on_active(&mut self, this: Id, ctx: &mut Context) {
        ctx.set_graphic(this, self.style.normal.clone());
    }

    fn on_event(&mut self, event: &dyn Any, this: Id, ctx: &mut Context) {
        if let Some(x) = event.downcast_ref::<ItemClicked>() {
            self.selected = Some(x.index);
            (self.on_select)(self.itens[x.index].clone(), this, ctx);
            self.opened = false;
        } else if event.is::<MenuClosed>() {
            self.opened = false;
        }
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        use MouseButton::*;
        match event {
            MouseEvent::Enter => {
                self.state = 1;
                ctx.set_graphic(this, self.style.hover.clone());
            }
            MouseEvent::Exit => {
                self.state = 0;
                if self.focus {
                    ctx.set_graphic(this, self.style.focus.clone());
                } else {
                    ctx.set_graphic(this, self.style.normal.clone());
                }
            }
            MouseEvent::Down(Left) => {
                self.state = 2;
                ctx.set_graphic(this, self.style.pressed.clone());
            }
            MouseEvent::Up(Left) => {
                if self.state == 2 {
                    if !self.opened {
                        self.opened = true;
                        // ctx.active(self.menu);
                        let size = *ctx.get_rect(this);
                        ctx.set_anchors(self.menu, [0.0, 0.0, 0.0, 0.0]);
                        ctx.set_margins(self.menu, [size[0], size[3], size[2], size[3]]);
                        ctx.send_event_to(
                            self.menu,
                            ShowMenu(this, self.selected, self.itens.clone()),
                        );
                    // ctx.send_event_to(self.menu, SetOwner(this));
                    // ctx.send_event_to(self.menu, SetItens(self.itens.clone()));
                    // if let Some(selected) = self.selected {
                    //     ctx.send_event_to(self.menu, SetFocus(selected));
                    // } else {
                    //     ctx.send_event(event::RequestFocus { id: self.menu });
                    // }
                    } else {
                        self.opened = false;
                        ctx.deactive(self.menu);
                    }
                }
                self.state = 1;
                ctx.set_graphic(this, self.style.hover.clone());
            }
            _ => {}
        }
        true
    }

    fn on_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {
        self.focus = focus;
        if self.state == 0 {
            if focus {
                ctx.set_graphic(this, self.style.focus.clone());
            } else {
                ctx.set_graphic(this, self.style.normal.clone());
            }
        }
    }
}