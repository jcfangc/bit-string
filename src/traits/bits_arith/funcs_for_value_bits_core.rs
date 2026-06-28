//! Leading-/trailing-value bit-count operations for `[u64]` slices.
//!
//! Each function is parameterised by `const FILL: u64` and
//! `const WORD_ALIGNED: bool`.  When `WORD_ALIGNED` is `true` the caller
//! guarantees that the first word of `bits` is fully included (no partial
//! offset), allowing the compiler to eliminate the first-word TZCNT/LZCNT
//! phase.

mod chunk_eq;
pub(crate) use chunk_eq::{LANES, chunk_eq};

use crate::{WORD_BITS, low_mask};

// ---------------------------------------------------------------------------
// Count trailing bits within a single u64 word
// ---------------------------------------------------------------------------

/// Counts trailing bits of a value that match `FILL`.
///
/// `FILL == 0` → [`u64::trailing_zeros`]; `FILL == !0` → trailing ones.
#[inline]
fn count_trailing<const FILL: u64>(val: u64) -> usize {
    if FILL == 0 {
        val.trailing_zeros() as usize
    } else {
        (!val).trailing_zeros() as usize
    }
}

/// Counts leading bits of a value that match `FILL`.
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

// ---------------------------------------------------------------------------
// leading — forward scan
// ---------------------------------------------------------------------------

/// Counts consecutive leading bits equal to `FILL`.
///
/// `bits` is pre-trimmed to `words[physical_start / 64..]`.
/// `start_offset` is `physical_start % 64`.
/// When `WORD_ALIGNED` is `true`, `start_offset` is guaranteed to be 0 and
/// the first-word TZCNT phase is eliminated at compile time.
#[inline]
pub(super) fn leading<const FILL: u64, const WORD_ALIGNED: bool>(
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
    let mut wi = 0usize;

    // First word — only bits from start_offset upward are in view.
    // When WORD_ALIGNED is true, start_offset is 0 and LLVM eliminates this.
    if !WORD_ALIGNED && start_offset != 0 {
        let first_val = bits[0] >> start_offset;
        let first_limit = (WORD_BITS - start_offset as usize).min(bit_len);
        let first_count = count_trailing::<FILL>(first_val).min(first_limit);
        if first_count < first_limit {
            return first_count;
        }
        scanned += first_limit;
        wi = 1;
    }

    // Full middle words — SIMD skip-while + scalar tail.
    let mid_end = if end_rem == 0 { last_wi + 1 } else { last_wi };
    if wi < mid_end {
        let ptr = bits.as_ptr();
        // SAFETY: wi..mid_end are full u64 words within `bits`.
        // chunk_eq uses unaligned loads which are safe on x86/aarch64.
        unsafe {
            while wi + LANES <= mid_end {
                if !chunk_eq::<FILL>(ptr.add(wi)) {
                    break;
                }
                scanned += LANES * WORD_BITS;
                wi += LANES;
            }
        }

        // Scalar: remaining full words.
        while wi < mid_end {
            if bits[wi] != FILL {
                return scanned + count_trailing::<FILL>(bits[wi]).min(WORD_BITS);
            }
            scanned += WORD_BITS;
            wi += 1;
        }
    }

    // Last partial word (only when end_rem != 0).
    if end_rem != 0 && wi == last_wi {
        let last_val = bits[wi] & low_mask(end_rem);
        scanned += count_trailing::<FILL>(last_val).min(end_rem);
    }

    scanned.min(bit_len)
}

// ---------------------------------------------------------------------------
// trailing — reverse scan
// ---------------------------------------------------------------------------

/// Counts consecutive trailing bits equal to `FILL` (from the end of the
/// view).
///
/// `bits` is pre-trimmed to `words[physical_start / 64..]`.
/// `start_offset` is `physical_start % 64`.
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

    // Full middle words — SIMD skip-while from right to left.
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
                // `done + LANES <= total_words` guarantees
                // `wi_end + 1 >= done + LANES`, so the
                // wrapping_sub result is always non-negative.
                let chunk_start = (wi_end + 1).wrapping_sub(done + LANES);
                if !chunk_eq::<FILL>(ptr.add(chunk_start)) {
                    break;
                }
                scanned += LANES * WORD_BITS;
                done += LANES;
            }
        }

        // Scalar tail — right to left.
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

#[cfg(test)]
mod tests_for_backend_equivalence;
