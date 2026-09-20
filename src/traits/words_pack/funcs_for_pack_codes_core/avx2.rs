//! AVX2 packing is intentionally implemented only for `BITS=1`, `BITS=3`,
//! `BITS=4`, `BITS=5`, and `BITS=6`.
//!
//! A `BITS=2` compression prototype and a `BITS=8` byte-copy prototype were
//! benchmarked against the scalar backend without a stable construction
//! speedup. Their extra dispatch and kernel complexity therefore was not
//! retained. Other widths continue to use scalar packing until a new kernel
//! demonstrates a measured benefit; re-run the construction benchmarks before
//! revisiting this support matrix.

use super::scalar;
use crate::traits::words_pack::layout_block_len;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, __m256i, _mm_storeu_si128, _mm256_castsi256_si128, _mm256_cvtepu32_epi64,
    _mm256_loadu_si256, _mm256_madd_epi16, _mm256_maddubs_epi16, _mm256_movemask_epi8,
    _mm256_or_si256, _mm256_permute4x64_epi64, _mm256_permutevar8x32_epi32, _mm256_setr_epi8,
    _mm256_setr_epi16, _mm256_setr_epi32, _mm256_shuffle_epi8, _mm256_slli_epi16,
    _mm256_slli_epi64, _mm256_storeu_si256,
};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, __m256i, _mm_storeu_si128, _mm256_castsi256_si128, _mm256_cvtepu32_epi64,
    _mm256_loadu_si256, _mm256_madd_epi16, _mm256_maddubs_epi16, _mm256_movemask_epi8,
    _mm256_or_si256, _mm256_permute4x64_epi64, _mm256_permutevar8x32_epi32, _mm256_setr_epi8,
    _mm256_setr_epi16, _mm256_setr_epi32, _mm256_shuffle_epi8, _mm256_slli_epi16,
    _mm256_slli_epi64, _mm256_storeu_si256,
};

const CODES_PER_SIMD_BLOCK: usize = 32;
const WORDS_PER_SIMD_BLOCK: usize = 2;

