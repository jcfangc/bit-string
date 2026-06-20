use crate::WORD_BITS;

use super::BitsEdit;

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
    /// Fast path when both source and destination are word-aligned; otherwise
    /// falls back to a chunk-by-chunk read/write loop.
    #[inline]
    pub(crate) fn paste_to(&self, dst: &mut [u64], dst_start: usize) {
        if self.aligned && dst_start.is_multiple_of(WORD_BITS) {
            let src_word = self.src_start / WORD_BITS;
            let dst_word = dst_start / WORD_BITS;
            let full_words = self.len / WORD_BITS;

            if full_words > 0 {
                dst[dst_word..dst_word + full_words]
                    .copy_from_slice(&self.src[src_word..src_word + full_words]);
            }

            let remainder_bits = self.len % WORD_BITS;
            if remainder_bits > 0 {
                let offset = full_words * WORD_BITS;
                let chunk = self.src.read_word_at(self.src_start + offset);
                dst.write_word_at(dst_start + offset, chunk, remainder_bits);
            }
            return;
        }

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
