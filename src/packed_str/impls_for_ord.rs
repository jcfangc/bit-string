use core::cmp::Ordering;

use super::*;

impl<C, const BITS: u8> PartialOrd for PackedStr<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        unimplemented!("PackedStr::partial_cmp")
    }
}

impl<C, const BITS: u8> Ord for PackedStr<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    fn cmp(&self, _other: &Self) -> Ordering {
        unimplemented!("PackedStr::cmp")
    }
}
