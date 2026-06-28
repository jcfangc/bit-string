//! Backend-equivalence tests for leading/trailing bit-count functions.
//!
//! Verifies that the const-generic `FILL` and `WORD_ALIGNED` parameters
//! produce correct results for a representative matrix of inputs.

use alloc::vec;

use crate::{FILL_ONES, FILL_ZEROS, WORD_BITS};

use super::*;

// ---------------------------------------------------------------------------
// Reference — pure scalar oracle
// ---------------------------------------------------------------------------

/// Scalar-only reference for `leading`.
fn leading_ref<const FILL: u64>(bits: &[u64], start_offset: u32, bit_len: usize) -> usize {
    if bit_len == 0 {
        return 0;
    }
    let mut count = 0usize;
    let mut off = start_offset as usize;
    for &w in bits {
        let limit = (WORD_BITS - off).min(bit_len - count);
        if limit == 0 {
            break;
        }
        let val = w >> off;
        let n = if FILL == 0 {
            val.trailing_zeros() as usize
        } else {
            (!val).trailing_zeros() as usize
        }
        .min(limit);
        count += n;
        if n < limit || count >= bit_len {
            break;
        }
        off = 0;
    }
    count.min(bit_len)
}

/// Scalar-only reference for `trailing`.
fn trailing_ref<const FILL: u64>(bits: &[u64], start_offset: u32, bit_len: usize) -> usize {
    if bit_len == 0 {
        return 0;
    }
    let mut count = 0usize;
    let end_offset = start_offset as usize + bit_len;
    let last_wi = (end_offset - 1) / WORD_BITS;
    let end_rem = end_offset % WORD_BITS;

    // Last word
    if end_rem != 0 {
        let limit = if last_wi == 0 {
            end_rem - start_offset as usize
        } else {
            end_rem
        };
        let val = if last_wi == 0 {
            bits[0] >> start_offset
        } else {
            bits[last_wi] & crate::low_mask(end_rem)
        };
        let n = if FILL == 0 {
            let shifted = val << (WORD_BITS - limit);
            (shifted.leading_zeros() as usize).min(limit)
        } else {
            let shifted = !val << (WORD_BITS - limit);
            (shifted.leading_zeros() as usize).min(limit)
        };
        count += n;
        if n < limit || last_wi == 0 {
            return count.min(bit_len);
        }
    }

    // Middle + first words (reverse)
    let wi_end = if end_rem != 0 { last_wi - 1 } else { last_wi };
    let mut w = wi_end;
    let mid_first = if start_offset > 0 { 1 } else { 0 };
    let mut mismatch = false;
    loop {
        if w < mid_first {
            break;
        }
        let val = bits[w];
        if val != FILL {
            let n = if FILL == 0 {
                val.leading_zeros() as usize
            } else {
                (!val).leading_zeros() as usize
            }
            .min(WORD_BITS);
            count += n;
            mismatch = true;
            break;
        }
        count += WORD_BITS;
        if w == 0 {
            break;
        }
        w -= 1;
    }

    // First partial — only when all middle words matched.
    if !mismatch && start_offset > 0 {
        let limit = WORD_BITS - start_offset as usize;
        let val = bits[0] >> start_offset;
        let n = if FILL == 0 {
            let shifted = val << (WORD_BITS - limit);
            (shifted.leading_zeros() as usize).min(limit)
        } else {
            let shifted = !val << (WORD_BITS - limit);
            (shifted.leading_zeros() as usize).min(limit)
        };
        count += n;
    }

    count.min(bit_len)
}

// ---------------------------------------------------------------------------
// Test data
// ---------------------------------------------------------------------------

