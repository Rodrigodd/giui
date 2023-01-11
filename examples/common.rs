use giui::{
    font::{Font, FontId, Fonts},
    render::{GuiRender, GuiRenderer},
    Gui,
};
use sprite_render::{Camera, GlSpriteRender, SpriteInstance, SpriteRender, TextureId};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::{WindowBuilder, WindowId},
};

#[allow(dead_code)]
fn main() {
    struct HelloWord;
    impl GiuiEventLoop<()> for HelloWord {
        fn create_textures(&mut self, _render: &mut dyn SpriteRender) {}
        fn init(gui: &mut Gui, fonts: MyFonts, _proxy: EventLoopProxy<()>) -> Self {
            use giui::graphics::{Text, TextStyle};
            let _text = gui
                .create_control()
                .graphic(Text::new(
                    "Hello Word!!".to_string(),
                    (0, 0),
                    TextStyle {
                        color: [0, 255, 0, 255].into(),
                        font_size: 70.0,
                        font_id: fonts.notosans,
                        ..Default::default()
                    },
                ))
                .build(gui);
            HelloWord
        }
    }

    run::<(), HelloWord>(400, 200);
}

#[derive(Clone)]
pub struct MyFonts {
    pub notosans: FontId,
    pub consolas: FontId,
    pub white_texture: u32,
    pub font_texture: u32,
}

fn resize(
    gui: &mut Gui,
    render: &mut dyn SpriteRender,
    camera: &mut Camera,
    size: PhysicalSize<u32>,
    scale_factor: f64,
    window: WindowId,
) {
    render.resize(window, size.width, size.height);
    camera.resize(size.width, size.height);
    let width = size.width as f32;
    let height = size.height as f32;
    camera.set_width(width);
    camera.set_height(height);
    camera.set_position(width / 2.0, height / 2.0);
    let width = width / scale_factor as f32;
    let height = height / scale_factor as f32;
    if cfg!(target_os = "android") {
        gui.set_root_rect([10.0, 0.0, width - 20.0, height - 50.0]);
    } else {
        gui.set_root_rect([0.0, 0.0, width, height]);
    }
}

pub trait GiuiEventLoop<T> {
    fn create_textures(&mut self, render: &mut dyn SpriteRender);

    fn init(gui: &mut Gui, fonts: MyFonts, proxy: EventLoopProxy<T>) -> Self;

    #[allow(unused_variables)]
    fn on_event(&mut self, event: &Event<T>, control: &mut ControlFlow) {}
}

fn create_textures<U: 'static, T: GiuiEventLoop<U> + 'static>(
    app: &mut T,
    gui_render: &mut GuiRender,
    render: &mut dyn SpriteRender,
    my_fonts: &mut MyFonts,
) {
    sprite_render::Texture::new(128, 128)
        .id(TextureId(my_fonts.font_texture))
        .create(render)
        .unwrap()
        .0;
    sprite_render::Texture::new(1, 1)
        .id(TextureId(my_fonts.white_texture))
        .data(&[255, 255, 255, 255])
        .create(render)
        .unwrap()
        .0;

    gui_render.set_font_texture(my_fonts.font_texture, [128, 128]);

    app.create_textures(render);
}

