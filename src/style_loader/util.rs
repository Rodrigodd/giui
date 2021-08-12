use std::rc::Rc;

use super::*;

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
            let num = u32::from_str_radix(v, 16).map_err(|_| {
                de::Error::invalid_value(de::Unexpected::Str(v), &"hexadecimal color")
            })?;
            if v.len() == 6 {
                Ok(Color::from_u32(num << 8 | 0xFF))
            } else if v.len() == 8 {
                Ok(Color::from_u32(num))
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

        Ok(Color {
            r: seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &EXPECT))?,
            b: seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(1, &EXPECT))?,
            g: seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(2, &EXPECT))?,
            a: seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(3, &EXPECT))?,
        })
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

impl<'a, 'b: 'a> LoadStyle<'a, 'b> for Color {
    type Loader = ColorLoader;
    fn new_loader(_: &'a mut StyleLoader<'b>) -> Self::Loader {
        ColorLoader
    }
}

pub struct ColorLoader;
impl<'de> DeserializeSeed<'de> for ColorLoader {
    type Value = Color;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Color as Deserialize>::deserialize(deserializer)
    }
}

impl<'a, 'b: 'a, T: 'a> LoadStyle<'a, 'b> for Rc<T>
where
    T: LoadStyle<'a, 'b>,
{
    type Loader = RcLoader<'a, 'b, T>;
    fn new_loader(loader: &'a mut StyleLoader<'b>) -> Self::Loader {
        RcLoader {
            loader,
            _phantom: PhantomData::<fn() -> T>::default(),
        }
    }
}

pub struct RcLoader<'a, 'b, T> {
    loader: &'a mut StyleLoader<'b>,
    _phantom: PhantomData<fn() -> T>,
}
impl<'de, 'a, 'b, T> DeserializeSeed<'de> for RcLoader<'a, 'b, T>
where
    T: LoadStyle<'a, 'b>,
{
    type Value = Rc<T>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        DeserializeSeed::deserialize(<T as LoadStyle>::new_loader(self.loader), deserializer)
            .map(Rc::new)
    }
}

impl<'a, 'b: 'a, T: 'a> LoadStyle<'a, 'b> for Option<T>
where
    T: LoadStyle<'a, 'b>,
{
    type Loader = OptionLoader<'a, 'b, T>;
    fn new_loader(loader: &'a mut StyleLoader<'b>) -> Self::Loader {
        OptionLoader {
            loader,
            _phantom: PhantomData::<fn() -> T>::default(),
        }
    }
}

pub struct OptionLoader<'a, 'b, T> {
    loader: &'a mut StyleLoader<'b>,
    _phantom: PhantomData<fn() -> T>,
}
impl<'de, 'a, 'b, T> DeserializeSeed<'de> for OptionLoader<'a, 'b, T>
where
    T: LoadStyle<'a, 'b>,
{
    type Value = Option<T>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        DeserializeSeed::deserialize(<T as LoadStyle>::new_loader(self.loader), deserializer)
            .map(Some)
    }
}
