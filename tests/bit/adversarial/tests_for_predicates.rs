use super::*;
use int_intervals::UsizeCO;

#[test]
fn attack_predicates_corner() {
    assert!(!BitString::new().any());
    assert!(BitString::new().all()); // vacuously true
    assert!(BitString::new().is_all_zeros());
    assert!(BitString::new().is_all_ones()); // also vacuously true

    let z = BitString::zeros(10);
    assert!(!z.any());
    assert!(!z.all());
    assert!(z.is_all_zeros());
    assert!(!z.is_all_ones());

    let o = BitString::ones(10);
    assert!(o.any());
    assert!(o.all());
    assert!(!o.is_all_zeros());
    assert!(o.is_all_ones());
}

#[test]
fn attack_predicates_word_boundaries() {
    for bit_len in [0, 1, 63, 64, 65, 127, 128, 129, 255, 256] {
        let zeros = BitString::zeros(bit_len);
        assert!(!zeros.any(), "any on zeros, len={bit_len}");
        assert_eq!(zeros.all(), bit_len == 0, "all on zeros, len={bit_len}");

        let ones = BitString::ones(bit_len);
        assert_eq!(ones.any(), bit_len != 0, "any on ones, len={bit_len}");
        assert!(ones.all(), "all on ones, len={bit_len}");

        if bit_len == 0 {
            continue;
        }

        let mut one_at_tail = zeros.clone();
        one_at_tail.set(bit_len - 1, true);
        assert!(one_at_tail.any(), "tail one, len={bit_len}");
        assert_eq!(one_at_tail.all(), bit_len == 1, "tail one, len={bit_len}");

        let mut zero_at_tail = ones;
        zero_at_tail.set(bit_len - 1, false);
        assert_eq!(zero_at_tail.any(), bit_len > 1, "tail zero, len={bit_len}");
        assert!(!zero_at_tail.all(), "tail zero, len={bit_len}");
    }
}

#[test]
fn attack_predicates_group_and_tail_boundaries() {
    let bit_len = 10 * 64 + 5;
    let zeros = BitString::zeros(bit_len);
    let ones = BitString::ones(bit_len);

    assert!(!zeros.any());
    assert!(!zeros.all());
    assert!(ones.any());
    assert!(ones.all());

    for index in [0, 511, 512, 639, 640, 644] {
        let mut one_at_index = zeros.clone();
        one_at_index.set(index, true);
        assert!(one_at_index.any(), "one at {index}");
        assert!(!one_at_index.all(), "one at {index}");

        let mut zero_at_index = ones.clone();
        zero_at_index.set(index, false);
        assert!(zero_at_index.any(), "zero at {index}");
        assert!(!zero_at_index.all(), "zero at {index}");
    }
}

// ===========================================================================
// K. BitStr predicates on unaligned views
// ===========================================================================

#[test]
fn attack_predicates_unaligned_all_ones() {
    let a = bs(&cat(&[
        "0".repeat(10).as_str(),
        "1".repeat(10).as_str(),
        "0".repeat(10).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(10, 10).unwrap());
    assert!(view.all());
    assert!(view.is_all_ones());
    assert!(!view.is_all_zeros());
    assert!(view.any());
}

#[test]
fn attack_predicates_unaligned_all_zeros() {
    let a = bs(&cat(&[
        "1".repeat(10).as_str(),
        "0".repeat(10).as_str(),
        "1".repeat(10).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(10, 10).unwrap());
    assert!(!view.any());
    assert!(!view.all());
    assert!(view.is_all_zeros());
}

#[test]
fn attack_predicates_unaligned_empty_view() {
    let bits = bs("10101");
    // empty view via sliced at boundary
    let v1 = bits.as_bit_str().slice_from(5);
    assert!(v1.is_empty());
    assert!(v1.all());
    assert!(v1.is_all_zeros());
    assert!(v1.is_all_ones());
    assert!(!v1.any());
}
