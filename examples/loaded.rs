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

use crui::{
    event::SetValue,
    graphics::{Graphic, Panel, Text, TextStyle, Texture},
    layouts::{FitText, HBoxLayout, MarginLayout, VBoxLayout},
    style::{ButtonStyle, MenuStyle, OnFocusStyle, SelectionColor, TabStyle, TextFieldStyle},
    widgets::{
        Blocker, Button, ButtonGroup, CloseMenu, DropMenu, Dropdown, Item, Menu, MenuBar, MenuItem,
        ScrollBar, ScrollView, Select, SetMaxValue, SetMinValue, SetSelected, Slider,
        SliderCallback, TabButton, TextField, TextFieldCallback, Toggle, ViewLayout,
    },
    BuilderContext, Color, Context, ControlBuilder, Gui, Id, RectFill,
};
use sprite_render::{GLSpriteRender, SpriteRender};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
};

mod common;
use common::{CruiEventLoop, MyFonts};

struct Loaded {
    options: Rc<RefCell<Options>>,
}
impl CruiEventLoop<UserEvent> for Loaded {
    fn init(
        gui: &mut Gui,
        render: &mut GLSpriteRender,
        fonts: MyFonts,
        event_loop: &EventLoop<UserEvent>,
    ) -> Self {
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
        let style_sheet = StyleSheet::new(texture, icon_texture, tab_texture, fonts);
        let options_gui = OptionsGui::new(
            gui,
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
        Self { options }
    }

    fn on_event(&mut self, event: &Event<UserEvent>, control: &mut ControlFlow) {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::UserEvent(UserEvent::Close) => {
                self.options.borrow().save();
                *control = ControlFlow::Exit;
            }
            _ => {}
        }
    }
}

fn main() {
    common::run::<UserEvent, Loaded>(250, 300);
}

#[derive(Clone)]
struct StyleSheet {
    // fonts: MyFonts,
    text_style: TextStyle,
    menu: Rc<MenuStyle>,
    text_field: Rc<TextFieldStyle>,
    on_focus: Rc<OnFocusStyle>,
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
    fn new(texture: u32, icon_texture: u32, tab_texture: u32, fonts: MyFonts) -> Self {
        let white = Texture::new(texture, [0.1, 0.1, 0.3, 0.3]);
        Self {
            // fonts: fonts.clone(),
            text_style: TextStyle {
                color: [40, 40, 100, 255].into(),
                font_size: 16.0,
                font_id: fonts.notosans,
                ..Default::default()
            },
            menu: MenuStyle {
                button: ButtonStyle {
                    normal: white.clone().into(),
                    hover: Texture::new(texture, [0.6, 0.1, 0.3, 0.3]).into(),
                    pressed: Texture::new(texture, [0.1, 0.6, 0.3, 0.3]).into(),
                    focus: Texture::new(texture, [0.5, 0.5, 0.001, 0.001]).into(),
                },
                arrow: Texture::new(icon_texture, [0.0, 0.0, 1.0, 1.0]).into(),
                separator: Texture::new(texture, [0.2, 0.2, 0.2, 0.2])
                    .with_color([180, 180, 180, 255].into())
                    .into(),
                text: TextStyle {
                    color: [0, 0, 0, 255].into(),
                    font_size: 16.0,
                    font_id: fonts.notosans,
                    ..Default::default()
                },
            }
            .into(),
            text_field: TextFieldStyle {
                background: OnFocusStyle {
                    normal: Panel::new(texture, [0.0, 0.0, 0.5, 0.5], [10.0; 4]).into(),
                    focus: Panel::new(texture, [0.5, 0.5, 0.5, 0.5], [10.0; 4]).into(),
                },
                caret_color: Color::BLACK,
                selection_color: SelectionColor {
                    bg: [170, 0, 255, 255].into(),
                    fg: Some(Color::WHITE),
                },
            }
            .into(),
            on_focus: OnFocusStyle {
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
            popup_background: white.clone().with_color([0, 0, 0, 160].into()).into(),
            popup_header: white.clone().into(),
            popup_window: white.clone().with_color([200, 200, 200, 255].into()).into(),
            list_background: Panel::new(texture, [0.0, 0.0, 0.5, 0.5], [10.0; 4]).into(),
            page_background: white.clone().into(),
            scroll_background: Graphic::None,
            scroll_handle: Rc::new(ButtonStyle {
                normal: white.clone().with_color([80, 80, 80, 255].into()).into(),
                hover: white.clone().with_color([100, 100, 100, 255].into()).into(),
                pressed: white.with_color([120, 120, 120, 255].into()).into(),
                focus: Graphic::None,
            }),
        }
    }
}

enum UserEvent {
    Close,
}

struct NumberField<F: Fn(&mut Context, i32)>(String, F);
impl<F: Fn(&mut Context, i32)> TextFieldCallback for NumberField<F> {
    fn on_submit(&mut self, _this: Id, ctx: &mut Context, text: &mut String) {
        match text.parse::<i32>() {
            Ok(x) => {
                (self.1)(ctx, x);
                self.0.clone_from(text);
            }
            Err(_) => {
                if text.chars().any(|c| !c.is_whitespace()) {
                    text.clone_from(&self.0);
                } else {
                    text.replace_range(.., "0");
                    (self.1)(ctx, 0);
                    self.0.clone_from(text);
                }
            }
        }
    }

