use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    fs::File,
    io::{BufReader, BufWriter},
    rc::Rc,
};

// struct HasDrop<T>(T);
// impl<T> std::ops::Deref for HasDrop<T> {
//     type Target = T;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
// impl<T> std::ops::DerefMut for HasDrop<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }
// impl<T> Drop for HasDrop<T> {
//     fn drop(&mut self) {
//         println!("drop!");
//     }
// }

// struct RefCell<T>(cell::RefCell<T>);
// impl<T> RefCell<T> {
//     fn new(value: T) -> Self {
//         Self(cell::RefCell::new(value))
//     }

//     #[track_caller]
//     fn borrow(&self) -> HasDrop<std::cell::Ref<T>> {
//         use std::panic::Location;
//         println!("borrow! {}", Location::caller());
//         HasDrop(self.0.borrow())
//     }

//     #[track_caller]
//     fn borrow_mut(&self) -> std::cell::RefMut<T> {
//         use std::panic::Location;
//         println!("borrow mut! {}", Location::caller());
//         self.0.borrow_mut()
//     }
// }

use ab_glyph::FontArc;
use sprite_render::{Camera, GLSpriteRender, SpriteInstance, SpriteRender};
use crui::{
    event::SetValue,
    graphics::{Graphic, Panel, Text, Texture},
    layouts::{FitText, HBoxLayout, MarginLayout, VBoxLayout},
    render::{GUIRender, GUIRenderer},
    style::{ButtonStyle, MenuStyle, OnFocusStyle, TabStyle},
    widgets::{
        Blocker, Button, ButtonGroup, CloseMenu, DropMenu, Dropdown, Item, Menu, MenuBar, MenuItem,
        ScrollBar, ScrollView, Select, SetMaxValue, SetMinValue, SetSelected, Slider,
        SliderCallback, TabButton, TextField, TextFieldCallback, Toggle, ViewLayout,
    },
    Context, ControlBuilder, Id, RectFill, GUI,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{WindowBuilder, WindowId},
};

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
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(250, 300))
        .with_resizable(false);

    // create the render and camera, and a texture for the glyphs rendering
    let (window, mut render) = GLSpriteRender::new(window, &event_loop, true);
    let mut camera = {
        let size = window.inner_size();
        let width = size.width;
        let height = size.height;
        Camera::new(width, height, height as f32)
    };
    let font_texture = render.new_texture(128, 128, &[], false);

    // load textures
    let texture = {
        let data = image::open("D:/repos/rust/crui/examples/panel.png").unwrap();
        let data = data.to_rgba8();
        render.new_texture(data.width(), data.height(), data.as_ref(), true)
    };
    let icon_texture = {
        let data = image::open("D:/repos/rust/crui/examples/icons.png").unwrap();
        let data = data.to_rgba8();
        render.new_texture(data.width(), data.height(), data.as_ref(), true)
    };
    let tab_texture = {
        let data = image::open("D:/repos/rust/crui/examples/tab.png").unwrap();
        let data = data.to_rgba8();
        render.new_texture(data.width(), data.height(), data.as_ref(), true)
    };

    // load a font
    let fonts: Vec<FontArc> = [include_bytes!("../examples/NotoSans-Regular.ttf")]
        .iter()
        .map(|&font| FontArc::try_from_slice(font).unwrap())
        .collect();

    // create the gui, and the gui_render
    let mut gui = GUI::new(0.0, 0.0, fonts);
    let mut gui_render = GUIRender::new(font_texture, [128, 128]);

    // populate the gui with controls.
    let err;
    let options = match Options::load() {
        Ok(x) => {
            err = None;
            x
        }
        Err(x) => {
            err = Some(x);
            Options::default()
        }
    };
    let options = Rc::new(RefCell::new(options));
    let style_sheet = StyleSheet::new(texture, icon_texture, tab_texture);
    let options_gui = OptionsGUI::new(
        &mut gui,
        options.clone(),
        event_loop.create_proxy(),
        &style_sheet,
    );

    if let Some(e) = err {
        let mut ctx = gui.get_context();
        ctx.get_graphic_mut(options_gui.popup_title)
            .set_text("Load Failed");
        ctx.get_graphic_mut(options_gui.popup_text).set_text(&e);
        ctx.active(options_gui.popup);
    }

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
        match event {
            Event::NewEvents(_) => {
                *control = ControlFlow::Wait;
                if is_animating {
                    window.request_redraw()
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::UserEvent(UserEvent::Close) => {
                options.borrow().save();
                *control = ControlFlow::Exit;
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

                if let WindowEvent::Resized(size) = event {
                    resize(&mut gui, &mut render, &mut camera, size, window_id);
                }
            }
            Event::RedrawRequested(window_id) => {
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

#[derive(Clone)]
struct StyleSheet {
    menu: Rc<MenuStyle>,
    text_field: Rc<OnFocusStyle>,
    button: Rc<ButtonStyle>,
    tab_button: Rc<TabStyle>,
    page_background: Graphic,
    popup_background: Graphic,
    popup_header: Graphic,
    popup_window: Graphic,
    list_background: Graphic,
    scroll_background: Graphic,
    scroll_handle: Rc<ButtonStyle>,
}
impl StyleSheet {
    fn new(texture: u32, icon_texture: u32, tab_texture: u32) -> Self {
        let white = Texture::new(texture, [0.1, 0.1, 0.3, 0.3]);
        Self {
            menu: MenuStyle {
                button: ButtonStyle {
                    normal: white.clone().into(),
                    hover: Texture::new(texture, [0.6, 0.1, 0.3, 0.3]).into(),
                    pressed: Texture::new(texture, [0.1, 0.6, 0.3, 0.3]).into(),
                    focus: Texture::new(texture, [0.5, 0.5, 0.001, 0.001]).into(),
                },
                arrow: Texture::new(icon_texture, [0.0, 0.0, 1.0, 1.0]).into(),
                separator: Texture::new(texture, [0.2, 0.2, 0.2, 0.2])
                    .with_color([180, 180, 180, 255])
                    .into(),
            }
            .into(),
            text_field: OnFocusStyle {
                normal: Panel::new(texture, [0.0, 0.0, 0.5, 0.5], [10.0; 4]).into(),
                focus: Panel::new(texture, [0.5, 0.5, 0.5, 0.5], [10.0; 4]).into(),
            }
            .into(),
            button: Rc::new(ButtonStyle {
                normal: Graphic::from(Panel::new(texture, [0.0, 0.0, 0.5, 0.5], [10.0; 4])),
                hover: Graphic::from(Panel::new(texture, [0.5, 0.0, 0.5, 0.5], [10.0; 4])),
                pressed: Graphic::from(Panel::new(texture, [0.0, 0.5, 0.5, 0.5], [10.0; 4])),
                focus: Graphic::from(Panel::new(texture, [0.5, 0.5, 0.5, 0.5], [10.0; 4])),
            }),
            tab_button: Rc::new(TabStyle {
                hover: Graphic::from(Panel::new(tab_texture, [0.5, 0.0, 0.5, 0.5], [10.0; 4])),
                pressed: Graphic::from(Panel::new(tab_texture, [0.0, 0.5, 0.5, 0.5], [10.0; 4])),
                unselected: Graphic::from(Panel::new(tab_texture, [0.0, 0.0, 0.5, 0.5], [10.0; 4])),
                selected: Graphic::from(Panel::new(tab_texture, [0.5, 0.5, 0.5, 0.5], [10.0; 4])),
            }),
            popup_background: white.clone().with_color([0, 0, 0, 160]).into(),
            popup_header: white.clone().into(),
            popup_window: white.clone().with_color([200, 200, 200, 255]).into(),
            list_background: Panel::new(texture, [0.0, 0.0, 0.5, 0.5], [10.0; 4]).into(),
            page_background: white.clone().into(),
            scroll_background: Graphic::None,
            scroll_handle: Rc::new(ButtonStyle {
                normal: white.clone().with_color([80, 80, 80, 255]).into(),
                hover: white.clone().with_color([100, 100, 100, 255]).into(),
                pressed: white.with_color([120, 120, 120, 255]).into(),
                focus: Graphic::None,
            }),
        }
    }
}

enum UserEvent {
    Close,
}

struct NumberField<F: Fn(&mut Context, i32)>(F);
impl<F: Fn(&mut Context, i32)> TextFieldCallback for NumberField<F> {
    fn on_submit(&mut self, _this: Id, ctx: &mut Context, text: &mut String) -> bool {
        match text.parse::<i32>() {
            Ok(x) => {
                (self.0)(ctx, x);
                true
            }
            Err(_) => {
                if text.chars().any(|c| !c.is_whitespace()) {
                    false
                } else {
                    text.replace_range(.., "0");
                    (self.0)(ctx, 0);
                    true
                }
            }
        }
    }

    fn on_change(&mut self, _: Id, _: &mut Context, _: &String) {}

    fn on_unfocus(&mut self, this: Id, ctx: &mut Context, text: &mut String) -> bool {
        self.on_submit(this, ctx, text)
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
struct Options {
    field_a: String,
    field_b: i32,
    min: i32,
    max: i32,
    slider: i32,
    check_a: bool,
    check_b: bool,
    dropdown: usize,
    list: Vec<(u32, String)>,
    tab_selected: usize,
}
impl Options {
    fn load() -> Result<Self, String> {
        let file = File::open("options.json").map_err(|e| e.to_string())?;
        serde_json::from_reader(BufReader::new(file)).map_err(|e| e.to_string())
    }
    fn save(&self) {
        if let Ok(file) = File::create("options.json") {
            serde_json::to_writer_pretty(BufWriter::new(file), self).unwrap();
        }
    }
}
impl Default for Options {
    fn default() -> Self {
        Self {
            field_a: "Field".into(),
            field_b: 10,
            min: 0,
            max: 100,
            slider: 50,
            check_a: true,
            check_b: false,
            dropdown: 0,
            list: Vec::default(),
            tab_selected: 0,
        }
    }
}

#[derive(Clone)]
struct OptionsGUI {
    options: Rc<RefCell<Options>>,
    surface: Id,
    field_a: Id,
    field_b: Id,
    min: Id,
    max: Id,
    slider: Id,
    check_a: Id,
    check_b: Id,
    dropdown: Id,
    list: Id,
    tabs: [Id; 2],
    pages: [Id; 2],
    popup: Id,
    popup_title: Id,
    popup_text: Id,
}
#[allow(clippy::too_many_arguments)]
impl OptionsGUI {
    fn update_gui(&self, ctx: &mut Context, style: &StyleSheet) {
        // TODO: this is very error prone.
        // For example, I could try send a number to the text field,
        // and never know that it is doing nothing.
        ctx.send_event_to(
            self.field_a,
            SetValue(self.options.borrow().field_a.clone()),
        );
        ctx.send_event_to(
            self.field_b,
            SetValue(self.options.borrow().field_b.to_string()),
        );
        ctx.send_event_to(self.min, SetValue(self.options.borrow().min.to_string()));
        ctx.send_event_to(self.max, SetValue(self.options.borrow().max.to_string()));
        ctx.send_event_to(self.slider, SetValue(self.options.borrow().slider));
        ctx.send_event_to(self.check_a, SetValue(self.options.borrow().check_a));
        ctx.send_event_to(self.check_b, SetValue(self.options.borrow().check_b));
        ctx.send_event_to(self.dropdown, SetSelected(self.options.borrow().dropdown));

        for child in ctx.get_active_children(self.list) {
            ctx.remove(child);
        }

        for (i, text) in self.options.borrow().list.iter() {
            create_item(
                ctx,
                self.list,
                text.clone(),
                style.list_background.clone(),
                style.button.clone(),
                self.options.clone(),
                *i,
            );
        }

        ctx.send_event_to(self.tabs[self.options.borrow().tab_selected], Select);
    }

    fn new(
        gui: &mut GUI,
        options: Rc<RefCell<Options>>,
        proxy: EventLoopProxy<UserEvent>,
        style: &StyleSheet,
    ) -> Self {
        let this = Self {
            options,
            surface: gui.reserve_id(),
            field_a: gui.reserve_id(),
            field_b: gui.reserve_id(),
            min: gui.reserve_id(),
            max: gui.reserve_id(),
            slider: gui.reserve_id(),
            check_a: gui.reserve_id(),
            check_b: gui.reserve_id(),
            dropdown: gui.reserve_id(),
            list: gui.reserve_id(),
            tabs: [gui.reserve_id(), gui.reserve_id()],
            pages: [gui.reserve_id(), gui.reserve_id()],
            popup: gui.reserve_id(),
            popup_title: gui.reserve_id(),
            popup_text: gui.reserve_id(),
        };

        let surface = gui
            .create_control_reserved(this.surface)
            .layout(VBoxLayout::new(0.0, [0.0, 0.0, 0.0, 0.0], -1))
            .build();

        this.create_menu_bar(surface, gui, proxy, style);
        let tab_selected = this.options.borrow().tab_selected;
        this.create_tabs(surface, gui, style);
        let page0_cont = gui
            .create_control()
            .graphic(
                style
                    .menu
                    .button
                    .normal
                    .clone()
                    .with_color([230, 230, 230, 255]),
            )
            .layout(VBoxLayout::new(2.0, [5.0, 5.0, 5.0, 5.0], -1))
            .build();
        scroll_view(gui, this.pages[0], page0_cont, style)
            .expand_y(true)
            .parent(surface)
            .active(false)
            .build();
        this.create_page0(gui, page0_cont, style);
        let page1 = gui
            .create_control_reserved(this.pages[1])
            .graphic(
                style
                    .menu
                    .button
                    .normal
                    .clone()
                    .with_color([230, 230, 230, 255]),
            )
            .expand_y(true)
            .parent(surface)
            .active(false)
            .layout(VBoxLayout::new(10.0, [10.0; 4], -1))
            .build();

        this.create_page1(gui, page1, style);
        this.create_popup(gui, style);

        gui.send_event_to(this.tabs[tab_selected], Box::new(Select));

        this
    }

    fn create_page0(&self, gui: &mut GUI, parent: Id, style: &StyleSheet) {
        // I should avoid borrow RefCell in parameter position, because
        // the borrow is only dropped after the function is called.
        let initial_value = self.options.borrow().field_a.clone();
        self.create_text_field(
            self.field_a,
            gui,
            "String".into(),
            initial_value,
            parent,
            style,
            {
                let options = self.options.clone();
                move |_, _: &mut Context, text: &mut String| {
                    options.borrow_mut().field_a.clone_from(text);
                    true
                }
            },
        );
        let initial_value = self.options.borrow().field_b.to_string();
        self.create_text_field(
            self.field_b,
            gui,
            "Number".into(),
            initial_value,
            parent,
            style,
            {
                let options = self.options.clone();
                NumberField(move |_, x| options.borrow_mut().field_b = x)
            },
        );
        self.create_slider(gui, parent, style);
        let initial_value = self.options.borrow().check_a;
        self.create_check(
            gui,
            self.check_a,
            "Check A".into(),
            initial_value,
            parent,
            style,
            {
                let options = self.options.clone();
                move |_, _, x| options.borrow_mut().check_a = x
            },
        );
        let initial_value = self.options.borrow().check_a;
        self.create_check(
            gui,
            self.check_b,
            "Check B".into(),
            initial_value,
            parent,
            style,
            {
                let options = self.options.clone();
                move |_, _, x| options.borrow_mut().check_b = x
            },
        );
        self.create_dropdown(gui, parent, style);
    }

    fn create_page1(&self, gui: &mut GUI, parent: Id, style: &StyleSheet) {
        let list = self.list;
        OptionsGUI::text_field(gui.reserve_id(), gui, "".into(), style, {
            let button_style = style.button.clone();
            let background = style.list_background.clone();
            let options = self.options.clone();
            move |_this: Id, ctx: &mut Context, text: &mut String| {
                if !text.chars().any(|c| !c.is_whitespace()) {
                    return true;
                }
                let mut borrow = options.borrow_mut();
                let i = borrow.list.last().map_or(0, |x| x.0) + 1;
                borrow.list.push((i, text.clone()));
                create_item(
                    ctx,
                    list,
                    text.clone(),
                    background.clone(),
                    button_style.clone(),
                    options.clone(),
                    i,
                );
                text.clear();
                true
            }
        })
        .parent(parent)
        .min_size([0.0, 24.0])
        .build();
        let list = gui
            .create_control_reserved(list)
            .graphic(style.list_background.clone())
            .layout(VBoxLayout::new(2.0, [2.0; 4], -1))
            .build();
        let id = gui.reserve_id();
        scroll_view(gui, id, list, style)
            .expand_y(true)
            .parent(parent)
            .build();
        let mut ctx = gui.get_context();
        for (i, text) in self.options.borrow().list.iter() {
            create_item(
                &mut ctx,
                list,
                text.clone(),
                style.list_background.clone(),
                style.button.clone(),
                self.options.clone(),
                *i,
            );
        }
    }

    fn create_menu_bar(
        &self,
        surface: Id,
        gui: &mut GUI,
        proxy: EventLoopProxy<UserEvent>,
        style: &StyleSheet,
    ) -> Id {
        let menu = gui.reserve_id();
        let blocker = gui
            .create_control()
            .active(false)
            .margins([0.0, 20.0, 0.0, 0.0])
            .behaviour(Blocker::new(move |_, ctx| {
                ctx.send_event_to(menu, CloseMenu)
            }))
            .build();
        use Item::*;
        gui.create_control_reserved(menu)
            .graphic(style.menu.button.normal.clone())
            .behaviour(MenuBar::new(
                style.menu.clone(),
                blocker,
                vec![Rc::new(Menu::new(
                    "File".to_string(),
                    vec![
                        Button("Load Config".to_string(), {
                            let options = self.options.clone();
                            let this = self.clone();
                            let style = style.clone();
                            Box::new(move |_, ctx| match Options::load() {
                                Ok(x) => {
                                    options.borrow_mut().clone_from(&x);
                                    this.update_gui(ctx, &style);
                                }
                                Err(e) => {
                                    ctx.get_graphic_mut(this.popup_title)
                                        .set_text("Load Failed");
                                    ctx.get_graphic_mut(this.popup_text).set_text(&e);
                                    ctx.active(this.popup);
                                }
                            })
                        }),
                        Button("Save Config".to_string(), {
                            let options = self.options.clone();
                            Box::new(move |_, _| options.borrow().save())
                        }),
                        Separator,
                        Button(
                            "Close".to_string(),
                            Box::new(move |_, _| {
                                let _ = proxy.send_event(UserEvent::Close);
                            }),
                        ),
                    ],
                ))],
            ))
            .layout(HBoxLayout::new(0.0, [0.0; 4], -1))
            .parent(surface)
            .build()
    }

    fn create_tabs(&self, surface: Id, gui: &mut GUI, style: &StyleSheet) -> Id {
        let line = gui
            .create_control()
            .layout(HBoxLayout::new(0.0, [0.0; 4], -1))
            .parent(surface)
            .build();
        let tab_group = ButtonGroup::new({
            let options = self.options.clone();
            let tabs = self.tabs;
            move |selected, _| {
                options.borrow_mut().tab_selected =
                    tabs.iter().position(|x| *x == selected).unwrap();
            }
        });
        OptionsGUI::tab_button(
            self.tabs[0],
            gui,
            self.pages[0],
            false,
            "Page 1".into(),
            tab_group.clone(),
            style,
        )
        .parent(line)
        .build();
        OptionsGUI::tab_button(
            self.tabs[1],
            gui,
            self.pages[1],
            false,
            "Page 2".into(),
            tab_group.clone(),
            style,
        )
        .parent(line)
        .build();
        drop(tab_group);
        line
    }

    fn tab_button<'a>(
        id: Id,
        gui: &'a mut GUI,
        page: Id,
        selected: bool,
        label: String,
        tab_group: ButtonGroup,
        style: &StyleSheet,
    ) -> ControlBuilder<'a> {
        let button = id;
        let graphic = Text::new([40, 40, 100, 255], label, 16.0, (0, 0)).into();
        gui.create_control()
            .graphic(graphic)
            .parent(button)
            .layout(FitText)
            .build();
        gui.create_control_reserved(button)
            .layout(MarginLayout::new([2.0; 4]))
            .behaviour(TabButton::new(
                tab_group,
                page,
                selected,
                style.tab_button.clone(),
            ))
    }

    fn create_text_field<C: TextFieldCallback + 'static>(
        &self,
        id: Id,
        gui: &mut GUI,
        name: String,
        initial_value: String,
        parent: Id,
        style: &StyleSheet,
        callback: C,
    ) {
        let line = gui
            .create_control()
            .layout(HBoxLayout::new(2.0, [2.0, 2.0, 2.0, 2.0], -1))
            .parent(parent)
            .build();
        let _text = gui
            .create_control()
            .graphic(Text::new([0, 0, 0, 255], name, 18.0, (-1, 0)).into())
            .layout(FitText)
            .expand_x(true)
            .parent(line)
            .build();
        OptionsGUI::text_field(id, gui, initial_value, style, callback)
            .min_size([100.0, 24.0])
            .parent(line)
            .build();
    }

    fn create_slider(&self, gui: &mut GUI, parent: Id, style: &StyleSheet) {
        let min = self.options.borrow().min;
        let max = self.options.borrow().max;
        let initial_value = self.options.borrow().slider;

        let label = gui
            .create_control()
            .min_size([0.0, 24.0])
            .graphic(Text::new([0, 0, 0, 255], initial_value.to_string(), 18.0, (0, 0)).into())
            .parent(parent)
            .build();
        let line = gui
            .create_control()
            .layout(HBoxLayout::new(2.0, [2.0, 2.0, 2.0, 2.0], -1))
            .parent(parent)
            .build();

        let slider = self.slider;

        let _min_field = OptionsGUI::text_field(self.min, gui, min.to_string(), style, {
            let options = self.options.clone();
            NumberField(move |ctx, x| {
                ctx.send_event_to(slider, SetMinValue(x));
                options.borrow_mut().min = x;
            })
        })
        .min_size([50.0, 18.0])
        .parent(line)
        .build();
        let slider = OptionsGUI::slider(slider, gui, min, max, initial_value, style, {
            let options = self.options.clone();
            move |_, ctx: &mut Context, value: i32| {
                ctx.get_graphic_mut(label).set_text(&value.to_string());
                options.borrow_mut().slider = value;
            }
        })
        .expand_x(true)
        .parent(line)
        .build();
        let _max_field = OptionsGUI::text_field(self.max, gui, max.to_string(), style, {
            let options = self.options.clone();
            NumberField(move |ctx, x| {
                ctx.send_event_to(slider, SetMaxValue(x));
                options.borrow_mut().max = x;
            })
        })
        .min_size([50.0, 18.0])
        .parent(line)
        .build();
    }

    fn create_check<F: Fn(Id, &mut Context, bool) + 'static>(
        &self,
        gui: &mut GUI,
        id: Id,
        name: String,
        initial_value: bool,
        parent: Id,
        style: &StyleSheet,
        on_change: F,
    ) {
        let toggle = id;

        let background = {
            let graphic = style
                .menu
                .button
                .normal
                .clone()
                .with_color([200, 200, 200, 255]);
            gui.create_control()
                .anchors([0.0, 0.5, 0.0, 0.5])
                .margins([5.0, -10.0, 25.0, 10.0])
                .graphic(graphic)
                .parent(toggle)
                .build()
        };
        let marker = gui
            .create_control()
            .anchors([0.5, 0.5, 0.5, 0.5])
            .margins([-6.0, -6.0, 6.0, 6.0])
            .graphic(style.menu.button.normal.clone().with_color([0, 0, 0, 255]))
            .parent(background)
            .build();
        gui.create_control_reserved(id)
            .behaviour(Toggle::new(
                background,
                marker,
                initial_value,
                style.button.clone(),
                style.text_field.clone(),
                on_change,
            ))
            .min_size([0.0, 30.0])
            .parent(parent)
            .build();

        let graphic = Text::new([40, 40, 100, 255], name, 16.0, (-1, 0)).into();
        gui.create_control()
            .anchors([0.0, 0.0, 1.0, 1.0])
            .margins([30.0, 0.0, 0.0, 0.0])
            .graphic(graphic)
            .parent(toggle)
            .build();
    }

    fn create_dropdown(&self, gui: &mut GUI, parent: Id, style: &StyleSheet) {
        let line = gui
            .create_control()
            .layout(HBoxLayout::new(2.0, [2.0, 2.0, 2.0, 2.0], -1))
            .parent(parent)
            .build();
        let _text = gui
            .create_control()
            .graphic(Text::new([0, 0, 0, 255], "Dropdown".into(), 18.0, (-1, 0)).into())
            .layout(FitText)
            .expand_x(true)
            .parent(line)
            .build();
        let initial_value = self.options.borrow().dropdown;
        OptionsGUI::dropdown(
            self.dropdown,
            gui,
            vec!["Option A".into(), "Option B".into(), "Option C".into()],
            initial_value,
            style,
            {
                let options = self.options.clone();
                move |(index, _), _, _| {
                    options.borrow_mut().dropdown = index;
                }
            },
        )
        .min_size([100.0, 24.0])
        .parent(line)
        .expand_x(true)
        .build();
    }

    fn create_popup(&self, gui: &mut GUI, style: &StyleSheet) {
        let popup = gui
            .create_control_reserved(self.popup)
            .graphic(style.popup_background.clone())
            .active(false)
            .build();
        let popup_window = gui
            .create_control()
            .anchors([0.5, 0.5, 0.5, 0.5])
            .margins([-80.0, -60.0, 80.0, 60.0])
            .graphic(style.popup_window.clone())
            .layout(VBoxLayout::new(5.0, [0.0; 4], -1))
            .parent(self.popup)
            .build();
        let popup_header = gui
            .create_control()
            .graphic(style.popup_header.clone())
            .min_size([0.0, 20.0])
            .parent(popup_window)
            .build();
        gui.create_control_reserved(self.popup_title)
            .graphic(Text::new([0, 0, 0, 255], "PopUp Title".into(), 16.0, (-1, 0)).into())
            .parent(popup_header)
            .build();
        gui.create_control_reserved(self.popup_text)
            .graphic(
                Text::new(
                    [0, 0, 0, 255],
                    "Somthing has happend!".into(),
                    16.0,
                    (-1, 0),
                )
                .into(),
            )
            .expand_y(true)
            .parent(popup_window)
            .build();
        let button_area = gui
            .create_control()
            .min_size([75.0, 30.0])
            .parent(popup_window)
            .build();
        let ok_button = gui
            .create_control()
            .behaviour(Button::new(style.button.clone(), move |_, ctx| {
                ctx.deactive(popup)
            }))
            .min_size([75.0, 20.0])
            .fill_x(crui::RectFill::ShrinkCenter)
            .fill_y(crui::RectFill::ShrinkCenter)
            .parent(button_area)
            .build();
        let _ok_button_text = gui
            .create_control()
            .graphic(Text::new([0, 0, 0, 255], "Ok".into(), 14.0, (0, 0)).into())
            .parent(ok_button)
            .build();
    }

    fn text_field<'a, C: TextFieldCallback + 'static>(
        id: Id,
        gui: &'a mut GUI,
        initial_value: String,
        style: &StyleSheet,
        callback: C,
    ) -> ControlBuilder<'a> {
        let input_box = id;
        let caret = gui
            .create_control()
            .anchors([0.0, 0.0, 0.0, 0.0])
            .graphic(style.text_field.normal.clone().with_color([0, 0, 0, 255]))
            .parent(input_box)
            .build();
        let input_text = gui
            .create_control()
            .graphic(Text::new([0, 0, 0, 255], String::new(), 18.0, (-1, 0)).into())
            .parent(input_box)
            .build();
        gui.create_control_reserved(input_box)
            .behaviour(TextField::new(
                initial_value,
                caret,
                input_text,
                style.text_field.clone(),
                callback,
            ))
    }

    fn slider<'a, C: SliderCallback + 'static>(
        id: Id,
        gui: &'a mut GUI,
        min: i32,
        max: i32,
        initial_value: i32,
        style: &StyleSheet,
        callback: C,
    ) -> ControlBuilder<'a> {
        let slider = id;
        let slide_area = gui
            .create_control()
            .anchors([0.0, 0.5, 1.0, 0.5])
            .margins([10.0, -3.0, -10.0, 3.0])
            .graphic(
                style
                    .menu
                    .button
                    .normal
                    .clone()
                    .with_color([170, 170, 170, 255]),
            )
            .parent(slider)
            .build();
        let handle = gui
            .create_control()
            .anchors([0.5, 0.5, 0.5, 0.5])
            .margins([-3.0, -14.0, 3.0, 14.0])
            .graphic(
                style
                    .menu
                    .button
                    .normal
                    .clone()
                    .with_color([200, 200, 200, 255]),
            )
            .parent(slider)
            .build();
        gui.create_control_reserved(slider).behaviour(Slider::new(
            handle,
            slide_area,
            min,
            max,
            initial_value,
            style.text_field.clone(),
            callback,
        ))
    }

    fn dropdown<'a, F: Fn((usize, String), Id, &mut Context) + 'static>(
        id: Id,
        gui: &'a mut GUI,
        itens: Vec<String>,
        initial_value: usize,
        style: &StyleSheet,
        callback: F,
    ) -> ControlBuilder<'a> {
        let float_menu = {
            let menu = gui.reserve_id();
            let blocker = gui
                .create_control()
                .active(false)
                .behaviour(Blocker::new(move |_, ctx| {
                    ctx.send_event_to(menu, CloseMenu)
                }))
                .build();
            let menu = gui
                .create_control_reserved(menu)
                .active(false)
                .graphic(style.button.normal.clone())
                .behaviour(DropMenu::<String, _>::new(blocker, {
                    let menu_button_style = Rc::new(style.menu.button.clone());
                    move |data, this, ctx| {
                        let id = ctx
                            .create_control()
                            .behaviour(MenuItem::new(this, menu_button_style.clone()))
                            .layout(MarginLayout::new([4.0, 4.0, 4.0, 4.0]))
                            .parent(this)
                            // .min_size([10.0, 25.0])
                            .build();
                        let _text = ctx
                            .create_control()
                            .margins([10.0, 0.0, -10.0, 0.0])
                            .graphic(
                                Text::new([40, 40, 100, 255], data.to_string(), 16.0, (-1, 0))
                                    .into(),
                            )
                            .layout(FitText)
                            .parent(id)
                            .build();
                        id
                    }
                }))
                .layout(VBoxLayout::new(0.0, [1.0, 1.0, 1.0, 1.0], -1))
                .min_size([0.0, 80.0])
                .build();
            menu
        };
        let text = gui
            .create_control()
            .margins([10.0, 0.0, -10.0, 0.0])
            .graphic(
                Text::new(
                    [40, 40, 100, 255],
                    itens[initial_value].clone(),
                    16.0,
                    (-1, 0),
                )
                .into(),
            )
            .parent(id)
            .build();
        gui.create_control_reserved(id)
            .min_size([0.0, 25.0])
            .behaviour(Dropdown::new(
                itens,
                Some(initial_value),
                float_menu,
                move |selected, this, ctx| {
                    ctx.get_graphic_mut(text).set_text(&selected.1);
                    (callback)(selected, this, ctx);
                },
                style.button.clone(),
            ))
    }
}

