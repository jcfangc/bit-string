use super::scalar;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_or_si256,
    _mm256_set1_epi8,
};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_or_si256,
    _mm256_set1_epi8,
};

const LANES: usize = 32;

/// AVX2 backend: validate + pack, 32 bytes / lane.
#[target_feature(enable = "avx2")]
pub(super) unsafe fn words(
    mut dst: *mut u64,
    mut src: *const u8,
    mut bit_len: usize,
) -> Option<(usize, u8)> {
    let ones = _mm256_set1_epi8(b'1' as i8);
    let zeros = _mm256_set1_epi8(b'0' as i8);
    let mut global_offset = 0usize;

    while bit_len >= 64 {
        // Load 2 × 32 bytes.
        // SAFETY: `bit_len >= 64`, two unaligned 32-byte loads are in bounds.
        let lo = unsafe { _mm256_loadu_si256(src.cast::<__m256i>()) };
        let hi = unsafe { _mm256_loadu_si256(src.add(LANES).cast::<__m256i>()) };

        // Validate: each byte must equal b'0' or b'1'.
        // cmpeq(v, b'1') → 0xFF for '1', 0x00 otherwise.
        // cmpeq(v, b'0') → 0xFF for '0', 0x00 otherwise.
        let lo_ones = _mm256_cmpeq_epi8(lo, ones);
        let lo_zeros = _mm256_cmpeq_epi8(lo, zeros);
        let hi_ones = _mm256_cmpeq_epi8(hi, ones);
        let hi_zeros = _mm256_cmpeq_epi8(hi, zeros);

        let lo_valid = _mm256_or_si256(lo_ones, lo_zeros);
        let hi_valid = _mm256_or_si256(hi_ones, hi_zeros);

        let lo_valid_mask = _mm256_movemask_epi8(lo_valid) as u32;
        let hi_valid_mask = _mm256_movemask_epi8(hi_valid) as u32;

        if lo_valid_mask != 0xFFFF_FFFF || hi_valid_mask != 0xFFFF_FFFF {
            // Fall back to scalar within this 64-byte chunk for exact
            // error position.
            // SAFETY: src points at this chunk, dst points at current slot.
            let (i, b) = unsafe { scalar::words(dst, src, 64) }.expect("chunk has invalid byte");
            return Some((global_offset + i, b));
        }

        // Pack: movemask extracts the MSB of cmpeq result.
        // b'1' → 0xFF (MSB=1), b'0' → 0x00 (MSB=0).
        let lo_bits = _mm256_movemask_epi8(lo_ones) as u32 as u64;
        let hi_bits = _mm256_movemask_epi8(hi_ones) as u32 as u64;

        // SAFETY: `dst` points to the next output slot.
        unsafe {
            *dst = lo_bits | (hi_bits << 32);
            dst = dst.add(1);
            src = src.add(64);
        }
        global_offset += 64;
        bit_len -= 64;
    }

    // Delegate tail to scalar.
    // SAFETY: `bit_len < 64`.
    unsafe {
        if let Some((i, b)) = scalar::words(dst, src, bit_len) {
            return Some((global_offset + i, b));
        }
    }
    None
}
