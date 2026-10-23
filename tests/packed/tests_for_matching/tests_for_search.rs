use super::*;

#[test]
fn packed_str_contains_searches_character_aligned_windows() {
    let haystack_owner = packed(&[0, 0, 0]);
    let haystack = haystack_owner.as_packed_str();
    let repeated = packed(&[0, 0]);
    let final_haystack_owner = packed(&[1, 2, 0]);
    let final_haystack = final_haystack_owner.as_packed_str();
    let final_window = packed(&[2, 0]);
    let different = packed(&[1, 2]);
    let oversized = packed(&[0, 0, 0, 1]);

    assert!(haystack.contains_with(
        repeated.as_packed_str(),
        bit_string::ContainsOptions::default()
    ));
    assert!(final_haystack.contains_with(
        final_window.as_packed_str(),
        bit_string::ContainsOptions::default()
    ));
    assert!(!haystack.contains_with(
        different.as_packed_str(),
        bit_string::ContainsOptions::default()
    ));
    assert!(!haystack.contains_with(
        oversized.as_packed_str(),
        bit_string::ContainsOptions::default()
    ));
    assert!(haystack.contains_with(
        haystack.slice_until(0),
        bit_string::ContainsOptions::default()
    ));

    let empty_owner = packed(&[]);
    let empty = empty_owner.as_packed_str();
    let singleton_owner = packed(&[0]);
    assert!(empty.contains_with(empty, bit_string::ContainsOptions::default()));
    assert!(!empty.contains_with(
        singleton_owner.as_packed_str(),
        bit_string::ContainsOptions::default()
    ));

    let offset_owner = packed(&[0, 1, 2, 1, 0]);
    let offset_haystack = offset_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_needle = offset_haystack.slice_from(1);
    let offset_different_owner = packed(&[2, 2]);
    let offset_different = offset_different_owner.as_packed_str();
    assert!(offset_haystack.contains_with(offset_needle, bit_string::ContainsOptions::default()));
    assert!(
        !offset_haystack.contains_with(offset_different, bit_string::ContainsOptions::default())
    );

    let binary = packed_as::<super::PackedSymbol, 1>(&[0, 1, 1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert!(binary.as_packed_str().contains_with(
        binary.as_packed_str().slice_from(1),
        bit_string::ContainsOptions::default()
    ));

    let bytes = packed_as::<super::SparseByte, 8>(&[0, 255], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        _ => unreachable!(),
    });
    assert!(bytes.as_packed_str().contains_with(
        bytes.as_packed_str().slice_from(1),
        bit_string::ContainsOptions::default()
    ));

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_haystack = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    let oct_different_owner = packed_as::<Oct, 3>(&[5, 4], oct);
    assert!(oct_haystack.contains_with(
        oct_haystack.slice_from(1),
        bit_string::ContainsOptions::default()
    ));
    assert!(!oct_haystack.contains_with(
        oct_different_owner.as_packed_str(),
        bit_string::ContainsOptions::default()
    ));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_haystack = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    let wide_different_owner = packed_as::<WideCode, 7>(&[11, 0], wide);
    assert!(wide_haystack.contains_with(
        wide_haystack.slice_from(1),
        bit_string::ContainsOptions::default()
    ));
    assert!(!wide_haystack.contains_with(
        wide_different_owner.as_packed_str(),
        bit_string::ContainsOptions::default()
    ));
}

#[test]
fn packed_string_contains_searches_character_aligned_windows() {
    let haystack = packed(&[0, 1, 0, 1, 2, 1]);
    assert!(haystack.contains_with(&packed(&[0, 1, 2]), bit_string::ContainsOptions::default()));
    assert!(haystack.contains_with(&packed(&[1, 2, 1]), bit_string::ContainsOptions::default()));
    assert!(!haystack.contains_with(&packed(&[2, 0]), bit_string::ContainsOptions::default()));
    assert!(!haystack.contains_with(
        &packed(&[0, 1, 0, 1, 2, 1, 0]),
        bit_string::ContainsOptions::default()
    ));
    assert!(haystack.contains_with(
        &PackedString::<super::Symbol, 2>::new(),
        bit_string::ContainsOptions::default()
    ));

    let empty = PackedString::<super::Symbol, 2>::new();
    assert!(!empty.contains_with(&packed(&[0]), bit_string::ContainsOptions::default()));

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_haystack = packed_as::<Oct, 3>(&oct_codes, oct);
    assert!(oct_haystack.contains_with(
        &packed_as::<Oct, 3>(&[4, 5, 6, 7], oct),
        bit_string::ContainsOptions::default()
    ));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_haystack = packed_as::<WideCode, 7>(&wide_codes, wide);
    assert!(wide_haystack.contains_with(
        &packed_as::<WideCode, 7>(&[88, 99], wide),
        bit_string::ContainsOptions::default()
    ));
}

#[test]
fn packed_string_find_returns_the_earliest_character_index() {
    let haystack = packed(&[0, 1, 0, 1, 2, 1]);
    assert_eq!(
        haystack.find_with(&packed(&[0, 1]), bit_string::FindOptions::default()),
        Some(0)
    );
    assert_eq!(
        haystack.find_with(&packed(&[1, 2]), bit_string::FindOptions::default()),
        Some(3)
    );
    assert_eq!(
        haystack.find_with(&packed(&[2, 0]), bit_string::FindOptions::default()),
        None
    );
    assert_eq!(
        haystack.find_with(
            &packed(&[0, 1, 0, 1, 2, 1, 0]),
            bit_string::FindOptions::default()
        ),
        None
    );
    assert_eq!(
        haystack.find_with(
            &PackedString::<super::Symbol, 2>::new(),
            bit_string::FindOptions::default()
        ),
        Some(0)
    );

    let empty = PackedString::<super::Symbol, 2>::new();
    assert_eq!(
        empty.find_with(&empty, bit_string::FindOptions::default()),
        Some(0)
    );
    assert_eq!(
        empty.find_with(&packed(&[0]), bit_string::FindOptions::default()),
        None
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_haystack = packed_as::<Oct, 3>(&oct_codes, oct);
    assert_eq!(
        oct_haystack.find_with(
            &packed_as::<Oct, 3>(&[4, 5, 6], oct),
            bit_string::FindOptions::default()
        ),
        Some(4)
    );

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_haystack = packed_as::<WideCode, 7>(&wide_codes, wide);
    assert_eq!(
        wide_haystack.find_with(
            &packed_as::<WideCode, 7>(&[88, 99], wide),
            bit_string::FindOptions::default()
        ),
        Some(8)
    );
}

#[test]
fn packed_str_find_returns_the_earliest_character_index() {
    let repeated_owner = packed(&[0, 0, 0]);
    let repeated_needle = packed(&[0, 0]);
    assert_eq!(
        repeated_owner.as_packed_str().find_with(
            repeated_needle.as_packed_str(),
            bit_string::FindOptions::default()
        ),
        Some(0)
    );

    let final_owner = packed(&[1, 2, 0]);
    let final_needle = packed(&[2, 0]);
    assert_eq!(
        final_owner.as_packed_str().find_with(
            final_needle.as_packed_str(),
            bit_string::FindOptions::default()
        ),
        Some(1)
    );

    let no_match = packed(&[1, 2]);
    let no_match_needle = packed(&[0]);
    let oversized = packed(&[0, 1, 2]);
    assert_eq!(
        no_match.as_packed_str().find_with(
            no_match_needle.as_packed_str(),
            bit_string::FindOptions::default()
        ),
        None
    );
    assert_eq!(
        no_match.as_packed_str().find_with(
            oversized.as_packed_str(),
            bit_string::FindOptions::default()
        ),
        None
    );

    let empty_owner = packed(&[]);
    let empty = empty_owner.as_packed_str();
    assert_eq!(
        empty.find_with(empty, bit_string::FindOptions::default()),
        Some(0)
    );
    assert_eq!(
        final_owner
            .as_packed_str()
            .find_with(empty, bit_string::FindOptions::default()),
        Some(0)
    );

    let offset_owner = packed(&[0, 1, 2, 1, 0]);
    let offset_haystack = offset_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_needle = packed(&[1, 2]);
    assert_eq!(
        offset_haystack.find_with(
            offset_needle.as_packed_str(),
            bit_string::FindOptions::default()
        ),
        Some(0)
    );

    let binary = packed_as::<super::PackedSymbol, 1>(&[1, 0, 1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    let binary_needle = packed_as::<super::PackedSymbol, 1>(&[1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(
        binary.as_packed_str().find_with(
            binary_needle.as_packed_str(),
            bit_string::FindOptions::default()
        ),
        Some(0)
    );

    let bytes = packed_as::<super::SparseByte, 8>(&[0, 255, 3], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        3 => super::SparseByte::Middle,
        _ => unreachable!(),
    });
    let byte_needle = packed_as::<super::SparseByte, 8>(&[255, 3], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        3 => super::SparseByte::Middle,
        _ => unreachable!(),
    });
    assert_eq!(
        bytes.as_packed_str().find_with(
            byte_needle.as_packed_str(),
            bit_string::FindOptions::default()
        ),
        Some(1)
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_haystack = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    assert_eq!(
        oct_haystack.find_with(
            oct_haystack.slice_from(1),
            bit_string::FindOptions::default()
        ),
        Some(1)
    );

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_haystack = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    assert_eq!(
        wide_haystack.find_with(
            wide_haystack.slice_from(1),
            bit_string::FindOptions::default()
        ),
        Some(1)
    );

    assert_eq!(offset_haystack.char_len(), 3);
    assert_eq!(offset_haystack.get(0), Some(super::Symbol::One));
    assert_eq!(offset_haystack.get(2), Some(super::Symbol::One));
}

#[test]
fn packed_str_rfind_returns_the_latest_character_index() {
    let repeated_owner = packed(&[0, 0, 0]);
    let repeated_needle = packed(&[0, 0]);
    assert_eq!(
        repeated_owner.as_packed_str().rfind_with(
            repeated_needle.as_packed_str(),
            bit_string::FindOptions::default()
        ),
        Some(1)
    );

    let final_owner = packed(&[1, 2, 0]);
    let final_needle = packed(&[2, 0]);
    assert_eq!(
        final_owner.as_packed_str().rfind_with(
            final_needle.as_packed_str(),
            bit_string::FindOptions::default()
        ),
        Some(1)
    );

    let no_match = packed(&[1, 2]);
    let no_match_needle = packed(&[0]);
    let oversized = packed(&[0, 1, 2]);
    assert_eq!(
        no_match.as_packed_str().rfind_with(
            no_match_needle.as_packed_str(),
            bit_string::FindOptions::default()
        ),
        None
    );
    assert_eq!(
        no_match.as_packed_str().rfind_with(
            oversized.as_packed_str(),
            bit_string::FindOptions::default()
        ),
        None
    );

    let empty_owner = packed(&[]);
    let empty = empty_owner.as_packed_str();
    assert_eq!(
        empty.rfind_with(empty, bit_string::FindOptions::default()),
        Some(0)
    );
    assert_eq!(
        final_owner
            .as_packed_str()
            .rfind_with(empty, bit_string::FindOptions::default()),
        Some(3)
    );

    let offset_owner = packed(&[0, 1, 2, 1, 0]);
    let offset_haystack = offset_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_needle = packed(&[1]);
    assert_eq!(
        offset_haystack.rfind_with(
            offset_needle.as_packed_str(),
            bit_string::FindOptions::default()
        ),
        Some(2)
    );

    let binary = packed_as::<super::PackedSymbol, 1>(&[1, 0, 1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    let binary_needle = packed_as::<super::PackedSymbol, 1>(&[1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(
        binary.as_packed_str().rfind_with(
            binary_needle.as_packed_str(),
            bit_string::FindOptions::default()
        ),
        Some(2)
    );

    let bytes = packed_as::<super::SparseByte, 8>(&[0, 255, 3], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        3 => super::SparseByte::Middle,
        _ => unreachable!(),
    });
    let byte_needle = packed_as::<super::SparseByte, 8>(&[255, 3], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        3 => super::SparseByte::Middle,
        _ => unreachable!(),
    });
    assert_eq!(
        bytes.as_packed_str().rfind_with(
            byte_needle.as_packed_str(),
            bit_string::FindOptions::default()
        ),
        Some(1)
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_haystack = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    assert_eq!(
        oct_haystack.rfind_with(
            oct_haystack.slice_from(1),
            bit_string::FindOptions::default()
        ),
        Some(1)
    );

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_haystack = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    assert_eq!(
        wide_haystack.rfind_with(
            wide_haystack.slice_from(1),
            bit_string::FindOptions::default()
        ),
        Some(1)
    );

    assert_eq!(offset_haystack.char_len(), 3);
    assert_eq!(offset_haystack.get(0), Some(super::Symbol::One));
    assert_eq!(offset_haystack.get(2), Some(super::Symbol::One));
}

#[test]
fn packed_string_rfind_returns_the_latest_character_index() {
    let haystack = packed(&[0, 1, 0, 1, 2, 1]);
    assert_eq!(
        haystack.rfind_with(&packed(&[0, 1]), bit_string::FindOptions::default()),
        Some(2)
    );
    assert_eq!(
        haystack.rfind_with(&packed(&[1, 2]), bit_string::FindOptions::default()),
        Some(3)
    );
    assert_eq!(
        haystack.rfind_with(&packed(&[2, 0]), bit_string::FindOptions::default()),
        None
    );
    assert_eq!(
        haystack.rfind_with(
            &packed(&[0, 1, 0, 1, 2, 1, 0]),
            bit_string::FindOptions::default()
        ),
        None
    );
    assert_eq!(
        haystack.rfind_with(
            &PackedString::<super::Symbol, 2>::new(),
            bit_string::FindOptions::default()
        ),
        Some(6)
    );

    let empty = PackedString::<super::Symbol, 2>::new();
    assert_eq!(
        empty.rfind_with(&empty, bit_string::FindOptions::default()),
        Some(0)
    );
    assert_eq!(
        empty.rfind_with(&packed(&[0]), bit_string::FindOptions::default()),
        None
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_haystack = packed_as::<Oct, 3>(&oct_codes, oct);
    assert_eq!(
        oct_haystack.rfind_with(
            &packed_as::<Oct, 3>(&[4, 5, 6], oct),
            bit_string::FindOptions::default()
        ),
        Some(20)
    );

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_haystack = packed_as::<WideCode, 7>(&wide_codes, wide);
    assert_eq!(
        wide_haystack.rfind_with(
            &packed_as::<WideCode, 7>(&[88, 99], wide),
            bit_string::FindOptions::default()
        ),
        Some(8)
    );
}
