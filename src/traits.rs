mod packed_char;
pub(crate) mod word_ord;
pub(crate) mod words_arith;
pub(crate) mod words_edit;
pub(crate) mod words_eq;
pub(crate) mod words_find;
pub(crate) mod words_ord;
pub(crate) mod words_pack;
pub(crate) mod words_scan;
#[cfg(any(
    test,
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    )
))]
pub(crate) mod words_unpack;

pub use packed_char::PackedChar;
pub(crate) use word_ord::*;
pub(crate) use words_arith::*;
pub(crate) use words_edit::*;
pub(crate) use words_eq::*;
pub(crate) use words_find::*;
pub(crate) use words_ord::*;
pub(crate) use words_pack::*;
pub(crate) use words_scan::*;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
pub(crate) use words_unpack::WordsUnpack;
