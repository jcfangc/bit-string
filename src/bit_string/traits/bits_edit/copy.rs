use crate::WORD_BITS;

use super::BitsEdit;

mod funcs_for_copy_words_core;
mod funcs_for_copy_words_shifted_core;

/// Zero-cost curried copy: captures a source snapshot for deferred paste.
///
/// Created by [`BitsEdit::copy_bits`] and materialized via
/// [`BitsCopied::paste_to`].
pub(crate) struct BitsCopied<'a> {
    pub(crate) src: &'a [u64],
    pub(crate) src_start: usize,
    pub(crate) len: usize,
    pub(crate) aligned: bool, // src_start % WORD_BITS == 0
}

impl BitsCopied<'_> {
    /// Paste the previously captured source bits into `dst` at `dst_start`.
    ///
    /// Three paths, in order:
    /// 1. Both source and destination word-aligned → `copy_from_slice`.
    /// 2. Destination word-aligned, source unaligned → SIMD shifted copy.
    /// 3. General case → per-chunk read/write loop.
    #[inline]
    pub(crate) fn paste_to(&self, dst: &mut [u64], dst_start: usize) {
        // Path 1: both word-aligned — memcpy.
        if self.aligned && dst_start.is_multiple_of(WORD_BITS) {
            let src_word = self.src_start / WORD_BITS;
            let dst_word = dst_start / WORD_BITS;
            let full_words = self.len / WORD_BITS;

            if full_words > 0 {
                funcs_for_copy_words_core::copy_words(
                    &mut dst[dst_word..],
                    &self.src[src_word..],
                    full_words,
                );
            }

            let remainder_bits = self.len % WORD_BITS;
            if remainder_bits > 0 {
                let offset = full_words * WORD_BITS;
                let chunk = self.src.read_word_at(self.src_start + offset);
                dst.write_word_at(dst_start + offset, chunk, remainder_bits);
            }
            return;
        }

        // Path 2: destination word-aligned, source unaligned — SIMD shifted copy.
        if !self.aligned && dst_start.is_multiple_of(WORD_BITS) {
            let shift = self.src_start % WORD_BITS;
            let base = self.src_start / WORD_BITS;
            let full_words = self.len / WORD_BITS;
            let src_slice = &self.src[base..];
            let dst_word = dst_start / WORD_BITS;

            if full_words > 0 {
                funcs_for_copy_words_shifted_core::copy_words_shifted(
                    &mut dst[dst_word..],
                    src_slice,
                    full_words,
                    shift,
                );
            }

            let remainder_bits = self.len % WORD_BITS;
            if remainder_bits > 0 {
                let offset = full_words * WORD_BITS;
                let chunk = self.src.read_word_at(self.src_start + offset);
                dst.write_word_at(dst_start + offset, chunk, remainder_bits);
            }
            return;
        }

        // Path 3: general case — per-chunk loop.
        let mut done = 0usize;
        while done < self.len {
            let take = (self.len - done).min(WORD_BITS);
            let chunk = self.src.read_word_at(self.src_start + done);
            dst.write_word_at(dst_start + done, chunk, take);
            done += take;
        }
    }
}

#[cfg(test)]
mod tests_for_copy;
