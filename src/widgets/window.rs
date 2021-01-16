use winit::window::CursorIcon;

use crate::{event, Behaviour, Context, Id, MouseButton, MouseEvent};

const LEFT: u8 = 0x1;
const RIGHT: u8 = 0x2;
const TOP: u8 = 0x4;
const BOTTOM: u8 = 0x8;
const LEFT_TOP: u8 = LEFT | TOP;
const RIGHT_BOTTOM: u8 = RIGHT | BOTTOM;
const TOP_RIGHT: u8 = TOP | RIGHT;
const BOTTOM_LEFT: u8 = BOTTOM | LEFT;

#[derive(Default)]
pub struct Window {
    state: u8,
    dragging: bool,
    start_dragging: [f32; 2],
    start_margins: [f32; 4],
    mouse_pos: [f32; 2],
}
impl Window {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Behaviour for Window {
    fn on_mouse_event(&mut self, event: MouseEvent, this: Id, ctx: &mut Context) -> bool {
        use MouseButton::*;
        match event {
            MouseEvent::Enter => {}
            MouseEvent::Exit => {
                ctx.set_cursor(CursorIcon::Default);
            }
            MouseEvent::Down(Left) => {
                self.dragging = true;
                ctx.send_event(event::LockOver);
                let mut margins = *ctx.get_margins(this);
                let min_size = ctx.get_min_size(this);
                if margins[2] - margins[0] < min_size[0] {
                    margins[2] = margins[0] + min_size[0];
                }
                if margins[3] - margins[1] < min_size[1] {
                    margins[3] = margins[1] + min_size[1];
                }
                self.start_dragging = self.mouse_pos;
                self.start_margins = margins;
            }
            MouseEvent::Up(Left) => {
                self.dragging = false;
                ctx.send_event(event::UnlockOver);
            }
            MouseEvent::Moved { mut x, mut y } => {
                if !self.dragging {
                    let rect = *ctx.get_rect(this);
                    self.state = 0;
                    if self.mouse_pos[0] - rect[0] < 5.0 {
                        self.state |= LEFT;
                        if self.mouse_pos[1] - rect[1] < 10.0 {
                            self.state |= TOP;
                        } else if rect[3] - self.mouse_pos[1] < 10.0 {
                            self.state |= BOTTOM;
                        }
                    } else if rect[2] - self.mouse_pos[0] < 5.0 {
                        self.state |= RIGHT;
                        if self.mouse_pos[1] - rect[1] < 10.0 {
                            self.state |= TOP;
                        } else if rect[3] - self.mouse_pos[1] < 10.0 {
                            self.state |= BOTTOM;
                        }
                    }

                    if self.mouse_pos[1] - rect[1] < 5.0 {
                        self.state |= TOP;
                        if self.mouse_pos[0] - rect[0] < 10.0 {
                            self.state |= LEFT;
                        } else if rect[2] - self.mouse_pos[0] < 10.0 {
                            self.state |= RIGHT;
                        }
                    } else if rect[3] - self.mouse_pos[1] < 5.0 {
                        self.state |= BOTTOM;
                        if self.mouse_pos[0] - rect[0] < 10.0 {
                            self.state |= LEFT;
                        } else if rect[2] - self.mouse_pos[0] < 10.0 {
                            self.state |= RIGHT;
                        }
                    }

                    match self.state {
                        LEFT => ctx.set_cursor(CursorIcon::WResize),
                        RIGHT => ctx.set_cursor(CursorIcon::EResize),
                        TOP => ctx.set_cursor(CursorIcon::NResize),
                        BOTTOM => ctx.set_cursor(CursorIcon::SResize),

                        LEFT_TOP => ctx.set_cursor(CursorIcon::NwResize),
                        RIGHT_BOTTOM => ctx.set_cursor(CursorIcon::SeResize),
                        TOP_RIGHT => ctx.set_cursor(CursorIcon::NeResize),
                        BOTTOM_LEFT => ctx.set_cursor(CursorIcon::SwResize),

                        _ => ctx.set_cursor(CursorIcon::Default),
                    }
                } else {
                    let parent = ctx
                        .get_parent(this)
                        .expect("A window cannot be the root control");

                    // ensure that the window cannot be easily drag out of reach
                    let desktop = *ctx.get_rect(parent);
                    if x < desktop[0] {
                        x = desktop[0];
                    } else if x > desktop[2] {
                        x = desktop[2];
                    }
                    if y < desktop[1] {
                        y = desktop[1];
                    } else if y > desktop[3] {
                        y = desktop[3];
                    }
                    let delta = [x - self.start_dragging[0], y - self.start_dragging[1]];
                    let mut margins = self.start_margins;
                    let min_size = ctx.get_min_size(this);
                    if self.state != 0 {
                        if (self.state & LEFT) != 0 {
                            margins[0] += delta[0];
                        }
                        if (self.state & TOP) != 0 {
                            margins[1] += delta[1];
                        }
                        if (self.state & RIGHT) != 0 {
                            margins[2] += delta[0];
                        }
                        if (self.state & BOTTOM) != 0 {
                            margins[3] += delta[1];
                        }
                        if margins[2] - margins[0] < min_size[0] {
                            if (self.state & LEFT) != 0 {
                                margins[0] = margins[2] - min_size[0];
                            } else {
                                margins[2] = margins[0] + min_size[0];
                            }
                        }
                        if margins[3] - margins[1] < min_size[1] {
                            if (self.state & TOP) != 0 {
                                margins[1] = margins[3] - min_size[1];
                            } else {
                                margins[3] = margins[1] + min_size[1];
                            }
                        }
                        ctx.set_margins(this, margins);
                    } else {
                        ctx.set_margins(
                            this,
                            [
                                margins[0] + delta[0],
                                margins[1] + delta[1],
                                margins[2] + delta[0],
                                margins[3] + delta[1],
                            ],
                        );
                    }
                }
                self.mouse_pos = [x, y];
            }
            MouseEvent::Up(_) => {}
            MouseEvent::Down(_) => {}
        }
        true
    }
}
