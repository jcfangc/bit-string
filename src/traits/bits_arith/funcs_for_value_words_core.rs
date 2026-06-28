//! Constants and re-exports for leading-/trailing-value-word scans.
//!
//! The two fill variants (`FILL_ZEROS` / `FILL_ONES`) are passed as
//! `const FILL: u64` generic parameters so they are monomorphised
//! separately, eliminating runtime fill-dispatch branches.

pub(super) const FILL_ZEROS: u64 = 0;
pub(super) const FILL_ONES: u64 = u64::MAX;

mod funcs_for_leading;
mod funcs_for_trailing;

pub(super) use funcs_for_leading::leading_value_words;
pub(super) use funcs_for_trailing::trailing_value_words;
