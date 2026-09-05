// ---------------------------------------------------------------------------
// AVX2 — 4 consecutive shifts at once, reverse
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
pub(super) unsafe fn rfind<F>(
    haystack: &[u64],
    needle_key: u64,
    needle_mask: u64,
    last_start: usize,
    verify: &mut F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    let needle = _mm256_set1_epi64x(needle_key as i64);
    let mask = _mm256_set1_epi64x(needle_mask as i64);

    let start_word = (last_start / WORD_BITS).min(haystack.len().saturating_sub(1));
    for i in (0..=start_word).rev() {
        let base = i * WORD_BITS;
        let w0 = haystack[i];
        let w1 = haystack.get(i + 1).copied().unwrap_or(0);
        let max_shift = WORD_BITS.min(last_start - base + 1);

        // Round up to a multiple of 4 so the SIMD loop
        // processes shifts in 4-lane groups.  Out-of-range
        // positions are guarded by `pos <= last_start`.
        let mut s = max_shift.next_multiple_of(4).min(WORD_BITS);
        while s > 0 {
            s -= 4;

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

            // Check higher shifts first for rightmost.
            if hits != 0 {
                for k in (0..(end - s)).rev() {
                    if hits & (1 << k) != 0 {
                        let pos = base + s + k;
                        if pos <= last_start && verify(pos) {
                            return Some(pos);
                        }
                    }
                }
            }
        }
    }

    None
}
