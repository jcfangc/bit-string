use core::cmp::Ordering;

use super::*;

impl<C: PackedChar> PartialOrd for PackedStr<'_, C> {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        unimplemented!("PackedStr::partial_cmp")
    }
}

impl<C: PackedChar> Ord for PackedStr<'_, C> {
    fn cmp(&self, _other: &Self) -> Ordering {
        unimplemented!("PackedStr::cmp")
    }
}
