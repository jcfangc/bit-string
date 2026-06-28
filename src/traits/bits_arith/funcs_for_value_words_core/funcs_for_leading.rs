//! SIMD-accelerated leading value-word scan (forward direction).
//!
//! Counts consecutive words equal to a fill value (zero or ones) from the
//! start of a slice.  All functions are parameterised by `const FILL: u64`
//! so the two fill variants (`0` / `u64::MAX`) are monomorphised separately
//! and runtime fill-dispatch branches are eliminated.

use crate::SMALL_WORDS;

// ---------------------------------------------------------------------------
// Dispatch — selects backend at compile time
// ---------------------------------------------------------------------------

/// Returns the number of consecutive words equal to the fill value at the
/// start of `words`.
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
        __m256i, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_set1_epi64x,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_set1_epi64x,
    };

    const LANES: usize = 4;

    /// AVX2 backend — forward scan.
    ///
    /// # Safety
    ///
    /// Caller must ensure AVX2 is available.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn scan<const FILL: u64>(words: &[u64]) -> usize {
        // SAFETY: all operations inside require AVX2, which is guaranteed by
        // the `target_feature` annotation on this function.
        unsafe {
            let fill_vec = _mm256_set1_epi64x(FILL as i64);
            let chunks = words.len() / LANES;
            let mut count = 0usize;

            for chunk in 0..chunks {
                let offset = chunk * LANES;
                // SAFETY: offset + LANES ≤ words.len(); _mm256_loadu_si256
                // supports unaligned reads.
                let data = _mm256_loadu_si256(words.as_ptr().add(offset).cast::<__m256i>());
                let cmp = _mm256_cmpeq_epi64(data, fill_vec);
                let mask = _mm256_movemask_epi8(cmp) as u32;
                if mask != 0xFFFF_FFFF {
                    for k in 0..LANES {
                        if (mask >> (k * 8)) & 0xFF != 0xFF {
                            return count + k;
                        }
                    }
                    // SAFETY: mask != 0xFFFF_FFFF guarantees at least one
                    // lane is not equal to fill, so the loop always returns.
                    core::hint::unreachable_unchecked();
                }
                count += LANES;
            }

            let done = chunks * LANES;
            count + super::scalar::scan::<FILL>(&words[done..])
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
        __m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi64x,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi64x,
    };

    const LANES: usize = 2;

    /// SSE4.1 backend — forward scan.
    ///
    /// # Safety
    ///
    /// Caller must ensure SSE4.1 is available.
    #[target_feature(enable = "sse4.1")]
    pub(super) unsafe fn scan<const FILL: u64>(words: &[u64]) -> usize {
        // SAFETY: all operations inside require SSE4.1, which is guaranteed by
        // the `target_feature` annotation.
        unsafe {
            let fill_vec = _mm_set1_epi64x(FILL as i64);
            let chunks = words.len() / LANES;
            let mut count = 0usize;

            for chunk in 0..chunks {
                let offset = chunk * LANES;
                let data = _mm_loadu_si128(words.as_ptr().add(offset).cast::<__m128i>());
                let cmp = _mm_cmpeq_epi64(data, fill_vec);
                let mask = _mm_movemask_epi8(cmp) as u32;
                if mask != 0xFFFF {
                    for k in 0..LANES {
                        if (mask >> (k * 8)) & 0xFF != 0xFF {
                            return count + k;
                        }
                    }
                    core::hint::unreachable_unchecked();
                }
                count += LANES;
            }

            let done = chunks * LANES;
            count + super::scalar::scan::<FILL>(&words[done..])
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
        // SAFETY: all operations inside require NEON, which is guaranteed by
        // the `target_feature` annotation.
        unsafe {
            let fill_vec = vdupq_n_u64(FILL);
            let chunks = words.len() / LANES;
            let mut count = 0usize;

            for chunk in 0..chunks {
                let offset = chunk * LANES;
                let data = vld1q_u64(words.as_ptr().add(offset));
                let cmp = vceqq_u64(data, fill_vec);
                if vgetq_lane_u64(cmp, 0) == 0 {
                    return count;
                }
                if vgetq_lane_u64(cmp, 1) == 0 {
                    return count + 1;
                }
                count += LANES;
            }

            let done = chunks * LANES;
            count + super::scalar::scan::<FILL>(&words[done..])
        }
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
