use std::{fmt, marker::PhantomData};

use serde::{
    de::{
        self, Deserialize, DeserializeSeed, EnumAccess, Error, MapAccess, Unexpected,
        VariantAccess, Visitor,
    },
    Deserializer,
};

use crate::{
    font::FontId,
    graphics::{AnimatedIcon, Graphic, Icon, Panel, Text, TextStyle, Texture},
};

#[cfg(test)]
mod test;

mod animated_icon;
mod font_id;
mod icon;
mod panel;
mod text;
mod text_style;
mod texture;
pub mod util;

use animated_icon::{AnimatedIconVisitor, FIELDS as ANIMATED_ICON_FIELDS};
use icon::{IconVisitor, FIELDS as ICON_FIELDS};
use panel::{PanelVisitor, FIELDS as PANEL_FIELDS};
use text::{TextVisitor, FIELDS as TEXT_FIELDS};
use texture::{TextureVisitor, FIELDS as TEXTURE_FIELDS};

use crate::Color;
pub trait StyleLoaderCallback {
    /// Loads a texture and returns its (index, width, height).
    fn load_texture(&mut self, name: String) -> (u32, u32, u32);
    /// Loads and font and return its FontId.
    fn load_font(&mut self, name: String) -> FontId;
    /// Modify each loaded Graphic.
    fn modify_graphic(&mut self, graphic: &mut Graphic) {
        let _ = graphic;
    }
}

pub struct StyleLoader<'l> {
    callback: Box<dyn StyleLoaderCallback + 'l>,
}
impl<'l> StyleLoader<'l> {
    fn new<C: StyleLoaderCallback + 'l>(callback: C) -> Self {
        Self {
            callback: Box::new(callback),
        }
    }
    /// Loads a texture and returns its (index, width, height).
    fn load_texture(&mut self, name: String) -> (u32, u32, u32) {
        self.callback.load_texture(name)
    }
    /// Loads and font and return its FontId.
    fn load_font(&mut self, name: String) -> FontId {
        self.callback.load_font(name)
    }
    /// Modify each loaded Graphic.
    fn modify_graphic(&mut self, graphic: &mut Graphic) {
        self.callback.modify_graphic(graphic)
    }
}

pub trait LoadStyle<'a, 'b> {
    type Loader: for<'de> DeserializeSeed<'de, Value = Self> + 'a;
    fn new_loader(loader: &'a mut StyleLoader<'b>) -> Self::Loader;
}

pub fn load_style<'de, T, D, C>(deserializer: D, callback: C) -> Result<T, D::Error>
where
    C: StyleLoaderCallback,
    D: Deserializer<'de> + 'de,
    // this bound should be something like `T: for<'b where 'b: 'a> LoadStyle<'b, 'a>`, but this does not exist (yet?).
    T: for<'b> LoadStyle<'b, 'static>,
{
    let mut loader: StyleLoader = unsafe { std::mem::transmute(StyleLoader::new(callback)) };
    let load = <T as LoadStyle>::new_loader(&mut loader);
    DeserializeSeed::deserialize(load, deserializer)
}

impl<'a, 'b: 'a> LoadStyle<'a, 'b> for Graphic {
    type Loader = GraphicLoader<'a, 'b>;
    fn new_loader(loader: &'a mut StyleLoader<'b>) -> Self::Loader {
        GraphicLoader { loader }
    }
}

pub struct GraphicLoader<'a, 'b> {
    loader: &'a mut StyleLoader<'b>,
}
impl<'de, 'a, 'b> DeserializeSeed<'de> for GraphicLoader<'a, 'b> {
    type Value = Graphic;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        const VARIANTS: &[&str] = &["Panel", "Texture", "Icon", "Text", "None"];
        #[allow(non_camel_case_types)]
        enum Field {
            Panel,
            Texture,
            Icon,
            AnimatedIcon,
            Text,
            None,
        }

