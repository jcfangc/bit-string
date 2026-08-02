use int_interval::UsizeCO;

use super::*;

impl<C: PackedChar> PackedString<C> {
    pub fn drain_interval(&self, _interval: UsizeCO) -> Self {
        unimplemented!("PackedString::drain_interval")
    }

    pub fn drain_interval_assign(&mut self, _interval: UsizeCO) {
        unimplemented!("PackedString::drain_interval_assign")
    }
}
