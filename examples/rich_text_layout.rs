mod common;
use common::{GiuiEventLoop, MyFonts};

use giui::{
    graphics::{Text, TextStyle, Texture},
    layouts::{FitGraphic, MarginLayout},
    text::{Span, SpannedString},
    Color, Gui, RectFill,
};
use sprite_render::GLSpriteRender;
use winit::event_loop::EventLoop;

fn main() {
    common::run::<(), App>(400, 200);
}

fn find(s: &str, subs: &str) -> std::ops::Range<usize> {
    let index = s.find(subs).unwrap();
    index..(index + subs.len())
}

struct App;
impl GiuiEventLoop<()> for App {
    fn init(
        gui: &mut Gui,
        _render: &mut GLSpriteRender,
        fonts: MyFonts,
        _event_loop: &EventLoop<()>,
    ) -> Self {
        let pangram = "\
In A Large New Kingdom,
Brawny Gods In Crazy
Pyjamas Exchange Xerxes'
Huge Mob Of Dull Sheep For A Unique
Zebra When Quick Red Vixens
Jump Over The Yacht.";

        let mut spanned_text = SpannedString::from_string(
            pangram.to_string(),
            TextStyle {
                color: Color::BLACK,
                font_id: fonts.notosans,
                font_size: 24.0,
            },
        );
        spanned_text.add_span(find(pangram, "Large"), Span::FontSize(36.0));
        spanned_text.add_span(find(pangram, "Gods"), Span::FontSize(40.0));
        spanned_text.add_span(find(pangram, "Xerxes'"), Span::FontSize(18.0));
        spanned_text.add_span(find(pangram, "Huge"), Span::FontSize(38.0));
        spanned_text.add_span(find(pangram, "Quick"), Span::FontSize(10.0));

        spanned_text.add_span(
            find(pangram, "A Unique\nZebra"),
            Span::Selection {
                fg: None,
                bg: [0, 255, 0, 255].into(),
            },
        );
        spanned_text.add_span(
            find(pangram, "When Quick Red"),
            Span::Selection {
                fg: Some([0, 0, 255, 255].into()),
                bg: [255, 0, 0, 255].into(),
            },
        );

        spanned_text.add_span(
            find(pangram, "ique\nZebra Whe"),
            Span::Color([255, 255, 0, 255].into()),
        );
        let text = Text::from_spanned_string(spanned_text, (0, 0));

        // populate the gui with controls.
        gui.create_control()
            .graphic(Texture::new(fonts.white_texture, [0.0, 0.0, 1.0, 1.0]))
            .child(gui, move |cb, _| cb.graphic(text).layout(FitGraphic))
            .layout(MarginLayout::default())
            .fill_x(RectFill::ShrinkCenter)
            .fill_y(RectFill::ShrinkCenter)
            .build(gui);
        App
    }
}
