use super::scalar;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, __m256i, _mm_storeu_si128, _mm256_castsi256_si128, _mm256_loadu_si256,
    _mm256_movemask_epi8, _mm256_or_si256, _mm256_permute4x64_epi64, _mm256_setr_epi8,
    _mm256_shuffle_epi8, _mm256_slli_epi16,
};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, __m256i, _mm_storeu_si128, _mm256_castsi256_si128, _mm256_loadu_si256,
    _mm256_movemask_epi8, _mm256_or_si256, _mm256_permute4x64_epi64, _mm256_setr_epi8,
    _mm256_shuffle_epi8, _mm256_slli_epi16,
};

const CODES_PER_SIMD_BLOCK: usize = 32;
const WORDS_PER_SIMD_BLOCK: usize = 2;

/// Packs the BITS=1 or BITS=4 SIMD prefix and delegates complete remaining
/// blocks to scalar.
///
/// # Safety
///
/// The caller must only invoke this function when AVX2 is available. The
/// slices must satisfy the `WordsPack` contract, and `BITS` must be 1 or 4.
#[target_feature(enable = "avx2")]
pub(super) unsafe fn pack_codes<const BITS: u8>(dst: &mut [u64], codes: &[u8]) {
    debug_assert!(matches!(BITS, 1 | 4));
    let codes_per_word = 64 / usize::from(BITS);
    debug_assert_eq!(codes.len() % codes_per_word, 0);
    debug_assert_eq!(dst.len(), codes.len() / codes_per_word);

    let simd_code_block_len = if BITS == 1 { 64 } else { CODES_PER_SIMD_BLOCK };
    let simd_code_len = codes.len() / simd_code_block_len * simd_code_block_len;
    let simd_word_len = simd_code_len * usize::from(BITS) / 64;
    let simd_word_block_len = if BITS == 1 { 1 } else { WORDS_PER_SIMD_BLOCK };

    for (code_chunk, word_chunk) in codes[..simd_code_len]
        .chunks_exact(simd_code_block_len)
        .zip(dst[..simd_word_len].chunks_exact_mut(simd_word_block_len))
    {
        // SAFETY: The selected kernel reads only initialized bytes from this
        // chunk, and the backend is compiled with AVX2 enabled.
        unsafe {
            if BITS == 1 {
                pack_64_bits(code_chunk, word_chunk);
            } else {
                pack_32_codes(code_chunk, word_chunk);
            }
        }
    }

    if simd_code_len < codes.len() {
        scalar::pack_codes::<BITS>(&mut dst[simd_word_len..], &codes[simd_code_len..]);
    }
}

#[target_feature(enable = "avx2")]
unsafe fn pack_64_bits(codes: &[u8], words: &mut [u64]) {
    debug_assert_eq!(codes.len(), 64);
    debug_assert_eq!(words.len(), 1);

    // SAFETY: Both loads read exactly 32 initialized bytes.
    let (low, high) = unsafe {
        (
            _mm256_loadu_si256(codes.as_ptr().cast::<__m256i>()),
            _mm256_loadu_si256(codes.as_ptr().add(32).cast::<__m256i>()),
        )
    };
    let low = _mm256_movemask_epi8(_mm256_slli_epi16(low, 7)) as u32;
    let high = _mm256_movemask_epi8(_mm256_slli_epi16(high, 7)) as u32;
    words[0] = u64::from(low) | (u64::from(high) << 32);
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
