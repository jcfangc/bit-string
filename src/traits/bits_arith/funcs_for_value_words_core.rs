//! Re-exports for leading-/trailing-value-word scans.
//!
//! Use [`crate::FILL_ZEROS`] / [`crate::FILL_ONES`] as `const FILL: u64`
//! generic parameters so the two fill variants are monomorphised separately,
//! eliminating runtime fill-dispatch branches.

mod funcs_for_leading;
mod funcs_for_trailing;

pub(super) use funcs_for_leading::leading_value_words;
pub(super) use funcs_for_trailing::trailing_value_words;
