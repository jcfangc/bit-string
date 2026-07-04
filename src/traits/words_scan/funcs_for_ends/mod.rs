// ── Inline CPUID cache (shared by leading and trailing) ────────────────

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod cpuid {
    use core::sync::atomic::{AtomicU8, Ordering};

    const UNINIT: u8 = 0;
    const AVX2: u8 = 1;
    const NO_AVX2: u8 = 2;

    static DETECTED: AtomicU8 = AtomicU8::new(UNINIT);

    #[cold]
    fn detect() -> u8 {
        // SAFETY: `__cpuid_count` is always safe — read-only instruction.
        #[cfg(target_arch = "x86_64")]
        let res = unsafe { core::arch::x86_64::__cpuid_count(7, 0) };
        #[cfg(target_arch = "x86")]
        let res = unsafe { core::arch::x86::__cpuid_count(7, 0) };

        let backend = if res.ebx & (1 << 5) != 0 {
            AVX2
        } else {
            NO_AVX2
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
}

mod funcs_for_leading_core;
mod funcs_for_trailing_core;

pub(crate) use funcs_for_leading_core::leading;
pub(crate) use funcs_for_trailing_core::trailing;
