//! CPU feature detection — once per process, shared by all SIMD dispatch.
//!
//! Runs CPUID exactly once on first access via `OnceCell`.  All runtime
//! dispatch sites read from this single cache via `crate::cpuid::features()`.

use once_cell::sync::OnceCell;

/// Cached CPU feature flags.  All fields are `false` on non-x86 targets.
#[allow(dead_code)]
pub(crate) struct CpuFeatures {
    pub(crate) avx2: bool,
    pub(crate) sse41: bool,
    pub(crate) ssse3: bool,
    pub(crate) sse2: bool,
}

static FEATURES: OnceCell<CpuFeatures> = OnceCell::new();

/// Returns a reference to the process-lifetime CPU feature cache.
///
/// CPUID runs at most once, on the very first call.
#[inline]
pub(crate) fn features() -> &'static CpuFeatures {
    FEATURES.get_or_init(|| {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            // SAFETY: `__cpuid_count` is always safe — read-only instruction.
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
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        CpuFeatures {
            avx2: false,
            sse41: false,
            ssse3: false,
            sse2: false,
        }
    })
}