        struct FieldVisitor;
        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = Field;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                fmt::Formatter::write_str(formatter, "variant identifier")
            }
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value {
                    0u64 => Ok(Field::Panel),
                    1u64 => Ok(Field::Texture),
                    2u64 => Ok(Field::Icon),
                    5u64 => Ok(Field::AnimatedIcon),
                    3u64 => Ok(Field::Text),
                    4u64 => Ok(Field::None),
                    _ => Err(Error::invalid_value(
                        Unexpected::Unsigned(value),
                        &"variant index 0 <= i < 5",
                    )),
                }
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value {
                    "Panel" => Ok(Field::Panel),
                    "Texture" => Ok(Field::Texture),
                    "Icon" => Ok(Field::Icon),
                    "AnimatedIcon" => Ok(Field::AnimatedIcon),
                    "Text" => Ok(Field::Text),
                    "None" => Ok(Field::None),
                    _ => Err(Error::unknown_variant(value, VARIANTS)),
                }
            }
            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value {
                    b"Panel" => Ok(Field::Panel),
                    b"Texture" => Ok(Field::Texture),
                    b"Icon" => Ok(Field::Icon),
                    b"AnimatedIcon" => Ok(Field::AnimatedIcon),
                    b"Text" => Ok(Field::Text),
                    b"None" => Ok(Field::None),
                    _ => {
                        let value = &String::from_utf8_lossy(value);
                        Err(Error::unknown_variant(value, VARIANTS))
                    }
                }
            }
        }
        impl<'de> serde::Deserialize<'de> for Field {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserializer::deserialize_identifier(deserializer, FieldVisitor)
            }
        }
        struct GraphicVisitor<'a, 'b> {
            loader: &'a mut StyleLoader<'b>,
        }
        impl<'de, 'a, 'b: 'a> Visitor<'de> for GraphicVisitor<'a, 'b> {
            type Value = Graphic;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                fmt::Formatter::write_str(formatter, "enum Graphic")
            }
            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                match EnumAccess::variant(data)? {
                    (Field::Panel, variant) => VariantAccess::struct_variant(
                        variant,
                        PANEL_FIELDS,
                        PanelVisitor {
                            loader: self.loader,
                        },
                    )
                    .map(Graphic::Panel),
                    (Field::Texture, variant) => VariantAccess::struct_variant(
                        variant,
                        TEXTURE_FIELDS,
                        TextureVisitor {
                            loader: self.loader,
                        },
                    )
                    .map(Graphic::Texture),
                    (Field::Icon, variant) => VariantAccess::struct_variant(
                        variant,
                        ICON_FIELDS,
                        IconVisitor {
                            loader: self.loader,
                        },
                    )
                    .map(Graphic::Icon),
                    (Field::AnimatedIcon, variant) => VariantAccess::struct_variant(
                        variant,
                        ANIMATED_ICON_FIELDS,
                        AnimatedIconVisitor {
                            loader: self.loader,
                        },
                    )
                    .map(Graphic::AnimatedIcon),
                    (Field::Text, variant) => VariantAccess::struct_variant(
                        variant,
                        TEXT_FIELDS,
                        TextVisitor {
                            loader: self.loader,
                        },
                    )
                    .map(Graphic::Text),
                    (Field::None, variant) => {
                        VariantAccess::unit_variant(variant)?;
                        Ok(Graphic::None)
                    } // (Field::Panel, variant) => Result::map(
                      //     VariantAccess::newtype_variant::<Panel>(variant),
                      //     Graphic::Panel,
                      // ),
                      // (Field::Texture, variant) => Result::map(
                      //     VariantAccess::newtype_variant::<Texture>(variant),
                      //     Graphic::Texture,
                      // ),
                      // (Field::Icon, variant) => Result::map(
                      //     VariantAccess::newtype_variant::<Icon>(variant),
                      //     Graphic::Icon,
                      // ),
                      // (Field::Text, variant) => Result::map(
                      //     VariantAccess::newtype_variant::<Text>(variant),
                      //     Graphic::Text,
                      // ),
                      // (Field::None, variant) => {
                      //     VariantAccess::unit_variant(variant)?;
                      //     Ok(Graphic::None)
                      // }
                }
            }
        }

        Deserializer::deserialize_enum(
            deserializer,
            "Graphic",
            VARIANTS,
            GraphicVisitor {
                loader: self.loader,
            },
        )
        .map(|mut graphic| {
            self.loader.modify_graphic(&mut graphic);
            graphic
        })
    }
}
