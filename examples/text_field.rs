mod common;
use common::MyFonts;
use std::rc::Rc;

use common::*;
use crui::{
    font::FontId,
    graphics::{Panel, Text},
    layouts::MarginLayout,
    style::{OnFocusStyle, SelectionColor, TextFieldStyle},
    widgets::{TextField, TextFieldCallback},
    BuilderContext, Color, ControlBuilder, Gui,
};
use sprite_render::{GLSpriteRender, SpriteRender};
use winit::event_loop::EventLoop;

fn main() {
    struct TextField;
    impl CruiEventLoop<()> for TextField {
        fn init(
            gui: &mut Gui,
            render: &mut GLSpriteRender,
            fonts: MyFonts,
            _event_loop: &EventLoop<()>,
        ) -> Self {
            let texture = {
                let data = image::open("D:/repos/rust/crui/examples/panel.png").unwrap();
                let data = data.to_rgba8();
                render.new_texture(data.width(), data.height(), data.as_ref(), true)
            };
            let surface = gui
                .create_control()
                .layout(MarginLayout::new([20.0; 4]))
                .build(gui);
            text_field(
                gui.create_control(),
                gui,
                "Hello Word!".to_string(),
                TextFieldStyle {
                    background: OnFocusStyle {
                        normal: Panel::new(texture, [0.0, 0.0, 0.5, 0.5], [10.0; 4]).into(),
                        focus: Panel::new(texture, [0.5, 0.5, 0.5, 0.5], [10.0; 4]).into(),
                    },
                    selection_color: SelectionColor {
                        bg: [170, 0, 255, 255].into(),
                        fg: Some(Color::WHITE),
                    },
                    caret_color: Color::BLACK,
                },
                fonts.notosans,
                (),
            )
            .parent(surface)
            .build(gui);
            TextField
        }
    }

    run::<(), TextField>(400, 200);
}

fn text_field<'a, C: TextFieldCallback + 'static>(
    cb: ControlBuilder,
    ctx: &mut impl BuilderContext,
    initial_value: String,
    style: TextFieldStyle,
    font_id: FontId,
    callback: C,
) -> ControlBuilder {
    let caret = ctx.reserve();
    let input_text = ctx.reserve();

    cb.behaviour(TextField::new(
        caret,
        input_text,
        Rc::new(style.clone()),
        callback,
    ))
    .min_size([0.0, 28.0])
    .expand_y(false)
    .child_reserved(caret, ctx, |cb, _| {
        cb.anchors([0.0, 0.0, 0.0, 0.0]).graphic(
            style
                .background
                .normal
                .clone()
                .with_color([0, 0, 0, 255].into()),
        )
    })
    .child_reserved(input_text, ctx, |cb, _| {
        cb.graphic(Text::new(
            initial_value,
            (-1, 0),
            crui::graphics::TextStyle {
                color: [0, 0, 0, 255].into(),
                font_size: 72.0,
                font_id,
                ..Default::default()
            },
        ))
    })
}
