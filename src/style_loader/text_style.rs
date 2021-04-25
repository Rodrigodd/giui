use super::*;

pub const FIELDS: &[&str] = &["font_id", "font_size", "size"];
#[allow(non_camel_case_types)]
enum Field {
    FontId,
    FontSize,
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
            0u64 => Ok(Field::FontId),
            1u64 => Ok(Field::FontSize),
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
            "font_id" => Ok(Field::FontId),
            "font_size" => Ok(Field::FontSize),
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
pub struct TextStyleVisitor<'a, 'b> {
    pub loader: &'a mut StyleLoader<'b>,
}
impl<'de, 'a, 'b: 'a> serde::de::Visitor<'de> for TextStyleVisitor<'a, 'b> {
    type Value = TextStyle;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Formatter::write_str(formatter, "struct TextStyle")
    }
    #[inline]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut font_id: Option<FontId> = None;
        let mut font_size: Option<f32> = None;
        let mut color = None;
        while let Some(key) = MapAccess::next_key::<Field>(&mut map)? {
            match key {
                Field::FontId => {
                    if Option::is_some(&font_id) {
                        return Err(de::Error::duplicate_field("font_id"));
                    }
                    font_id = Some(map.next_value_seed(font_id::FontIdLoader {
                        loader: self.loader,
                    })?);
                }
                Field::FontSize => {
                    if Option::is_some(&font_size) {
                        return Err(de::Error::duplicate_field("font_size"));
                    }
                    font_size = Some(map.next_value()?);
                }
                Field::Color => {
                    if Option::is_some(&color) {
                        return Err(de::Error::duplicate_field("color"));
                    }
                    color = Some(map.next_value::<Color>()?.0);
                }
            }
        }
        let font_id = font_id.ok_or_else(|| de::Error::missing_field("font_id"))?;
        let font_size = font_size.ok_or_else(|| de::Error::missing_field("font_size"))?;
        let color = color.unwrap_or([255; 4]);
        Ok(TextStyle {
            font_id,
            font_size,
            color,
        })
    }
}

impl<'a, 'b: 'a> LoadStyle<'a, 'b> for TextStyle {
    type Loader = TextStyleLoader<'a, 'b>;
    fn new_loader(loader: &'a mut StyleLoader<'b>) -> Self::Loader {
        TextStyleLoader { loader }
    }
}

pub struct TextStyleLoader<'a, 'b> {
    pub loader: &'a mut StyleLoader<'b>,
}
impl<'de, 'a, 'b> DeserializeSeed<'de> for TextStyleLoader<'a, 'b> {
    type Value = TextStyle;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde::Deserializer::deserialize_struct(
            deserializer,
            "TextStyle",
            FIELDS,
            TextStyleVisitor {
                loader: self.loader,
            },
        )
    }
}
