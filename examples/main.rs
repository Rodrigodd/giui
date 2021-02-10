#![allow(clippy::useless_vec)]
use std::cell::RefCell;
use std::rc::Rc;

use ab_glyph::FontArc;
use sprite_render::{Camera, GLSpriteRender, SpriteInstance, SpriteRender};
use ui_engine::{
    layouts::{FitText, GridLayout, HBoxLayout, MarginLayout, RatioLayout, VBoxLayout},
    render::{GUIRender, GUIRenderer, Graphic, Panel, Text, Texture},
    style::{ButtonStyle, MenuStyle, OnFocusStyle, TabStyle},
    widgets::{
        self, Blocker, Button, ButtonGroup, CloseMenu, ContextMenu, DropMenu, Dropdown, Hoverable,
        Item, Menu, MenuBar, MenuItem, ScrollBar, ScrollView, Slider, TabButton, TextField, Toggle,
        ViewLayout,
    },
    Context, ControlBuilder, Id, RectFill, GUI,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
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
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new().with_inner_size(PhysicalSize::new(800, 600));
    let (window, mut render) = GLSpriteRender::new(wb, &event_loop, true);
    let window_size = window.inner_size();
    let font_texture = render.new_texture(128, 128, &[], false);
    let mut gui_render = GUIRender::new(font_texture, [128, 128]);
    let fonts: Vec<FontArc> = [include_bytes!("../examples/NotoSans-Regular.ttf")]
        .iter()
        .map(|&font| FontArc::try_from_slice(font).unwrap())
        .collect();
    let mut gui = GUI::new(window_size.width as f32, window_size.height as f32, fonts);
    let texture = {
        let data = image::open("D:/repos/rust/ui_engine/examples/panel.png").unwrap();
        let data = data.to_rgba8();
        render.new_texture(data.width(), data.height(), data.as_ref(), true)
    };
    let tab_texture = {
        let data = image::open("D:/repos/rust/ui_engine/examples/tab.png").unwrap();
        let data = data.to_rgba8();
        render.new_texture(data.width(), data.height(), data.as_ref(), true)
    };
    let icon_texture = {
        let data = image::open("D:/repos/rust/ui_engine/examples/icons.png").unwrap();
        let data = data.to_rgba8();
        render.new_texture(data.width(), data.height(), data.as_ref(), true)
    };
    let mut camera = sprite_render::Camera::new(
        window_size.width,
        window_size.height,
        window_size.height as f32,
    );
    camera.set_position(
        window_size.width as f32 / 2.0,
        window_size.height as f32 / 2.0,
    );
    let painel: Graphic = Panel::new(texture, [0.0, 0.0, 0.5, 0.5], 10.0).into();
    let page_painel: Graphic = Panel::new(texture, [0.0, 0.1, 0.5, 0.4], 10.0).into();

    let button_style = Rc::new(ButtonStyle {
        normal: Graphic::from(Panel::new(texture, [0.0, 0.0, 0.5, 0.5], 10.0)),
        hover: Graphic::from(Panel::new(texture, [0.5, 0.0, 0.5, 0.5], 10.0)),
        pressed: Graphic::from(Panel::new(texture, [0.0, 0.5, 0.5, 0.5], 10.0)),
        focus: Graphic::from(Panel::new(texture, [0.5, 0.5, 0.5, 0.5], 10.0)),
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
            .with_color([180, 180, 180, 255])
            .into(),
    });
    let tab_style = Rc::new(TabStyle {
        hover: Graphic::from(Panel::new(tab_texture, [0.5, 0.0, 0.5, 0.5], 10.0)),
        pressed: Graphic::from(Panel::new(tab_texture, [0.0, 0.5, 0.5, 0.5], 10.0)),
        unselected: Graphic::from(Panel::new(tab_texture, [0.0, 0.0, 0.5, 0.5], 10.0)),
        selected: Graphic::from(Panel::new(tab_texture, [0.5, 0.5, 0.5, 0.5], 10.0)),
    });
    let focus_style = Rc::new(OnFocusStyle {
        normal: Graphic::None,
        focus: button_style.focus.clone(),
    });

    let (hover, hover_label) = {
        let graphic = painel
            .clone()
            .with_color([50, 50, 50, 255])
            .with_border(0.0);
        let hover = gui
            .create_control()
            .with_anchors([0.0, 0.0, 0.0, 0.0])
            .with_graphic(graphic)
            .with_margins([3.0, 6.0, 6.0, 9.0])
            .with_layout(MarginLayout::new([3.0, 3.0, 3.0, 3.0]))
            .build();
        let graphic = Text::new(
            [255, 255, 255, 255],
            "This is a Hover".to_owned(),
            12.0,
            (-1, 0),
        )
        .into();
        let label = gui
            .create_control()
            .with_graphic(graphic)
            .with_parent(hover)
            .with_layout(FitText)
            .build();

        (hover, label)
    };

    let surface = gui
        .create_control()
        .with_layout(VBoxLayout::new(0.0, [0.0; 4], -1))
        .build();
    let _menubar = {
        let menu = gui.reserve_id();
        // TODO: the menu_bar's blocker espect that I know the size of menu_bar
        let blocker = gui
            .create_control()
            .with_active(false)
            .with_margins([0.0, 20.0, 0.0, 0.0])
            .with_behaviour(Blocker::new(move |_, ctx| {
                ctx.send_event_to(menu, CloseMenu)
            }))
            .build();
        use Item::*;
        let proxy = event_loop.create_proxy();
        gui.create_control_reserved(menu)
            .with_graphic(menu_button_style.normal.clone())
            .with_behaviour(MenuBar::new(
                menu_style.clone(),
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
            .with_layout(HBoxLayout::new(0.0, [0.0; 4], -1))
            .with_parent(surface)
            .build()
    };

    let header = gui
        .create_control()
        .with_layout(HBoxLayout::new(0.0, [0.0, 0.0, 0.0, 0.0], -1))
        .with_parent(surface)
        .build();

    let page_area = gui
        .create_control()
        .with_margins([0.0, 45.0, 0.0, 0.0])
        .with_parent(surface)
        .with_expand_y(true)
        .build();
    let page_1 = {
        let page_1 = gui.create_control().with_parent(page_area).build();
        let menu = gui
            .create_control()
            .with_anchors([0.0, 0.0, 0.0, 1.0])
            .with_margins([0.0, 0.0, 200.0, 0.0])
            .with_graphic(page_painel.clone())
            .with_layout(VBoxLayout::new(5.0, [5.0, 5.0, 5.0, 5.0], -1))
            .with_parent(page_1)
            .build();
        let right_painel = gui
            .create_control()
            .with_margins([200.0, 0.0, 0.0, 0.0])
            .with_graphic(page_painel.clone())
            .with_parent(page_1)
            .build();
        let top_text = {
            let text_box = gui
                .create_control()
                .with_anchors([0.0, 0.0, 1.0, 0.5])
                .with_margins([15.0, 15.0, -15.0, -7.5])
                .with_graphic(painel.clone().with_color([200, 200, 200, 255]))
                .with_parent(right_painel)
                .build();
            let graphic = Text::new(
            [0, 0, 0, 255],
            "This is a example text. Please, don't mind me. Continue doing what you need to do. If you cannot ignore this text, I don't mind.".to_owned(),
            20.0,
            (0, -1),
        ).into();
            gui.create_control()
                .with_anchors([0.0, 0.0, 1.0, 1.0])
                .with_graphic(graphic)
                .with_parent(text_box)
                .build()
        };
        let bottom_text = {
            let text_box = gui
                .create_control()
                .with_anchors([0.0, 0.5, 1.0, 1.0])
                .with_margins([15.0, 7.5, -15.0, -15.0])
                .with_graphic(painel.clone().with_color([200, 200, 200, 255]))
                .with_parent(right_painel)
                .build();
            let graphic = Text::new(
            [0, 0, 0, 255],
            "This is another example text. Please, also don't mind me. Continue doing what you was doing. If you cannot ignore this text, I don't mind either.".to_owned(),
            20.0,
            (-1, 0),
        ).into();
            gui.create_control()
                .with_anchors([0.0, 0.0, 1.0, 1.0])
                .with_margins([5.0, 5.0, -5.0, -5.0])
                .with_graphic(graphic)
                .with_parent(text_box)
                .build();
            text_box
        };

        let _my_button = create_button(
            &mut gui,
            "My Button".to_string(),
            button_style.clone(),
            |_, _| println!("clicked my button!"),
        )
        .with_min_size([0.0, 30.0])
        .with_parent(menu)
        .build();
        // {
        //     let button = gui
        //         .create_control()
        //         .with_behaviour(Box::new(Button::new(button_style.clone(), |_, _| {
        //             println!("clicked my button!")
        //         })))
        //         .with_min_size([0.0, 30.0])
        //         .with_parent(menu)
        //         .build();
        //     let graphic =
        //         Text::new([40, 40, 100, 255], "My Button".to_owned(), 16.0, (0, 0)).into();
        //     gui.create_control()
        //         .with_anchors([0.0, 0.0, 1.0, 1.0])
        //         .with_margins([0.0, 0.0, 0.0, 0.0])
        //         .with_graphic(graphic)
        //         .with_parent(button)
        //         .build();
        //     button
        // };
        let _slider = {
            let slider = gui
                .create_control()
                .with_min_size([0.0, 30.0])
                .with_parent(menu)
                .build();
            let slide_area = gui
                .create_control()
                .with_anchors([0.0, 0.5, 1.0, 0.5])
                .with_margins([10.0, -3.0, -10.0, 3.0])
                .with_graphic(painel.clone().with_color([170, 170, 170, 255]))
                .with_parent(slider)
                .build();
            let handle = gui
                .create_control()
                .with_anchors([0.5, 0.5, 0.5, 0.5])
                .with_margins([-3.0, -14.0, 3.0, 14.0])
                .with_graphic(painel.clone().with_color([200, 200, 200, 255]))
                .with_parent(slider)
                .build();
            gui.set_behaviour(
                slider,
                Slider::new(
                    handle,
                    slide_area,
                    100,
                    300,
                    250,
                    focus_style.clone(),
                    move |_, ctx: &mut Context, value: i32| {
                        if let Graphic::Text(text) = ctx.get_graphic_mut(top_text) {
                            text.set_font_size(value as f32 / 10.0);
                        }
                    },
                ),
            );
            slider
        };
        let _toggle = {
            let toggle = gui
                .create_control()
                .with_min_size([0.0, 30.0])
                .with_parent(menu)
                .build();

            let background = {
                let graphic = painel
                    .clone()
                    .with_color([200, 200, 200, 255])
                    .with_border(0.0);
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
                .with_graphic(painel.clone().with_color([0, 0, 0, 255]).with_border(0.0))
                .with_parent(background)
                .build();
            gui.set_behaviour(
                toggle,
                Toggle::new(
                    background,
                    marker,
                    false,
                    button_style.clone(),
                    focus_style,
                    move |_, ctx, value| {
                        println!("Toogle changed to {}!", value);
                        if value {
                            ctx.active(bottom_text);
                        } else {
                            ctx.deactive(bottom_text);
                        }
                    },
                ),
            );

            let graphic =
                Text::new([40, 40, 100, 255], "Bottom Text".to_owned(), 16.0, (-1, 0)).into();
            gui.create_control()
                .with_anchors([0.0, 0.0, 1.0, 1.0])
                .with_margins([30.0, 0.0, 0.0, 0.0])
                .with_graphic(graphic)
                .with_parent(toggle)
                .build();

            toggle
        };
        let _my_dropdown = {
            let float_menu = {
                let blocker = gui.create_control().with_active(false).build();
                let menu = gui
                    .create_control()
                    .with_active(false)
                    .with_graphic(button_style.normal.clone())
                    .with_behaviour(DropMenu::<String, _>::new(blocker, {
                        let menu_button_style = menu_button_style.clone();
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

            let my_dropdown = gui
                .create_control()
                .with_min_size([0.0, 25.0])
                .with_parent(menu)
                .build();
            let text = gui
                .create_control()
                .with_margins([10.0, 0.0, -10.0, 0.0])
                .with_graphic(
                    Text::new(
                        [40, 40, 100, 255],
                        "Select one, please".to_owned(),
                        16.0,
                        (-1, 0),
                    )
                    .into(),
                )
                .with_parent(my_dropdown)
                .build();
            gui.set_behaviour(
                my_dropdown,
                Dropdown::new(
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
                    button_style.clone(),
                ),
            );

            my_dropdown
        };
        page_1
    };
    let page_2 = {
        let page_2 = gui
            .create_control()
            .with_graphic(page_painel.clone())
            .with_parent(page_area)
            .with_layout(GridLayout::new([10.0, 15.0], [10.0, 10.0, 10.0, 10.0], 3))
            .build();

        let create_vbox = |gui: &mut GUI, expand: [bool; 2], align: i8| {
            gui.create_control()
                .with_parent(page_2)
                .with_expand_x(expand[0])
                .with_expand_y(expand[1])
                .with_graphic(painel.clone().with_color([100, 100, 100, 255]))
                .with_layout(VBoxLayout::new(5.0, [0.0, 0.0, 0.0, 0.0], align))
                .build()
        };

        let create_rect = |gui: &mut GUI,
                           min_size: [f32; 2],
                           expand: [bool; 2],
                           fill: [RectFill; 2],
                           parent: Id| {
            let rect = gui
                .create_control()
                .with_min_size(min_size)
                .with_fill_x(fill[0])
                .with_fill_y(fill[1])
                .with_expand_x(expand[0])
                .with_expand_y(expand[1])
                .with_graphic(painel.clone().with_color([200, 200, 200, 255]))
                .with_behaviour(Hoverable::new(
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
                .with_parent(parent)
                .build();
            let graphic = Text::new(
                [40, 40, 100, 255],
                format!("{}x{}", min_size[0], min_size[1]),
                12.0,
                (0, 0),
            )
            .into();
            gui.create_control()
                .with_graphic(graphic)
                .with_parent(rect)
                .build();
            rect
        };

        {
            let vbox = create_vbox(&mut gui, [true, true], -1);
            create_rect(
                &mut gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [75.0, 50.0],
                [true, true],
                [RectFill::Fill, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [50.0, 75.0],
                [true, true],
                [RectFill::Fill, RectFill::Fill],
                vbox,
            );
        }
        {
            let vbox = create_vbox(&mut gui, [false, true], 0);
            create_rect(
                &mut gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [75.0, 50.0],
                [false, false],
                [RectFill::ShrinkCenter, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [50.0, 75.0],
                [false, false],
                [RectFill::ShrinkEnd, RectFill::Fill],
                vbox,
            );
        }
        {
            let vbox = create_vbox(&mut gui, [false, false], 1);
            create_rect(
                &mut gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [75.0, 50.0],
                [false, false],
                [RectFill::ShrinkCenter, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [50.0, 75.0],
                [false, false],
                [RectFill::ShrinkEnd, RectFill::Fill],
                vbox,
            );
        }

        {
            let vbox = create_vbox(&mut gui, [true, false], -1);
            create_rect(
                &mut gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [75.0, 50.0],
                [false, false],
                [RectFill::ShrinkCenter, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [50.0, 75.0],
                [false, false],
                [RectFill::ShrinkEnd, RectFill::Fill],
                vbox,
            );
        }
        {
            let vbox = create_vbox(&mut gui, [false, false], 0);
            create_rect(
                &mut gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [75.0, 50.0],
                [false, false],
                [RectFill::ShrinkCenter, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [50.0, 75.0],
                [false, false],
                [RectFill::ShrinkEnd, RectFill::Fill],
                vbox,
            );
        }
        {
            let vbox = create_vbox(&mut gui, [false, false], 1);
            create_rect(
                &mut gui,
                [50.0, 50.0],
                [false, false],
                [RectFill::ShrinkStart, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
                [75.0, 50.0],
                [false, false],
                [RectFill::ShrinkCenter, RectFill::Fill],
                vbox,
            );
            create_rect(
                &mut gui,
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
            .with_graphic(page_painel.clone())
            .with_parent(page_area)
            .with_layout(VBoxLayout::new(5.0, [10.0, 10.0, 10.0, 10.0], -1))
            .build();
        let list = gui.reserve_id();
        let _input_box = {
            let hbox = gui
                .create_control()
                .with_parent(page_3)
                .with_layout(HBoxLayout::new(0.0, Default::default(), -1))
                .build();
            let _label = gui
                .create_control()
                .with_graphic(
                    Text::new([0, 0, 0, 255], "Add new: ".to_owned(), 16.0, (-1, 0)).into(),
                )
                .with_layout(FitText)
                .with_parent(hbox)
                .build();
            let input_box = gui
                .create_control()
                .with_min_size([0.0, 24.0])
                .with_parent(hbox)
                .with_expand_x(true)
                .build();
            let caret = gui
                .create_control()
                .with_anchors([0.0, 0.0, 0.0, 0.0])
                .with_graphic(painel.clone().with_color([0, 0, 0, 255]).with_border(0.0))
                .with_parent(input_box)
                .build();
            let input_text = gui
                .create_control()
                .with_graphic(Text::new([0, 0, 0, 255], String::new(), 16.0, (-1, 0)).into())
                .with_parent(input_box)
                .build();
            gui.set_behaviour(
                input_box,
                TextField::new(
                    "".into(),
                    caret,
                    input_text,
                    OnFocusStyle {
                        normal: painel.clone().with_color([200, 200, 200, 255]),
                        focus: button_style.focus.clone().with_color([200, 200, 200, 255]),
                    }
                    .into(),
                    {
                        let button_style = button_style.clone();
                        move |_this: Id, ctx: &mut Context, text: &mut String| {
                            println!("Submited {}!", text);
                            create_item(
                                ctx,
                                list,
                                texture,
                                text.clone(),
                                [130, 150, 255, 255],
                                button_style.clone(),
                            );
                            text.clear();
                            true
                        }
                    },
                ),
            );
            input_box
        };
        let scroll_view = gui
            .create_control()
            .with_graphic(
                painel
                    .clone()
                    .with_color([100, 100, 100, 255])
                    .with_border(0.0),
            )
            .with_expand_y(true)
            .with_parent(page_3)
            .build();
        let view = gui
            .create_control()
            .with_graphic(Graphic::None)
            .with_parent(scroll_view)
            .with_layout(ViewLayout::new(true, true))
            .build();
        let h_scroll_bar = gui
            .create_control()
            .with_min_size([20.0, 20.0])
            .with_graphic(
                painel
                    .clone()
                    .with_color([150, 150, 150, 255])
                    .with_border(0.0),
            )
            .with_parent(scroll_view)
            .build();
        let h_scroll_bar_handle = gui
            .create_control()
            .with_graphic(
                painel
                    .clone()
                    .with_color([220, 220, 220, 255])
                    .with_border(0.0),
            )
            .with_parent(h_scroll_bar)
            .build();
        gui.set_behaviour(
            h_scroll_bar,
            ScrollBar::new(
                h_scroll_bar_handle,
                scroll_view,
                false,
                button_style.clone(),
            ),
        );
        let v_scroll_bar = gui
            .create_control()
            .with_min_size([20.0, 20.0])
            .with_graphic(
                painel
                    .clone()
                    .with_color([150, 150, 150, 255])
                    .with_border(0.0),
            )
            .with_parent(scroll_view)
            .build();
        let v_scroll_bar_handle = gui
            .create_control()
            .with_graphic(
                painel
                    .clone()
                    .with_color([220, 220, 220, 255])
                    .with_border(0.0),
            )
            .with_parent(v_scroll_bar)
            .build();
        gui.set_behaviour(
            v_scroll_bar,
            ScrollBar::new(v_scroll_bar_handle, scroll_view, true, button_style.clone()),
        );
        let list = gui
            .create_control_reserved(list)
            .with_layout(VBoxLayout::new(3.0, [5.0, 5.0, 5.0, 5.0], -1))
            .with_parent(view)
            .build();

        let behaviour = ScrollView::new(
            view,
            list,
            Some((h_scroll_bar, h_scroll_bar_handle)),
            Some((v_scroll_bar, v_scroll_bar_handle)),
        );
        let behaviour = Rc::new(RefCell::new(behaviour));
        gui.set_behaviour(scroll_view, behaviour.clone());
        gui.set_layout(scroll_view, behaviour);

        let mut seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        seed = seed ^ (seed << 64);
        for i in 0..5 {
            let color = (seed.rotate_left(i * 3)) as u32 | 0xff;
            create_item(&mut gui.get_context(), list, texture, format!("This is the item number {} with the color which hexadecimal representation is #{:0x}", i + 1, color), color.to_be_bytes(),button_style.clone());
        }
        page_3
    };
    let page_4 = {
        let page_4 = gui
            .create_control()
            .with_parent(page_area)
            .with_layout(RatioLayout::new(1.0, (0, 0)))
            .build();
        let graphic = Texture::new(font_texture, [0.0, 0.0, 1.0, 1.0]).into();
        gui.create_control()
            .with_graphic(graphic)
            .with_behaviour(ContextMenu::new(menu_style, {
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
            .with_parent(page_4)
            .build();
        page_4
    };
    let page_na = {
        let page_na = gui.create_control().with_parent(page_area).build();
        let graphic = Text::new(
        [255, 255, 255, 255],
        "This tab page is yet not avaliable. In fact, it is not even planned what will have in this page, sorry...".to_owned(),
        20.0,
        (0, -1),
    ).into();
        gui.create_control()
            .with_margins([15.0, 15.0, -15.0, -15.0])
            .with_graphic(graphic)
            .with_parent(page_na)
            .build();
        page_na
    };
    let _tabs = {
        let tab_group = ButtonGroup::new(|_, _| {});
        let create_button = |gui: &mut GUI, page: Id, selected: bool, label: String| {
            let button = gui
                .create_control()
                .with_graphic(painel.clone())
                .with_min_size([0.0, 30.0])
                .with_parent(header)
                .with_expand_x(true)
                .with_behaviour(TabButton::new(
                    tab_group.clone(),
                    page,
                    selected,
                    tab_style.clone(),
                ))
                .build();
            let graphic = Text::new([40, 40, 100, 255], label, 16.0, (0, 0)).into();
            gui.create_control()
                .with_graphic(graphic)
                .with_parent(button)
                .with_layout(FitText)
                .build();
            button
        };
        create_button(&mut gui, page_1, true, "Random Controls".to_owned());
        create_button(&mut gui, page_2, false, "Grid Layout".to_owned());
        create_button(&mut gui, page_3, false, "ScrollView".to_owned());
        create_button(&mut gui, page_4, false, "Font Texture".to_owned());
        create_button(&mut gui, page_na, false, "To be continued...".to_owned());
    };
    let _window = {
        let window = gui
            .create_control()
            .with_anchors([0.0, 0.0, 0.0, 0.0])
            .with_margins([20.0, 20.0, 20.0, 20.0])
            .with_behaviour(widgets::Window::new())
            .with_layout(VBoxLayout::new(0.0, [0.0, 0.0, 0.0, 0.0], -1))
            .with_graphic(painel.clone())
            .build();
        let header = gui
            .create_control()
            .with_graphic(painel.clone().with_color([0, 0, 255, 255]))
            .with_layout(HBoxLayout::new(2.0, [2.0, 2.0, 2.0, 2.0], -1))
            .with_fill_y(RectFill::ShrinkStart)
            .with_parent(window)
            .build();
        let style = Rc::new(ButtonStyle {
            normal: painel.clone().with_color([255, 0, 0, 255]),
            hover: painel.clone().with_color([240, 0, 0, 255]),
            pressed: painel.clone().with_color([230, 0, 0, 255]),
            focus: painel.clone().with_color([255, 0, 0, 255]),
        });
        let _title = gui
            .create_control()
            .with_graphic(Graphic::from(Text::new(
                [255, 255, 255, 255],
                "This is a Window".to_string(),
                20.0,
                (-1, 0),
            )))
            .with_layout(FitText)
            .with_parent(header)
            .with_expand_x(true)
            .build();
        let _close_button = gui
            .create_control()
            .with_behaviour(Button::new(style, move |_this, ctx| ctx.deactive(window)))
            .with_parent(header)
            .with_min_size([20.0, 20.0])
            .build();
        let content = gui
            .create_control()
            .with_layout(MarginLayout::new([5.0, 5.0, 5.0, 5.0]))
            .with_parent(window)
            .with_expand_y(true)
            .build();
        let content = gui
            .create_control()
            .with_behaviour(Blocker::new(|_, _| ()))
            .with_layout(VBoxLayout::new(15.0, [10.0, 10.0, 10.0, 10.0], -1))
            .with_parent(content)
            .with_expand_y(true)
            .build();
        let _text = gui
            .create_control()
            .with_graphic(Graphic::from(Text::new(
                [0, 0, 0, 255],
                "This is the content of a window.\nPlease be aware of it.".to_string(),
                20.0,
                (0, 0),
            )))
            .with_layout(FitText)
            .with_parent(content)
            .with_expand_y(true)
            .build();
        create_button(
            &mut gui,
            "OK".to_string(),
            button_style.clone(),
            move |_this, ctx| ctx.deactive(window),
        )
        .with_min_size([40.0, 25.0])
        .with_parent(content)
        .with_fill_x(RectFill::ShrinkCenter)
        .build();
        window
    };
    drop(painel);
    drop(button_style);
    drop(menu_button_style);
    drop(page_painel);

    println!("Starting");
    gui.start();
    println!("Started");

    // resize everthing to the screen size
    resize(
        &mut gui,
        &mut render,
        &mut camera,
        window.inner_size(),
        window.id(),
    );

    event_loop.run(move |event, _, control| {
        *control = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => {
                // gui receive events
                gui.handle_event(&event);
                if gui.render_is_dirty() {
                    window.request_redraw();
                }
                if let Some(cursor) = gui.cursor_change() {
                    window.set_cursor_icon(cursor);
                }
                match event {
                    WindowEvent::CloseRequested => *control = ControlFlow::Exit,
                    WindowEvent::Resized(size) => {
                        resize(&mut gui, &mut render, &mut camera, size, window_id);
                    }
                    _ => {}
                }
            }
            Event::UserEvent(()) => *control = ControlFlow::Exit,
            Event::RedrawRequested(window_id) if window_id == window.id() => {
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
    })
}

fn create_item(
    ctx: &mut Context,
    list: Id,
    texture: u32,
    text: String,
    color: [u8; 4],
    button_style: Rc<ButtonStyle>,
) {
    let painel: Graphic =
        Panel::new(texture, [0.0 / 2.0, 0.0 / 2.0, 1.0 / 2.0, 1.0 / 2.0], 10.0).into();
    let item = ctx
        .create_control()
        .with_min_size([100.0, 35.0])
        .with_graphic(painel.clone().with_color(color))
        .with_parent(list)
        .with_layout(HBoxLayout::new(0.0, [5.0, 0.0, 5.0, 0.0], 0))
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
        .with_graphic(painel)
        .with_behaviour(Button::new(button_style, move |_, ctx| {
            ctx.remove(item);
        }))
        .with_min_size([15.0, 15.0])
        .with_fill_x(RectFill::ShrinkCenter)
        .with_fill_y(RectFill::ShrinkCenter)
        .build();
}

fn create_button<F: Fn(Id, &mut Context) + 'static>(
    gui: &mut GUI,
    text: String,
    button_style: Rc<ButtonStyle>,
    on_click: F,
) -> ControlBuilder {
    let button_id: Id = gui.reserve_id();
    let _text = gui
        .create_control()
        .with_parent(button_id)
        .with_graphic(Text::new([40, 40, 100, 255], text, 16.0, (0, 0)).into())
        .with_layout(FitText)
        .build();
    gui.create_control_reserved(button_id)
        .with_behaviour(Button::new(button_style, on_click))
    // .with_layout(Box::new(MarginLayout::new([7.0, 7.0, 7.0, 7.0])))
}
