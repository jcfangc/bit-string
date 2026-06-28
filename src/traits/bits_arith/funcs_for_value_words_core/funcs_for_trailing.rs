//! SIMD-accelerated trailing value-word scan (reverse direction).
//!
//! Counts consecutive words equal to a fill value (zero or ones) from the
//! end of a slice.  All functions are parameterised by `const FILL: u64`
//! so the two fill variants (`0` / `u64::MAX`) are monomorphised separately
//! and runtime fill-dispatch branches are eliminated.

use crate::SMALL_WORDS;

// ---------------------------------------------------------------------------
// Dispatch — selects backend at compile time
// ---------------------------------------------------------------------------

/// Returns the number of consecutive words equal to the fill value at the
/// **end** of `words`.
///
/// Use [`FILL_ZEROS`](super::FILL_ZEROS) for trailing zeros,
/// [`FILL_ONES`](super::FILL_ONES) for trailing ones.  All words from
/// `words.len() - count` onwards match.  If the return value equals
/// `words.len()`, every word matches.
#[inline]
pub(crate) fn trailing_value_words<const FILL: u64>(words: &[u64]) -> usize {
    if words.len() < SMALL_WORDS {
        return scalar::scan_rev::<FILL>(words);
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        unsafe {
            return avx2::scan_rev::<FILL>(words);
        }
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1",
        not(target_feature = "avx2")
    ))]
    {
        unsafe {
            return sse41::scan_rev::<FILL>(words);
        }
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        unsafe {
            return neon::scan_rev::<FILL>(words);
        }
    }

    #[allow(unused)]
    scalar::scan_rev::<FILL>(words)
}

// ---------------------------------------------------------------------------
// Scalar
// ---------------------------------------------------------------------------

mod scalar {
    #[inline]
    pub(super) fn scan_rev<const FILL: u64>(words: &[u64]) -> usize {
        for i in 0..words.len() {
            if words[words.len() - 1 - i] != FILL {
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

    /// AVX2 backend — reverse scan (from the end backwards).
    ///
    /// # Safety
    ///
    /// Caller must ensure AVX2 is available.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn scan_rev<const FILL: u64>(words: &[u64]) -> usize {
        unsafe {
            let fill_vec = _mm256_set1_epi64x(FILL as i64);
            let full_chunks = words.len() / LANES;
            let done = full_chunks * LANES;

            // Process the partial tail first.
            let tail_count = super::scalar::scan_rev::<FILL>(&words[done..]);
            if tail_count < words.len() - done {
                return tail_count;
            }
            let mut count = tail_count;

            // Full chunks from right to left.
            for chunk in (0..full_chunks).rev() {
                let offset = chunk * LANES;
                let data = _mm256_loadu_si256(words.as_ptr().add(offset).cast::<__m256i>());
                let cmp = _mm256_cmpeq_epi64(data, fill_vec);
                let mask = _mm256_movemask_epi8(cmp) as u32;
                if mask != 0xFFFF_FFFF {
                    for k in (0..LANES).rev() {
                        if (mask >> (k * 8)) & 0xFF != 0xFF {
                            return count + (LANES - 1 - k);
                        }
                    }
                    core::hint::unreachable_unchecked();
                }
                count += LANES;
            }

            count
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

    /// SSE4.1 backend — reverse scan.
    ///
    /// # Safety
    ///
    /// Caller must ensure SSE4.1 is available.
    #[target_feature(enable = "sse4.1")]
    pub(super) unsafe fn scan_rev<const FILL: u64>(words: &[u64]) -> usize {
        unsafe {
            let fill_vec = _mm_set1_epi64x(FILL as i64);
            let full_chunks = words.len() / LANES;
            let done = full_chunks * LANES;

            let tail_count = super::scalar::scan_rev::<FILL>(&words[done..]);
            if tail_count < words.len() - done {
                return tail_count;
            }
            let mut count = tail_count;

            for chunk in (0..full_chunks).rev() {
                let offset = chunk * LANES;
                let data = _mm_loadu_si128(words.as_ptr().add(offset).cast::<__m128i>());
                let cmp = _mm_cmpeq_epi64(data, fill_vec);
                let mask = _mm_movemask_epi8(cmp) as u32;
                if mask != 0xFFFF {
                    for k in (0..LANES).rev() {
                        if (mask >> (k * 8)) & 0xFF != 0xFF {
                            return count + (LANES - 1 - k);
                        }
                    }
                    core::hint::unreachable_unchecked();
                }
                count += LANES;
            }

            count
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

    /// NEON backend — reverse scan.
    ///
    /// # Safety
    ///
    /// Caller must ensure NEON is available.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn scan_rev<const FILL: u64>(words: &[u64]) -> usize {
        unsafe {
            let fill_vec = vdupq_n_u64(FILL);
            let full_chunks = words.len() / LANES;
            let done = full_chunks * LANES;

            let tail_count = super::scalar::scan_rev::<FILL>(&words[done..]);
            if tail_count < words.len() - done {
                return tail_count;
            }
            let mut count = tail_count;

            for chunk in (0..full_chunks).rev() {
                let offset = chunk * LANES;
                let data = vld1q_u64(words.as_ptr().add(offset));
                let cmp = vceqq_u64(data, fill_vec);
                if vgetq_lane_u64(cmp, 1) == 0 {
                    return count;
                }
                if vgetq_lane_u64(cmp, 0) == 0 {
                    return count + 1;
                }
                count += LANES;
            }

            count
        }
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
