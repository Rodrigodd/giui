use super::*;

pub const FIELDS: &[&str] = &["text", "font_size", "align", "color", "color_dirty"];
#[allow(non_camel_case_types)]
enum Field {
    Text,
    FontSize,
    Color,
    Align,
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
            1u64 => Ok(Field::Text),
            2u64 => Ok(Field::FontSize),
            3u64 => Ok(Field::Color),
            4u64 => Ok(Field::Align),
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
            "text" => Ok(Field::Text),
            "font_size" => Ok(Field::FontSize),
            "color" => Ok(Field::Color),
            "align" => Ok(Field::Align),
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
pub struct TextVisitor<'a, C: StyleLoaderCallback> {
    pub loader: &'a mut StyleLoader<C>,
}
impl<'de, 'a, C: StyleLoaderCallback> serde::de::Visitor<'de> for TextVisitor<'a, C> {
    type Value = Text;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Formatter::write_str(formatter, "struct Text")
    }
    #[inline]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut text = None;
        let mut font_size = None;
        let mut align = None;
        let mut color = None;
        while let Some(key) = MapAccess::next_key::<Field>(&mut map)? {
            match key {
                Field::Text => {
                    if Option::is_some(&text) {
                        return Err(de::Error::duplicate_field("text"));
                    }
                    text = Some(map.next_value()?);
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
                Field::Align => {
                    if Option::is_some(&align) {
                        return Err(de::Error::duplicate_field("align"));
                    }
                    align = Some(map.next_value()?);
                }
            }
        }
        let text = text.ok_or_else(|| de::Error::missing_field("text"))?;
        let font_size = font_size.ok_or_else(|| de::Error::missing_field("font_size"))?;
        let align = align.ok_or_else(|| de::Error::missing_field("align"))?;
        let color = color.ok_or_else(|| de::Error::missing_field("color"))?;
        Ok(Text::new(color, text, font_size, align))
    }
}

impl<'de, 'a, C: StyleLoaderCallback> DeserializeSeed<'de> for LoadStyle<'a, Text, C> {
    type Value = Text;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde::Deserializer::deserialize_struct(deserializer, "Text", FIELDS, TextVisitor { loader: self.loader })
    }
}
