use core::hash::{Hash, Hasher};

use super::*;

impl<C, const BITS: u8> Hash for PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    fn hash<H: Hasher>(&self, _state: &mut H) {
        unimplemented!("PackedString::hash")
    }
}
