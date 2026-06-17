use super::Bits;
use crate::WORD_BITS;

#[test]
fn sets_selected_bit_to_true() {
    let cases = [
        0,
        1,
        WORD_BITS - 1,
        WORD_BITS,
        WORD_BITS + 1,
        WORD_BITS * 2 - 1,
    ];

    for index in cases {
        let mut bits = [0u64; 2];

        Bits::set_a_bit_at(&mut bits, index, true);

        let word = index / WORD_BITS;
        let offset = index % WORD_BITS;

        assert_eq!(bits[word], 1u64 << offset, "index={index}");
    }
}

#[test]
fn sets_selected_bit_to_false() {
    let cases = [
        0,
        1,
        WORD_BITS - 1,
        WORD_BITS,
        WORD_BITS + 1,
        WORD_BITS * 2 - 1,
    ];

    for index in cases {
        let mut bits = [u64::MAX; 2];

        Bits::set_a_bit_at(&mut bits, index, false);

        let word = index / WORD_BITS;
        let offset = index % WORD_BITS;

        assert_eq!(bits[word] & (1u64 << offset), 0, "index={index}");
    }
}

#[test]
fn preserves_unselected_bits_when_setting_true() {
    let mut bits = [0b1010u64, 0b0101u64];

    Bits::set_a_bit_at(&mut bits, 0, true);
    Bits::set_a_bit_at(&mut bits, WORD_BITS + 1, true);

    assert_eq!(bits[0], 0b1011);
    assert_eq!(bits[1], 0b0111);
}

#[test]
fn preserves_unselected_bits_when_setting_false() {
    let mut bits = [0b1111u64, 0b1111u64];

    Bits::set_a_bit_at(&mut bits, 1, false);
    Bits::set_a_bit_at(&mut bits, WORD_BITS + 2, false);

    assert_eq!(bits[0], 0b1101);
    assert_eq!(bits[1], 0b1011);
}

#[test]
fn setting_same_value_is_idempotent() {
    let mut bits = [0u64; 2];

    Bits::set_a_bit_at(&mut bits, WORD_BITS + 3, true);
    let once = bits;

    Bits::set_a_bit_at(&mut bits, WORD_BITS + 3, true);
    assert_eq!(bits, once);

    Bits::set_a_bit_at(&mut bits, WORD_BITS + 3, false);
    let cleared = bits;

    Bits::set_a_bit_at(&mut bits, WORD_BITS + 3, false);
    assert_eq!(bits, cleared);
}
