use super::*;

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
        let bits = usize::from(BITS);
        let bit_index = self.bits.start() + index * bits;
        let word_index = bit_index / 64;
        let offset = bit_index % 64;
        let words = self.bits.source().words();

        let mut code = words[word_index] >> offset;
        if 64 % bits != 0 && offset + bits > 64 {
            code |= words[word_index + 1] << (64 - offset);
        }

        (code & u64::from(crate::code_mask::<BITS>())) as u8
    }
}
