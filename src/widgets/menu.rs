use crate::{
    graphics::Text,
    layouts::{FitText, HBoxLayout, MarginLayout, VBoxLayout},
    style::MenuStyle,
    widgets::CloseMenu,
    Behaviour, Context, Id, InputFlags, MouseButton, MouseEvent, RectFill,
};

use std::{any::Any, rc::Rc};

pub enum Item {
    Separator,
    Button(String, Box<dyn Fn(Id, &mut Context)>),
    SubMenu(Rc<Menu>),
}

pub struct Menu {
    name: String,
    itens: Vec<Item>,
}
impl Menu {
    pub fn new(name: String, itens: Vec<Item>) -> Self {
        Self { name, itens }
    }
}

pub struct ItemClicked;

pub struct MenuBehaviour {
    menu: Rc<Menu>,
    over: Option<usize>,
    is_over: bool,
    open: Option<Id>,
    click: bool,
    style: Rc<MenuStyle>,
    owner: Id,
}
impl MenuBehaviour {
    pub fn new(menu: Rc<Menu>, style: Rc<MenuStyle>, owner: Id) -> Self {
        Self {
            menu,
            over: None,
            is_over: false,
            open: None,
            click: false,
            style,
            owner,
        }
    }

    fn close_menu(&mut self, ctx: &mut Context) {
        if let Some(open) = self.open.take() {
            ctx.remove(open);
        }
    }

    fn open_menu(&mut self, i: usize, this: Id, ctx: &mut Context) {
        self.close_menu(ctx);
        match &self.menu.itens[i] {
            Item::Separator => {}
            Item::Button(_, _) => {}
            Item::SubMenu(menu) => {
                let child = ctx.get_children(this)[i];
                let rect = *ctx.get_rect(child);
                let x = rect[2];
                let y = rect[1];

                let menu = ctx
                    .create_control()
                    .with_anchors([0.0, 0.0, 0.0, 0.0])
                    .with_margins([x, y, x, y])
                    .with_behaviour(MenuBehaviour::new(menu.clone(), self.style.clone(), this))
                    .with_graphic(self.style.button.normal.clone())
                    .with_layout(VBoxLayout::new(0.0, [0.0, 0.0, 0.0, 0.0], -1))
                    .build();
                self.open = Some(menu);
            }
        }
    }
}
impl Behaviour for MenuBehaviour {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        for item in self.menu.itens.iter() {
            match item {
                Item::Separator => {
                    let item = ctx
                        .create_control()
                        .with_parent(this)
                        .with_min_size([0.0, 5.0])
                        .build();
                    let _dash = ctx
                        .create_control()
                        .with_graphic(self.style.separator.clone())
                        .with_parent(item)
                        .with_margins([8.0, 2.0, -8.0, -2.0])
                        .build();
                }
                Item::Button(text, _) => {
                    let item = ctx
                        .create_control()
                        .with_parent(this)
                        .with_layout(MarginLayout::new([18.0, 2.0, 18.0, 2.0]))
                        .build();
                    let _text = ctx
                        .create_control()
                        .with_parent(item)
                        .with_graphic(Text::new([0, 0, 0, 255], text.clone(), 16.0, (-1, 0)).into())
                        .with_layout(FitText)
                        .build();
                }
                Item::SubMenu(menu) => {
                    let item = ctx
                        .create_control()
                        .with_parent(this)
                        .with_layout(HBoxLayout::new(0.0, [18.0, 2.0, 2.0, 2.0], -1))
                        .build();
                    let _text = ctx
                        .create_control()
                        .with_parent(item)
                        .with_graphic(
                            Text::new([0, 0, 0, 255], menu.name.clone(), 16.0, (-1, 0)).into(),
                        )
                        .with_layout(FitText)
                        .with_expand_x(true)
                        .build();
                    let _arrow = ctx
                        .create_control()
                        .with_min_size([16.0, 16.0])
                        .with_fill_y(RectFill::ShrinkCenter)
                        .with_graphic(self.style.arrow.clone())
                        .with_parent(item)
                        .build();
                }
            }
        }
    }

    fn on_event(&mut self, event: Box<dyn Any>, _: Id, ctx: &mut Context) {
        if event.is::<ItemClicked>() {
            self.close_menu(ctx);
            ctx.send_event_to(self.owner, ItemClicked);
        }
    }

    fn on_deactive(&mut self, _this: Id, ctx: &mut Context) {
        self.close_menu(ctx);
    }

    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) {
        use MouseButton::*;
        match event {
            MouseEvent::Down(Left) => {
                self.click = true;
            }
            MouseEvent::Up(Left) => {
                if self.is_over && self.click {
                    let i = self.over.unwrap();
                    match &self.menu.itens[i] {
                        Item::Separator => {}
                        Item::Button(_, call) => {
                            (call)(this, ctx);
                            ctx.send_event_to(self.owner, ItemClicked);
                        }
                        Item::SubMenu(_) => {}
                    }
                }
            }
            MouseEvent::Moved { x, y } => {
                let children = ctx.get_children(this);
                self.is_over = false;
                for (i, child) in children.iter().enumerate().rev() {
                    let rect = *ctx.get_rect(*child);
                    if rect[0] < x && x < rect[2] && rect[1] < y && y < rect[3] {
                        if Some(i) != self.over {
                            if let Some(i) = self.over {
                                ctx.set_graphic(children[i], self.style.button.normal.clone());
                            }
                            use Item::*;
                            match self.menu.itens[i] {
                                Button(_, _) | SubMenu(_) => {
                                    ctx.set_graphic(*child, self.style.button.hover.clone());
                                }
                                Separator => {}
                            }
                            self.over = Some(i);
                            self.open_menu(i, this, ctx);
                            self.click = false;
                        }
                        self.is_over = true;
                        break;
                    }
                }
            }
            _ => {}
        }
    }
}

