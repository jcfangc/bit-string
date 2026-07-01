use alloc::vec::Vec;

pub(super) const OP_AND: u8 = 0;
pub(super) const OP_OR: u8 = 1;
pub(super) const OP_XOR: u8 = 2;

#[inline]
pub(super) fn owned<const OP: u8>(lhs: &[u64], rhs: &[u64]) -> Vec<u64> {
    debug_assert_eq!(lhs.len(), rhs.len());

    let len = lhs.len();
    let mut out = Vec::<u64>::with_capacity(len);

    // SAFETY:
    // - `out` has capacity for exactly `len` u64 values.
    // - `lhs` and `rhs` are valid for reads of `len` u64 values.
    // - `out.as_mut_ptr()` is valid for writes of `len` u64 values.
    // - `out` is freshly allocated, so it cannot overlap `lhs` or `rhs`.
    // - `dispatch` writes every slot in `0..len` exactly once.
    unsafe {
        dispatch::<OP>(out.as_mut_ptr(), lhs.as_ptr(), rhs.as_ptr(), len);
        out.set_len(len);
    }

    out
}

#[inline]
pub(super) fn assign<const OP: u8>(lhs: &mut [u64], rhs: &[u64]) {
    debug_assert_eq!(lhs.len(), rhs.len());

    let len = lhs.len();
    let lhs_ptr = lhs.as_mut_ptr();

    // SAFETY:
    // - `lhs_ptr` is valid for reads and writes of `len` u64 values.
    // - `rhs` is valid for reads of `len` u64 values.
    // - `dst == lhs` is explicitly allowed by `dispatch`.
    // - Safe Rust prevents `rhs` from aliasing `lhs` in normal calls.
    // - `dispatch` writes every slot in `0..len` exactly once.
    unsafe {
        dispatch::<OP>(lhs_ptr, lhs_ptr.cast_const(), rhs.as_ptr(), len);
    }
}

/// Writes `lhs[i] OP rhs[i]` into `dst[i]` for every `i in 0..len`.
///
/// `dst` may be exactly equal to `lhs`, which enables in-place assignment.
/// Partial overlaps are not allowed.
///
/// # Safety
///
/// - `dst` must be valid for writes of `len` initialized `u64` values.
/// - `lhs` and `rhs` must be valid for reads of `len` initialized `u64` values.
/// - `dst` must either not overlap `lhs`, or be exactly equal to `lhs`.
/// - `dst` must not overlap `rhs`.
#[inline]
unsafe fn dispatch<const OP: u8>(dst: *mut u64, lhs: *const u64, rhs: *const u64, len: usize) {
    // ── Default: runtime AVX2 detection ──────────────────────────
    #[cfg(not(feature = "compile-time-dispatch"))]
    {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            // SAFETY: CPUID leaf 7, subleaf 0: EBX bit 5 = AVX2.
            let has_avx2 = {
                #[cfg(target_arch = "x86_64")]
                {
                    unsafe { core::arch::x86_64::__cpuid_count(7, 0).ebx & (1 << 5) != 0 }
                }
                #[cfg(target_arch = "x86")]
                {
                    unsafe { core::arch::x86::__cpuid_count(7, 0).ebx & (1 << 5) != 0 }
                }
            };
            if has_avx2 {
                // SAFETY: runtime CPUID confirmed AVX2 is available.
                unsafe { avx2::words::<OP>(dst, lhs, rhs, len) };
                return;
            }
        }
        // Fallback: SSE2 (x86_64 baseline), NEON (aarch64), scalar.
        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            any(target_feature = "sse2", target_feature = "avx2")
        ))]
        {
            // SAFETY: SSE2 is baseline on x86-64.
            unsafe { sse2::words::<OP>(dst, lhs, rhs, len) };
            return;
        }
        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            // SAFETY: NEON is available per `#[target_feature]` gating.
            unsafe { neon::words::<OP>(dst, lhs, rhs, len) };
            return;
        }
        #[allow(unused)]
        // SAFETY: Forwarded from `dispatch`'s safety contract.
        unsafe {
            scalar::words::<OP>(dst, lhs, rhs, len);
        }
    }

    // ── compile-time-dispatch: existing #[cfg] cascade ───────────
    #[cfg(feature = "compile-time-dispatch")]
    {
        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        ))]
        {
            // SAFETY:
            // - Forwarded from `dispatch`'s safety contract.
            // - This branch is compiled only when AVX2 is enabled.
            unsafe { avx2::words::<OP>(dst, lhs, rhs, len) };
            return;
        }

        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse2",
            not(target_feature = "avx2")
        ))]
        {
            // SAFETY:
            // - Forwarded from `dispatch`'s safety contract.
            // - This branch is compiled only when SSE2 is enabled.
            unsafe { sse2::words::<OP>(dst, lhs, rhs, len) };
            return;
        }

        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            // SAFETY:
            // - Forwarded from `dispatch`'s safety contract.
            // - This branch is compiled only when NEON is enabled.
            unsafe { neon::words::<OP>(dst, lhs, rhs, len) };
            return;
        }

        #[allow(unused)]
        // SAFETY: Forwarded from `dispatch`'s safety contract.
        unsafe {
            scalar::words::<OP>(dst, lhs, rhs, len);
        }
    }
}

