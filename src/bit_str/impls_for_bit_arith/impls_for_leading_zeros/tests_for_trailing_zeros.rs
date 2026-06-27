use int_interval::UsizeCO;

use crate::BitString;

/// An empty view has no trailing zeros.
#[test]
fn empty_view_returns_zero() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str().slice(UsizeCO::try_new(10, 20).unwrap());
    assert_eq!(v.trailing_zeros(), 0);
}

/// Ends with a 1 → zero trailing zeros.
#[test]
fn ends_with_one() {
    let bits = BitString::try_from("11001").unwrap();
    assert_eq!(bits.as_bit_str().trailing_zeros(), 0);
}

/// Ends with several zeros.
#[test]
fn trailing_zero_run() {
    let bits = BitString::try_from("10100").unwrap();
    assert_eq!(bits.as_bit_str().trailing_zeros(), 2);
}

/// Single zero bit.
#[test]
fn single_zero() {
    let bits = BitString::zeros(1);
    assert_eq!(bits.as_bit_str().trailing_zeros(), 1);
}

/// Single one bit.
#[test]
fn single_one() {
    let bits = BitString::ones(1);
    assert_eq!(bits.as_bit_str().trailing_zeros(), 0);
}

/// All zeros at various lengths.
#[test]
fn all_zeros_at_various_lengths() {
    for len in [1, 63, 64, 65, 127, 128, 129, 130] {
        let bits = BitString::zeros(len);
        assert_eq!(bits.as_bit_str().trailing_zeros(), len, "len={len}");
    }
}

/// All ones at various lengths.
#[test]
fn all_ones_at_various_lengths() {
    for len in [1, 63, 64, 65, 127, 128, 129, 130] {
        let bits = BitString::ones(len);
        assert_eq!(bits.as_bit_str().trailing_zeros(), 0, "len={len}");
    }
}

/// Last 1 is deep in the first word (left side).
#[test]
fn last_one_in_first_word() {
    let mut bits = BitString::zeros(130);
    bits.set(30, true);
    assert_eq!(bits.as_bit_str().trailing_zeros(), 99); // bits 31..129 are zero
}

/// Last 1 at word boundary.
#[test]
fn last_one_at_word_boundary() {
    let mut bits = BitString::zeros(130);
    bits.set(63, true);
    assert_eq!(bits.as_bit_str().trailing_zeros(), 66); // bits 64..129
}

/// Unaligned view.
#[test]
fn unaligned_start() {
    let mut bits = BitString::zeros(130);
    bits.set(120, true); // last 1 at bit 120
    // View bits 3..130 → bit_len 127
    let v = bits.as_bit_str().slice(UsizeCO::try_new(3, 130).unwrap());
    assert_eq!(v.trailing_zeros(), 9); // bits 121..129 in view
}

/// Unaligned all zeros.
#[test]
fn unaligned_all_zeros() {
    let bits = BitString::zeros(200);
    let v = bits.as_bit_str().slice(UsizeCO::try_new(3, 130).unwrap());
    assert_eq!(v.trailing_zeros(), 127);
}

/// Unaligned single word.
#[test]
fn unaligned_single_word() {
    let bits = BitString::try_from("00011111").unwrap();
    // bits: 0 0 0 1 1 1 1 1

    // View bits 0..5 → "00011"
    let v = bits.as_bit_str().slice(UsizeCO::try_new(0, 5).unwrap());
    assert_eq!(v.trailing_zeros(), 0); // ends with 1

    // View bits 0..3 → "000"
    let v = bits.as_bit_str().slice(UsizeCO::try_new(0, 3).unwrap());
    assert_eq!(v.trailing_zeros(), 3);
}

/// First bit is one (all other zeros).
#[test]
fn first_bit_is_one() {
    let mut bits = BitString::zeros(130);
    // Set the leftmost bit.
    bits.set(0, true);
    // Bits 1..129 are all zeros, so trailing from the right = 129.
    assert_eq!(bits.as_bit_str().trailing_zeros(), 129);
}

/// Invariant: trailing_zeros ≤ bit_len and trailing_zeros == bit_len iff all zeros.
#[test]
fn invariant_trailing_zeros_bounds() {
    let mut bits = BitString::zeros(200);
    for i in (0..200).step_by(7) {
        bits.set(i, true);
    }
    let full = bits.as_bit_str();

    for start in [0, 1, 5, 63, 64, 65, 127, 128] {
        for len in [10, 63, 64, 65, 128, 129] {
            let end = (start + len).min(full.bit_len());
            if start == end {
                continue;
            }
            let v = full.slice(UsizeCO::try_new(start, end).unwrap());
            let tz = v.trailing_zeros();
            assert!(tz <= v.bit_len(), "start={start} end={end} tz={tz}");
            if v.is_all_zeros() {
                assert_eq!(tz, v.bit_len(), "all zeros: start={start} end={end}");
            } else {
                // The bit at `bit_len - 1 - tz` must be 1.
                assert_eq!(
                    v.get(v.bit_len() - 1 - tz),
                    Some(true),
                    "start={start} end={end} tz={tz}"
                );
            }
        }
    }
}
