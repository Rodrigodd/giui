#![allow(clippy::useless_vec)]

use std::rc::Rc;

use crui::{
    font::FontId,
    graphics::{Graphic, Icon, Panel, Text, TextStyle, Texture},
    layouts::{FitText, GridLayout, HBoxLayout, MarginLayout, RatioLayout, VBoxLayout},
    style::{ButtonStyle, MenuStyle, OnFocusStyle, TabStyle, TextFieldStyle},
    widgets::{
        self, Blocker, Button, ButtonGroup, CloseMenu, ContextMenu, DropMenu, Dropdown, Hoverable,
        Item, Menu, MenuBar, MenuItem, ScrollBar, ScrollView, Slider, TabButton, TextField, Toggle,
        ViewLayout,
    },
    Color, Context, ControlBuilder, Gui, Id, RectFill,
};
use sprite_render::{GLSpriteRender, SpriteRender};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
};

mod common;
use common::MyFonts;

fn main() {
    common::run::<(), Main>(800, 600);
}

struct Main;
impl common::CruiEventLoop<()> for Main {
    fn init(
        gui: &mut Gui,
        render: &mut GLSpriteRender,
        fonts: MyFonts,
        event_loop: &EventLoop<()>,
    ) -> Self {
        let texture = {
            let data = image::open("D:/repos/rust/crui/examples/panel.png").unwrap();
            let data = data.to_rgba8();
            render.new_texture(data.width(), data.height(), data.as_ref(), true)
        };
        let tab_texture = {
            let data = image::open("D:/repos/rust/crui/examples/tab.png").unwrap();
            let data = data.to_rgba8();
            render.new_texture(data.width(), data.height(), data.as_ref(), true)
        };
        let icon_texture = {
            let data = image::open("D:/repos/rust/crui/examples/icons.png").unwrap();
            let data = data.to_rgba8();
            render.new_texture(data.width(), data.height(), data.as_ref(), true)
        };
        let marker_texture = {
            let data = image::open("D:/repos/rust/crui/examples/check.png").unwrap();
            let data = data.to_rgba8();
            render.new_texture(data.width(), data.height(), data.as_ref(), true)
        };

        let painel: Graphic = Panel::new(texture, [0.0, 0.0, 0.5, 0.5], [10.0; 4]).into();
        let white: Graphic = Texture::new(texture, [0.2, 0.2, 0.2, 0.2]).into();
        let page_painel: Graphic = Panel::new(texture, [0.0, 0.1, 0.5, 0.4], [10.0; 4]).into();
        let marker_icon: Graphic =
            Icon::new(marker_texture, [0.0, 0.0, 1.0, 1.0], [18.0; 2]).into();
        // let marker_icon: Graphic = Texture::new(marker_texture, [0.0, 0.0, 1.0, 1.0]).into();

        let button_style = Rc::new(ButtonStyle {
            normal: Graphic::from(Panel::new(texture, [0.0, 0.0, 0.5, 0.5], [10.0; 4])),
            hover: Graphic::from(Panel::new(texture, [0.5, 0.0, 0.5, 0.5], [10.0; 4])),
            pressed: Graphic::from(Panel::new(texture, [0.0, 0.5, 0.5, 0.5], [10.0; 4])),
            focus: Graphic::from(Panel::new(texture, [0.5, 0.5, 0.5, 0.5], [10.0; 4])),
        });
        let menu_button_style = Rc::new(ButtonStyle {
            normal: Graphic::from(Texture::new(texture, [0.1, 0.1, 0.3, 0.3])),
            hover: Graphic::from(Texture::new(texture, [0.6, 0.1, 0.3, 0.3])),
            pressed: Graphic::from(Texture::new(texture, [0.1, 0.6, 0.3, 0.3])),
            focus: Graphic::from(Texture::new(texture, [0.5, 0.5, 0.001, 0.001])),
        });
        let menu_style = Rc::new(MenuStyle {
            button: (*menu_button_style).clone(),
            arrow: Texture::new(icon_texture, [0.0, 0.0, 1.0, 1.0]).into(),
            separator: Texture::new(texture, [0.2, 0.2, 0.2, 0.2])
                .with_color([180, 180, 180, 255].into())
                .into(),
            text: TextStyle {
                color: [0, 0, 0, 255].into(),
                font_size: 16.0,
                font_id: fonts.notosans,
            },
        });
        let tab_style = Rc::new(TabStyle {
            hover: Graphic::from(Panel::new(tab_texture, [0.5, 0.0, 0.5, 0.5], [10.0; 4])),
            pressed: Graphic::from(Panel::new(tab_texture, [0.0, 0.5, 0.5, 0.5], [10.0; 4])),
            unselected: Graphic::from(Panel::new(tab_texture, [0.0, 0.0, 0.5, 0.5], [10.0; 4])),
            selected: Graphic::from(Panel::new(tab_texture, [0.5, 0.5, 0.5, 0.5], [10.0; 4])),
        });
        let focus_style = Rc::new(OnFocusStyle {
            normal: Graphic::None,
            focus: button_style.focus.clone(),
        });
        let close_button = Rc::new(ButtonStyle {
            normal: painel.clone().with_color([255, 0, 0, 255].into()),
            hover: painel.clone().with_color([240, 0, 0, 255].into()),
            pressed: painel.clone().with_color([230, 0, 0, 255].into()),
            focus: painel.clone().with_color([255, 0, 0, 255].into()),
        });

        let style = Style {
            painel,
            white,
            page_painel,
            marker_icon,
            button_style,
            menu_button_style,
            menu_style,
            tab_style,
            focus_style,
            close_button,
            fonts,
        };

        let proxy = event_loop.create_proxy();

        build_gui(gui, proxy, style);

        Main
    }