fn create_item(
    ctx: &mut Context,
    list: Id,
    text: String,
    background: Graphic,
    button_style: Rc<ButtonStyle>,
    options: Rc<RefCell<Options>>,
    i: u32,
) {
    let item = ctx
        .create_control()
        .min_size([100.0, 35.0])
        .graphic(background)
        .parent(list)
        .layout(HBoxLayout::new(0.0, [5.0, 0.0, 5.0, 0.0], 0))
        .build();
    // TODO: there must be a better way of set the minsize of the control above,
    // instead of relying in a child.
    ctx.create_control().min_height(35.0).parent(item).build();
    let _text = ctx
        .create_control()
        .parent(item)
        .graphic(Text::new([0, 0, 0, 255], text, 16.0, (-1, 0)).into())
        .layout(FitText)
        .expand_x(true)
        .build();
    let _button = ctx
        .create_control()
        .parent(item)
        .behaviour(Button::new(button_style, move |_, ctx| {
            ctx.remove(item);
            let list = &mut options.borrow_mut().list;
            let i = list
                .binary_search_by_key(&i, |x| x.0)
                .expect("List desync from Options");
            list.remove(i);
        }))
        .min_size([15.0, 15.0])
        .fill_x(RectFill::ShrinkCenter)
        .fill_y(RectFill::ShrinkCenter)
        .build();
}

