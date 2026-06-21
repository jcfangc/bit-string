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
    /// Full-word copy picks the fastest available backend; the remainder
    /// (partial last word) is handled uniformly by scalar read/write.
    #[inline]
    pub(crate) fn paste_to(&self, dst: &mut [u64], dst_start: usize) {
        let full_words = self.len / WORD_BITS;
        let remainder_bits = self.len % WORD_BITS;
        let dst_aligned = dst_start.is_multiple_of(WORD_BITS);

        // Full-word copy — pick the fastest available backend.
        if self.aligned && dst_aligned {
            if full_words > 0 {
                let sw = self.src_start / WORD_BITS;
                let dw = dst_start / WORD_BITS;
                funcs_for_copy_words_core::copy_words(&mut dst[dw..], &self.src[sw..], full_words);
            }
        } else if !self.aligned && dst_aligned {
            if full_words > 0 {
                let shift = self.src_start % WORD_BITS;
                let base = self.src_start / WORD_BITS;
                let dw = dst_start / WORD_BITS;
                funcs_for_copy_words_shifted_core::copy_words_shifted(
                    &mut dst[dw..],
                    &self.src[base..],
                    full_words,
                    shift,
                );
            }
        } else {
            for i in 0..full_words {
                let chunk = self.src.read_word_at(self.src_start + i * WORD_BITS);
                dst.write_word_at(dst_start + i * WORD_BITS, chunk, WORD_BITS);
            }
        }

        // Remainder — same for all paths.
        if remainder_bits > 0 {
            let offset = full_words * WORD_BITS;
            let chunk = self.src.read_word_at(self.src_start + offset);
            dst.write_word_at(dst_start + offset, chunk, remainder_bits);
        }
    }
}

#[cfg(test)]
mod tests_for_copy;
