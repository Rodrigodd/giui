use crate::{
    layouts::VBoxLayout,
    style::MenuStyle,
    widgets::{Blocker, CloseMenu, ItemClicked, Menu, MenuBehaviour},
    Behaviour, Context, Id, InputFlags, MouseButton, MouseEvent,
};

use std::{any::Any, rc::Rc};

struct Repos;

pub struct ContextMenu {
    menu: Rc<Menu>,
    open: Option<Id>,
    mouse_pos: [f32; 2],
    style: Rc<MenuStyle>,
    blocker: Option<Id>,
}
impl ContextMenu {
    pub fn new(style: Rc<MenuStyle>, menu: Rc<Menu>) -> Self {
        Self {
            menu,
            open: None,
            mouse_pos: [0.0, 0.0],
            style,
            blocker: None,
        }
    }
}
impl Behaviour for ContextMenu {
    fn on_start(&mut self, this: Id, ctx: &mut Context) {
        let blocker = ctx
            .create_control()
            .behaviour(Blocker::new(move |_, ctx| {
                ctx.send_event_to(this, CloseMenu)
            }))
            .active(false)
            .build();
        self.blocker = Some(blocker);
    }

    fn on_event(&mut self, event: Box<dyn Any>, _this: Id, ctx: &mut Context) {
        if event.is::<ItemClicked>() || event.is::<CloseMenu>() {
            if let Some(menu) = self.open.take() {
                ctx.remove(menu);
                ctx.deactive(self.blocker.unwrap());
            }
        } else if event.is::<Repos>() {
            if let Some(menu) = self.open {
                let desktop = *ctx.get_rect(ctx.get_parent(menu).unwrap());

                let menu_rect = ctx.get_rect(menu);
                let width = menu_rect[2] - menu_rect[0];
                let height = menu_rect[3] - menu_rect[1];

                let mut margins = *ctx.get_margins(menu);
                if menu_rect[2] > desktop[2] && menu_rect[0] - width > 0.0 {
                    margins[0] -= width;
                    margins[2] = margins[0];
                }
                if menu_rect[3] > desktop[3] && menu_rect[1] - height > 0.0 {
                    margins[1] -= height;
                    margins[3] = margins[1];
                }
                ctx.set_margins(menu, margins);
            }
        }
    }

    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) {
        use MouseButton::*;
        match event {
            MouseEvent::Up(Right) => {
                if self.open.is_none() {
                    let [x, y] = self.mouse_pos;

                    let menu = ctx
                        .create_control()
                        .anchors([0.0, 0.0, 0.0, 0.0])
                        .margins([x, y, x, y])
                        .behaviour(MenuBehaviour::new(
                            self.menu.clone(),
                            self.style.clone(),
                            this,
                        ))
                        .graphic(self.style.button.normal.clone())
                        .layout(VBoxLayout::new(0.0, [0.0, 0.0, 0.0, 0.0], -1))
                        .build();
                    self.open = Some(menu);
                    // when 'this' receive the event 'Repos', the 'menu' will already have its size defined.
                    ctx.send_event_to(this, Repos);
                    ctx.move_to_front(self.blocker.unwrap());
                    ctx.active(self.blocker.unwrap());
                }
            }
            MouseEvent::Moved { x, y } => {
                self.mouse_pos = [x, y];
            }
            _ => {}
        }
    }
}
