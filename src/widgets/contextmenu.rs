use std::any::Any;

use crate::{
    widgets::{ItemClicked, ShowMenu},
    Behaviour, Context, Id, MouseButton, MouseEvent,
};

struct Repos;

pub struct ContextMenu {
    itens: Vec<(String, Box<dyn Fn(Id, &mut Context)>)>,
    menu: Id,
    mouse_pos: [f32; 2],
}
impl ContextMenu {
    pub fn new(itens: Vec<(String, Box<dyn Fn(Id, &mut Context)>)>, menu: Id) -> Self {
        Self {
            itens,
            menu,
            mouse_pos: [0.0, 0.0],
        }
    }
}
impl Behaviour for ContextMenu {
    fn on_event(&mut self, event: &dyn Any, this: Id, ctx: &mut Context) {
        if let Some(ItemClicked { index }) = event.downcast_ref() {
            (self.itens[*index].1)(this, ctx);
        } else if event.is::<Repos>() {
            let desktop = *ctx.get_rect(
                ctx.get_parent(self.menu)
                    .expect("Menu cannot be the root control"),
            );

            let menu_rect = ctx.get_rect(self.menu);
            let width = menu_rect[2] - menu_rect[0];
            let height = menu_rect[3] - menu_rect[1];

            let mut margins = *ctx.get_margins(self.menu);
            if menu_rect[2] > desktop[2] && menu_rect[0] - width > 0.0 {
                margins[0] -= width;
                margins[2] = margins[0];
            }
            if menu_rect[3] > desktop[3] && menu_rect[1] - height > 0.0 {
                margins[1] -= height;
                margins[3] = margins[1];
            }
            ctx.set_margins(self.menu, margins);
        }
    }

    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        use MouseButton::*;
        match event {
            MouseEvent::Up(Right) => {
                let [x, y] = self.mouse_pos;
                ctx.set_anchors(self.menu, [0.0, 0.0, 0.0, 0.0]);
                ctx.set_margins(self.menu, [x, y, x + 20.0, y]);
                ctx.send_event_to(
                    self.menu,
                    ShowMenu(
                        this,
                        None,
                        self.itens.iter().map(|(a, _)| a.clone()).collect(),
                    ),
                );
                // when 'this' receive the event 'Repos', the 'menu' will already have its size defined.
                ctx.send_event_to(this, Repos);
                true
            }
            MouseEvent::Moved { x, y } => {
                self.mouse_pos = [x, y];
                false
            }
            _ => false,
        }
    }
}
