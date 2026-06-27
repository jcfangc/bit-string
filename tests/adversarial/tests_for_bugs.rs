use super::*;
use int_interval::UsizeCO;

/// Bug 1: `set_chunk` doesn't mask unused high bits after writing.
/// When `bit_start + len > self.bit_len`, bits beyond `bit_len` in the
/// last word get OR'd in but never masked, violating the invariant that
/// unused high bits are always zero.
#[test]
#[ignore = "BUG: set_chunk invariant break — reproducer (marker: B1)"]
fn diagnostic_set_chunk_invariant_break() {
    let mut bits = BitString::zeros(3);
    // Write 64 bits starting at position 0 — only bits 0-2 are valid.
    bits.set_chunk(0, u64::MAX, 64);
    // Unused bits in the last word should be zero, but they aren't.
    assert!(!view_has_same_invariants(&bits));
}

/// Bug 2: `BitStr::count_ones` doesn't mask the last partial word.
/// When a BitStr view has length not divisible by 64, the count includes
/// bits from the source beyond the view's end.
#[test]
#[ignore = "BUG: BitStr::count_ones partial word mask — reproducer (marker: B2)"]
fn diagnostic_bitstr_count_ones_last_word_mask() {
    // A 20-bit pattern with 10 ones
    let bits: BitString = "01010101010101010101".parse().unwrap();
    let view = bits.as_bit_str();

    // Slice just the first bit (which is '0')
    let sub = view.slice(UsizeCO::checked_from_start_len(0, 1).unwrap());
    assert_eq!(sub.bit_len(), 1);
    // Should be 0, but returns 10 (all the ones in the source's first word)
    let ones = sub.count_ones();
    assert_eq!(ones, 0, "BUG: count_ones({sub}) = {ones}, expected 0");
}

/// Bug 3: `BitStr::find` on unaligned starts misses occurrences that
/// straddle the unaligned->aligned word boundary.
#[test]
#[ignore = "BUG: BitStr::find blind spot at unaligned->aligned boundary — reproducer (marker: B3)"]
fn diagnostic_find_misses_cross_boundary_needle() {
    // "101" at positions 0, 63, and 126, with 60 zeros between each
    let bits: BitString = ("101".to_owned() + &"0".repeat(60) + "101" + &"0".repeat(60) + "101")
        .parse()
        .unwrap();

    let needle: BitString = "101".parse().unwrap();
    let needle_view = needle.as_bit_str();

    // First occurrence at 0 — found correctly
    assert_eq!(bits.find(needle_view), Some(0));

    // Second occurrence at 63 — let's check
    let remaining = bits.as_bit_str().slice_from(3);
    assert_eq!(
        remaining.find(needle_view),
        Some(60),
        "BUG: find on slice_from(3) should return Some(60), got {:?}",
        remaining.find(needle_view)
    );
}
