use super::*;

#[test]
fn packed_str_clone_preserves_sliced_cross_word_views() {
    let oct_codes: Vec<u8> = (0..64).map(|index| index as u8 % 8).collect();
    let oct_string = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_view = oct_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 24).unwrap());
    let oct_copy = oct_view;
    let oct_clone = oct_view.clone();
    assert!(oct_clone == oct_view);
    assert!(oct_clone == oct_copy);
    assert_eq!(
        oct_clone.iter().collect::<Vec<_>>(),
        oct_codes[20..44]
            .iter()
            .copied()
            .map(oct)
            .collect::<Vec<_>>()
    );
    assert_eq!(oct_clone.iter().count(), 24);

    let wide_codes: Vec<u8> = (0..20).map(|index| (index * 7) as u8 % 128).collect();
    let wide_string = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_view = wide_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 5).unwrap());
    let wide_clone = wide_view.clone();
    assert!(wide_clone == wide_view);
    assert_eq!(
        wide_clone.iter().collect::<Vec<_>>(),
        wide_codes[8..13]
            .iter()
            .copied()
            .map(wide)
            .collect::<Vec<_>>()
    );
    assert_eq!(wide_clone.first(), Some(wide(wide_codes[8])));
    assert_eq!(wide_clone.last(), Some(wide(wide_codes[12])));

    let empty = wide_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(999, 1).unwrap());
    assert!(empty.clone().is_empty());
    assert!(empty.clone() == empty);
}

#[test]
fn packed_str_adds_no_runtime_metadata() {
    assert_eq!(
        core::mem::size_of::<bit_string::PackedStr<'static, Oct, 3>>(),
        core::mem::size_of::<BitStr<'static>>()
    );
    assert_eq!(
        core::mem::size_of::<bit_string::PackedStr<'static, WideCode, 7>>(),
        core::mem::size_of::<BitStr<'static>>()
    );
}

#[test]
fn packed_str_char_len_tracks_widths_boundaries_and_clamped_slices() {
    let binary = packed_as::<PackedSymbol, 1>(&[0, 1], |code| {
        if code == 0 {
            PackedSymbol::Zero
        } else {
            PackedSymbol::One
        }
    });
    assert_eq!(binary.as_packed_str().char_len(), 2);

    let bytes = packed_as::<SparseByte, 8>(&[0, 3, 255], |code| match code {
        0 => SparseByte::Zero,
        3 => SparseByte::Middle,
        255 => SparseByte::Maximum,
        _ => unreachable!(),
    });
    assert_eq!(bytes.as_packed_str().char_len(), 3);

    for (codes, expected) in [
        ((0..21).map(|index| index as u8 % 8).collect::<Vec<_>>(), 21),
        ((0..22).map(|index| index as u8 % 8).collect::<Vec<_>>(), 22),
    ] {
        let string = packed_as::<Oct, 3>(&codes, oct);
        assert_eq!(string.as_packed_str().char_len(), expected);
        let sliced = string
            .as_packed_str()
            .slice(UsizeCO::checked_from_start_len(1, expected - 2).unwrap());
        assert_eq!(sliced.char_len(), expected - 2);
    }

    for length in [9, 10] {
        let codes: Vec<_> = (0..length).map(|index| (index * 7) as u8 % 128).collect();
        let string = packed_as::<WideCode, 7>(&codes, wide);
        assert_eq!(string.as_packed_str().char_len(), length);
    }

    let empty = bytes
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(99, 1).unwrap());
    assert_eq!(empty.char_len(), 0);
}

#[test]
fn packed_str_is_empty_depends_only_on_view_length() {
    let empty_binary_owner = PackedString::<PackedSymbol, 1>::new();
    let empty_binary = empty_binary_owner.as_packed_str();
    assert!(empty_binary.is_empty());
    assert_eq!(empty_binary.char_len(), 0);

    let singleton = packed_as::<SparseByte, 8>(&[0], |code| match code {
        0 => SparseByte::Zero,
        _ => unreachable!(),
    });
    assert!(!singleton.as_packed_str().is_empty());

    let string = packed_as::<Oct, 3>(&[0, 1, 2, 3, 4, 5, 6, 7], oct);
    let empty_at_end = string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(99, 1).unwrap());
    let empty_from_end = string.as_packed_str().slice_from(99);
    let empty_until_start = string.as_packed_str().slice_until(0);
    for view in [empty_at_end, empty_from_end, empty_until_start] {
        assert!(view.is_empty());
        assert_eq!(view.char_len(), 0);
        assert!(view == view.clone());
    }

    let cross_word_owner = packed_as::<Oct, 3>(&[0; 22], oct);
    let cross_word = cross_word_owner.as_packed_str();
    assert!(!cross_word.is_empty());
    assert_eq!(cross_word.char_len(), 22);
}
