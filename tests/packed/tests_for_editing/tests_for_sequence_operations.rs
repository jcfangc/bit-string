use super::*;

#[test]
fn packed_reverse_reverses_characters_without_mutating_source() {
    let original: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();
    let string = PackedString::from_chars(original.clone());
    let reversed = string.reverse();

    let mut expected = original.clone();
    expected.reverse();
    assert_eq!(reversed.to_vec(), expected);
    assert_eq!(string.to_vec(), original);
    assert_eq!(reversed.char_len(), expected.len());
    assert_eq!(reversed.bits().bit_len(), expected.len() * 3);

    let wide_values: Vec<_> = (0..10)
        .map(|index| WideCode((index * 17) as u8 % 128))
        .collect();
    let wide = PackedString::from_chars(wide_values.clone());
    let wide_reversed = wide.reverse();
    let mut wide_expected = wide_values.clone();
    wide_expected.reverse();
    assert_eq!(wide_reversed.to_vec(), wide_expected);
    assert_eq!(wide.to_vec(), wide_values);
    assert_eq!(wide_reversed.bits().bit_len(), wide_expected.len() * 7);

    assert!(PackedString::<Oct, 3>::new().reverse().is_empty());
    assert_eq!(
        PackedString::from_chars([Oct::V6]).reverse().to_vec(),
        vec![Oct::V6]
    );
}

#[test]
fn packed_reverse_assign_matches_reverse_and_preserves_packed_invariants() {
    let original: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();
    let expected = PackedString::from_chars(original.clone()).reverse();
    let mut string = PackedString::from_chars(original);
    string.reverse_assign();

    assert_eq!(string.to_vec(), expected.to_vec());
    assert_eq!(string.bits().words(), expected.bits().words());
    assert_eq!(string.char_len(), expected.char_len());
    assert_eq!(string.bits().bit_len(), string.char_len() * 3);

    let wide_values: Vec<_> = (0..10)
        .map(|index| WideCode((index * 17) as u8 % 128))
        .collect();
    let wide_expected = PackedString::from_chars(wide_values.clone()).reverse();
    let mut wide = PackedString::from_chars(wide_values);
    wide.reverse_assign();
    assert_eq!(wide.to_vec(), wide_expected.to_vec());
    assert_eq!(wide.bits().bit_len(), wide.char_len() * 7);

    let mut empty = PackedString::<Oct, 3>::new();
    empty.reverse_assign();
    assert!(empty.is_empty());
}

#[test]
fn packed_truncate_keeps_the_prefix_and_clamps_at_current_length() {
    let original: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();
    let mut string = PackedString::from_chars(original.clone());
    let original_bits = string.bits().clone();

    string.truncate(21);
    assert_eq!(string.to_vec(), original[..21].to_vec());
    assert_eq!(string.char_len(), 21);
    assert_eq!(string.bits().bit_len(), 21 * 3);

    let truncated_bits = string.bits().clone();
    string.truncate(usize::MAX);
    assert_eq!(string.bits().words(), truncated_bits.words());
    assert_eq!(string.to_vec(), original[..21].to_vec());

    string.truncate(0);
    assert!(string.is_empty());
    assert!(string.bits().words().is_empty());

    let wide_values: Vec<_> = (0..10)
        .map(|index| WideCode((index * 17) as u8 % 128))
        .collect();
    let mut wide = PackedString::from_chars(wide_values.clone());
    wide.truncate(9);
    assert_eq!(wide.to_vec(), wide_values[..9].to_vec());
    assert_eq!(wide.bits().bit_len(), 9 * 7);

    let mut unchanged = PackedString::from_chars(original);
    unchanged.truncate(22);
    assert_eq!(unchanged.bits().words(), original_bits.words());
}

#[test]
fn packed_push_appends_one_aligned_code_at_a_time() {
    let mut oct_string = PackedString::<Oct, 3>::new();
    let mut oracle = Vec::new();
    let values: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();

    for value in values.iter().copied() {
        let old_prefix = oct_string.to_vec();
        let old_bit_len = oct_string.bits().bit_len();
        oct_string.push(value);
        oracle.push(value);
        assert_eq!(oct_string.to_vec(), oracle);
        assert_eq!(oct_string.bits().bit_len(), old_bit_len + 3);
        assert_eq!(oct_string.to_vec()[..old_prefix.len()], old_prefix);
    }
    assert_eq!(oct_string.get(21), Some(Oct::V5));
    assert_eq!(oct_string.char_len(), 22);

    oct_string.push(Oct::V0);
    oracle.push(Oct::V0);
    assert_eq!(oct_string.to_vec(), oracle);
    assert_eq!(oct_string.bits().bit_len(), 23 * 3);
    assert_eq!(oct_string.get(22), Some(Oct::V0));
    assert!(!oct_string.is_empty());

    let mut binary = PackedString::<PackedSymbol, 1>::new();
    binary.push(PackedSymbol::One);
    binary.push(PackedSymbol::Zero);
    assert_eq!(binary.to_vec(), vec![PackedSymbol::One, PackedSymbol::Zero]);
    assert_eq!(binary.bits().bit_len(), 2);

    let mut bytes = PackedString::<super::SparseByte, 8>::new();
    bytes.push(super::SparseByte::Zero);
    bytes.push(super::SparseByte::Maximum);
    assert_eq!(
        bytes.to_vec(),
        vec![super::SparseByte::Zero, super::SparseByte::Maximum]
    );
    assert_eq!(bytes.bits().bit_len(), 16);
}

