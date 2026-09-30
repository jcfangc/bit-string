use core::iter::FusedIterator;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
use crate::traits::WordsUnpack;
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
            #[cfg(all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "avx2"
            ))]
            decoded_codes: [0; 64],
            #[cfg(all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "avx2"
            ))]
            decoded_code_index: 0,
            #[cfg(all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "avx2"
            ))]
            decoded_code_len: 0,
        }
    }

    /// Collects the decoded characters into a vector.
    pub fn to_vec(&self) -> alloc::vec::Vec<C> {
        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        ))]
        if BITS == 4 && self.char_len() >= 64 {
            return self.to_vec_unpacked();
        }

        self.iter().collect()
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    fn to_vec_unpacked(&self) -> alloc::vec::Vec<C> {
        const CODE_BATCH_LEN: usize = 64;

        let char_len = self.char_len();
        let layout_len = crate::traits::words_unpack::layout_block_len::<BITS>();
        let full_code_len = char_len / layout_len * layout_len;
        let mut result = alloc::vec::Vec::with_capacity(char_len);
        let mut decoded = [0u8; CODE_BATCH_LEN];
        let mut code_start = 0;

        while code_start < full_code_len {
            let code_len = (full_code_len - code_start).min(CODE_BATCH_LEN);
            let word_start = code_start * usize::from(BITS) / WORD_BITS;
            let word_len = code_len * usize::from(BITS) / WORD_BITS;
            self.bits.words()[word_start..word_start + word_len]
                .unpack_codes::<BITS>(&mut decoded[..code_len]);

            result.extend(decoded[..code_len].iter().map(|&code| {
                C::from_code(code).expect("PackedChar rejected a code it previously produced")
            }));
            code_start += code_len;
        }

        result.extend((full_code_len..char_len).map(|index| {
            self.get(index)
                .expect("packed character index is within the string")
        }));

        result
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
    // Invariant without a decoded batch:
    // `front * BITS + front_pending_bit_count == front_next_word_index * WORD_BITS`.
    // With an AVX2 batch, add its unconsumed code bits to the left side.
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    decoded_codes: [u8; 64],
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    decoded_code_index: usize,
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    decoded_code_len: usize,
}

impl<C, const BITS: u8> Iterator for Iter<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    type Item = C;

    #[inline(always)]
    fn next(&mut self) -> Option<C> {
        if self.front == self.back {
            return None;
        }

        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        ))]
        if BITS == 4 {
            if self.decoded_code_index == self.decoded_code_len
                && self.back - self.front >= self.decoded_codes.len()
            {
                let words = self.string.bits.words();
                let word_start = self.front_next_word_index;
                words[word_start..word_start + 4].unpack_codes::<BITS>(&mut self.decoded_codes);
                self.front_next_word_index += 4;
                self.decoded_code_index = 0;
                self.decoded_code_len = self.decoded_codes.len();
            }

            if self.decoded_code_index < self.decoded_code_len {
                let code = self.decoded_codes[self.decoded_code_index];
                self.decoded_code_index += 1;
                self.front += 1;
                let character =
                    C::from_code(code).expect("PackedChar rejected a code it previously produced");
                return Some(character);
            }
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
            let next_word = words[self.front_next_word_index];
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
