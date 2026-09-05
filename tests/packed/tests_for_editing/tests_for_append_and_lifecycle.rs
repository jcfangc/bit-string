use super::{Oct, WideCode, packed_as};
use bit_string::{BitString, PackedString, traits::PackedChar};

#[test]
fn packed_out_of_bounds_intervals_preserve_empty_semantics() {
    let string = super::packed(&[0, 1, 2]);
    let out_of_bounds = int_intervals::UsizeCO::checked_from_start_len(99, 1).unwrap();

    assert!(string.slice(out_of_bounds).is_empty());

    let mut drained = string.clone();
    drained.drain_interval_assign(out_of_bounds);
    assert_eq!(drained, string);
}

#[test]
fn packed_clear_removes_all_codes_and_allows_reuse() {
    let mut empty = PackedString::<super::Symbol, 2>::new();
    empty.clear();
    empty.clear();
    assert!(empty.is_empty());
    assert_eq!(empty.char_len(), 0);
    assert_eq!(empty.bits().bit_len(), 0);
    assert!(empty.bits().words().is_empty());
    assert!(empty.to_vec().is_empty());
    assert_eq!(empty.get(0), None);
    assert_eq!(empty.first(), None);
    assert_eq!(empty.last(), None);
    assert_eq!(empty.as_packed_str().iter().next(), None);

    let mut zero = super::packed_as::<super::SparseByte, 8>(&[0], |code| match code {
        0 => super::SparseByte::Zero,
        _ => unreachable!(),
    });
    assert!(!zero.is_empty());
    assert_eq!(zero.bits().words(), &[0]);
    zero.clear();
    assert!(zero.is_empty());
    assert!(zero.bits().words().is_empty());

    let oct_codes: Vec<_> = (0..22).map(|index| index as u8 % 8).collect();
    let mut oct_string = packed_as::<Oct, 3>(&oct_codes, super::oct);
    oct_string.clear();
    assert_eq!(oct_string.char_len(), 0);
    assert_eq!(oct_string.bits().bit_len(), 0);
    assert!(oct_string.bits().words().is_empty());
    oct_string.push(Oct::V7);
    assert_eq!(oct_string.to_vec(), vec![Oct::V7]);
    assert_eq!(oct_string.bits().bit_len(), 3);

    let wide_codes: Vec<_> = (0..10).map(|index| (index * 11) as u8 % 128).collect();
    let mut wide_string = packed_as::<WideCode, 7>(&wide_codes, super::wide);
    wide_string.clear();
    wide_string.push(WideCode(127));
    assert_eq!(wide_string.to_vec(), vec![WideCode(127)]);
    assert_eq!(wide_string.bits().bit_len(), 7);
}

#[test]
fn packed_extend_copies_borrowed_codes_in_order() {
    let mut oct_string = packed_as::<Oct, 3>(&[Oct::V7.code()], super::oct);
    let mut oct_source: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();
    let mut oct_expected = oct_string.to_vec();
    oct_expected.extend(oct_source.iter().copied());
    oct_string.extend(oct_source.iter());
    assert_eq!(oct_string.to_vec(), oct_expected);
    assert_eq!(oct_string.char_len(), oct_expected.len());
    assert_eq!(oct_string.bits().bit_len(), oct_expected.len() * 3);

    oct_source[0] = Oct::V0;
    oct_source.clear();
    assert_eq!(oct_string.to_vec(), oct_expected);

    let mut wide_string = packed_as::<WideCode, 7>(&[126], super::wide);
    let mut wide_source: Vec<_> = (0..10)
        .map(|index| {
            if index == 9 {
                WideCode(127)
            } else {
                WideCode((index * 11) as u8 % 128)
            }
        })
        .collect();
    let mut wide_expected = wide_string.to_vec();
    wide_expected.extend(wide_source.iter().copied());
    wide_string.extend(wide_source.iter());
    assert_eq!(wide_string.to_vec(), wide_expected);
    assert_eq!(wide_string.bits().bit_len(), wide_expected.len() * 7);

    wide_source.clear();
    wide_string.extend(wide_source.iter());
    assert_eq!(wide_string.to_vec(), wide_expected);

    let mut bytes = PackedString::<super::SparseByte, 8>::new();
    let byte_source = [
        super::SparseByte::Zero,
        super::SparseByte::Maximum,
        super::SparseByte::Middle,
    ];
    bytes.extend(byte_source.iter());
    assert_eq!(bytes.to_vec(), byte_source);
    assert_eq!(bytes.bits().bit_len(), 24);
}

#[test]
fn packed_extend_appends_owned_codes_without_changing_the_prefix() {
    let mut oct_string = packed_as::<Oct, 3>(&[7; 20], super::oct);
    let original = oct_string.to_vec();
    oct_string.extend(core::iter::empty::<Oct>());
    assert_eq!(oct_string.to_vec(), original);

    let appended = [Oct::V0, Oct::V7, Oct::V3, Oct::V4, Oct::V1, Oct::V6];
    let mut expected = original.clone();
    expected.extend(appended);
    oct_string.extend(appended);
    assert_eq!(oct_string.to_vec(), expected);
    assert_eq!(oct_string.char_len(), expected.len());
    assert_eq!(oct_string.bits().bit_len(), expected.len() * 3);

    let expected_bits = BitString::from_iter(
        expected
            .iter()
            .flat_map(|character| (0..3).map(move |offset| character.code() & (1 << offset) != 0)),
    );
    assert_eq!(oct_string.bits().words(), expected_bits.words());

    let mut wide_string = PackedString::<WideCode, 7>::new();
    let wide_values = [WideCode(0), WideCode(127), WideCode(64), WideCode(1)];
    wide_string.extend(wide_values);
    wide_string.extend([WideCode(126), WideCode(2), WideCode(127)]);
    assert_eq!(
        wide_string.to_vec(),
        vec![
            WideCode(0),
            WideCode(127),
            WideCode(64),
            WideCode(1),
            WideCode(126),
            WideCode(2),
            WideCode(127),
        ]
    );
    assert_eq!(wide_string.bits().bit_len(), 7 * 7);
}

