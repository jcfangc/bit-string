use super::*;

#[test]
fn packed_views_match_only_at_character_boundaries() {
    let string = packed(&[0, 1, 2, 1]);
    let view = string.as_packed_str();
    assert_eq!(
        view.slice(UsizeCO::checked_from_start_len(1, 2).unwrap())
            .get(0),
        Some(super::Symbol::One)
    );
    assert_eq!(view.find(view.slice_until(1)), Some(0));
    assert!(view.contains(view.slice(UsizeCO::checked_from_start_len(1, 1).unwrap())));

    let haystack_string = packed(&[0, 0, 1]);
    let needle_string = packed(&[2]);
    let haystack = haystack_string.as_packed_str();
    let needle = needle_string.as_packed_str();
    assert!(!haystack.contains(needle));
    assert_eq!(haystack.find(needle), None);
    assert_eq!(haystack.rfind(needle), None);
    assert!(!haystack.matches_at(1, needle));
}

#[test]
fn packed_str_matches_at_is_character_aligned_and_bounds_checked() {
    let haystack_owner = packed(&[0, 1, 2, 1, 0]);
    let haystack = haystack_owner.as_packed_str();
    let needle_owner = packed(&[1, 2]);
    let needle = needle_owner.as_packed_str();

    assert!(haystack.matches_at(1, needle));
    assert!(!haystack.matches_at(0, needle));
    assert!(!haystack.matches_at(4, needle));
    assert!(!haystack.matches_at(5, needle));
    assert!(!haystack.matches_at(usize::MAX, needle));

    let empty = haystack.slice_until(0);
    assert!(haystack.matches_at(0, empty));
    assert!(haystack.matches_at(haystack.char_len(), empty));
    assert!(!haystack.matches_at(haystack.char_len() + 1, empty));

    let offset_haystack = haystack.slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_needle = haystack.slice(UsizeCO::checked_from_start_len(2, 2).unwrap());
    assert!(offset_haystack.matches_at(1, offset_needle));
    assert!(!offset_haystack.matches_at(0, offset_needle));

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_haystack = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    let oct_needle = oct_haystack.slice(UsizeCO::checked_from_start_len(1, 2).unwrap());
    assert!(oct_haystack.matches_at(1, oct_needle));
    assert!(!oct_haystack.matches_at(2, oct_needle));
    assert!(!oct_haystack.matches_at(usize::MAX, oct_needle));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_haystack = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    let wide_needle = wide_haystack.slice(UsizeCO::checked_from_start_len(1, 2).unwrap());
    assert!(wide_haystack.matches_at(1, wide_needle));
    assert!(!wide_haystack.matches_at(3, wide_needle));
    assert!(!wide_haystack.matches_at(usize::MAX, wide_needle));
}

#[test]
fn packed_string_matches_at_is_character_aligned_and_bounds_checked() {
    let haystack = packed_as::<Oct, 3>(&[0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3], oct);
    let needle = packed_as::<Oct, 3>(&[4, 5], oct);
    assert!(haystack.matches_at(4, &needle));
    assert!(!haystack.matches_at(3, &needle));
    assert!(!haystack.matches_at(11, &needle));
    assert!(!haystack.matches_at(usize::MAX, &needle));

    let empty = PackedString::<Oct, 3>::new();
    assert!(haystack.matches_at(haystack.char_len(), &empty));
    assert!(!haystack.matches_at(haystack.char_len() + 1, &empty));

    let wide_haystack = packed_as::<WideCode, 7>(&[0, 11, 22, 33, 44, 55, 66, 77, 88, 99], wide);
    let wide_needle = packed_as::<WideCode, 7>(&[88, 99], wide);
    assert!(wide_haystack.matches_at(8, &wide_needle));
    assert!(!wide_haystack.matches_at(7, &wide_needle));
    assert!(!wide_haystack.matches_at(9, &wide_needle));
}

