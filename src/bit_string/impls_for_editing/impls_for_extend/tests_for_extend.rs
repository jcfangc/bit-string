extern crate std;

use alloc::string::ToString;
use alloc::vec::Vec;

use crate::BitString;

#[test]
fn extends_with_owned_bool_items() {
    let mut bits = BitString::try_from("101").unwrap();

    bits.extend([false, true, false]);

    assert_eq!(bits.bit_len(), 6);
    assert_eq!(bits.to_string(), "101010");
}

#[test]
fn extends_with_borrowed_bool_items() {
    let mut bits = BitString::try_from("101").unwrap();
    let values = [false, true, false];

    bits.extend(values.iter());

    assert_eq!(bits.bit_len(), 6);
    assert_eq!(bits.to_string(), "101010");
}

#[test]
fn extending_with_empty_iter_is_noop() {
    let mut bits = BitString::try_from("101").unwrap();

    bits.extend(core::iter::empty::<bool>());

    assert_eq!(bits.bit_len(), 3);
    assert_eq!(bits.to_string(), "101");
}

#[test]
fn extends_empty_bit_string() {
    let mut bits = BitString::new();

    bits.extend([true, false, true, true]);

    assert_eq!(bits.bit_len(), 4);
    assert_eq!(bits.to_string(), "1011");
}

#[test]
fn extends_across_word_boundary() {
    let mut bits = BitString::zeros(63);

    bits.extend([true, false, true]);

    assert_eq!(bits.bit_len(), 66);

    assert_eq!(bits.get(62), Some(false));
    assert_eq!(bits.get(63), Some(true));
    assert_eq!(bits.get(64), Some(false));
    assert_eq!(bits.get(65), Some(true));
    assert_eq!(bits.get(66), None);
}

#[test]
fn extend_updates_counts() {
    let mut bits = BitString::try_from("101").unwrap();

    bits.extend([true, false, true]);

    assert_eq!(bits.to_string(), "101101");
    assert_eq!(bits.count_ones(), 4);
    assert_eq!(bits.count_zeros(), 2);
}

#[test]
fn extends_across_batch_and_word_boundaries() {
    for prefix_len in [0, 1, 63, 64, 65, 257] {
        for append_len in [
            0, 1, 63, 64, 65, 255, 256, 257, 511, 512, 513, 1023, 1024, 1025, 4095, 4096, 4097,
        ] {
            let prefix: Vec<_> = (0..prefix_len).map(|index| index % 2 == 0).collect();
            let suffix: Vec<_> = (0..append_len).map(|index| index % 3 != 0).collect();
            let mut expected = prefix.clone();
            expected.extend(suffix.iter().copied());

            let mut bits = BitString::from_bool_iter(prefix);
            bits.extend(suffix.iter());

            assert_eq!(bits.bit_len(), expected.len());
            assert!(bits.iter().eq(expected));
        }
    }
}

#[test]
fn iterator_panic_keeps_completed_batches_and_discards_partial_batch() {
    let mut actual = BitString::try_from("101").unwrap();
    let mut expected = actual.clone();
    for index in 0..256 {
        expected.push(index % 3 != 0);
    }

    let mut produced = 0;
    let input = core::iter::from_fn(move || {
        assert!(produced < 300, "iterator panic after 300 values");
        let value = produced % 3 != 0;
        produced += 1;
        Some(value)
    });
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        actual.extend(input);
    }));

    assert!(result.is_err());
    assert_eq!(actual, expected);
}

#[test]
fn iterator_panic_before_first_batch_leaves_bits_unchanged() {
    let mut actual = BitString::try_from("101").unwrap();
    let expected = actual.clone();

    let mut produced = 0;
    let input = core::iter::from_fn(move || {
        assert!(produced < 17, "iterator panic before first batch");
        produced += 1;
        Some(true)
    });
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        actual.extend(input);
    }));

    assert!(result.is_err());
    assert_eq!(actual, expected);
}
