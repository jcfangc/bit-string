//! SIMD-accelerated leading-zero-word scan.
//!
//! Counts consecutive zero-valued `u64` words from the start of a slice.

use crate::SMALL_WORDS;

/// Returns the number of consecutive zero words at the start of `words`.
///
/// All words up to (but not including) the returned index are zero.  If the
/// return value equals `words.len()`, every word is zero.
#[inline]
pub(crate) fn leading_zero_words(words: &[u64]) -> usize {
    if words.len() < SMALL_WORDS {
        return scalar::scan(words);
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY: AVX2 is available (compiled with target_feature check).
        unsafe {
            return avx2::scan(words);
        }
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY: SSE2 is available.
        unsafe {
            return sse2::scan(words);
        }
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY: NEON is available.
        unsafe {
            return neon::scan(words);
        }
    }

    #[allow(unused)]
    scalar::scan(words)
}

// ---------------------------------------------------------------------------
// Scalar
// ---------------------------------------------------------------------------

mod scalar {
    #[inline]
    pub(super) fn scan(words: &[u64]) -> usize {
        for (i, &w) in words.iter().enumerate() {
            if w != 0 {
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
        __m256i, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_setzero_si256,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_setzero_si256,
    };

    const LANES: usize = 4;

    /// AVX2 backend.
    ///
    /// # Safety
    ///
    /// Caller must ensure AVX2 is available.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn scan(words: &[u64]) -> usize {
        // SAFETY: all operations inside require AVX2, which is guaranteed by
        // the `target_feature` annotation on this function.
        unsafe {
            let zero = _mm256_setzero_si256();
            let chunks = words.len() / LANES;
            let mut count = 0usize;

            for chunk in 0..chunks {
                let offset = chunk * LANES;
                // SAFETY: offset + LANES ≤ words.len(); _mm256_loadu_si256
                // supports unaligned reads.
                let data = _mm256_loadu_si256(words.as_ptr().add(offset).cast::<__m256i>());
                // Compare each u64 lane to zero → -1 for zero, 0 for non-zero.
                let cmp = _mm256_cmpeq_epi64(data, zero);
                // movemask extracts the MSB of each byte → 32-bit mask.
                // A zero-word lane (all bytes 0xFF after cmpeq) contributes
                // 0xFF (8 bits of 1).  A non-zero lane contributes 0x00
                // (8 bits of 0).
                let mask = _mm256_movemask_epi8(cmp) as u32;
                if mask != 0xFFFF_FFFF {
                    // At least one word is non-zero. Find the first one.
                    for k in 0..LANES {
                        if (mask >> (k * 8)) & 0xFF != 0xFF {
                            return count + k;
                        }
                    }
                    // SAFETY: mask != 0xFFFF_FFFF guarantees at least one
                    // lane is non-zero, so the loop above always returns.
                    core::hint::unreachable_unchecked();
                }
                count += LANES;
            }

            let done = chunks * LANES;
            count + super::scalar::scan(&words[done..])
        }
    }
}

// ---------------------------------------------------------------------------
// SSE2 — 2 words at a time
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8, _mm_setzero_si128,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8, _mm_setzero_si128,
    };

    const LANES: usize = 2;

    /// SSE2 backend.
    ///
    /// # Safety
    ///
    /// Caller must ensure SSE2 is available.
    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn scan(words: &[u64]) -> usize {
        // SAFETY: all operations inside require SSE2, which is guaranteed by
        // the `target_feature` annotation.
        unsafe {
            let zero = _mm_setzero_si128();
            let chunks = words.len() / LANES;
            let mut count = 0usize;

            for chunk in 0..chunks {
                let offset = chunk * LANES;
                let data = _mm_loadu_si128(words.as_ptr().add(offset).cast::<__m128i>());
                let cmp = _mm_cmpeq_epi64(data, zero);
                let mask = _mm_movemask_epi8(cmp) as u32;
                // 16-bit mask from 2 u64 lanes: 2 × 8 bytes = 16 bits.
                if mask != 0xFFFF {
                    for k in 0..LANES {
                        if (mask >> (k * 8)) & 0xFF != 0xFF {
                            return count + k;
                        }
                    }
                    // SAFETY: mask != 0xFFFF guarantees at least one lane is
                    // non-zero, so the loop above always returns.
                    core::hint::unreachable_unchecked();
                }
                count += LANES;
            }

            let done = chunks * LANES;
            count + super::scalar::scan(&words[done..])
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

    /// NEON backend.
    ///
    /// # Safety
    ///
    /// Caller must ensure NEON is available.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn scan(words: &[u64]) -> usize {
        // SAFETY: all operations inside require NEON, which is guaranteed by
        // the `target_feature` annotation.
        unsafe {
            let zero = vdupq_n_u64(0);
            let chunks = words.len() / LANES;
            let mut count = 0usize;

            for chunk in 0..chunks {
                let offset = chunk * LANES;
                let data = vld1q_u64(words.as_ptr().add(offset));
                let cmp = vceqq_u64(data, zero);
                // vceqq returns all-1s (-1) for equal, all-0s for not equal.
                if vgetq_lane_u64(cmp, 0) == 0 {
                    // First word is non-zero.
                    return count;
                }
                if vgetq_lane_u64(cmp, 1) == 0 {
                    // Second word is non-zero.
                    return count + 1;
                }
                count += LANES;
            }

            let done = chunks * LANES;
            count + super::scalar::scan(&words[done..])
        }
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
