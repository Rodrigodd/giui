use crate::Color;
use crate::{font::Fonts, text::ShapeSpan, text::layout::GlyphPosition};

use ab_glyph::{point, Glyph, GlyphId};
use harfbuzz_rs::{shape as hb_shape, Face, Font as HbFont, UnicodeBuffer};

pub(crate) fn shape(fonts: &Fonts, text: &str, style: &ShapeSpan) -> Vec<GlyphPosition> {
    let bytes = &fonts.get(style.font_id).unwrap().data;
    let face = Face::from_bytes(bytes, 0);
    let font = HbFont::new(face);
    let scale = {
        let extends = font.get_font_h_extents().unwrap();
        let height = extends.ascender - extends.descender;
        style.font_size / height as f32
    };
    // let scale = style.px / ppem;
    let buffer = UnicodeBuffer::new().add_str(text);
    let output = hb_shape(&font, buffer, &[]);

    let positions = output.get_glyph_positions();
    let infos = output.get_glyph_infos();

    let mut glyphs: Vec<GlyphPosition> = Vec::with_capacity(positions.len());
    let mut x = 0.0;

    for (position, info) in positions.iter().zip(infos) {
        let gid = info.codepoint;
        let cluster = info.cluster as usize;
        let x_offset = position.x_offset as f32 * scale;
        let y_offset = position.y_offset as f32 * scale;
        let x_advance = position.x_advance as f32 * scale;

        if let Some(last) = glyphs.last_mut() {
            last.byte_range.end = cluster;
        }
        let is_whitespace = text[cluster..].chars().next().map_or(false, |x| x.is_whitespace());
        glyphs.push(GlyphPosition {
            glyph: Glyph {
                id: GlyphId(gid as u16),
                scale: style.font_size.into(),
                position: point(x + x_offset, y_offset),
            },
            font_id: style.font_id,
            byte_range: cluster..text.len(),
            width: x_advance,
            color: Color::WHITE,
            is_whitespace,
        });
        x += x_advance;
    }

    glyphs
}
