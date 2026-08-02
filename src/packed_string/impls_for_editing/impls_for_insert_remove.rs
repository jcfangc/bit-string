use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn insert(&mut self, _index: usize, _character: C) {
        unimplemented!("PackedString::insert")
    }

    pub fn remove(&mut self, _index: usize) -> C {
        unimplemented!("PackedString::remove")
    }
}
