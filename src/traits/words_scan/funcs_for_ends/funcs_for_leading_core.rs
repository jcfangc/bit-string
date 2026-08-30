//! Leading value-bit count — forward scan.
//!
//! Parameterised by `const FILL: u64` and `const WORD_ALIGNED: bool`.

use super::count_matching;
use crate::{SMALL_WORDS, WORD_BITS, low_mask};

// ── Dispatch ───────────────────────────────────────────────────────────

#[inline]
pub(crate) fn leading<const FILL: u64, const WORD_ALIGNED: bool>(
    bits: &[u64],
    start_offset: u32,
    bit_len: usize,
) -> usize {
    debug_assert!(!WORD_ALIGNED || start_offset == 0);
    if bit_len == 0 {
        return 0;
    }

    let end_offset = start_offset as usize + bit_len;
    let end_rem = end_offset % WORD_BITS;
    let last_wi = (end_offset - 1) / WORD_BITS;

    let mut scanned = 0usize;
    let mut wi = 0usize;

    // ── Unaligned first word ───────────────────────────────────────
    if !WORD_ALIGNED && start_offset != 0 {
        let first_val = bits[0] >> start_offset;
        let first_limit = (WORD_BITS - start_offset as usize).min(bit_len);
        let first_count = count_matching::<FILL, false>(first_val).min(first_limit);
        if first_count < first_limit {
            return first_count;
        }
        scanned += first_limit;
        wi = 1;
    }

    let mid_end = if end_rem == 0 { last_wi + 1 } else { last_wi };
    if wi < mid_end {
        let total = mid_end - wi;

        // ── Tiny inputs: scalar ────────────────────────────────────
        if total < SMALL_WORDS {
            for i in 0..total {
                let w = bits[wi + i];
                if w != FILL {
                    return (scanned + count_matching::<FILL, false>(w)).min(bit_len);
                }
                scanned += WORD_BITS;
            }
            wi = mid_end;
        } else {
            // SAFETY: `wi < mid_end` and `total = mid_end - wi`,
            // so `bits[wi..mid_end]` is within the input slice.
            let base = unsafe { bits.as_ptr().add(wi) };
            // SAFETY: `end` is one past the last word — used only as a
            // limit pointer, never dereferenced.
            let end = unsafe { base.add(total) };

            // First-word fast path — catches early non-FILL.
            // SAFETY: `total > 0` (we are in the `total >= SMALL_WORDS`
            // branch), so `base` is valid for at least one u64 read.
            let w0 = unsafe { *base };
            if w0 != FILL {
                return (scanned + count_matching::<FILL, false>(w0)).min(bit_len);
            }
            // Start SIMD from `base` (not base+1).  Word 0 is
            // double-checked (fast path + SIMD) but this keeps the
            // iteration count a clean multiple of the SIMD stride.
            let mut p = base;

            #[cfg(all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "avx2"
            ))]
            {
                // SAFETY: AVX2 is guaranteed by compile-time
                // `#[cfg]` gate.
                p = unsafe { avx2::leading_scan::<FILL>(p, end, base, total) };
            }

            #[cfg(all(
                any(target_arch = "x86", target_arch = "x86_64"),
                any(target_feature = "sse2", target_feature = "ssse3"),
                not(target_feature = "avx2")
            ))]
            {
                // SAFETY: SSE2 is guaranteed by compile-time
                // `#[cfg]` gate.
                p = unsafe { sse2::leading_scan::<FILL>(p, end, total) };
            }

            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
            {
                // SAFETY: NEON is guaranteed by compile-time
                // `#[cfg]` gate.
                p = unsafe { neon::leading_scan::<FILL>(p, end, total) };
            }

            #[allow(unused)]
            {
                // Scalar fallback: `p` stays at base.
            }

            // ── Post-SIMD: shared scalar remainder ─────────────────
            let done_words = (p as usize - base as usize) / 8;
            scanned += done_words * WORD_BITS;

            if (p as usize) >= (end as usize) && end_rem == 0 {
                return scanned.min(bit_len);
            }

            let rem = (end as usize - p as usize) / 8;
            // SAFETY: `rem` is computed from `end - p`, so `p` through
            // `p.add(rem - 1)` lies within `[base, end)`.
            for _ in 0..rem {
                unsafe {
                    if *p != FILL {
                        scanned += count_matching::<FILL, false>(*p);
                        return (scanned).min(bit_len);
                    }
                    scanned += WORD_BITS;
                    p = p.add(1);
                }
            }
            wi = mid_end;
        }
    }

    if end_rem != 0 && wi == last_wi {
        let last_val = bits[wi] & low_mask(end_rem);
        scanned += count_matching::<FILL, false>(last_val).min(end_rem);
    }

    scanned.min(bit_len)
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2;

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon;
