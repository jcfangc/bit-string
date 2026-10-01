//! AVX2 kernels are retained for BITS=1, BITS=2, BITS=4, BITS=6, and BITS=8.
//! BITS=3, BITS=5, and BITS=7 remain scalar pending width-specific wins.

use super::scalar;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, __m256i, _mm_and_si128, _mm_cvtsi64_si128, _mm_loadu_si128, _mm_set1_epi8,
    _mm_srli_epi16, _mm_srli_si128, _mm_storeu_si128, _mm_unpackhi_epi8, _mm_unpackhi_epi16,
    _mm_unpacklo_epi8, _mm_unpacklo_epi16, _mm256_and_si256, _mm256_castsi256_si128,
    _mm256_cvtepu8_epi64, _mm256_extracti128_si256, _mm256_loadu_si256, _mm256_or_si256,
    _mm256_packus_epi16, _mm256_packus_epi32, _mm256_set1_epi8, _mm256_set1_epi32,
    _mm256_set1_epi64x, _mm256_setr_epi8, _mm256_shuffle_epi8, _mm256_slli_epi64,
    _mm256_srli_epi16, _mm256_srli_epi32, _mm256_storeu_si256, _mm256_unpackhi_epi8,
    _mm256_unpacklo_epi8,
};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, __m256i, _mm_and_si128, _mm_cvtsi64_si128, _mm_loadu_si128, _mm_set1_epi8,
    _mm_srli_epi16, _mm_srli_si128, _mm_storeu_si128, _mm_unpackhi_epi8, _mm_unpackhi_epi16,
    _mm_unpacklo_epi8, _mm_unpacklo_epi16, _mm256_and_si256, _mm256_castsi256_si128,
    _mm256_cvtepu8_epi64, _mm256_extracti128_si256, _mm256_loadu_si256, _mm256_or_si256,
    _mm256_packus_epi16, _mm256_packus_epi32, _mm256_set1_epi8, _mm256_set1_epi32,
    _mm256_set1_epi64x, _mm256_setr_epi8, _mm256_shuffle_epi8, _mm256_slli_epi64,
    _mm256_srli_epi16, _mm256_srli_epi32, _mm256_storeu_si256, _mm256_unpackhi_epi8,
    _mm256_unpacklo_epi8,
};

const CODES_PER_BLOCK: usize = 64;
const BITS_1_WORDS_PER_BLOCK: usize = 1;
const BITS_2_WORDS_PER_BLOCK: usize = 2;
const BITS_4_WORDS_PER_BLOCK: usize = 4;
const BITS_6_CODES_PER_BLOCK: usize = 32;
const BITS_6_WORDS_PER_BLOCK: usize = 3;
const BITS_8_WORDS_PER_BLOCK: usize = 8;

/// AVX2 backend entry for BITS=1, BITS=2, BITS=4, BITS=6, and BITS=8.
/// It handles its SIMD prefix and scalar tail.
///
/// # Safety
///
/// The caller must ensure AVX2 is available and that the slices satisfy the
/// complete-layout-block `WordsUnpack` contract for `BITS=1`, `BITS=2`, `BITS=4`, `BITS=6`, or `BITS=8`.
#[target_feature(enable = "avx2")]
pub(super) unsafe fn unpack_codes<const BITS: u8>(words: &[u64], codes: &mut [u8]) {
    if BITS == 6 {
        // BITS=6 has a 32-code AVX2 block, two codes per 64-code batch.
        let simd_code_len = codes.len() / BITS_6_CODES_PER_BLOCK * BITS_6_CODES_PER_BLOCK;
        let simd_word_len = simd_code_len / BITS_6_CODES_PER_BLOCK * BITS_6_WORDS_PER_BLOCK;
        for (word_block, code_block) in words[..simd_word_len]
            .chunks_exact(BITS_6_WORDS_PER_BLOCK)
            .zip(codes[..simd_code_len].chunks_exact_mut(BITS_6_CODES_PER_BLOCK))
        {
            // SAFETY: Each block has exactly three initialized words and 32
            // output bytes, as required by the BITS=6 kernel.
            unsafe { unpack_32_six_bit(word_block, code_block) };
        }

        scalar::unpack_codes::<BITS>(&words[simd_word_len..], &mut codes[simd_code_len..]);
        return;
    }

    debug_assert!(matches!(BITS, 1 | 2 | 4 | 8));
    let words_per_block = match BITS {
        1 => BITS_1_WORDS_PER_BLOCK,
        2 => BITS_2_WORDS_PER_BLOCK,
        4 => BITS_4_WORDS_PER_BLOCK,
        8 => BITS_8_WORDS_PER_BLOCK,
        _ => unreachable!(),
    };

    let simd_code_len = codes.len() / CODES_PER_BLOCK * CODES_PER_BLOCK;
    let simd_word_len = simd_code_len / CODES_PER_BLOCK * words_per_block;

    for (word_block, code_block) in words[..simd_word_len]
        .chunks_exact(words_per_block)
        .zip(codes[..simd_code_len].chunks_exact_mut(CODES_PER_BLOCK))
    {
        // SAFETY: Each paired block has the width-specific initialized input
        // words and 64 initialized output bytes required by its kernel.
        unsafe {
            match BITS {
                1 => unpack_64_one_bit(word_block, code_block),
                2 => unpack_64_two_bit(word_block, code_block),
                4 => unpack_64_four_bit(word_block, code_block),
                8 => unpack_64_eight_bit(word_block, code_block),
                _ => unreachable!(),
            }
        };
    }

    scalar::unpack_codes::<BITS>(&words[simd_word_len..], &mut codes[simd_code_len..]);
}