/// Returns a static set of word-arrays used as test inputs.
fn cases() -> &'static [&'static [u64]] {
    &[
        &[],
        &[0],
        &[u64::MAX],
        &[1],
        &[0x8000_0000_0000_0000],
        &[0, 1],
        &[u64::MAX, 0],
        &[u64::MAX, u64::MAX, 0],
        &[0, 0, 0, 0, 1],
        &[0, 0, 0, 0, 0],
        &[u64::MAX; 8],
        &[0u64; 16],
        &[0x5555_5555_5555_5555, 0xAAAA_AAAA_AAAA_AAAA],
        &[0, 1, u64::MAX, 0x0123_4567_89AB_CDEF, 0],
    ]
}

// ---------------------------------------------------------------------------
// leading — aligned
// ---------------------------------------------------------------------------

#[test]
fn leading_zeros_aligned() {
    for &src in cases() {
        for len in [0, src.len() * WORD_BITS] {
            let len = len.min(src.len() * WORD_BITS);
            if len == 0 && src.is_empty() {
                // both work
            }
            let expected = leading_ref::<{ FILL_ZEROS }>(src, 0, len);
            let actual = leading::<{ FILL_ZEROS }, true>(src, 0, len);
            assert_eq!(
                actual, expected,
                "leading zeros aligned mismatch: src={src:?} len={len}"
            );
        }
    }
}

#[test]
fn leading_ones_aligned() {
    for &src in cases() {
        for len in [0, src.len() * WORD_BITS] {
            let len = len.min(src.len() * WORD_BITS);
            let expected = leading_ref::<{ FILL_ONES }>(src, 0, len);
            let actual = leading::<{ FILL_ONES }, true>(src, 0, len);
            assert_eq!(
                actual, expected,
                "leading ones aligned mismatch: src={src:?} len={len}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// leading — unaligned
// ---------------------------------------------------------------------------

#[test]
fn leading_zeros_unaligned() {
    for &src in cases() {
        if src.is_empty() {
            continue;
        }
        for offset in [1u32, 3, 7, 31, 63] {
            let max_len = (src.len() * WORD_BITS).saturating_sub(offset as usize);
            if max_len == 0 {
                continue;
            }
            let expected = leading_ref::<{ FILL_ZEROS }>(src, offset, max_len);
            let actual = leading::<{ FILL_ZEROS }, false>(src, offset, max_len);
            assert_eq!(
                actual, expected,
                "leading zeros unaligned mismatch: src={src:?} offset={offset}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// trailing — aligned
// ---------------------------------------------------------------------------

#[test]
fn trailing_zeros_aligned() {
    for &src in cases() {
        for len in [0, src.len() * WORD_BITS] {
            let len = len.min(src.len() * WORD_BITS);
            let expected = trailing_ref::<{ FILL_ZEROS }>(src, 0, len);
            let actual = trailing::<{ FILL_ZEROS }, true>(src, 0, len);
            assert_eq!(
                actual, expected,
                "trailing zeros aligned mismatch: src={src:?} len={len}"
            );
        }
    }
}

#[test]
fn trailing_ones_aligned() {
    for &src in cases() {
        for len in [0, src.len() * WORD_BITS] {
            let len = len.min(src.len() * WORD_BITS);
            let expected = trailing_ref::<{ FILL_ONES }>(src, 0, len);
            let actual = trailing::<{ FILL_ONES }, true>(src, 0, len);
            assert_eq!(
                actual, expected,
                "trailing ones aligned mismatch: src={src:?} len={len}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// trailing — unaligned
// ---------------------------------------------------------------------------

#[test]
fn trailing_zeros_unaligned() {
    for &src in cases() {
        if src.is_empty() {
            continue;
        }
        for offset in [1u32, 3, 7, 31, 63] {
            let max_len = (src.len() * WORD_BITS).saturating_sub(offset as usize);
            if max_len == 0 {
                continue;
            }
            let expected = trailing_ref::<{ FILL_ZEROS }>(src, offset, max_len);
            let actual = trailing::<{ FILL_ZEROS }, false>(src, offset, max_len);
            assert_eq!(
                actual, expected,
                "trailing zeros unaligned mismatch: src={src:?} offset={offset}"
            );
        }
    }
}
