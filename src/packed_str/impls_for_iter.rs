use core::iter::FusedIterator;

use crate::{WORD_BITS, code_mask};

use super::*;

impl<'ps, C, const BITS: u8> PackedStr<'ps, C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn iter(&self) -> Iter<'ps, C, BITS> {
        let char_len = self.char_len();
        let bit_start = self.bits.start();
        let front_cursor = if char_len == 0 {
            FrontCursor {
                pending_bits: 0,
                pending_bit_count: 0,
                next_word_index: 0,
            }
        } else {
            let word_index = bit_start / WORD_BITS;
            let bit_offset = bit_start % WORD_BITS;
            let first_word = self.bits.source().words()[word_index];
            FrontCursor {
                pending_bits: first_word >> bit_offset,
                pending_bit_count: WORD_BITS - bit_offset,
                next_word_index: word_index + 1,
            }
        };

        Iter {
            view: *self,
            front: 0,
            back: char_len,
            front_cursor,
        }
    }
}

struct FrontCursor {
    // Loaded but unconsumed bits; the next code starts at bit 0.
    pending_bits: u64,
    // Includes source bits past this view's end; front/back prevents yielding them.
    pending_bit_count: usize,
    // Index of the next backing word not yet loaded into `pending_bits`.
    next_word_index: usize,
    // For a non-empty iterator, with `Iter::front` and the view start:
    // `view.bits.start() + front * BITS + pending_bit_count`
    // equals `next_word_index * WORD_BITS`.
}

pub struct Iter<'ps, C, const BITS: u8>
where
    C: PackedChar<BITS>,
{
    view: PackedStr<'ps, C, BITS>,
    front: usize,
    back: usize,
    front_cursor: FrontCursor,
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
        let words = self.view.bits.source().words();
        let code = if self.front_cursor.pending_bit_count >= width {
            let code = self.front_cursor.pending_bits;
            self.front_cursor.pending_bits >>= width;
            self.front_cursor.pending_bit_count -= width;
            code
        } else {
            let old_fill = self.front_cursor.pending_bit_count;
            // SAFETY: This refill branch requires `old_fill < width`, and
            // `front < back`. The cursor invariant gives
            // `front_cursor.next_word_index * WORD_BITS = view.bits.start()
            // + front * BITS + old_fill`, which is less than
            // `view.bits.start() + (front + 1) * BITS
            // <= view.bits.start() + back * BITS
            // <= view.bits.start() + view.bits.bit_len()
            // <= words.len() * WORD_BITS`. Thus this word index is in bounds.
            let next_word = unsafe { *words.get_unchecked(self.front_cursor.next_word_index) };
            self.front_cursor.next_word_index += 1;

            let code = self.front_cursor.pending_bits | (next_word << old_fill);
            let used_from_next = width - old_fill;
            self.front_cursor.pending_bits = next_word >> used_from_next;
            self.front_cursor.pending_bit_count = WORD_BITS - used_from_next;
            code
        };

        let character = C::from_code((code & u64::from(code_mask::<BITS>())) as u8)
            .expect("PackedChar rejected a code in a PackedStr invariant");
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
            None
        } else {
            self.back -= 1;
            self.view.get(self.back)
        }
    }
}

impl<C, const BITS: u8> ExactSizeIterator for Iter<'_, C, BITS> where C: PackedChar<BITS> {}
impl<C, const BITS: u8> FusedIterator for Iter<'_, C, BITS> where C: PackedChar<BITS> {}

impl<'ps, C, const BITS: u8> IntoIterator for &'ps PackedStr<'ps, C, BITS>
where
    C: PackedChar<BITS>,
{
    type Item = C;
    type IntoIter = Iter<'ps, C, BITS>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests_for_iter;
