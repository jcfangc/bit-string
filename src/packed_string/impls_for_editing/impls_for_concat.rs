use super::*;

impl<C: PackedChar> PackedString<C> {
    pub fn push_packed_string(&mut self, _other: &Self) {
        unimplemented!("PackedString::push_packed_string")
    }

    pub fn insert_packed_string(&mut self, _index: usize, _other: &Self) {
        unimplemented!("PackedString::insert_packed_string")
    }

    pub fn split_off(&mut self, _at: usize) -> Self {
        unimplemented!("PackedString::split_off")
    }
}
