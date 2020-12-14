use crate::{
    event, widgets::ButtonStyle, Behaviour, Context, Id, LayoutContext, MinSizeContext, MouseEvent,
};

use std::any::Any;

struct SetIndex(usize);
// struct SetOwner(Id);
// struct SetItens<T: 'static + Clone>(Vec<T>);
// struct SetFocus(usize);
struct ShowMenu<T: 'static + Clone>(Id, Option<usize>, Vec<T>);
struct CloseMenu;
#[derive(Clone, Copy)]
struct ItemClicked {
    index: usize,
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
        ctx.send_event(event::Redraw);
    }

    fn on_event(&mut self, event: &dyn Any, _this: Id, _ctx: &mut Context) {
        if let Some(SetIndex(index)) = event.downcast_ref() {
            self.index = *index;
        }
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        match event {
            MouseEvent::Enter => {
                self.state = 1;
                ctx.set_graphic(this, self.style.hover.clone());
                ctx.send_event(event::Redraw);
            }
            MouseEvent::Exit => {
                self.state = 0;
                if self.focus {
                    ctx.set_graphic(this, self.style.focus.clone());
                } else {
                    ctx.set_graphic(this, self.style.normal.clone());
                }
                ctx.send_event(event::Redraw);
            }
            MouseEvent::Down => {
                self.state = 2;
                ctx.set_graphic(this, self.style.pressed.clone());
                ctx.send_event(event::Redraw);
            }
            MouseEvent::Up => {
                if self.state == 2 {
                    ctx.send_event_to(self.menu, ItemClicked { index: self.index });
                }
                self.state = 1;
                ctx.set_graphic(this, self.style.hover.clone());
                ctx.send_event(event::Redraw);
            }
            MouseEvent::Moved { .. } => {}
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
            ctx.send_event(event::Redraw);
        }
    }
}

pub struct Blocker {
    pub menu: Id,
}
impl Behaviour for Blocker {
    fn on_mouse_event(&mut self, event: MouseEvent, _this: Id, ctx: &mut Context) -> bool {
        if let MouseEvent::Down = event {
            ctx.send_event_to(self.menu, CloseMenu);
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
    margins: [f32; 4],
    spacing: f32,
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
            spacing: 0.0,
            margins: [1.0, 1.0, 1.0, 1.0],
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
    T: 'static + Clone,
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

    fn compute_min_size(&mut self, this: Id, ctx: &mut MinSizeContext) {
        let children = ctx.get_children(this);
        if children.is_empty() {
            ctx.set_this_min_size([
                self.margins[0] + self.margins[2],
                self.margins[1] + self.margins[3],
            ]);
        } else {
            let mut min_width: f32 = 0.0;
            let mut min_height: f32 =
                self.margins[1] + self.margins[3] + (children.len() - 1) as f32 * self.spacing;
            for child in children {
                let [width, height] = ctx.get_layouting(child).get_min_size();
                min_width = min_width.max(width);
                min_height += height;
            }
            ctx.set_this_min_size([min_width + self.margins[0] + self.margins[2], min_height]);
        }
    }

    fn update_layouts(&mut self, this: Id, ctx: &mut LayoutContext) {
        let children = ctx.get_children(this);
        if children.is_empty() {
            return;
        }
        let mut reserved_height = self.spacing * (children.len() - 1) as f32;
        let mut max_weight = 0.0;
        for child in children {
            let rect = ctx.get_layouting(child);
            reserved_height += rect.get_min_size()[1];
            if rect.is_expand_y() {
                max_weight += rect.ratio_y;
            }
        }
        let rect = ctx.get_layouting(this);
        let height = rect.get_height() - self.margins[1] - self.margins[3];
        let rect = *rect.get_rect();
        let left = rect[0] + self.margins[0];
        let right = rect[2] - self.margins[2];
        let mut y = rect[1] + self.margins[1];
        let free_height = height - reserved_height;
        if free_height <= 0.0 || max_weight == 0.0 {
            for child in ctx.get_children(this) {
                let height = ctx.get_min_size(child)[1];
                ctx.set_designed_rect(child, [left, y, right, y + height]);
                y += self.spacing + height;
            }
        } else {
            for child in ctx.get_children(this) {
                let rect = ctx.get_layouting(child);
                if rect.is_expand_y() {
                    // FIXME: this implementation imply that rect with same ratio,
                    // may not have the same size when expanded
                    let height = rect.get_min_size()[1] + free_height * rect.ratio_y / max_weight;
                    ctx.set_designed_rect(child, [left, y, right, y + height]);
                    y += self.spacing + height;
                } else {
                    let height = rect.get_min_size()[1];
                    ctx.set_designed_rect(child, [left, y, right, y + height]);
                    y += self.spacing + height;
                }
            }
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
        ctx.send_event(event::Redraw);
    }

    fn on_event(&mut self, event: &dyn Any, this: Id, ctx: &mut Context) {
        if let Some(x) = event.downcast_ref::<ItemClicked>() {
            self.selected = Some(x.index);
            (self.on_select)(self.itens[x.index].clone(), this, ctx);
            self.opened = false;
        // ctx.send_event_to(self.menu.clo);
        } else if event.is::<MenuClosed>() {
            self.opened = false;
        }
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        match event {
            MouseEvent::Enter => {
                self.state = 1;
                ctx.set_graphic(this, self.style.hover.clone());
                ctx.send_event(event::Redraw);
            }
            MouseEvent::Exit => {
                self.state = 0;
                if self.focus {
                    ctx.set_graphic(this, self.style.focus.clone());
                } else {
                    ctx.set_graphic(this, self.style.normal.clone());
                }
                ctx.send_event(event::Redraw);
            }
            MouseEvent::Down => {
                self.state = 2;
                ctx.set_graphic(this, self.style.pressed.clone());
                ctx.send_event(event::Redraw);
            }
            MouseEvent::Up => {
                if self.state == 2 {
                    if !self.opened {
                        self.opened = true;
                        ctx.active(self.menu);
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
                ctx.send_event(event::Redraw);
            }
            MouseEvent::Moved { .. } => {}
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
            ctx.send_event(event::Redraw);
        }
    }
}