#[test]
fn packed_str_starts_with_compares_character_aligned_prefixes() {
    let owner = packed(&[0, 1, 2, 1, 0]);
    let receiver = owner.as_packed_str();
    let exact = packed(&[0, 1, 2, 1, 0]);
    let shorter = packed(&[0, 1, 2]);
    let different = packed(&[0, 2]);
    let oversized = packed(&[0, 1, 2, 1, 0, 1]);
    assert!(receiver.starts_with(exact.as_packed_str()));
    assert!(receiver.starts_with(shorter.as_packed_str()));
    assert!(!receiver.starts_with(different.as_packed_str()));
    assert!(!receiver.starts_with(oversized.as_packed_str()));

    let empty_owner = packed(&[]);
    let empty = empty_owner.as_packed_str();
    let nonempty_owner = packed(&[0]);
    assert!(empty.starts_with(empty));
    assert!(receiver.starts_with(empty));
    assert!(!empty.starts_with(nonempty_owner.as_packed_str()));

    let offset_receiver = receiver.slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_prefix = receiver.slice(UsizeCO::checked_from_start_len(1, 2).unwrap());
    let offset_different = receiver.slice(UsizeCO::checked_from_start_len(2, 2).unwrap());
    assert!(offset_receiver.starts_with(offset_prefix));
    assert!(!offset_receiver.starts_with(offset_different));

    let binary = packed_as::<super::PackedSymbol, 1>(&[0, 1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert!(
        binary
            .as_packed_str()
            .starts_with(binary.as_packed_str().slice_until(1))
    );

    let bytes = packed_as::<super::SparseByte, 8>(&[255, 0], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        _ => unreachable!(),
    });
    assert!(
        bytes
            .as_packed_str()
            .starts_with(bytes.as_packed_str().slice_until(1))
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_receiver = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    assert!(oct_receiver.starts_with(oct_receiver.slice_until(2)));
    assert!(!oct_receiver.starts_with(oct_receiver.slice_from(1).slice_until(2)));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_receiver = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    assert!(wide_receiver.starts_with(wide_receiver.slice_until(2)));
    assert!(!wide_receiver.starts_with(wide_receiver.slice_from(1).slice_until(2)));
}

#[test]
fn packed_string_starts_with_compares_character_prefixes() {
    let receiver = packed(&[0, 1, 2, 1, 0]);
    assert!(receiver.starts_with(&packed(&[0, 1, 2, 1, 0])));
    assert!(receiver.starts_with(&packed(&[0, 1, 2])));
    assert!(!receiver.starts_with(&packed(&[0, 2])));
    assert!(!receiver.starts_with(&packed(&[0, 1, 2, 1, 0, 1])));
    assert!(receiver.starts_with(&PackedString::<super::Symbol, 2>::new()));

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_receiver = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_prefix = packed_as::<Oct, 3>(&oct_codes[..22], oct);
    assert!(oct_receiver.starts_with(&oct_prefix));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_receiver = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_prefix = packed_as::<WideCode, 7>(&wide_codes[..10], wide);
    assert!(wide_receiver.starts_with(&wide_prefix));
}

#[test]
fn packed_str_ends_with_compares_character_aligned_suffixes() {
    let owner = packed(&[0, 1, 2, 1, 0]);
    let receiver = owner.as_packed_str();
    let exact = packed(&[0, 1, 2, 1, 0]);
    let shorter = packed(&[2, 1, 0]);
    let different = packed(&[1, 1, 0]);
    let oversized = packed(&[0, 1, 2, 1, 0, 1]);
    assert!(receiver.ends_with(exact.as_packed_str()));
    assert!(receiver.ends_with(shorter.as_packed_str()));
    assert!(!receiver.ends_with(different.as_packed_str()));
    assert!(!receiver.ends_with(oversized.as_packed_str()));

    let empty_owner = packed(&[]);
    let empty = empty_owner.as_packed_str();
    let nonempty_owner = packed(&[0]);
    assert!(empty.ends_with(empty));
    assert!(receiver.ends_with(empty));
    assert!(!empty.ends_with(nonempty_owner.as_packed_str()));

    let offset_receiver = receiver.slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_suffix = receiver.slice(UsizeCO::checked_from_start_len(2, 2).unwrap());
    let offset_different = receiver.slice(UsizeCO::checked_from_start_len(1, 2).unwrap());
    assert!(offset_receiver.ends_with(offset_suffix));
    assert!(!offset_receiver.ends_with(offset_different));

    let binary = packed_as::<super::PackedSymbol, 1>(&[0, 1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert!(
        binary
            .as_packed_str()
            .ends_with(binary.as_packed_str().slice_from(1))
    );

    let bytes = packed_as::<super::SparseByte, 8>(&[0, 255], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        _ => unreachable!(),
    });
    assert!(
        bytes
            .as_packed_str()
            .ends_with(bytes.as_packed_str().slice_from(1))
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_receiver = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    assert!(oct_receiver.ends_with(oct_receiver.slice_from(2)));
    assert!(!oct_receiver.ends_with(oct_receiver.slice_from(1).slice_until(2)));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_receiver = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    assert!(wide_receiver.ends_with(wide_receiver.slice_from(2)));
    assert!(!wide_receiver.ends_with(wide_receiver.slice_from(1).slice_until(2)));
}

#[test]
fn packed_string_ends_with_compares_character_suffixes() {
    let receiver = packed(&[0, 1, 2, 1, 0]);
    assert!(receiver.ends_with(&packed(&[0, 1, 2, 1, 0])));
    assert!(receiver.ends_with(&packed(&[2, 1, 0])));
    assert!(!receiver.ends_with(&packed(&[1, 1, 0])));
    assert!(!receiver.ends_with(&packed(&[0, 1, 2, 1, 0, 1])));
    assert!(receiver.ends_with(&PackedString::<super::Symbol, 2>::new()));

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_receiver = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_suffix = packed_as::<Oct, 3>(&oct_codes[2..], oct);
    assert!(oct_receiver.ends_with(&oct_suffix));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_receiver = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_suffix = packed_as::<WideCode, 7>(&wide_codes[6..], wide);
    assert!(wide_receiver.ends_with(&wide_suffix));
}
