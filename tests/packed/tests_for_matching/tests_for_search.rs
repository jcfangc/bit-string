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

    assert!(haystack.contains(repeated.as_packed_str()));
    assert!(final_haystack.contains(final_window.as_packed_str()));
    assert!(!haystack.contains(different.as_packed_str()));
    assert!(!haystack.contains(oversized.as_packed_str()));
    assert!(haystack.contains(haystack.slice_until(0)));

    let empty_owner = packed(&[]);
    let empty = empty_owner.as_packed_str();
    let singleton_owner = packed(&[0]);
    assert!(empty.contains(empty));
    assert!(!empty.contains(singleton_owner.as_packed_str()));

    let offset_owner = packed(&[0, 1, 2, 1, 0]);
    let offset_haystack = offset_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_needle = offset_haystack.slice_from(1);
    let offset_different_owner = packed(&[2, 2]);
    let offset_different = offset_different_owner.as_packed_str();
    assert!(offset_haystack.contains(offset_needle));
    assert!(!offset_haystack.contains(offset_different));

    let binary = packed_as::<super::PackedSymbol, 1>(&[0, 1, 1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert!(
        binary
            .as_packed_str()
            .contains(binary.as_packed_str().slice_from(1))
    );

    let bytes = packed_as::<super::SparseByte, 8>(&[0, 255], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        _ => unreachable!(),
    });
    assert!(
        bytes
            .as_packed_str()
            .contains(bytes.as_packed_str().slice_from(1))
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_haystack = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    let oct_different_owner = packed_as::<Oct, 3>(&[5, 4], oct);
    assert!(oct_haystack.contains(oct_haystack.slice_from(1)));
    assert!(!oct_haystack.contains(oct_different_owner.as_packed_str()));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_haystack = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    let wide_different_owner = packed_as::<WideCode, 7>(&[11, 0], wide);
    assert!(wide_haystack.contains(wide_haystack.slice_from(1)));
    assert!(!wide_haystack.contains(wide_different_owner.as_packed_str()));
}

#[test]
fn packed_string_contains_searches_character_aligned_windows() {
    let haystack = packed(&[0, 1, 0, 1, 2, 1]);
    assert!(haystack.contains(&packed(&[0, 1, 2])));
    assert!(haystack.contains(&packed(&[1, 2, 1])));
    assert!(!haystack.contains(&packed(&[2, 0])));
    assert!(!haystack.contains(&packed(&[0, 1, 0, 1, 2, 1, 0])));
    assert!(haystack.contains(&PackedString::<super::Symbol, 2>::new()));

    let empty = PackedString::<super::Symbol, 2>::new();
    assert!(!empty.contains(&packed(&[0])));

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_haystack = packed_as::<Oct, 3>(&oct_codes, oct);
    assert!(oct_haystack.contains(&packed_as::<Oct, 3>(&[4, 5, 6, 7], oct)));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_haystack = packed_as::<WideCode, 7>(&wide_codes, wide);
    assert!(wide_haystack.contains(&packed_as::<WideCode, 7>(&[88, 99], wide)));
}

#[test]
fn packed_string_find_returns_the_earliest_character_index() {
    let haystack = packed(&[0, 1, 0, 1, 2, 1]);
    assert_eq!(haystack.find(&packed(&[0, 1])), Some(0));
    assert_eq!(haystack.find(&packed(&[1, 2])), Some(3));
    assert_eq!(haystack.find(&packed(&[2, 0])), None);
    assert_eq!(haystack.find(&packed(&[0, 1, 0, 1, 2, 1, 0])), None);
    assert_eq!(
        haystack.find(&PackedString::<super::Symbol, 2>::new()),
        Some(0)
    );

    let empty = PackedString::<super::Symbol, 2>::new();
    assert_eq!(empty.find(&empty), Some(0));
    assert_eq!(empty.find(&packed(&[0])), None);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_haystack = packed_as::<Oct, 3>(&oct_codes, oct);
    assert_eq!(
        oct_haystack.find(&packed_as::<Oct, 3>(&[4, 5, 6], oct)),
        Some(4)
    );

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_haystack = packed_as::<WideCode, 7>(&wide_codes, wide);
    assert_eq!(
        wide_haystack.find(&packed_as::<WideCode, 7>(&[88, 99], wide)),
        Some(8)
    );
}

#[test]
fn packed_str_find_returns_the_earliest_character_index() {
    let repeated_owner = packed(&[0, 0, 0]);
    let repeated_needle = packed(&[0, 0]);
    assert_eq!(
        repeated_owner
            .as_packed_str()
            .find(repeated_needle.as_packed_str()),
        Some(0)
    );

    let final_owner = packed(&[1, 2, 0]);
    let final_needle = packed(&[2, 0]);
    assert_eq!(
        final_owner
            .as_packed_str()
            .find(final_needle.as_packed_str()),
        Some(1)
    );

    let no_match = packed(&[1, 2]);
    let no_match_needle = packed(&[0]);
    let oversized = packed(&[0, 1, 2]);
    assert_eq!(
        no_match
            .as_packed_str()
            .find(no_match_needle.as_packed_str()),
        None
    );
    assert_eq!(
        no_match.as_packed_str().find(oversized.as_packed_str()),
        None
    );

    let empty_owner = packed(&[]);
    let empty = empty_owner.as_packed_str();
    assert_eq!(empty.find(empty), Some(0));
    assert_eq!(final_owner.as_packed_str().find(empty), Some(0));

    let offset_owner = packed(&[0, 1, 2, 1, 0]);
    let offset_haystack = offset_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_needle = packed(&[1, 2]);
    assert_eq!(offset_haystack.find(offset_needle.as_packed_str()), Some(0));

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
        binary.as_packed_str().find(binary_needle.as_packed_str()),
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
        bytes.as_packed_str().find(byte_needle.as_packed_str()),
        Some(1)
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_haystack = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    assert_eq!(oct_haystack.find(oct_haystack.slice_from(1)), Some(1));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_haystack = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    assert_eq!(wide_haystack.find(wide_haystack.slice_from(1)), Some(1));

    assert_eq!(offset_haystack.char_len(), 3);
    assert_eq!(offset_haystack.get(0), Some(super::Symbol::One));
    assert_eq!(offset_haystack.get(2), Some(super::Symbol::One));
}

#[test]
fn packed_str_rfind_returns_the_latest_character_index() {
    let repeated_owner = packed(&[0, 0, 0]);
    let repeated_needle = packed(&[0, 0]);
    assert_eq!(
        repeated_owner
            .as_packed_str()
            .rfind(repeated_needle.as_packed_str()),
        Some(1)
    );

    let final_owner = packed(&[1, 2, 0]);
    let final_needle = packed(&[2, 0]);
    assert_eq!(
        final_owner
            .as_packed_str()
            .rfind(final_needle.as_packed_str()),
        Some(1)
    );

    let no_match = packed(&[1, 2]);
    let no_match_needle = packed(&[0]);
    let oversized = packed(&[0, 1, 2]);
    assert_eq!(
        no_match
            .as_packed_str()
            .rfind(no_match_needle.as_packed_str()),
        None
    );
    assert_eq!(
        no_match.as_packed_str().rfind(oversized.as_packed_str()),
        None
    );

    let empty_owner = packed(&[]);
    let empty = empty_owner.as_packed_str();
    assert_eq!(empty.rfind(empty), Some(0));
    assert_eq!(final_owner.as_packed_str().rfind(empty), Some(3));

    let offset_owner = packed(&[0, 1, 2, 1, 0]);
    let offset_haystack = offset_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_needle = packed(&[1]);
    assert_eq!(
        offset_haystack.rfind(offset_needle.as_packed_str()),
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
        binary.as_packed_str().rfind(binary_needle.as_packed_str()),
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
        bytes.as_packed_str().rfind(byte_needle.as_packed_str()),
        Some(1)
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_haystack = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    assert_eq!(oct_haystack.rfind(oct_haystack.slice_from(1)), Some(1));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_haystack = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    assert_eq!(wide_haystack.rfind(wide_haystack.slice_from(1)), Some(1));

    assert_eq!(offset_haystack.char_len(), 3);
    assert_eq!(offset_haystack.get(0), Some(super::Symbol::One));
    assert_eq!(offset_haystack.get(2), Some(super::Symbol::One));
}

#[test]
fn packed_string_rfind_returns_the_latest_character_index() {
    let haystack = packed(&[0, 1, 0, 1, 2, 1]);
    assert_eq!(haystack.rfind(&packed(&[0, 1])), Some(2));
    assert_eq!(haystack.rfind(&packed(&[1, 2])), Some(3));
    assert_eq!(haystack.rfind(&packed(&[2, 0])), None);
    assert_eq!(haystack.rfind(&packed(&[0, 1, 0, 1, 2, 1, 0])), None);
    assert_eq!(
        haystack.rfind(&PackedString::<super::Symbol, 2>::new()),
        Some(6)
    );

    let empty = PackedString::<super::Symbol, 2>::new();
    assert_eq!(empty.rfind(&empty), Some(0));
    assert_eq!(empty.rfind(&packed(&[0])), None);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_haystack = packed_as::<Oct, 3>(&oct_codes, oct);
    assert_eq!(
        oct_haystack.rfind(&packed_as::<Oct, 3>(&[4, 5, 6], oct)),
        Some(20)
    );

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_haystack = packed_as::<WideCode, 7>(&wide_codes, wide);
    assert_eq!(
        wide_haystack.rfind(&packed_as::<WideCode, 7>(&[88, 99], wide)),
        Some(8)
    );
}
