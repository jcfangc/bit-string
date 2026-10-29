use crate::traits::*;

use super::*;

const BOOL_EXTEND_BATCH_LEN: usize = 256;
const BOOL_EXTEND_LARGE_BATCH_LEN: usize = 4096;
const BOOL_EXTEND_SMALL_BATCH_LEN: usize = 64;

// Completed batches are committed as they are read. If the iterator panics,
// the current incomplete batch is discarded and earlier batches stay.
impl BitString {
    fn append_bool_batch<const WORDS_LEN: usize>(&mut self, codes: &[u8]) {
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

        let word_count = crate::word_len(batch_len);
        let mut packed_words = [0u64; WORDS_LEN];
        crate::bit_string::impls_for_construction::pack_into(
            codes,
            &mut packed_words[..word_count],
        );
        self.push_packed_words(&packed_words[..word_count], batch_len);
    }

    fn extend_bool_batches<const BATCH_LEN: usize, const WORDS_LEN: usize, I>(
        &mut self,
        iter: &mut I,
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
            self.append_bool_batch::<WORDS_LEN>(codes);

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

        // Keep tiny exact iterators on a small stack buffer. Larger known
        // inputs amortize backend dispatch with a larger batch; unknown-size
        // iterators use the bounded default batch.
        if upper_bound == Some(lower_bound) && lower_bound <= BOOL_EXTEND_SMALL_BATCH_LEN {
            self.extend_bool_batches::<BOOL_EXTEND_SMALL_BATCH_LEN, 1, _>(&mut iter);
        } else if lower_bound >= BOOL_EXTEND_LARGE_BATCH_LEN {
            self.extend_bool_batches::<BOOL_EXTEND_LARGE_BATCH_LEN, 64, _>(&mut iter);
        } else {
            self.extend_bool_batches::<BOOL_EXTEND_BATCH_LEN, 4, _>(&mut iter);
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
