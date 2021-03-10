use super::*;

pub const FIELDS: &[&str] = &["texture", "uv_rects", "border", "color", "color_dirty"];
#[allow(non_camel_case_types)]
enum Field {
    Texture,
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
            1u64 => Ok(Field::UVRects),
            2u64 => Ok(Field::Border),
            3u64 => Ok(Field::Color),
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
pub struct PanelVisitor;
impl<'de> serde::de::Visitor<'de> for PanelVisitor {
    type Value = Panel;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Formatter::write_str(formatter, "struct Panel")
    }
    #[inline]
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        const EXPECT: &str = "struct Panel with 5 elements";
        let texture = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &EXPECT))?;
        let uv_rects = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &EXPECT))?;
        let border = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &EXPECT))?;
        let color = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &EXPECT))?;
        Ok(Panel {
            texture,
            uv_rects,
            border,
            color,
            color_dirty: true,
        })
    }
    #[inline]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut texture = None;
        let mut uv_rects = None;
        let mut border = None;
        let mut color = None;
        while let Some(key) = MapAccess::next_key::<Field>(&mut map)? {
            match key {
                Field::Texture => {
                    if Option::is_some(&texture) {
                        return Err(de::Error::duplicate_field("texture"));
                    }
                    texture = Some(map.next_value()?);
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
                    border = Some(map.next_value()?);
                }
                Field::Color => {
                    if Option::is_some(&color) {
                        return Err(de::Error::duplicate_field("color"));
                    }
                    color = Some(map.next_value()?);
                }
            }
        }
        let texture = texture.ok_or_else(|| de::Error::missing_field("texture"))?;
        let uv_rects = uv_rects.ok_or_else(|| de::Error::missing_field("uv_rects"))?;
        let border = border.ok_or_else(|| de::Error::missing_field("border"))?;
        let color = color.ok_or_else(|| de::Error::missing_field("color"))?;
        Ok(Panel {
            texture,
            uv_rects,
            border,
            color,
            color_dirty: true,
        })
    }
}

impl<'de> serde::Deserialize<'de> for Panel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde::Deserializer::deserialize_struct(deserializer, "Panel", FIELDS, PanelVisitor)
    }
}
