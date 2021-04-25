use super::*;

pub const FIELDS: &[&str] = &["text", "align", "style"];
#[allow(non_camel_case_types)]
enum Field {
    Text,
    Align,
    Style,
}
struct FieldVisitor;
impl<'de> serde::de::Visitor<'de> for FieldVisitor {
    type Value = Field;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Formatter::write_str(formatter, "field identifier")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match value {
            "text" => Ok(Field::Text),
            "align" => Ok(Field::Align),
            "style" => Ok(Field::Style),
            _ => Err(de::Error::unknown_field(value, FIELDS)),
        }
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
pub struct TextVisitor<'a, 'b> {
    pub loader: &'a mut StyleLoader<'b>,
}
impl<'de, 'a, 'b: 'a> serde::de::Visitor<'de> for TextVisitor<'a, 'b> {
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
        let mut align = None;
        let mut style = None;
        while let Some(key) = MapAccess::next_key::<Field>(&mut map)? {
            match key {
                Field::Text => {
                    if Option::is_some(&text) {
                        return Err(de::Error::duplicate_field("text"));
                    }
                    text = Some(map.next_value()?);
                }
                Field::Align => {
                    if Option::is_some(&align) {
                        return Err(de::Error::duplicate_field("align"));
                    }
                    align = Some(map.next_value()?);
                }
                Field::Style => {
                    if Option::is_some(&style) {
                        return Err(de::Error::duplicate_field("style"));
                    }
                    style = Some(map.next_value_seed(text_style::TextStyleLoader {
                        loader: self.loader,
                    })?);
                }
            }
        }
        let text = text.ok_or_else(|| de::Error::missing_field("text"))?;
        let align = align.ok_or_else(|| de::Error::missing_field("align"))?;
        let style = style.ok_or_else(|| de::Error::missing_field("style"))?;
        Ok(Text::new(text, align, style))
    }
}

impl<'a, 'b: 'a> LoadStyle<'a, 'b> for Text {
    type Loader = TextLoader<'a, 'b>;
    fn new_loader(loader: &'a mut StyleLoader<'b>) -> Self::Loader {
        TextLoader { loader }
    }
}

pub struct TextLoader<'a, 'b> {
    loader: &'a mut StyleLoader<'b>,
}
impl<'de, 'a, 'b> DeserializeSeed<'de> for TextLoader<'a, 'b> {
    type Value = Text;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde::Deserializer::deserialize_struct(
            deserializer,
            "Text",
            FIELDS,
            TextVisitor {
                loader: self.loader,
            },
        )
    }
}
