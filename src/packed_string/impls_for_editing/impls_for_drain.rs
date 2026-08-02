use int_interval::UsizeCO;

use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn drain_interval(&self, _interval: UsizeCO) -> Self {
        unimplemented!("PackedString::drain_interval")
    }

    pub fn drain_interval_assign(&mut self, _interval: UsizeCO) {
        unimplemented!("PackedString::drain_interval_assign")
    }
}
