use super::*;

pub struct Color(pub [u8; 4]);

struct ColorVisitor;
impl<'de> Visitor<'de> for ColorVisitor {
    type Value = Color;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Formatter::write_str(formatter, "i32 or [i32; 4]")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if v.starts_with('#') {
            let v = v.trim_start_matches('#');
            let num = u32::from_str_radix(v, 16)
                .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(v), &"hexadecimal color"))?;
            if v.len() == 6 {
                let color = (num << 8 | 0xFF).to_be_bytes();
                Ok(Color(color))
            } else if v.len() == 8 {
                let color = num.to_be_bytes();
                Ok(Color(color))
            } else {
                Err(de::Error::invalid_value(
                    de::Unexpected::Str(v),
                    &"RGB or RGBA hexadecimal color",
                ))
            }
        } else {
            Err(de::Error::invalid_value(
                de::Unexpected::Str(v),
                &"hexadecimal color, starting with #",
            ))
        }
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        const EXPECT: &str = "[i32; 4], with 4 elements";

        Ok(Color([
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
impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ColorVisitor)
    }
}