#[test]
fn packed_pop_returns_final_codes_and_shrinks_to_empty() {
    let mut empty = PackedString::<Oct, 3>::new();
    assert_eq!(empty.pop(), None);
    assert!(empty.bits().words().is_empty());

    let mut maximum =
        PackedString::<super::SparseByte, 8>::from_chars([super::SparseByte::Maximum]);
    assert_eq!(maximum.pop(), Some(super::SparseByte::Maximum));
    assert!(maximum.is_empty());
    assert_eq!(maximum.pop(), None);

    let mut oct_codes: Vec<_> = (0..23)
        .map(|index| if index == 22 { 0 } else { index as u8 % 8 })
        .map(super::oct)
        .collect();
    let mut oct_string = PackedString::from_chars(oct_codes.clone());
    while let Some(expected) = oct_codes.pop() {
        assert_eq!(oct_string.pop(), Some(expected));
        assert_eq!(oct_string.to_vec(), oct_codes);
        assert_eq!(oct_string.char_len(), oct_codes.len());
        assert_eq!(oct_string.bits().bit_len(), oct_codes.len() * 3);
        assert_eq!(
            oct_string.bits().words().len(),
            (oct_codes.len() * 3).div_ceil(64)
        );
        assert_eq!(oct_string.last(), oct_codes.last().copied());
    }
    assert_eq!(oct_string.pop(), None);

    let mut wide_codes: Vec<_> = (0..10)
        .map(|index| {
            if index == 9 {
                127
            } else {
                (index * 11) as u8 % 128
            }
        })
        .map(super::wide)
        .collect();
    let mut wide_string = PackedString::from_chars(wide_codes.clone());
    while let Some(expected) = wide_codes.pop() {
        assert_eq!(wide_string.pop(), Some(expected));
        assert_eq!(wide_string.last(), wide_codes.last().copied());
        assert_eq!(wide_string.bits().bit_len(), wide_codes.len() * 7);
    }
    assert!(wide_string.is_empty());
}

#[test]
fn packed_push_packed_string_concatenates_raw_payloads() {
    let mut oct_left = packed_as::<Oct, 3>(&[7; 21], super::oct);
    let oct_right_codes = [0, 7, 3];
    let oct_right = packed_as::<Oct, 3>(&oct_right_codes, super::oct);
    let oct_right_snapshot = oct_right.clone();
    let mut oct_oracle = oct_left.to_vec();
    oct_oracle.extend(oct_right.to_vec());

    oct_left.push_packed_string(&oct_right);
    assert_eq!(oct_left.to_vec(), oct_oracle);
    assert_eq!(oct_left.char_len(), 24);
    assert_eq!(oct_left.bits().bit_len(), 24 * 3);
    assert!(oct_right == oct_right_snapshot);
    assert_eq!(oct_left.get(21), Some(Oct::V0));
    assert_eq!(oct_left.get(22), Some(Oct::V7));
    assert_eq!(oct_left.get(23), Some(Oct::V3));

    let expected_bits = BitString::from_iter(
        oct_oracle
            .iter()
            .flat_map(|character| (0..3).map(move |offset| character.code() & (1 << offset) != 0)),
    );
    assert_eq!(oct_left.bits().words(), expected_bits.words());

    let empty = PackedString::<Oct, 3>::new();
    let before_empty_append = oct_left.clone();
    oct_left.push_packed_string(&empty);
    assert!(oct_left == before_empty_append);

    let original = packed_as::<Oct, 3>(&[0, 7, 3, 1], super::oct);
    let mut duplicated = original.clone();
    let source = original.clone();
    duplicated.push_packed_string(&source);
    assert_eq!(
        duplicated.to_vec(),
        [0, 7, 3, 1, 0, 7, 3, 1].map(super::oct)
    );

    let mut wide_left = packed_as::<WideCode, 7>(&[126; 9], super::wide);
    let wide_right = packed_as::<WideCode, 7>(&[0, 127, 64], super::wide);
    wide_left.push_packed_string(&wide_right);
    assert_eq!(wide_left.char_len(), 12);
    assert_eq!(wide_left.bits().bit_len(), 12 * 7);
    assert_eq!(wide_left.get(9), Some(WideCode(0)));
    assert_eq!(wide_left.get(10), Some(WideCode(127)));
    assert_eq!(wide_left.get(11), Some(WideCode(64)));

    let mut bytes = PackedString::<super::SparseByte, 8>::new();
    let byte_right =
        PackedString::from_chars([super::SparseByte::Zero, super::SparseByte::Maximum]);
    bytes.push_packed_string(&byte_right);
    assert_eq!(
        bytes.to_vec(),
        vec![super::SparseByte::Zero, super::SparseByte::Maximum]
    );
}
