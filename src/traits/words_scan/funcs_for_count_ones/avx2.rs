use super::scalar;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m256i, _mm256_add_epi8, _mm256_add_epi64, _mm256_and_si256, _mm256_loadu_si256,
    _mm256_sad_epu8, _mm256_set1_epi8, _mm256_setr_epi8, _mm256_setzero_si256, _mm256_shuffle_epi8,
    _mm256_srli_epi16, _mm256_storeu_si256,
};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m256i, _mm256_add_epi8, _mm256_add_epi64, _mm256_and_si256, _mm256_loadu_si256,
    _mm256_sad_epu8, _mm256_set1_epi8, _mm256_setr_epi8, _mm256_setzero_si256, _mm256_shuffle_epi8,
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

    // SAFETY:
    // - These constructors require AVX2 to be available.
    // - This function is compiled with `target_feature = "avx2"`.
    let lookup = _mm256_setr_epi8(
        0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, //
        0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4,
    );
    let low_mask = _mm256_set1_epi8(0x0F);
    let zero = _mm256_setzero_si256();

    // Accumulate popcounts in a SIMD register to avoid per-chunk
    // store-to-memory round-trips. Reduced to scalar only once at the end.
    let mut acc = zero;

    for chunk in 0..chunks {
        let offset = chunk * LANES;

        // SAFETY:
        // - `offset + LANES <= len`.
        // - `_mm256_loadu_si256` permits unaligned reads.
        // - `src` validity is guaranteed by the caller.
        unsafe {
            let bytes = _mm256_loadu_si256(src.add(offset).cast::<__m256i>());

            let low = _mm256_and_si256(bytes, low_mask);
            let high = _mm256_and_si256(_mm256_srli_epi16(bytes, 4), low_mask);

            let low_counts = _mm256_shuffle_epi8(lookup, low);
            let high_counts = _mm256_shuffle_epi8(lookup, high);
            let byte_counts = _mm256_add_epi8(low_counts, high_counts);

            let sums = _mm256_sad_epu8(byte_counts, zero);
            acc = _mm256_add_epi64(acc, sums);
        }
    }

    let mut lane_sums = [0u64; LANES];
    // SAFETY: `lane_sums` has sufficient space for 4 u64 values.
    unsafe { _mm256_storeu_si256(lane_sums.as_mut_ptr().cast::<__m256i>(), acc) };
    let mut count = lane_sums.iter().map(|&sum| sum as usize).sum::<usize>();

    let done = chunks * LANES;

    // SAFETY:
    // - `done <= len`.
    // - Tail range is `done..len`.
    // - Pointer validity is guaranteed by the caller.
    count += unsafe { scalar::count_words(src.add(done), len - done) };
    count
}
