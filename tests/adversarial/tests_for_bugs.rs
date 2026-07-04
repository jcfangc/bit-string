use super::*;
use int_interval::UsizeCO;

/// Regression test for B1: `set_chunk` must mask unused high bits after writing.
#[test]
fn diagnostic_set_chunk_invariant_maintained() {
    let mut bits = BitString::zeros(3);
    bits.set_chunk(0, u64::MAX, 64);
    assert!(view_has_same_invariants(&bits));
}

/// Regression test for B2: `BitStr::count_ones` must mask the last partial word.
#[test]
fn diagnostic_bitstr_count_ones_last_word_mask() {
    let bits: BitString = "01010101010101010101".parse().unwrap();
    let view = bits.as_bit_str();

    let sub = view.slice(UsizeCO::checked_from_start_len(0, 1).unwrap());
    assert_eq!(sub.bit_len(), 1);
    assert_eq!(sub.count_ones(), 0);
}

/// Regression test for B3: `BitStr::find` must find needles that straddle
/// the unaligned→aligned word boundary.
#[test]
fn diagnostic_find_cross_boundary_needle() {
    let bits: BitString = ("101".to_owned() + &"0".repeat(60) + "101" + &"0".repeat(60) + "101")
        .parse()
        .unwrap();

    let needle: BitString = "101".parse().unwrap();
    let needle_view = needle.as_bit_str();

    assert_eq!(bits.find_str(needle_view), Some(0));

    let remaining = bits.as_bit_str().slice_from(3);
    assert_eq!(remaining.find_str(needle_view), Some(60));
}
