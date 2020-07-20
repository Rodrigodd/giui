#![allow(clippy::useless_vec)]
use glyph_brush_layout::ab_glyph::FontArc;
use sprite_render::{Camera, GLSpriteRender, SpriteRender};
use ui_engine::render::{GUISpriteRender, Graphic};
use ui_engine::{
    event as ui_event,
    layouts::{FitText, GridLayout, MarginLayout, VBoxLayout},
    widgets::{Button, Hoverable, Slider, TabButton, TabGroup, Toggle},
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
    let fonts: Vec<FontArc> = [include_bytes!("../examples/Comfortaa-Bold.ttf")]
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
    let painel = Graphic::Panel {
        texture,
        uv_rect: [0.0, 0.0, 1.0, 1.0],
        color: [255, 255, 255, 255],
        border: 5.0,
    };
    let page_area = gui
        .create_widget()
        .with_margins([0.0, 45.0, 0.0, 0.0])
        .build();
    let (hover, hover_label) = {
        let graphic = painel
            .clone()
            .with_color([50, 50, 50, 255])
            .with_border(0.0);
        let hover = gui
            .create_widget()
            .with_anchors([0.0, 0.0, 0.0, 0.0])
            .with_graphic(graphic)
            .with_margins([3.0, 6.0, 6.0, 9.0])
            .with_layout(Box::new(MarginLayout::new([3.0, 3.0, 3.0, 3.0])))
            .build();
        let graphic = Graphic::Text {
            color: [255, 255, 255, 255],
            text: "This is a Hover".to_owned(),
            font_size: 12.0,
            align: (-1, 0),
        };
        let label = gui
            .create_widget()
            .with_graphic(graphic)
            .with_parent(hover)
            .with_layout(Box::new(FitText))
            .build();

        (hover, label)
    };
    let page_1 = gui.create_widget().with_parent(page_area).build();
    let menu = {
        let graphic = painel.clone();
        gui.create_widget()
            .with_anchors([0.0, 0.0, 0.0, 1.0])
            .with_margins([10.0, 0.0, 190.0, -10.0])
            .with_graphic(graphic)
            .with_layout(Box::new(VBoxLayout::new(5.0, [5.0, 5.0, 5.0, 5.0], -1)))
            .with_parent(page_1)
            .build()
    };
    let right_painel = gui
        .create_widget()
        .with_margins([200.0, 0.0, -10.0, -10.0])
        .with_graphic(painel.clone())
        .with_parent(page_1)
        .build();
    let top_text = {
        let text_box = gui
            .create_widget()
            .with_anchors([0.0, 0.0, 1.0, 0.5])
            .with_margins([15.0, 15.0, -15.0, -7.5])
            .with_graphic(painel.clone().with_color([200, 200, 200, 255]))
            .with_parent(right_painel)
            .build();
        let graphic = Graphic::Text {
            text: "This is a example text. Please, don't mind me. Continue doing what you need to do. If you cannot ignore this text, I don't mind.".to_owned(),
            color: [0, 0, 0, 255],
            font_size: 20.0,
            align: (0, -1),
        };
        gui.create_widget()
            .with_anchors([0.0, 0.0, 1.0, 1.0])
            .with_margins([5.0, 5.0, -5.0, -5.0])
            .with_graphic(graphic)
            .with_parent(text_box)
            .build()
    };
    let bottom_text = {
        let text_box = gui
            .create_widget()
            .with_anchors([0.0, 0.5, 1.0, 1.0])
            .with_margins([15.0, 7.5, -15.0, -15.0])
            .with_graphic(painel.clone().with_color([200, 200, 200, 255]))
            .with_parent(right_painel)
            .build();
        let graphic = Graphic::Text {
            text: "This is another example text. Please, also don't mind me. Continue doing what you was doing. If you cannot ignore this text, I don't mind either.".to_owned(),
            font_size: 20.0,
            align: (-1, 0),
            color: [0, 0, 0, 255],
        };
        gui.create_widget()
            .with_anchors([0.0, 0.0, 1.0, 1.0])
            .with_margins([5.0, 5.0, -5.0, -5.0])
            .with_graphic(graphic)
            .with_parent(text_box)
            .build();
        text_box
    };

    let my_button = {
        let button = gui
            .create_widget()
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
        let graphic = Graphic::Text {
            text: "My Button".to_owned(),
            font_size: 16.0,
            align: (0, 0),
            color: [40, 40, 100, 255],
        };
        gui.create_widget()
            .with_anchors([0.0, 0.0, 1.0, 1.0])
            .with_margins([0.0, 0.0, 0.0, 0.0])
            .with_graphic(graphic)
            .with_parent(button)
            .build();
        button
    };
    let my_slider = {
        let slider = gui
            .create_widget()
            .with_min_size([0.0, 30.0])
            .with_parent(menu)
            .build();
        let slide_area = gui
            .create_widget()
            .with_anchors([0.0, 0.5, 1.0, 0.5])
            .with_margins([10.0, -3.0, -10.0, 3.0])
            .with_graphic(painel.clone().with_color([170, 170, 170, 255]))
            .with_parent(slider)
            .build();
        let handle = gui
            .create_widget()
            .with_anchors([0.5, 0.5, 0.5, 0.5])
            .with_margins([-3.0, -14.0, 3.0, 14.0])
            .with_graphic(painel.clone().with_color([200, 200, 200, 255]))
            .with_parent(slide_area)
            .build();
        gui.add_behaviour(
            slider,
            Box::new(Slider::new(handle, slide_area, 10.0, 30.0, 25.0)),
        );
        slider
    };
    let my_toggle = {
        let toggle = gui
            .create_widget()
            .with_min_size([0.0, 30.0])
            .with_parent(menu)
            .build();

        let background = {
            let graphic = painel
                .clone()
                .with_color([200, 200, 200, 255])
                .with_border(0.0);
            gui.create_widget()
                .with_anchors([0.0, 0.5, 0.0, 0.5])
                .with_margins([5.0, -10.0, 25.0, 10.0])
                .with_graphic(graphic)
                .with_parent(toggle)
                .build()
        };
        let marker = gui
            .create_widget()
            .with_anchors([0.5, 0.5, 0.5, 0.5])
            .with_margins([-6.0, -6.0, 6.0, 6.0])
            .with_graphic(painel.clone().with_color([0, 0, 0, 255]).with_border(0.0))
            .with_parent(background)
            .build();
        gui.add_behaviour(toggle, Box::new(Toggle::new(background, marker)));
        let page_2 = {
            let page_2 = gui
                .create_widget()
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
                gui.create_widget()
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
                    .create_widget()
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
                let graphic = Graphic::Text {
                    text: format!("{}x{}", min_size[0], min_size[1]),
                    font_size: 12.0,
                    align: (0, 0),
                    color: [40, 40, 100, 255],
                };
                gui.create_widget()
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
        let page_na = {
            let page_na = gui.create_widget().with_parent(page_area).build();
            let graphic = Graphic::Text {
                color: [255, 255, 255, 255],
                text: "This tab page is yet not avaliable. In fact, it is not even planned what will have in this page, sorry...".to_owned(),
                font_size: 20.0,
                align: (0, -1),
            };
            gui.create_widget()
                .with_margins([15.0, 15.0, -15.0, -15.0])
                .with_graphic(graphic)
                .with_parent(page_na)
                .build();
            page_na
        };
        {
            let header = gui
                .create_widget()
                .with_anchors([0.0, 0.0, 1.0, 0.0])
                .with_margins([5.0, 10.0, -10.0, 40.0])
                .build();
            let create_button = |gui: &mut GUI<GUISpriteRender>, i: usize, total: usize| {
                let x = i as f32 / total as f32;
                let button = gui
                    .create_widget()
                    .with_anchors([x, 0.0, x + 1.0 / total as f32, 1.0])
                    .with_margins([5.0, 0.0, 0.0, 0.0])
                    .with_graphic(painel.clone())
                    .with_parent(header)
                    .with_behaviour(Box::new(TabButton::new(header)))
                    .build();
                let graphic = Graphic::Text {
                    text: format!("Tab {}", i + 1),
                    font_size: 16.0,
                    align: (0, 0),
                    color: [40, 40, 100, 255],
                };
                gui.create_widget()
                    .with_anchors([0.0, 0.0, 1.0, 1.0])
                    .with_margins([0.0, 0.0, 0.0, 0.0])
                    .with_graphic(graphic)
                    .with_parent(button)
                    .build();
                button
            };
            let buttons = vec![
                create_button(&mut gui, 0, 4),
                create_button(&mut gui, 1, 4),
                create_button(&mut gui, 2, 4),
                create_button(&mut gui, 3, 4),
            ];
            let pages = vec![page_1, page_2, page_na, page_na];
            gui.add_behaviour(header, Box::new(TabGroup::new(buttons, pages)));
        }

        {
            let graphic = Graphic::Text {
                text: "Bottom Text".to_owned(),
                color: [40, 40, 100, 255],
                font_size: 16.0,
                align: (-1, 0),
            };
            gui.create_widget()
                .with_anchors([0.0, 0.0, 1.0, 1.0])
                .with_margins([30.0, 0.0, 0.0, 0.0])
                .with_graphic(graphic)
                .with_parent(toggle)
                .build();
        }

        toggle
    };
    drop(painel);

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
                    if let Some(Graphic::Text { font_size, .. }) = gui.get_graphic(top_text) {
                        *font_size = *value;
                    }
                }
            } else if let Some(ui_event::ValueSet { id, value }) = event.downcast_ref() {
                if *id == my_slider {
                    if let Some(Graphic::Text { font_size, .. }) = gui.get_graphic(top_text) {
                        *font_size = *value;
                    }
                    println!("Slide value set!! {}", value);
                }
            } else if let Some(ui_event::ToogleChanged { id, value }) = event.downcast_ref() {
                if *id == my_toggle {
                    println!("Toogle changed to {}!", value);
                    if *value {
                        gui.active_widget(bottom_text);
                    } else {
                        gui.deactive_widget(bottom_text);
                    }
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
                GUISpriteRender::prepare_render(&mut gui, &mut render);
                let mut renderer = render.render();
                renderer.clear_screen(&[0.0, 0.0, 0.0, 1.0]);
                gui.render().render(renderer.as_mut(), &mut screen_camera);
                renderer.finish();
            }
            _ => {}
        }
    })
}
