use super::*;

impl<C: PackedChar> PackedString<C> {
    pub fn insert(&mut self, _index: usize, _character: C) {
        unimplemented!("PackedString::insert")
    }

    pub fn remove(&mut self, _index: usize) -> C {
        unimplemented!("PackedString::remove")
    }
}
