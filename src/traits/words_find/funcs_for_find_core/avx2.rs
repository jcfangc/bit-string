// ---------------------------------------------------------------------------
// AVX2 — checks 4 consecutive shifts at once
// ---------------------------------------------------------------------------

use super::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m256i, _mm256_and_si256, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd,
    _mm256_set1_epi64x,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m256i, _mm256_and_si256, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd,
    _mm256_set1_epi64x,
};

#[target_feature(enable = "avx2")]
pub(super) unsafe fn find<F>(
    haystack: &[u64],
    needle_first: u64,
    needle_mask: u64,
    last_start: usize,
    verify: &mut F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    let needle = _mm256_set1_epi64x(needle_first as i64);
    let mask = _mm256_set1_epi64x(needle_mask as i64);

    for i in 0..haystack.len() {
        let base = i * WORD_BITS;
        if base > last_start {
            break;
        }
        let w0 = haystack[i];
        let w1 = haystack.get(i + 1).copied().unwrap_or(0);

        let mut s = 0;
        while s < WORD_BITS {
            if base + s > last_start {
                break;
            }
            let end = WORD_BITS.min(s + 4);
            let mut wins = [0u64; 4];
            for k in 0..(end - s) {
                let shift = s + k;
                wins[k] = if shift == 0 {
                    w0
                } else {
                    (w0 >> shift) | (w1 << (WORD_BITS - shift))
                };
            }
            let windows = unsafe { _mm256_loadu_si256(wins.as_ptr().cast::<__m256i>()) };
            let m = _mm256_and_si256(windows, mask);
            let c = _mm256_cmpeq_epi64(m, needle);
            let hits = unsafe { _mm256_movemask_pd(core::mem::transmute(c)) } as u32;
            if hits != 0 {
                for k in 0..(end - s) {
                    if hits & (1 << k) != 0 {
                        let pos = base + s + k;
                        if pos <= last_start && verify(pos) {
                            return Some(pos);
                        }
                    }
                }
            }

            s += 4;
        }
    }

    None
}
