use crate::Color;
use crate::{font::Fonts, text::layout::GlyphPosition, text::ShapeSpan};

#[cfg(feature = "complex_shaping")]
pub(crate) fn shape(fonts: &Fonts, text: &str, style: &ShapeSpan) -> Vec<GlyphPosition> {
    use ab_glyph::{point, Glyph, GlyphId};
    use harfbuzz_rs::{shape as hb_shape, Face, Font as HbFont, UnicodeBuffer};

    let bytes = &fonts.get(style.font_id).unwrap().data;
    let face = Face::from_bytes(bytes, 0);
    let font = HbFont::new(face);
    let scale = {
        let extends = font.get_font_h_extents().unwrap();
        let height = extends.ascender - extends.descender;
        style.font_size / height as f32
    };

    let cleanup_text = text.replace(|x: char| x.is_control(), " ");
    debug_assert_eq!(cleanup_text.len(), text.len());
    // let scale = style.px / ppem;
    let buffer = UnicodeBuffer::new().add_str(&cleanup_text);
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

        if let Some((mut last, split)) = glyphs.split_last_mut() {
            if last.byte_range.start != cluster {
                last.byte_range.end = cluster;
                // if multiple glyphs are from the same cluster, the first one will map to the
                // entire byte range of cluster, and the others will have a empty byte_range
                for prev in split.iter_mut().rev() {
                    if !prev.byte_range.is_empty() && prev.byte_range.start == last.byte_range.start
                    {
                        last.byte_range.start = cluster;
                        prev.byte_range.end = cluster;
                        last = prev;
                    }
                }
            }
        }
        let is_whitespace = text[cluster..]
            .chars()
            .next()
            .map_or(false, |x| x.is_whitespace());
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

    if let Some((mut last, split)) = glyphs.split_last_mut() {
        let cluster = text.len();
        last.byte_range.end = cluster;
        // if multiple glyphs are from the same cluster, the first one will map to the
        // entire byte range of cluster, and the others will have a empty byte_range
        for prev in split.iter_mut().rev() {
            if !prev.byte_range.is_empty() && prev.byte_range.start == last.byte_range.start {
                last.byte_range.start = cluster;
                prev.byte_range.end = cluster;
                last = prev;
            }
        }
    }

    glyphs
}

#[cfg(not(feature = "complex_shaping"))]
pub(crate) fn shape(fonts: &Fonts, text: &str, style: &ShapeSpan) -> Vec<GlyphPosition> {
    use ab_glyph::{point, Font as _, Glyph, ScaleFont as _};
    let font = fonts
        .get(style.font_id)
        .expect("FontId is out of bounds")
        .as_scaled(style.font_size);

    let mut glyphs: Vec<GlyphPosition> = Vec::new();
    let mut x = 0.0;
    for (byte_offset, mut c) in text.char_indices() {
        if c.is_control() {
            c = ' ';
        }
        let is_whitespace = c.is_whitespace();
        let mut font = font;
        let mut glyph = font.scaled_glyph(c);
        if glyph.id.0 == 0 {
            while let Some(fallback) = font.font.fallback {
                font = fonts.get(fallback).unwrap().as_scaled(style.font_size);
                glyph = font.scaled_glyph(c);
                if glyph.id.0 != 0 {
                    break;
                }
            }
        }

        let advance = font.h_advance(glyph.id);
        if !glyphs.is_empty() {
            let last = glyphs.last_mut().unwrap();
            last.width += font.kern(last.glyph.id, glyph.id);
        }

        // if c.is_whitespace() {
        //     glyph.id = font.glyph_id('·');
        // }

        glyphs.push(GlyphPosition {
            // glyph,
            glyph: Glyph {
                id: glyph.id,
                scale: glyph.scale,
                position: point(x, 0.0),
            },
            font_id: font.font.id(),
            byte_range: byte_offset..byte_offset + c.len_utf8(),
            width: advance,
            color: Color::WHITE,
            is_whitespace,
        });

        x += advance;
    }
    glyphs
}
