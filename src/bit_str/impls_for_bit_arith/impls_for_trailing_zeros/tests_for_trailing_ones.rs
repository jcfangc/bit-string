use int_interval::UsizeCO;

use crate::BitString;

#[test]
fn empty_view_returns_zero() {
    let bits = BitString::try_from("01010").unwrap();
    let v = bits.as_bit_str().slice(UsizeCO::try_new(10, 20).unwrap());
    assert_eq!(v.trailing_ones(), 0);
}

#[test]
fn ends_with_zero() {
    let bits = BitString::try_from("10110").unwrap();
    assert_eq!(bits.as_bit_str().trailing_ones(), 0);
}

#[test]
fn trailing_one_run() {
    let bits = BitString::try_from("01011").unwrap();
    assert_eq!(bits.as_bit_str().trailing_ones(), 2);
}

#[test]
fn single_one() {
    let bits = BitString::ones(1);
    assert_eq!(bits.as_bit_str().trailing_ones(), 1);
}

#[test]
fn single_zero() {
    let bits = BitString::zeros(1);
    assert_eq!(bits.as_bit_str().trailing_ones(), 0);
}

#[test]
fn all_ones_at_various_lengths() {
    for len in [1, 63, 64, 65, 127, 128, 129, 130] {
        let bits = BitString::ones(len);
        assert_eq!(bits.as_bit_str().trailing_ones(), len, "len={len}");
    }
}

#[test]
fn all_zeros_at_various_lengths() {
    for len in [1, 63, 64, 65, 127, 128, 129, 130] {
        let bits = BitString::zeros(len);
        assert_eq!(bits.as_bit_str().trailing_ones(), 0, "len={len}");
    }
}

#[test]
fn last_zero_cross_word() {
    let mut bits = BitString::ones(130);
    bits.set(30, false);
    assert_eq!(bits.as_bit_str().trailing_ones(), 99);
}

#[test]
fn unaligned_view() {
    let mut bits = BitString::ones(130);
    bits.set(120, false);
    let v = bits.as_bit_str().slice(UsizeCO::try_new(3, 130).unwrap());
    assert_eq!(v.trailing_ones(), 9);
}

#[test]
fn unaligned_all_ones() {
    let bits = BitString::ones(200);
    let v = bits.as_bit_str().slice(UsizeCO::try_new(3, 130).unwrap());
    assert_eq!(v.trailing_ones(), 127);
}

#[test]
fn unaligned_single_word() {
    let bits = BitString::try_from("11100000").unwrap();
    // bits: 1 1 1 0 0 0 0 0
    let v = bits.as_bit_str().slice(UsizeCO::try_new(0, 5).unwrap());
    // "11100" → trailing ones = 0
    assert_eq!(v.trailing_ones(), 0);

    let v = bits.as_bit_str().slice(UsizeCO::try_new(0, 3).unwrap());
    // "111" → trailing ones = 3
    assert_eq!(v.trailing_ones(), 3);
}

#[test]
fn invariant_trailing_ones_bounds() {
    let mut bits = BitString::ones(200);
    for i in (1..200).step_by(7) {
        bits.set(i, false);
    }
    let full = bits.as_bit_str();

    for start in [0, 1, 5, 63, 64, 65, 127, 128] {
        for len in [10, 63, 64, 65, 128, 129] {
            let end = (start + len).min(full.bit_len());
            if start == end {
                continue;
            }
            let v = full.slice(UsizeCO::try_new(start, end).unwrap());
            let to = v.trailing_ones();
            assert!(to <= v.bit_len(), "start={start} end={end} to={to}");
            if v.is_all_ones() {
                assert_eq!(to, v.bit_len(), "all ones: start={start} end={end}");
            } else {
                assert_eq!(
                    v.get(v.bit_len() - 1 - to),
                    Some(false),
                    "start={start} end={end} to={to}"
                );
            }
        }
    }
}
