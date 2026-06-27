//! SIMD-accelerated leading-value-word scan.
//!
//! Counts consecutive words equal to `fill` from the start of a slice.
//! Used to implement both `leading_zero_words` (`fill = 0`) and
//! `leading_one_words` (`fill = !0`).

use crate::SMALL_WORDS;

/// Returns the number of consecutive words equal to `fill` at the start of
/// `words`.
///
/// All words up to (but not including) the returned index equal `fill`.  If
/// the return value equals `words.len()`, every word equals `fill`.
#[inline]
pub(crate) fn leading_value_words(words: &[u64], fill: u64) -> usize {
    if words.len() < SMALL_WORDS {
        return scalar::scan(words, fill);
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY: AVX2 is available (compiled with target_feature check).
        unsafe {
            return avx2::scan(words, fill);
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
            return sse41::scan(words, fill);
        }
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY: NEON is available.
        unsafe {
            return neon::scan(words, fill);
        }
    }

    #[allow(unused)]
    scalar::scan(words, fill)
}

/// Returns the number of consecutive words equal to `fill` at the **end** of
/// `words`.
///
/// All words from `words.len() - count` onwards equal `fill`.  If the return
/// value equals `words.len()`, every word equals `fill`.
#[inline]
pub(crate) fn trailing_value_words(words: &[u64], fill: u64) -> usize {
    if words.len() < SMALL_WORDS {
        return scalar::scan_rev(words, fill);
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        unsafe {
            return avx2::scan_rev(words, fill);
        }
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1",
        not(target_feature = "avx2")
    ))]
    {
        unsafe {
            return sse41::scan_rev(words, fill);
        }
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        unsafe {
            return neon::scan_rev(words, fill);
        }
    }

    #[allow(unused)]
    scalar::scan_rev(words, fill)
}

// ---------------------------------------------------------------------------
// Scalar
// ---------------------------------------------------------------------------

mod scalar {
    #[inline]
    pub(super) fn scan(words: &[u64], fill: u64) -> usize {
        for (i, &w) in words.iter().enumerate() {
            if w != fill {
                return i;
            }
        }
        words.len()
    }

    #[inline]
    pub(super) fn scan_rev(words: &[u64], fill: u64) -> usize {
        for i in 0..words.len() {
            if words[words.len() - 1 - i] != fill {
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

    /// AVX2 backend.
    ///
    /// # Safety
    ///
    /// Caller must ensure AVX2 is available.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn scan(words: &[u64], fill: u64) -> usize {
        // SAFETY: all operations inside require AVX2, which is guaranteed by
        // the `target_feature` annotation on this function.
        unsafe {
            let fill_vec = _mm256_set1_epi64x(fill as i64);
            let chunks = words.len() / LANES;
            let mut count = 0usize;

            for chunk in 0..chunks {
                let offset = chunk * LANES;
                // SAFETY: offset + LANES ≤ words.len(); _mm256_loadu_si256
                // supports unaligned reads.
                let data = _mm256_loadu_si256(words.as_ptr().add(offset).cast::<__m256i>());
                // Compare each u64 lane to fill → -1 for equal, 0 for not.
                let cmp = _mm256_cmpeq_epi64(data, fill_vec);
                // movemask extracts the MSB of each byte → 32-bit mask.
                // An equal-word lane (all bytes 0xFF after cmpeq) contributes
                // 0xFF (8 bits of 1).  A non-equal lane contributes 0x00
                // (8 bits of 0).
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
            count + super::scalar::scan(&words[done..], fill)
        }
    }

    /// AVX2 backend — reverse scan.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn scan_rev(words: &[u64], fill: u64) -> usize {
        unsafe {
            let fill_vec = _mm256_set1_epi64x(fill as i64);
            let full_chunks = words.len() / LANES;
            let done = full_chunks * LANES;

            // Process the partial tail first.
            let tail_count = super::scalar::scan_rev(&words[done..], fill);
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

    /// SSE4.1 backend.
    ///
    /// # Safety
    ///
    /// Caller must ensure SSE4.1 is available.
    #[target_feature(enable = "sse4.1")]
    pub(super) unsafe fn scan(words: &[u64], fill: u64) -> usize {
        // SAFETY: all operations inside require SSE4.1, which is guaranteed by
        // the `target_feature` annotation.
        unsafe {
            let fill_vec = _mm_set1_epi64x(fill as i64);
            let chunks = words.len() / LANES;
            let mut count = 0usize;

            for chunk in 0..chunks {
                let offset = chunk * LANES;
                let data = _mm_loadu_si128(words.as_ptr().add(offset).cast::<__m128i>());
                let cmp = _mm_cmpeq_epi64(data, fill_vec);
                let mask = _mm_movemask_epi8(cmp) as u32;
                // 16-bit mask from 2 u64 lanes: 2 × 8 bytes = 16 bits.
                if mask != 0xFFFF {
                    for k in 0..LANES {
                        if (mask >> (k * 8)) & 0xFF != 0xFF {
                            return count + k;
                        }
                    }
                    // SAFETY: mask != 0xFFFF guarantees at least one lane is
                    // not equal to fill, so the loop always returns.
                    core::hint::unreachable_unchecked();
                }
                count += LANES;
            }

            let done = chunks * LANES;
            count + super::scalar::scan(&words[done..], fill)
        }
    }

    /// SSE4.1 backend — reverse scan.
    #[target_feature(enable = "sse4.1")]
    pub(super) unsafe fn scan_rev(words: &[u64], fill: u64) -> usize {
        unsafe {
            let fill_vec = _mm_set1_epi64x(fill as i64);
            let full_chunks = words.len() / LANES;
            let done = full_chunks * LANES;

            let tail_count = super::scalar::scan_rev(&words[done..], fill);
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

    /// NEON backend.
    ///
    /// # Safety
    ///
    /// Caller must ensure NEON is available.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn scan(words: &[u64], fill: u64) -> usize {
        // SAFETY: all operations inside require NEON, which is guaranteed by
        // the `target_feature` annotation.
        unsafe {
            let fill_vec = vdupq_n_u64(fill);
            let chunks = words.len() / LANES;
            let mut count = 0usize;

            for chunk in 0..chunks {
                let offset = chunk * LANES;
                let data = vld1q_u64(words.as_ptr().add(offset));
                let cmp = vceqq_u64(data, fill_vec);
                // vceqq returns all-1s (-1) for equal, all-0s for not equal.
                if vgetq_lane_u64(cmp, 0) == 0 {
                    return count;
                }
                if vgetq_lane_u64(cmp, 1) == 0 {
                    return count + 1;
                }
                count += LANES;
            }

            let done = chunks * LANES;
            count + super::scalar::scan(&words[done..], fill)
        }
    }

    /// NEON backend — reverse scan.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn scan_rev(words: &[u64], fill: u64) -> usize {
        unsafe {
            let fill_vec = vdupq_n_u64(fill);
            let full_chunks = words.len() / LANES;
            let done = full_chunks * LANES;

            let tail_count = super::scalar::scan_rev(&words[done..], fill);
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
