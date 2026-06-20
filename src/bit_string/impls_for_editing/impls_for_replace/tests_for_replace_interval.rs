use crate::BitString;
use alloc::string::ToString;

use int_interval::UsizeCO;

#[inline]
fn iv(start: usize, end_excl: usize) -> UsizeCO {
    UsizeCO::try_new(start, end_excl).unwrap()
}

// ---------------------------------------------------------------------------
// replace_interval  (borrowing)
// ---------------------------------------------------------------------------

#[test]
fn borrowing_variant_leaves_original_unchanged() {
    let bits = BitString::try_from("00111100").unwrap();
    let replacement = BitString::try_from("1010").unwrap();

    let result = bits.replace_interval(iv(2, 6), &replacement);

    // original unchanged
    assert_eq!(bits.bit_len(), 8);
    assert_eq!(bits.to_string(), "00111100");
    // result is correct
    assert_eq!(result.bit_len(), 8);
    assert_eq!(result.to_string(), "00101000");
}

#[test]
fn borrowing_variant_with_different_lengths() {
    let bits = BitString::try_from("111000").unwrap();
    let replacement = BitString::try_from("10101").unwrap();

    let result = bits.replace_interval(iv(1, 5), &replacement);

    // original unchanged
    assert_eq!(bits.to_string(), "111000");
    // result expanded
    assert_eq!(result.to_string(), "1101010");
    assert_eq!(result.bit_len(), 7);
}

#[test]
fn borrowing_variant_clamps_out_of_bounds() {
    let bits = BitString::try_from("101").unwrap();
    let replacement = BitString::try_from("0").unwrap();

    // end beyond len → clamped, no panic
    let result = bits.replace_interval(iv(1, 10), &replacement);

    assert_eq!(result.to_string(), "10");
}

// ---------------------------------------------------------------------------
// replace_interval_assign  (mutable)
// ---------------------------------------------------------------------------

#[test]
fn assign_replaces_same_length() {
    let mut bits = BitString::try_from("00111100").unwrap();
    let replacement = BitString::try_from("1010").unwrap();

    bits.replace_interval_assign(iv(2, 6), &replacement);

    assert_eq!(bits.bit_len(), 8);
    assert_eq!(bits.to_string(), "00101000");
}

#[test]
fn assign_replaces_shorter() {
    let mut bits = BitString::try_from("00111100").unwrap();
    let replacement = BitString::try_from("10").unwrap();

    bits.replace_interval_assign(iv(2, 6), &replacement);

    assert_eq!(bits.bit_len(), 6);
    assert_eq!(bits.to_string(), "001000");
}

#[test]
fn assign_replaces_longer() {
    let mut bits = BitString::try_from("001100").unwrap();
    let replacement = BitString::try_from("10101").unwrap();

    bits.replace_interval_assign(iv(2, 4), &replacement);

    assert_eq!(bits.bit_len(), 9);
    assert_eq!(bits.to_string(), "001010100");
}

#[test]
fn assign_replaces_with_empty() {
    let mut bits = BitString::try_from("00111100").unwrap();
    let replacement = BitString::new();

    bits.replace_interval_assign(iv(2, 6), &replacement);

    assert_eq!(bits.bit_len(), 4);
    assert_eq!(bits.to_string(), "0000");
}

#[test]
fn assign_replaces_at_start() {
    let mut bits = BitString::try_from("111000").unwrap();
    let replacement = BitString::try_from("00").unwrap();

    bits.replace_interval_assign(iv(0, 3), &replacement);

    assert_eq!(bits.bit_len(), 5);
    assert_eq!(bits.to_string(), "00000");
}

#[test]
fn assign_replaces_at_end() {
    let mut bits = BitString::try_from("000111").unwrap();
    let replacement = BitString::try_from("10").unwrap();

    bits.replace_interval_assign(iv(3, 6), &replacement);

    assert_eq!(bits.bit_len(), 5);
    assert_eq!(bits.to_string(), "00010");
}

#[test]
fn assign_across_word_boundary() {
    let mut bits = BitString::zeros(130);

    bits.set(62, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);
    bits.set(129, true);

    let replacement = BitString::try_from("01").unwrap();

    bits.replace_interval_assign(iv(63, 65), &replacement);

    assert_eq!(bits.bit_len(), 130);

    assert_eq!(bits.get(62), Some(true));
    assert_eq!(bits.get(63), Some(false));
    assert_eq!(bits.get(64), Some(true));
    assert_eq!(bits.get(65), Some(true));
    assert_eq!(bits.get(129), Some(true));
    assert_eq!(bits.get(130), None);
}

#[test]
fn assign_clamps_out_of_bounds_start() {
    let mut bits = BitString::try_from("101").unwrap();
    let replacement = BitString::try_from("0").unwrap();

    bits.replace_interval_assign(iv(5, 7), &replacement);

    assert_eq!(bits.to_string(), "1010");
}

#[test]
fn assign_clamps_out_of_bounds_end() {
    let mut bits = BitString::try_from("101").unwrap();
    let replacement = BitString::try_from("0").unwrap();

    bits.replace_interval_assign(iv(1, 10), &replacement);

    assert_eq!(bits.to_string(), "10");
}

#[test]
fn assign_updates_counts() {
    let mut bits = BitString::try_from("111000").unwrap();
    let replacement = BitString::try_from("101").unwrap();

    bits.replace_interval_assign(iv(1, 5), &replacement);

    assert_eq!(bits.to_string(), "11010");
    assert_eq!(bits.count_ones(), 3);
    assert_eq!(bits.count_zeros(), 2);
}

// ---------------------------------------------------------------------------
// replace_interval_into  (consuming)
// ---------------------------------------------------------------------------

#[test]
fn into_replaces_same_length() {
    let bits = BitString::try_from("00111100").unwrap();
    let replacement = BitString::try_from("1010").unwrap();

    let result = bits.replace_interval(iv(2, 6), &replacement);

    assert_eq!(result.bit_len(), 8);
    assert_eq!(result.to_string(), "00101000");
}

#[test]
fn into_replaces_shorter() {
    let bits = BitString::try_from("00111100").unwrap();
    let replacement = BitString::try_from("10").unwrap();

    let result = bits.replace_interval(iv(2, 6), &replacement);

    assert_eq!(result.bit_len(), 6);
    assert_eq!(result.to_string(), "001000");
}

#[test]
fn into_replaces_longer() {
    let bits = BitString::try_from("001100").unwrap();
    let replacement = BitString::try_from("10101").unwrap();

    let result = bits.replace_interval(iv(2, 4), &replacement);

    assert_eq!(result.bit_len(), 9);
    assert_eq!(result.to_string(), "001010100");
}

#[test]
fn into_clamps_out_of_bounds() {
    let bits = BitString::try_from("101").unwrap();
    let replacement = BitString::try_from("0").unwrap();

    let result = bits.replace_interval(iv(1, 10), &replacement);

    assert_eq!(result.to_string(), "10");
}
