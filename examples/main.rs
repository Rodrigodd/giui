#![allow(clippy::useless_vec)]
use sprite_render::{Camera, GLSpriteRender, SpriteRender};
use ui_engine::render::{GUISpriteRender, GraphicId, Painel, Text};
use ui_engine::{
    event as ui_event,
    widgets::{Button, Slider, Toggle},
    GUIRender, Rect, Widget, GUI,
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
    let menu = {
        let graphic = Some(gui.render().add_painel(painel.clone()));
        let rect = Rect::new([0.0, 0.0, 0.0, 1.0], [10.0, 10.0, 190.0, -10.0]);
        gui.add_widget(Widget::new(rect, graphic, None), None)
    };
    let right_painel = {
        let graphic = Some(gui.render().add_painel(painel.clone()));
        let rect = Rect::new([0.0, 0.0, 1.0, 1.0], [200.0, 10.0, -10.0, -10.0]);
        gui.add_widget(Widget::new(rect, graphic, None), None)
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
                None,
            ),
            Some(right_painel),
        );
        let graphic = gui.render().add_text(Text::new(
            "This is a example text. Please, don't mind me. Continue doing what you need to do. If you cannot ignore this text, I don't mind.".to_string(),
            20.0,
            (0, -1),
        ));
        gui.add_widget(
            Widget::new(
                Rect::new([0.0, 0.0, 1.0, 1.0], [5.0, 5.0, -5.0, -5.0]),
                Some(graphic.clone()),
                None,
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
                None,
            ),
            Some(right_painel),
        );
        let graphic = Some(gui.render().add_text(Text::new(
            "This is another example text. Please, also don't mind me. Continue doing what you was doing. If you cannot ignore this text, I don't mind either.".to_string(),
            20.0,
            (-1, 0),
        )));
        gui.add_widget(
            Widget::new(
                Rect::new([0.0, 0.0, 1.0, 1.0], [5.0, 5.0, -5.0, -5.0]),
                graphic,
                None,
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
        let button = gui.add_widget(
            Widget::new(
                Rect::new([0.0, 0.0, 1.0, 0.0], [5.0, 5.0, -5.0, 35.0]),
                graphic,
                Some(Box::new(Button::new())),
            ),
            Some(menu),
        );
        let graphic = Some(gui.render().add_text(
            Text::new("My Button".to_string(), 16.0, (0, 0)).with_color([40, 40, 100, 255]),
        ));
        gui.add_widget(
            Widget::new(
                Rect::new([0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0]),
                graphic,
                None,
            ),
            Some(button),
        );
        button
    };
    let my_slider = {
        let slider = {
            let graphic = None;
            gui.add_widget(
                Widget::new(
                    Rect::new([0.0, 0.0, 1.0, 0.0], [5.0, 40.0, -5.0, 75.0]),
                    graphic,
                    None,
                ),
                Some(menu),
            )
        };
        let slide_area = {
            let graphic = Some(
                gui.render()
                    .add_painel(painel.clone().with_color([170, 170, 170, 255])),
            );
            gui.add_widget(
                Widget::new(
                    Rect::new([0.0, 0.5, 1.0, 0.5], [10.0, -3.0, -10.0, 3.0]),
                    graphic,
                    None,
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
                    None,
                ),
                Some(slide_area),
            )
        };
        gui.set_behaviour_of(
            slider,
            Some(Box::new(Slider::new(handle, slide_area, 10.0, 30.0, 25.0))),
        );
        slider
    };
    let my_toggle = {
        let toggle = gui.add_widget(
            Widget::new(
                Rect::new([0.0, 0.0, 1.0, 0.0], [5.0, 80.0, -5.0, 115.0]),
                None,
                None,
            ),
            Some(menu),
        );

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
                    None,
                ),
                Some(toggle),
            )
        };
        let marker = {
            let graphic = Some(
                gui.render().add_painel(
                    painel
                        .clone()
                        .with_color([0, 0, 0, 255])
                        .with_border(0.0),
                ),
            );
            gui.add_widget(
                Widget::new(
                    Rect::new([0.5, 0.5, 0.5, 0.5], [-6.0, -6.0, 6.0, 6.0]),
                    graphic,
                    None,
                ),
                Some(background),
            )
        };
        gui.set_behaviour_of(toggle, Some(Box::new(Toggle::new(background, marker))));

        {
            let graphic = Some(gui.render().add_text(
                Text::new("Bottom Text".to_string(), 16.0, (-1, 0)).with_color([40, 40, 100, 255]),
            ));
            gui.add_widget(
                Widget::new(
                    Rect::new([0.0, 0.0, 1.0, 1.0], [30.0, 0.0, 0.0, 0.0]),
                    graphic,
                    None,
                ),
                Some(toggle),
            );
        }

        toggle
    };
    drop(painel);

    gui.start();

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
            } else if let Some(ui_event::ToogleChanged{ id, value }) = event.downcast_ref() {
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
