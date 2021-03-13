use super::*;

pub const FIELDS: &[&str] = &[
    "texture",
    "uv_rects",
    "uv_rect",
    "border",
    "color",
    "color_dirty",
];
#[allow(non_camel_case_types)]
#[derive(Debug)]
enum Field {
    Texture,
    UVRect,
    UVRects,
    Border,
    Color,
}
struct FieldVisitor;
impl<'de> serde::de::Visitor<'de> for FieldVisitor {
    type Value = Field;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Formatter::write_str(formatter, "field identifier")
    }
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match value {
            0u64 => Ok(Field::Texture),
            1u64 => Ok(Field::UVRect),
            2u64 => Ok(Field::UVRects),
            3u64 => Ok(Field::Border),
            4u64 => Ok(Field::Color),
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Unsigned(value),
                &"field index 0 <= i < 5",
            )),
        }
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match value {
            "texture" => Ok(Field::Texture),
            "uv_rect" => Ok(Field::UVRect),
            "uv_rects" => Ok(Field::UVRects),
            "border" => Ok(Field::Border),
            "color" => Ok(Field::Color),
            _ => Err(de::Error::unknown_field(value, FIELDS)),
        }
    }
    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&String::from_utf8_lossy(value))
    }
}
impl<'de> serde::Deserialize<'de> for Field {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde::Deserializer::deserialize_identifier(deserializer, FieldVisitor)
    }
}

struct Border([i32; 4]);
struct BorderVisitor;
impl<'de> Visitor<'de> for BorderVisitor {
    type Value = Border;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Formatter::write_str(formatter, "i32 or [i32; 4]")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Border([v as i32; 4]))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        const EXPECT: &str = "[i32; 4], with 4 elements";

        Ok(Border([
            seq.next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &EXPECT))?,
            seq.next_element()?
                .ok_or_else(|| de::Error::invalid_length(1, &EXPECT))?,
            seq.next_element()?
                .ok_or_else(|| de::Error::invalid_length(2, &EXPECT))?,
            seq.next_element()?
                .ok_or_else(|| de::Error::invalid_length(3, &EXPECT))?,
        ]))
    }
}
impl<'de> Deserialize<'de> for Border {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(BorderVisitor)
    }
}

pub struct PanelVisitor<'a, C: StyleLoaderCallback> {
    pub loader: &'a mut StyleLoader<C>,
}
impl<'de, 'a, C: StyleLoaderCallback> serde::de::Visitor<'de> for PanelVisitor<'a, C> {
    type Value = Panel;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Formatter::write_str(formatter, "struct Panel")
    }

    #[allow(clippy::many_single_char_names)]
    #[inline]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut texture: Option<String> = None;
        let mut uv_rects: Option<[[i32; 4]; 9]> = None;
        let mut uv_rect: Option<[i32; 4]> = None;
        let mut border: Option<[i32; 4]> = None;
        let mut color = None;
        while let Some(key) = MapAccess::next_key::<Field>(&mut map)? {
            match dbg!(key) {
                Field::Texture => {
                    if Option::is_some(&texture) {
                        return Err(de::Error::duplicate_field("texture"));
                    }
                    texture = Some(map.next_value()?);
                }
                Field::UVRect => {
                    if Option::is_some(&uv_rect) {
                        return Err(de::Error::duplicate_field("uv_rect"));
                    }
                    uv_rect = Some(dbg!(map.next_value())?);
                }
                Field::UVRects => {
                    if Option::is_some(&uv_rects) {
                        return Err(de::Error::duplicate_field("uv_rects"));
                    }
                    uv_rects = Some(map.next_value()?);
                }
                Field::Border => {
                    if Option::is_some(&border) {
                        return Err(de::Error::duplicate_field("border"));
                    }
                    border = Some(map.next_value::<Border>()?.0);
                }
                Field::Color => {
                    if Option::is_some(&color) {
                        return Err(de::Error::duplicate_field("color"));
                    }
                    color = Some(map.next_value::<Color>()?.0);
                }
            }
        }
        let texture = texture.ok_or_else(|| de::Error::missing_field("texture"))?;
        let (texture, width, height) = self.loader.load_texture(texture);
        let border = border.ok_or_else(|| de::Error::missing_field("border"))?;
        let uv_rects = if let Some(uv_rect) = uv_rect {
            let w = uv_rect[2];
            let h = uv_rect[3];

            let x = [
                uv_rect[0],
                uv_rect[0] + border[0],
                uv_rect[1] + w - border[2],
            ];
            let y = [
                uv_rect[1],
                uv_rect[1] + border[1],
                uv_rect[1] + h - border[3],
            ];

            let w = [border[0], w - border[0] - border[2], border[2]];
            let h = [border[1], h - border[1] - border[3], border[3]];

            let mut uv_rects = [[0.0; 4]; 9];
            for (i, uv_rect) in uv_rects.iter_mut().enumerate() {
                let n = i % 3;
                let m = i / 3;
                *uv_rect = [
                    x[n] as f32 / width as f32,
                    y[m] as f32 / height as f32,
                    w[n] as f32 / width as f32,
                    h[m] as f32 / height as f32,
                ];
            }
            uv_rects
        } else {
            let uv_rects = uv_rects.ok_or_else(|| de::Error::missing_field("uv_rects"))?;
            let mut uvs = [[0.0; 4]; 9];
            for (i, uv_rect) in uv_rects.iter().enumerate() {
                uvs[i] = [
                    uv_rect[0] as f32 / width as f32,
                    uv_rect[1] as f32 / height as f32,
                    uv_rect[2] as f32 / width as f32,
                    uv_rect[3] as f32 / height as f32,
                ];
            }
            uvs
        };
        let color = color.ok_or_else(|| de::Error::missing_field("color"))?;
        Ok(Panel {
            texture,
            uv_rects,
            border: [
                border[0] as f32,
                border[1] as f32,
                border[2] as f32,
                border[3] as f32,
            ],
            color,
            color_dirty: true,
        })
    }
}

impl<'de, 'a, C: StyleLoaderCallback> DeserializeSeed<'de> for LoadStyle<'a, Panel, C> {
    type Value = Panel;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde::Deserializer::deserialize_struct(
            deserializer,
            "Panel",
            FIELDS,
            PanelVisitor {
                loader: self.loader,
            },
        )
    }
}
