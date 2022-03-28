mod common;
use common::MyFonts;
use std::rc::Rc;

use common::*;
use giui::{
    font::FontId,
    graphics::{Graphic, Panel, Text},
    layouts::{FitGraphic, HBoxLayout, VBoxLayout},
    style::{ButtonStyle, OnFocusStyle, SelectionColor, TextFieldStyle},
    widgets::{
        Button, List, ListBuilder, ScrollBar, TextField, TextFieldCallback, UpdateItems, ViewLayout,
    },
    BuilderContext, Color, ControlBuilder, Gui, Id,
};
use sprite_render::{GLSpriteRender, SpriteRender};
use winit::event_loop::EventLoop;

fn main() {
    struct Todos;
    impl GiuiEventLoop<()> for Todos {
        fn init(
            gui: &mut Gui,
            render: &mut GLSpriteRender,
            fonts: MyFonts,
            _event_loop: &EventLoop<()>,
        ) -> Self {
            let texture = {
                let data = image::open("examples/panel.png").unwrap();
                let data = data.to_rgba8();
                render.new_texture(data.width(), data.height(), data.as_ref(), true)
            };
            build_gui(
                gui,
                StyleSheet {
                    fonts,
                    text_field: TextFieldStyle {
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
                    button_style: ButtonStyle {
                        normal: Panel::new(texture, [0.0, 0.0, 0.5, 0.5], [10.0; 4]).into(),
                        hover: Panel::new(texture, [0.5, 0.0, 0.5, 0.5], [10.0; 4]).into(),
                        pressed: Panel::new(texture, [0.0, 0.5, 0.5, 0.5], [10.0; 4]).into(),
                        focus: Panel::new(texture, [0.5, 0.5, 0.5, 0.5], [10.0; 4]).into(),
                    }
                    .into(),
                },
            );
            Todos
        }
    }

    run::<(), Todos>(400, 200);
}

struct StyleSheet {
    text_field: TextFieldStyle,
    button_style: Rc<ButtonStyle>,
    fonts: MyFonts,
}

fn build_gui(gui: &mut Gui, style: StyleSheet) {
    let surface = gui
        .create_control()
        .layout(VBoxLayout::new(5.0, [20.0; 4], -1))
        .build(gui);
    let list_id = gui.reserve_id();
    eprintln!("this_list: {}", list_id);

    struct App {
        list: Vec<String>,
    }
    gui.set(App {
        list: (0..10)
            .map(|x| "this is text:\n".to_string() + &x.to_string() + " number")
            .collect(),
    });

    struct Callback {
        list: Id,
    }
    impl TextFieldCallback for Callback {
        fn on_submit(&mut self, _this: giui::Id, ctx: &mut giui::Context, text: &mut String) {
            if text.is_empty() {
                return;
            }
            let app = ctx.get_mut::<App>();

            if let Some((index, text)) = text
                .split_once('=')
                .map_or(None, |(a, b)| a.parse::<usize>().ok().map(|a| (a, b)))
            {
                app.list[index] = text.to_string();
            } else {
                app.list.push(text.clone());
            }
            text.clear();
            ctx.send_event_to(self.list, UpdateItems);
        }

        fn on_change(&mut self, _this: giui::Id, _ctx: &mut giui::Context, _text: &str) {}

        fn on_unfocus(&mut self, _this: giui::Id, _ctx: &mut giui::Context, _text: &mut String) {}
    }

    text_field(
        gui.create_control(),
        gui,
        "Hello Word!".to_string(),
        style.text_field.clone(),
        style.fonts.notosans,
        Callback { list: list_id },
    )
    .parent(surface)
    .build(gui);

    let panel = gui
        .create_control()
        .parent(surface)
        .graphic(style.text_field.background.normal.clone())
        .expand_y(true)
        .build(gui);

    struct MyList(FontId, Rc<ButtonStyle>);
    impl ListBuilder for MyList {
        fn item_count(&mut self, ctx: &mut dyn BuilderContext) -> usize {
            (ctx.get::<App>().list.len() * 2).saturating_sub(1)
        }

        fn create_item<'a>(
            &mut self,
            index: usize,
            this_list: giui::Id,
            cb: ControlBuilder,
            ctx: &mut dyn BuilderContext,
        ) -> ControlBuilder {
            if index % 2 == 0 {
                eprintln!("this_list: {}", this_list);
                let index = index / 2;
                let x = &ctx.get::<App>().list[index].clone();

                println!("create item {}!!", x);
                let cb = cb.min_size([10.0, 20.0]).layout(HBoxLayout::default());

                ctx.create_control()
                    .parent(cb.id())
                    .graphic(Text::new(
                        x.to_string(),
                        (-1, 0),
                        giui::graphics::TextStyle {
                            color: [0, 0, 0, 255].into(),
                            font_size: 18.0,
                            font_id: self.0,
                            ..Default::default()
                        },
                    ))
                    .layout(FitGraphic)
                    .expand_x(true)
                    .build(ctx);

                ctx.create_control()
                    .min_size([20.0, 20.0])
                    .fill_y(giui::RectFill::ShrinkCenter)
                    .behaviour(Button::new(self.1.clone(), true, move |_, ctx| {
                        // remove this item from the list
                        ctx.get_mut::<App>().list.remove(index);
                        ctx.send_event_to(this_list, UpdateItems);
                    }))
                    .parent(cb.id())
                    .build(ctx);

                cb
            } else {
                cb.min_size([10.0, 2.0])
                    .graphic(self.1.normal.clone().with_color(Color::BLACK))
                    .expand_x(true)
            }
        }

        fn update_item(&mut self, index: usize, item_id: Id, ctx: &mut dyn BuilderContext) -> bool {
            let index = if index % 2 == 0 {
                index / 2
            } else {
                return true;
            };
            let text_id = ctx.get_active_children(item_id)[0];
            let x = ctx.get::<App>().list[index].clone();
            println!("updated item {}!!", x);
            if let Graphic::Text(text) = ctx.get_graphic_mut(text_id) {
                text.set_string(&x);
            }
            true
        }
    }

    list(
        gui.create_control_reserved(list_id),
        gui,
        &style,
        MyList(style.fonts.consolas, style.button_style.clone()),
    )
    .parent(panel)
    .build(gui);
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
        false,
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
            giui::graphics::TextStyle {
                color: [0, 0, 0, 255].into(),
                font_size: 18.0,
                font_id,
                ..Default::default()
            },
        ))
    })
}

