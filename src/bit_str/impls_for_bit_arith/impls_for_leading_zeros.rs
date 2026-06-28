use crate::{FILL_ONES, FILL_ZEROS, WORD_BITS, low_mask};

use crate::BitStr;
use crate::traits::bits_arith::funcs_for_value_bits_core::{LANES, chunk_eq};

// ---------------------------------------------------------------------------
// Shared helper — parameterised by `FILL` (0 → zeros, !0 → ones)
// ---------------------------------------------------------------------------

/// Counts consecutive bits equal to `FILL` from the start of a view.
///
/// `ALIGNED`: when `true`, the caller guarantees `start % WORD_BITS == 0`,
/// eliminating the first-word TZCNT and letting the SIMD loop start from
/// word 0.  `BitString::leading_zeros()` always sets this.
#[inline]
pub(crate) fn leading_value_count<const FILL: u64, const ALIGNED: bool>(
    words: &[u64],
    start: usize,
    bit_len: usize,
) -> usize {
    let end = start + bit_len;
    let start_offset = start % WORD_BITS;
    let end_rem = end % WORD_BITS;
    let last_wi = (end - 1) / WORD_BITS;

    let mut scanned = 0usize;
    let mut wi = start / WORD_BITS;

    // First word — only bits from start_offset upward are in view.
    // When ALIGNED is true, start_offset is 0 and this block is a no-op
    // (LLVM will eliminate it entirely at compile time).
    if !ALIGNED && start_offset != 0 {
        let first_val = words[wi] >> start_offset;
        let first_limit = (WORD_BITS - start_offset).min(bit_len);
        let first_count = count_trailing::<FILL>(first_val).min(first_limit);
        if first_count < first_limit {
            return first_count;
        }
        scanned += first_limit;
        wi += 1;
    }

    // Full middle words — SIMD skip-while + scalar tail.
    let mid_end = if end_rem == 0 { last_wi + 1 } else { last_wi };
    if wi < mid_end {
        let ptr = words.as_ptr();
        // SAFETY: wi..mid_end are full u64 words within `words`.
        // chunk_eq uses unaligned loads which are always safe on x86/aarch64.
        unsafe {
            // SIMD: skip full chunks.
            while wi + LANES <= mid_end {
                if !chunk_eq::<FILL>(ptr.add(wi)) {
                    break;
                }
                scanned += LANES * WORD_BITS;
                wi += LANES;
            }
        }

        // Scalar: handle remaining full words.
        while wi < mid_end {
            if words[wi] != FILL {
                return scanned + count_trailing::<FILL>(words[wi]).min(WORD_BITS);
            }
            scanned += WORD_BITS;
            wi += 1;
        }
    }

    // Last partial word (only when end_rem != 0).
    if end_rem != 0 && wi == last_wi {
        let last_val = words[wi] & low_mask(end_rem);
        scanned += count_trailing::<FILL>(last_val).min(end_rem);
    }

    scanned.min(bit_len)
}

/// Counts trailing bits of a given value within a single u64 word.
#[inline]
fn count_trailing<const FILL: u64>(val: u64) -> usize {
    if FILL == 0 {
        val.trailing_zeros() as usize
    } else {
        (!val).trailing_zeros() as usize
    }
}

impl<'bs> BitStr<'bs> {
    /// Returns the number of consecutive `false` bits from the start of this
    /// view.
    #[inline]
    pub fn leading_zeros(&self) -> usize {
        if self.bit_len == 0 {
            return 0;
        }
        leading_value_count::<FILL_ZEROS, false>(self.source.words(), self.start, self.bit_len)
    }

    /// Returns the number of consecutive `true` bits from the start of this
    /// view.
    #[inline]
    pub fn leading_ones(&self) -> usize {
        if self.bit_len == 0 {
            return 0;
        }
        leading_value_count::<FILL_ONES, false>(self.source.words(), self.start, self.bit_len)
    }
}

#[cfg(test)]
mod tests_for_leading_zeros;

#[cfg(test)]
mod tests_for_leading_ones;