    fn on_change(&mut self, _: Id, _: &mut Context, _: &str) {}

    fn on_unfocus(&mut self, this: Id, ctx: &mut Context, text: &mut String) {
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
struct OptionsGui {
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
impl OptionsGui {
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
                style.text_style.clone(),
                style.button.clone(),
                self.options.clone(),
                *i,
            );
        }

        ctx.send_event_to(self.tabs[self.options.borrow().tab_selected], Select);
    }

    fn new(
        gui: &mut Gui,
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
            .build(gui);

        this.create_menu_bar(surface, gui, proxy, style);
        let tab_selected = this.options.borrow().tab_selected;
        this.create_tabs(surface, gui, style);
        let page0_cont = gui.reserve_id();
        scroll_view(
            gui,
            this.pages[0],
            page0_cont,
            |cb, _| {
                cb.graphic(
                    style
                        .menu
                        .button
                        .normal
                        .clone()
                        .with_color([230, 230, 230, 255].into()),
                )
                .layout(VBoxLayout::new(2.0, [5.0, 5.0, 5.0, 5.0], -1))
            },
            style,
        )
        .expand_y(true)
        .parent(surface)
        .active(false)
        .build(gui);
        this.create_page0(gui, page0_cont, style);
        let page1 = gui
            .create_control_reserved(this.pages[1])
            .graphic(
                style
                    .menu
                    .button
                    .normal
                    .clone()
                    .with_color([230, 230, 230, 255].into()),
            )
            .expand_y(true)
            .parent(surface)
            .active(false)
            .layout(VBoxLayout::new(10.0, [10.0; 4], -1))
            .build(gui);

        this.create_page1(gui, page1, style);
        this.create_popup(gui, style);

        gui.send_event_to(this.tabs[tab_selected], Box::new(Select));

        this
    }

