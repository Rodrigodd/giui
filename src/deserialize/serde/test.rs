use crate::graphics::{Graphic, Icon, Panel, Text, Texture};

#[test]
fn deserialize_panel() {
    let panel: Graphic = ron::from_str(
        r#"Panel(
    texture: 1,
    border: (10.0, 10.0, 10.0, 10.0),
    uv_rects: (
        (0.0, 0.0, 1.0, 1.0), (0.0, 0.0, 1.0, 1.0), (0.0, 0.0, 1.0, 1.0),
        (0.0, 0.0, 1.0, 1.0), (0.0, 0.0, 1.0, 1.0), (0.0, 0.0, 1.0, 1.0),
        (0.0, 0.0, 1.0, 1.0), (0.0, 0.0, 1.0, 1.0), (0.0, 0.0, 1.0, 1.0),
    ),
    color: (255, 255, 255, 255),
)
"#,
    )
    .unwrap();

    assert_eq!(
        format!("{:?}", panel),
        format!(
            "{:?}",
            Graphic::from(Panel {
                texture: 1,
                uv_rects: [
                    [0.0, 0.0, 1.0, 1.0],
                    [0.0, 0.0, 1.0, 1.0],
                    [0.0, 0.0, 1.0, 1.0],
                    [0.0, 0.0, 1.0, 1.0],
                    [0.0, 0.0, 1.0, 1.0],
                    [0.0, 0.0, 1.0, 1.0],
                    [0.0, 0.0, 1.0, 1.0],
                    [0.0, 0.0, 1.0, 1.0],
                    [0.0, 0.0, 1.0, 1.0],
                ],
                border: [10.0, 10.0, 10.0, 10.0],
                color: [255, 255, 255, 255],
                color_dirty: true,
            })
        )
    );
}

#[test]
fn deserialize_texture() {
    let texture: Graphic = ron::from_str(
        r#"Texture(
    texture: 1,
    uv_rect: (0.0, 0.0, 1.0, 1.0),
    color: (255, 255, 255, 255),
)
"#,
    )
    .unwrap();

    assert_eq!(
        format!("{:?}", texture),
        format!(
            "{:?}",
            Graphic::from(Texture {
                texture: 1,
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: [255, 255, 255, 255],
                color_dirty: true,
            })
        )
    );
}

#[test]
fn deserialize_icon() {
    let icon: Graphic = ron::from_str(
        r#"Icon(
    texture: 1,
    size: (10.0, 10.0),
    uv_rect: (0.0, 0.0, 1.0, 1.0),
    color: (255, 255, 255, 255),
)
"#,
    )
    .unwrap();

    assert_eq!(
        format!("{:?}", icon),
        format!(
            "{:?}",
            Graphic::from(Icon {
                texture: 1,
                size: [10.0, 10.0],
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: [255, 255, 255, 255],
                color_dirty: true,
            })
        )
    );
}

#[test]
fn deserialize_text() {
    let text: Graphic = ron::from_str(
        r#"Text(
    color: (255, 255, 255, 255),
    text: "Hello World",
    font_size: 16.0,
    align: (0, 0),
)
"#,
    )
    .unwrap();

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

// #[test]
// fn deserialize_my_style() {
//     #[derive(serde::Deserialize, Debug)]
//     struct MyStyle {
//         white: Graphic,
//         button: ButtonStyle,
//         focus: OnFocusStyle,
//         my_color: [u8; 4],
//     }

