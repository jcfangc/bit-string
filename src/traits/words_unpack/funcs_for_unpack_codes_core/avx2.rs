#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, __m256i, _mm_storeu_si128, _mm256_and_si256, _mm256_castsi256_si128,
    _mm256_extracti128_si256, _mm256_loadu_si256, _mm256_set1_epi8, _mm256_srli_epi16,
    _mm256_storeu_si256, _mm256_unpackhi_epi8, _mm256_unpacklo_epi8,
};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, __m256i, _mm_storeu_si128, _mm256_and_si256, _mm256_castsi256_si128,
    _mm256_extracti128_si256, _mm256_loadu_si256, _mm256_set1_epi8, _mm256_srli_epi16,
    _mm256_storeu_si256, _mm256_unpackhi_epi8, _mm256_unpacklo_epi8,
};

/// Unpacks complete 64-code BITS=4 blocks with AVX2 and uses scalar for the tail.
///
/// # Safety
///
/// The caller must ensure AVX2 is available, `words.len() * 16 == codes.len()`,
/// and both slices contain initialized storage for all elements.
#[target_feature(enable = "avx2")]
pub(super) unsafe fn unpack_four_bit(words: &[u64], codes: &mut [u8]) {
    debug_assert_eq!(codes.len() % 64, 0);
    debug_assert_eq!(words.len(), codes.len() / 16);

    let nibble_mask = _mm256_set1_epi8(0x0f);
    for (word_chunk, code_chunk) in words.chunks_exact(4).zip(codes.chunks_exact_mut(64)) {
        // SAFETY: A four-word chunk is exactly 32 initialized bytes.
        let packed = unsafe { _mm256_loadu_si256(word_chunk.as_ptr().cast::<__m256i>()) };
        let low = _mm256_and_si256(packed, nibble_mask);
        let high = _mm256_and_si256(_mm256_srli_epi16(packed, 4), nibble_mask);
        let first = _mm256_unpacklo_epi8(low, high);
        let second = _mm256_unpackhi_epi8(low, high);

        // AVX2 byte unpack is lane-local. Store its four 128-bit halves in
        // logical order: codes 0..16, 16..32, 32..48, then 48..64.
        unsafe {
            _mm_storeu_si128(
                code_chunk.as_mut_ptr().cast::<__m128i>(),
                _mm256_castsi256_si128(first),
            );
            _mm_storeu_si128(
                code_chunk.as_mut_ptr().add(16).cast::<__m128i>(),
                _mm256_castsi256_si128(second),
            );
            _mm_storeu_si128(
                code_chunk.as_mut_ptr().add(32).cast::<__m128i>(),
                _mm256_extracti128_si256::<1>(first),
            );
            _mm_storeu_si128(
                code_chunk.as_mut_ptr().add(48).cast::<__m128i>(),
                _mm256_extracti128_si256::<1>(second),
            );
        }
    }
}
