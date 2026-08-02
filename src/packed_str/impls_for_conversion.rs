use super::*;

impl<'ps, C, const BITS: u8> PackedStr<'ps, C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn to_packed_string(&self) -> PackedString<C, BITS> {
        unimplemented!("PackedStr::to_packed_string")
    }
}

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn as_packed_str(&self) -> PackedStr<'_, C, BITS> {
        unimplemented!("PackedString::as_packed_str")
    }
}
