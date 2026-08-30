#[cfg(target_arch = "x86")]
use core::arch::x86::{__m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{__m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8};

#[target_feature(enable = "sse4.1")]
pub(super) unsafe fn eq_words(src: &[u64], other: &[u64], len: usize) -> bool {
    let mut i = 0;
    while i + 2 <= len {
        // SAFETY: `#[target_feature(enable = "sse4.1")]` ensures SSE4.1 is enabled.
        // Pointers `src` and `other` are valid for `len` elements (guaranteed by caller).
        let a = unsafe { _mm_loadu_si128(src.as_ptr().add(i).cast::<__m128i>()) };
        // SAFETY: same as above; load from `other`.
        let b = unsafe { _mm_loadu_si128(other.as_ptr().add(i).cast::<__m128i>()) };
        // SAFETY: `_mm_cmpeq_epi64` and `_mm_movemask_epi8` are pure register operations; SSE4.1 is enabled by `#[target_feature]`.
        let cmp = unsafe { _mm_cmpeq_epi64(a, b) };
        // SAFETY: same as above
        if unsafe { _mm_movemask_epi8(cmp) } as u32 != 0xFFFF {
            return false;
        }
        i += 2;
    }
    while i < len {
        if src[i] != other[i] {
            return false;
        }
        i += 1;
    }
    true
}
