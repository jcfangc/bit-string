//! Leading value-bit count — forward scan.
//!
//! Parameterised by `const FILL: u64` and `const WORD_ALIGNED: bool`.
//! When `WORD_ALIGNED` is `true` the caller guarantees `start_offset == 0`,
//! allowing the compiler to eliminate the first-word TZCNT phase.

use super::funcs_for_chunk_eq::{LANES, chunk_eq};
use crate::{WORD_BITS, low_mask};

/// Counts trailing bits within a single u64 word that match `FILL`.
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

/// Counts consecutive leading bits equal to `FILL`.
///
/// `bits` is pre-trimmed to `words[physical_start / WORD_BITS..]`.
/// `start_offset` is `physical_start % WORD_BITS`.
/// When `WORD_ALIGNED` is `true`, `start_offset` is guaranteed to be 0.
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