#[target_feature(enable = "avx2")]
unsafe fn unpack_32_six_bit(words: &[u64], codes: &mut [u8]) {
    debug_assert_eq!(words.len(), 3);
    debug_assert_eq!(codes.len(), 32);

    let mut groups = [0u32; 8];
    for (group_index, group) in groups.iter_mut().enumerate() {
        let bit_position = group_index * 24;
        let word_index = bit_position / 64;
        let bit_offset = bit_position % 64;
        let mut packed = words[word_index] >> bit_offset;
        if bit_offset + 24 > 64 {
            packed |= words[word_index + 1] << (64 - bit_offset);
        }
        *group = packed as u32 & 0x00ff_ffff;
    }

    // SAFETY: `groups` contains eight initialized 24-bit groups.
    let packed = unsafe { _mm256_loadu_si256(groups.as_ptr().cast::<__m256i>()) };
    let code_mask = _mm256_set1_epi32(0x3f);
    let first = _mm256_and_si256(packed, code_mask);
    let second = _mm256_and_si256(_mm256_srli_epi32::<6>(packed), code_mask);
    let third = _mm256_and_si256(_mm256_srli_epi32::<12>(packed), code_mask);
    let fourth = _mm256_and_si256(_mm256_srli_epi32::<18>(packed), code_mask);

    let first_pair = _mm256_packus_epi32(first, second);
    let second_pair = _mm256_packus_epi32(third, fourth);
    let column_major = _mm256_packus_epi16(first_pair, second_pair);
    let transpose = _mm256_setr_epi8(
        0, 4, 8, 12, 1, 5, 9, 13, 2, 6, 10, 14, 3, 7, 11, 15, //
        0, 4, 8, 12, 1, 5, 9, 13, 2, 6, 10, 14, 3, 7, 11, 15,
    );
    let output = _mm256_shuffle_epi8(column_major, transpose);

    // SAFETY: `output` contains exactly 32 codes and the output block has
    // exactly 32 initialized bytes.
    unsafe { _mm256_storeu_si256(codes.as_mut_ptr().cast::<__m256i>(), output) };
}

#[target_feature(enable = "avx2")]
unsafe fn unpack_64_one_bit(words: &[u64], codes: &mut [u8]) {
    debug_assert_eq!(words.len(), BITS_1_WORDS_PER_BLOCK);
    debug_assert_eq!(codes.len(), CODES_PER_BLOCK);

    let packed = _mm_cvtsi64_si128(words[0] as i64);
    let low = _mm256_cvtepu8_epi64(packed);
    let high = _mm256_cvtepu8_epi64(_mm_srli_si128::<4>(packed));
    let spread_mask = _mm256_set1_epi64x(0x0101_0101_0101_0101);
    let low = spread_one_bit_bytes(low, spread_mask);
    let high = spread_one_bit_bytes(high, spread_mask);

    // SAFETY: Each vector contains exactly 32 output bytes, and the output
    // slice contains exactly 64 initialized bytes.
    unsafe {
        _mm256_storeu_si256(codes.as_mut_ptr().cast::<__m256i>(), low);
        _mm256_storeu_si256(codes.as_mut_ptr().add(32).cast::<__m256i>(), high);
    }
}