fn list(
    cb: ControlBuilder,
    ctx: &mut impl BuilderContext,
    style: &StyleSheet,
    list_builder: impl ListBuilder + 'static,
) -> ControlBuilder {
    let scroll_view = cb.id();
    let view = ctx
        .create_control()
        .parent(scroll_view)
        .layout(ViewLayout::new(false, true))
        .build(ctx);
    let h_scroll_bar_handle = ctx.reserve();
    let h_scroll_bar = ctx
        .create_control()
        .min_size([10.0, 10.0])
        .parent(scroll_view)
        .behaviour(ScrollBar::new(
            h_scroll_bar_handle,
            scroll_view,
            false,
            style.button_style.clone(),
        ))
        .build(ctx);
    let h_scroll_bar_handle = ctx
        .create_control_reserved(h_scroll_bar_handle)
        .min_size([10.0, 10.0])
        .parent(h_scroll_bar)
        .build(ctx);
    let v_scroll_bar_handle = ctx.reserve();
    let v_scroll_bar = ctx
        .create_control()
        .min_size([10.0, 10.0])
        // .graphic(style.scroll_background.clone())
        .parent(scroll_view)
        .behaviour(ScrollBar::new(
            v_scroll_bar_handle,
            scroll_view,
            true,
            style.button_style.clone(),
        ))
        .build(ctx);
    let v_scroll_bar_handle = ctx
        .create_control_reserved(v_scroll_bar_handle)
        .min_size([10.0, 10.0])
        .parent(v_scroll_bar)
        .build(ctx);

    cb.behaviour_and_layout(List::new(
        10.0,
        10.0,
        [10.0; 4],
        view,
        v_scroll_bar,
        v_scroll_bar_handle,
        h_scroll_bar,
        h_scroll_bar_handle,
        list_builder,
    ))
}
