use crate::BitString;
use alloc::string::ToString;

// ---------------------------------------------------------------------------
// replace  (borrowing)
// ---------------------------------------------------------------------------

#[test]
fn borrowing_variant_leaves_original_unchanged() {
    let bits = BitString::try_from("00111100").unwrap();
    let replacement = BitString::try_from("1010").unwrap();

    let result = bits.replace(2, &replacement);

    assert_eq!(bits.to_string(), "00111100");
    assert_eq!(result.bit_len(), 8);
    assert_eq!(result.to_string(), "00101000");
}

#[test]
fn borrowing_variant_extends() {
    let bits = BitString::try_from("0011").unwrap();
    let replacement = BitString::try_from("10101").unwrap();

    let result = bits.replace(2, &replacement);

    assert_eq!(bits.to_string(), "0011");
    assert_eq!(result.bit_len(), 7);
    assert_eq!(result.to_string(), "0010101");
}

#[test]
fn borrowing_variant_clamps_start() {
    let bits = BitString::try_from("110").unwrap();
    let replacement = BitString::try_from("01").unwrap();

    let result = bits.replace(7, &replacement);

    assert_eq!(result.to_string(), "11001");
}

// ---------------------------------------------------------------------------
// replace_assign  (mutable)
// ---------------------------------------------------------------------------

#[test]
fn assign_overwrites_in_place() {
    let mut bits = BitString::try_from("00111100").unwrap();
    let replacement = BitString::try_from("1010").unwrap();

    bits.replace_assign(2, &replacement);

    assert_eq!(bits.bit_len(), 8);
    assert_eq!(bits.to_string(), "00101000");
}

#[test]
fn assign_clamps_start_beyond_len() {
    let mut bits = BitString::try_from("110").unwrap();
    let replacement = BitString::try_from("01").unwrap();

    bits.replace_assign(7, &replacement);

    assert_eq!(bits.bit_len(), 5);
    assert_eq!(bits.to_string(), "11001");
}

#[test]
fn assign_extends_when_replacement_exceeds_tail() {
    let mut bits = BitString::try_from("001100").unwrap();
    let replacement = BitString::try_from("10101").unwrap();

    bits.replace_assign(2, &replacement);

    assert_eq!(bits.bit_len(), 7);
    assert_eq!(bits.to_string(), "0010101");
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

    bits.replace_assign(63, &replacement);

    assert_eq!(bits.bit_len(), 130);

    assert_eq!(bits.get(62), Some(true));
    assert_eq!(bits.get(63), Some(false));
    assert_eq!(bits.get(64), Some(true));
    assert_eq!(bits.get(65), Some(true));
    assert_eq!(bits.get(129), Some(true));
    assert_eq!(bits.get(130), None);
}

#[test]
fn assign_empty_replacement_is_noop() {
    let mut bits = BitString::try_from("101").unwrap();
    let replacement = BitString::new();

    bits.replace_assign(1, &replacement);

    assert_eq!(bits.to_string(), "101");
    assert_eq!(bits.bit_len(), 3);
}

#[test]
fn assign_keeps_length_when_within_bounds() {
    let mut bits = BitString::try_from("00111100").unwrap();
    let replacement = BitString::try_from("10").unwrap();

    bits.replace_assign(2, &replacement);

    assert_eq!(bits.bit_len(), 8);
    assert_eq!(bits.to_string(), "00101100");
}

// ---------------------------------------------------------------------------
// replace_into  (consuming)
// ---------------------------------------------------------------------------

#[test]
fn into_overwrites_in_place() {
    let bits = BitString::try_from("00111100").unwrap();
    let replacement = BitString::try_from("1010").unwrap();

    let result = bits.replace_into(2, &replacement);

    assert_eq!(result.bit_len(), 8);
    assert_eq!(result.to_string(), "00101000");
}

#[test]
fn into_extends() {
    let bits = BitString::try_from("0011").unwrap();
    let replacement = BitString::try_from("10101").unwrap();

    let result = bits.replace_into(2, &replacement);

    assert_eq!(result.bit_len(), 7);
    assert_eq!(result.to_string(), "0010101");
}

#[test]
fn into_clamps_start() {
    let bits = BitString::try_from("110").unwrap();
    let replacement = BitString::try_from("01").unwrap();

    let result = bits.replace_into(7, &replacement);

    assert_eq!(result.to_string(), "11001");
}
