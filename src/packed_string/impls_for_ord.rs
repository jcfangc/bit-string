use core::cmp::Ordering;

use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    /// Compares characters lexicographically by their packed code values.
    pub fn cmp_string(&self, _other: &Self) -> Ordering {
        unimplemented!("PackedString::cmp_string")
    }
}

impl<C, const BITS: u8> PartialOrd for PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        unimplemented!("PackedString::partial_cmp")
    }
}

impl<C, const BITS: u8> Ord for PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    fn cmp(&self, _other: &Self) -> Ordering {
        unimplemented!("PackedString::cmp")
    }
}
