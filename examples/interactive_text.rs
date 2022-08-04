mod common;
use common::{GiuiEventLoop, MyFonts};

use giui::{
    graphics::{Graphic, Text, TextStyle, Texture},
    layouts::MarginLayout,
    text::{Span, SpannedString},
    widgets::InteractiveText,
    Color, Context, Gui, Id, MouseEvent, MouseInfo,
};
use sprite_render::SpriteRender;
use winit::event_loop::EventLoopProxy;

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
        _render: &mut dyn SpriteRender,
        fonts: MyFonts,
        _event_loop: EventLoopProxy<()>,
    ) -> Self {
        let pangram = "This is a text where you can click here.\nAnd has two lines.";

        let mut spanned_text = SpannedString::from_string(
            pangram.to_string(),
            TextStyle {
                color: Color::BLACK,
                font_id: fonts.notosans,
                font_size: 24.0,
            },
        );
        let click_here = find(pangram, "click here");
        let two_lines = find(pangram, "two lines.");
        let mut click_span =
            spanned_text.add_span(click_here.clone(), Span::Color([0, 0, 255, 255].into()));
        let mut two_span =
            spanned_text.add_span(two_lines.clone(), Span::Color([0, 0, 255, 255].into()));
        let text = Text::from_spanned_string(spanned_text, (0, 0));

        // populate the gui with controls.
        gui.create_control()
            .graphic(Texture::new(fonts.white_texture, [0.0, 0.0, 1.0, 1.0]))
            .child(gui, move |cb, _| {
                cb.graphic(text).behaviour(InteractiveText::new(vec![
                    (
                        click_here.clone(),
                        Box::new(move |mouse: MouseInfo, this: Id, ctx: &mut Context| {
                            let text = match ctx.get_graphic_mut(this) {
                                Graphic::Text(x) => x,
                                _ => return,
                            };
                            match mouse.event {
                                MouseEvent::Enter => {
                                    click_span = text.add_span(
                                        click_here.clone(),
                                        Span::Underline(Some([0, 0, 255, 255].into())),
                                    );
                                }
                                MouseEvent::Exit => {
                                    text.remove_span(click_span);
                                }
                                _ if mouse.click() => {
                                    println!("click!!");
                                }
                                _ => {}
                            }
                        }),
                    ),
                    (
                        two_lines.clone(),
                        Box::new(move |mouse: MouseInfo, this: Id, ctx: &mut Context| {
                            let text = match ctx.get_graphic_mut(this) {
                                Graphic::Text(x) => x,
                                _ => return,
                            };
                            match mouse.event {
                                MouseEvent::Enter => {
                                    two_span = text.add_span(
                                        two_lines.clone(),
                                        Span::Underline(Some([0, 0, 255, 255].into())),
                                    );
                                }
                                MouseEvent::Exit => {
                                    text.remove_span(two_span);
                                }
                                _ => {}
                            }
                        }),
                    ),
                ]))
            })
            .layout(MarginLayout::default())
            .build(gui);
        App
    }
}