pub struct MenuBar {
    menus: Vec<Rc<Menu>>,
    over: Option<usize>,
    is_over: bool,
    open: Option<Id>,
    style: Rc<MenuStyle>,
    blocker: Id,
}
impl MenuBar {
    pub fn new(style: Rc<MenuStyle>, blocker: Id, menus: Vec<Rc<Menu>>) -> Self {
        Self {
            menus,
            over: None,
            open: None,
            is_over: false,
            style,
            blocker,
        }
    }

    fn close_menu(&mut self, ctx: &mut Context) {
        if let Some(open) = self.open.take() {
            ctx.remove(open);
        }
        ctx.deactive(self.blocker);
    }

    fn open_menu(&mut self, i: usize, this: Id, ctx: &mut Context) {
        self.close_menu(ctx);
        ctx.active(self.blocker);
        let child = ctx.get_children(this)[i];
        let rect = *ctx.get_rect(child);
        let x = rect[0];
        let y = rect[3];

        let menu = ctx
            .create_control()
            .with_anchors([0.0, 0.0, 0.0, 0.0])
            .with_margins([x, y, x, y])
            .with_behaviour(MenuBehaviour::new(
                self.menus[i].clone(),
                self.style.clone(),
                this,
            ))
            .with_graphic(self.style.button.normal.clone())
            .with_layout(VBoxLayout::new(0.0, [0.0, 0.0, 0.0, 0.0], -1))
            .build();
        self.open = Some(menu);
    }
}
impl Behaviour for MenuBar {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        for menu in self.menus.iter() {
            let item = ctx
                .create_control()
                .with_parent(this)
                .with_layout(MarginLayout::new([2.0, 2.0, 2.0, 2.0]))
                .with_graphic(self.style.button.normal.clone())
                .build();
            ctx.create_control()
                .with_parent(item)
                .with_graphic(Text::new([0, 0, 0, 255], menu.name.clone(), 16.0, (0, 0)).into())
                .with_layout(FitText)
                .build();
        }
    }

    fn on_event(&mut self, event: Box<dyn Any>, _: Id, ctx: &mut Context) {
        if event.is::<ItemClicked>() || event.is::<CloseMenu>() {
            self.close_menu(ctx);
        }
    }

    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) {
        use MouseButton::*;
        match event {
            MouseEvent::Down(Left) => {
                if self.is_over {
                    if self.open.is_none() {
                        self.open_menu(self.over.unwrap(), this, ctx);
                    } else {
                        ctx.remove(self.open.take().unwrap());
                    }
                } else if let Some(open) = self.open.take() {
                    ctx.remove(open);
                }
            }
            MouseEvent::Up(_) => {}
            MouseEvent::Moved { x, y } => {
                let children = ctx.get_children(this);
                self.is_over = false;
                for (i, child) in children.iter().enumerate().rev() {
                    let rect = *ctx.get_rect(*child);
                    if rect[0] < x && x < rect[2] && rect[1] < y && y < rect[3] {
                        if Some(i) != self.over {
                            if self.open.is_some() {
                                self.open_menu(i, this, ctx);
                            }
                            if let Some(i) = self.over {
                                ctx.set_graphic(children[i], self.style.button.normal.clone());
                            }
                            ctx.set_graphic(*child, self.style.button.hover.clone());
                            self.over = Some(i);
                        }
                        self.is_over = true;
                        break;
                    }
                }
            }
            _ => {}
        }
    }
}
