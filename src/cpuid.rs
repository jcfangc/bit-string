//! CPU feature detection — once per process, shared by all SIMD dispatch.
//!
//! Runs CPUID exactly once on first access and caches the result in a
//! `static`.  All runtime dispatch sites read from this single cache via
//! `crate::cpuid::features()`.

/// Cached CPU feature flags.  All fields are `false` on non-x86 targets.
#[allow(dead_code)]
pub(crate) struct CpuFeatures {
    pub(crate) avx2: bool,
    pub(crate) sse41: bool,
    pub(crate) ssse3: bool,
    pub(crate) sse2: bool,
}

static mut FEATURES: *const CpuFeatures = core::ptr::null();

/// Returns a reference to the process-lifetime CPU feature cache.
///
/// CPUID runs at most once, on the very first call.  Every subsequent
/// call reads the cached pointer.
#[inline]
pub(crate) fn features() -> &'static CpuFeatures {
    // SAFETY: `FEATURES` is either null (uninit) or a valid `&'static
    // CpuFeatures` that was written exactly once by `detect_and_store`.
    // Once written, the pointer is never modified.
    let p = unsafe { FEATURES };
    if !p.is_null() {
        // SAFETY: `FEATURES` was initialised by a previous call to
        // `detect_and_store`, which stored a `Box<CpuFeatures>` leaked
        // into a `'static` reference.
        unsafe { &*p }
    } else {
        detect_and_store()
    }
}

#[cold]
fn detect_and_store() -> &'static CpuFeatures {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    let features = {
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
        CpuFeatures {
            avx2: leaf7.ebx & (1 << 5) != 0,
            sse41: leaf1.ecx & (1 << 19) != 0,
            ssse3: leaf1.ecx & (1 << 9) != 0,
            sse2: leaf1.edx & (1 << 26) != 0,
        }
    };
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    let features = CpuFeatures {
        avx2: false,
        sse41: false,
        ssse3: false,
        sse2: false,
    };

    // Leak a `Box<CpuFeatures>` to get a `&'static` reference.
    let bx = alloc::boxed::Box::new(features);
    let ptr: *const CpuFeatures = alloc::boxed::Box::leak(bx);
    // SAFETY: `FEATURES` is written exactly once, before any read.
    unsafe {
        FEATURES = ptr;
        &*ptr
    }
}
