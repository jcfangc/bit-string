use super::*;

impl<C, const BITS: u8> PartialEq for PackedStr<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    fn eq(&self, _other: &Self) -> bool {
        unimplemented!("PackedStr::eq")
    }
}

impl<C, const BITS: u8> Eq for PackedStr<'_, C, BITS> where C: PackedChar<BITS> {}