    fn create_page0(&self, gui: &mut Gui, parent: Id, style: &StyleSheet) {
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
                }
            },
        );
        let initial_value = self.options.borrow().field_b.to_string();
        self.create_text_field(
            self.field_b,
            gui,
            "Number".into(),
            initial_value.clone(),
            parent,
            style,
            {
                let options = self.options.clone();
                NumberField(initial_value, move |_, x| options.borrow_mut().field_b = x)
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

    fn create_page1(&self, gui: &mut Gui, parent: Id, style: &StyleSheet) {
        let list = self.list;
        OptionsGui::text_field(gui.reserve_id(), gui, "".into(), style, {
            let button_style = style.button.clone();
            let background = style.list_background.clone();
            let options = self.options.clone();
            let text_style = style.text_style.clone();
            move |_this: Id, ctx: &mut Context, text: &mut String| {
                if !text.chars().any(|c| !c.is_whitespace()) {
                    return;
                }
                let mut borrow = options.borrow_mut();
                let i = borrow.list.last().map_or(0, |x| x.0) + 1;
                borrow.list.push((i, text.clone()));
                create_item(
                    ctx,
                    list,
                    text.clone(),
                    background.clone(),
                    text_style.clone(),
                    button_style.clone(),
                    options.clone(),
                    i,
                );
                text.clear();
            }
        })
        .parent(parent)
        .min_size([0.0, 24.0])
        .build(gui);
        let id = gui.reserve_id();
        scroll_view(
            gui,
            id,
            list,
            |cb, _| {
                cb.graphic(style.list_background.clone())
                    .layout(VBoxLayout::new(2.0, [2.0; 4], -1))
            },
            style,
        )
        .expand_y(true)
        .parent(parent)
        .build(gui);
        let mut ctx = gui.get_context();
        for (i, text) in self.options.borrow().list.iter() {
            create_item(
                &mut ctx,
                list,
                text.clone(),
                style.list_background.clone(),
                style.text_style.clone(),
                style.button.clone(),
                self.options.clone(),
                *i,
            );
        }
    }

    fn create_menu_bar(
        &self,
        surface: Id,
        gui: &mut Gui,
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
            .build(gui);
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
            .build(gui)
    }

    fn create_tabs(&self, surface: Id, gui: &mut Gui, style: &StyleSheet) -> Id {
        let line = gui
            .create_control()
            .layout(HBoxLayout::new(0.0, [0.0; 4], -1))
            .parent(surface)
            .build(gui);
        let tab_group = ButtonGroup::new({
            let options = self.options.clone();
            let tabs = self.tabs;
            move |selected, _| {
                options.borrow_mut().tab_selected =
                    tabs.iter().position(|x| *x == selected).unwrap();
            }
        });
        OptionsGui::tab_button(
            self.tabs[0],
            gui,
            self.pages[0],
            false,
            "Page 1".into(),
            tab_group.clone(),
            style,
        )
        .parent(line)
        .build(gui);
        OptionsGui::tab_button(
            self.tabs[1],
            gui,
            self.pages[1],
            false,
            "Page 2".into(),
            tab_group.clone(),
            style,
        )
        .parent(line)
        .build(gui);
        drop(tab_group);
        line
    }

    fn tab_button<'a>(
        id: Id,
        gui: &'a mut Gui,
        page: Id,
        selected: bool,
        label: String,
        tab_group: ButtonGroup,
        style: &StyleSheet,
    ) -> ControlBuilder {
        let button = id;
        let graphic = Text::new(label, (0, 0), style.text_style.clone());
        gui.create_control()
            .graphic(graphic)
            .parent(button)
            .layout(FitText)
            .build(gui);
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
        gui: &mut Gui,
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
            .build(gui);
        let _text = gui
            .create_control()
            .graphic(Text::new(name, (-1, 0), style.text_style.clone()))
            .layout(FitText)
            .expand_x(true)
            .parent(line)
            .build(gui);
        OptionsGui::text_field(id, gui, initial_value, style, callback)
            .min_size([100.0, 24.0])
            .parent(line)
            .build(gui);
    }

    fn create_slider(&self, gui: &mut Gui, parent: Id, style: &StyleSheet) {
        let min = self.options.borrow().min;
        let max = self.options.borrow().max;
        let initial_value = self.options.borrow().slider;

        let label = gui
            .create_control()
            .min_size([0.0, 24.0])
            .graphic(Text::new(
                initial_value.to_string(),
                (0, 0),
                style.text_style.clone(),
            ))
            .parent(parent)
            .build(gui);
        let line = gui
            .create_control()
            .layout(HBoxLayout::new(2.0, [2.0, 2.0, 2.0, 2.0], -1))
            .parent(parent)
            .build(gui);

        let slider = self.slider;

        let _min_field = OptionsGui::text_field(self.min, gui, min.to_string(), style, {
            let options = self.options.clone();
            NumberField(min.to_string(), move |ctx, x| {
                ctx.send_event_to(slider, SetMinValue(x));
                options.borrow_mut().min = x;
            })
        })
        .min_size([50.0, 18.0])
        .parent(line)
        .build(gui);
        let slider = OptionsGui::slider(slider, gui, min, max, initial_value, style, {
            let options = self.options.clone();
            move |_, ctx: &mut Context, value: i32| {
                ctx.get_graphic_mut(label).set_text(&value.to_string());
                options.borrow_mut().slider = value;
            }
        })
        .expand_x(true)
        .parent(line)
        .build(gui);
        let _max_field = OptionsGui::text_field(self.max, gui, max.to_string(), style, {
            let options = self.options.clone();
            NumberField(max.to_string(), move |ctx, x| {
                ctx.send_event_to(slider, SetMaxValue(x));
                options.borrow_mut().max = x;
            })
        })
        .min_size([50.0, 18.0])
        .parent(line)
        .build(gui);
    }

    fn create_check<F: Fn(Id, &mut Context, bool) + 'static>(
        &self,
        gui: &mut Gui,
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
                .with_color([200, 200, 200, 255].into());
            gui.create_control()
                .anchors([0.0, 0.5, 0.0, 0.5])
                .margins([5.0, -10.0, 25.0, 10.0])
                .graphic(graphic)
                .parent(toggle)
                .build(gui)
        };
        let marker = gui
            .create_control()
            .anchors([0.5, 0.5, 0.5, 0.5])
            .margins([-6.0, -6.0, 6.0, 6.0])
            .graphic(
                style
                    .menu
                    .button
                    .normal
                    .clone()
                    .with_color([0, 0, 0, 255].into()),
            )
            .parent(background)
            .build(gui);
        gui.create_control_reserved(id)
            .behaviour(Toggle::new(
                background,
                marker,
                initial_value,
                style.button.clone(),
                style.on_focus.clone(),
                on_change,
            ))
            .min_size([0.0, 30.0])
            .parent(parent)
            .build(gui);

        let graphic = Text::new(name, (-1, 0), style.text_style.clone());
        gui.create_control()
            .anchors([0.0, 0.0, 1.0, 1.0])
            .margins([30.0, 0.0, 0.0, 0.0])
            .graphic(graphic)
            .parent(toggle)
            .build(gui);
    }

    fn create_dropdown(&self, gui: &mut Gui, parent: Id, style: &StyleSheet) {
        let line = gui
            .create_control()
            .layout(HBoxLayout::new(2.0, [2.0, 2.0, 2.0, 2.0], -1))
            .parent(parent)
            .build(gui);
        let _text = gui
            .create_control()
            .graphic(Text::new(
                "Dropdown".into(),
                (-1, 0),
                style.text_style.clone(),
            ))
            .layout(FitText)
            .expand_x(true)
            .parent(line)
            .build(gui);
        let initial_value = self.options.borrow().dropdown;
        OptionsGui::dropdown(
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
        .build(gui);
    }

    fn create_popup(&self, gui: &mut Gui, style: &StyleSheet) {
        let popup = gui
            .create_control_reserved(self.popup)
            .graphic(style.popup_background.clone())
            .active(false)
            .build(gui);
        let popup_window = gui
            .create_control()
            .anchors([0.5, 0.5, 0.5, 0.5])
            .margins([-80.0, -60.0, 80.0, 60.0])
            .graphic(style.popup_window.clone())
            .layout(VBoxLayout::new(5.0, [0.0; 4], -1))
            .parent(self.popup)
            .build(gui);
        let popup_header = gui
            .create_control()
            .graphic(style.popup_header.clone())
            .min_size([0.0, 20.0])
            .parent(popup_window)
            .build(gui);
        gui.create_control_reserved(self.popup_title)
            .graphic(Text::new(
                "PopUp Title".into(),
                (-1, 0),
                style.text_style.clone(),
            ))
            .parent(popup_header)
            .build(gui);
        gui.create_control_reserved(self.popup_text)
            .graphic(Text::new(
                "Somthing has happend!".into(),
                (-1, 0),
                style.text_style.clone(),
            ))
            .expand_y(true)
            .parent(popup_window)
            .build(gui);
        let button_area = gui
            .create_control()
            .min_size([75.0, 30.0])
            .parent(popup_window)
            .build(gui);
        let ok_button = gui
            .create_control()
            .behaviour(Button::new(style.button.clone(), true, move |_, ctx| {
                ctx.deactive(popup)
            }))
            .min_size([75.0, 20.0])
            .fill_x(crui::RectFill::ShrinkCenter)
            .fill_y(crui::RectFill::ShrinkCenter)
            .parent(button_area)
            .build(gui);
        let _ok_button_text = gui
            .create_control()
            .graphic(Text::new("Ok".into(), (0, 0), style.text_style.clone()))
            .parent(ok_button)
            .build(gui);
    }

    fn text_field<'a, C: TextFieldCallback + 'static>(
        id: Id,
        gui: &'a mut Gui,
        initial_value: String,
        style: &StyleSheet,
        callback: C,
    ) -> ControlBuilder {
        let input_box = id;
        let caret = gui
            .create_control()
            .anchors([0.0, 0.0, 0.0, 0.0])
            .graphic(
                style
                    .on_focus
                    .normal
                    .clone()
                    .with_color([0, 0, 0, 255].into()),
            )
            .parent(input_box)
            .build(gui);
        let input_text = gui
            .create_control()
            .graphic(Text::new(initial_value, (-1, 0), style.text_style.clone()))
            .parent(input_box)
            .build(gui);
        gui.create_control_reserved(input_box)
            .behaviour(TextField::new(
                caret,
                input_text,
                style.text_field.clone(),
                callback,
            ))
    }

    fn slider<'a, C: SliderCallback + 'static>(
        id: Id,
        gui: &'a mut Gui,
        min: i32,
        max: i32,
        initial_value: i32,
        style: &StyleSheet,
        callback: C,
    ) -> ControlBuilder {
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
                    .with_color([170, 170, 170, 255].into()),
            )
            .parent(slider)
            .build(gui);
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
                    .with_color([200, 200, 200, 255].into()),
            )
            .parent(slider)
            .build(gui);
        gui.create_control_reserved(slider).behaviour(Slider::new(
            handle,
            slide_area,
            min,
            max,
            initial_value,
            style.on_focus.clone(),
            callback,
        ))
    }

    fn dropdown<'a, F: Fn((usize, String), Id, &mut Context) + 'static>(
        id: Id,
        gui: &'a mut Gui,
        itens: Vec<String>,
        initial_value: usize,
        style: &StyleSheet,
        callback: F,
    ) -> ControlBuilder {
        let float_menu = {
            let menu = gui.reserve_id();
            let blocker = gui
                .create_control()
                .active(false)
                .behaviour(Blocker::new(move |_, ctx| {
                    ctx.send_event_to(menu, CloseMenu)
                }))
                .build(gui);
            let menu = gui
                .create_control_reserved(menu)
                .active(false)
                .graphic(style.button.normal.clone())
                .behaviour(DropMenu::<String, _>::new(blocker, {
                    let menu_button_style = Rc::new(style.menu.button.clone());
                    let text_style = style.text_style.clone();
                    move |data, this, ctx| {
                        let id = ctx
                            .create_control()
                            .behaviour(MenuItem::new(this, menu_button_style.clone()))
                            .layout(MarginLayout::new([4.0, 4.0, 4.0, 4.0]))
                            .parent(this)
                            // .min_size([10.0, 25.0])
                            .build(ctx);
                        let _text = ctx
                            .create_control()
                            .margins([10.0, 0.0, -10.0, 0.0])
                            .graphic(Text::new(data.to_string(), (-1, 0), text_style.clone()))
                            .layout(FitText)
                            .parent(id)
                            .build(ctx);
                        id
                    }
                }))
                .layout(VBoxLayout::new(0.0, [1.0, 1.0, 1.0, 1.0], -1))
                .min_size([0.0, 80.0])
                .build(gui);
            menu
        };
        let text = gui
            .create_control()
            .margins([10.0, 0.0, -10.0, 0.0])
            .graphic(Text::new(
                itens[initial_value].clone(),
                (-1, 0),
                style.text_style.clone(),
            ))
            .parent(id)
            .build(gui);
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

#[allow(clippy::too_many_arguments)]
fn create_item(
    ctx: &mut Context,
    list: Id,
    text: String,
    background: Graphic,
    text_style: TextStyle,
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
        .build(ctx);
    // TODO: there must be a better way of set the minsize of the control above,
    // instead of relying in a child.
    ctx.create_control()
        .min_height(35.0)
        .parent(item)
        .build(ctx);
    let _text = ctx
        .create_control()
        .parent(item)
        .graphic(Text::new(text, (-1, 0), text_style))
        .layout(FitText)
        .expand_x(true)
        .build(ctx);
    let _button = ctx
        .create_control()
        .parent(item)
        .behaviour(Button::new(button_style, true, move |_, ctx| {
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
        .build(ctx);
}

fn scroll_view<'a>(
    gui: &'a mut Gui,
    id: Id,
    content: Id,
    content_builder: impl for<'b> FnOnce(ControlBuilder, &mut dyn BuilderContext) -> ControlBuilder,
    style: &StyleSheet,
) -> ControlBuilder {
    let scroll_view = id;
    let view = gui
        .create_control()
        .graphic(Graphic::None)
        .parent(scroll_view)
        .layout(ViewLayout::new(false, true))
        .build(gui);
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
        .build(gui);
    let v_scroll_bar_handle = gui
        .create_control_reserved(v_scroll_bar_handle)
        .parent(v_scroll_bar)
        .build(gui);

    // gui.get_context().set_parent(content, view);

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
        .child_reserved(content, gui, content_builder)
}
