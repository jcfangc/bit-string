use core::cmp::Ordering;

use super::*;

impl<C, const BITS: u8> PartialOrd for PackedStr<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.bits.cmp_str(&other.bits))
    }
}

impl<C, const BITS: u8> Ord for PackedStr<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.bits.cmp_str(&other.bits)
    }
}
