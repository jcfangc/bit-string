use super::*;
use crate::extract_code_unchecked;

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
        // SAFETY: `PackedStr` maintains a character-aligned BitStr start, so
        // `self.bits.start()` is a multiple of BITS as required by the helper.
        // `index < char_len()` implies
        // `(index + 1) * BITS <= self.bits.bit_len()`, so the complete code lies
        // inside this view. The BitStr invariant places
        // `[self.bits.start(), self.bits.start() + self.bits.bit_len())` inside
        // the source bit range. Therefore the code's first word and, when it
        // crosses a word boundary, its successor are present in the source
        // words, including when the view starts at a non-word offset.
        let code = unsafe {
            extract_code_unchecked::<BITS>(self.bits.source().words(), self.bits.start(), index)
        };
        Some(C::from_code(code).expect("PackedChar rejected a code in a PackedStr invariant"))
    }

    pub fn first(&self) -> Option<C> {
        self.get(0)
    }

    pub fn last(&self) -> Option<C> {
        self.char_len()
            .checked_sub(1)
            .and_then(|index| self.get(index))
    }
}

#[cfg(test)]
mod tests_for_access;
