use int_interval::UsizeCO;

use super::*;

impl<C: PackedChar> PackedString<C> {
    pub fn replace_interval(&self, _interval: UsizeCO, _replacement: &Self) -> Self {
        unimplemented!("PackedString::replace_interval")
    }

    pub fn replace_interval_assign(&mut self, _interval: UsizeCO, _replacement: &Self) {
        unimplemented!("PackedString::replace_interval_assign")
    }

    pub fn replace(&self, _start: usize, _replacement: &Self) -> Self {
        unimplemented!("PackedString::replace")
    }

    pub fn replace_assign(&mut self, _start: usize, _replacement: &Self) {
        unimplemented!("PackedString::replace_assign")
    }
}