fn scroll_view<'a>(
    gui: &'a mut GUI,
    id: Id,
    content: Id,
    style: &StyleSheet,
) -> ControlBuilder<'a> {
    let scroll_view = id;
    let view = gui
        .create_control()
        .graphic(Graphic::None)
        .parent(scroll_view)
        .layout(ViewLayout::new(false, true))
        .build();
    let v_scroll_bar_handle = gui.reserve_id();
    let v_scroll_bar = gui
        .create_control()
        .min_size([5.0, 5.0])
        .graphic(style.scroll_background.clone())
        .parent(scroll_view)
        .behaviour(ScrollBar::new(
            v_scroll_bar_handle,
            scroll_view,
            true,
            style.scroll_handle.clone(),
        ))
        .build();
    let v_scroll_bar_handle = gui
        .create_control_reserved(v_scroll_bar_handle)
        .parent(v_scroll_bar)
        .build();

    gui.get_context().set_parent(content, view);

    let behaviour = ScrollView::new(
        view,
        content,
        None,
        Some((v_scroll_bar, v_scroll_bar_handle)),
    );
    let behaviour_layout = Rc::new(RefCell::new(behaviour));

    gui.create_control_reserved(scroll_view)
        .behaviour(behaviour_layout.clone())
        .layout(behaviour_layout)
}
