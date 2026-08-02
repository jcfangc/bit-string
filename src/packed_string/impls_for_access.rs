use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    /// Number of packed characters, not number of bits.
    #[inline]
    pub fn char_len(&self) -> usize {
        self.char_len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.char_len == 0
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
        if index >= self.char_len {
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
        self.char_len.checked_sub(1).and_then(|i| self.get(i))
    }

    #[inline]
    fn code_at(&self, index: usize) -> u8 {
        if BITS == 0 {
            return 0;
        }
        let start = index * usize::from(BITS);
        (self.bits.get_chunk(start) & u64::from(code_mask::<BITS>())) as u8
    }
}

#[cfg(test)]
mod tests_for_access;