#[allow(unused)]
mod scalar {
    use super::*;

    #[inline]
    fn apply<const OP: u8>(lhs: u64, rhs: u64) -> u64 {
        match OP {
            OP_AND => lhs & rhs,
            OP_OR => lhs | rhs,
            OP_XOR => lhs ^ rhs,
            _ => unreachable!("unsupported binary bit operation"),
        }
    }

    /// Scalar backend for `dst[i] = lhs[i] OP rhs[i]`.
    ///
    /// Supports `dst == lhs`.
    ///
    /// # Safety
    ///
    /// - `dst` must be valid for writes of `len` initialized `u64` values.
    /// - `lhs` and `rhs` must be valid for reads of `len` initialized `u64` values.
    /// - `dst` must either not overlap `lhs`, or be exactly equal to `lhs`.
    /// - `dst` must not overlap `rhs`.
    #[inline]
    pub(super) unsafe fn words<const OP: u8>(
        dst: *mut u64,
        lhs: *const u64,
        rhs: *const u64,
        len: usize,
    ) {
        for i in 0..len {
            // SAFETY:
            // - `i < len`.
            // - Pointer validity and overlap constraints are guaranteed by the caller.
            // - `dst == lhs` is safe because both operands are read before writing `dst[i]`.
            unsafe {
                let lhs_word = lhs.add(i).read();
                let rhs_word = rhs.add(i).read();
                dst.add(i).write(apply::<OP>(lhs_word, rhs_word));
            }
        }
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2 {
    use super::*;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, _mm_and_si128, _mm_loadu_si128, _mm_or_si128, _mm_storeu_si128, _mm_xor_si128,
    };

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, _mm_and_si128, _mm_loadu_si128, _mm_or_si128, _mm_storeu_si128, _mm_xor_si128,
    };

    const LANES: usize = 2;

    #[inline]
    fn apply<const OP: u8>(lhs: __m128i, rhs: __m128i) -> __m128i {
        match OP {
            OP_AND => unsafe { _mm_and_si128(lhs, rhs) },
            OP_OR => unsafe { _mm_or_si128(lhs, rhs) },
            OP_XOR => unsafe { _mm_xor_si128(lhs, rhs) },
            _ => unreachable!("unsupported binary bit operation"),
        }
    }

