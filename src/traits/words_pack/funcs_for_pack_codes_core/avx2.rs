use super::scalar;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, __m256i, _mm_storeu_si128, _mm256_castsi256_si128, _mm256_loadu_si256,
    _mm256_or_si256, _mm256_permute4x64_epi64, _mm256_setr_epi8, _mm256_shuffle_epi8,
    _mm256_slli_epi16,
};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, __m256i, _mm_storeu_si128, _mm256_castsi256_si128, _mm256_loadu_si256,
    _mm256_or_si256, _mm256_permute4x64_epi64, _mm256_setr_epi8, _mm256_shuffle_epi8,
    _mm256_slli_epi16,
};

const CODES_PER_SIMD_BLOCK: usize = 32;
const WORDS_PER_SIMD_BLOCK: usize = 2;

/// Packs the BITS=4 SIMD prefix and delegates complete remaining blocks to scalar.
///
/// # Safety
///
/// The caller must only invoke this function when AVX2 is available. The
/// slices must satisfy the `WordsPack` contract, and `BITS` must be 4.
#[target_feature(enable = "avx2")]
pub(super) unsafe fn pack_codes<const BITS: u8>(dst: &mut [u64], codes: &[u8]) {
    debug_assert_eq!(BITS, 4);
    debug_assert_eq!(codes.len() % 16, 0);
    debug_assert_eq!(dst.len(), codes.len() / 16);

    let simd_code_len = codes.len() / CODES_PER_SIMD_BLOCK * CODES_PER_SIMD_BLOCK;
    let simd_word_len = simd_code_len / 16;

    for (code_chunk, word_chunk) in codes[..simd_code_len]
        .chunks_exact(CODES_PER_SIMD_BLOCK)
        .zip(dst[..simd_word_len].chunks_exact_mut(WORDS_PER_SIMD_BLOCK))
    {
        // SAFETY: The chunk contains 32 initialized bytes, and the backend is
        // compiled with AVX2 enabled.
        unsafe { pack_32_codes(code_chunk, word_chunk) };
    }

    if simd_code_len < codes.len() {
        scalar::pack_codes::<BITS>(&mut dst[simd_word_len..], &codes[simd_code_len..]);
    }
}

#[target_feature(enable = "avx2")]
unsafe fn pack_32_codes(codes: &[u8], words: &mut [u64]) {
    debug_assert_eq!(codes.len(), CODES_PER_SIMD_BLOCK);
    debug_assert_eq!(words.len(), WORDS_PER_SIMD_BLOCK);

    let even_indices = _mm256_setr_epi8(
        0, 2, 4, 6, 8, 10, 12, 14, -1, -1, -1, -1, -1, -1, -1, -1, //
        0, 2, 4, 6, 8, 10, 12, 14, -1, -1, -1, -1, -1, -1, -1, -1,
    );
    let odd_indices = _mm256_setr_epi8(
        1, 3, 5, 7, 9, 11, 13, 15, -1, -1, -1, -1, -1, -1, -1, -1, //
        1, 3, 5, 7, 9, 11, 13, 15, -1, -1, -1, -1, -1, -1, -1, -1,
    );

    // SAFETY: `codes` contains exactly 32 initialized bytes.
    let input = unsafe { _mm256_loadu_si256(codes.as_ptr().cast::<__m256i>()) };
    let even = _mm256_shuffle_epi8(input, even_indices);
    let odd = _mm256_shuffle_epi8(input, odd_indices);
    let packed = _mm256_or_si256(even, _mm256_slli_epi16(odd, 4));
    let compacted = _mm256_permute4x64_epi64(packed, 0x88);

    // SAFETY: `compacted` contains two output words in its low 128 bits, and
    // `words` has space for exactly those two words.
    unsafe {
        _mm_storeu_si128(
            words.as_mut_ptr().cast::<__m128i>(),
            _mm256_castsi256_si128(compacted),
        );
    }
}
