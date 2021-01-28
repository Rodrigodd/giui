use std::{collections::HashMap, rc::Rc};

use ab_glyph::FontArc;
use sprite_render::{Camera, GLSpriteRender, SpriteInstance, SpriteRender};
use ui_engine::{
    layouts::{FitText, MarginLayout},
    render::{GUIRender, GUIRenderer, Panel, Text},
    style::ButtonStyle,
    widgets::Button,
    GUI,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowBuilder, WindowId},
};

struct Instance {
    gui: GUI,
    gui_render: GUIRender,
    camera: Camera,
    window: Window,
}

enum UserEvent {
    CreateNewWindow {
        window_builder: WindowBuilder,
        build: Box<dyn Fn(&mut GUI)>,
    },
}

fn resize(
    gui: &mut GUI,
    render: &mut GLSpriteRender,
    camera: &mut Camera,
    size: PhysicalSize<u32>,
    window: WindowId,
) {
    render.resize(window, size.width, size.height);
    camera.resize(size.width, size.height);
    let width = size.width as f32;
    let height = size.height as f32;
    gui.resize(width, height);
    camera.set_width(width);
    camera.set_height(height);
    camera.set_position(width / 2.0, height / 2.0);
}

fn main() {
    // create winit's window and event_loop
    let event_loop = EventLoop::with_user_event();
    let window = WindowBuilder::new().with_inner_size(PhysicalSize::new(400, 200));

    // create the render and camera, and a texture for the glyphs rendering
    let (window, mut render) = GLSpriteRender::new(window, &event_loop, true);
    let mut camera = {
        let size = window.inner_size();
        let width = size.width;
        let height = size.height;
        Camera::new(width, height, height as f32)
    };
    let font_texture = render.new_texture(128, 128, &[], false);
    let texture = {
        let data = image::open("D:/repos/rust/ui_engine/examples/panel.png").unwrap();
        let data = data.to_rgba8();
        render.new_texture(data.width(), data.height(), data.as_ref(), true)
    };

    // load a font
    let fonts: Vec<FontArc> = [include_bytes!("../examples/NotoSans-Regular.ttf")]
        .iter()
        .map(|&font| FontArc::try_from_slice(font).unwrap())
        .collect();

    // create the gui, and the gui_render
    let mut gui = GUI::new(0.0, 0.0, fonts.clone());
    let gui_render = GUIRender::new(font_texture, [128, 128]);

    // populate the gui with controls. In this case a green 'Hello Word' text covering the entire of the screen.
    let button_style = Rc::new(ButtonStyle {
        normal: Panel::new(texture, [0.0, 0.0, 0.5, 0.5], 10.0).into(),
        hover: Panel::new(texture, [0.5, 0.0, 0.5, 0.5], 10.0).into(),
        pressed: Panel::new(texture, [0.0, 0.5, 0.5, 0.5], 10.0).into(),
        focus: Panel::new(texture, [0.5, 0.5, 0.5, 0.5], 10.0).into(),
    });
    create_main_gui(&mut gui, event_loop.create_proxy(), button_style);

    // resize everthing to the screen size
    resize(
        &mut gui,
        &mut render,
        &mut camera,
        window.inner_size(),
        window.id(),
    );

    let main_window = window.id();

    let mut windows: HashMap<WindowId, Instance> = HashMap::new();
    windows.insert(
        window.id(),
        Instance {
            gui,
            gui_render,
            camera,
            window,
        },
    );

    // winit event loop
    event_loop.run(move |event, event_loop, control| {
        *control = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::CreateNewWindow {
                window_builder: wb,
                build,
            }) => {
                let font_texture = render.new_texture(128, 128, &[], false);
                let window = render.add_window(wb, event_loop);
                let size = window.inner_size();
                let width = size.width;
                let height = size.height;
                let mut gui = GUI::new(width as f32, height as f32, fonts.clone());
                let gui_render = GUIRender::new(font_texture, [128, 128]);
                let mut camera = Camera::new(width, height, height as f32);

                resize(
                    &mut gui,
                    &mut render,
                    &mut camera,
                    window.inner_size(),
                    window.id(),
                );

                (build)(&mut gui);

                windows.insert(
                    window.id(),
                    Instance {
                        gui,
                        gui_render,
                        camera,
                        window,
                    },
                );
            }
            Event::WindowEvent {
                event, window_id, ..
            } => {
                let Instance {
                    ref mut gui,
                    ref mut camera,
                    ref mut window,
                    ..
                } = match windows.get_mut(&window_id) {
                    Some(x) => x,
                    None => return,
                };
                // gui receive events
                gui.handle_event(&event);
                if gui.render_is_dirty() {
                    window.request_redraw();
                }
                if let Some(cursor) = gui.cursor_change() {
                    window.set_cursor_icon(cursor);
                }
                match event {
                    WindowEvent::CloseRequested => {
                        if window_id == main_window {
                            for (_, Instance { window, .. }) in windows.drain() {
                                render.remove_window(&window);
                            }
                            *control = ControlFlow::Exit;
                        } else {
                            let Instance { window, .. } = windows.remove(&window_id).unwrap();
                            render.remove_window(&window);
                            if windows.is_empty() {
                                *control = ControlFlow::Exit;
                            }
                        }
                    }
                    WindowEvent::Resized(size) => {
                        resize(gui, &mut render, camera, size, window_id);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) => {
                let Instance {
                    ref mut gui,
                    ref mut gui_render,
                    ref mut camera,
                    ..
                } = windows.get_mut(&window_id).unwrap();

                // render the gui
                struct Render<'a>(&'a mut GLSpriteRender);
                impl<'a> GUIRenderer for Render<'a> {
                    fn update_font_texure(
                        &mut self,
                        font_texture: u32,
                        rect: [u32; 4],
                        data_tex: &[u8],
                    ) {
                        let mut data = Vec::with_capacity(data_tex.len() * 4);
                        for byte in data_tex.iter() {
                            data.extend([0xff, 0xff, 0xff, *byte].iter());
                        }
                        self.0.update_texture(
                            font_texture,
                            &data,
                            Some([rect[0], rect[1], rect[2] - rect[0], rect[3] - rect[1]]),
                        );
                    }
                    fn resize_font_texture(&mut self, font_texture: u32, new_size: [u32; 2]) {
                        self.0
                            .resize_texture(font_texture, new_size[0], new_size[1], &[]);
                    }
                }
                let mut ctx = gui.get_context();
                let sprites = gui_render.render(&mut ctx, Render(&mut render));
                let mut renderer = render.render(window_id);
                renderer.clear_screen(&[0.0, 0.0, 0.0, 1.0]);
                renderer.draw_sprites(
                    camera,
                    &sprites
                        .iter()
                        .map(|x| {
                            let width = x.rect[2] - x.rect[0];
                            let height = x.rect[3] - x.rect[1];
                            SpriteInstance {
                                scale: [width, height],
                                angle: 0.0,
                                uv_rect: x.uv_rect,
                                color: x.color,
                                pos: [x.rect[0] + width / 2.0, x.rect[1] + height / 2.0],
                                texture: x.texture,
                            }
                        })
                        .collect::<Vec<_>>(),
                );
                renderer.finish();
            }
            _ => {}
        }
    });
}

