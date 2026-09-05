use super::*;

#[test]
fn packed_replace_splices_characters_without_mutating_the_source() {
    let original_codes: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();
    let original = PackedString::from_chars(original_codes.clone());
    let replacement = PackedString::from_chars([Oct::V7, Oct::V0, Oct::V3]);

    let mut expected = original_codes.clone();
    expected.splice(10..13, [Oct::V7, Oct::V0, Oct::V3]);
    let replaced = original.replace(10, &replacement);
    assert_eq!(replaced.to_vec(), expected);
    assert_eq!(replaced.bits().bit_len(), expected.len() * 3);
    assert_eq!(original.to_vec(), original_codes);

    let mut appended = original_codes.clone();
    appended.extend([Oct::V7, Oct::V0, Oct::V3]);
    assert_eq!(
        original.replace(usize::MAX, &replacement).to_vec(),
        appended
    );

    let empty_replacement = PackedString::<Oct, 3>::new();
    assert!(original.replace(5, &empty_replacement) == original);

    let wide_original = PackedString::from_chars([
        WideCode(0),
        WideCode(11),
        WideCode(22),
        WideCode(33),
        WideCode(44),
        WideCode(55),
        WideCode(66),
        WideCode(77),
        WideCode(88),
    ]);
    let wide_replacement = PackedString::from_chars([WideCode(127), WideCode(1), WideCode(64)]);
    let wide_result = wide_original.replace(8, &wide_replacement);
    assert_eq!(
        wide_result.to_vec(),
        vec![
            WideCode(0),
            WideCode(11),
            WideCode(22),
            WideCode(33),
            WideCode(44),
            WideCode(55),
            WideCode(66),
            WideCode(77),
            WideCode(127),
            WideCode(1),
            WideCode(64),
        ]
    );
    assert_eq!(wide_result.bits().bit_len(), 11 * 7);
}

#[test]
fn packed_replace_assign_matches_replace_in_place() {
    let original = packed_as::<Oct, 3>(&[0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3], super::oct);
    let replacement = packed_as::<Oct, 3>(
        &[Oct::V7.code(), Oct::V0.code(), Oct::V3.code()],
        super::oct,
    );

    let expected = original.replace(8, &replacement);
    let mut assigned = original.clone();
    assigned.replace_assign(8, &replacement);
    assert!(assigned == expected);
    assert_eq!(assigned.bits().words(), expected.bits().words());

    let mut appended = original.clone();
    appended.replace_assign(usize::MAX, &replacement);
    assert_eq!(appended.to_vec(), {
        let mut values = original.to_vec();
        values.extend(replacement.to_vec());
        values
    });

    let empty = PackedString::<Oct, 3>::new();
    let before = assigned.clone();
    assigned.replace_assign(4, &empty);
    assert!(assigned == before);

    let wide = packed_as::<WideCode, 7>(&[0; 9], super::wide);
    let wide_replacement = packed_as::<WideCode, 7>(&[127, 64, 1], super::wide);
    let wide_expected = wide.replace(8, &wide_replacement);
    let mut wide_assigned = wide.clone();
    wide_assigned.replace_assign(8, &wide_replacement);
    assert!(wide_assigned == wide_expected);
    assert_eq!(wide_assigned.bits().bit_len(), 11 * 7);
}

