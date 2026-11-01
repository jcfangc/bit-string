use super::*;
use crate::traits::WordsUnpack;

const UNPACK_BATCH_WORDS: usize = 64;
const UNPACK_BATCH_BITS: usize = UNPACK_BATCH_WORDS * crate::WORD_BITS;

impl<'bs> BitStr<'bs> {
    /// Returns a double-ended iterator over the bits of this view.
    #[inline]
    pub fn iter(&self) -> Iter<'bs> {
        Iter {
            bitstr: *self,
            front: 0,
            back: self.bit_len,
        }
    }

    /// Collects the bits into a `Vec<bool>`.
    #[inline]
    pub fn to_bool_vec(&self) -> alloc::vec::Vec<bool> {
        let mut output = alloc::vec::Vec::with_capacity(self.bit_len);
        let complete_word_count = self.bit_len / crate::WORD_BITS;

        if self.start.is_multiple_of(crate::WORD_BITS) {
            let first_word = self.start / crate::WORD_BITS;
            let words = &self.source.words()[first_word..first_word + complete_word_count];
            for word_batch in words.chunks(UNPACK_BATCH_WORDS) {
                append_unpacked_words(word_batch, &mut output);
            }
        } else {
            let mut word_buffer = [0u64; UNPACK_BATCH_WORDS];
            let mut processed_words = 0;

            while processed_words < complete_word_count {
                let batch_word_count =
                    (complete_word_count - processed_words).min(UNPACK_BATCH_WORDS);
                for (word_index, word) in word_buffer[..batch_word_count].iter_mut().enumerate() {
                    *word = self.get_chunk(
                        processed_words * crate::WORD_BITS + word_index * crate::WORD_BITS,
                    );
                }

                append_unpacked_words(&word_buffer[..batch_word_count], &mut output);
                processed_words += batch_word_count;
            }
        }

        for bit_index in complete_word_count * crate::WORD_BITS..self.bit_len {
            // The loop index is strictly less than the view length.
            output.push(self.get(bit_index).unwrap());
        }

        output
    }
}

fn append_unpacked_words(words: &[u64], output: &mut alloc::vec::Vec<bool>) {
    let code_len = words.len() * crate::WORD_BITS;
    let mut codes = [0u8; UNPACK_BATCH_BITS];
    words.unpack_codes::<1>(&mut codes[..code_len]);
    output.extend(codes[..code_len].iter().map(|&code| code != 0));
}

/// A double-ended, exact-size iterator over the bits of a [`BitStr`].
pub struct Iter<'bs> {
    bitstr: BitStr<'bs>,
    front: usize,
    back: usize,
}

impl Iterator for Iter<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        // SAFETY: `front < back <= bit_len`, so `get(front)` is Some.
        let value = self.bitstr.get(self.front).unwrap();
        self.front += 1;
        Some(value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.back - self.front;
        (len, Some(len))
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        // SAFETY: `back >= front` after decrement, so `get(back)` is Some.
        Some(self.bitstr.get(self.back).unwrap())
    }
}

impl ExactSizeIterator for Iter<'_> {}
impl core::iter::FusedIterator for Iter<'_> {}

impl<'bs> IntoIterator for &'bs BitStr<'bs> {
    type Item = bool;
    type IntoIter = Iter<'bs>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests_for_iter;
