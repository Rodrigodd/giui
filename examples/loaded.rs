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
use ui_engine::{
    event::SetValue,
    layouts::{FitText, HBoxLayout, MarginLayout, VBoxLayout},
    render::{GUIRender, GUIRenderer, Graphic, Panel, Text, Texture},
    style::{ButtonStyle, MenuStyle, OnFocusStyle, TabStyle},
    widgets::{
        Blocker, Button, ButtonGroup, CloseMenu, DropMenu, Dropdown, Item, Menu, MenuBar, MenuItem,
        NoneLayout, ScrollBar, ScrollView, Select, SetMaxValue, SetMinValue, SetSelected, Slider,
        SliderCallback, TabButton, TextField, TextFieldCallback, Toggle,
    },
    Context, ControlBuilder, Id, RectFill, GUI,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
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
    let event_loop = EventLoop::with_user_event();
    let window = WindowBuilder::new().with_inner_size(PhysicalSize::new(250, 300));

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
        let data = image::open("D:/repos/rust/ui_engine/examples/panel.png").unwrap();
        let data = data.to_rgba8();
        render.new_texture(data.width(), data.height(), data.as_ref(), true)
    };
    let icon_texture = {
        let data = image::open("D:/repos/rust/ui_engine/examples/icons.png").unwrap();
        let data = data.to_rgba8();
        render.new_texture(data.width(), data.height(), data.as_ref(), true)
    };
    let tab_texture = {
        let data = image::open("D:/repos/rust/ui_engine/examples/tab.png").unwrap();
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
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::UserEvent(UserEvent::Close) => {
                options.borrow().save();
                *control = ControlFlow::Exit;
            }
            Event::WindowEvent { event, .. } => {
                if let WindowEvent::Resized(size) = event {
                    resize(&mut gui, &mut render, &mut camera, &size);
                }
            }
            Event::RedrawRequested(_) => {
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
                normal: Panel::new(texture, [0.0, 0.0, 0.5, 0.5], 10.0).into(),
                focus: Panel::new(texture, [0.5, 0.5, 0.5, 0.5], 10.0).into(),
            }
            .into(),
            button: Rc::new(ButtonStyle {
                normal: Graphic::from(Panel::new(texture, [0.0, 0.0, 0.5, 0.5], 10.0)),
                hover: Graphic::from(Panel::new(texture, [0.5, 0.0, 0.5, 0.5], 10.0)),
                pressed: Graphic::from(Panel::new(texture, [0.0, 0.5, 0.5, 0.5], 10.0)),
                focus: Graphic::from(Panel::new(texture, [0.5, 0.5, 0.5, 0.5], 10.0)),
            }),
            tab_button: Rc::new(TabStyle {
                hover: Graphic::from(Panel::new(tab_texture, [0.5, 0.0, 0.5, 0.5], 10.0)),
                pressed: Graphic::from(Panel::new(tab_texture, [0.0, 0.5, 0.5, 0.5], 10.0)),
                unselected: Graphic::from(Panel::new(tab_texture, [0.0, 0.0, 0.5, 0.5], 10.0)),
                selected: Graphic::from(Panel::new(tab_texture, [0.5, 0.5, 0.5, 0.5], 10.0)),
            }),
            popup_background: white.clone().with_color([0, 0, 0, 160]).into(),
            popup_header: white.clone().into(),
            popup_window: white.clone().with_color([200, 200, 200, 255]).into(),
            list_background: Panel::new(texture, [0.0, 0.0, 0.5, 0.5], 10.0).into(),
            page_background: white.clone().into(),
            scroll_background: Graphic::None,
            scroll_handle: Rc::new(ButtonStyle {
                normal: white.clone().with_color([80, 80, 80, 255]).into(),
                hover: white.clone().with_color([100, 100, 100, 255]).into(),
                pressed: white.clone().with_color([120, 120, 120, 255]).into(),
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

    fn on_change(&mut self, _: Id, _: &mut Context, _: &mut String) {}

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

        for child in ctx.get_children(self.list) {
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
            .create_control()
            .with_layout(VBoxLayout::new(0.0, [0.0, 0.0, 0.0, 0.0], -1))
            .build();

        this.create_menu_bar(surface, gui, proxy, style);
        let tab_selected = this.options.borrow().tab_selected;
        this.create_tabs(surface, gui, tab_selected, style);
        let page0_cont = gui
            .create_control()
            .with_graphic(
                style
                    .menu
                    .button
                    .normal
                    .clone()
                    .with_color([230, 230, 230, 255]),
            )
            .with_layout(VBoxLayout::new(2.0, [5.0, 5.0, 5.0, 5.0], -1))
            .build();
        scroll_view(gui, this.pages[0], page0_cont, style)
            .with_expand_y(true)
            .with_parent(surface)
            .with_active(false)
            .build();
        this.create_page0(gui, page0_cont, style);
        let page1 = gui
            .create_control_reserved(this.pages[1])
            .with_graphic(
                style
                    .menu
                    .button
                    .normal
                    .clone()
                    .with_color([230, 230, 230, 255]),
            )
            .with_expand_y(true)
            .with_parent(surface)
            .with_active(false)
            .with_layout(VBoxLayout::new(10.0, [10.0; 4], -1))
            .build();

        this.create_page1(gui, page1, style);
        this.create_popup(gui, style);

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
        .with_parent(parent)
        .with_min_size([0.0, 24.0])
        .build();
        let list = gui
            .create_control_reserved(list)
            .with_graphic(style.list_background.clone())
            .with_layout(VBoxLayout::new(2.0, [2.0; 4], -1))
            .build();
        let id = gui.reserve_id();
        scroll_view(gui, id, list, style)
            .with_expand_y(true)
            .with_parent(parent)
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
            .with_active(false)
            .with_margins([0.0, 20.0, 0.0, 0.0])
            .with_behaviour(Blocker::new(move |_, ctx| {
                ctx.send_event_to(menu, CloseMenu)
            }))
            .build();
        use Item::*;
        gui.create_control_reserved(menu)
            .with_graphic(style.menu.button.normal.clone())
            .with_behaviour(MenuBar::new(
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
            .with_layout(HBoxLayout::new(0.0, [0.0; 4], -1))
            .with_parent(surface)
            .build()
    }

    fn create_tabs(
        &self,
        surface: Id,
        gui: &mut GUI,
        tab_selected: usize,
        style: &StyleSheet,
    ) -> Id {
        let line = gui
            .create_control()
            .with_layout(HBoxLayout::new(0.0, [0.0; 4], -1))
            .with_parent(surface)
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
        .with_parent(line)
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
        .with_parent(line)
        .build();
        gui.send_event_to(self.tabs[tab_selected], Box::new(Select));
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
            .with_graphic(graphic)
            .with_parent(button)
            .with_layout(FitText)
            .build();
        gui.create_control_reserved(button)
            .with_layout(MarginLayout::new([2.0; 4]))
            .with_behaviour(TabButton::new(
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
            .with_layout(HBoxLayout::new(2.0, [2.0, 2.0, 2.0, 2.0], -1))
            .with_parent(parent)
            .build();
        let _text = gui
            .create_control()
            .with_graphic(Text::new([0, 0, 0, 255], name, 18.0, (-1, 0)).into())
            .with_layout(FitText)
            .with_expand_x(true)
            .with_parent(line)
            .build();
        OptionsGUI::text_field(id, gui, initial_value, style, callback)
            .with_min_size([100.0, 24.0])
            .with_parent(line)
            .build();
    }

    fn create_slider(&self, gui: &mut GUI, parent: Id, style: &StyleSheet) {
        let min = self.options.borrow().min;
        let max = self.options.borrow().max;
        let initial_value = self.options.borrow().slider;

        let label = gui
            .create_control()
            .with_min_size([0.0, 24.0])
            .with_graphic(Text::new([0, 0, 0, 255], initial_value.to_string(), 18.0, (0, 0)).into())
            .with_parent(parent)
            .build();
        let line = gui
            .create_control()
            .with_layout(HBoxLayout::new(2.0, [2.0, 2.0, 2.0, 2.0], -1))
            .with_parent(parent)
            .build();

        let slider = self.slider;

        let _min_field = OptionsGUI::text_field(self.min, gui, min.to_string(), style, {
            let options = self.options.clone();
            NumberField(move |ctx, x| {
                ctx.send_event_to(slider, SetMinValue(x));
                options.borrow_mut().min = x;
            })
        })
        .with_min_size([50.0, 18.0])
        .with_parent(line)
        .build();
        let slider = OptionsGUI::slider(slider, gui, min, max, initial_value, style, {
            let options = self.options.clone();
            move |_, ctx: &mut Context, value: i32| {
                ctx.get_graphic_mut(label).set_text(&value.to_string());
                options.borrow_mut().slider = value;
            }
        })
        .with_expand_x(true)
        .with_parent(line)
        .build();
        let _max_field = OptionsGUI::text_field(self.max, gui, max.to_string(), style, {
            let options = self.options.clone();
            NumberField(move |ctx, x| {
                ctx.send_event_to(slider, SetMaxValue(x));
                options.borrow_mut().max = x;
            })
        })
        .with_min_size([50.0, 18.0])
        .with_parent(line)
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
        let toggle = gui
            .create_control_reserved(id)
            .with_min_size([0.0, 30.0])
            .with_parent(parent)
            .build();

        let background = {
            let graphic = style
                .menu
                .button
                .normal
                .clone()
                .with_color([200, 200, 200, 255]);
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
            .with_graphic(style.menu.button.normal.clone().with_color([0, 0, 0, 255]))
            .with_parent(background)
            .build();
        gui.set_behaviour(
            toggle,
            Toggle::new(
                background,
                marker,
                initial_value,
                style.button.clone(),
                style.text_field.clone(),
                on_change,
            ),
        );

        let graphic = Text::new([40, 40, 100, 255], name, 16.0, (-1, 0)).into();
        gui.create_control()
            .with_anchors([0.0, 0.0, 1.0, 1.0])
            .with_margins([30.0, 0.0, 0.0, 0.0])
            .with_graphic(graphic)
            .with_parent(toggle)
            .build();
    }

    fn create_dropdown(&self, gui: &mut GUI, parent: Id, style: &StyleSheet) {
        let line = gui
            .create_control()
            .with_layout(HBoxLayout::new(2.0, [2.0, 2.0, 2.0, 2.0], -1))
            .with_parent(parent)
            .build();
        let _text = gui
            .create_control()
            .with_graphic(Text::new([0, 0, 0, 255], "Dropdown".into(), 18.0, (-1, 0)).into())
            .with_layout(FitText)
            .with_expand_x(true)
            .with_parent(line)
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
        .with_min_size([100.0, 24.0])
        .with_parent(line)
        .with_expand_x(true)
        .build();
    }

    fn create_popup(&self, gui: &mut GUI, style: &StyleSheet) {
        let popup = gui
            .create_control_reserved(self.popup)
            .with_graphic(style.popup_background.clone())
            .with_active(false)
            .build();
        let popup_window = gui
            .create_control()
            .with_anchors([0.5, 0.5, 0.5, 0.5])
            .with_margins([-80.0, -60.0, 80.0, 60.0])
            .with_graphic(style.popup_window.clone())
            .with_layout(VBoxLayout::new(5.0, [0.0; 4], -1))
            .with_parent(self.popup)
            .build();
        let popup_header = gui
            .create_control()
            .with_graphic(style.popup_header.clone())
            .with_min_size([0.0, 20.0])
            .with_parent(popup_window)
            .build();
        gui.create_control_reserved(self.popup_title)
            .with_graphic(Text::new([0, 0, 0, 255], "PopUp Title".into(), 16.0, (-1, 0)).into())
            .with_parent(popup_header)
            .build();
        gui.create_control_reserved(self.popup_text)
            .with_graphic(
                Text::new(
                    [0, 0, 0, 255],
                    "Somthing has happend!".into(),
                    16.0,
                    (-1, 0),
                )
                .into(),
            )
            .with_expand_y(true)
            .with_parent(popup_window)
            .build();
        let button_area = gui
            .create_control()
            .with_min_size([75.0, 30.0])
            .with_parent(popup_window)
            .build();
        let ok_button = gui
            .create_control()
            .with_behaviour(Button::new(style.button.clone(), move |_, ctx| {
                ctx.deactive(popup)
            }))
            .with_min_size([75.0, 20.0])
            .with_fill_x(ui_engine::RectFill::ShrinkCenter)
            .with_fill_y(ui_engine::RectFill::ShrinkCenter)
            .with_parent(button_area)
            .build();
        let _ok_button_text = gui
            .create_control()
            .with_graphic(Text::new([0, 0, 0, 255], "Ok".into(), 14.0, (0, 0)).into())
            .with_parent(ok_button)
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
            .with_anchors([0.0, 0.0, 0.0, 0.0])
            .with_graphic(style.text_field.normal.clone().with_color([0, 0, 0, 255]))
            .with_parent(input_box)
            .build();
        let input_text = gui
            .create_control()
            .with_graphic(Text::new([0, 0, 0, 255], String::new(), 18.0, (-1, 0)).into())
            .with_parent(input_box)
            .build();
        gui.create_control_reserved(input_box)
            .with_behaviour(TextField::new(
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
            .with_anchors([0.0, 0.5, 1.0, 0.5])
            .with_margins([10.0, -3.0, -10.0, 3.0])
            .with_graphic(
                style
                    .menu
                    .button
                    .normal
                    .clone()
                    .with_color([170, 170, 170, 255]),
            )
            .with_parent(slider)
            .build();
        let handle = gui
            .create_control()
            .with_anchors([0.5, 0.5, 0.5, 0.5])
            .with_margins([-3.0, -14.0, 3.0, 14.0])
            .with_graphic(
                style
                    .menu
                    .button
                    .normal
                    .clone()
                    .with_color([200, 200, 200, 255]),
            )
            .with_parent(slider)
            .build();
        gui.create_control_reserved(slider)
            .with_behaviour(Slider::new(
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
            let blocker = gui.create_control().with_active(false).build();
            let menu = gui
                .create_control()
                .with_active(false)
                .with_graphic(style.button.normal.clone())
                .with_behaviour(DropMenu::<String, _>::new(blocker, {
                    let menu_button_style = Rc::new(style.menu.button.clone());
                    move |data, this, ctx| {
                        let id = ctx
                            .create_control()
                            .with_behaviour(MenuItem::new(this, menu_button_style.clone()))
                            .with_layout(MarginLayout::new([4.0, 4.0, 4.0, 4.0]))
                            .with_parent(this)
                            // .with_min_size([10.0, 25.0])
                            .build();
                        let _text = ctx
                            .create_control()
                            .with_margins([10.0, 0.0, -10.0, 0.0])
                            .with_graphic(
                                Text::new([40, 40, 100, 255], data.to_string(), 16.0, (-1, 0))
                                    .into(),
                            )
                            .with_layout(FitText)
                            .with_parent(id)
                            .build();
                        id
                    }
                }))
                .with_layout(VBoxLayout::new(0.0, [1.0, 1.0, 1.0, 1.0], -1))
                .with_min_size([0.0, 80.0])
                .build();
            gui.set_behaviour(
                blocker,
                Blocker::new(move |_, ctx| ctx.send_event_to(menu, CloseMenu)),
            );
            menu
        };
        let text = gui
            .create_control()
            .with_margins([10.0, 0.0, -10.0, 0.0])
            .with_graphic(
                Text::new(
                    [40, 40, 100, 255],
                    itens[initial_value].clone(),
                    16.0,
                    (-1, 0),
                )
                .into(),
            )
            .with_parent(id)
            .build();
        gui.create_control_reserved(id)
            .with_min_size([0.0, 25.0])
            .with_behaviour(Dropdown::new(
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
        .with_min_size([100.0, 35.0])
        .with_graphic(background)
        .with_parent(list)
        .with_layout(HBoxLayout::new(0.0, [5.0, 0.0, 5.0, 0.0], 0))
        .build();
    // TODO: there must be a better way of set the minsize of the control above,
    // instead of relying in a child.
    ctx.create_control()
        .with_min_height(35.0)
        .with_parent(item)
        .build();
    let _text = ctx
        .create_control()
        .with_parent(item)
        .with_graphic(Text::new([0, 0, 0, 255], text, 16.0, (-1, 0)).into())
        .with_layout(FitText)
        .with_expand_x(true)
        .build();
    let _button = ctx
        .create_control()
        .with_parent(item)
        .with_behaviour(Button::new(button_style, move |_, ctx| {
            ctx.remove(item);
            let list = &mut options.borrow_mut().list;
            let i = list
                .binary_search_by_key(&i, |x| x.0)
                .expect("List desync from Options");
            list.remove(i);
        }))
        .with_min_size([15.0, 15.0])
        .with_fill_x(RectFill::ShrinkCenter)
        .with_fill_y(RectFill::ShrinkCenter)
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
        .with_graphic(Graphic::None)
        .with_parent(scroll_view)
        .with_layout(NoneLayout)
        .build();
    let h_scroll_bar = gui
        .create_control()
        .with_min_size([5.0, 5.0])
        .with_graphic(style.scroll_background.clone())
        .with_parent(scroll_view)
        .build();
    let h_scroll_bar_handle = gui.create_control().with_parent(h_scroll_bar).build();
    gui.set_behaviour(
        h_scroll_bar,
        ScrollBar::new(
            h_scroll_bar_handle,
            scroll_view,
            false,
            style.scroll_handle.clone(),
        ),
    );
    let v_scroll_bar = gui
        .create_control()
        .with_min_size([5.0, 5.0])
        .with_graphic(style.scroll_background.clone())
        .with_parent(scroll_view)
        .build();
    let v_scroll_bar_handle = gui.create_control().with_parent(v_scroll_bar).build();
    gui.set_behaviour(
        v_scroll_bar,
        ScrollBar::new(
            v_scroll_bar_handle,
            scroll_view,
            true,
            style.scroll_handle.clone(),
        ),
    );

    gui.get_context().set_parent(content, view);

    let behaviour = ScrollView::new(
        view,
        content,
        h_scroll_bar,
        h_scroll_bar_handle,
        v_scroll_bar,
        v_scroll_bar_handle,
    );
    let behaviour_layout = Rc::new(RefCell::new(behaviour));

    gui.create_control_reserved(scroll_view)
        .with_behaviour(behaviour_layout.clone())
        .with_layout(behaviour_layout)
}
