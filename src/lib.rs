#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate crui_derive;
#[doc(hidden)]
pub use crui_derive::*;

pub mod font;
pub mod text;
mod text_layout;
mod util;

mod unicode;

mod color;
mod context;
mod control;
pub mod graphics;
mod gui;
pub mod layouts;
mod rect;
pub mod render;
pub mod style;
pub mod widgets;

pub mod style_loader;

pub use color::Color;
pub use context::*;
pub use control::*;
pub use gui::*;
pub use rect::*;
pub use render::GuiRender;
