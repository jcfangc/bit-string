use core::cmp::Ordering;

use crate::traits::WordOrd;

#[cfg(target_arch = "x86")]
use core::arch::x86::{__m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{__m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8};

#[target_feature(enable = "sse4.1")]
pub(super) unsafe fn cmp_aligned(src: &[u64], other: &[u64], len: usize) -> Option<Ordering> {
    let mut i = 0;
    while i + 2 <= len {
        let a = unsafe { _mm_loadu_si128(src.as_ptr().add(i).cast::<__m128i>()) };
        let b = unsafe { _mm_loadu_si128(other.as_ptr().add(i).cast::<__m128i>()) };
        let cmp = unsafe { _mm_cmpeq_epi64(a, b) };
        let mask = unsafe { _mm_movemask_epi8(cmp) } as u32;
        if mask != 0xFFFF {
            // Each lane is 8 bytes → 8 high bits.  The first zero byte
            // tells us which lane differs.
            let lane = mask.trailing_ones() as usize / 8;
            return Some(WordOrd::bitwise_cmp(src[i + lane], other[i + lane]));
        }
        i += 2;
    }
    while i < len {
        if src[i] != other[i] {
            return Some(WordOrd::bitwise_cmp(src[i], other[i]));
        }
        i += 1;
    }
    None
}
