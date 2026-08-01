mod funcs_for_leading_core;
mod funcs_for_trailing_core;

pub(crate) use funcs_for_leading_core::leading;
pub(crate) use funcs_for_trailing_core::trailing;

/// Counts consecutive bits matching `FILL` from either end of one word.
///
/// `FROM_MSB` selects `leading_zeros`; otherwise the count starts at the LSB.
#[inline]
fn count_matching<const FILL: u64, const FROM_MSB: bool>(word: u64) -> usize {
    let mismatches = word ^ FILL;
    if FROM_MSB {
        mismatches.leading_zeros() as usize
    } else {
        mismatches.trailing_zeros() as usize
    }
}
