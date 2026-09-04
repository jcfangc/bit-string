use super::*;

impl<C, const BITS: u8> PartialEq for PackedStr<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
    }
}

impl<C, const BITS: u8> Eq for PackedStr<'_, C, BITS> where C: PackedChar<BITS> {}
