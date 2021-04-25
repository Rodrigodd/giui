use ab_glyph::{Font, FontVec};

#[derive(Clone, Copy, Debug)]
pub struct FontId {
    index: u32,
}
impl FontId {
    /// Create a new FontId from a index. This is discouraged, use the FontId
    /// return by [Fonts::Add](crate::font::Fonts::add) instead
    pub fn new(index: u32) -> Self {
        Self { index }
    }
    /// Get the index of the font in the Fonts inner storage.
    pub fn index(&self) -> usize {
        self.index as usize
    }
}

#[derive(Default)]
pub struct Fonts {
    fonts: Vec<FontVec>,
}
impl Fonts {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add(&mut self, font: FontVec) -> FontId {
        self.fonts.push(font);
        FontId {
            index: (self.fonts.len() - 1) as u32,
        }
    }

    pub fn get(&self, id: FontId) -> Option<&FontVec> {
        self.fonts.get(id.index())
    }

    pub fn as_slice(&self) -> &[impl Font] {
        &self.fonts
    }
}
