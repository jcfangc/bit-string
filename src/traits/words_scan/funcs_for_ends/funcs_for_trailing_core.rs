//! Trailing value-bit count — reverse scan.
//!
//! Parameterised by `const FILL: u64` and `const WORD_ALIGNED: bool`.
//! When `WORD_ALIGNED` is `true` the caller guarantees `start_offset == 0`,
//! allowing the compiler to eliminate the first-word LZCNT phase.

use super::count_matching;
use crate::{SMALL_WORDS, WORD_BITS};

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2;

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon;

// ── Dispatch ───────────────────────────────────────────────────────────

#[inline]
pub(crate) fn trailing<const FILL: u64, const WORD_ALIGNED: bool>(
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

    // ── Last partial word ─────────────────────────────────────────
    if end_rem != 0 {
        let last_limit = if last_wi == 0 {
            end_rem - start_offset as usize
        } else {
            end_rem
        };
        let shifted = bits[last_wi] << (WORD_BITS - end_rem);
        let last_count = count_matching::<FILL, true>(shifted).min(last_limit);
        if last_count < last_limit {
            return last_count;
        }
        scanned += last_limit;
        if last_wi == 0 {
            return scanned.min(bit_len);
        }
    }

    // ── Full middle words — reverse SIMD scan ────────────────────
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

        // ── Rightmost-word fast path ─────────────────────────────
        // Early exit if the answer is in the rightmost full word,
        // without disrupting the SIMD stride alignment.
        {
            let w = bits[wi_end];
            if w != FILL {
                scanned += count_matching::<FILL, true>(w);
                return scanned.min(bit_len);
            }
        }

        // ── Tiny inputs — simple scalar reverse scan ────────────
        if total_words < SMALL_WORDS {
            while done < total_words {
                let wi = wi_end - done;
                if bits[wi] != FILL {
                    scanned += count_matching::<FILL, true>(bits[wi]);
                    return scanned.min(bit_len);
                }
                scanned += WORD_BITS;
                done += 1;
            }
            // All full words match FILL — skip SIMD.
        } else {
            #[cfg(all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "avx2"
            ))]
            {
                let done_before = done;
                // SAFETY: AVX2 is guaranteed by compile-time gate.
                done = unsafe { avx2::trailing_scan::<FILL>(ptr, wi_end, done, total_words) };
                scanned += (done - done_before) * WORD_BITS;
            }

            #[cfg(all(
                any(target_arch = "x86", target_arch = "x86_64"),
                any(target_feature = "sse2", target_feature = "ssse3"),
                not(target_feature = "avx2")
            ))]
            {
                let done_before = done;
                // SAFETY: SSE2 is guaranteed by compile-time gate.
                done = unsafe { sse2::trailing_scan::<FILL>(ptr, wi_end, done, total_words) };
                scanned += (done - done_before) * WORD_BITS;
            }

            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
            {
                let done_before = done;
                // SAFETY: NEON is guaranteed by compile-time gate.
                done = unsafe { neon::trailing_scan::<FILL>(ptr, wi_end, done, total_words) };
                scanned += (done - done_before) * WORD_BITS;
            }

            #[allow(unused)]
            {
                // Scalar fallback.
            }
        } // else (SIMD path)

        // ── Scalar tail ──────────────────────────────────────────
        while done < total_words {
            let wi = wi_end - done;
            if bits[wi] != FILL {
                scanned += count_matching::<FILL, true>(bits[wi]);
                return scanned.min(bit_len);
            }
            scanned += WORD_BITS;
            done += 1;
        }
    }

    // ── First-word partial (trailing side) ───────────────────────
    if !WORD_ALIGNED && start_offset > 0 {
        let first_limit = WORD_BITS - start_offset as usize;
        let first_count = count_matching::<FILL, true>(bits[0]).min(first_limit);
        scanned += first_count;
    }

    scanned.min(bit_len)
}
