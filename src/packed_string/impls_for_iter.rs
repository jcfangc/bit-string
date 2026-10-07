use core::iter::FusedIterator;

use crate::{WORD_BITS, code_mask};

use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    #[inline]
    pub fn iter(&self) -> Iter<'_, C, BITS> {
        Iter {
            string: self,
            front: 0,
            back: self.char_len(),
            front_pending_bits: 0,
            front_pending_bit_count: 0,
            front_next_word_index: 0,
        }
    }
}

#[derive(Clone)]
pub struct Iter<'a, C, const BITS: u8>
where
    C: PackedChar<BITS>,
{
    string: &'a PackedString<C, BITS>,
    front: usize,
    back: usize,
    // Inspired by sux's bit-field iterator: cache unread bits and refill on demand.
    // Loaded but unconsumed bits; the next code starts at bit 0.
    front_pending_bits: u64,
    // Number of buffered, unconsumed low bits; the final word may include tail padding.
    front_pending_bit_count: usize,
    // Index of the next backing word not yet loaded into `front_pending_bits`.
    front_next_word_index: usize,
    // Invariant: `front * BITS + front_pending_bit_count == front_next_word_index * WORD_BITS`.
}

impl<C, const BITS: u8> Iterator for Iter<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    type Item = C;

    fn next(&mut self) -> Option<C> {
        if self.front == self.back {
            return None;
        }

        let width = usize::from(BITS);
        let words = self.string.bits.words();
        let code = if self.front_pending_bit_count >= width {
            let code = self.front_pending_bits;
            self.front_pending_bits >>= width;
            self.front_pending_bit_count -= width;
            code
        } else {
            let old_fill = self.front_pending_bit_count;
            // SAFETY: This refill branch requires `old_fill < width`, and
            // `front < back`. The cursor invariant gives
            // `front_next_word_index * WORD_BITS = front * BITS + old_fill`,
            // which is less than `(front + 1) * BITS <= back * BITS <=
            // words.len() * WORD_BITS`; therefore this word index is in bounds.
            let next_word = unsafe { *words.get_unchecked(self.front_next_word_index) };
            self.front_next_word_index += 1;

            let code = self.front_pending_bits | (next_word << old_fill);
            let used_from_next = width - old_fill;
            self.front_pending_bits = next_word >> used_from_next;
            self.front_pending_bit_count = WORD_BITS - used_from_next;
            code
        };

        let character = C::from_code((code & u64::from(code_mask::<BITS>())) as u8)
            .expect("PackedChar rejected a code it previously produced");
        self.front += 1;

        Some(character)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.back - self.front;
        (len, Some(len))
    }
}

impl<C, const BITS: u8> DoubleEndedIterator for Iter<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    fn next_back(&mut self) -> Option<C> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        self.string.get(self.back)
    }
}

impl<C, const BITS: u8> ExactSizeIterator for Iter<'_, C, BITS> where C: PackedChar<BITS> {}
impl<C, const BITS: u8> FusedIterator for Iter<'_, C, BITS> where C: PackedChar<BITS> {}

impl<'a, C, const BITS: u8> IntoIterator for &'a PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    type Item = C;
    type IntoIter = Iter<'a, C, BITS>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests_for_iter;
