use super::zero_words;

#[test]
fn returns_empty_box_for_zero_words() {
    let bits = zero_words(0);

    assert!(bits.is_empty());
}

#[test]
fn returns_requested_number_of_words() {
    let cases = [1, 2, 7, 64, 65];

    for words in cases {
        let bits = zero_words(words);

        assert_eq!(bits.len(), words, "words={words}");
    }
}

#[test]
fn initializes_all_words_to_zero() {
    let cases = [1, 2, 7, 64, 65];

    for words in cases {
        let bits = zero_words(words);

        assert!(
            bits.iter().all(|&word| word == 0),
            "words={words}, bits={bits:?}"
        );
    }
}
