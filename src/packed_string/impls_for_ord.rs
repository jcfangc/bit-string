use core::cmp::Ordering;

use super::*;

impl<C: PackedChar> PackedString<C> {
    /// Compares characters lexicographically by their packed code values.
    pub fn cmp_string(&self, _other: &Self) -> Ordering {
        unimplemented!("PackedString::cmp_string")
    }
}

impl<C: PackedChar> PartialOrd for PackedString<C> {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        unimplemented!("PackedString::partial_cmp")
    }
}

impl<C: PackedChar> Ord for PackedString<C> {
    fn cmp(&self, _other: &Self) -> Ordering {
        unimplemented!("PackedString::cmp")
    }
}
