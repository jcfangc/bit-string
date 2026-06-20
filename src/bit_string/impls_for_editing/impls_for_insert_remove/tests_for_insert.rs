use crate::BitString;
use alloc::string::ToString;

#[test]
fn inserts_into_empty_bit_string() {
    let mut bits = BitString::new();

    bits.insert(0, true);

    assert_eq!(bits.bit_len(), 1);
    assert_eq!(bits.to_string(), "1");
}

#[test]
fn inserts_at_front() {
    let mut bits = BitString::try_from("0101").unwrap();

    bits.insert(0, true);

    assert_eq!(bits.bit_len(), 5);
    assert_eq!(bits.to_string(), "10101");
}

#[test]
fn inserts_in_middle() {
    let mut bits = BitString::try_from("1001").unwrap();

    bits.insert(2, true);

    assert_eq!(bits.bit_len(), 5);
    assert_eq!(bits.to_string(), "10101");
}

#[test]
fn inserts_false_in_middle() {
    let mut bits = BitString::try_from("1111").unwrap();

    bits.insert(2, false);

    assert_eq!(bits.bit_len(), 5);
    assert_eq!(bits.to_string(), "11011");
}

#[test]
fn inserts_at_back_using_push_path() {
    let mut bits = BitString::try_from("1010").unwrap();

    bits.insert(bits.bit_len(), true);

    assert_eq!(bits.bit_len(), 5);
    assert_eq!(bits.to_string(), "10101");
}

#[test]
fn shifts_bits_across_word_boundary() {
    let mut bits = BitString::zeros(130);

    bits.set(0, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);
    bits.set(129, true);

    bits.insert(64, false);

    assert_eq!(bits.bit_len(), 131);

    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(62), Some(false));
    assert_eq!(bits.get(63), Some(true));

    assert_eq!(bits.get(64), Some(false));
    assert_eq!(bits.get(65), Some(true));
    assert_eq!(bits.get(66), Some(true));
    assert_eq!(bits.get(67), Some(false));

    assert_eq!(bits.get(129), Some(false));
    assert_eq!(bits.get(130), Some(true));
    assert_eq!(bits.get(131), None);
}

#[test]
fn inserts_true_at_word_boundary() {
    let mut bits = BitString::zeros(128);

    bits.insert(64, true);

    assert_eq!(bits.bit_len(), 129);
    assert_eq!(bits.get(63), Some(false));
    assert_eq!(bits.get(64), Some(true));
    assert_eq!(bits.get(65), Some(false));
    assert_eq!(bits.get(128), Some(false));
    assert_eq!(bits.get(129), None);
}

#[test]
fn inserts_at_clamped_index_when_index_gt_len() {
    let mut bits = BitString::try_from("101").unwrap();

    bits.insert(4, true);

    assert_eq!(bits.bit_len(), 4);
    assert_eq!(bits.to_string(), "1011");
}
