use ab_glyph::{Font, ScaleFont};

use crate::{font::Fonts, unicode::read_utf8};

use super::{GlyphPosition, TextLayoutStyle};

pub fn shape(fonts: &Fonts, style: &TextLayoutStyle) -> Vec<GlyphPosition> {
    let font = fonts
        .get(style.font_id)
        .expect("FontId is out of bounds")
        .as_scaled(style.px);

    let mut byte_offset = 0;
    let mut glyphs: Vec<GlyphPosition> = Vec::new();
    while byte_offset < style.text.len() {
        let cur_character_offset = byte_offset;
        let c = unsafe { read_utf8(&style.text, &mut byte_offset) };
        println!("{:?}", c);
        let mut font = font;
        let mut glyph = font.scaled_glyph(c);
        if glyph.id.0 == 0 {
            while let Some(fallback) = font.font.fallback {
                font = fonts.get(fallback).unwrap().as_scaled(style.px);
                glyph = font.scaled_glyph(c);
                if glyph.id.0 != 0 {
                    break;
                }
            }
        }

        let mut advance = font.h_advance(glyph.id);
        if !glyphs.is_empty() {
            let last = glyphs.last_mut().unwrap();
            last.width += font.kern(last.glyph.id, glyph.id);
        }

        // if c.is_whitespace() {
        //     glyph.id = font.glyph_id('·');
        // }

        if c.is_control() {
            advance = 0.0;
            glyph.id = font.glyph_id(' ');
        }

        glyphs.push(GlyphPosition {
            glyph,
            font_id: font.font.id(),
            byte_range: cur_character_offset..byte_offset,
            width: advance,
            color: style.color,
        });
    }
    glyphs
}
