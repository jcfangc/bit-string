use super::Bits;
use crate::WORD_BITS;

#[test]
fn reads_low_bits_within_first_word() {
    let bits = [0b1010u64];

    assert!(!Bits::read_a_bit_at(&bits, 0));
    assert!(Bits::read_a_bit_at(&bits, 1));
    assert!(!Bits::read_a_bit_at(&bits, 2));
    assert!(Bits::read_a_bit_at(&bits, 3));
}

#[test]
fn reads_high_bits_within_first_word() {
    let bits = [1u64 << (WORD_BITS - 1)];

    assert!(!Bits::read_a_bit_at(&bits, WORD_BITS - 2));
    assert!(Bits::read_a_bit_at(&bits, WORD_BITS - 1));
}

#[test]
fn reads_bits_across_word_boundary() {
    let bits = [1u64 << (WORD_BITS - 1), 0b101u64];

    assert!(Bits::read_a_bit_at(&bits, WORD_BITS - 1));
    assert!(Bits::read_a_bit_at(&bits, WORD_BITS));
    assert!(!Bits::read_a_bit_at(&bits, WORD_BITS + 1));
    assert!(Bits::read_a_bit_at(&bits, WORD_BITS + 2));
}

#[test]
fn reads_from_later_words() {
    let bits = [0, 0, 1u64 << 7];

    assert!(!Bits::read_a_bit_at(&bits, WORD_BITS * 2 + 6));
    assert!(Bits::read_a_bit_at(&bits, WORD_BITS * 2 + 7));
    assert!(!Bits::read_a_bit_at(&bits, WORD_BITS * 2 + 8));
}

#[test]
fn works_for_sparse_endpoint_cases() {
    let bits = [1, 1u64 << (WORD_BITS - 1), 1];

    let cases = [
        (0, true),
        (1, false),
        (WORD_BITS - 1, false),
        (WORD_BITS, false),
        (WORD_BITS * 2 - 1, true),
        (WORD_BITS * 2, true),
        (WORD_BITS * 2 + 1, false),
    ];

    for (index, expected) in cases {
        assert_eq!(Bits::read_a_bit_at(&bits, index), expected, "index={index}");
    }
}
