#![allow(clippy::useless_vec)]
use ab_glyph::FontArc;
use sprite_render::{Camera, GLSpriteRender, SpriteRender};
use ui_engine::{
    event as ui_event,
    layouts::{FitText, GridLayout, HBoxLayout, MarginLayout, RatioLayout, VBoxLayout},
    render::{GUISpriteRender, Graphic, Panel, Text, Texture},
    widgets::{
        Button, Hoverable, ScrollBar, ScrollView, Slider, TabButton, TabGroup, TextField, Toggle,
    },
    GUIRender, Id, RectFill, GUI,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new().with_inner_size(PhysicalSize::new(800, 600));
    let (window, mut render) = GLSpriteRender::new(wb, &event_loop, true);
    let window_size = window.inner_size();
    let font_texture = render.new_texture(1024, 1024, &vec![0; 1024 * 1024 * 4], true);
    let gui_render = GUISpriteRender::new(font_texture);
    let fonts: Vec<FontArc> = [include_bytes!("../examples/NotoSans-Regular.ttf")]
        .iter()
        .map(|&font| FontArc::try_from_slice(font).unwrap())
        .collect();
    let mut gui = GUI::new(
        window_size.width as f32,
        window_size.height as f32,
        fonts,
        gui_render,
    );
    let texture = {
        let data = image::load_from_memory(include_bytes!("panel.png")).unwrap();
        let data = data.as_rgba8().unwrap();
        render.new_texture(data.width(), data.height(), data, true)
    };
    let mut screen_camera = sprite_render::Camera::new(
        window_size.width,
        window_size.height,
        window_size.height as f32,
    );
    screen_camera.set_position(
        window_size.width as f32 / 2.0,
        window_size.height as f32 / 2.0,
    );
    let painel: Graphic = Panel::new(texture, [0.0, 0.0, 1.0, 1.0], 5.0).into();
    let page_area = gui
        .create_control()
        .with_margins([0.0, 45.0, 0.0, 0.0])
        .build();
    let (hover, hover_label) = {
        let graphic = painel
            .clone()
            .with_color([50, 50, 50, 255])
            .with_border(0.0);
        let hover = gui
            .create_control()
            .with_anchors([0.0, 0.0, 0.0, 0.0])
            .with_graphic(graphic)
            .with_margins([3.0, 6.0, 6.0, 9.0])
            .with_layout(Box::new(MarginLayout::new([3.0, 3.0, 3.0, 3.0])))
            .build();
        let graphic = Text::new(
            [255, 255, 255, 255],
            "This is a Hover".to_owned(),
            12.0,
            (-1, 0),
        )
        .into();
        let label = gui
            .create_control()
            .with_graphic(graphic)
            .with_parent(hover)
            .with_layout(Box::new(FitText))
            .build();

        (hover, label)
    };
    let (page_1, top_text, bottom_text, my_button, my_slider, my_toggle) = {
        let page_1 = gui.create_control().with_parent(page_area).build();
        let menu = {
            let graphic = painel.clone();
            gui.create_control()
                .with_anchors([0.0, 0.0, 0.0, 1.0])
                .with_margins([10.0, 0.0, 190.0, -10.0])
                .with_graphic(graphic)
                .with_layout(Box::new(VBoxLayout::new(5.0, [5.0, 5.0, 5.0, 5.0], -1)))
                .with_parent(page_1)
                .build()
        };
        let right_painel = gui
            .create_control()
            .with_margins([200.0, 0.0, -10.0, -10.0])
            .with_graphic(painel.clone())
            .with_parent(page_1)
            .build();
        let top_text = {
            let text_box = gui
                .create_control()
                .with_anchors([0.0, 0.0, 1.0, 0.5])
                .with_margins([15.0, 15.0, -15.0, -7.5])
                .with_graphic(painel.clone().with_color([200, 200, 200, 255]))
                .with_parent(right_painel)
                .build();
            let graphic = Text::new(
            [0, 0, 0, 255],
            "This is a example text. Please, don't mind me. Continue doing what you need to do. If you cannot ignore this text, I don't mind.".to_owned(),
            20.0,
            (0, -1),
        ).into();
            gui.create_control()
                .with_anchors([0.0, 0.0, 1.0, 1.0])
                .with_graphic(graphic)
                .with_parent(text_box)
                .build()
        };
        let bottom_text = {
            let text_box = gui
                .create_control()
                .with_anchors([0.0, 0.5, 1.0, 1.0])
                .with_margins([15.0, 7.5, -15.0, -15.0])
                .with_graphic(painel.clone().with_color([200, 200, 200, 255]))
                .with_parent(right_painel)
                .build();
            let graphic = Text::new(
            [0, 0, 0, 255],
            "This is another example text. Please, also don't mind me. Continue doing what you was doing. If you cannot ignore this text, I don't mind either.".to_owned(),
            20.0,
            (-1, 0),
        ).into();
            gui.create_control()
                .with_anchors([0.0, 0.0, 1.0, 1.0])
                .with_margins([5.0, 5.0, -5.0, -5.0])
                .with_graphic(graphic)
                .with_parent(text_box)
                .build();
            text_box
        };

        let my_button = {
            let button = gui
                .create_control()
                .with_min_size([0.0, 30.0])
                .with_graphic(painel.clone().with_color([200, 200, 200, 255]))
                .with_behaviour(Box::new(Button::new()))
                .with_behaviour(Box::new(Hoverable::new(
                    hover,
                    hover_label,
                    "This is\na button".to_owned(),
                )))
                .with_parent(menu)
                .build();
            let graphic =
                Text::new([40, 40, 100, 255], "My Button".to_owned(), 16.0, (0, 0)).into();
            gui.create_control()
                .with_anchors([0.0, 0.0, 1.0, 1.0])
                .with_margins([0.0, 0.0, 0.0, 0.0])
                .with_graphic(graphic)
                .with_parent(button)
                .build();
            button
        };
        let my_slider = {
            let slider = gui
                .create_control()
                .with_min_size([0.0, 30.0])
                .with_parent(menu)
                .build();
            let slide_area = gui
                .create_control()
                .with_anchors([0.0, 0.5, 1.0, 0.5])
                .with_margins([10.0, -3.0, -10.0, 3.0])
                .with_graphic(painel.clone().with_color([170, 170, 170, 255]))
                .with_parent(slider)
                .build();
            let handle = gui
                .create_control()
                .with_anchors([0.5, 0.5, 0.5, 0.5])
                .with_margins([-3.0, -14.0, 3.0, 14.0])
                .with_graphic(painel.clone().with_color([200, 200, 200, 255]))
                .with_parent(slider)
                .build();
            gui.add_behaviour(
                slider,
                Box::new(Slider::new(handle, slide_area, 10.0, 30.0, 25.0)),
            );
            slider
        };
        let my_toggle = {
            let toggle = gui
                .create_control()
                .with_min_size([0.0, 30.0])
                .with_parent(menu)
                .build();

            let background = {
                let graphic = painel
                    .clone()
                    .with_color([200, 200, 200, 255])
                    .with_border(0.0);
                gui.create_control()
                    .with_anchors([0.0, 0.5, 0.0, 0.5])
                    .with_margins([5.0, -10.0, 25.0, 10.0])
                    .with_graphic(graphic)
                    .with_parent(toggle)
                    .build()
            };
            let marker = gui
                .create_control()
                .with_anchors([0.5, 0.5, 0.5, 0.5])
                .with_margins([-6.0, -6.0, 6.0, 6.0])
                .with_graphic(painel.clone().with_color([0, 0, 0, 255]).with_border(0.0))
                .with_parent(background)
                .build();
            gui.add_behaviour(toggle, Box::new(Toggle::new(background, marker)));

            let graphic =
                Text::new([40, 40, 100, 255], "Bottom Text".to_owned(), 16.0, (-1, 0)).into();
            gui.create_control()
                .with_anchors([0.0, 0.0, 1.0, 1.0])
                .with_margins([30.0, 0.0, 0.0, 0.0])
                .with_graphic(graphic)
                .with_parent(toggle)
                .build();

            toggle
        };
        (
            page_1,
            top_text,
            bottom_text,
            my_button,
            my_slider,
            my_toggle,
        )
    };
    let page_2 = {
        let page_2 = gui
            .create_control()
            .with_margins([10.0, 0.0, -10.0, -10.0])
            .with_graphic(painel.clone())
            .with_parent(page_area)
            .with_layout(Box::new(GridLayout::new(
                [10.0, 15.0],
                [10.0, 10.0, 10.0, 10.0],
                3,
            )))
            .build();

        let create_vbox = |gui: &mut GUI<GUISpriteRender>, expand: [bool; 2], align: i8| {
            gui.create_control()
                .with_parent(page_2)
                .with_expand_x(expand[0])
                .with_expand_y(expand[1])
                .with_graphic(painel.clone().with_color([100, 100, 100, 255]))
                .with_layout(Box::new(VBoxLayout::new(5.0, [0.0, 0.0, 0.0, 0.0], align)))
                .build()
        };

        let create_rect = |gui: &mut GUI<GUISpriteRender>,
                           min_size: [f32; 2],
                           expand: [bool; 2],
                           fill: [RectFill; 2],
                           parent: Id| {
            let rect = gui
                .create_control()
                .with_min_size(min_size)
                .with_fill_x(fill[0])
                .with_fill_y(fill[1])
                .with_expand_x(expand[0])
                .with_expand_y(expand[1])
                .with_graphic(painel.clone().with_color([200, 200, 200, 255]))
                .with_behaviour(Box::new(Hoverable::new(
                    hover,
                    hover_label,
                    format!(
                        "X: {:?}\nY: {:?}{}{}{}",
                        fill[0],
                        fill[1],
                        if expand[0] || expand[1] {
                            "\nExpand"
                        } else {
                            ""
                        },
                        if expand[0] { "X" } else { "" },
                        if expand[1] { "Y" } else { "" },
                    ),
                )))
                .with_parent(parent)
                .build();
            let graphic = Text::new(
                [40, 40, 100, 255],
                format!("{}x{}", min_size[0], min_size[1]),
                12.0,
                (0, 0),
            )
            .into();
            gui.create_control()
                .with_graphic(graphic)
                .with_parent(rect)
                .build();
            rect
        };

        {
            let vbox = create_vbox(&mut gui, [true, true], -1);
            create_rect(
                &mut gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [75.0, 50.0],
                [true, true],
                [RectFill::Fill, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [50.0, 75.0],
                [true, true],
                [RectFill::Fill, RectFill::Fill],
                vbox,
            );
        }
        {
            let vbox = create_vbox(&mut gui, [false, true], 0);
            create_rect(
                &mut gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [75.0, 50.0],
                [false, false],
                [RectFill::ShrinkCenter, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [50.0, 75.0],
                [false, false],
                [RectFill::ShrinkEnd, RectFill::Fill],
                vbox,
            );
        }
        {
            let vbox = create_vbox(&mut gui, [false, false], 1);
            create_rect(
                &mut gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [75.0, 50.0],
                [false, false],
                [RectFill::ShrinkCenter, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [50.0, 75.0],
                [false, false],
                [RectFill::ShrinkEnd, RectFill::Fill],
                vbox,
            );
        }

        {
            let vbox = create_vbox(&mut gui, [true, false], -1);
            create_rect(
                &mut gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [75.0, 50.0],
                [false, false],
                [RectFill::ShrinkCenter, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [50.0, 75.0],
                [false, false],
                [RectFill::ShrinkEnd, RectFill::Fill],
                vbox,
            );
        }
        {
            let vbox = create_vbox(&mut gui, [false, false], 0);
            create_rect(
                &mut gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [75.0, 50.0],
                [false, false],
                [RectFill::ShrinkCenter, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [50.0, 75.0],
                [false, false],
                [RectFill::ShrinkEnd, RectFill::Fill],
                vbox,
            );
        }
        {
            let vbox = create_vbox(&mut gui, [false, false], 1);
            create_rect(
                &mut gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [75.0, 50.0],
                [false, false],
                [RectFill::ShrinkCenter, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [50.0, 75.0],
                [false, false],
                [RectFill::ShrinkEnd, RectFill::Fill],
                vbox,
            );
        }

        page_2
    };
    let (page_3, input_box, list) = {
        let page_3 = gui
            .create_control()
            .with_graphic(painel.clone())
            .with_parent(page_area)
            .with_layout(Box::new(VBoxLayout::new(5.0, [10.0, 10.0, 10.0, 10.0], -1)))
            .build();
        let input_box = {
            let hbox = gui
                .create_control()
                .with_parent(page_3)
                .with_layout(Box::new(HBoxLayout::new(0.0, Default::default(), -1)))
                .build();
            let _label = gui
                .create_control()
                .with_graphic(
                    Text::new([0, 0, 0, 255], "Add new: ".to_owned(), 16.0, (-1, 0)).into(),
                )
                .with_layout(Box::new(FitText))
                .with_parent(hbox)
                .build();
            let input_box = gui
                .create_control()
                .with_min_size([0.0, 24.0])
                .with_graphic(painel.clone().with_color([200, 200, 200, 255]))
                .with_parent(hbox)
                .with_expand_x(true)
                .build();
            let caret = gui
                .create_control()
                .with_anchors([0.0, 0.0, 0.0, 0.0])
                .with_graphic(painel.clone().with_color([0, 0, 0, 255]).with_border(0.0))
                .with_parent(input_box)
                .build();
            let input_text = gui
                .create_control()
                .with_graphic(Text::new([0, 0, 0, 255], String::new(), 16.0, (-1, 0)).into())
                .with_parent(input_box)
                .build();
            gui.add_behaviour(input_box, Box::new(TextField::new(caret, input_text)));
            input_box
        };
        let scroll_view = gui
            .create_control()
            .with_graphic(
                painel
                    .clone()
                    .with_color([100, 100, 100, 255])
                    .with_border(0.0),
            )
            .with_expand_y(true)
            .with_parent(page_3)
            .build();
        let view = gui
            .create_control()
            .with_graphic(Graphic::Mask)
            .with_parent(scroll_view)
            .build();
        let h_scroll_bar = gui
            .create_control()
            .with_min_size([20.0, 20.0])
            .with_graphic(
                painel
                    .clone()
                    .with_color([150, 150, 150, 255])
                    .with_border(0.0),
            )
            .with_parent(scroll_view)
            .build();
        let h_scroll_bar_handle = gui
            .create_control()
            .with_graphic(
                painel
                    .clone()
                    .with_color([220, 220, 220, 255])
                    .with_border(0.0),
            )
            .with_parent(h_scroll_bar)
            .build();
        gui.add_behaviour(
            h_scroll_bar,
            Box::new(ScrollBar::new(h_scroll_bar_handle, scroll_view, false)),
        );
        let v_scroll_bar = gui
            .create_control()
            .with_min_size([20.0, 20.0])
            .with_graphic(
                painel
                    .clone()
                    .with_color([150, 150, 150, 255])
                    .with_border(0.0),
            )
            .with_parent(scroll_view)
            .build();
        let v_scroll_bar_handle = gui
            .create_control()
            .with_graphic(
                painel
                    .clone()
                    .with_color([220, 220, 220, 255])
                    .with_border(0.0),
            )
            .with_parent(v_scroll_bar)
            .build();
        gui.add_behaviour(
            v_scroll_bar,
            Box::new(ScrollBar::new(v_scroll_bar_handle, scroll_view, true)),
        );
        let list = gui
            .create_control()
            .with_layout(Box::new(VBoxLayout::new(3.0, [5.0, 5.0, 5.0, 5.0], -1)))
            .with_parent(view)
            .build();
        gui.add_behaviour(
            scroll_view,
            Box::new(ScrollView::new(
                view,
                list,
                h_scroll_bar,
                h_scroll_bar_handle,
                v_scroll_bar,
                v_scroll_bar_handle,
            )),
        );
        let create_item = |gui: &mut GUI<GUISpriteRender>, text: String, color: [u8; 4]| -> Id {
            let item = gui
                .create_control()
                .with_min_size([100.0, 35.0])
                .with_graphic(painel.clone().with_border(0.0).with_color(color))
                .with_parent(list)
                .with_layout(Box::new(MarginLayout::new([5.0, 0.0, 5.0, 0.0])))
                .build();
            //TODO: there must be a better way to increase the min_size height
            gui.create_control()
                .with_min_size([0.0, 35.0])
                .with_parent(item)
                .build();
            gui.create_control()
                .with_parent(item)
                .with_graphic(Text::new([0, 0, 0, 255], text, 16.0, (-1, 0)).into())
                .with_layout(Box::new(FitText))
                .build();
            item
        };
        let mut seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        seed = seed ^ (seed << 64);
        for i in 0..5 {
            let color = (seed.rotate_left(i * 3)) as u32 | 0xff;
            create_item(&mut gui, format!("This is the item number {} with the color which hexadecimal representation is #{:0x}", i + 1, color), color.to_be_bytes());
        }
        (page_3, input_box, list)
    };
    let page_4 = {
        let page_4 = gui
            .create_control()
            .with_parent(page_area)
            .with_layout(Box::new(RatioLayout::new(1.0, (0, 0))))
            .build();
        let graphic = Texture::new(font_texture, [0.0, 0.0, 1.0, 1.0]).into();
        gui.create_control()
            .with_graphic(graphic)
            .with_parent(page_4)
            .build();
        page_4
    };
    let page_na = {
        let page_na = gui.create_control().with_parent(page_area).build();
        let graphic = Text::new(
        [255, 255, 255, 255],
        "This tab page is yet not avaliable. In fact, it is not even planned what will have in this page, sorry...".to_owned(),
        20.0,
        (0, -1),
    ).into();
        gui.create_control()
            .with_margins([15.0, 15.0, -15.0, -15.0])
            .with_graphic(graphic)
            .with_parent(page_na)
            .build();
        page_na
    };
    let _tabs = {
        let header = gui
            .create_control()
            .with_anchors([0.0, 0.0, 1.0, 0.0])
            .with_margins([10.0, 10.0, -10.0, 40.0])
            .with_layout(Box::new(HBoxLayout::new(3.0, [0.0, 0.0, 0.0, 0.0], -1)))
            .build();
        let create_button = |gui: &mut GUI<GUISpriteRender>, label: String| {
            let button = gui
                .create_control()
                .with_graphic(painel.clone())
                .with_parent(header)
                .with_expand_x(true)
                .with_behaviour(Box::new(TabButton::new(header)))
                .with_layout(Box::new(MarginLayout::new([3.0, 0.0, 3.0, 0.0])))
                .build();
            let graphic = Text::new([40, 40, 100, 255], label, 16.0, (0, 0)).into();
            gui.create_control()
                .with_graphic(graphic)
                .with_parent(button)
                .with_layout(Box::new(FitText))
                .build();
            button
        };
        let buttons = vec![
            create_button(&mut gui, "Random Controls".to_owned()),
            create_button(&mut gui, "Grid Layout".to_owned()),
            create_button(&mut gui, "ScrollView".to_owned()),
            create_button(&mut gui, "Font Texture".to_owned()),
            create_button(&mut gui, "To be continued...".to_owned()),
        ];
        let pages = vec![page_1, page_2, page_3, page_4, page_na];
        gui.add_behaviour(header, Box::new(TabGroup::new(buttons, pages)));
    };

    println!("Starting");
    gui.start();
    println!("Started");

    fn resize<R: GUIRender>(
        size: PhysicalSize<u32>,
        ui: &mut GUI<R>,
        render: &mut GLSpriteRender,
        screen_camera: &mut Camera,
    ) {
        ui.resize(size.width as f32, size.height as f32);
        render.resize(size.width, size.height);
        screen_camera.resize(size.width, size.height);
        screen_camera.set_height(size.height as f32);
        screen_camera.set_position(size.width as f32 / 2.0, size.height as f32 / 2.0);
    };

    resize(window_size, &mut gui, &mut render, &mut screen_camera);

    event_loop.run(move |event, _, control| {
        *control = ControlFlow::Wait;
        gui.handle_event(&event);

        for event in gui.get_events().collect::<Vec<_>>() {
            if event.is::<ui_event::Redraw>() {
                window.request_redraw();
            } else if let Some(ui_event::ButtonClicked { id }) = event.downcast_ref() {
                if *id == my_button {
                    println!("Button clicked!!")
                }
            } else if let Some(ui_event::ValueChanged { id, value }) = event.downcast_ref() {
                if *id == my_slider {
                    if let Some(Graphic::Text(text)) = gui.get_graphic(top_text) {
                        text.set_font_size(*value);
                    }
                }
            } else if let Some(ui_event::ValueSet { id, value }) = event.downcast_ref() {
                if *id == my_slider {
                    if let Some(Graphic::Text(text)) = gui.get_graphic(top_text) {
                        text.set_font_size(*value);
                    }
                    println!("Slide value set!! {}", value);
                }
            } else if let Some(ui_event::ToggleChanged { id, value }) = event.downcast_ref() {
                if *id == my_toggle {
                    println!("Toogle changed to {}!", value);
                    if *value {
                        gui.active_control(bottom_text);
                    } else {
                        gui.deactive_control(bottom_text);
                    }
                }
            } else if let Ok(ui_event::SubmitText { id, text }) =
                event.downcast::<ui_event::SubmitText>().map(|x| *x)
            {
                if id == input_box {
                    println!("Submited {}!", text);
                    gui.send_event_to(id, Box::new(ui_event::ClearText));
                    let item = gui
                        .create_control()
                        .with_min_size([100.0, 35.0])
                        .with_graphic(
                            painel
                                .clone()
                                .with_border(0.0)
                                .with_color([200, 230, 255, 255]),
                        )
                        .with_parent(list)
                        .with_layout(Box::new(MarginLayout::new([5.0, 0.0, 5.0, 0.0])))
                        .build();
                    //TODO: there must be a better way to increase the min_size height
                    gui.create_control()
                        .with_min_size([0.0, 35.0])
                        .with_parent(item)
                        .build();
                    gui.create_control()
                        .with_parent(item)
                        .with_graphic(Text::new([0, 0, 0, 255], text, 16.0, (-1, 0)).into())
                        .with_layout(Box::new(FitText))
                        .build();
                }
            }
        }

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    resize(size, &mut gui, &mut render, &mut screen_camera);
                }
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let (gui_render, mut controls) = gui.get_render_and_controls();
                gui_render.prepare_render(&mut controls, &mut render);
                let mut renderer = render.render();
                renderer.clear_screen(&[0.0, 0.0, 0.0, 1.0]);
                gui.render().render(renderer.as_mut(), &mut screen_camera);
                renderer.finish();
            }
            _ => {}
        }
    })
}
