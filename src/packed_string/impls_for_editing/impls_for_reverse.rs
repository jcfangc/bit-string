use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn reverse(&self) -> Self {
        Self::from_chars(self.iter().rev())
    }

    pub fn reverse_assign(&mut self) {
        *self = self.reverse();
    }
}