    fn on_event(&mut self, event: &Event<()>, control: &mut ControlFlow) {
        if let Event::UserEvent(()) = event {
            *control = ControlFlow::Exit;
        }
    }
}

struct Style {
    painel: Graphic,
    white: Graphic,
    page_painel: Graphic,
    marker_icon: Graphic,
    button_style: Rc<ButtonStyle>,
    menu_button_style: Rc<ButtonStyle>,
    menu_style: Rc<MenuStyle>,
    tab_style: Rc<TabStyle>,
    focus_style: Rc<OnFocusStyle>,
    close_button: Rc<ButtonStyle>,
    fonts: MyFonts,
}

fn build_gui(gui: &mut Gui, proxy: EventLoopProxy<()>, style: Style) {
    let (hover, hover_label) = {
        let graphic = style.white.clone().with_color([50, 50, 50, 255].into());
        let hover = gui
            .create_control()
            .anchors([0.0, 0.0, 0.0, 0.0])
            .graphic(graphic)
            .margins([3.0, 6.0, 6.0, 9.0])
            .layout(MarginLayout::new([3.0, 3.0, 3.0, 3.0]))
            .build();
        let graphic = Text::new(
            "This is a Hover".to_owned(),
            (-1, 0),
            TextStyle {
                color: [255, 255, 255, 255].into(),
                font_size: 12.0,
                font_id: style.fonts.notosans,
            },
        )
        .into();
        let label = gui
            .create_control()
            .graphic(graphic)
            .parent(hover)
            .layout(FitText)
            .build();

        (hover, label)
    };

    let surface = gui
        .create_control()
        .layout(VBoxLayout::new(0.0, [0.0; 4], -1))
        .build();
    let _menubar = {
        let menu = gui.reserve_id();
        // TODO: the menu_bar's blocker espect that I know the size of menu_bar
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
            .graphic(style.menu_button_style.normal.clone())
            .behaviour(MenuBar::new(
                style.menu_style.clone(),
                blocker,
                vec![
                    Rc::new(Menu::new(
                        "File".to_string(),
                        vec![
                            Button(
                                "Open".to_string(),
                                Box::new(move |_, _| println!("Click on 'Open'")),
                            ),
                            Button(
                                "About".to_string(),
                                Box::new(move |_, _| println!("Click on 'About'")),
                            ),
                            Separator,
                            Button(
                                "Close".to_string(),
                                Box::new(move |_, _| {
                                    let _ = proxy.send_event(());
                                }),
                            ),
                        ],
                    )),
                    Rc::new(Menu::new(
                        "Edit".to_string(),
                        vec![
                            SubMenu(Rc::new(Menu::new(
                                "Submenu".to_string(),
                                vec![
                                    Button(
                                        "Open".to_string(),
                                        Box::new(move |_, _| println!("Click on 'Open'")),
                                    ),
                                    Button(
                                        "About".to_string(),
                                        Box::new(move |_, _| println!("Click on 'About'")),
                                    ),
                                    Separator,
                                    SubMenu(Rc::new(Menu::new(
                                        "SubSubmenu".to_string(),
                                        vec![
                                            Button(
                                                "Open".to_string(),
                                                Box::new(move |_, _| println!("Click on 'Open'")),
                                            ),
                                            Button(
                                                "About".to_string(),
                                                Box::new(move |_, _| println!("Click on 'About'")),
                                            ),
                                        ],
                                    ))),
                                ],
                            ))),
                            Separator,
                            Button(
                                "Undo".to_string(),
                                Box::new(move |_, _| println!("Click on 'Undo'")),
                            ),
                            Button(
                                "Redo".to_string(),
                                Box::new(move |_, _| println!("Click on 'Redo'")),
                            ),
                            Separator,
                            Button(
                                "Copy".to_string(),
                                Box::new(move |_, _| println!("Click on 'Copy'")),
                            ),
                            Button(
                                "Paste".to_string(),
                                Box::new(move |_, _| println!("Click on 'Paste'")),
                            ),
                            Button(
                                "Cut".to_string(),
                                Box::new(move |_, _| println!("Click on 'Cut'")),
                            ),
                        ],
                    )),
                    Rc::new(Menu::new(
                        "Help".to_string(),
                        vec![
                            Button(
                                "Please".to_string(),
                                Box::new(move |_, _| println!("Click on 'Please'")),
                            ),
                            Button(
                                "Help".to_string(),
                                Box::new(move |_, _| println!("Click on 'Help'")),
                            ),
                            Button(
                                "Me".to_string(),
                                Box::new(move |_, _| println!("Click on 'Me'")),
                            ),
                        ],
                    )),
                ],
            ))
            .layout(HBoxLayout::new(0.0, [0.0; 4], -1))
            .parent(surface)
            .build()
    };

    let header = gui
        .create_control()
        .layout(HBoxLayout::new(0.0, [0.0, 0.0, 0.0, 0.0], -1))
        .parent(surface)
        .build();

    let page_area = gui
        .create_control()
        .margins([0.0, 45.0, 0.0, 0.0])
        .parent(surface)
        .expand_y(true)
        .build();
    let page_1 = {
        let page_1 = gui.create_control().parent(page_area).build();
        let menu = gui
            .create_control()
            .anchors([0.0, 0.0, 0.0, 1.0])
            .margins([0.0, 0.0, 200.0, 0.0])
            .graphic(style.page_painel.clone())
            .layout(VBoxLayout::new(5.0, [5.0, 5.0, 5.0, 5.0], -1))
            .parent(page_1)
            .build();
        let right_painel = gui
            .create_control()
            .margins([200.0, 0.0, 0.0, 0.0])
            .graphic(style.page_painel.clone())
            .parent(page_1)
            .build();
        let top_text = {
            let text_box = gui
                .create_control()
                .anchors([0.0, 0.0, 1.0, 0.5])
                .margins([15.0, 15.0, -15.0, -7.5])
                .graphic(style.painel.clone().with_color([200, 200, 200, 255].into()))
                .parent(right_painel)
                .build();
            let text = "This is a example text. Please, don't mind me. Continue doing what you need to do. If you cannot ignore this text, I don't mind.".to_owned();
            let graphic = Text::new(
                text,
                (0, -1),
                TextStyle {
                    color: [0, 0, 0, 255].into(),
                    font_size: 20.0,
                    font_id: style.fonts.consolas,
                },
            )
            .into();
            gui.create_control()
                .anchors([0.0, 0.0, 1.0, 1.0])
                .graphic(graphic)
                .parent(text_box)
                .build()
        };
        let bottom_text = {
            let text_box = gui
                .create_control()
                .anchors([0.0, 0.5, 1.0, 1.0])
                .margins([15.0, 7.5, -15.0, -15.0])
                .graphic(style.painel.clone().with_color([200, 200, 200, 255].into()))
                .parent(right_painel)
                .build();
            let text = "This is another example text. Please, also don't mind me. Continue doing what you was doing. If you cannot ignore this text, I don't mind either.".to_owned();
            let graphic = Text::new(
                text,
                (-1, 0),
                TextStyle {
                    color: [0, 0, 0, 255].into(),
                    font_size: 20.0,
                    font_id: style.fonts.notosans,
                },
            )
            .into();
            gui.create_control()
                .anchors([0.0, 0.0, 1.0, 1.0])
                .margins([5.0, 5.0, -5.0, -5.0])
                .graphic(graphic)
                .parent(text_box)
                .build();
            text_box
        };

        let _my_button = create_button(
            gui,
            "My Button".to_string(),
            style.fonts.notosans,
            style.button_style.clone(),
            |_, _| println!("clicked my button!"),
        )
        .min_size([0.0, 30.0])
        .parent(menu)
        .build();
        // {
        //     let button = gui
        //         .create_control()
        //         .behaviour(Box::new(Button::new(style.button_style.clone(), |_, _| {
        //             println!("clicked my button!")
        //         })))
        //         .min_size([0.0, 30.0])
        //         .parent(menu)
        //         .build();
        //     let graphic =
        //         Text::new([40, 40, 100, 255], "My Button".to_owned(), 16.0, (0, 0)).into();
        //     gui.create_control()
        //         .anchors([0.0, 0.0, 1.0, 1.0])
        //         .margins([0.0, 0.0, 0.0, 0.0])
        //         .graphic(graphic)
        //         .parent(button)
        //         .build();
        //     button
        // };
        let _slider = {
            let slide_area = gui.reserve_id();
            let handle = gui.reserve_id();
            let slider = gui
                .create_control()
                .min_size([0.0, 30.0])
                .parent(menu)
                .behaviour(Slider::new(
                    handle,
                    slide_area,
                    100,
                    300,
                    250,
                    style.focus_style.clone(),
                    move |_, ctx: &mut Context, value: i32| {
                        if let Graphic::Text(text) = ctx.get_graphic_mut(top_text) {
                            text.set_font_size(value as f32 / 10.0);
                        }
                    },
                ))
                .build();
            let _slide_area = gui
                .create_control_reserved(slide_area)
                .anchors([0.0, 0.5, 1.0, 0.5])
                .margins([10.0, -3.0, -10.0, 3.0])
                .graphic(style.painel.clone().with_color([170, 170, 170, 255].into()))
                .parent(slider)
                .build();
            let _handle = gui
                .create_control_reserved(handle)
                .anchors([0.5, 0.5, 0.5, 0.5])
                .margins([-3.0, -14.0, 3.0, 14.0])
                .graphic(style.painel.clone().with_color([200, 200, 200, 255].into()))
                .parent(slider)
                .build();

            slider
        };
        let _toggle = {
            let background = gui.reserve_id();
            let marker = gui.reserve_id();
            let toggle = gui
                .create_control()
                .min_size([0.0, 30.0])
                .parent(menu)
                .behaviour(Toggle::new(
                    background,
                    marker,
                    false,
                    style.button_style.clone(),
                    style.focus_style.clone(),
                    move |_, ctx, value| {
                        println!("Toggle changed to {}!", value);
                        if value {
                            ctx.active(bottom_text);
                        } else {
                            ctx.deactive(bottom_text);
                        }
                    },
                ))
                .build();

            let background = {
                let graphic = style.white.clone().with_color([200, 200, 200, 255].into());
                gui.create_control_reserved(background)
                    .anchors([0.0, 0.5, 0.0, 0.5])
                    .margins([5.0, -10.0, 25.0, 10.0])
                    .graphic(graphic)
                    .parent(toggle)
                    .build()
            };
            let _marker = gui
                .create_control_reserved(marker)
                // .anchors([0.5, 0.5, 0.5, 0.5])
                // .margins([-9.0, -9.0, 9.0, 9.0])
                .graphic(style.marker_icon.clone())
                .parent(background)
                .build();

            let graphic = Text::new(
                "Bottom Text".to_owned(),
                (-1, 0),
                TextStyle {
                    color: [40, 40, 100, 255].into(),
                    font_size: 16.0,
                    font_id: style.fonts.notosans,
                },
            )
            .into();
            gui.create_control()
                .anchors([0.0, 0.0, 1.0, 1.0])
                .margins([30.0, 0.0, 0.0, 0.0])
                .graphic(graphic)
                .parent(toggle)
                .build();

            toggle
        };
        let _my_dropdown = {
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
                    .graphic(style.button_style.normal.clone())
                    .behaviour(DropMenu::<String, _>::new(blocker, {
                        let menu_button_style = style.menu_button_style.clone();
                        let notosans = style.fonts.notosans;
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
                                    Text::new(
                                        data.to_string(),
                                        (-1, 0),
                                        TextStyle {
                                            color: [40, 40, 100, 255].into(),
                                            font_size: 16.0,
                                            font_id: notosans,
                                        },
                                    )
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

            let text = gui.reserve_id();
            let my_dropdown = gui
                .create_control()
                .min_size([0.0, 25.0])
                .parent(menu)
                .behaviour(Dropdown::new(
                    vec![
                        "Item A".to_string(),
                        "Item B".to_string(),
                        "Item C".to_string(),
                        "Item D".to_string(),
                        "Item E".to_string(),
                    ],
                    None,
                    float_menu,
                    move |selected, _this, ctx| {
                        ctx.get_graphic_mut(text).set_text(&selected.1);
                    },
                    style.button_style.clone(),
                ))
                .build();
            let _text = gui
                .create_control_reserved(text)
                .margins([10.0, 0.0, -10.0, 0.0])
                .graphic(
                    Text::new(
                        "Select one, please".to_owned(),
                        (-1, 0),
                        TextStyle {
                            color: [40, 40, 100, 255].into(),
                            font_size: 16.0,
                            font_id: style.fonts.notosans,
                        },
                    )
                    .into(),
                )
                .parent(my_dropdown)
                .build();

            my_dropdown
        };
        page_1
    };
    let page_2 = {
        let page_2 = gui
            .create_control()
            .graphic(style.page_painel.clone())
            .parent(page_area)
            .layout(GridLayout::new([10.0, 15.0], [10.0, 10.0, 10.0, 10.0], 3))
            .build();

        let create_vbox = |gui: &mut Gui, expand: [bool; 2], align: i8| {
            gui.create_control()
                .parent(page_2)
                .expand_x(expand[0])
                .expand_y(expand[1])
                .graphic(style.painel.clone().with_color([100, 100, 100, 255].into()))
                .layout(VBoxLayout::new(5.0, [0.0, 0.0, 0.0, 0.0], align))
                .build()
        };

        let create_rect = |gui: &mut Gui,
                           min_size: [f32; 2],
                           expand: [bool; 2],
                           fill: [RectFill; 2],
                           parent: Id| {
            let rect = gui
                .create_control()
                .min_size(min_size)
                .fill_x(fill[0])
                .fill_y(fill[1])
                .expand_x(expand[0])
                .expand_y(expand[1])
                .graphic(style.painel.clone().with_color([200, 200, 200, 255].into()))
                .behaviour(Hoverable::new(
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
                ))
                .parent(parent)
                .build();
            let graphic = Text::new(
                format!("{}x{}", min_size[0], min_size[1]),
                (0, 0),
                TextStyle {
                    color: [40, 40, 100, 255].into(),
                    font_size: 12.0,
                    font_id: style.fonts.notosans,
                },
            )
            .into();
            gui.create_control().graphic(graphic).parent(rect).build();
            rect
        };

        {
            let vbox = create_vbox(gui, [true, true], -1);
            create_rect(
                gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                gui,
                [75.0, 50.0],
                [true, true],
                [RectFill::Fill, RectFill::Fill],
                vbox,
            );
            create_rect(
                gui,
                [50.0, 75.0],
                [true, true],
                [RectFill::Fill, RectFill::Fill],
                vbox,
            );
        }
        {
            let vbox = create_vbox(gui, [false, true], 0);
            create_rect(
                gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                gui,
                [75.0, 50.0],
                [false, false],
                [RectFill::ShrinkCenter, RectFill::Fill],
                vbox,
            );
            create_rect(
                gui,
                [50.0, 75.0],
                [false, false],
                [RectFill::ShrinkEnd, RectFill::Fill],
                vbox,
            );
        }
        {
            let vbox = create_vbox(gui, [false, false], 1);
            create_rect(
                gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                gui,
                [75.0, 50.0],
                [false, false],
                [RectFill::ShrinkCenter, RectFill::Fill],
                vbox,
            );
            create_rect(
                gui,
                [50.0, 75.0],
                [false, false],
                [RectFill::ShrinkEnd, RectFill::Fill],
                vbox,
            );
        }

        {
            let vbox = create_vbox(gui, [true, false], -1);
            create_rect(
                gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                gui,
                [75.0, 50.0],
                [false, false],
                [RectFill::ShrinkCenter, RectFill::Fill],
                vbox,
            );
            create_rect(
                gui,
                [50.0, 75.0],
                [false, false],
                [RectFill::ShrinkEnd, RectFill::Fill],
                vbox,
            );
        }
        {
            let vbox = create_vbox(gui, [false, false], 0);
            create_rect(
                gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                gui,
                [75.0, 50.0],
                [false, false],
                [RectFill::ShrinkCenter, RectFill::Fill],
                vbox,
            );
            create_rect(
                gui,
                [50.0, 75.0],
                [false, false],
                [RectFill::ShrinkEnd, RectFill::Fill],
                vbox,
            );
        }
        {
            let vbox = create_vbox(gui, [false, false], 1);
            create_rect(
                gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                gui,
                [75.0, 50.0],
                [false, false],
                [RectFill::ShrinkCenter, RectFill::Fill],
                vbox,
            );
            create_rect(
                gui,
                [50.0, 75.0],
                [false, false],
                [RectFill::ShrinkEnd, RectFill::Fill],
                vbox,
            );
        }

        page_2
    };
    let page_3 = {
        let page_3 = gui
            .create_control()
            .graphic(style.page_painel.clone())
            .parent(page_area)
            .layout(VBoxLayout::new(5.0, [10.0, 10.0, 10.0, 10.0], -1))
            .build();
        let list = gui.reserve_id();
        let _input_box = {
            let hbox = gui
                .create_control()
                .parent(page_3)
                .layout(HBoxLayout::new(0.0, Default::default(), -1))
                .build();
            let _label = gui
                .create_control()
                .graphic(
                    Text::new(
                        "Add new: ".to_owned(),
                        (-1, 0),
                        TextStyle {
                            color: [0, 0, 0, 255].into(),
                            font_size: 16.0,
                            font_id: style.fonts.notosans,
                        },
                    )
                    .into(),
                )
                .layout(FitText)
                .parent(hbox)
                .build();
            let caret = gui.reserve_id();
            let input_text = gui.reserve_id();
            let input_box = gui
                .create_control()
                .min_size([0.0, 24.0])
                .parent(hbox)
                .expand_x(true)
                .behaviour(TextField::new(
                    "".into(),
                    caret,
                    input_text,
                    TextFieldStyle {
                        caret_color: Color::BLACK,
                        selection_color: [170, 0, 255, 255].into(),
                        background: OnFocusStyle {
                            normal: style.painel.clone().with_color([200, 200, 200, 255].into()),
                            focus: style
                                .button_style
                                .focus
                                .clone()
                                .with_color([200, 200, 200, 255].into()),
                        },
                    }
                    .into(),
                    {
                        let button_style = style.button_style.clone();
                        let painel = style.white.clone();
                        let notosans = style.fonts.notosans;
                        move |_this: Id, ctx: &mut Context, text: &mut String| {
                            println!("Submited {}!", text);
                            create_item(
                                ctx,
                                list,
                                painel.clone(),
                                text.clone(),
                                notosans,
                                [130, 150, 255, 255].into(),
                                button_style.clone(),
                            );
                            text.clear();
                            true
                        }
                    },
                ))
                .build();
            let _caret = gui
                .create_control_reserved(caret)
                .anchors([0.0, 0.0, 0.0, 0.0])
                .graphic(style.white.clone().with_color([0, 0, 0, 255].into()))
                .parent(input_box)
                .build();
            let _input_text = gui
                .create_control_reserved(input_text)
                .graphic(
                    Text::new(
                        String::new(),
                        (-1, 0),
                        TextStyle {
                            color: [0, 0, 0, 255].into(),
                            font_size: 16.0,
                            font_id: style.fonts.notosans,
                        },
                    )
                    .into(),
                )
                .parent(input_box)
                .build();
            input_box
        };
        let scroll_view = gui.reserve_id();
        let view = gui
            .create_control()
            .graphic(Graphic::None)
            .parent(scroll_view)
            .layout(ViewLayout::new(true, true))
            .build();
        let h_scroll_bar_handle = gui.reserve_id();
        let h_scroll_bar = gui
            .create_control()
            .min_size([20.0, 20.0])
            .graphic(style.white.clone().with_color([150, 150, 150, 255].into()))
            .behaviour(ScrollBar::new(
                h_scroll_bar_handle,
                scroll_view,
                false,
                style.button_style.clone(),
            ))
            .parent(scroll_view)
            .build();
        let h_scroll_bar_handle = gui
            .create_control_reserved(h_scroll_bar_handle)
            .min_size([20.0, 20.0])
            .graphic(style.white.clone().with_color([220, 220, 220, 255].into()))
            .parent(h_scroll_bar)
            .build();
        let v_scroll_bar_handle = gui.reserve_id();
        let v_scroll_bar = gui
            .create_control()
            .min_size([20.0, 20.0])
            .graphic(style.white.clone().with_color([150, 150, 150, 255].into()))
            .parent(scroll_view)
            .behaviour(ScrollBar::new(
                v_scroll_bar_handle,
                scroll_view,
                true,
                style.button_style.clone(),
            ))
            .build();
        let v_scroll_bar_handle = gui
            .create_control_reserved(v_scroll_bar_handle)
            .min_size([20.0, 20.0])
            .graphic(style.white.clone().with_color([220, 220, 220, 255].into()))
            .parent(v_scroll_bar)
            .build();
        let list = gui
            .create_control_reserved(list)
            .layout(VBoxLayout::new(3.0, [5.0, 5.0, 5.0, 5.0], -1))
            .parent(view)
            .build();

        let _scroll_view = gui
            .create_control_reserved(scroll_view)
            .graphic(style.white.clone().with_color([100, 100, 100, 255].into()))
            .expand_y(true)
            .parent(page_3)
            .behaviour_and_layout(ScrollView::new(
                view,
                list,
                Some((h_scroll_bar, h_scroll_bar_handle)),
                Some((v_scroll_bar, v_scroll_bar_handle)),
            ))
            .build();

        let mut seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        seed = seed ^ (seed << 64);
        for i in 0..5 {
            let color = (seed.rotate_left(i * 3)) as u32 | 0xff;
            create_item(
                &mut gui.get_context(),
                list,
                style.painel.clone(),
                format!("This is the item number {} with the color which hexadecimal representation is #{:0x}", i + 1, color), 
                style.fonts.notosans,
                Color::from_u32(color),
                style.button_style.clone()
            );
        }
        page_3
    };
    let page_4 = {
        let page_4 = gui
            .create_control()
            .parent(page_area)
            .layout(RatioLayout::new(1.0, (0, 0)))
            .build();
        let font_texture = 1; // TODO
        let graphic = Texture::new(font_texture, [0.0, 0.0, 1.0, 1.0]).into();
        gui.create_control()
            .graphic(graphic)
            .behaviour(ContextMenu::new(style.menu_style.clone(), {
                use Item::*;
                Rc::new(Menu::new(
                    String::new(),
                    vec![
                        Button(
                            "Option 0".to_string(),
                            Box::new(|_, _| println!("Option 0")),
                        ),
                        Button(
                            "Option 1".to_string(),
                            Box::new(|_, _| println!("Option 1")),
                        ),
                        Separator,
                        Button(
                            "Option A".to_string(),
                            Box::new(|_, _| println!("Option A")),
                        ),
                        Button(
                            "Option B".to_string(),
                            Box::new(|_, _| println!("Option B")),
                        ),
                    ],
                ))
            }))
            .parent(page_4)
            .build();
        page_4
    };
    let page_na = {
        let page_na = gui.create_control().parent(page_area).build();
        let graphic = Text::new("This tab page is yet not avaliable. In fact, it is not even planned what will have in this page, sorry...".to_owned(), (0, -1), TextStyle { color: [255, 255, 255, 255].into(), font_size: 20.0, font_id: style.fonts.notosans }).into();
        gui.create_control()
            .margins([15.0, 15.0, -15.0, -15.0])
            .graphic(graphic)
            .parent(page_na)
            .build();
        page_na
    };
    let _tabs = {
        let tab_group = ButtonGroup::new(|_, _| {});
        let painel = style.painel.clone();
        let create_button = |gui: &mut Gui, page: Id, selected: bool, label: String| {
            let button = gui
                .create_control()
                .graphic(painel.clone())
                .min_size([0.0, 30.0])
                .parent(header)
                .expand_x(true)
                .behaviour(TabButton::new(
                    tab_group.clone(),
                    page,
                    selected,
                    style.tab_style.clone(),
                ))
                .build();
            let graphic = Text::new(
                label,
                (0, 0),
                TextStyle {
                    color: [40, 40, 100, 255].into(),
                    font_size: 16.0,
                    font_id: style.fonts.notosans,
                },
            )
            .into();
            gui.create_control()
                .graphic(graphic)
                .parent(button)
                .layout(FitText)
                .build();
            button
        };
        create_button(gui, page_1, true, "Random Controls".to_owned());
        create_button(gui, page_2, false, "Grid Layout".to_owned());
        create_button(gui, page_3, false, "ScrollView".to_owned());
        create_button(gui, page_4, false, "Font Texture".to_owned());
        create_button(gui, page_na, false, "To be continued...".to_owned());
    };
    let _window = {
        let window = gui
            .create_control()
            .anchors([0.0, 0.0, 0.0, 0.0])
            .margins([20.0, 20.0, 20.0, 20.0])
            .behaviour(widgets::Window::new())
            .layout(VBoxLayout::new(0.0, [0.0, 0.0, 0.0, 0.0], -1))
            .graphic(style.painel.clone())
            .build();
        let header = gui
            .create_control()
            .graphic(style.painel.clone().with_color([0, 0, 255, 255].into()))
            .layout(HBoxLayout::new(2.0, [2.0, 2.0, 2.0, 2.0], -1))
            .fill_y(RectFill::ShrinkStart)
            .parent(window)
            .build();
        let _title = gui
            .create_control()
            .graphic(Graphic::from(Text::new(
                "This is a Window".to_string(),
                (-1, 0),
                TextStyle {
                    color: [255, 255, 255, 255].into(),
                    font_size: 20.0,
                    font_id: style.fonts.notosans,
                },
            )))
            .layout(FitText)
            .parent(header)
            .expand_x(true)
            .build();
        let _close_button = gui
            .create_control()
            .behaviour(Button::new(style.close_button, true, move |_this, ctx| {
                ctx.deactive(window)
            }))
            .parent(header)
            .min_size([20.0, 20.0])
            .build();
        let content = gui
            .create_control()
            .layout(MarginLayout::new([5.0, 5.0, 5.0, 5.0]))
            .parent(window)
            .expand_y(true)
            .build();
        let content = gui
            .create_control()
            .behaviour(Blocker::new(|_, _| ()))
            .layout(VBoxLayout::new(15.0, [10.0, 10.0, 10.0, 10.0], -1))
            .parent(content)
            .expand_y(true)
            .build();
        let _text = gui
            .create_control()
            .graphic(Graphic::from(Text::new(
                "This is the content of a window.\nPlease be aware of it.".to_string(),
                (0, 0),
                TextStyle {
                    color: [0, 0, 0, 255].into(),
                    font_size: 20.0,
                    font_id: style.fonts.notosans,
                },
            )))
            .layout(FitText)
            .parent(content)
            .expand_y(true)
            .build();
        create_button(
            gui,
            "OK".to_string(),
            style.fonts.notosans,
            style.button_style,
            move |_this, ctx| ctx.deactive(window),
        )
        .min_size([40.0, 25.0])
        .parent(content)
        .fill_x(RectFill::ShrinkCenter)
        .build();
        window
    };
}

fn create_item(
    ctx: &mut Context,
    list: Id,
    painel: Graphic,
    text: String,
    font_id: FontId,
    color: Color,
    button_style: Rc<ButtonStyle>,
) {
    let item = ctx
        .create_control()
        .min_size([100.0, 35.0])
        .graphic(painel.clone().with_color(color))
        .parent(list)
        .layout(HBoxLayout::new(0.0, [5.0, 0.0, 5.0, 0.0], 0))
        .build();
    let _text = ctx
        .create_control()
        .parent(item)
        .graphic(
            Text::new(
                text,
                (-1, 0),
                TextStyle {
                    color: [0, 0, 0, 255].into(),
                    font_size: 16.0,
                    font_id,
                },
            )
            .into(),
        )
        .layout(FitText)
        .expand_x(true)
        .build();
    let _button = ctx
        .create_control()
        .parent(item)
        .graphic(painel)
        .behaviour(Button::new(button_style, true, move |_, ctx| {
            ctx.remove(item);
        }))
        .min_size([15.0, 15.0])
        .fill_x(RectFill::ShrinkCenter)
        .fill_y(RectFill::ShrinkCenter)
        .build();
}

fn create_button<F: Fn(Id, &mut Context) + 'static>(
    gui: &mut Gui,
    text: String,
    font_id: FontId,
    button_style: Rc<ButtonStyle>,
    on_click: F,
) -> ControlBuilder {
    let button_id: Id = gui.reserve_id();
    gui.create_control_reserved(button_id)
        .child(|x| {
            // text
            x.graphic(
                Text::new(
                    text,
                    (0, 0),
                    TextStyle {
                        color: [40, 40, 100, 255].into(),
                        font_size: 16.0,
                        font_id,
                    },
                )
                .into(),
            )
            .layout(FitText)
        })
        .behaviour(Button::new(button_style, true, on_click))
    // .layout(Box::new(MarginLayout::new([7.0, 7.0, 7.0, 7.0])))
}
