//! Trailing value-bit count — reverse scan.
//!
//! Parameterised by `const FILL: u64` and `const WORD_ALIGNED: bool`.
//! When `WORD_ALIGNED` is `true` the caller guarantees `start_offset == 0`,
//! allowing the compiler to eliminate the first-word LZCNT phase.

#[cfg(not(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
)))]
use super::chunk_eq::{LANES, LANES_2X, chunk_eq, chunk_eq_2x};
use crate::{SMALL_WORDS, WORD_BITS, low_mask};

/// Counts leading bits within a single u64 word that match `FILL`.
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

#[inline(always)]
pub(crate) fn trailing<const FILL: u64, const WORD_ALIGNED: bool>(
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

    // ── Last partial word ─────────────────────────────────────────
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
                scanned += count_leading::<FILL>(w);
                return scanned.min(bit_len);
            }
        }

        // ── Tiny inputs — simple scalar reverse scan ────────────
        if total_words < SMALL_WORDS {
            while done < total_words {
                let wi = wi_end - done;
                if bits[wi] != FILL {
                    scanned += count_leading::<FILL>(bits[wi]);
                    return scanned.min(bit_len);
                }
                scanned += WORD_BITS;
                done += 1;
            }
            // All full words match FILL — skip SIMD.
        } else {
            // ── AVX2 (2×‑unrolled reverse) ───────────────────────
            #[cfg(all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "avx2"
            ))]
            // SAFETY: `ptr = bits.as_ptr()` is valid for the entire slice.
            // `chunk_start = wi_end + 1 - (done + STRIDE)` is ≥ 0 because
            // `done + STRIDE ≤ total_words = wi_end + 1 - mid_first ≤ wi_end + 1`.
            // The `ptr.add(chunk_start)` thus stays within `[ptr, ptr + wi_end + 1)`.
            // AVX2 is available per `#[target_feature]` gating.
            unsafe {
                #[cfg(target_arch = "x86")]
                use core::arch::x86::{
                    __m256i, _mm256_loadu_si256, _mm256_set1_epi64x, _mm256_testz_si256,
                    _mm256_xor_si256,
                };
                #[cfg(target_arch = "x86_64")]
                use core::arch::x86_64::{
                    __m256i, _mm256_loadu_si256, _mm256_set1_epi64x, _mm256_testz_si256,
                    _mm256_xor_si256,
                };
                const LANES: usize = 4;
                const STRIDE: usize = 8;

                macro_rules! is_all_fill_chunk {
                    ($ptr:expr) => {
                        if FILL == 0 {
                            let d = _mm256_loadu_si256($ptr.cast::<__m256i>());
                            _mm256_testz_si256(d, d) != 0
                        } else {
                            let fill_vec = _mm256_set1_epi64x(FILL as i64);
                            let d = _mm256_loadu_si256($ptr.cast::<__m256i>());
                            let x = _mm256_xor_si256(d, fill_vec);
                            _mm256_testz_si256(x, x) != 0
                        }
                    };
                }

                // 2×‑unrolled
                while done + STRIDE <= total_words {
                    let chunk_start = wi_end + 1 - (done + STRIDE);
                    let d0_ok = is_all_fill_chunk!(ptr.add(chunk_start));
                    let d1_ok = is_all_fill_chunk!(ptr.add(chunk_start + LANES));
                    if !d0_ok || !d1_ok {
                        break;
                    }
                    scanned += STRIDE * WORD_BITS;
                    done += STRIDE;
                }
                // Single-chunk remainder
                while done + LANES <= total_words {
                    let chunk_start = wi_end + 1 - (done + LANES);
                    if !is_all_fill_chunk!(ptr.add(chunk_start)) {
                        break;
                    }
                    scanned += LANES * WORD_BITS;
                    done += LANES;
                }
            }

            // ── SSE2 ─────────────────────────────────────────────────
            #[cfg(all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "sse2",
                not(target_feature = "avx2")
            ))]
            // SAFETY: same chunk_start bound as AVX2 (adjusted for LANES_2X/ LANES).
            // SSE2 is baseline on x86-64 per `#[cfg(target_feature = "sse2")]`.
            unsafe {
                while done + LANES_2X <= total_words {
                    let chunk_start = wi_end + 1 - (done + LANES_2X);
                    if !chunk_eq_2x::<FILL>(ptr.add(chunk_start)) {
                        break;
                    }
                    scanned += LANES_2X * WORD_BITS;
                    done += LANES_2X;
                }
                while done + LANES <= total_words {
                    let chunk_start = wi_end + 1 - (done + LANES);
                    if !chunk_eq::<FILL>(ptr.add(chunk_start)) {
                        break;
                    }
                    scanned += LANES * WORD_BITS;
                    done += LANES;
                }
            }

            // ── NEON ─────────────────────────────────────────────────
            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
            // SAFETY: same pointer-bound invariant as SSE2 path.
            // NEON is available per `#[target_feature]` gating.
            unsafe {
                while done + LANES_2X <= total_words {
                    let chunk_start = wi_end + 1 - (done + LANES_2X);
                    if !chunk_eq_2x::<FILL>(ptr.add(chunk_start)) {
                        break;
                    }
                    scanned += LANES_2X * WORD_BITS;
                    done += LANES_2X;
                }
                while done + LANES <= total_words {
                    let chunk_start = wi_end + 1 - (done + LANES);
                    if !chunk_eq::<FILL>(ptr.add(chunk_start)) {
                        break;
                    }
                    scanned += LANES * WORD_BITS;
                    done += LANES;
                }
            }

            // ── Scalar ───────────────────────────────────────────────
            #[cfg(not(any(
                all(
                    any(target_arch = "x86", target_arch = "x86_64"),
                    any(target_feature = "avx2", target_feature = "sse2")
                ),
                all(target_arch = "aarch64", target_feature = "neon"),
            )))]
            // SAFETY: same chunk_start bound as the SIMD paths above.
            // `chunk_eq` requires `LANES` valid u64 reads, ensured by
            // `done + LANES ≤ total_words`.
            unsafe {
                while done + LANES <= total_words {
                    let chunk_start = wi_end + 1 - (done + LANES);
                    if !chunk_eq::<FILL>(ptr.add(chunk_start)) {
                        break;
                    }
                    scanned += LANES * WORD_BITS;
                    done += LANES;
                }
            }
        } // else (SIMD path)

        // ── Scalar tail ──────────────────────────────────────────
        while done < total_words {
            let wi = wi_end - done;
            if bits[wi] != FILL {
                scanned += count_leading::<FILL>(bits[wi]);
                return scanned.min(bit_len);
            }
            scanned += WORD_BITS;
            done += 1;
        }
    }

    // ── First-word partial (trailing side) ───────────────────────
    if !WORD_ALIGNED && start_offset > 0 {
        let first_limit = WORD_BITS - start_offset as usize;
        let first_val = bits[0] >> start_offset;
        let first_count = count_leading_within::<FILL>(first_val, first_limit);
        scanned += first_count.min(first_limit);
    }

    scanned.min(bit_len)
}