/// Packs the BITS=1, BITS=3, BITS=4, BITS=5, or BITS=6 SIMD prefix and
/// delegates complete remaining blocks to scalar.
///
/// # Safety
///
/// The caller must only invoke this function when AVX2 is available. The
/// slices must satisfy the `WordsPack` contract, and `BITS` must be 1, 3, 4, 5, or 6.
#[target_feature(enable = "avx2")]
pub(super) unsafe fn pack_codes<const BITS: u8>(dst: &mut [u64], codes: &[u8]) {
    debug_assert!(matches!(BITS, 1 | 3 | 4 | 5 | 6));
    debug_assert_eq!(codes.len() % layout_block_len::<BITS>(), 0);
    debug_assert_eq!(dst.len(), codes.len() * usize::from(BITS) / 64);

    let simd_code_block_len = if matches!(BITS, 1 | 3 | 5) {
        64
    } else {
        CODES_PER_SIMD_BLOCK
    };
    let simd_code_len = codes.len() / simd_code_block_len * simd_code_block_len;
    let simd_word_len = simd_code_len * usize::from(BITS) / 64;
    let simd_word_block_len = match BITS {
        1 => 1,
        3 => 3,
        4 => WORDS_PER_SIMD_BLOCK,
        5 => 5,
        6 => 3,
        _ => unreachable!(),
    };

    for (code_chunk, word_chunk) in codes[..simd_code_len]
        .chunks_exact(simd_code_block_len)
        .zip(dst[..simd_word_len].chunks_exact_mut(simd_word_block_len))
    {
        // SAFETY: The selected kernel reads only initialized bytes from this
        // chunk, and the backend is compiled with AVX2 enabled.
        unsafe {
            match BITS {
                1 => pack_64_bits(code_chunk, word_chunk),
                3 => pack_64_three_bits(code_chunk, word_chunk),
                4 => pack_32_codes(code_chunk, word_chunk),
                5 => pack_64_five_bits(code_chunk, word_chunk),
                6 => pack_32_six_bits(code_chunk, word_chunk),
                _ => unreachable!(),
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

#[target_feature(enable = "avx2")]
unsafe fn pack_64_three_bits(codes: &[u8], words: &mut [u64]) {
    debug_assert_eq!(codes.len(), 64);
    debug_assert_eq!(words.len(), 3);

    let mut groups = [0u64; 8];
    // SAFETY: Each half contains exactly 32 initialized codes and has space
    // for the four 24-bit groups produced by the kernel.
    unsafe {
        pack_32_three_bit_groups(&codes[..32], &mut groups[..4]);
        pack_32_three_bit_groups(&codes[32..], &mut groups[4..]);
    }

    let mut accumulator = 0u64;
    let mut accumulated_bits = 0;
    let mut word_index = 0;
    for group in groups {
        let next_bits = accumulated_bits + 24;
        if next_bits < 64 {
            accumulator |= group << accumulated_bits;
            accumulated_bits = next_bits;
        } else if next_bits == 64 {
            words[word_index] = accumulator | (group << accumulated_bits);
            word_index += 1;
            accumulator = 0;
            accumulated_bits = 0;
        } else {
            words[word_index] = accumulator | (group << accumulated_bits);
            word_index += 1;
            accumulator = group >> (64 - accumulated_bits);
            accumulated_bits = next_bits - 64;
        }
    }

    debug_assert_eq!(accumulated_bits, 0);
    debug_assert_eq!(word_index, 3);
}

#[target_feature(enable = "avx2")]
unsafe fn pack_32_three_bit_groups(codes: &[u8], groups: &mut [u64]) {
    debug_assert_eq!(codes.len(), CODES_PER_SIMD_BLOCK);
    debug_assert_eq!(groups.len(), 4);

    let pair_weights = _mm256_setr_epi8(
        1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8, //
        1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8,
    );
    let group_weights = _mm256_setr_epi16(
        1, 64, 1, 64, 1, 64, 1, 64, //
        1, 64, 1, 64, 1, 64, 1, 64,
    );

    // SAFETY: `codes` contains exactly 32 initialized bytes.
    let input = unsafe { _mm256_loadu_si256(codes.as_ptr().cast::<__m256i>()) };
    let pairs = _mm256_maddubs_epi16(input, pair_weights);
    let values = _mm256_madd_epi16(pairs, group_weights);

    let even_indices = _mm256_setr_epi32(0, 2, 4, 6, 0, 2, 4, 6);
    let odd_indices = _mm256_setr_epi32(1, 3, 5, 7, 1, 3, 5, 7);
    let even = _mm256_cvtepu32_epi64(_mm256_castsi256_si128(_mm256_permutevar8x32_epi32(
        values,
        even_indices,
    )));
    let odd = _mm256_cvtepu32_epi64(_mm256_castsi256_si128(_mm256_permutevar8x32_epi32(
        values,
        odd_indices,
    )));
    let packed = _mm256_or_si256(even, _mm256_slli_epi64(odd, 12));

    // SAFETY: `groups` has space for exactly four u64 values.
    unsafe { _mm256_storeu_si256(groups.as_mut_ptr().cast::<__m256i>(), packed) };
}

#[target_feature(enable = "avx2")]
unsafe fn pack_64_five_bits(codes: &[u8], words: &mut [u64]) {
    debug_assert_eq!(codes.len(), 64);
    debug_assert_eq!(words.len(), 5);

    let mut groups = [0u64; 8];
    // SAFETY: Each half contains exactly 32 initialized codes and has space
    // for the four 40-bit groups produced by the kernel.
    unsafe {
        pack_32_five_bit_groups(&codes[..32], &mut groups[..4]);
        pack_32_five_bit_groups(&codes[32..], &mut groups[4..]);
    }

    let mut accumulator = 0u64;
    let mut accumulated_bits = 0;
    let mut word_index = 0;
    for group in groups {
        let next_bits = accumulated_bits + 40;
        if next_bits < 64 {
            accumulator |= group << accumulated_bits;
            accumulated_bits = next_bits;
        } else if next_bits == 64 {
            words[word_index] = accumulator | (group << accumulated_bits);
            word_index += 1;
            accumulator = 0;
            accumulated_bits = 0;
        } else {
            words[word_index] = accumulator | (group << accumulated_bits);
            word_index += 1;
            accumulator = group >> (64 - accumulated_bits);
            accumulated_bits = next_bits - 64;
        }
    }

    debug_assert_eq!(accumulated_bits, 0);
    debug_assert_eq!(word_index, 5);
}

#[target_feature(enable = "avx2")]
unsafe fn pack_32_five_bit_groups(codes: &[u8], groups: &mut [u64]) {
    debug_assert_eq!(codes.len(), CODES_PER_SIMD_BLOCK);
    debug_assert_eq!(groups.len(), 4);

    let pair_weights = _mm256_setr_epi8(
        1, 32, 1, 32, 1, 32, 1, 32, 1, 32, 1, 32, 1, 32, 1, 32, //
        1, 32, 1, 32, 1, 32, 1, 32, 1, 32, 1, 32, 1, 32, 1, 32,
    );
    let group_weights = _mm256_setr_epi16(
        1, 1024, 1, 1024, 1, 1024, 1, 1024, //
        1, 1024, 1, 1024, 1, 1024, 1, 1024,
    );

    // SAFETY: `codes` contains exactly 32 initialized bytes.
    let input = unsafe { _mm256_loadu_si256(codes.as_ptr().cast::<__m256i>()) };
    let pairs = _mm256_maddubs_epi16(input, pair_weights);
    let values = _mm256_madd_epi16(pairs, group_weights);

    let even_indices = _mm256_setr_epi32(0, 2, 4, 6, 0, 2, 4, 6);
    let odd_indices = _mm256_setr_epi32(1, 3, 5, 7, 1, 3, 5, 7);
    let even = _mm256_cvtepu32_epi64(_mm256_castsi256_si128(_mm256_permutevar8x32_epi32(
        values,
        even_indices,
    )));
    let odd = _mm256_cvtepu32_epi64(_mm256_castsi256_si128(_mm256_permutevar8x32_epi32(
        values,
        odd_indices,
    )));
    let packed = _mm256_or_si256(even, _mm256_slli_epi64(odd, 20));

    // SAFETY: `groups` has space for exactly four u64 values.
    unsafe { _mm256_storeu_si256(groups.as_mut_ptr().cast::<__m256i>(), packed) };
}

#[target_feature(enable = "avx2")]
unsafe fn pack_32_six_bits(codes: &[u8], words: &mut [u64]) {
    debug_assert_eq!(codes.len(), CODES_PER_SIMD_BLOCK);
    debug_assert_eq!(words.len(), 3);

    // First combine adjacent codes into 12-bit pairs, then combine adjacent
    // pairs into 24-bit values. Two values make one complete 48-bit group.
    let pair_weights = _mm256_setr_epi8(
        1, 64, 1, 64, 1, 64, 1, 64, 1, 64, 1, 64, 1, 64, 1, 64, //
        1, 64, 1, 64, 1, 64, 1, 64, 1, 64, 1, 64, 1, 64, 1, 64,
    );
    let group_weights = _mm256_setr_epi16(
        1, 4096, 1, 4096, 1, 4096, 1, 4096, //
        1, 4096, 1, 4096, 1, 4096, 1, 4096,
    );

    // SAFETY: `codes` contains exactly 32 initialized bytes.
    let input = unsafe { _mm256_loadu_si256(codes.as_ptr().cast::<__m256i>()) };
    let pairs = _mm256_maddubs_epi16(input, pair_weights);
    let values = _mm256_madd_epi16(pairs, group_weights);

    let even_indices = _mm256_setr_epi32(0, 2, 4, 6, 0, 2, 4, 6);
    let odd_indices = _mm256_setr_epi32(1, 3, 5, 7, 1, 3, 5, 7);
    let even = _mm256_cvtepu32_epi64(_mm256_castsi256_si128(_mm256_permutevar8x32_epi32(
        values,
        even_indices,
    )));
    let odd = _mm256_cvtepu32_epi64(_mm256_castsi256_si128(_mm256_permutevar8x32_epi32(
        values,
        odd_indices,
    )));
    let groups = _mm256_or_si256(even, _mm256_slli_epi64(odd, 24));

    let mut packed_groups = [0u64; 4];
    // SAFETY: `packed_groups` has space for exactly four u64 values.
    unsafe { _mm256_storeu_si256(packed_groups.as_mut_ptr().cast::<__m256i>(), groups) };

    words[0] = packed_groups[0] | (packed_groups[1] << 48);
    words[1] = (packed_groups[1] >> 16) | (packed_groups[2] << 32);
    words[2] = (packed_groups[2] >> 32) | (packed_groups[3] << 16);
}
