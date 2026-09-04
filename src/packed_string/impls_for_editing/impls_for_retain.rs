use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(C) -> bool,
    {
        let retained = self.iter().filter(|&character| predicate(character));
        *self = Self::from_chars(retained);
    }
}
