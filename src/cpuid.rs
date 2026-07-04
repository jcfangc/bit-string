//! CPU feature detection — once per process, shared by all SIMD dispatch.
//!
//! A simple OnceCell pattern (AtomicU8 state machine) runs CPUID exactly
//! once on first access.  All runtime dispatch sites read from this single
//! cache via `crate::cpuid::features()`.

use core::sync::atomic::{AtomicU8, Ordering};

/// Cached CPU feature flags.  All fields are `false` on non-x86 targets.
#[allow(dead_code)]
pub(crate) struct CpuFeatures {
    pub(crate) avx2: bool,
    pub(crate) sse41: bool,
    pub(crate) ssse3: bool,
    pub(crate) sse2: bool,
}

const UNINIT: u8 = 0;
const INIT: u8 = 1;

static STATE: AtomicU8 = AtomicU8::new(UNINIT);

// SAFETY: written exactly once, on the first call to `features()`,
// before any read.  Uses a raw-pointer approach to avoid edition 2024's
// restriction on `&static mut`.
static mut FEATURES: CpuFeatures = CpuFeatures {
    avx2: false,
    sse41: false,
    ssse3: false,
    sse2: false,
};

/// Returns a shared reference to the cached features.
///
/// # Safety
///
/// `FEATURES` must have been fully initialised (STATE == INIT).
#[inline]
unsafe fn features_ref() -> &'static CpuFeatures {
    // SAFETY: caller guarantees STATE == INIT and FEATURES will not be
    // mutated again.
    unsafe { &*(&raw const FEATURES) }
}

/// Returns a reference to the process-lifetime CPU feature cache.
///
/// CPUID is called at most once — on the very first invocation.  Every
/// subsequent call reads the cached `static` via a single `Relaxed` load.
#[inline]
pub(crate) fn features() -> &'static CpuFeatures {
    if STATE.load(Ordering::Relaxed) == INIT {
        // SAFETY: `FEATURES` was initialized on the first call.
        unsafe { features_ref() }
    } else {
        detect_and_store()
    }
}

#[cold]
fn detect_and_store() -> &'static CpuFeatures {
    // If another thread raced here, spin until it finishes.
    if STATE
        .compare_exchange(UNINIT, INIT, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        while STATE.load(Ordering::Relaxed) != INIT {
            core::hint::spin_loop();
        }
        // SAFETY: `FEATURES` was initialized by the winning thread.
        return unsafe { features_ref() };
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        // SAFETY: `__cpuid_count` is always safe to call on x86/x86_64 —
        // it is a read-only instruction that queries CPU capabilities.
        #[cfg(target_arch = "x86_64")]
        let (leaf1, leaf7) = unsafe {
            (
                core::arch::x86_64::__cpuid_count(1, 0),
                core::arch::x86_64::__cpuid_count(7, 0),
            )
        };
        #[cfg(target_arch = "x86")]
        let (leaf1, leaf7) = unsafe {
            (
                core::arch::x86::__cpuid_count(1, 0),
                core::arch::x86::__cpuid_count(7, 0),
            )
        };
        // SAFETY: `FEATURES` is written exactly once, gated by `STATE`.
        unsafe {
            FEATURES = CpuFeatures {
                avx2: leaf7.ebx & (1 << 5) != 0,
                sse41: leaf1.ecx & (1 << 19) != 0,
                ssse3: leaf1.ecx & (1 << 9) != 0,
                sse2: leaf1.edx & (1 << 26) != 0,
            };
        }
    }
    // On non-x86 targets, the `static mut` was already initialized to
    // all-false above.  We just need to set STATE so subsequent calls
    // take the fast path.

    STATE.store(INIT, Ordering::Release);
    // SAFETY: `FEATURES` is now fully initialized.
    unsafe { features_ref() }
}
