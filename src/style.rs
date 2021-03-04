use crate::graphics::Graphic;

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

#[derive(Clone)]
pub struct MenuStyle {
    pub button: ButtonStyle,
    pub separator: Graphic,
    pub arrow: Graphic,
}
