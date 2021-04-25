use super::*;
use crate::font::FontId;

impl<'a, 'b: 'a> LoadStyle<'a, 'b> for FontId {
    type Loader = FontIdLoader<'a, 'b>;
    fn new_loader(loader: &'a mut StyleLoader<'b>) -> Self::Loader {
        FontIdLoader { loader }
    }
}

pub struct FontIdLoader<'a, 'b> {
    pub loader: &'a mut StyleLoader<'b>,
}
impl<'de, 'a, 'b> DeserializeSeed<'de> for FontIdLoader<'a, 'b> {
    type Value = FontId;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = DeserializeSeed::deserialize(PhantomData::<String>, deserializer)?;
        Ok(self.loader.load_font(name))
    }
}
