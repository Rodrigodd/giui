use crate::{
    style::ButtonStyle, Behaviour, Context, Id, InputFlags, MouseButton, MouseEvent, MouseInfo,
};

use std::{any::Any, rc::Rc};

pub struct SetSelected(pub usize);
struct SetIndex(usize);
// struct SetOwner(Id);
// struct SetItens<T: 'static + Clone>(Vec<T>);
// struct SetFocus(usize);
struct ShowMenu<T: 'static + Clone>(pub Id, pub Option<usize>, pub Vec<T>);
pub struct CloseMenu;
#[derive(Clone, Copy)]
struct ItemClicked {
    pub index: usize,
}
struct MenuClosed;

pub struct MenuItem {
    index: usize,
    state: u8,
    menu: Id,
    style: Rc<ButtonStyle>,
    focus: bool,
}
impl MenuItem {
    pub fn new(menu: Id, style: Rc<ButtonStyle>) -> Self {
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

    fn on_event(&mut self, event: Box<dyn Any>, _this: Id, _ctx: &mut Context) {
        if let Some(SetIndex(index)) = event.downcast_ref() {
            self.index = *index;
        }
    }

    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE | InputFlags::FOCUS
    }

    fn on_mouse_event(&mut self, mouse: MouseInfo, this: Id, ctx: &mut Context) {
        use MouseButton::*;
        match mouse.event {
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
    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE
    }

    fn on_mouse_event(&mut self, mouse: MouseInfo, this: Id, ctx: &mut Context) {
        if let MouseEvent::Down(_) = mouse.event {
            (self.on_down)(this, ctx);
        }
    }
}

pub struct DropMenu<T, F>
where
    T: 'static + Clone,
    F: Fn(&T, Id, &mut Context) -> Id,
{
    blocker: Id,
    list: Vec<T>,
    create_item: F,
    owner: Id,
}
impl<T, F> DropMenu<T, F>
where
    T: 'static + Clone,
    F: Fn(&T, Id, &mut Context) -> Id,
{
    pub fn new(blocker: Id, create_item: F) -> Self {
        Self {
            blocker,
            list: Vec::new(),
            create_item,
            owner: crate::Id::ROOT_ID,
        }
    }

    pub fn close(&self, this: Id, ctx: &mut Context) {
        ctx.deactive(this);
        ctx.deactive(self.blocker);
        ctx.send_event_to(self.owner, MenuClosed);
    }
}
impl<T, F> Behaviour for DropMenu<T, F>
where
    T: 'static + Clone + std::fmt::Debug,
    F: Fn(&T, Id, &mut Context) -> Id,
{
    fn on_event(&mut self, event: Box<dyn Any>, this: Id, ctx: &mut Context) {
        if let Some(ShowMenu(owner, selected, itens)) = event.downcast_ref() {
            // set owner
            self.owner = *owner;
            // set focus
            if selected.is_none() {
                ctx.set_focus(this);
            }
            // set itens
            for child in ctx.get_active_children(this) {
                ctx.remove(child);
            }
            for (i, item) in itens.iter().enumerate() {
                let id = (self.create_item)(item, this, ctx);
                ctx.send_event_to(id, SetIndex(i));
                // set focus
                if let Some(index) = selected {
                    if *index == i {
                        ctx.set_focus(id);
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
    F: Fn((usize, T), Id, &mut Context),
{
    itens: Vec<T>,
    selected: Option<usize>,
    menu: Id,
    state: u8,
    style: Rc<ButtonStyle>,
    focus: bool,
    on_select: F,
    opened: bool,
}
impl<T, F> Dropdown<T, F>
where
    T: 'static + Clone,
    F: Fn((usize, T), Id, &mut Context),
{
    pub fn new(
        itens: Vec<T>,
        intial_selected: Option<usize>,
        menu: Id,
        on_select: F,
        style: Rc<ButtonStyle>,
    ) -> Self {
        Self {
            itens,
            selected: intial_selected,
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
    F: Fn((usize, T), Id, &mut Context),
{
    fn on_active(&mut self, this: Id, ctx: &mut Context) {
        ctx.set_graphic(this, self.style.normal.clone());
    }

    fn on_event(&mut self, event: Box<dyn Any>, this: Id, ctx: &mut Context) {
        if let Some(x) = event.downcast_ref::<ItemClicked>() {
            self.selected = Some(x.index);
            (self.on_select)((x.index, self.itens[x.index].clone()), this, ctx);
            self.opened = false;
        } else if event.is::<MenuClosed>() {
            self.opened = false;
        } else if let Some(SetSelected(index)) = event.downcast_ref() {
            self.selected = Some(*index);
            (self.on_select)((*index, self.itens[*index].clone()), this, ctx);
        }
    }

    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE | InputFlags::FOCUS
    }

    fn on_mouse_event(&mut self, mouse: MouseInfo, this: Id, ctx: &mut Context) {
        use MouseButton::*;
        match mouse.event {
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
                        let size = {
                            let root = ctx.get_rect(Id::ROOT_ID);
                            let rect = ctx.get_rect(this);
                            [
                                rect[0] - root[0],
                                rect[1] - root[1],
                                rect[2] - root[0],
                                rect[3] - root[1],
                            ]
                        };
                        ctx.set_anchors(self.menu, [0.0, 0.0, 0.0, 0.0]);
                        ctx.set_margins(self.menu, [size[0], size[3], size[2], size[3]]);
                        ctx.send_event_to(
                            self.menu,
                            ShowMenu(this, self.selected, self.itens.clone()),
                        );
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
