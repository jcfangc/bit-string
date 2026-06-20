use alloc::string::ToString;

use crate::BitString;

fn assert_not_variants(input: &BitString, expected: &BitString) {
    let owned = input.not();

    let mut assigned = input.clone();
    assigned.not_assign();

    let into = input.clone().not();

    assert_eq!(owned, *expected);
    assert_eq!(assigned, *expected);
    assert_eq!(into, *expected);

    assert_eq!(owned, assigned);
    assert_eq!(owned, into);
}

#[test]
fn computes_bitwise_not() {
    let input = BitString::try_from("101101").unwrap();
    let expected = BitString::try_from("010010").unwrap();

    assert_not_variants(&input, &expected);
}

#[test]
fn not_of_zeros_is_ones() {
    let input = BitString::zeros(9);
    let expected = BitString::ones(9);

    assert_not_variants(&input, &expected);
}

#[test]
fn not_of_ones_is_zeros() {
    let input = BitString::ones(9);
    let expected = BitString::zeros(9);

    assert_not_variants(&input, &expected);

    let result = input.not();
    assert_eq!(result.count_ones(), 0);
}

#[test]
fn works_across_word_boundaries_and_masks_unused_bits() {
    let mut input = BitString::zeros(130);
    let mut expected = BitString::ones(130);

    for index in [0, 63, 64, 65, 129] {
        input.set(index, true);
        expected.set(index, false);
    }

    assert_not_variants(&input, &expected);

    let result = input.not();

    assert_eq!(result.bit_len(), 130);
    assert_eq!(result.count_ones(), 125);

    assert_eq!(result.get(0), Some(false));
    assert_eq!(result.get(63), Some(false));
    assert_eq!(result.get(64), Some(false));
    assert_eq!(result.get(65), Some(false));
    assert_eq!(result.get(128), Some(true));
    assert_eq!(result.get(129), Some(false));
    assert_eq!(result.get(130), None);
}

#[test]
fn not_does_not_mutate_input() {
    let input = BitString::try_from("101101").unwrap();
    let before = input.clone();

    let result = input.not();

    assert_eq!(result.to_string(), "010010");
    assert_eq!(input, before);
}

#[test]
fn not_assign_mutates_input() {
    let mut input = BitString::try_from("101101").unwrap();

    input.not_assign();

    assert_eq!(input.to_string(), "010010");
}

#[test]
fn not_into_matches_not() {
    let input = BitString::try_from("101101").unwrap();

    let expected = input.not();
    let actual = input.not();

    assert_eq!(actual, expected);
}

#[test]
fn double_not_returns_original() {
    let input = BitString::try_from("101001101").unwrap();

    let actual = input.not().not();

    assert_eq!(actual, input);
}

#[test]
fn works_for_empty_bit_string() {
    let input = BitString::zeros(0);

    assert_not_variants(&input, &input);
}
