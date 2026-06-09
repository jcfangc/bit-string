use super::{WORD_BITS, word_len};

#[test]
fn returns_zero_for_empty_bit_len() {
    assert_eq!(word_len(0), 0);
}

#[test]
fn returns_one_for_non_empty_lengths_within_one_word() {
    assert_eq!(word_len(1), 1);
    assert_eq!(word_len(WORD_BITS / 2), 1);
    assert_eq!(word_len(WORD_BITS - 1), 1);
    assert_eq!(word_len(WORD_BITS), 1);
}

#[test]
fn rounds_up_for_partial_trailing_words() {
    let cases = [
        (WORD_BITS + 1, 2),
        (WORD_BITS * 2 - 1, 2),
        (WORD_BITS * 2, 2),
        (WORD_BITS * 2 + 1, 3),
    ];

    for (bit_len, expected) in cases {
        assert_eq!(word_len(bit_len), expected, "bit_len={bit_len}");
    }
}

#[test]
fn handles_usize_max_without_addition_overflow() {
    let expected = usize::MAX / WORD_BITS + usize::from(usize::MAX % WORD_BITS != 0);

    assert_eq!(word_len(usize::MAX), expected);
}

#[test]
fn covers_bit_len_with_minimal_number_of_words() {
    let cases = [
        0,
        1,
        7,
        WORD_BITS - 1,
        WORD_BITS,
        WORD_BITS + 1,
        WORD_BITS * 2 - 1,
        WORD_BITS * 2,
        WORD_BITS * 2 + 1,
        usize::MAX,
    ];

    for bit_len in cases {
        let words = word_len(bit_len);

        assert!(
            words.saturating_mul(WORD_BITS) >= bit_len,
            "word_len({bit_len})={words} does not cover all bits"
        );

        if words > 0 {
            assert!(
                (words - 1).saturating_mul(WORD_BITS) < bit_len,
                "word_len({bit_len})={words} is not minimal"
            );
        }
    }
}
