use super::Bits;

#[test]
fn returns_empty_box_when_target_word_count_is_zero() {
    let bits = [1, 2, 3];

    let out = Bits::shrink_words(&bits, 0);

    let empty: &[u64] = &[];
    assert_eq!(&*out, empty);
}

#[test]
fn copies_prefix_with_requested_word_count() {
    let bits = [10, 20, 30, 40];

    let out = Bits::shrink_words(&bits, 2);

    assert_eq!(&*out, &[10, 20]);
}

#[test]
fn copies_all_words_when_target_word_count_matches_input_len() {
    let bits = [u64::MAX, 0, 0x1234_5678_9abc_def0];

    let out = Bits::shrink_words(&bits, bits.len());

    assert_eq!(&*out, &bits);
}

#[test]
fn output_is_independent_from_source_slice() {
    let mut bits = [1, 2, 3];

    let out = Bits::shrink_words(&bits, 2);
    bits[0] = 99;

    assert_eq!(&*out, &[1, 2]);
    assert_eq!(bits, [99, 2, 3]);
}

#[test]
#[should_panic]
fn panics_when_target_word_count_exceeds_input_len() {
    let bits = [1, 2];

    let _ = Bits::shrink_words(&bits, 3);
}
