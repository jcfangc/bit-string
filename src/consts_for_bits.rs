/// Number of bits in a `u64` — the word size for `BitString` backing storage.
pub(crate) const WORD_BITS: usize = u64::BITS as usize;

/// Below this many full words, scalar loops beat SIMD dispatch overhead.
/// Must match each backend's `LANES`.
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
pub(crate) const SMALL_WORDS: usize = 4;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "ssse3",
    not(target_feature = "avx2")
))]
pub(crate) const SMALL_WORDS: usize = 2;
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub(crate) const SMALL_WORDS: usize = 2;
#[cfg(not(any(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        any(target_feature = "avx2", target_feature = "ssse3")
    ),
    all(target_arch = "aarch64", target_feature = "neon"),
)))]
pub(crate) const SMALL_WORDS: usize = 0;
