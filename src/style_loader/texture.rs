use super::*;

pub const FIELDS: &[&str] = &["texture", "uv_rect", "color"];
#[allow(non_camel_case_types)]
enum Field {
    Texture,
    UvRect,
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
            1u64 => Ok(Field::UvRect),
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
            "uv_rect" => Ok(Field::UvRect),
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
pub struct TextureVisitor<'a, 'b> {
    pub loader: &'a mut StyleLoader<'b>,
}
impl<'de, 'a, 'b: 'a> serde::de::Visitor<'de> for TextureVisitor<'a, 'b> {
    type Value = Texture;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Formatter::write_str(formatter, "struct Texture")
    }
    #[inline]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut texture: Option<String> = None;
        let mut uv_rect: Option<[i32; 4]> = None;
        let mut color = None;
        while let Some(key) = MapAccess::next_key::<Field>(&mut map)? {
            match key {
                Field::Texture => {
                    if Option::is_some(&texture) {
                        return Err(de::Error::duplicate_field("texture"));
                    }
                    texture = Some(map.next_value()?);
                }
                Field::UvRect => {
                    if Option::is_some(&uv_rect) {
                        return Err(de::Error::duplicate_field("uv_rect"));
                    }
                    uv_rect = Some(map.next_value()?);
                }
                Field::Color => {
                    if Option::is_some(&color) {
                        return Err(de::Error::duplicate_field("color"));
                    }
                    color = Some(map.next_value::<Color>()?);
                }
            }
        }
        let texture = texture.ok_or_else(|| de::Error::missing_field("texture"))?;
        let (texture, width, height) = self.loader.load_texture(texture);
        let uv_rect = uv_rect.ok_or_else(|| de::Error::missing_field("uv_rect"))?;
        let uv_rect = [
            uv_rect[0] as f32 / width as f32,
            uv_rect[1] as f32 / height as f32,
            uv_rect[2] as f32 / width as f32,
            uv_rect[3] as f32 / height as f32,
        ];
        let color = color.unwrap_or(Color::WHITE);
        Ok(Texture {
            texture,
            uv_rect,
            color,
            color_dirty: true,
        })
    }
}

impl<'a, 'b: 'a> LoadStyle<'a, 'b> for Texture {
    type Loader = TextureLoader<'a, 'b>;
    fn new_loader(loader: &'a mut StyleLoader<'b>) -> Self::Loader {
        TextureLoader { loader }
    }
}

pub struct TextureLoader<'a, 'b> {
    loader: &'a mut StyleLoader<'b>,
}
impl<'de, 'a, 'b> DeserializeSeed<'de> for TextureLoader<'a, 'b> {
    type Value = Texture;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde::Deserializer::deserialize_struct(
            deserializer,
            "Texture",
            FIELDS,
            TextureVisitor {
                loader: self.loader,
            },
        )
    }
}
