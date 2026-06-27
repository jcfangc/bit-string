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
    /// Picks the fastest available backend based on alignment:
    ///
    /// | Src aligned | Dst aligned | Backend              |
    /// |:-:|:-:|-:|
    /// |     ✓       |     ✓       | `copy_words` (memcpy)|
    /// |     ✗       |     ✓       | `copy_words_shifted` |
    /// |     ✓       |     ✗       | SIMD reversed shift  |
    /// |     ✗       |     ✗       | scalar               |
    ///
    /// The partial last word (remainder) is handled at the bottom.
    #[inline]
    pub(crate) fn paste_to(&self, dst: &mut [u64], dst_start: usize) {
        let full_words = self.len / WORD_BITS;
        let remainder_bits = self.len % WORD_BITS;
        let dst_aligned = dst_start.is_multiple_of(WORD_BITS);

        // === Case 1: both word-aligned ===
        // Each dst[i] = src[i] — pure memcpy.
        if self.aligned && dst_aligned {
            if full_words > 0 {
                let sw = self.src_start / WORD_BITS;
                let dw = dst_start / WORD_BITS;
                funcs_for_copy_words_core::copy_words(&mut dst[dw..], &self.src[sw..], full_words);
            }
        }
        // === Case 2: dst aligned, src shifted ===
        // dst[i] = (src[i] >> shift) | (src[i+1] << (64-shift))
        // Each output word is a shifted 64-bit window from src.
        else if !self.aligned && dst_aligned {
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
        }
        // === Case 3: src aligned, dst shifted (reversed shift) ===
        // The scalar write_word_at loop distributes each src word across two
        // dst words.  The middle dst words form shifted windows mirrored from
        // Case 2, so we reuse copy_words_shifted with shift = 64 - dst_shift.
        //   dst[dw]    ←  src[0] << dst_shift            (low boundary)
        //   dst[dw+1..] ←  reversed windows              (SIMD bulk)
        //   dst[dw+N]  ←  src[N-1] >> (64-dst_shift)    (high boundary)
        //
        // Guard: need src[N] in the backing array for the SIMD window, hence
        // `self.src.len() > sw + full_words`.
        else if self.aligned
            && full_words > 1
            && self.src.len() > self.src_start / WORD_BITS + full_words
        {
            let dst_shift = dst_start % WORD_BITS;
            let sw = self.src_start / WORD_BITS;
            let dw = dst_start / WORD_BITS;
            let mid_count = full_words - 1;

            // Low boundary: OR to preserve bits below dst_shift (tail of
            // previous data, e.g. in push_bit_string).
            dst[dw] |= self.src[sw] << dst_shift;

            // Middle: mid_count complete shifted windows.
            funcs_for_copy_words_shifted_core::copy_words_shifted(
                &mut dst[dw + 1..],
                &self.src[sw..],
                mid_count,
                WORD_BITS - dst_shift,
            );

            // High boundary: spill from the last src word.
            dst[dw + full_words] |= self.src[sw + mid_count] >> (WORD_BITS - dst_shift);
        }
        // === Case 4: dst unaligned, src unaligned ===
        // Both offsets are non-zero — read_word_at + write_word_at are the
        // safest fallback.  Each src word is read with its own intra-word shift,
        // then OR'd into two dst words.
        else {
            for i in 0..full_words {
                let chunk = self.src.read_word_at(self.src_start + i * WORD_BITS);
                dst.write_word_at(dst_start + i * WORD_BITS, chunk, WORD_BITS);
            }
        }

        // Partial last word — uniform scalar handling for all paths.
        if remainder_bits > 0 {
            let offset = full_words * WORD_BITS;
            let chunk = self.src.read_word_at(self.src_start + offset);
            dst.write_word_at(dst_start + offset, chunk, remainder_bits);
        }
    }
}

#[cfg(test)]
mod tests_for_copy;
