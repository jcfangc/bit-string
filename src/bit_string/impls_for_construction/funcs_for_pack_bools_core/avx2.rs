use super::scalar;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_set1_epi8,
};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_set1_epi8,
};

const LANES: usize = 32;

/// AVX2 backend: 32 bytes → 32 movemask bits, 2 iterations per u64.
///
/// # Safety
///
/// - Caller must only call this when AVX2 is available.
/// - `src` must be valid for reads of `bit_len` u8 values.
/// - `dst` must be valid for writes of `ceil(bit_len / 64)` u64 values.
#[target_feature(enable = "avx2")]
pub(super) unsafe fn words(mut dst: *mut u64, mut src: *const u8, mut bit_len: usize) {
    // SAFETY: this function is only callable when AVX2 is available
    // (enforced by the caller / dispatch gating).
    let ones = _mm256_set1_epi8(1);

    while bit_len >= 64 {
        // SAFETY:
        // - `bit_len >= 64`, so two 32-byte reads from `src` are in bounds.
        // - `_mm256_loadu_si256` permits unaligned loads.
        let lo = unsafe { _mm256_loadu_si256(src.cast::<__m256i>()) };
        // SAFETY: `src + 32` is valid; `bit_len >= 64`.
        let hi = unsafe { _mm256_loadu_si256(src.add(LANES).cast::<__m256i>()) };

        // cmpeq extracts LSB: 0x01 → 0xFF, 0x00 → 0x00
        let lo_eq = _mm256_cmpeq_epi8(lo, ones);
        let hi_eq = _mm256_cmpeq_epi8(hi, ones);

        // movemask takes the MSB of each byte → the comparison result.
        let lo_bits = _mm256_movemask_epi8(lo_eq) as u32 as u64;
        let hi_bits = _mm256_movemask_epi8(hi_eq) as u32 as u64;

        // SAFETY: `dst` points to the next output slot.
        unsafe {
            *dst = lo_bits | (hi_bits << 32);
        }

        // SAFETY: destination has capacity; source has `bit_len >= 64`.
        unsafe {
            dst = dst.add(1);
            src = src.add(64);
        }
        bit_len -= 64;
    }

    // SAFETY: `bit_len < 64`, delegate tail to scalar.
    unsafe {
        scalar::words(dst, src, bit_len);
    }
}
