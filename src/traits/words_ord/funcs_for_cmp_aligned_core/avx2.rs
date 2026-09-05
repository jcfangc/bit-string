use core::cmp::Ordering;

use crate::traits::WordOrd;

#[cfg(target_arch = "x86")]
use core::arch::x86::{__m256i, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{__m256i, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd};

#[target_feature(enable = "avx2")]
pub(super) unsafe fn cmp_aligned(src: &[u64], other: &[u64], len: usize) -> Option<Ordering> {
    let mut i = 0;
    while i + 4 <= len {
        let a = unsafe { _mm256_loadu_si256(src.as_ptr().add(i).cast::<__m256i>()) };
        let b = unsafe { _mm256_loadu_si256(other.as_ptr().add(i).cast::<__m256i>()) };
        let cmp = unsafe { _mm256_cmpeq_epi64(a, b) };
        let mask = unsafe { _mm256_movemask_pd(core::mem::transmute(cmp)) } as u32;
        if mask != 0b1111 {
            let lane = mask.trailing_ones() as usize;
            return Some(WordOrd::bitwise_cmp(src[i + lane], other[i + lane]));
        }
        i += 4;
    }
    while i < len {
        if src[i] != other[i] {
            return Some(WordOrd::bitwise_cmp(src[i], other[i]));
        }
        i += 1;
    }
    None
}
