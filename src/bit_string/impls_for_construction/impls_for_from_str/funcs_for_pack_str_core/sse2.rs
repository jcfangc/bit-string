use super::scalar;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128, _mm_set1_epi8,
};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128, _mm_set1_epi8,
};

const LANES: usize = 16;

/// SSE2 backend: validate + pack, 16 bytes / lane.
#[target_feature(enable = "sse2")]
pub(super) unsafe fn words(
    mut dst: *mut u64,
    mut src: *const u8,
    mut bit_len: usize,
) -> Option<(usize, u8)> {
    let ones = _mm_set1_epi8(b'1' as i8);
    let zeros = _mm_set1_epi8(b'0' as i8);
    let mut global_offset = 0usize;

    while bit_len >= 64 {
        // Load 4 × 16 bytes.
        // SAFETY: `bit_len >= 64`, four unaligned 16-byte loads are in bounds.
        let v0 = unsafe { _mm_loadu_si128(src.cast::<__m128i>()) };
        let v1 = unsafe { _mm_loadu_si128(src.add(LANES).cast::<__m128i>()) };
        let v2 = unsafe { _mm_loadu_si128(src.add(LANES * 2).cast::<__m128i>()) };
        let v3 = unsafe { _mm_loadu_si128(src.add(LANES * 3).cast::<__m128i>()) };

        let ones0 = _mm_cmpeq_epi8(v0, ones);
        let zeros0 = _mm_cmpeq_epi8(v0, zeros);
        let ones1 = _mm_cmpeq_epi8(v1, ones);
        let zeros1 = _mm_cmpeq_epi8(v1, zeros);
        let ones2 = _mm_cmpeq_epi8(v2, ones);
        let zeros2 = _mm_cmpeq_epi8(v2, zeros);
        let ones3 = _mm_cmpeq_epi8(v3, ones);
        let zeros3 = _mm_cmpeq_epi8(v3, zeros);

        let valid0 = _mm_or_si128(ones0, zeros0);
        let valid1 = _mm_or_si128(ones1, zeros1);
        let valid2 = _mm_or_si128(ones2, zeros2);
        let valid3 = _mm_or_si128(ones3, zeros3);

        let mask0 = _mm_movemask_epi8(valid0) as u16;
        let mask1 = _mm_movemask_epi8(valid1) as u16;
        let mask2 = _mm_movemask_epi8(valid2) as u16;
        let mask3 = _mm_movemask_epi8(valid3) as u16;

        if mask0 != 0xFFFF || mask1 != 0xFFFF || mask2 != 0xFFFF || mask3 != 0xFFFF {
            // SAFETY: caller guarantees pointer validity. Scalar backend is always safe.
            let (i, b) = unsafe { scalar::words(dst, src, 64) }.expect("chunk has invalid byte");
            return Some((global_offset + i, b));
        }

        let b0 = _mm_movemask_epi8(ones0) as u32 as u64;
        let b1 = _mm_movemask_epi8(ones1) as u32 as u64;
        let b2 = _mm_movemask_epi8(ones2) as u32 as u64;
        let b3 = _mm_movemask_epi8(ones3) as u32 as u64;

        // SAFETY: `dst` points to the next output slot.
        unsafe {
            *dst = b0 | (b1 << 16) | (b2 << 32) | (b3 << 48);
            dst = dst.add(1);
            src = src.add(64);
        }
        global_offset += 64;
        bit_len -= 64;
    }

    // SAFETY: `bit_len < 64`.
    unsafe {
        if let Some((i, b)) = scalar::words(dst, src, bit_len) {
            return Some((global_offset + i, b));
        }
    }
    None
}
