use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn retain<F>(&mut self, _predicate: F)
    where
        F: FnMut(C) -> bool,
    {
        unimplemented!("PackedString::retain")
    }
}