fn create_main_gui(gui: &mut GUI, proxy: EventLoopProxy<UserEvent>, button_style: Rc<ButtonStyle>) {
    let surface = gui
        .create_control()
        .with_layout(MarginLayout::new([10.0; 4]))
        .build();
    let button = gui
        .create_control()
        .with_behaviour(Button::new(button_style, move |_, _| {
            let window_builder = WindowBuilder::new().with_inner_size(PhysicalSize::new(200, 200));
            let _ = proxy.send_event(UserEvent::CreateNewWindow {
                window_builder,
                build: Box::new(create_second_gui),
            });
        }))
        .with_parent(surface)
        .with_layout(MarginLayout::new([5.0; 4]))
        .with_fill_x(ui_engine::RectFill::ShrinkCenter)
        .with_fill_y(ui_engine::RectFill::ShrinkCenter)
        .build();
    let _text = gui
        .create_control()
        .with_graphic(Text::new([0, 0, 0, 255], "Open Second Window!".into(), 16.0, (0, 0)).into())
        .with_layout(FitText)
        .with_parent(button)
        .build();
}

fn create_second_gui(gui: &mut GUI) {
    let _text = gui
        .create_control()
        .with_graphic(Text::new([0, 255, 0, 255], "Hello Word!!".into(), 48.0, (0, 0)).into())
        .build();
}
