use crate::{
    graphics::{Graphic, TextStyle},
    Color,
};

#[derive(Clone, Debug, LoadStyle)]
#[giui(crate = "crate")]
pub struct OnFocusStyle {
    pub normal: Graphic,
    pub focus: Graphic,
}

#[derive(Clone, Debug, LoadStyle)]
#[giui(crate = "crate")]
pub struct ButtonStyle {
    pub normal: Graphic,
    pub hover: Graphic,
    pub pressed: Graphic,
    pub focus: Graphic,
}

#[derive(Clone, Debug, LoadStyle)]
#[giui(crate = "crate")]
pub struct SelectionColor {
    pub fg: Option<Color>,
    pub bg: Color,
}

#[derive(Clone, Debug, LoadStyle)]
#[giui(crate = "crate")]
pub struct TextFieldStyle {
    pub background: OnFocusStyle,
    pub caret_color: Color,
    pub selection_color: SelectionColor,
}

#[derive(Clone, Debug, LoadStyle)]
#[giui(crate = "crate")]
pub struct TabStyle {
    pub unselected: Graphic,
    pub hover: Graphic,
    pub pressed: Graphic,
    pub selected: Graphic,
}

#[derive(Clone, Debug, LoadStyle)]
#[giui(crate = "crate")]
pub struct MenuStyle {
    pub button: ButtonStyle,
    pub separator: Graphic,
    pub arrow: Graphic,
    pub text: TextStyle,
}
