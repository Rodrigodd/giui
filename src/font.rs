use ab_glyph::{Font as AbFont, FontVec};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontId {
    index: u32,
}
impl Default for FontId {
    fn default() -> Self {
        Self { index: 0 }
    }
}
impl FontId {
    /// Create a new FontId from a index. This is discouraged, use the FontId
    /// return by [Fonts::add](crate::font::Fonts::add) instead.
    pub fn new(index: u32) -> Self {
        Self { index }
    }
    /// Get the index of the font in the Fonts inner storage.
    pub fn index(&self) -> usize {
        self.index as usize
    }
}

pub struct Font {
    // TODO: keeping a FontVec and data is redundant.
    pub data: Vec<u8>,
    id: FontId,
    inner: FontVec,
    pub fallback: Option<FontId>,
}
impl Font {
    pub fn new(data: &[u8]) -> Self {
        let inner = FontVec::try_from_vec(data.to_vec()).unwrap();
        Self {
            data: data.to_vec(),
            id: FontId {
                index: u32::max_value(),
            },
            inner,
            fallback: None,
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

    fn glyph_raster_image(
        &self,
        glyph_id: ab_glyph::GlyphId,
        pixel_size: u16,
    ) -> Option<ab_glyph::GlyphImage<'_>> {
        self.inner.glyph_raster_image(glyph_id, pixel_size)
    }
}

pub struct Fonts {
    fonts: Vec<Font>,
}

impl Default for Fonts {
    fn default() -> Self {
        Self::new()
    }
}
impl Fonts {
    pub fn new() -> Self {
        Self { fonts: Vec::new() }
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
