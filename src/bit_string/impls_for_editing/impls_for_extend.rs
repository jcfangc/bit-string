use crate::traits::*;

use super::*;

const BOOL_EXTEND_BATCH_LEN: usize = 256;
const BOOL_EXTEND_LARGE_BATCH_LEN: usize = 4096;
const BOOL_EXTEND_SMALL_BATCH_LEN: usize = 64;

// Completed batches are committed as they are read. If the iterator panics,
// the current incomplete batch is discarded and earlier batches stay.
impl BitString {
    fn append_bool_batch<const WORDS_LEN: usize>(&mut self, codes: &[u8], direct_write: bool) {
        let batch_len = codes.len();
        let new_bit_len = self
            .bit_len
            .checked_add(batch_len)
            .expect("bit string length overflow");

        if self.bit_len % crate::WORD_BITS == 0 {
            let start_word = self.words.len();
            self.words.resize(crate::word_len(new_bit_len), 0);
            crate::bit_string::impls_for_construction::pack_into(
                codes,
                &mut self.words[start_word..],
            );
            self.bit_len = new_bit_len;
            self.words.mask_unused_bits(new_bit_len);
            return;
        }

        if !direct_write {
            let word_count = crate::word_len(batch_len);
            let mut packed_words = [0u64; WORDS_LEN];
            crate::bit_string::impls_for_construction::pack_into(
                codes,
                &mut packed_words[..word_count],
            );
            self.push_packed_words(&packed_words[..word_count], batch_len);
            return;
        }

        let old_bit_len = self.bit_len;
        let offset = old_bit_len % crate::WORD_BITS;
        let head_len = if offset == 0 {
            0
        } else {
            (crate::WORD_BITS - offset).min(batch_len)
        };
        let body_len = (batch_len - head_len) / crate::WORD_BITS * crate::WORD_BITS;
        let tail_start = head_len + body_len;

        self.words.resize(crate::word_len(new_bit_len), 0);

        if head_len > 0 {
            let head_word = codes[..head_len]
                .iter()
                .enumerate()
                .fold(0, |word, (index, &code)| word | (u64::from(code) << index));
            self.words[old_bit_len / crate::WORD_BITS] |= head_word << offset;
        }

        if body_len > 0 {
            let body_bit_start = old_bit_len + head_len;
            let body_word_start = body_bit_start / crate::WORD_BITS;
            let body_word_len = body_len / crate::WORD_BITS;
            crate::bit_string::impls_for_construction::pack_into(
                &codes[head_len..tail_start],
                &mut self.words[body_word_start..body_word_start + body_word_len],
            );
        }

        let tail_bit_start = old_bit_len + tail_start;
        if tail_start < batch_len {
            let tail_word = codes[tail_start..]
                .iter()
                .enumerate()
                .fold(0, |word, (index, &code)| word | (u64::from(code) << index));
            self.words[tail_bit_start / crate::WORD_BITS] |=
                tail_word << (tail_bit_start % crate::WORD_BITS);
        }

        self.bit_len = new_bit_len;
        self.words.mask_unused_bits(new_bit_len);
    }

    fn extend_bool_batches<const BATCH_LEN: usize, const WORDS_LEN: usize, I>(
        &mut self,
        iter: &mut I,
        direct_write: bool,
    ) where
        I: Iterator<Item = bool>,
    {
        let mut codes = core::mem::MaybeUninit::<[u8; BATCH_LEN]>::uninit();
        let codes_ptr = codes.as_mut_ptr().cast::<u8>();

        loop {
            let mut batch_len = 0;
            while batch_len < BATCH_LEN {
                let Some(value) = iter.next() else {
                    break;
                };
                // SAFETY: `batch_len` is within the array and this slot is
                // initialized before it is included in the slice below.
                unsafe { codes_ptr.add(batch_len).write(u8::from(value)) };
                batch_len += 1;
            }

            if batch_len == 0 {
                break;
            }

            // SAFETY: the first `batch_len` bytes were initialized above.
            let codes = unsafe { core::slice::from_raw_parts(codes_ptr, batch_len) };
            self.append_bool_batch::<WORDS_LEN>(codes, direct_write);

            if batch_len < BATCH_LEN {
                break;
            }
        }
    }
}

impl Extend<bool> for BitString {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = bool>,
    {
        let mut iter = iter.into_iter();
        let (lower_bound, upper_bound) = iter.size_hint();
        self.reserve_bits(lower_bound);
        let direct_write =
            upper_bound == Some(lower_bound) && lower_bound >= BOOL_EXTEND_LARGE_BATCH_LEN;

        // Keep tiny exact iterators on a small stack buffer. Larger known
        // inputs amortize backend dispatch with a larger batch; unknown-size
        // iterators use the bounded default batch.
        if upper_bound == Some(lower_bound) && lower_bound <= BOOL_EXTEND_SMALL_BATCH_LEN {
            self.extend_bool_batches::<BOOL_EXTEND_SMALL_BATCH_LEN, 1, _>(&mut iter, false);
        } else if lower_bound >= BOOL_EXTEND_LARGE_BATCH_LEN {
            self.extend_bool_batches::<BOOL_EXTEND_LARGE_BATCH_LEN, 64, _>(&mut iter, direct_write);
        } else {
            self.extend_bool_batches::<BOOL_EXTEND_BATCH_LEN, 4, _>(&mut iter, false);
        }
    }
}

impl<'a> Extend<&'a bool> for BitString {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a bool>,
    {
        self.extend(iter.into_iter().copied());
    }
}

#[cfg(test)]
mod tests_for_extend;