    /// SSE2 backend for `dst[i] = lhs[i] OP rhs[i]`.
    ///
    /// Supports `dst == lhs`.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when SSE2 is available.
    /// - `dst` must be valid for writes of `len` initialized `u64` values.
    /// - `lhs` and `rhs` must be valid for reads of `len` initialized `u64` values.
    /// - `dst` must either not overlap `lhs`, or be exactly equal to `lhs`.
    /// - `dst` must not overlap `rhs`.
    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn words<const OP: u8>(
        dst: *mut u64,
        lhs: *const u64,
        rhs: *const u64,
        len: usize,
    ) {
        let chunks = len / LANES;

        for chunk in 0..chunks {
            let offset = chunk * LANES;

            // SAFETY:
            // - `offset + LANES <= len`.
            // - Unaligned load/store intrinsics permit unaligned access.
            // - Pointer validity and overlap constraints are guaranteed by the caller.
            // - `dst == lhs` is safe because loads happen before the store.
            unsafe {
                let lhs_vec = _mm_loadu_si128(lhs.add(offset).cast::<__m128i>());
                let rhs_vec = _mm_loadu_si128(rhs.add(offset).cast::<__m128i>());
                let out_vec = apply::<OP>(lhs_vec, rhs_vec);

                _mm_storeu_si128(dst.add(offset).cast::<__m128i>(), out_vec);
            }
        }

        let done = chunks * LANES;

        // SAFETY:
        // - `done <= len`.
        // - Tail range is `done..len`.
        // - Pointer validity and overlap constraints are guaranteed by the caller.
        unsafe {
            scalar::words::<OP>(dst.add(done), lhs.add(done), rhs.add(done), len - done);
        }
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use super::*;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m256i, _mm256_and_si256, _mm256_loadu_si256, _mm256_or_si256, _mm256_storeu_si256,
        _mm256_xor_si256,
    };

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_and_si256, _mm256_loadu_si256, _mm256_or_si256, _mm256_storeu_si256,
        _mm256_xor_si256,
    };

    const LANES: usize = 4;

    #[inline]
    fn apply<const OP: u8>(lhs: __m256i, rhs: __m256i) -> __m256i {
        match OP {
            OP_AND => unsafe { _mm256_and_si256(lhs, rhs) },
            OP_OR => unsafe { _mm256_or_si256(lhs, rhs) },
            OP_XOR => unsafe { _mm256_xor_si256(lhs, rhs) },
            _ => unreachable!("unsupported binary bit operation"),
        }
    }

    /// AVX2 backend for `dst[i] = lhs[i] OP rhs[i]`.
    ///
    /// Supports `dst == lhs`.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when AVX2 is available.
    /// - `dst` must be valid for writes of `len` initialized `u64` values.
    /// - `lhs` and `rhs` must be valid for reads of `len` initialized `u64` values.
    /// - `dst` must either not overlap `lhs`, or be exactly equal to `lhs`.
    /// - `dst` must not overlap `rhs`.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn words<const OP: u8>(
        dst: *mut u64,
        lhs: *const u64,
        rhs: *const u64,
        len: usize,
    ) {
        let chunks = len / LANES;

        for chunk in 0..chunks {
            let offset = chunk * LANES;

            // SAFETY:
            // - `offset + LANES <= len`.
            // - `_mm256_loadu_si256` and `_mm256_storeu_si256` permit unaligned access.
            // - Pointer validity and overlap constraints are guaranteed by the caller.
            // - `dst == lhs` is safe because both loads happen before the store.
            unsafe {
                let lhs_vec = _mm256_loadu_si256(lhs.add(offset).cast::<__m256i>());
                let rhs_vec = _mm256_loadu_si256(rhs.add(offset).cast::<__m256i>());
                let out_vec = apply::<OP>(lhs_vec, rhs_vec);

                _mm256_storeu_si256(dst.add(offset).cast::<__m256i>(), out_vec);
            }
        }

        let done = chunks * LANES;

        // SAFETY:
        // - `done <= len`.
        // - Tail range is `done..len`.
        // - Pointer validity and overlap constraints are guaranteed by the caller.
        unsafe {
            scalar::words::<OP>(dst.add(done), lhs.add(done), rhs.add(done), len - done);
        }
    }
}

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use super::*;

    use core::arch::aarch64::{uint64x2_t, vandq_u64, veorq_u64, vld1q_u64, vorrq_u64, vst1q_u64};

    const LANES: usize = 2;

    #[inline]
    fn apply<const OP: u8>(lhs: uint64x2_t, rhs: uint64x2_t) -> uint64x2_t {
        // SAFETY:
        // - This helper is only called from `words`, which has
        //   `#[target_feature(enable = "neon")]`.
        // - The dispatch path only reaches `words` when NEON is enabled.
        unsafe {
            match OP {
                OP_AND => vandq_u64(lhs, rhs),
                OP_OR => vorrq_u64(lhs, rhs),
                OP_XOR => veorq_u64(lhs, rhs),
                _ => unreachable!("unsupported binary bit operation"),
            }
        }
    }

    /// NEON backend for `dst[i] = lhs[i] OP rhs[i]`.
    ///
    /// Supports `dst == lhs`.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when NEON is available.
    /// - `dst` must be valid for writes of `len` initialized `u64` values.
    /// - `lhs` and `rhs` must be valid for reads of `len` initialized `u64` values.
    /// - `dst` must either not overlap `lhs`, or be exactly equal to `lhs`.
    /// - `dst` must not overlap `rhs`.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn words<const OP: u8>(
        dst: *mut u64,
        lhs: *const u64,
        rhs: *const u64,
        len: usize,
    ) {
        let chunks = len / LANES;

        for chunk in 0..chunks {
            let offset = chunk * LANES;

            // SAFETY:
            // - `offset + LANES <= len`.
            // - Each NEON vector reads/writes 2 u64 values.
            // - Pointer validity and overlap constraints are guaranteed by the caller.
            // - `dst == lhs` is safe because both loads happen before the store.
            unsafe {
                let lhs_vec = vld1q_u64(lhs.add(offset));
                let rhs_vec = vld1q_u64(rhs.add(offset));
                let out_vec = apply::<OP>(lhs_vec, rhs_vec);

                vst1q_u64(dst.add(offset), out_vec);
            }
        }

        let done = chunks * LANES;

        // SAFETY:
        // - `done <= len`.
        // - Tail range is `done..len`.
        // - Pointer validity and overlap constraints are guaranteed by the caller.
        unsafe {
            scalar::words::<OP>(dst.add(done), lhs.add(done), rhs.add(done), len - done);
        }
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
