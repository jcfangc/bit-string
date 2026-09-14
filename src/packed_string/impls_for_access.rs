use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    /// Number of packed characters, not number of bits.
    #[inline]
    pub fn char_len(&self) -> usize {
        self.bits.bit_len() / usize::from(BITS)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }

    #[inline]
    pub const fn bits_per_char(&self) -> usize {
        BITS as usize
    }

    #[inline]
    pub fn bits(&self) -> &BitString {
        &self.bits
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<C> {
        if index >= self.char_len() {
            return None;
        }
        Some(
            C::from_code(self.code_at(index))
                .expect("PackedChar rejected a code it previously produced"),
        )
    }

    #[inline]
    pub fn first(&self) -> Option<C> {
        self.get(0)
    }

    #[inline]
    pub fn last(&self) -> Option<C> {
        self.char_len().checked_sub(1).and_then(|i| self.get(i))
    }

    #[inline]
    fn code_at(&self, index: usize) -> u8 {
        let bits = usize::from(BITS);
        let bit_index = index * bits;
        let word_index = bit_index / 64;
        let offset = bit_index % 64;
        let words = self.bits.words();

        let mut code = words[word_index] >> offset;
        if 64 % bits != 0 && offset + bits > 64 {
            code |= words[word_index + 1] << (64 - offset);
        }

        (code & u64::from(code_mask::<BITS>())) as u8
    }
}

#[cfg(test)]
mod tests_for_access;
