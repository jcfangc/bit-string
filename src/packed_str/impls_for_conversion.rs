use super::*;

impl<'ps, C: PackedChar> PackedStr<'ps, C> {
    pub fn to_packed_string(&self) -> PackedString<C> {
        unimplemented!("PackedStr::to_packed_string")
    }
}

impl<C: PackedChar> PackedString<C> {
    pub fn as_packed_str(&self) -> PackedStr<'_, C> {
        unimplemented!("PackedString::as_packed_str")
    }
}
