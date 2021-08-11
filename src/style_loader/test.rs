use super::*;
use crate::{
    graphics::{Graphic, Icon, Panel, Text, Texture},
    style::{ButtonStyle, OnFocusStyle},
};
use std::{collections::HashMap, rc::Rc};

struct MyLoader {
    avaliable_textures: HashMap<String, (u32, u32)>,
    textures: HashMap<String, (u32, u32, u32)>,
    loaded_fonts: HashMap<String, FontId>,
}
impl MyLoader {
    fn new(avl_text: Vec<(String, u32, u32)>) -> Self {
        let avaliable_textures = avl_text
            .into_iter()
            .map(|(name, w, h)| (name, (w, h)))
            .collect();
        let mut textures = HashMap::new();
        textures.insert("ERROR".to_string(), (1, 256, 256));
        Self {
            avaliable_textures,
            textures,
            loaded_fonts: Default::default(),
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
    fn load_font(&mut self, name: String) -> FontId {
        // let id = self.fonts.add(font);
        let id = FontId::new(self.loaded_fonts.len() as u32);
        self.loaded_fonts.insert(name, id);
        id
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
                    [0.0, 0.0000, 0.0625, 0.0625],
                    [0.0625, 0.0000, 0.0625, 0.0625],
                    [0.125, 0.0000, 0.0625, 0.0625],
                    [0.0, 0.0625, 0.0625, 0.0625],
                    [0.0625, 0.0625, 0.0625, 0.0625],
                    [0.125, 0.0625, 0.0625, 0.0625],
                    [0.0, 0.1250, 0.0625, 0.0625],
                    [0.0625, 0.1250, 0.0625, 0.0625],
                    [0.125, 0.1250, 0.0625, 0.0625],
                ],
                border: [16.0, 16.0, 16.0, 16.0],
                color: [255, 255, 255, 255].into(),
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
                    [0.0, 0.0000, 0.0625, 0.0625],
                    [0.0625, 0.0000, 0.0625, 0.0625],
                    [0.125, 0.0000, 0.0625, 0.0625],
                    [0.0, 0.0625, 0.0625, 0.0625],
                    [0.0625, 0.0625, 0.0625, 0.0625],
                    [0.125, 0.0625, 0.0625, 0.0625],
                    [0.0, 0.1250, 0.0625, 0.0625],
                    [0.0625, 0.1250, 0.0625, 0.0625],
                    [0.125, 0.1250, 0.0625, 0.0625],
                ],
                border: [16.0, 16.0, 16.0, 16.0],
                color: [255, 0, 170, 255].into(),
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
                color: [255, 255, 255, 255].into(),
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
                color: [255, 255, 255, 255].into(),
                color_dirty: true,
            })
        )
    );
}

#[test]
fn deserialize_animated_icon() {
    let my_loader = MyLoader::new(vec![("my_texture.png".into(), 256, 256)]);

    let mut deser = ron::de::Deserializer::from_str(
        r#"AnimatedIcon(
    texture: "my_texture.png",
    grid: (
        rect: (0, 0, 256, 256),
        cols: 4,
        rows: 2,
        len: 6,
    ),
    fps: 30.0,
    size: (18, 18),
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
            Graphic::from(AnimatedIcon {
                texture: 2,
                size: [18.0, 18.0],
                frames: vec![
                    [0.0, 0.0, 0.25, 0.5],
                    [0.25, 0.0, 0.25, 0.5],
                    [0.5, 0.0, 0.25, 0.5],
                    [0.75, 0.0, 0.25, 0.5],
                    [0.0, 0.5, 0.25, 0.5],
                    [0.25, 0.5, 0.25, 0.5]
                ],
                curr_time: 0.0,
                fps: 30.0,
                color: [255, 255, 255, 255].into(),
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
    text: "Hello World",
    align: (0, 0),
    style: (color: (255, 255, 255, 255), font_size: 16.0, font_id: "CascadiaCode")
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
                "Hello World".into(),
                (0, 0),
                TextStyle {
                    color: [255, 255, 255, 255].into(),
                    font_size: 16.0,
                    font_id: FontId::new(0),
                    ..Default::default()
                }
            ))
        )
    );
}

