use crate::graphics::{Graphic, TextStyle};
use crate::Color;

#[derive(Clone, Debug, LoadStyle)]
#[crui(crate = "crate")]
pub struct OnFocusStyle {
    pub normal: Graphic,
    pub focus: Graphic,
}

#[derive(Clone, Debug, LoadStyle)]
#[crui(crate = "crate")]
pub struct ButtonStyle {
    pub normal: Graphic,
    pub hover: Graphic,
    pub pressed: Graphic,
    pub focus: Graphic,
}

#[derive(Clone, Debug, LoadStyle)]
#[crui(crate = "crate")]
pub struct TextFieldStyle {
    pub background: OnFocusStyle,
    pub caret_color: Color,
    pub selection_color: Color,
}

#[derive(Clone, Debug, LoadStyle)]
#[crui(crate = "crate")]
pub struct TabStyle {
    pub unselected: Graphic,
    pub hover: Graphic,
    pub pressed: Graphic,
    pub selected: Graphic,
}

#[derive(Clone, Debug, LoadStyle)]
#[crui(crate = "crate")]
pub struct MenuStyle {
    pub button: ButtonStyle,
    pub separator: Graphic,
    pub arrow: Graphic,
    pub text: TextStyle,
}
