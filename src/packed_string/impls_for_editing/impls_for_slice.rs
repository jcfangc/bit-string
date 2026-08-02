use int_interval::UsizeCO;

use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn slice(&self, _interval: UsizeCO) -> Self {
        unimplemented!("PackedString::slice")
    }

    pub fn slice_from(&self, _start: usize) -> Self {
        unimplemented!("PackedString::slice_from")
    }

    pub fn slice_until(&self, _end: usize) -> Self {
        unimplemented!("PackedString::slice_until")
    }
}
