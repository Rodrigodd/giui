#![allow(clippy::useless_vec)]
use sprite_render::{Camera, GLSpriteRender, SpriteRender};
use ui_engine::render::{GUISpriteRender, GraphicId, Painel, Text};
use ui_engine::{
    event as ui_event,
    widgets::{Button, Hoverable, Slider, TabButton, TabGroup, Toggle},
    layouts::{VBoxLayout, GridLayout},
    GUIRender, Rect, Widget, GUI, Id, RectFill
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
    let gui_render = GUISpriteRender::new(
        vec![include_bytes!("../examples/Comfortaa-Bold.ttf").to_vec()],
        font_texture,
    );
    let mut gui = GUI::new(
        window_size.width as f32,
        window_size.height as f32,
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
    let painel = Painel::new(texture, [0.0, 0.0, 1.0, 1.0], 5.0);
    let page_area = gui
        .create_widget()
        .with_margins([0.0, 45.0, 0.0, 0.0])
        .build();
    let (hover, hover_label) = {
        let graphic = gui.render().add_painel(
            painel
                .clone()
                .with_color([50, 50, 50, 255])
                .with_border(0.0),
        );
        let hover = gui
            .create_widget()
            .with_anchors([0.0, 0.0, 0.0, 0.0])
            .with_margins([3.0, 6.0, 93.0, 24.0])
            .with_graphic(Some(graphic))
            .build();
        let graphic = Some(gui.render().add_text(
            Text::new("This is a Hover".to_owned(), 12.0, (0, 0)).with_color([255, 255, 255, 255]),
        ));
        let label = gui
            .create_widget()
            .with_graphic(graphic)
            .with_parent(Some(hover))
            .build();

        (hover, label)
    };
    let page_1 = gui.create_widget().with_parent(Some(page_area)).build();
    let menu = {
        let graphic = Some(gui.render().add_painel(painel.clone()));
        gui.create_widget()
            .with_anchors([0.0, 0.0, 0.0, 1.0])
            .with_margins([10.0, 0.0, 190.0, -10.0])
            .with_graphic(graphic)
            .with_layout(Box::new(VBoxLayout::new(5.0, [5.0, 5.0, 5.0, 5.0], -1)))
            .with_parent(Some(page_1))
            .build()
    };
    let right_painel = {
        let graphic = Some(gui.render().add_painel(painel.clone()));
        let rect = Rect::new([0.0, 0.0, 1.0, 1.0], [200.0, 0.0, -10.0, -10.0]);
        gui.add_widget(Widget::new(rect, graphic), Some(page_1))
    };
    let top_text = {
        let graphic = Some(
            gui.render()
                .add_painel(painel.clone().with_color([200, 200, 200, 255])),
        );
        let text_box = gui.add_widget(
            Widget::new(
                Rect::new([0.0, 0.0, 1.0, 0.5], [15.0, 15.0, -15.0, -7.5]),
                graphic,
            ),
            Some(right_painel),
        );
        let graphic = gui.render().add_text(Text::new(
            "This is a example text. Please, don't mind me. Continue doing what you need to do. If you cannot ignore this text, I don't mind.".to_owned(),
            20.0,
            (0, -1),
        ));
        gui.add_widget(
            Widget::new(
                Rect::new([0.0, 0.0, 1.0, 1.0], [5.0, 5.0, -5.0, -5.0]),
                Some(graphic.clone()),
            ),
            Some(text_box),
        );
        graphic
    };
    let bottom_text = {
        let graphic = Some(
            gui.render()
                .add_painel(painel.clone().with_color([200, 200, 200, 255])),
        );
        let text_box = gui.add_widget(
            Widget::new(
                Rect::new([0.0, 0.5, 1.0, 1.0], [15.0, 7.5, -15.0, -15.0]),
                graphic,
            ),
            Some(right_painel),
        );
        let graphic = Some(gui.render().add_text(Text::new(
            "This is another example text. Please, also don't mind me. Continue doing what you was doing. If you cannot ignore this text, I don't mind either.".to_owned(),
            20.0,
            (-1, 0),
        )));
        gui.add_widget(
            Widget::new(
                Rect::new([0.0, 0.0, 1.0, 1.0], [5.0, 5.0, -5.0, -5.0]),
                graphic,
            ),
            Some(text_box),
        );
        text_box
    };

    let my_button = {
        let graphic = Some(
            gui.render()
                .add_painel(painel.clone().with_color([200, 200, 200, 255])),
        );
        let button = gui
            .create_widget()
            // .with_anchors([0.0, 0.0, 1.0, 0.0])
            // .with_margins([5.0, 5.0, -5.0, 35.0])
            .with_min_size([0.0, 30.0])
            .with_graphic(graphic)
            .with_behaviour(Box::new(Button::new()))
            .with_behaviour(Box::new(Hoverable::new(
                hover,
                hover_label,
                "This is a button".to_owned(),
            )))
            .with_parent(Some(menu))
            .build();
        let graphic = Some(gui.render().add_text(
            Text::new("My Button".to_owned(), 16.0, (0, 0)).with_color([40, 40, 100, 255]),
        ));
        gui.add_widget(
            Widget::new(
                Rect::new([0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0]),
                graphic,
            ),
            Some(button),
        );
        button
    };
    let my_slider = {
        let slider = gui
            .create_widget()
            // .with_anchors([0.0, 0.0, 1.0, 0.0])
            // .with_margins([5.0, 40.0, -5.0, 75.0])
            .with_min_size([0.0, 30.0])
            .with_parent(Some(menu))
            .build();
        let slide_area = {
            let graphic = Some(
                gui.render()
                    .add_painel(painel.clone().with_color([170, 170, 170, 255])),
            );
            gui.add_widget(
                Widget::new(
                    Rect::new([0.0, 0.5, 1.0, 0.5], [10.0, -3.0, -10.0, 3.0]),
                    graphic,
                ),
                Some(slider),
            )
        };
        let handle = {
            let graphic = Some(
                gui.render()
                    .add_painel(painel.clone().with_color([200, 200, 200, 255])),
            );
            gui.add_widget(
                Widget::new(
                    Rect::new([0.5, 0.5, 0.5, 0.5], [-3.0, -14.0, 3.0, 14.0]),
                    graphic,
                ),
                Some(slide_area),
            )
        };
        gui.add_behaviour(
            slider,
            Box::new(Slider::new(handle, slide_area, 10.0, 30.0, 25.0)),
        );
        slider
    };
    let my_toggle = {
        let toggle = gui
            .create_widget()
            // .with_anchors([0.0, 0.0, 1.0, 0.0])
            // .with_margins([5.0, 80.0, -5.0, 115.0])
            .with_min_size([0.0, 30.0])
            .with_parent(Some(menu))
            .build();

        let background = {
            let graphic = Some(
                gui.render().add_painel(
                    painel
                        .clone()
                        .with_color([200, 200, 200, 255])
                        .with_border(0.0),
                ),
            );
            gui.add_widget(
                Widget::new(
                    Rect::new([0.0, 0.5, 0.0, 0.5], [5.0, -10.0, 25.0, 10.0]),
                    graphic,
                ),
                Some(toggle),
            )
        };
        let marker = {
            let graphic = Some(
                gui.render()
                    .add_painel(painel.clone().with_color([0, 0, 0, 255]).with_border(0.0)),
            );
            gui.add_widget(
                Widget::new(
                    Rect::new([0.5, 0.5, 0.5, 0.5], [-6.0, -6.0, 6.0, 6.0]),
                    graphic,
                ),
                Some(background),
            )
        };
        gui.add_behaviour(toggle, Box::new(Toggle::new(background, marker)));
        let page_2 = {
            let graphic = gui.render().add_painel(painel.clone());
            let page_2 = gui
                .create_widget()
                .with_margins([10.0, 0.0, -10.0, -10.0])
                .with_graphic(Some(graphic))
                .with_parent(Some(page_area))
                .with_layout(Box::new(GridLayout::new([10.0, 15.0], [10.0, 10.0, 10.0, 10.0], 3)))
                .build();

            let create_vbox = |gui: &mut GUI<GUISpriteRender>, expand: [bool; 2], aling: i8| {
                let graphic = gui
                    .render()
                    .add_painel(painel.clone().with_color([100, 100, 100, 255]));
                gui
                    .create_widget()
                    .with_parent(Some(page_2))
                    .with_expand_x(expand[0])
                    .with_expand_y(expand[1])
                    .with_graphic(Some(graphic))
                    .with_layout(Box::new(VBoxLayout::new(5.0, [0.0, 0.0, 0.0, 0.0], aling)))
                    .build()
            };

            let create_rect = |gui: &mut GUI<GUISpriteRender>, min_size: [f32; 2], expand: [bool; 2], fill: [RectFill; 2], parent: Id| {
                let graphic = gui
                    .render()
                    .add_painel(painel.clone().with_color([200, 200, 200, 255]));

                let rect = gui.create_widget()
                    .with_min_size(min_size)
                    .with_fill_x(fill[0])
                    .with_fill_y(fill[1])
                    .with_expand_x(expand[0])
                    .with_expand_y(expand[1])
                    .with_graphic(Some(graphic))
                    .with_behaviour(Box::new(Hoverable::new(
                        hover,
                        hover_label,
                        "1: 30x30".to_owned(),
                    )))
                    .with_parent(Some(parent))
                    .build();
                let graphic = Some(
                    gui.render().add_text(
                        Text::new(format!("{}x{}", min_size[0], min_size[1]), 12.0, (0, 0))
                            .with_color([40, 40, 100, 255]),
                    ),
                );
                gui.add_widget(
                    Widget::new(
                        Rect::new([0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0]),
                        graphic,
                    ),
                    Some(rect),
                );
                rect
            };

            {
                let vbox = create_vbox(&mut gui, [true, true], -1);
                create_rect(&mut gui, [50.0, 50.0], [false, false], [RectFill::ShrinkStart, RectFill::Fill], vbox);
                create_rect(&mut gui, [75.0, 50.0], [true, true], [RectFill::Fill, RectFill::Fill], vbox);
                create_rect(&mut gui, [50.0, 75.0], [true, true], [RectFill::Fill, RectFill::Fill], vbox);
            }
            {
                let vbox = create_vbox(&mut gui, [false, true], 0);
                create_rect(&mut gui, [50.0, 50.0], [false, false], [RectFill::ShrinkStart, RectFill::Fill], vbox);
                create_rect(&mut gui, [75.0, 50.0], [false, false], [RectFill::ShrinkCenter, RectFill::Fill], vbox);
                create_rect(&mut gui, [50.0, 75.0], [false, false], [RectFill::ShrinkEnd, RectFill::Fill], vbox);
            }
            {
                let vbox = create_vbox(&mut gui, [false, false], 1);
                create_rect(&mut gui, [50.0, 50.0], [false, false], [RectFill::ShrinkStart, RectFill::Fill], vbox);
                create_rect(&mut gui, [75.0, 50.0], [false, false], [RectFill::ShrinkCenter, RectFill::Fill], vbox);
                create_rect(&mut gui, [50.0, 75.0], [false, false], [RectFill::ShrinkEnd, RectFill::Fill], vbox);
            }
            
            {
                let vbox = create_vbox(&mut gui, [true, false], -1);
                create_rect(&mut gui, [50.0, 50.0], [false, false], [RectFill::ShrinkStart, RectFill::Fill], vbox);
                create_rect(&mut gui, [75.0, 50.0], [false, false], [RectFill::ShrinkCenter, RectFill::Fill], vbox);
                create_rect(&mut gui, [50.0, 75.0], [false, false], [RectFill::ShrinkEnd, RectFill::Fill], vbox);
            }
            {
                let vbox = create_vbox(&mut gui, [false, false], 0);
                create_rect(&mut gui, [50.0, 50.0], [false, false], [RectFill::ShrinkStart, RectFill::Fill], vbox);
                create_rect(&mut gui, [75.0, 50.0], [false, false], [RectFill::ShrinkCenter, RectFill::Fill], vbox);
                create_rect(&mut gui, [50.0, 75.0], [false, false], [RectFill::ShrinkEnd, RectFill::Fill], vbox);
            }
            {
                let vbox = create_vbox(&mut gui, [false, false], 1);
                create_rect(&mut gui, [50.0, 50.0], [false, false], [RectFill::ShrinkStart, RectFill::Fill], vbox);
                create_rect(&mut gui, [75.0, 50.0], [false, false], [RectFill::ShrinkCenter, RectFill::Fill], vbox);
                create_rect(&mut gui, [50.0, 75.0], [false, false], [RectFill::ShrinkEnd, RectFill::Fill], vbox);
            }

            page_2
        };
        let page_na = {
            let rect = Rect::new([0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0]);
            let page_2 = gui.add_widget(Widget::new(rect, None), Some(page_area));
            let graphic = gui.render().add_text(Text::new(
                "This tab page is yet not avaliable. In fact, it is not even planned what will have in this page, sorry...".to_owned(),
                20.0,
                (0, -1),
            ).with_color([255, 255, 255, 255]));
            gui.add_widget(
                Widget::new(
                    Rect::new([0.0, 0.0, 1.0, 1.0], [15.0, 15.0, -15.0, -15.0]),
                    Some(graphic),
                ),
                Some(page_2),
            );
            page_2
        };
        {
            let rect = Rect::new([0.0, 0.0, 1.0, 0.0], [5.0, 10.0, -10.0, 40.0]);
            let header = gui.add_widget(Widget::new(rect, None), None);
            let create_button = |gui: &mut GUI<GUISpriteRender>, i: usize, total: usize| {
                let graphic = Some(
                    gui.render()
                        .add_painel(painel.clone().with_color([200, 200, 200, 255])),
                );
                let x = i as f32 / total as f32;
                let button = gui.add_widget(
                    Widget::new(
                        Rect::new([x, 0.0, x + 1.0 / total as f32, 1.0], [5.0, 0.0, 0.0, 0.0]),
                        graphic,
                    )
                    .with_behaviour(Box::new(TabButton::new(header))),
                    Some(header),
                );
                let graphic = Some(
                    gui.render().add_text(
                        Text::new(format!("Tab {}", i + 1), 16.0, (0, 0))
                            .with_color([40, 40, 100, 255]),
                    ),
                );
                gui.add_widget(
                    Widget::new(
                        Rect::new([0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0]),
                        graphic,
                    ),
                    Some(button),
                );
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
            let graphic = Some(gui.render().add_text(
                Text::new("Bottom Text".to_owned(), 16.0, (-1, 0)).with_color([40, 40, 100, 255]),
            ));
            gui.add_widget(
                Widget::new(
                    Rect::new([0.0, 0.0, 1.0, 1.0], [30.0, 0.0, 0.0, 0.0]),
                    graphic,
                ),
                Some(toggle),
            );
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
                    if let GraphicId::Text { index, .. } = top_text {
                        gui.render().get_text(index).set_scale(*value);
                    }
                }
            } else if let Some(ui_event::ValueSet { id, value }) = event.downcast_ref() {
                if *id == my_slider {
                    if let GraphicId::Text { index, .. } = top_text {
                        gui.render().get_text(index).set_scale(*value);
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
