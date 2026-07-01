//! Runtime CPU feature detection for SIMD backend selection.
//!
//! Compiles only in default mode (no `compile-time-dispatch`) on x86/x86_64.

use core::sync::atomic::{AtomicU8, Ordering};

const UNINIT: u8 = 0;
const AVX2: u8 = 1;

static DETECTED: AtomicU8 = AtomicU8::new(UNINIT);

#[cold]
fn detect() -> u8 {
    // CPUID leaf 7, subleaf 0: EBX bit 5 = AVX2.
    #[cfg(target_arch = "x86_64")]
    let res = unsafe { core::arch::x86_64::__cpuid_count(7, 0) };
    #[cfg(target_arch = "x86")]
    let res = unsafe { core::arch::x86::__cpuid_count(7, 0) };

    let backend = if res.ebx & (1 << 5) != 0 {
        AVX2
    } else {
        UNINIT
    };
    DETECTED.store(backend, Ordering::Relaxed);
    backend
}

#[inline(always)]
pub(super) fn has_avx2() -> bool {
    let b = DETECTED.load(Ordering::Relaxed);
    if b != UNINIT {
        return b == AVX2;
    }
    detect() == AVX2
}
