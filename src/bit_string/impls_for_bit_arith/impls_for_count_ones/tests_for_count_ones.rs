use crate::BitString;

#[test]
fn counts_empty_bit_string() {
    let bits = BitString::new();

    assert_eq!(bits.count_ones(), 0);
    assert_eq!(bits.count_zeros(), 0);
}

#[test]
fn counts_all_zero_bits() {
    let bits = BitString::zeros(130);

    assert_eq!(bits.count_ones(), 0);
    assert_eq!(bits.count_zeros(), 130);
}

#[test]
fn counts_all_one_bits() {
    let cases = [1, 63, 64, 65, 127, 128, 129, 130];

    for len in cases {
        let bits = BitString::ones(len);

        assert_eq!(bits.count_ones(), len, "len={len}");
        assert_eq!(bits.count_zeros(), 0, "len={len}");
    }
}

#[test]
fn counts_mixed_bits_from_string() {
    let bits = BitString::try_from("1010011100").unwrap();

    assert_eq!(bits.count_ones(), 5);
    assert_eq!(bits.count_zeros(), 5);
}

#[test]
fn counts_bits_across_word_boundaries() {
    let mut bits = BitString::zeros(130);

    for index in [0, 1, 63, 64, 65, 127, 128, 129] {
        bits.set(index, true);
    }

    assert_eq!(bits.count_ones(), 8);
    assert_eq!(bits.count_zeros(), 122);
}

#[test]
fn count_changes_after_set() {
    let mut bits = BitString::zeros(65);

    assert_eq!(bits.count_ones(), 0);

    assert_eq!(bits.set(64, true), Some(false));
    assert_eq!(bits.count_ones(), 1);

    assert_eq!(bits.set(64, false), Some(true));
    assert_eq!(bits.count_ones(), 0);
}

#[test]
fn count_ones_ignores_unused_tail_bits() {
    let bits = BitString::ones(65).not();

    assert_eq!(bits.bit_len(), 65);
    assert_eq!(bits.count_ones(), 0);
    assert_eq!(bits.count_zeros(), 65);
}

#[test]
fn ignores_unused_bits_in_last_word() {
    let bits = BitString::try_from("101001101").unwrap();

    assert_eq!(bits.bit_len(), 9);
    assert_eq!(bits.count_ones(), 5);
    assert_eq!(bits.count_zeros(), 4);
}

#[test]
fn works_across_word_boundaries() {
    let mut bits = BitString::zeros(130);

    for index in [0, 63, 64, 65, 129] {
        bits.set(index, true);
    }

    assert_eq!(bits.count_ones(), 5);
    assert_eq!(bits.count_zeros(), 125);
}
