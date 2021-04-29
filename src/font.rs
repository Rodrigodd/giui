use ab_glyph::{Font as AbFont, FontVec};

#[derive(Clone, Copy, Debug)]
pub struct FontId {
    index: u32,
}
impl FontId {
    /// Create a new FontId from a index. This is discouraged, use the FontId
    /// return by [Fonts::Add](crate::font::Fonts::add) instead.
    pub fn new(index: u32) -> Self {
        Self { index }
    }
    /// Get the index of the font in the Fonts inner storage.
    pub fn index(&self) -> usize {
        self.index as usize
    }
}

pub struct Font {
    id: FontId,
    inner: FontVec,
    pub fallback: Option<FontId>,
}
impl Font {
    pub fn new(font: FontVec) -> Self {
        Self {
            id: FontId { index: u32::max_value() },
            inner: font,
            fallback: None
        }
    }

    pub fn with_fallback(mut self, fallback: FontId) -> Self {
        self.fallback = Some(fallback);
        self
    }

    pub fn id(&self) -> FontId {
        self.id
    }
}
impl From<FontVec> for Font {
    fn from(font: FontVec) -> Self {
        Self::new(font)
    }
}
impl AbFont for Font {
    fn units_per_em(&self) -> Option<f32> {
        self.inner.units_per_em()
    }

    fn ascent_unscaled(&self) -> f32 {
        self.inner.ascent_unscaled()
    }

    fn descent_unscaled(&self) -> f32 {
        self.inner.descent_unscaled()
    }

    fn line_gap_unscaled(&self) -> f32 {
        self.inner.line_gap_unscaled()
    }

    fn glyph_id(&self, c: char) -> ab_glyph::GlyphId {
        self.inner.glyph_id(c)
    }

    fn h_advance_unscaled(&self, id: ab_glyph::GlyphId) -> f32 {
        self.inner.h_advance_unscaled(id)
    }

    fn h_side_bearing_unscaled(&self, id: ab_glyph::GlyphId) -> f32 {
        self.inner.h_side_bearing_unscaled(id)
    }

    fn v_advance_unscaled(&self, id: ab_glyph::GlyphId) -> f32 {
        self.inner.v_advance_unscaled(id)
    }

    fn v_side_bearing_unscaled(&self, id: ab_glyph::GlyphId) -> f32 {
        self.inner.v_side_bearing_unscaled(id)
    }

    fn kern_unscaled(&self, first: ab_glyph::GlyphId, second: ab_glyph::GlyphId) -> f32 {
        self.inner.kern_unscaled(first, second)
    }

    fn outline(&self, id: ab_glyph::GlyphId) -> Option<ab_glyph::Outline> {
        self.inner.outline(id)
    }

    fn glyph_count(&self) -> usize {
        self.inner.glyph_count()
    }

    fn codepoint_ids(&self) -> ab_glyph::CodepointIdIter<'_> {
        self.inner.codepoint_ids()
    }
}

#[derive(Default)]
pub struct Fonts {
    fonts: Vec<Font>,
}
impl Fonts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, mut font: Font) -> FontId {
        let id = FontId {
            index: self.fonts.len() as u32,
        };
        font.id = id;
        self.fonts.push(font);
        id
    }

    pub fn get(&self, id: FontId) -> Option<&Font> {
        self.fonts.get(id.index())
    }

    pub fn as_slice(&self) -> &[Font] {
        &self.fonts
    }
}
