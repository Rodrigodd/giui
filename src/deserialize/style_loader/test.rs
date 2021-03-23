use super::*;
use crate::{
    graphics::{Graphic, Icon, Panel, Text, Texture},
    style::{ButtonStyle, OnFocusStyle},
};
use std::{collections::HashMap, rc::Rc};

struct MyLoader {
    avaliable_textures: HashMap<String, (u32, u32)>,
    textures: HashMap<String, (u32, u32, u32)>,
}
impl MyLoader {
    fn new(avl_text: Vec<(String, u32, u32)>) -> Self {
        let avaliable_textures = avl_text.into_iter().map(|(name, w, h)| (name, (w, h))).collect();
        let mut textures = HashMap::new();
        textures.insert("ERROR".to_string(), (1, 256, 256));
        Self {
            avaliable_textures,
            textures,
        }
    }
}
impl StyleLoaderCallback for MyLoader {
    fn load_texture(&mut self, name: String) -> (u32, u32, u32) {
        if let Some(&(w, h)) = self.avaliable_textures.get(&name) {
            let next = self.textures.len() as u32 + 1;
            *self.textures.entry(name).or_insert((next, w, h))
        } else {
            // return the error texture
            self.textures["ERROR"]
        }
    }
}

#[test]
fn deserialize_panel_a() {
    let my_loader = MyLoader::new(vec![("my_texture.png".into(), 256, 256)]);

    let mut deser = ron::de::Deserializer::from_str(
        r#"Panel(
    texture: "my_texture.png",
    border: 16,
    uv_rect: (0, 0, 48, 48),
    color: (255, 255, 255, 255),
)
"#,
    )
    .unwrap();

    let panel: Graphic = load_style(&mut deser, my_loader).unwrap();

    assert_eq!(
        format!("{:?}", panel),
        format!(
            "{:?}",
            Graphic::from(Panel {
                texture: 2,
                uv_rects: [
                    [0.0, 0.0000, 0.0625, 0.0625], [0.0625, 0.0000, 0.0625, 0.0625], [0.125, 0.0000, 0.0625, 0.0625],
                    [0.0, 0.0625, 0.0625, 0.0625], [0.0625, 0.0625, 0.0625, 0.0625], [0.125, 0.0625, 0.0625, 0.0625],
                    [0.0, 0.1250, 0.0625, 0.0625], [0.0625, 0.1250, 0.0625, 0.0625], [0.125, 0.1250, 0.0625, 0.0625],
                ],
                border: [16.0, 16.0, 16.0, 16.0],
                color: [255, 255, 255, 255],
                color_dirty: true,
            })
        )
    );
}

#[test]
fn deserialize_panel_b() {
    let my_loader = MyLoader::new(vec![("my_texture.png".into(), 256, 256)]);

    let mut deser = ron::de::Deserializer::from_str(
        r##"Panel(
    texture: "my_texture.png",
    border: (16, 16, 16, 16),
    uv_rect: (0, 0, 48, 48),
    color: "#ff00aa",
)
"##,
    )
    .unwrap();

    let panel: Graphic = load_style(&mut deser, my_loader).unwrap();

    assert_eq!(
        format!("{:?}", panel),
        format!(
            "{:?}",
            Graphic::from(Panel {
                texture: 2,
                uv_rects: [
                    [0.0, 0.0000, 0.0625, 0.0625], [0.0625, 0.0000, 0.0625, 0.0625], [0.125, 0.0000, 0.0625, 0.0625],
                    [0.0, 0.0625, 0.0625, 0.0625], [0.0625, 0.0625, 0.0625, 0.0625], [0.125, 0.0625, 0.0625, 0.0625],
                    [0.0, 0.1250, 0.0625, 0.0625], [0.0625, 0.1250, 0.0625, 0.0625], [0.125, 0.1250, 0.0625, 0.0625],
                ],
                border: [16.0, 16.0, 16.0, 16.0],
                color: [255, 0, 170, 255],
                color_dirty: true,
            })
        )
    );
}

#[test]
fn deserialize_texture() {
    let my_loader = MyLoader::new(vec![("my_texture.png".into(), 256, 256)]);

    let mut deser = ron::de::Deserializer::from_str(
        r#"Texture(
    texture: "my_texture.png",
    uv_rect: (0, 0, 64, 64),
    color: (255, 255, 255, 255),
)
"#,
    )
    .unwrap();

    let texture: Graphic = load_style(&mut deser, my_loader).unwrap();

    assert_eq!(
        format!("{:?}", texture),
        format!(
            "{:?}",
            Graphic::from(Texture {
                texture: 2,
                uv_rect: [0.0, 0.0, 0.25, 0.25],
                color: [255, 255, 255, 255],
                color_dirty: true,
            })
        )
    );
}

