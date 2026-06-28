//! SIMD-accelerated leading value-word scan (forward direction).
//!
//! Counts consecutive words equal to a fill value (`FILL`) from the start of
//! a slice.  Each SIMD backend only tests whether a full chunk matches (hot
//! path: one `ptest` per chunk); when a mismatch is detected the tail is
//! handed to the scalar loop — there is no lane-level scanning in SIMD code.

use crate::SMALL_WORDS;

// ---------------------------------------------------------------------------
// Dispatch — selects backend at compile time
// ---------------------------------------------------------------------------

/// Returns the number of consecutive words equal to `FILL` at the start of
/// `words`.
///
/// Use [`FILL_ZEROS`](super::FILL_ZEROS) for leading zeros,
/// [`FILL_ONES`](super::FILL_ONES) for leading ones.  All words up to
/// (but not including) the returned index match.  If the return value
/// equals `words.len()`, every word matches.
#[inline]
pub(crate) fn leading_value_words<const FILL: u64>(words: &[u64]) -> usize {
    if words.len() < SMALL_WORDS {
        return scalar::scan::<FILL>(words);
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY: AVX2 is available (compiled with target_feature check).
        unsafe {
            return avx2::scan::<FILL>(words);
        }
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY: SSE4.1 is available.
        unsafe {
            return sse41::scan::<FILL>(words);
        }
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY: NEON is available.
        unsafe {
            return neon::scan::<FILL>(words);
        }
    }

    #[allow(unused)]
    scalar::scan::<FILL>(words)
}

// ---------------------------------------------------------------------------
// Scalar
// ---------------------------------------------------------------------------

mod scalar {
    #[inline]
    pub(super) fn scan<const FILL: u64>(words: &[u64]) -> usize {
        for (i, &w) in words.iter().enumerate() {
            if w != FILL {
                return i;
            }
        }
        words.len()
    }
}

// ---------------------------------------------------------------------------
// AVX2 — 4 words at a time
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m256i, _mm256_loadu_si256, _mm256_set1_epi64x, _mm256_testz_si256, _mm256_xor_si256,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_loadu_si256, _mm256_set1_epi64x, _mm256_testz_si256, _mm256_xor_si256,
    };

    const LANES: usize = 4;

    /// AVX2 backend — forward scan.
    ///
    /// Hot path: one `vptest` per 4-word chunk.  On mismatch the chunk
    /// (and any remaining words) are delegated to the scalar loop — at
    /// most `LANES` words of scalar work per call.
    ///
    /// # Safety
    ///
    /// Caller must ensure AVX2 is available.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn scan<const FILL: u64>(words: &[u64]) -> usize {
        // SAFETY: all operations inside require AVX2, guaranteed by
        // `target_feature`.
        unsafe {
            let chunks = words.len() / LANES;

            for chunk in 0..chunks {
                let offset = chunk * LANES;
                // SAFETY: offset + LANES ≤ words.len(); unaligned load.
                let data = _mm256_loadu_si256(words.as_ptr().add(offset).cast::<__m256i>());

                // `vptest` sets ZF when (data & data) == 0, i.e. data is all
                // zeros.  For FILL != 0 we first xor against the broadcast fill
                // so zeros in the xor'd result mean equality.
                let all_eq = if FILL == 0 {
                    _mm256_testz_si256(data, data) != 0
                } else {
                    let fill_vec = _mm256_set1_epi64x(FILL as i64);
                    let xor = _mm256_xor_si256(data, fill_vec);
                    _mm256_testz_si256(xor, xor) != 0
                };

                if !all_eq {
                    // Fall to scalar for the mismatching chunk and beyond.
                    return chunk * LANES + super::scalar::scan::<FILL>(&words[offset..]);
                }
            }

            let done = chunks * LANES;
            done + super::scalar::scan::<FILL>(&words[done..])
        }
    }
}

// ---------------------------------------------------------------------------
// SSE4.1 — 2 words at a time
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse41 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, _mm_loadu_si128, _mm_set1_epi64x, _mm_testz_si128, _mm_xor_si128,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, _mm_loadu_si128, _mm_set1_epi64x, _mm_testz_si128, _mm_xor_si128,
    };

    const LANES: usize = 2;

    /// SSE4.1 backend — forward scan.
    ///
    /// Hot path: one `ptest` per 2-word chunk; scalar fallback on mismatch.
    ///
    /// # Safety
    ///
    /// Caller must ensure SSE4.1 is available.
    #[target_feature(enable = "sse4.1")]
    pub(super) unsafe fn scan<const FILL: u64>(words: &[u64]) -> usize {
        // SAFETY: all operations inside require SSE4.1, guaranteed by
        // `target_feature`.
        unsafe {
            let chunks = words.len() / LANES;

            for chunk in 0..chunks {
                let offset = chunk * LANES;
                let data = _mm_loadu_si128(words.as_ptr().add(offset).cast::<__m128i>());

                let all_eq = if FILL == 0 {
                    _mm_testz_si128(data, data) != 0
                } else {
                    let fill_vec = _mm_set1_epi64x(FILL as i64);
                    let xor = _mm_xor_si128(data, fill_vec);
                    _mm_testz_si128(xor, xor) != 0
                };

                if !all_eq {
                    return chunk * LANES + super::scalar::scan::<FILL>(&words[offset..]);
                }
            }

            let done = chunks * LANES;
            done + super::scalar::scan::<FILL>(&words[done..])
        }
    }
}

// ---------------------------------------------------------------------------
// NEON — 2 words at a time
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use core::arch::aarch64::{uint64x2_t, vceqq_u64, vdupq_n_u64, vgetq_lane_u64, vld1q_u64};

    const LANES: usize = 2;

    /// NEON backend — forward scan.
    ///
    /// # Safety
    ///
    /// Caller must ensure NEON is available.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn scan<const FILL: u64>(words: &[u64]) -> usize {
        // SAFETY: all operations inside require NEON, guaranteed by
        // `target_feature`.
        unsafe {
            let fill_vec = vdupq_n_u64(FILL);
            let chunks = words.len() / LANES;

            for chunk in 0..chunks {
                let offset = chunk * LANES;
                let data = vld1q_u64(words.as_ptr().add(offset));
                let cmp = vceqq_u64(data, fill_vec);

                if vgetq_lane_u64(cmp, 0) == 0 {
                    return chunk * LANES;
                }
                if vgetq_lane_u64(cmp, 1) == 0 {
                    return chunk * LANES + 1;
                }
            }

            let done = chunks * LANES;
            done + super::scalar::scan::<FILL>(&words[done..])
        }
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
