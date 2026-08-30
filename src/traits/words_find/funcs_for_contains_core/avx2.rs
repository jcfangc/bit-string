use super::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, __m256i, _mm_set1_epi64x, _mm256_and_si256, _mm256_cmpeq_epi64, _mm256_loadu_si256,
    _mm256_movemask_pd, _mm256_or_si256, _mm256_set1_epi64x, _mm256_sll_epi64, _mm256_srl_epi64,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, __m256i, _mm_set1_epi64x, _mm256_and_si256, _mm256_cmpeq_epi64, _mm256_loadu_si256,
    _mm256_movemask_pd, _mm256_or_si256, _mm256_set1_epi64x, _mm256_sll_epi64, _mm256_srl_epi64,
};

const LANES: usize = 4;

/// AVX2 backend: same as SSE2 but with 4-lane (256-bit) vectors.
#[target_feature(enable = "avx2")]
pub(super) unsafe fn find_any<F>(
    haystack: &[u64],
    needle_first: u64,
    needle_mask: u64,
    last_start: usize,
    word_limit: usize,
    verify: &mut F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    let needle = unsafe { _mm256_set1_epi64x(needle_first as i64) };
    let mask = unsafe { _mm256_set1_epi64x(needle_mask as i64) };

    for shift in 0..WORD_BITS {
        let mut i = 0;
        while i + LANES <= word_limit {
            if i * WORD_BITS + shift > last_start {
                break;
            }

            let window = if shift == 0 {
                unsafe { _mm256_loadu_si256(haystack.as_ptr().add(i).cast::<__m256i>()) }
            } else {
                let src = haystack.as_ptr();
                let w0 = unsafe { _mm256_loadu_si256(src.add(i).cast::<__m256i>()) };
                let w1 = unsafe { _mm256_loadu_si256(src.add(i + 1).cast::<__m256i>()) };
                let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
                let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
                let lo = unsafe { _mm256_srl_epi64(w0, count_lo) };
                let hi = unsafe { _mm256_sll_epi64(w1, count_hi) };
                unsafe { _mm256_or_si256(lo, hi) }
            };

            let masked = unsafe { _mm256_and_si256(window, mask) };
            let cmp = unsafe { _mm256_cmpeq_epi64(masked, needle) };
            let hits = unsafe { _mm256_movemask_pd(core::mem::transmute(cmp)) } as u32;

            if hits != 0 {
                for k in 0..LANES {
                    if hits & (1 << k) != 0 {
                        let pos = (i + k) * WORD_BITS + shift;
                        if pos <= last_start && verify(pos) {
                            return Some(pos);
                        }
                    }
                }
            }

            i += LANES;
        }

        for j in i..word_limit {
            let pos = j * WORD_BITS + shift;
            if pos > last_start {
                break;
            }
            let window = if shift == 0 {
                haystack[j]
            } else {
                let w0 = haystack[j];
                let w1 = haystack.get(j + 1).copied().unwrap_or(0);
                (w0 >> shift) | (w1 << (WORD_BITS - shift))
            };
            if (window & needle_mask) == needle_first && verify(pos) {
                return Some(pos);
            }
        }
    }

    None
}
