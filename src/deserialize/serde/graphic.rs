use std::fmt;

use crate::graphics::{Graphic, Icon, Panel, Text, Texture};
use serde::{
    de::{self, EnumAccess, Error, MapAccess, Unexpected, VariantAccess, Visitor},
    Deserialize, Deserializer,
};

mod panel;
use panel::{PanelVisitor, FIELDS as PANEL_FIELDS};

mod texture;
use texture::{TextureVisitor, FIELDS as TEXTURE_FIELDS};

mod icon;
use icon::{IconVisitor, FIELDS as ICON_FIELDS};

mod text;
use text::{TextVisitor, FIELDS as TEXT_FIELDS};

impl<'de> Deserialize<'de> for Graphic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const VARIANTS: &[&str] = &["Panel", "Texture", "Icon", "Text", "None"];
        #[allow(non_camel_case_types)]
        enum Field {
            Panel,
            Texture,
            Icon,
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
        struct GraphicVisitor;
        impl<'de> Visitor<'de> for GraphicVisitor {
            type Value = Graphic;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                fmt::Formatter::write_str(formatter, "enum Graphic")
            }
            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                match EnumAccess::variant(data)? {
                    (Field::Panel, variant) => {
                        VariantAccess::struct_variant(variant, PANEL_FIELDS, PanelVisitor)
                            .map(Graphic::Panel)
                    }
                    (Field::Texture, variant) => {
                        VariantAccess::struct_variant(variant, TEXTURE_FIELDS, TextureVisitor)
                            .map(Graphic::Texture)
                    }
                    (Field::Icon, variant) => {
                        VariantAccess::struct_variant(variant, ICON_FIELDS, IconVisitor)
                            .map(Graphic::Icon)
                    }
                    (Field::Text, variant) => {
                        VariantAccess::struct_variant(variant, TEXT_FIELDS, TextVisitor)
                            .map(Graphic::Text)
                    }
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

        Deserializer::deserialize_enum(deserializer, "Graphic", VARIANTS, GraphicVisitor)
    }
}
