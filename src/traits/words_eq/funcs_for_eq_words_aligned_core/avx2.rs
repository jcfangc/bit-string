#[cfg(target_arch = "x86")]
use core::arch::x86::{__m256i, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{__m256i, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd};

#[target_feature(enable = "avx2")]
pub(super) unsafe fn eq_words(src: &[u64], other: &[u64], len: usize) -> bool {
    let mut i = 0;
    while i + 4 <= len {
        // SAFETY: `#[target_feature(enable = "avx2")]` ensures AVX2 is enabled.
        // Pointers `src` and `other` are valid for `len` elements (guaranteed by caller).
        let a = unsafe { _mm256_loadu_si256(src.as_ptr().add(i).cast::<__m256i>()) };
        // SAFETY: same as above; load from `other`.
        let b = unsafe { _mm256_loadu_si256(other.as_ptr().add(i).cast::<__m256i>()) };
        // SAFETY: `_mm256_cmpeq_epi64` and `_mm256_movemask_pd` are pure register operations; AVX2 is enabled by `#[target_feature]`.
        let cmp = unsafe { _mm256_cmpeq_epi64(a, b) };
        // SAFETY: same as above
        if unsafe { _mm256_movemask_pd(core::mem::transmute(cmp)) } as u32 != 0b1111 {
            return false;
        }
        i += 4;
    }
    while i < len {
        if src[i] != other[i] {
            return false;
        }
        i += 1;
    }
    true
}