#[test]
fn packed_replace_interval_splices_clamped_character_ranges() {
    let original_codes: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();
    let original = PackedString::from_chars(original_codes.clone());
    let replacement = PackedString::from_chars([Oct::V7, Oct::V0, Oct::V3]);

    let interval = UsizeCO::checked_from_start_len(9, 5).unwrap();
    let mut expected = original_codes.clone();
    expected.splice(9..14, replacement.to_vec());
    let result = original.replace_interval(interval, &replacement);
    assert_eq!(result.to_vec(), expected);
    assert_eq!(result.bits().bit_len(), expected.len() * 3);
    assert_eq!(original.to_vec(), original_codes);

    let tail_interval = UsizeCO::checked_from_start_len(21, 2).unwrap();
    let tail_result = original.replace_interval(tail_interval, &replacement);
    let mut tail_expected = original_codes.clone();
    tail_expected.splice(21..22, replacement.to_vec());
    assert_eq!(tail_result.to_vec(), tail_expected);
    assert_eq!(tail_result.bits().bit_len(), tail_expected.len() * 3);

    let out_of_bounds = UsizeCO::checked_from_start_len(99, 4).unwrap();
    let out_result = original.replace_interval(out_of_bounds, &replacement);
    let mut out_expected = original_codes.clone();
    out_expected.extend(replacement.to_vec());
    assert_eq!(out_result.to_vec(), out_expected);

    let empty = PackedString::<Oct, 3>::new();
    let remove_result =
        original.replace_interval(UsizeCO::checked_from_start_len(20, 2).unwrap(), &empty);
    assert_eq!(remove_result.to_vec(), original_codes[..20].to_vec());

    let wide_original = PackedString::from_chars([
        WideCode(0),
        WideCode(11),
        WideCode(22),
        WideCode(33),
        WideCode(44),
        WideCode(55),
        WideCode(66),
        WideCode(77),
        WideCode(88),
        WideCode(99),
    ]);
    let wide_replacement = PackedString::from_chars([WideCode(127), WideCode(1)]);
    let wide_result = wide_original.replace_interval(
        UsizeCO::checked_from_start_len(8, 2).unwrap(),
        &wide_replacement,
    );
    assert_eq!(
        wide_result.to_vec(),
        vec![
            WideCode(0),
            WideCode(11),
            WideCode(22),
            WideCode(33),
            WideCode(44),
            WideCode(55),
            WideCode(66),
            WideCode(77),
            WideCode(127),
            WideCode(1),
        ]
    );
    assert_eq!(wide_result.bits().bit_len(), 10 * 7);
}

#[test]
fn packed_replace_interval_assign_matches_vec_splice_semantics() {
    let original_codes: Vec<_> = (0..22).map(|index| index as u8 % 8).collect();
    let replacement_codes = vec![7, 0, 3];
    let interval = UsizeCO::checked_from_start_len(9, 5).unwrap();
    let replacement = packed_as::<Oct, 3>(&replacement_codes, super::oct);
    let mut string = packed_as::<Oct, 3>(&original_codes, super::oct);
    let mut oracle = original_codes.clone();

    string.replace_interval_assign(interval, &replacement);
    oracle.splice(9..14, replacement_codes.iter().copied());
    assert_eq!(
        string
            .to_vec()
            .iter()
            .map(|value| value.code())
            .collect::<Vec<_>>(),
        oracle
    );
    assert_eq!(string.bits().bit_len(), oracle.len() * 3);

    let empty = PackedString::<Oct, 3>::new();
    string.replace_interval_assign(UsizeCO::checked_from_start_len(4, 2).unwrap(), &empty);
    oracle.splice(4..6, core::iter::empty());
    assert_eq!(
        string
            .to_vec()
            .iter()
            .map(|value| value.code())
            .collect::<Vec<_>>(),
        oracle
    );
    assert_eq!(string.bits().bit_len(), oracle.len() * 3);

    string.replace_interval_assign(
        UsizeCO::checked_from_start_len(99, 4).unwrap(),
        &replacement,
    );
    oracle.extend(replacement_codes.iter().copied());
    assert_eq!(
        string
            .to_vec()
            .iter()
            .map(|value| value.code())
            .collect::<Vec<_>>(),
        oracle
    );

    let wide_replacement = packed_as::<WideCode, 7>(&[127, 1], super::wide);
    let mut wide = packed_as::<WideCode, 7>(&[0; 10], super::wide);
    wide.replace_interval_assign(
        UsizeCO::checked_from_start_len(8, 2).unwrap(),
        &wide_replacement,
    );
    assert_eq!(
        wide.to_vec(),
        vec![
            WideCode(0),
            WideCode(0),
            WideCode(0),
            WideCode(0),
            WideCode(0),
            WideCode(0),
            WideCode(0),
            WideCode(0),
            WideCode(127),
            WideCode(1),
        ]
    );
    assert_eq!(wide.bits().bit_len(), 10 * 7);
}
