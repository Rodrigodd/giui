use crui::{
    font::{Font, FontId, Fonts},
    render::{GuiRender, GuiRenderer},
    Gui,
};
use sprite_render::{Camera, GLSpriteRender, SpriteInstance, SpriteRender};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, WindowId},
};

#[allow(dead_code)]
fn main() {
    struct HelloWord;
    impl CruiEventLoop<()> for HelloWord {
        fn init(
            gui: &mut Gui,
            _render: &mut GLSpriteRender,
            fonts: MyFonts,
            _event_loop: &EventLoop<()>,
        ) -> Self {
            use crui::graphics::{Text, TextStyle};
            let _text = gui
                .create_control()
                .graphic(
                    Text::new(
                        "Hello Word!!".to_string(),
                        (0, 0),
                        TextStyle {
                            color: [0, 255, 0, 255],
                            font_size: 70.0,
                            font_id: fonts.notosans,
                        },
                    )
                    .into(),
                )
                .build();
            HelloWord
        }
    }

    run::<(), HelloWord>(400, 200);
}

pub struct MyFonts {
    pub notosans: FontId,
    pub consolas: FontId,
}

fn resize(
    gui: &mut Gui,
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

pub trait CruiEventLoop<T> {
    fn init(
        gui: &mut Gui,
        _render: &mut GLSpriteRender,
        fonts: MyFonts,
        _event_loop: &EventLoop<T>,
    ) -> Self;
    #[allow(unused_variables)]
    fn on_event(&mut self, event: &Event<T>, control: &mut ControlFlow) {}
}

pub fn run<U: 'static, T: CruiEventLoop<U> + 'static>(width: u32, height: u32) -> ! {
    // create winit's window and event_loop
    let event_loop = EventLoop::<U>::with_user_event();
    let window = WindowBuilder::new().with_inner_size(PhysicalSize::new(width, height));

    // create the render and camera, and a texture for the glyphs rendering
    let (window, mut render) = GLSpriteRender::new(window, &event_loop, true);
    let mut camera = {
        let size = window.inner_size();
        let width = size.width;
        let height = size.height;
        Camera::new(width, height, height as f32)
    };
    let font_texture = render.new_texture(128, 128, &[], false);

    // load a font
    let mut fonts = Fonts::new();
    let my_fonts = MyFonts {
        notosans: fonts.add(Font::new(include_bytes!(
            "../examples/NotoSans-Regular.ttf"
        ))),
        consolas: fonts.add(Font::new(include_bytes!("../examples/cour.ttf"))),
    };

    // create the gui, and the gui_render
    let mut gui = Gui::new(0.0, 0.0, fonts);
    let mut gui_render = GuiRender::new(font_texture, [128, 128]);

    // populate the gui with controls.
    let mut app = T::init(&mut gui, &mut render, my_fonts, &event_loop);

    // resize everthing to the screen size
    resize(
        &mut gui,
        &mut render,
        &mut camera,
        window.inner_size(),
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
                        resize(&mut gui, &mut render, &mut camera, size, window_id);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) => {
                // render the gui
                struct Render<'a>(&'a mut GLSpriteRender);
                impl<'a> GuiRenderer for Render<'a> {
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
                let mut ctx = gui.get_render_context();
                let (sprites, is_anim) = gui_render.render(&mut ctx, Render(&mut render));
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
                                color: x.color,
                                pos: [x.rect[0] + width / 2.0, x.rect[1] + height / 2.0],
                                texture: x.texture,
                            }
                        })
                        .collect::<Vec<_>>(),
                );

                if is_animating {
                    *control = ControlFlow::Poll;
                }

                renderer.finish();
            }
            _ => {}
        }
    });
}
