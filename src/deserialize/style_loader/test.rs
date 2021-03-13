use super::*;
use crate::{
    graphics::{Graphic, Icon, Panel, Text, Texture},
    // style::{ButtonStyle, OnFocusStyle},
};
use std::collections::{HashMap, HashSet};

struct MyLoader {
    avaliable_textures: HashSet<String>,
    textures: HashMap<String, (u32, u32, u32)>,
}
impl MyLoader {
    fn new(avl_text: &[&str]) -> Self {
        let avaliable_textures = avl_text.iter().map(|x| x.to_string()).collect();
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
        if self.avaliable_textures.contains(&name) {
            let next = self.textures.len() as u32 + 1;
            *self.textures.entry(name).or_insert((next, 256, 256))
        } else {
            // return the error texture
            self.textures["ERROR"]
        }
    }
}

#[test]
fn deserialize_panel_a() {
    let my_loader = MyLoader::new(&["my_texture.png"]);

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
    let my_loader = MyLoader::new(&["my_texture.png"]);

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
    let my_loader = MyLoader::new(&["my_texture.png"]);

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
    let my_loader = MyLoader::new(&["my_texture.png"]);

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
    let my_loader = MyLoader::new(&["my_texture.png"]);

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