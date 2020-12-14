use crate::render::Graphic;

#[derive(Clone)]
pub struct OnFocusStyle {
    pub normal: Graphic,
    pub focus: Graphic,
}

#[derive(Clone)]
pub struct ButtonStyle {
    pub normal: Graphic,
    pub hover: Graphic,
    pub pressed: Graphic,
    pub focus: Graphic,
}

#[derive(Clone)]
pub struct TabStyle {
    pub unselected: Graphic,
    pub hover: Graphic,
    pub pressed: Graphic,
    pub selected: Graphic,
}

mod button;
pub use button::*;

mod slider;
pub use slider::*;

mod toggle;
pub use toggle::*;

mod tab;
pub use tab::*;

mod hoverable;
pub use hoverable::*;

mod scrollview;
pub use scrollview::*;

mod textfield;
pub use textfield::*;

mod dropdown;
pub use dropdown::*;
