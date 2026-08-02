use super::*;

impl<C: PackedChar> PartialEq for PackedStr<'_, C> {
    fn eq(&self, _other: &Self) -> bool {
        unimplemented!("PackedStr::eq")
    }
}

impl<C: PackedChar> Eq for PackedStr<'_, C> {}