#[test]
fn my_style() {
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
        text: "My World",
        align: (-1, 0),
        style: (
            color: (255, 0, 0, 255),
            font_size: 16.0,
            font_id: "Consolas",
        )
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
        &format!("{:?}", my_style),
        r##"MyStyle { graphic1: Text(Text { text: "My World", text_dirty: true, glyphs: [], layout: None, min_size: None, last_pos: [0.0, 0.0], align: (-1, 0), color_dirty: true, style: TextStyle { color: Color { r: 255, g: 0, b: 0, a: 255 }, font_size: 16.0, font_id: FontId { index: 0 } } }), graphic2: Icon(Icon { texture: 2, uv_rect: [0.0, 0.0, 1.0, 1.0], size: [18.0, 18.0], color: Color { r: 255, g: 255, b: 255, a: 255 }, color_dirty: true }), button: ButtonStyle { normal: Panel(Panel { texture: 3, uv_rects: [[0.0, 0.0, 0.16666667, 0.16666667], [0.16666667, 0.0, 0.16666667, 0.16666667], [0.33333334, 0.0, 0.16666667, 0.16666667], [0.0, 0.16666667, 0.16666667, 0.16666667], [0.16666667, 0.16666667, 0.16666667, 0.16666667], [0.33333334, 0.16666667, 0.16666667, 0.16666667], [0.0, 0.33333334, 0.16666667, 0.16666667], [0.16666667, 0.33333334, 0.16666667, 0.16666667], [0.33333334, 0.33333334, 0.16666667, 0.16666667]], border: [10.0, 10.0, 10.0, 10.0], color: Color { r: 255, g: 255, b: 255, a: 255 }, color_dirty: true }), hover: Panel(Panel { texture: 3, uv_rects: [[0.5, 0.0, 0.16666667, 0.16666667], [0.6666667, 0.0, 0.16666667, 0.16666667], [0.8333333, 0.0, 0.16666667, 0.16666667], [0.5, 0.16666667, 0.16666667, 0.16666667], [0.6666667, 0.16666667, 0.16666667, 0.16666667], [0.8333333, 0.16666667, 0.16666667, 0.16666667], [0.5, 0.33333334, 0.16666667, 0.16666667], [0.6666667, 0.33333334, 0.16666667, 0.16666667], [0.8333333, 0.33333334, 0.16666667, 0.16666667]], border: [10.0, 10.0, 10.0, 10.0], color: Color { r: 255, g: 255, b: 255, a: 255 }, color_dirty: true }), pressed: Panel(Panel { texture: 3, uv_rects: [[0.0, 0.5, 0.16666667, 0.16666667], [0.16666667, 0.5, 0.16666667, 0.16666667], [0.33333334, 0.5, 0.16666667, 0.16666667], [0.0, 0.6666667, 0.16666667, 0.16666667], [0.16666667, 0.6666667, 0.16666667, 0.16666667], [0.33333334, 0.6666667, 0.16666667, 0.16666667], [0.0, 0.8333333, 0.16666667, 0.16666667], [0.16666667, 0.8333333, 0.16666667, 0.16666667], [0.33333334, 0.8333333, 0.16666667, 0.16666667]], border: [10.0, 10.0, 10.0, 10.0], color: Color { r: 255, g: 255, b: 255, a: 255 }, color_dirty: true }), focus: Panel(Panel { texture: 3, uv_rects: [[0.5, 0.5, 0.16666667, 0.16666667], [0.6666667, 0.5, 0.16666667, 0.16666667], [0.8333333, 0.5, 0.16666667, 0.16666667], [0.5, 0.6666667, 0.16666667, 0.16666667], [0.6666667, 0.6666667, 0.16666667, 0.16666667], [0.8333333, 0.6666667, 0.16666667, 0.16666667], [0.5, 0.8333333, 0.16666667, 0.16666667], [0.6666667, 0.8333333, 0.16666667, 0.16666667], [0.8333333, 0.8333333, 0.16666667, 0.16666667]], border: [10.0, 10.0, 10.0, 10.0], color: Color { r: 255, g: 255, b: 255, a: 255 }, color_dirty: true }) }, on_focus: OnFocusStyle { normal: Panel(Panel { texture: 4, uv_rects: [[0.0, 0.0, 0.16666667, 0.33333334], [0.16666667, 0.0, 0.16666667, 0.33333334], [0.33333334, 0.0, 0.16666667, 0.33333334], [0.0, 0.33333334, 0.16666667, 0.33333334], [0.16666667, 0.33333334, 0.16666667, 0.33333334], [0.33333334, 0.33333334, 0.16666667, 0.33333334], [0.0, 0.6666667, 0.16666667, 0.33333334], [0.16666667, 0.6666667, 0.16666667, 0.33333334], [0.33333334, 0.6666667, 0.16666667, 0.33333334]], border: [10.0, 10.0, 10.0, 10.0], color: Color { r: 255, g: 255, b: 255, a: 255 }, color_dirty: true }), focus: Panel(Panel { texture: 4, uv_rects: [[0.5, 0.0, 0.16666667, 0.33333334], [0.6666667, 0.0, 0.16666667, 0.33333334], [0.8333333, 0.0, 0.16666667, 0.33333334], [0.5, 0.33333334, 0.16666667, 0.33333334], [0.6666667, 0.33333334, 0.16666667, 0.33333334], [0.8333333, 0.33333334, 0.16666667, 0.33333334], [0.5, 0.6666667, 0.16666667, 0.33333334], [0.6666667, 0.6666667, 0.16666667, 0.33333334], [0.8333333, 0.6666667, 0.16666667, 0.33333334]], border: [10.0, 10.0, 10.0, 10.0], color: Color { r: 255, g: 255, b: 255, a: 255 }, color_dirty: true }) } }"##,
    );
}