pub fn run<U: 'static, T: GiuiEventLoop<U> + 'static>(width: u32, height: u32) -> ! {
    #[cfg(not(target_os = "android"))]
    env_logger::init();
    // create winit's window and event_loop
    let event_loop = EventLoopBuilder::<U>::with_user_event().build();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(width, height))
        .build(&event_loop)
        .unwrap();

    // create the render and camera, and a texture for the glyphs rendering
    let mut render: Box<dyn SpriteRender> = Box::new(GlSpriteRender::new(&window, true).unwrap());

    let font_texture = 1000;
    let white_texture = 1001;

    let mut camera = {
        let size = window.inner_size();
        let width = size.width;
        let height = size.height;
        Camera::new(width, height, height as f32)
    };

    // load a font
    let mut fonts = Fonts::new();
    let mut my_fonts = MyFonts {
        notosans: fonts.add(Font::new(include_bytes!(
            "../examples/NotoSans-Regular.ttf"
        ))),
        consolas: fonts.add(Font::new(include_bytes!("../examples/cour.ttf"))),
        white_texture,
        font_texture,
    };

    // create the gui, and the gui_render
    let mut gui = Gui::new(0.0, 0.0, window.scale_factor(), fonts);
    let mut gui_render = GuiRender::new(font_texture, white_texture, [128, 128]);

    let proxy = event_loop.create_proxy();

    // populate the gui with controls.
    let mut app: T = T::init(&mut gui, my_fonts.clone(), proxy.clone());
    if cfg!(target_os = "android") {
    } else {
        create_textures(&mut app, &mut gui_render, render.as_mut(), &mut my_fonts);
    };

    // resize everthing to the screen size
    resize(
        &mut gui,
        &mut *render,
        &mut camera,
        window.inner_size(),
        window.scale_factor(),
        window.id(),
    );

    let mut is_animating = false;

    // winit event loop
    event_loop.run(move |event, _, control| {
        app.on_event(&event, control);
        match event {
            Event::NewEvents(_) => {
                *control = match gui.handle_scheduled_event() {
                    Some(time) => ControlFlow::WaitUntil(time),
                    None => ControlFlow::Wait,
                };
                if gui.render_is_dirty() {
                    window.request_redraw();
                }
                if let Some(cursor) = gui.cursor_change() {
                    window.set_cursor_icon(cursor);
                }
                if is_animating {
                    window.request_redraw();
                }
            }
            #[cfg(target_os = "android")]
            Event::Resumed => {
                render.resume(&window);
                create_textures(&mut app, &mut gui_render, render.as_mut(), &mut my_fonts);
            }
            #[cfg(target_os = "android")]
            Event::Suspended => {
                render.suspend();
            }
            Event::WindowEvent {
                event, window_id, ..
            } => {
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
                        *control = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(size) => {
                        resize(
                            &mut gui,
                            &mut *render,
                            &mut camera,
                            size,
                            window.scale_factor(),
                            window_id,
                        );
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) => {
                // render the gui
                struct Render<'a>(&'a mut dyn SpriteRender);
                impl<'a> GuiRenderer for Render<'a> {
                    fn update_font_texture(
                        &mut self,
                        font_texture: u32,
                        rect: [u32; 4],
                        data_tex: &[u8],
                    ) {
                        let mut data = Vec::with_capacity(data_tex.len() * 4);
                        for byte in data_tex.iter() {
                            data.extend([0xff, 0xff, 0xff, *byte].iter());
                        }
                        let _ = self.0.update_texture(
                            TextureId(font_texture),
                            Some(&data),
                            Some([rect[0], rect[1], rect[2] - rect[0], rect[3] - rect[1]]),
                        );
                    }
                    fn resize_font_texture(&mut self, font_texture: u32, new_size: [u32; 2]) {
                        let _ = sprite_render::Texture::new(new_size[0], new_size[1])
                            .id(sprite_render::TextureId(font_texture))
                            .create(self.0);
                    }
                }
                let mut ctx = gui.get_render_context();
                let (sprites, is_anim) = gui_render.render(&mut ctx, Render(&mut *render));
                is_animating = is_anim;
                let mut renderer = render.render(window_id);

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
                                color: x.color.to_array(),
                                pos: [x.rect[0] + width / 2.0, x.rect[1] + height / 2.0],
                                texture: TextureId(x.texture),
                            }
                        })
                        .collect::<Vec<_>>(),
                );
                renderer.finish();

                if is_animating {
                    *control = ControlFlow::Poll;
                }
            }
            _ => {}
        }
    })
}
