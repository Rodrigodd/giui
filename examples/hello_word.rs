use ab_glyph::FontArc;
use sprite_render::{Camera, GLSpriteRender, SpriteInstance, SpriteRender};
use ui_engine::{
    render::{GUIRender, Text},
    GUI,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn resize(
    gui: &mut GUI,
    render: &mut GLSpriteRender,
    camera: &mut Camera,
    size: &PhysicalSize<u32>,
) {
    render.resize(size.width, size.height);
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
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_inner_size(PhysicalSize::new(400, 200));

    // create the render and camera, and a texture for the glyphs rendering
    let (window, mut render) = GLSpriteRender::new(window, &event_loop, true);
    let mut camera = {
        let size = window.inner_size();
        let width = size.width;
        let height = size.height;
        Camera::new(width, height, height as f32)
    };
    let font_texture = render.new_texture(1024, 1024, &[], false);

    // load a font
    let fonts: Vec<FontArc> = [include_bytes!("../examples/NotoSans-Regular.ttf")]
        .iter()
        .map(|&font| FontArc::try_from_slice(font).unwrap())
        .collect();

    // create the gui, and the gui_render
    let mut gui = GUI::new(0.0, 0.0, fonts);
    let mut gui_render = GUIRender::new(font_texture);

    // populate the gui with controls. In this case a green 'Hello Word' text covering the entire of the screen.
    let _text = gui
        .create_control()
        .with_graphic(Text::new([0, 255, 0, 255], "Hello Word!!".to_string(), 70.0, (0, 0)).into())
        .build();

    // resize everthing to the screen size
    resize(&mut gui, &mut render, &mut camera, &window.inner_size());

    // winit event loop
    event_loop.run(move |event, _, control| {
        *control = ControlFlow::Wait;

        // gui receive events
        gui.handle_event(&event);
        if gui.render_is_dirty() {
            window.request_redraw();
        }

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    resize(&mut gui, &mut render, &mut camera, &size);
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                // render the gui
                let mut ctx = gui.get_render_context();
                let sprites = gui_render.render(&mut ctx, |rect, tex_data| {
                    let mut data = Vec::with_capacity(tex_data.len() * 4);
                    for byte in tex_data.iter() {
                        data.extend([0xff, 0xff, 0xff, *byte].iter());
                    }
                    render.update_texture(
                        font_texture,
                        &data,
                        Some([rect.min[0], rect.min[1], rect.width(), rect.height()]),
                    );
                });
                let mut renderer = render.render();
                renderer.clear_screen(&[0.0, 0.0, 0.0, 1.0]);
                renderer.draw_sprites(
                    &mut camera,
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
