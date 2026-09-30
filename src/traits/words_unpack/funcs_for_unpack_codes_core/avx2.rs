use super::scalar;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, __m256i, _mm_storeu_si128, _mm256_and_si256, _mm256_castsi256_si128,
    _mm256_extracti128_si256, _mm256_loadu_si256, _mm256_set1_epi8, _mm256_srli_epi16,
    _mm256_unpackhi_epi8, _mm256_unpacklo_epi8,
};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, __m256i, _mm_storeu_si128, _mm256_and_si256, _mm256_castsi256_si128,
    _mm256_extracti128_si256, _mm256_loadu_si256, _mm256_set1_epi8, _mm256_srli_epi16,
    _mm256_unpackhi_epi8, _mm256_unpacklo_epi8,
};

const CODES_PER_BLOCK: usize = 64;
const WORDS_PER_BLOCK: usize = 4;

/// AVX2 backend entry for BITS=4. It handles its SIMD prefix and scalar tail.
///
/// # Safety
///
/// The caller must ensure AVX2 is available and that the slices satisfy the
/// complete-layout-block `WordsUnpack` contract for `BITS=4`.
#[target_feature(enable = "avx2")]
pub(super) unsafe fn unpack_codes<const BITS: u8>(words: &[u64], codes: &mut [u8]) {
    debug_assert_eq!(BITS, 4);

    let simd_code_len = codes.len() / CODES_PER_BLOCK * CODES_PER_BLOCK;
    let simd_word_len = simd_code_len / CODES_PER_BLOCK * WORDS_PER_BLOCK;

    for (word_block, code_block) in words[..simd_word_len]
        .chunks_exact(WORDS_PER_BLOCK)
        .zip(codes[..simd_code_len].chunks_exact_mut(CODES_PER_BLOCK))
    {
        // SAFETY: Each paired block has exactly four initialized words and
        // 64 initialized output bytes, as required by the kernel.
        unsafe { unpack_64_four_bit(word_block, code_block) };
    }

    scalar::unpack_codes::<BITS>(&words[simd_word_len..], &mut codes[simd_code_len..]);
}

#[target_feature(enable = "avx2")]
unsafe fn unpack_64_four_bit(words: &[u64], codes: &mut [u8]) {
    debug_assert_eq!(words.len(), WORDS_PER_BLOCK);
    debug_assert_eq!(codes.len(), CODES_PER_BLOCK);

    let nibble_mask = _mm256_set1_epi8(0x0f);
    // SAFETY: This block contains exactly 32 initialized bytes.
    let packed = unsafe { _mm256_loadu_si256(words.as_ptr().cast::<__m256i>()) };
    let low = _mm256_and_si256(packed, nibble_mask);
    let high = _mm256_and_si256(_mm256_srli_epi16(packed, 4), nibble_mask);
    let first = _mm256_unpacklo_epi8(low, high);
    let second = _mm256_unpackhi_epi8(low, high);

    // AVX2 byte unpack is lane-local. Store its four 128-bit halves in code order.
    unsafe {
        _mm_storeu_si128(
            codes.as_mut_ptr().cast::<__m128i>(),
            _mm256_castsi256_si128(first),
        );
        _mm_storeu_si128(
            codes.as_mut_ptr().add(16).cast::<__m128i>(),
            _mm256_castsi256_si128(second),
        );
        _mm_storeu_si128(
            codes.as_mut_ptr().add(32).cast::<__m128i>(),
            _mm256_extracti128_si256::<1>(first),
        );
        _mm_storeu_si128(
            codes.as_mut_ptr().add(48).cast::<__m128i>(),
            _mm256_extracti128_si256::<1>(second),
        );
    }
}