#[test]
fn packed_views_and_editing_remain_character_aligned() {
    let mut string = PackedString::<super::Symbol, 2>::from_chars([
        super::Symbol::Zero,
        super::Symbol::One,
        super::Symbol::Two,
        super::Symbol::One,
    ]);

    let view = string.as_packed_str();
    assert_eq!(view.char_len(), 4);
    assert_eq!(
        view.slice(UsizeCO::checked_from_start_len(1, 2).unwrap())
            .get(0),
        Some(super::Symbol::One)
    );
    assert_eq!(view.find(view.slice_until(1)), Some(0));
    assert!(view.contains(view.slice(UsizeCO::checked_from_start_len(1, 1).unwrap())));

    let haystack_string = PackedString::<super::Symbol, 2>::from_chars([
        super::Symbol::Zero,
        super::Symbol::Zero,
        super::Symbol::One,
    ]);
    let needle_string = PackedString::<super::Symbol, 2>::from_chars([super::Symbol::Two]);
    let haystack = haystack_string.as_packed_str();
    let needle = needle_string.as_packed_str();
    assert!(!haystack.contains(needle));
    assert_eq!(haystack.find(needle), None);
    assert_eq!(haystack.rfind(needle), None);
    assert!(!haystack.matches_at(1, needle));

    string.insert(2, super::Symbol::Zero);
    assert_eq!(string.remove(2), super::Symbol::Zero);
    assert_eq!(
        string.reverse().to_vec(),
        vec![
            super::Symbol::One,
            super::Symbol::Two,
            super::Symbol::One,
            super::Symbol::Zero
        ]
    );
    string.retain(|symbol| symbol != super::Symbol::Two);
    assert_eq!(
        string.to_vec(),
        vec![super::Symbol::Zero, super::Symbol::One, super::Symbol::One]
    );
}

#[test]
fn packed_string_set_replaces_only_one_character_slot() {
    let mut string = packed(&[0, 1, 2, 1, 0]);
    let original_bits = string.bits().clone();
    assert_eq!(string.set(0, super::Symbol::Two), Some(super::Symbol::Zero));
    assert_eq!(string.get(0), Some(super::Symbol::Two));
    assert_eq!(string.get(1), Some(super::Symbol::One));
    assert_eq!(string.get(4), Some(super::Symbol::Zero));
    assert_eq!(string.char_len(), 5);
    assert_eq!(string.bits().bit_len(), original_bits.bit_len());

    assert_eq!(
        string.set(4, super::Symbol::Zero),
        Some(super::Symbol::Zero)
    );
    assert_eq!(string.set(99, super::Symbol::One), None);
    assert_eq!(string.get(0), Some(super::Symbol::Two));
    assert_eq!(string.get(4), Some(super::Symbol::Zero));

    let mut binary = packed_as::<super::PackedSymbol, 1>(&[0, 1, 0], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(
        binary.set(1, super::PackedSymbol::Zero),
        Some(super::PackedSymbol::One)
    );
    assert_eq!(binary.to_vec(), vec![super::PackedSymbol::Zero; 3]);
    assert_eq!(binary.bits().bit_len(), 3);

    let mut bytes = packed_as::<super::SparseByte, 8>(&[0, 255, 3], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        3 => super::SparseByte::Middle,
        _ => unreachable!(),
    });
    assert_eq!(
        bytes.set(1, super::SparseByte::Zero),
        Some(super::SparseByte::Maximum)
    );
    assert_eq!(
        bytes.to_vec(),
        vec![
            super::SparseByte::Zero,
            super::SparseByte::Zero,
            super::SparseByte::Middle,
        ]
    );
    assert_eq!(bytes.bits().bit_len(), 24);

    let oct_codes: Vec<u8> = (0..22).map(|index| index as u8 % 8).collect();
    let mut oct_string = packed_as::<Oct, 3>(&oct_codes, super::oct);
    assert_eq!(oct_string.set(21, Oct::V7), Some(Oct::V5));
    assert_eq!(oct_string.get(20), Some(Oct::V4));
    assert_eq!(oct_string.get(21), Some(Oct::V7));
    assert_eq!(oct_string.bits().bit_len(), 66);

    let wide_codes: Vec<u8> = (0..10).map(|index| (index * 11) as u8 % 128).collect();
    let mut wide_string = packed_as::<WideCode, 7>(&wide_codes, super::wide);
    assert_eq!(wide_string.set(9, WideCode(127)), Some(WideCode(99)));
    assert_eq!(wide_string.get(8), Some(WideCode(88)));
    assert_eq!(wide_string.get(9), Some(WideCode(127)));
    assert_eq!(wide_string.bits().bit_len(), 70);
}
