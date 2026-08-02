use super::*;

impl<C: PackedChar> PackedString<C> {
    pub fn retain<F>(&mut self, _predicate: F)
    where
        F: FnMut(C) -> bool,
    {
        unimplemented!("PackedString::retain")
    }
}