#[test]
fn deserialize_icon() {
    let my_loader = MyLoader::new(vec![("my_texture.png".into(), 256, 256)]);

    let mut deser = ron::de::Deserializer::from_str(
        r#"Icon(
    texture: "my_texture.png",
    size: (18, 18),
    uv_rect: (0, 0, 256, 256),
    color: (255, 255, 255, 255),
)
"#,
    )
    .unwrap();

    let icon: Graphic = load_style(&mut deser, my_loader).unwrap();

    assert_eq!(
        format!("{:?}", icon),
        format!(
            "{:?}",
            Graphic::from(Icon {
                texture: 2,
                size: [18.0, 18.0],
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: [255, 255, 255, 255],
                color_dirty: true,
            })
        )
    );
}

#[test]
fn deserialize_text() {
    let my_loader = MyLoader::new(vec![("my_texture.png".into(), 256, 256)]);

    let mut deser = ron::de::Deserializer::from_str(
        r#"Text(
    color: (255, 255, 255, 255),
    text: "Hello World",
    font_size: 16.0,
    align: (0, 0),
)
"#,
    )
    .unwrap();

    let text: Graphic = load_style(&mut deser, my_loader).unwrap();

    assert_eq!(
        format!("{:?}", text),
        format!(
            "{:?}",
            Graphic::from(Text::new(
                [255, 255, 255, 255],
                "Hello World".into(),
                16.0,
                (0, 0)
            ))
        )
    );
}

#[test]
fn a() {
    #[derive(LoadStyle, Debug)]
    #[crui(crate = "crate")]
    pub struct MyStyle {
        graphic1: Graphic,
        graphic2: Graphic,
        button: Rc<ButtonStyle>,
        on_focus: OnFocusStyle,
    }

    let my_loader = MyLoader::new(vec![
        ("icon.png".into(), 18, 18),
        ("button.png".into(), 60, 60),
        ("panel.png".into(), 60, 30),
    ]);

    let mut deser = ron::de::Deserializer::from_str(
        r##"MyStyle(
    graphic1: Text(
        color: (255, 0, 0, 255),
        text: "My World",
        font_size: 16.0,
        align: (-1, 0),
    ),
    graphic2: Icon (
        texture: "icon.png",
        size: (18, 18),
        uv_rect: (0, 0, 18, 18),
        color: (255, 255, 255, 255),
    ),
    button: ButtonStyle (
        normal: Panel(
            texture: "button.png",
            uv_rect: (0, 0, 30, 30),
            border: 10
        ),
        hover: Panel(
            texture: "button.png",
            uv_rect: (30, 0, 30, 30),
            border: 10
        ),
        pressed: Panel(
            texture: "button.png",
            uv_rect: (0, 30, 30, 30),
            border: 10
        ),
        focus: Panel(
            texture: "button.png",
            uv_rect: (30, 30, 30, 30),
            border: 10
        ),
    ),
    on_focus: OnFocusStyle (
        normal: Panel(
            texture: "panel.png",
            uv_rect: (0, 0, 30, 30),
            border: 10
        ),
        focus: Panel(
            texture: "panel.png",
            uv_rect: (30, 0, 30, 30),
            border: 10
        ),
    ),
)
"##,
    )
    .unwrap();

    let my_style: MyStyle = load_style(&mut deser, my_loader).unwrap();
    
    assert_eq!(
        format!("{:?}", my_style),
        format!(
            "{:?}",
            MyStyle {
                graphic1: Graphic::from(Text::new(
                    [255, 0, 0, 255],
                    "My World".into(),
                    16.0,
                    (-1, 0)
                )),
                graphic2: Graphic::from(
                    Icon {
                        texture: 2,
                        size: [18.0, 18.0],
                        uv_rect: [0.0, 0.0, 1.0, 1.0],
                        color: [255, 255, 255, 255],
                        color_dirty: true,
                    }
                ),
                button: Rc::new(ButtonStyle {
                    normal: Graphic::from(Panel::new(3, [0.0, 0.0, 0.5, 0.5], [10.0; 4])),
                    hover: Graphic::from(Panel::new(3, [0.5, 0.0, 0.5, 0.5], [10.0; 4])),
                    pressed: Graphic::from(Panel::new(3, [0.0, 0.5, 0.5, 0.5], [10.0; 4])),
                    focus: Graphic::from(Panel::new(3, [0.5, 0.5, 0.5, 0.5], [10.0; 4])),
                }),
                on_focus: OnFocusStyle {
                    normal: Panel::new(4, [0.0, 0.0, 0.5, 1.0], [10.0; 4]).into(),
                    focus: Panel::new(4, [0.5, 0.0, 0.5, 1.0], [10.0; 4]).into(),
                },
            }
        )
    );
}