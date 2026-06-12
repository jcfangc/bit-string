use crate::bit_string::funcs_for_share::last_word_mask;

use super::*;

#[inline]
pub(super) fn count_ones(bits: &[u64], bit_len: usize) -> usize {
    let full_words = bit_len / WORD_BITS;
    let rem = bit_len % WORD_BITS;

    let mut count = count_full_words(&bits[..full_words]);

    if rem != 0 {
        count += (bits[full_words] & last_word_mask(bit_len)).count_ones() as usize;
    }

    count
}

#[inline]
fn count_full_words(words: &[u64]) -> usize {
    // SAFETY:
    // - `words.as_ptr()` is valid for reads of `words.len()` u64 values.
    // - `dispatch` only reads from `words[0..len]`.
    unsafe { dispatch(words.as_ptr(), words.len()) }
}

/// Counts set bits in `src[0..len]`.
///
/// # Safety
///
/// - `src` must be valid for reads of `len` initialized `u64` values.
#[inline]
unsafe fn dispatch(src: *const u64, len: usize) -> usize {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when AVX2 is enabled.
        return unsafe { avx2::count_words(src, len) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "ssse3",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when SSSE3 is enabled.
        return unsafe { ssse3::count_words(src, len) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when NEON is enabled.
        return unsafe { neon::count_words(src, len) };
    }

    #[allow(unused)]
    // SAFETY: Forwarded from `dispatch`'s safety contract.
    unsafe {
        scalar::count_words(src, len)
    }
}

#[allow(unused)]
mod scalar {
    /// Scalar backend for counting set bits in `src[0..len]`.
    ///
    /// # Safety
    ///
    /// - `src` must be valid for reads of `len` initialized `u64` values.
    #[inline]
    pub(super) unsafe fn count_words(src: *const u64, len: usize) -> usize {
        let mut count = 0usize;

        for i in 0..len {
            // SAFETY:
            // - `i < len`.
            // - Pointer validity is guaranteed by the caller.
            unsafe {
                count += src.add(i).read().count_ones() as usize;
            }
        }

        count
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use super::scalar;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m256i, _mm256_add_epi8, _mm256_and_si256, _mm256_loadu_si256, _mm256_sad_epu8,
        _mm256_set1_epi8, _mm256_setr_epi8, _mm256_setzero_si256, _mm256_shuffle_epi8,
        _mm256_srli_epi16, _mm256_storeu_si256,
    };

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_add_epi8, _mm256_and_si256, _mm256_loadu_si256, _mm256_sad_epu8,
        _mm256_set1_epi8, _mm256_setr_epi8, _mm256_setzero_si256, _mm256_shuffle_epi8,
        _mm256_srli_epi16, _mm256_storeu_si256,
    };

    const LANES: usize = 4;

    /// AVX2 backend for counting set bits in `src[0..len]`.
    ///
    /// Uses nibble lookup with `vpshufb`, then sums byte counts with `vpsadbw`.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when AVX2 is available.
    /// - `src` must be valid for reads of `len` initialized `u64` values.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn count_words(src: *const u64, len: usize) -> usize {
        let chunks = len / LANES;
        let mut count = 0usize;

        // SAFETY:
        // - These constructors require AVX2 to be available.
        // - This function is compiled with `target_feature = "avx2"`.
        let lookup = _mm256_setr_epi8(
            0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, //
            0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4,
        );
        let low_mask = _mm256_set1_epi8(0x0F);
        let zero = _mm256_setzero_si256();

        for chunk in 0..chunks {
            let offset = chunk * LANES;

            // SAFETY:
            // - `offset + LANES <= len`.
            // - `_mm256_loadu_si256` permits unaligned reads.
            // - `src` validity is guaranteed by the caller.
            // - `lane_sums` has exactly 4 u64 lanes for `_mm256_storeu_si256`.
            unsafe {
                let bytes = _mm256_loadu_si256(src.add(offset).cast::<__m256i>());

                let low = _mm256_and_si256(bytes, low_mask);
                let high = _mm256_and_si256(_mm256_srli_epi16(bytes, 4), low_mask);

                let low_counts = _mm256_shuffle_epi8(lookup, low);
                let high_counts = _mm256_shuffle_epi8(lookup, high);
                let byte_counts = _mm256_add_epi8(low_counts, high_counts);

                let sums = _mm256_sad_epu8(byte_counts, zero);

                let mut lane_sums = [0u64; LANES];
                _mm256_storeu_si256(lane_sums.as_mut_ptr().cast::<__m256i>(), sums);

                count += lane_sums.iter().map(|&sum| sum as usize).sum::<usize>();
            }
        }

        let done = chunks * LANES;

        // SAFETY:
        // - `done <= len`.
        // - Tail range is `done..len`.
        // - Pointer validity is guaranteed by the caller.
        count + unsafe { scalar::count_words(src.add(done), len - done) }
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod ssse3 {
    use super::scalar;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, _mm_add_epi8, _mm_and_si128, _mm_loadu_si128, _mm_sad_epu8, _mm_set1_epi8,
        _mm_setr_epi8, _mm_setzero_si128, _mm_shuffle_epi8, _mm_srli_epi16, _mm_storeu_si128,
    };

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, _mm_add_epi8, _mm_and_si128, _mm_loadu_si128, _mm_sad_epu8, _mm_set1_epi8,
        _mm_setr_epi8, _mm_setzero_si128, _mm_shuffle_epi8, _mm_srli_epi16, _mm_storeu_si128,
    };

    const LANES: usize = 2;

    /// SSSE3 backend for counting set bits in `src[0..len]`.
    ///
    /// Uses nibble lookup with `pshufb`, then sums byte counts with `psadbw`.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when SSSE3 is available.
    /// - `src` must be valid for reads of `len` initialized `u64` values.
    #[target_feature(enable = "ssse3")]
    pub(super) unsafe fn count_words(src: *const u64, len: usize) -> usize {
        let chunks = len / LANES;
        let mut count = 0usize;

        let lookup = _mm_setr_epi8(
            0, 1, 1, 2, 1, 2, 2, 3, //
            1, 2, 2, 3, 2, 3, 3, 4,
        );
        let low_mask = _mm_set1_epi8(0x0F);
        let zero = _mm_setzero_si128();

        for chunk in 0..chunks {
            let offset = chunk * LANES;

            // SAFETY:
            // - `offset + LANES <= len`.
            // - `_mm_loadu_si128` permits unaligned reads.
            // - `src` validity is guaranteed by the caller.
            // - `lane_sums` has exactly 2 u64 lanes for `_mm_storeu_si128`.
            unsafe {
                let bytes = _mm_loadu_si128(src.add(offset).cast::<__m128i>());

                let low = _mm_and_si128(bytes, low_mask);
                let high = _mm_and_si128(_mm_srli_epi16(bytes, 4), low_mask);

                let low_counts = _mm_shuffle_epi8(lookup, low);
                let high_counts = _mm_shuffle_epi8(lookup, high);
                let byte_counts = _mm_add_epi8(low_counts, high_counts);

                let sums = _mm_sad_epu8(byte_counts, zero);

                let mut lane_sums = [0u64; LANES];
                _mm_storeu_si128(lane_sums.as_mut_ptr().cast::<__m128i>(), sums);

                count += lane_sums.iter().map(|&sum| sum as usize).sum::<usize>();
            }
        }

        let done = chunks * LANES;

        // SAFETY:
        // - `done <= len`.
        // - Tail range is `done..len`.
        // - Pointer validity is guaranteed by the caller.
        count + unsafe { scalar::count_words(src.add(done), len - done) }
    }
}

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use super::scalar;

    use core::arch::aarch64::{vaddvq_u8, vcntq_u8, vld1q_u64, vreinterpretq_u8_u64};

    const LANES: usize = 2;

    /// NEON backend for counting set bits in `src[0..len]`.
    ///
    /// Counts bits per byte with `vcntq_u8`, then horizontally sums byte counts.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when NEON is available.
    /// - `src` must be valid for reads of `len` initialized `u64` values.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn count_words(src: *const u64, len: usize) -> usize {
        let chunks = len / LANES;
        let mut count = 0usize;

        for chunk in 0..chunks {
            let offset = chunk * LANES;

            // SAFETY:
            // - `offset + LANES <= len`.
            // - `vld1q_u64` reads exactly 2 u64 values.
            // - `src` validity is guaranteed by the caller.
            unsafe {
                let words = vld1q_u64(src.add(offset));
                let bytes = vreinterpretq_u8_u64(words);
                let byte_counts = vcntq_u8(bytes);

                count += vaddvq_u8(byte_counts) as usize;
            }
        }

        let done = chunks * LANES;

        // SAFETY:
        // - `done <= len`.
        // - Tail range is `done..len`.
        // - Pointer validity is guaranteed by the caller.
        count + unsafe { scalar::count_words(src.add(done), len - done) }
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