//     let text: MyStyle = ron::from_str(
//         r#"MyStyle(
//     white: Texture(
//         texture: 1,
//         uv_rect: (0.0, 0.0, 1.0, 1.0),
//         color: (255, 255, 255, 255),
//     ),
//     button: ButtonStyle(
//         focus: Panel(
//             texture: 1,
//             uv_rects: (
//                 (0.0, 0.0, 0.3, 0.3), (0.3, 0.0, 0.4, 0.3), (0.7, 0.0, 0.3, 0.3),
//                 (0.0, 0.3, 0.3, 0.4), (0.3, 0.3, 0.4, 0.4), (0.7, 0.3, 0.3, 0.4),
//                 (0.0, 0.7, 0.3, 0.3), (0.3, 0.7, 0.4, 0.3), (0.7, 0.7, 0.3, 0.3),
//             ),
//             border: (10.0, 10.0, 10.0, 10.0),
//             color: (200, 200, 255, 255),
//         ),
//         hover: Panel(
//             texture: 1,
//             uv_rects: (
//                 (0.0, 0.0, 0.3, 0.3), (0.3, 0.0, 0.4, 0.3), (0.7, 0.0, 0.3, 0.3),
//                 (0.0, 0.3, 0.3, 0.4), (0.3, 0.3, 0.4, 0.4), (0.7, 0.3, 0.3, 0.4),
//                 (0.0, 0.7, 0.3, 0.3), (0.3, 0.7, 0.4, 0.3), (0.7, 0.7, 0.3, 0.3),
//             ),
//             border: (10.0, 10.0, 10.0, 10.0),
//             color: (200, 200, 200, 255),
//         ),
//         normal: Panel(
//             texture: 1,
//             uv_rects: (
//                 (0.0, 0.0, 0.3, 0.3), (0.3, 0.0, 0.4, 0.3), (0.7, 0.0, 0.3, 0.3),
//                 (0.0, 0.3, 0.3, 0.4), (0.3, 0.3, 0.4, 0.4), (0.7, 0.3, 0.3, 0.4),
//                 (0.0, 0.7, 0.3, 0.3), (0.3, 0.7, 0.4, 0.3), (0.7, 0.7, 0.3, 0.3),
//             ),
//             border: (10.0, 10.0, 10.0, 10.0),
//             color: (255, 255, 255, 255),
//         ),
//         pressed: Panel(
//             texture: 1,
//             uv_rects: (
//                 (0.0, 0.0, 0.3, 0.3), (0.3, 0.0, 0.4, 0.3), (0.7, 0.0, 0.3, 0.3),
//                 (0.0, 0.3, 0.3, 0.4), (0.3, 0.3, 0.4, 0.4), (0.7, 0.3, 0.3, 0.4),
//                 (0.0, 0.7, 0.3, 0.3), (0.3, 0.7, 0.4, 0.3), (0.7, 0.7, 0.3, 0.3),
//             ),
//             border: (10.0, 10.0, 10.0, 10.0),
//             color: (150, 150, 150, 255),
//         ),
//     ),
//     focus: OnFocusStyle(
//         focus: Panel(
//             texture: 1,
//             uv_rects: (
//                 (0.0, 0.0, 0.3, 0.3), (0.3, 0.0, 0.4, 0.3), (0.7, 0.0, 0.3, 0.3),
//                 (0.0, 0.3, 0.3, 0.4), (0.3, 0.3, 0.4, 0.4), (0.7, 0.3, 0.3, 0.4),
//                 (0.0, 0.7, 0.3, 0.3), (0.3, 0.7, 0.4, 0.3), (0.7, 0.7, 0.3, 0.3),
//             ),
//             border: (10.0, 10.0, 10.0, 10.0),
//             color: (200, 200, 255, 255),
//         ),
//         normal: Panel(
//             texture: 1,
//             uv_rects: (
//                 (0.0, 0.0, 0.3, 0.3), (0.3, 0.0, 0.4, 0.3), (0.7, 0.0, 0.3, 0.3),
//                 (0.0, 0.3, 0.3, 0.4), (0.3, 0.3, 0.4, 0.4), (0.7, 0.3, 0.3, 0.4),
//                 (0.0, 0.7, 0.3, 0.3), (0.3, 0.7, 0.4, 0.3), (0.7, 0.7, 0.3, 0.3),
//             ),
//             border: (10.0, 10.0, 10.0, 10.0),
//             color: (255, 255, 255, 255),
//         ),
//     ),
//     my_color: (0, 155, 255, 255),
// )
// "#,
//     )
//     .unwrap();