#[target_feature(enable = "avx2")]
fn spread_one_bit_bytes(mut bits: __m256i, output_mask: __m256i) -> __m256i {
    bits = _mm256_and_si256(
        _mm256_or_si256(bits, _mm256_slli_epi64::<28>(bits)),
        _mm256_set1_epi64x(0x0000_000f_0000_000f),
    );
    bits = _mm256_and_si256(
        _mm256_or_si256(bits, _mm256_slli_epi64::<14>(bits)),
        _mm256_set1_epi64x(0x0003_0003_0003_0003),
    );
    _mm256_and_si256(
        _mm256_or_si256(bits, _mm256_slli_epi64::<7>(bits)),
        output_mask,
    )
}

#[target_feature(enable = "avx2")]
unsafe fn unpack_64_two_bit(words: &[u64], codes: &mut [u8]) {
    debug_assert_eq!(words.len(), BITS_2_WORDS_PER_BLOCK);
    debug_assert_eq!(codes.len(), CODES_PER_BLOCK);

    let code_mask = _mm_set1_epi8(0x03);
    // SAFETY: Two words contain exactly 16 initialized packed bytes.
    let packed = unsafe { _mm_loadu_si128(words.as_ptr().cast::<__m128i>()) };
    let first = _mm_and_si128(packed, code_mask);
    let second = _mm_and_si128(_mm_srli_epi16::<2>(packed), code_mask);
    let third = _mm_and_si128(_mm_srli_epi16::<4>(packed), code_mask);
    let fourth = _mm_and_si128(_mm_srli_epi16::<6>(packed), code_mask);

    let first_pair = _mm_unpacklo_epi8(first, second);
    let second_pair = _mm_unpacklo_epi8(third, fourth);
    let third_pair = _mm_unpackhi_epi8(first, second);
    let fourth_pair = _mm_unpackhi_epi8(third, fourth);

    // Interleave four 2-bit codes from each packed byte into sequential order.
    let output0 = _mm_unpacklo_epi16(first_pair, second_pair);
    let output1 = _mm_unpackhi_epi16(first_pair, second_pair);
    let output2 = _mm_unpacklo_epi16(third_pair, fourth_pair);
    let output3 = _mm_unpackhi_epi16(third_pair, fourth_pair);

    // SAFETY: Four stores write exactly 64 bytes to the output slice.
    unsafe {
        _mm_storeu_si128(codes.as_mut_ptr().cast::<__m128i>(), output0);
        _mm_storeu_si128(codes.as_mut_ptr().add(16).cast::<__m128i>(), output1);
        _mm_storeu_si128(codes.as_mut_ptr().add(32).cast::<__m128i>(), output2);
        _mm_storeu_si128(codes.as_mut_ptr().add(48).cast::<__m128i>(), output3);
    }
}

#[target_feature(enable = "avx2")]
unsafe fn unpack_64_four_bit(words: &[u64], codes: &mut [u8]) {
    debug_assert_eq!(words.len(), BITS_4_WORDS_PER_BLOCK);
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

#[target_feature(enable = "avx2")]
unsafe fn unpack_64_eight_bit(words: &[u64], codes: &mut [u8]) {
    debug_assert_eq!(words.len(), BITS_8_WORDS_PER_BLOCK);
    debug_assert_eq!(codes.len(), CODES_PER_BLOCK);

    // SAFETY: Eight words and 64 output bytes are initialized, so each load
    // and store accesses exactly 32 bytes within its slice.
    unsafe {
        let input = words.as_ptr().cast::<__m256i>();
        let output = codes.as_mut_ptr().cast::<__m256i>();
        let first = _mm256_loadu_si256(input);
        let second = _mm256_loadu_si256(input.add(1));
        _mm256_storeu_si256(output, first);
        _mm256_storeu_si256(output.add(1), second);
    }
}
