use super::*;
use crate::extract_code;

impl<'ps, C, const BITS: u8> PackedStr<'ps, C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn char_len(&self) -> usize {
        self.bits.bit_len() / usize::from(BITS)
    }

    pub fn is_empty(&self) -> bool {
        self.bits.bit_len() == 0
    }

    pub fn get(&self, index: usize) -> Option<C> {
        if index >= self.char_len() {
            return None;
        }
        Some(
            C::from_code(self.code_at(index))
                .expect("PackedChar rejected a code in a PackedStr invariant"),
        )
    }

    pub fn first(&self) -> Option<C> {
        self.get(0)
    }

    pub fn last(&self) -> Option<C> {
        self.char_len()
            .checked_sub(1)
            .and_then(|index| self.get(index))
    }

    #[inline]
    fn code_at(&self, index: usize) -> u8 {
        extract_code::<BITS>(self.bits.source().words(), self.bits.start(), index)
    }
}