//     #[rustfmt::skip]
//     assert_eq!(
//         format!("{:?}", text),
//         format!(
//             "{:?}",
//             MyStyle {
//                 white: Graphic::Texture(Texture {
//                     texture: 1,
//                     uv_rect: [0.0, 0.0, 1.0, 1.0],
//                     color: [255, 255, 255, 255],
//                     color_dirty: true,
//                 }),
//                 button: ButtonStyle {
//                     focus: Graphic::from(Panel {
//                         texture: 1,
//                         uv_rects: [
//                             [0.0, 0.0, 0.3, 0.3], [0.3, 0.0, 0.4, 0.3], [0.7, 0.0, 0.3, 0.3],
//                             [0.0, 0.3, 0.3, 0.4], [0.3, 0.3, 0.4, 0.4], [0.7, 0.3, 0.3, 0.4],
//                             [0.0, 0.7, 0.3, 0.3], [0.3, 0.7, 0.4, 0.3], [0.7, 0.7, 0.3, 0.3],
//                         ],
//                         border: [10.0, 10.0, 10.0, 10.0],
//                         color: [200, 200, 255, 255],
//                         color_dirty: true,
//                     }),
//                     hover: Graphic::from(Panel {
//                         texture: 1,
//                         uv_rects: [
//                             [0.0, 0.0, 0.3, 0.3], [0.3, 0.0, 0.4, 0.3], [0.7, 0.0, 0.3, 0.3],
//                             [0.0, 0.3, 0.3, 0.4], [0.3, 0.3, 0.4, 0.4], [0.7, 0.3, 0.3, 0.4],
//                             [0.0, 0.7, 0.3, 0.3], [0.3, 0.7, 0.4, 0.3], [0.7, 0.7, 0.3, 0.3],
//                         ],
//                         border: [10.0, 10.0, 10.0, 10.0],
//                         color: [200, 200, 200, 255],
//                         color_dirty: true,
//                     }),
//                     normal: Graphic::from(Panel {
//                         texture: 1,
//                         uv_rects: [
//                             [0.0, 0.0, 0.3, 0.3], [0.3, 0.0, 0.4, 0.3], [0.7, 0.0, 0.3, 0.3],
//                             [0.0, 0.3, 0.3, 0.4], [0.3, 0.3, 0.4, 0.4], [0.7, 0.3, 0.3, 0.4],
//                             [0.0, 0.7, 0.3, 0.3], [0.3, 0.7, 0.4, 0.3], [0.7, 0.7, 0.3, 0.3],
//                         ],
//                         border: [10.0, 10.0, 10.0, 10.0],
//                         color: [255, 255, 255, 255],
//                         color_dirty: true,
//                     }),
//                     pressed: Graphic::from(Panel {
//                         texture: 1,
//                         uv_rects: [
//                             [0.0, 0.0, 0.3, 0.3], [0.3, 0.0, 0.4, 0.3], [0.7, 0.0, 0.3, 0.3],
//                             [0.0, 0.3, 0.3, 0.4], [0.3, 0.3, 0.4, 0.4], [0.7, 0.3, 0.3, 0.4],
//                             [0.0, 0.7, 0.3, 0.3], [0.3, 0.7, 0.4, 0.3], [0.7, 0.7, 0.3, 0.3],
//                         ],
//                         border: [10.0, 10.0, 10.0, 10.0],
//                         color: [150, 150, 150, 255],
//                         color_dirty: true,
//                     }),
//                 },
//                 focus: OnFocusStyle {
//                     focus: Graphic::from(Panel {
//                         texture: 1,
//                         uv_rects: [
//                             [0.0, 0.0, 0.3, 0.3], [0.3, 0.0, 0.4, 0.3], [0.7, 0.0, 0.3, 0.3],
//                             [0.0, 0.3, 0.3, 0.4], [0.3, 0.3, 0.4, 0.4], [0.7, 0.3, 0.3, 0.4],
//                             [0.0, 0.7, 0.3, 0.3], [0.3, 0.7, 0.4, 0.3], [0.7, 0.7, 0.3, 0.3],
//                         ],
//                         border: [10.0, 10.0, 10.0, 10.0],
//                         color: [200, 200, 255, 255],
//                         color_dirty: true,
//                     }),
//                     normal: Graphic::from(Panel {
//                         texture: 1,
//                         uv_rects: [
//                             [0.0, 0.0, 0.3, 0.3], [0.3, 0.0, 0.4, 0.3], [0.7, 0.0, 0.3, 0.3],
//                             [0.0, 0.3, 0.3, 0.4], [0.3, 0.3, 0.4, 0.4], [0.7, 0.3, 0.3, 0.4],
//                             [0.0, 0.7, 0.3, 0.3], [0.3, 0.7, 0.4, 0.3], [0.7, 0.7, 0.3, 0.3],
//                         ],
//                         border: [10.0, 10.0, 10.0, 10.0],
//                         color: [255, 255, 255, 255],
//                         color_dirty: true,
//                     }),
//                 },
//                 my_color: [0, 155, 255, 255],
//             }
//         )
//     );
// }
