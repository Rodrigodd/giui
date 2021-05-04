use super::TextLayoutStyle;
use crate::{font::Fonts, text_layout::GlyphPosition};

use ab_glyph::{point, Font, Glyph, GlyphId};
use harfbuzz_rs::{shape as hb_shape, Face, Font as HbFont, UnicodeBuffer};

pub fn shape(fonts: &Fonts, style: &TextLayoutStyle) -> Vec<GlyphPosition> {
    let ppem = fonts.get(style.font_id).unwrap().units_per_em().unwrap();
    let bytes = &fonts.get(style.font_id).unwrap().data;
    let face = Face::from_bytes(bytes, 0);
    let font = HbFont::new(face);
    let scale = {
        let extends = font.get_font_h_extents().unwrap();
        let height = extends.ascender - extends.descender;
        style.px / height as f32
    };
    // let scale = style.px / ppem;
    println!("scale: {}, ppem: {}", scale, ppem);
    let buffer = UnicodeBuffer::new().add_str(style.text);
    let output = hb_shape(&font, buffer, &[]);

    let positions = output.get_glyph_positions();
    let infos = output.get_glyph_infos();

    let mut glyphs: Vec<GlyphPosition> = Vec::with_capacity(positions.len());
    // iterate over the shaped glyphs
    for (position, info) in positions.iter().zip(infos) {
        let gid = info.codepoint;
        let cluster = info.cluster as usize;
        let x_offset = position.x_offset as f32 * scale;
        let y_offset = position.y_offset as f32 * scale;
        let x_advance = position.x_advance as f32 * scale;

        // Here you would usually draw the glyphs.
        println!(
            "gid{:04x?}={:3?} {:?},{:?}+{:?}",
            gid, cluster, x_advance, x_offset, y_offset
        );
        if let Some(last) = glyphs.last_mut() {
            last.byte_range.end = cluster;
        }
        glyphs.push(GlyphPosition {
            glyph: Glyph {
                id: GlyphId(gid as u16),
                scale: style.px.into(),
                position: point(x_offset, y_offset),
            },
            font_id: style.font_id,
            byte_range: cluster..style.text.len(),
            width: x_advance,
        })
    }

    glyphs
}
