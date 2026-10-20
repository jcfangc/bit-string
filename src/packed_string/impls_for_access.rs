use super::*;
use crate::extract_code_unchecked;

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
        // SAFETY: `index < char_len()` implies
        // `(index + 1) * BITS <= self.bits.bit_len()`, so this character's
        // complete range lies in the owned packed bit range. Its first word,
        // and the next word if it crosses a boundary, are therefore present.
        let code = unsafe { extract_code_unchecked::<BITS>(self.bits.words(), 0, index) };
        Some(C::from_code(code).expect("PackedChar rejected a code it previously produced"))
    }

    #[inline]
    pub fn first(&self) -> Option<C> {
        self.get(0)
    }

    #[inline]
    pub fn last(&self) -> Option<C> {
        self.char_len().checked_sub(1).and_then(|i| self.get(i))
    }
}

#[cfg(test)]
mod tests_for_access;
