use crate::{BitString, low_mask, word_len};
use alloc::vec::Vec;

const LENGTHS: &[usize] = &[0, 1, 7, 8, 15, 31, 32, 63, 64, 65, 127, 128, 129, 193];
const WIDTHS: &[usize] = &[0, 1, 2, 3, 5, 6, 7, 8, 31, 32, 63, 64];

fn input(len: usize) -> Vec<bool> {
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state & 1 != 0
        })
        .collect()
}

fn assert_bits(actual: &BitString, expected: &[bool]) {
    assert_eq!(actual.bit_len(), expected.len());
    assert_eq!(actual.words().len(), word_len(expected.len()));
    assert_eq!(actual.iter().collect::<Vec<_>>(), expected);
    if let Some(&last) = actual.words().last() {
        let used = (expected.len() - 1) % 64 + 1;
        assert_eq!(last & !low_mask(used), 0, "unused bits must remain zero");
    }
}

#[test]
fn insert_bits_matches_boolean_model_at_every_position() {
    for &len in LENGTHS {
        let original = input(len);
        let source = BitString::from_bool_iter(original.iter().copied());
        for &width in WIDTHS {
            for index in 0..=len + 1 {
                for code in [0, u64::MAX, 0xa53c_79e1_d24b_860fu64] {
                    let mut expected = original.clone();
                    let bits = (0..width).map(|shift| (code >> shift) & 1 != 0);
                    expected.splice(index.min(len)..index.min(len), bits);
                    let mut actual = source.clone();
                    actual.insert_bits(index, code, width);
                    assert_bits(&actual, &expected);
                }
            }
        }
    }
}

#[test]
fn remove_bits_matches_boolean_model_at_every_position() {
    for &len in LENGTHS {
        let original = input(len);
        let source = BitString::from_bool_iter(original.iter().copied());
        for &width in WIDTHS {
            if width > len {
                continue;
            }
            for index in 0..=len - width {
                let mut expected = original.clone();
                expected.drain(index..index + width);
                let mut actual = source.clone();
                actual.remove_bits(index, width);
                assert_bits(&actual, &expected);
            }
        }
    }
}

#[test]
fn repeated_edits_preserve_zero_padding() {
    let mut actual = BitString::from_bool_iter(input(193));
    let original = actual.clone();
    for &width in WIDTHS {
        actual.insert_bits(63, u64::MAX, width);
        actual.remove_bits(63, width);
        assert_eq!(actual, original);
    }
    actual.insert_bits(usize::MAX, 0, 0);
    actual.remove_bits(usize::MAX, 0);
    assert_eq!(actual, original);
}

#[test]
fn invalid_arguments_leave_payload_unchanged() {
    extern crate std;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let original = BitString::from_bool_iter(input(129));
    let mut actual = original.clone();
    assert!(catch_unwind(AssertUnwindSafe(|| actual.insert_bits(0, 1, 65))).is_err());
    assert_eq!(actual, original);
    for (index, width) in [(0, 65), (129, 1), (128, 2), (usize::MAX, 1)] {
        assert!(catch_unwind(AssertUnwindSafe(|| actual.remove_bits(index, width))).is_err());
        assert_eq!(actual, original);
    }
}
