use alloc::string::ToString;
use int_interval::UsizeCO;

use crate::BitString;

#[inline]
fn iv(start: usize, end_excl: usize) -> UsizeCO {
    UsizeCO::try_new(start, end_excl).unwrap()
}

// ---------------------------------------------------------------------------
// drain_interval  (borrowing)
// ---------------------------------------------------------------------------

#[test]
fn borrowing_variant_leaves_original_unchanged() {
    let bits = BitString::try_from("101001").unwrap();
    let interval = iv(2, 5);

    let removed = bits.slice(interval);
    let result = bits.drain_interval(interval);

    // original unchanged
    assert_eq!(bits.to_string(), "101001");
    assert_eq!(bits.bit_len(), 6);
    // removed slice matches
    assert_eq!(removed.to_string(), "100");
    // result has interval removed
    assert_eq!(result.to_string(), "101");
    assert_eq!(result.bit_len(), 3);
}

#[test]
fn borrowing_variant_drains_entire_string() {
    let bits = BitString::try_from("101001").unwrap();

    let result = bits.drain_interval(iv(0, bits.bit_len()));

    assert_eq!(bits.to_string(), "101001"); // original unchanged
    assert_eq!(result.bit_len(), 0);
    assert_eq!(result.to_string(), "");
}

// ---------------------------------------------------------------------------
// drain_interval_assign  (mutable)
// ---------------------------------------------------------------------------

#[test]
fn assign_drains_prefix() {
    let mut bits = BitString::try_from("101001").unwrap();

    let removed = bits.slice(iv(0, 2));
    bits.drain_interval_assign(iv(0, 2));

    assert_eq!(removed.to_string(), "10");
    assert_eq!(bits.to_string(), "1001");
}

#[test]
fn assign_drains_middle() {
    let mut bits = BitString::try_from("101001").unwrap();

    let removed = bits.slice(iv(2, 5));
    bits.drain_interval_assign(iv(2, 5));

    assert_eq!(removed.to_string(), "100");
    assert_eq!(bits.to_string(), "101");
}

#[test]
fn assign_drains_suffix() {
    let mut bits = BitString::try_from("101001").unwrap();

    let removed = bits.slice(iv(3, 6));
    bits.drain_interval_assign(iv(3, 6));

    assert_eq!(removed.to_string(), "001");
    assert_eq!(bits.to_string(), "101");
}

#[test]
fn assign_drains_entire_bit_string() {
    let mut bits = BitString::try_from("101001").unwrap();

    let removed = bits.slice(iv(0, bits.bit_len()));
    bits.drain_interval_assign(iv(0, bits.bit_len()));

    assert_eq!(removed.to_string(), "101001");
    assert_eq!(bits.bit_len(), 0);
    assert_eq!(bits.to_string(), "");
    assert_eq!(bits.as_words().len(), 0);
}

#[test]
fn assign_drains_across_word_boundary() {
    let mut bits = BitString::zeros(130);

    bits.set(0, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);
    bits.set(129, true);

    let removed = bits.slice(iv(63, 66));
    bits.drain_interval_assign(iv(63, 66));

    assert_eq!(removed.bit_len(), 3);
    assert_eq!(removed.to_string(), "111");

    assert_eq!(bits.bit_len(), 127);
    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(62), Some(false));
    assert_eq!(bits.get(63), Some(false));
    assert_eq!(bits.get(126), Some(true));
    assert_eq!(bits.get(127), None);
}

#[test]
fn assign_drains_large_middle_interval() {
    let mut bits = BitString::try_from("111000101011").unwrap();

    let removed = bits.slice(iv(3, 9));
    bits.drain_interval_assign(iv(3, 9));

    assert_eq!(removed.to_string(), "000101");
    assert_eq!(bits.to_string(), "111011");
}

#[test]
fn assign_removed_and_remaining_are_independent() {
    let mut bits = BitString::try_from("101001").unwrap();

    let mut removed = bits.slice(iv(1, 4));
    bits.drain_interval_assign(iv(1, 4));

    assert_eq!(bits.to_string(), "101");
    assert_eq!(removed.to_string(), "010");

    bits.set(0, false);
    removed.set(1, false);

    assert_eq!(bits.to_string(), "001");
    assert_eq!(removed.to_string(), "000");
}

#[test]
#[should_panic(expected = "bit string interval out of bounds")]
fn assign_panics_when_interval_end_exceeds_len() {
    let mut bits = BitString::try_from("101").unwrap();
    bits.drain_interval_assign(iv(1, 4));
}

// ---------------------------------------------------------------------------
// drain_interval_into  (consuming)
// ---------------------------------------------------------------------------

#[test]
fn into_drains_prefix() {
    let bits = BitString::try_from("101001").unwrap();

    let result = bits.drain_interval_into(iv(0, 2));

    assert_eq!(result.to_string(), "1001");
    assert_eq!(result.bit_len(), 4);
}

#[test]
fn into_drains_middle() {
    let bits = BitString::try_from("101001").unwrap();

    let result = bits.drain_interval_into(iv(2, 5));

    assert_eq!(result.to_string(), "101");
    assert_eq!(result.bit_len(), 3);
}

#[test]
fn into_drains_suffix() {
    let bits = BitString::try_from("101001").unwrap();

    let result = bits.drain_interval_into(iv(3, 6));

    assert_eq!(result.to_string(), "101");
    assert_eq!(result.bit_len(), 3);
}

#[test]
fn into_drains_large_gap_for_in_place_path() {
    let mut bits = BitString::zeros(200);
    bits.set(0, true);
    bits.set(50, true);
    bits.set(130, true);
    bits.set(199, true);

    // Drain [10, 120): bit 50 is removed (inside interval);
    // bits at 130 and 199 shift left by 110.
    let result = bits.drain_interval_into(iv(10, 120));

    // New length: 200 - 110 = 90.
    assert_eq!(result.bit_len(), 90);
    // Bit 0 stays at 0.
    assert_eq!(result.get(0), Some(true));
    // Bit 50 was drained — nothing at position 50.
    assert_eq!(result.get(50), Some(false));
    // Bit 130 moves to 130 - 110 = 20.
    assert_eq!(result.get(20), Some(true));
    // Bit 199 moves to 199 - 110 = 89.
    assert_eq!(result.get(89), Some(true));
    // Nothing beyond new length.
    assert_eq!(result.get(90), None);
}
