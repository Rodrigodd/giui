#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}
impl Color {
    pub const WHITE: Color = Color::from_array([255; 4]);
    pub const BLACK: Color = Color::from_array([0, 0, 0, 255]);

    pub const fn from_u32(value: u32) -> Self {
        Self::from_array(value.to_be_bytes())
    }

    pub const fn from_array(value: [u8; 4]) -> Self {
        Self {
            r: value[0],
            g: value[1],
            b: value[2],
            a: value[3],
        }
    }

    pub const fn to_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}
impl From<[u8; 4]> for Color {
    fn from(value: [u8; 4]) -> Self {
        Self::from_array(value)
    }
}
impl From<u32> for Color {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}
