use core::cmp::Ordering;

use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    /// Compares characters lexicographically by their packed code values.
    pub fn cmp_string(&self, other: &Self) -> Ordering {
        self.as_packed_str().cmp_codes(&other.as_packed_str())
    }
}

impl<C, const BITS: u8> PartialOrd for PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp_string(other))
    }
}

impl<C, const BITS: u8> Ord for PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_string(other)
    }
}
