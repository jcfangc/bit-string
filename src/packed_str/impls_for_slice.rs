use int_interval::UsizeCO;

use super::*;

impl<'ps, C, const BITS: u8> PackedStr<'ps, C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn slice(&self, _interval: UsizeCO) -> Self {
        unimplemented!("PackedStr::slice")
    }

    pub fn slice_from(&self, _start: usize) -> Self {
        unimplemented!("PackedStr::slice_from")
    }

    pub fn slice_until(&self, _end: usize) -> Self {
        unimplemented!("PackedStr::slice_until")
    }
}
