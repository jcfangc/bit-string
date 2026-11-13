use crate::BitString;
use alloc::string::ToString;

#[test]
fn retain_all_is_noop() {
    let mut bits = BitString::try_from("101001").unwrap();

    bits.retain(|_| true);

    assert_eq!(bits.bit_len(), 6);
    assert_eq!(bits.to_string(), "101001");
    assert_eq!(bits.count_ones(), 3);
    assert_eq!(bits.count_zeros(), 3);
}

#[test]
fn retain_none_clears_bit_string() {
    let mut bits = BitString::try_from("101001").unwrap();

    bits.retain(|_| false);

    assert_eq!(bits.bit_len(), 0);
    assert!(bits.is_empty());
    assert_eq!(bits.to_string(), "");
    assert_eq!(bits.count_ones(), 0);
    assert_eq!(bits.count_zeros(), 0);
}

#[test]
fn retains_only_true_bits() {
    let mut bits = BitString::try_from("101001").unwrap();

    bits.retain(|value| value);

    assert_eq!(bits.bit_len(), 3);
    assert_eq!(bits.to_string(), "111");
    assert_eq!(bits.count_ones(), 3);
    assert_eq!(bits.count_zeros(), 0);
}

#[test]
fn retains_only_false_bits() {
    let mut bits = BitString::try_from("101001").unwrap();

    bits.retain(|value| !value);

    assert_eq!(bits.bit_len(), 3);
    assert_eq!(bits.to_string(), "000");
    assert_eq!(bits.count_ones(), 0);
    assert_eq!(bits.count_zeros(), 3);
}

#[test]
fn calls_predicate_in_original_order() {
    let mut bits = BitString::try_from("101001").unwrap();
    let mut seen = alloc::vec::Vec::new();

    bits.retain(|value| {
        seen.push(value);
        true
    });

    assert_eq!(seen, [true, false, true, false, false, true]);
    assert_eq!(bits.to_string(), "101001");
}

#[test]
fn retain_predicate_can_use_state() {
    let mut bits = BitString::try_from("111111").unwrap();
    let mut index = 0usize;

    bits.retain(|_| {
        let keep = index % 2 == 0;
        index += 1;
        keep
    });

    assert_eq!(bits.bit_len(), 3);
    assert_eq!(bits.to_string(), "111");
}

#[test]
fn retain_compresses_across_word_boundary() {
    let mut bits = BitString::zeros(130);

    bits.set(0, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);
    bits.set(129, true);

    bits.retain(|value| value);

    assert_eq!(bits.bit_len(), 5);
    assert_eq!(bits.to_string(), "11111");
    assert_eq!(bits.count_ones(), 5);
    assert_eq!(bits.count_zeros(), 0);
}

#[test]
fn retain_false_masks_old_true_bits_after_truncate() {
    let mut bits = BitString::ones(70);

    bits.retain(|value| !value);

    assert_eq!(bits.bit_len(), 0);
    assert_eq!(bits.count_ones(), 0);
    assert_eq!(bits.count_zeros(), 0);
}

#[test]
fn retain_mixed_values_preserves_relative_order() {
    let mut bits = BitString::try_from("010110100").unwrap();

    bits.retain(|value| value);

    assert_eq!(bits.to_string(), "1111");
}

#[test]
fn retain_handles_word_boundary_lengths() {
    for len in [0, 1, 63, 64, 65, 127, 128, 129] {
        let original = (0..len)
            .map(|index| index % 3 == 0)
            .collect::<alloc::vec::Vec<_>>();
        let expected = original
            .iter()
            .copied()
            .filter(|&value| value)
            .collect::<alloc::vec::Vec<_>>();
        let mut bits = BitString::from_bool_iter(original.iter().copied());

        bits.retain(|value| value);

        assert_eq!(bits.to_bool_vec(), expected, "length {len}");
    }
}

#[test]
fn predicate_panic_leaves_a_valid_bit_string() {
    extern crate std;

    let mut bits = BitString::from_bool_iter((0..130).map(|index| index % 3 == 0));
    let mut calls = 0usize;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        bits.retain(|value| {
            if calls == 70 {
                panic!("predicate panic");
            }
            calls += 1;
            value
        });
    }));

    assert!(result.is_err());
    assert_eq!(calls, 70);
    assert_eq!(bits.bit_len(), 130);
    assert_eq!(bits.words.len(), 3);
    assert_eq!(bits.words[2] & !crate::low_mask(2), 0);
    assert_eq!(bits.to_bool_vec().len(), bits.bit_len());

    bits.push(true);
    assert_eq!(bits.bit_len(), 131);
    assert_eq!(bits.get(130), Some(true));
}
