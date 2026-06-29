//! Trailing value-bit count — reverse scan.
//!
//! Parameterised by `const FILL: u64` and `const WORD_ALIGNED: bool`.
//! When `WORD_ALIGNED` is `true` the caller guarantees `start_offset == 0`,
//! allowing the compiler to eliminate the first-word LZCNT phase.

use super::funcs_for_chunk_eq::{LANES, chunk_eq};
use crate::{WORD_BITS, low_mask};

/// Counts leading bits within a single u64 word that match `FILL`.
///
/// `FILL == 0` → [`u64::leading_zeros`]; `FILL == !0` → leading ones.
#[inline]
fn count_leading<const FILL: u64>(val: u64) -> usize {
    if FILL == 0 {
        val.leading_zeros() as usize
    } else {
        (!val).leading_zeros() as usize
    }
}

/// Counts leading bits of `val` within its highest `limit` bits.
#[inline]
fn count_leading_within<const FILL: u64>(val: u64, limit: usize) -> usize {
    if limit == 0 {
        return 0;
    }
    let shifted = val << (WORD_BITS - limit);
    if FILL == 0 {
        (shifted.leading_zeros() as usize).min(limit)
    } else {
        ((!shifted).leading_zeros() as usize).min(limit)
    }
}

/// Counts consecutive trailing bits equal to `FILL` (from the end of the
/// view).
///
/// `bits` is pre-trimmed to `words[physical_start / WORD_BITS..]`.
/// `start_offset` is `physical_start % WORD_BITS`.
/// When `WORD_ALIGNED` is `true`, `start_offset` is guaranteed to be 0.
#[inline]
pub(super) fn trailing<const FILL: u64, const WORD_ALIGNED: bool>(
    bits: &[u64],
    start_offset: u32,
    bit_len: usize,
) -> usize {
    if bit_len == 0 {
        return 0;
    }

    let end_offset = start_offset as usize + bit_len;
    let end_rem = end_offset % WORD_BITS;
    let last_wi = (end_offset - 1) / WORD_BITS;

    let mut scanned = 0usize;

    // Last word (partial, if end_rem != 0).
    if end_rem != 0 {
        let last_limit = if last_wi == 0 {
            end_rem - start_offset as usize
        } else {
            end_rem
        };
        let last_val = if last_wi == 0 {
            bits[0] >> start_offset
        } else {
            bits[last_wi] & low_mask(end_rem)
        };
        let last_count = count_leading_within::<FILL>(last_val, last_limit);
        if last_count < last_limit {
            return last_count;
        }
        scanned += last_limit;

        // Entire view was within a single partial word.
        if last_wi == 0 {
            return scanned.min(bit_len);
        }
    }

    // Full middle words — SIMD countdown from right to left.
    let wi_end = if end_rem != 0 { last_wi - 1 } else { last_wi };
    let mid_first = if !WORD_ALIGNED && start_offset > 0 {
        1
    } else {
        0
    };

    if wi_end >= mid_first {
        let total_words = wi_end + 1 - mid_first;
        let ptr = bits.as_ptr();
        let mut done = 0usize;

        // SAFETY: mid_first..=wi_end are full u64 words within `bits`.
        unsafe {
            while done + LANES <= total_words {
                // `done + LANES ≤ total_words` implies
                // `wi_end + 1 ≥ done + LANES`, so this never wraps.
                let chunk_start = wi_end + 1 - (done + LANES);
                if !chunk_eq::<FILL>(ptr.add(chunk_start)) {
                    break;
                }
                scanned += LANES * WORD_BITS;
                done += LANES;
            }
        }

        // Scalar tail — right to left on remainder.
        while done < total_words {
            let w = wi_end - done;
            if bits[w] != FILL {
                scanned += count_leading::<FILL>(bits[w]).min(WORD_BITS);
                return scanned.min(bit_len);
            }
            scanned += WORD_BITS;
            done += 1;
        }
    }

    // First-word partial (from trailing side, when start_offset != 0).
    if !WORD_ALIGNED && start_offset > 0 {
        let first_limit = WORD_BITS - start_offset as usize;
        let first_val = bits[0] >> start_offset;
        let first_count = count_leading_within::<FILL>(first_val, first_limit);
        scanned += first_count.min(first_limit);
    }

    scanned.min(bit_len)
}
