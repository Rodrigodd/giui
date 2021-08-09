use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Grid {
    rect: [i32; 4],
    rows: u32,
    cols: u32,
    #[serde(default)]
    #[serde(deserialize_with = "grid_len")]
    len: Option<u32>,
}

fn grid_len<'de, D>(d: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(Option::Some)
}

pub const FIELDS: &[&str] = &[
    "texture",
    "frames",
    "fps",
    "grid",
    "size",
    "color",
    "color_dirty",
];
#[allow(non_camel_case_types)]
enum Field {
    Texture,
    Frames,
    Fps,
    Grid,
    Size,
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
            1u64 => Ok(Field::Frames),
            2u64 => Ok(Field::Grid),
            3u64 => Ok(Field::Size),
            4u64 => Ok(Field::Color),
            5u64 => Ok(Field::Fps),
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
            "frames" => Ok(Field::Frames),
            "fps" => Ok(Field::Fps),
            "grid" => Ok(Field::Grid),
            "size" => Ok(Field::Size),
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

pub struct AnimatedIconVisitor<'a, 'b> {
    pub loader: &'a mut StyleLoader<'b>,
}
impl<'de, 'a, 'b: 'a> serde::de::Visitor<'de> for AnimatedIconVisitor<'a, 'b> {
    type Value = AnimatedIcon;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Formatter::write_str(formatter, "struct AnimatedIcon")
    }
    #[inline]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut texture: Option<String> = None;
        let mut frames: Option<Vec<[i32; 4]>> = None;
        let mut fps: Option<f32> = None;
        let mut size = None;
        let mut color = None;
        while let Some(key) = MapAccess::next_key::<Field>(&mut map)? {
            match key {
                Field::Texture => {
                    if Option::is_some(&texture) {
                        return Err(de::Error::duplicate_field("texture"));
                    }
                    texture = Some(map.next_value()?);
                }
                Field::Frames => {
                    if Option::is_some(&frames) {
                        return Err(de::Error::duplicate_field("frames"));
                    }
                    frames = Some(map.next_value()?);
                }
                Field::Fps => {
                    if Option::is_some(&fps) {
                        return Err(de::Error::duplicate_field("fps"));
                    }
                    fps = Some(map.next_value()?);
                }
                Field::Grid => {
                    if Option::is_some(&frames) {
                        return Err(de::Error::duplicate_field("grid"));
                    }
                    let Grid {
                        rect,
                        rows,
                        cols,
                        len,
                    } = map.next_value()?;
                    let len = len.unwrap_or(rows * cols) as usize;
                    let mut grid_frames = Vec::with_capacity(len);
                    let w = (rect[2] - rect[0]) / cols as i32;
                    let h = (rect[3] - rect[1]) / rows as i32;
                    'grid: for y in 0..rows as i32 {
                        for x in 0..cols as i32 {
                            grid_frames.push([rect[0] + w * x, rect[1] + h * y, w, h]);
                            if grid_frames.len() == len {
                                break 'grid;
                            }
                        }
                    }
                    frames = Some(grid_frames);
                }
                Field::Size => {
                    if Option::is_some(&size) {
                        return Err(de::Error::duplicate_field("size"));
                    }
                    size = Some(map.next_value()?);
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
        let frames = frames.ok_or_else(|| de::Error::missing_field("frames"))?;
        let frames = frames
            .into_iter()
            .map(|frame| {
                [
                    frame[0] as f32 / width as f32,
                    frame[1] as f32 / height as f32,
                    frame[2] as f32 / width as f32,
                    frame[3] as f32 / height as f32,
                ]
            })
            .collect();
        let size = size.ok_or_else(|| de::Error::missing_field("size"))?;
        let color = color.unwrap_or([255; 4].into());
        let fps = fps.unwrap_or(60.0);
        Ok(AnimatedIcon {
            texture,
            frames,
            curr_time: 0.0,
            fps,
            size,
            color,
            color_dirty: true,
        })
    }
}

impl<'a, 'b: 'a> LoadStyle<'a, 'b> for AnimatedIcon {
    type Loader = AnimatedIconLoader<'a, 'b>;
    fn new_loader(loader: &'a mut StyleLoader<'b>) -> Self::Loader {
        AnimatedIconLoader { loader }
    }
}

pub struct AnimatedIconLoader<'a, 'b> {
    loader: &'a mut StyleLoader<'b>,
}
impl<'de, 'a, 'b> DeserializeSeed<'de> for AnimatedIconLoader<'a, 'b> {
    type Value = AnimatedIcon;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde::Deserializer::deserialize_struct(
            deserializer,
            "AnimatedIcon",
            FIELDS,
            AnimatedIconVisitor {
                loader: self.loader,
            },
        )
    }
}
