use super::{Oct, PackedSymbol, SparseByte, WideCode, oct, packed_as, wide};
use bit_string::{BitStr, PackedString, traits::PackedChar};
use int_intervals::UsizeCO;
use proptest::prelude::*;

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
fn packed_str_is_a_fixed_size_typed_view_with_value_semantics() {
    assert_eq!(
        core::mem::size_of::<bit_string::PackedStr<'static, Oct, 3>>(),
        core::mem::size_of::<BitStr<'static>>()
    );
    assert_eq!(
        core::mem::size_of::<bit_string::PackedStr<'static, WideCode, 7>>(),
        core::mem::size_of::<BitStr<'static>>()
    );

    let left = packed_as::<Oct, 3>(&[0, 1, 2, 3], oct);
    let right = packed_as::<Oct, 3>(&[0, 1, 2, 3], oct);
    assert!(left.as_packed_str() == right.as_packed_str());

    let different = packed_as::<Oct, 3>(&[0, 1, 2, 4], oct);
    assert!(left.as_packed_str() != different.as_packed_str());
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

#[test]
fn packed_str_get_decodes_bounds_offsets_and_cross_word_codes() {
    let empty = PackedString::<PackedSymbol, 1>::new();
    assert_eq!(empty.as_packed_str().get(0), None);
    assert_eq!(empty.as_packed_str().get(usize::MAX), None);

    let binary = packed_as::<PackedSymbol, 1>(&[0, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(binary.as_packed_str().get(0), Some(PackedSymbol::Zero));
    assert_eq!(binary.as_packed_str().get(1), Some(PackedSymbol::One));
    assert_eq!(binary.as_packed_str().get(2), None);

    let bytes = packed_as::<SparseByte, 8>(&[0, 3, 255], |code| match code {
        0 => SparseByte::Zero,
        3 => SparseByte::Middle,
        255 => SparseByte::Maximum,
        _ => unreachable!(),
    });
    let byte_view = bytes.as_packed_str();
    assert_eq!(byte_view.get(0), Some(SparseByte::Zero));
    assert_eq!(byte_view.get(2), Some(SparseByte::Maximum));
    assert_eq!(byte_view.get(3), None);
    assert_eq!(byte_view.get(usize::MAX), None);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_string = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_view = oct_string.as_packed_str();
    for (index, &code) in oct_codes.iter().enumerate() {
        assert_eq!(oct_view.get(index), Some(oct(code)));
    }
    assert_eq!(oct_view.get(oct_codes.len()), None);
    let oct_slice = oct_view.slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    for (index, &code) in oct_codes[20..24].iter().enumerate() {
        assert_eq!(oct_slice.get(index), Some(oct(code)));
    }
    assert_eq!(oct_slice.get(4), None);

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_string = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_view = wide_string.as_packed_str();
    for (index, &code) in wide_codes.iter().enumerate() {
        assert_eq!(wide_view.get(index), Some(wide(code)));
    }
    assert_eq!(wide_view.get(wide_codes.len()), None);
    let wide_slice = wide_view.slice(UsizeCO::checked_from_start_len(8, 5).unwrap());
    for (index, &code) in wide_codes[8..13].iter().enumerate() {
        assert_eq!(wide_slice.get(index), Some(wide(code)));
    }
    assert_eq!(wide_slice.get(usize::MAX), None);
}

#[test]
fn packed_string_get_isolates_codes_at_width_boundaries() {
    let binary = packed_as::<PackedSymbol, 1>(&[0, 1, 0, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(
        binary.to_vec(),
        vec![
            PackedSymbol::Zero,
            PackedSymbol::One,
            PackedSymbol::Zero,
            PackedSymbol::One,
        ]
    );
    assert_eq!(binary.get(4), None);
    assert_eq!(binary.get(usize::MAX), None);

    let bytes = packed_as::<SparseByte, 8>(&[0, 255, 3, 255], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        3 => SparseByte::Middle,
        _ => unreachable!(),
    });
    for (index, code) in [0, 255, 3, 255].into_iter().enumerate() {
        assert_eq!(bytes.get(index).map(PackedChar::code), Some(code));
    }

    let oct_codes: Vec<_> = (0..22)
        .map(|index| if index == 21 { 1 } else { index as u8 % 8 })
        .collect();
    let oct_string = packed_as::<Oct, 3>(&oct_codes, oct);
    for (index, &code) in oct_codes.iter().enumerate() {
        assert_eq!(oct_string.get(index).map(PackedChar::code), Some(code));
    }
    assert_eq!(
        oct_string.get(20).map(PackedChar::code),
        Some(oct_codes[20])
    );
    assert_eq!(oct_string.get(21).map(PackedChar::code), Some(1));

    let wide_codes: Vec<_> = (0..10)
        .map(|index| {
            if index == 9 {
                1
            } else {
                (index * 11) as u8 % 128
            }
        })
        .collect();
    let wide_string = packed_as::<WideCode, 7>(&wide_codes, wide);
    for (index, &code) in wide_codes.iter().enumerate() {
        assert_eq!(wide_string.get(index).map(PackedChar::code), Some(code));
    }
    assert_eq!(
        wide_string.get(8).map(PackedChar::code),
        Some(wide_codes[8])
    );
    assert_eq!(wide_string.get(9).map(PackedChar::code), Some(1));
    assert_eq!(wide_string.get(10), None);
}

#[test]
fn packed_str_first_is_view_relative_and_empty_safe() {
    let empty = PackedString::<PackedSymbol, 1>::new();
    assert_eq!(empty.as_packed_str().first(), None);

    let binary = packed_as::<PackedSymbol, 1>(&[1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(binary.as_packed_str().first(), Some(PackedSymbol::One));

    let bytes = packed_as::<SparseByte, 8>(&[255, 0], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        _ => unreachable!(),
    });
    let byte_view = bytes.as_packed_str();
    assert_eq!(byte_view.first(), Some(SparseByte::Maximum));
    assert_eq!(byte_view.slice_from(1).first(), Some(SparseByte::Zero));
    assert_eq!(byte_view.slice_from(2).first(), None);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_string = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_view = oct_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(21, 2).unwrap());
    assert_eq!(oct_view.first(), Some(oct(5)));
    assert_eq!(oct_view.first(), oct_view.clone().get(0));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_string = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_view = wide_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(9, 2).unwrap());
    assert_eq!(wide_view.first(), Some(wide(wide_codes[9])));
}

#[test]
fn packed_str_last_is_view_relative_and_empty_safe() {
    let empty = PackedString::<PackedSymbol, 1>::new();
    assert_eq!(empty.as_packed_str().last(), None);

    let binary = packed_as::<PackedSymbol, 1>(&[0, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(binary.as_packed_str().last(), Some(PackedSymbol::One));

    let bytes = packed_as::<SparseByte, 8>(&[255, 3, 0], |code| match code {
        0 => SparseByte::Zero,
        3 => SparseByte::Middle,
        255 => SparseByte::Maximum,
        _ => unreachable!(),
    });
    let byte_view = bytes.as_packed_str();
    assert_eq!(byte_view.last(), Some(SparseByte::Zero));
    assert_eq!(byte_view.slice_until(2).last(), Some(SparseByte::Middle));
    assert_eq!(byte_view.slice_until(0).last(), None);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_string = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_view = oct_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 2).unwrap());
    assert_eq!(oct_view.last(), Some(oct(5)));
    assert_eq!(oct_view.last(), oct_view.clone().get(1));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_string = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_view = wide_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 2).unwrap());
    assert_eq!(wide_view.last(), Some(wide(wide_codes[9])));
    assert_eq!(wide_string.as_packed_str().slice_from(16).last(), None);
}

#[test]
fn packed_str_slice_uses_clamped_character_ranges() {
    let owner = super::packed(&[0, 1, 2, 1, 0]);
    let view = owner.as_packed_str();
    let full = view.slice(UsizeCO::checked_from_start_len(0, 5).unwrap());
    assert_eq!(full.iter().collect::<Vec<_>>(), owner.to_vec());

    let middle = view.slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    assert_eq!(
        middle.iter().collect::<Vec<_>>(),
        vec![super::Symbol::One, super::Symbol::Two, super::Symbol::One]
    );
    let singleton = view.slice(UsizeCO::checked_from_start_len(4, 1).unwrap());
    assert_eq!(
        singleton.iter().collect::<Vec<_>>(),
        vec![super::Symbol::Zero]
    );
    let clamped_end = view.slice(UsizeCO::checked_from_start_len(3, 99).unwrap());
    assert_eq!(
        clamped_end.iter().collect::<Vec<_>>(),
        vec![super::Symbol::One, super::Symbol::Zero]
    );
    assert!(
        view.slice(UsizeCO::checked_from_start_len(99, 1).unwrap())
            .is_empty()
    );
    assert!(
        view.slice(UsizeCO::checked_from_start_len(5, 1).unwrap())
            .is_empty()
    );

    let nested = middle.slice(UsizeCO::checked_from_start_len(1, 1).unwrap());
    assert_eq!(nested.iter().collect::<Vec<_>>(), vec![super::Symbol::Two]);
    assert_eq!(nested.to_packed_string().to_vec(), vec![super::Symbol::Two]);

    let binary = packed_as::<PackedSymbol, 1>(&[0, 1, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    let binary_slice = binary
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 2).unwrap());
    assert_eq!(
        binary_slice.iter().collect::<Vec<_>>(),
        vec![PackedSymbol::One, PackedSymbol::One]
    );

    let bytes = packed_as::<SparseByte, 8>(&[0, 255, 3], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        3 => SparseByte::Middle,
        _ => unreachable!(),
    });
    let byte_slice = bytes
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 2).unwrap());
    assert_eq!(
        byte_slice.to_packed_string().to_vec(),
        vec![SparseByte::Maximum, SparseByte::Middle]
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_slice = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap())
        .slice(UsizeCO::checked_from_start_len(1, 2).unwrap());
    assert_eq!(
        oct_slice
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![5, 6]
    );
    assert_eq!(oct_slice.to_packed_string().bits().bit_len(), 6);

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_slice = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 5).unwrap())
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    assert_eq!(
        wide_slice
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![99, 110, 121]
    );
    assert_eq!(wide_slice.to_packed_string().bits().bit_len(), 21);

    assert_eq!(view.char_len(), 5);
    assert_eq!(view.get(0), Some(super::Symbol::Zero));
    assert_eq!(view.get(4), Some(super::Symbol::Zero));
}

#[test]
fn packed_str_slice_from_clamps_at_the_view_end() {
    let owner = super::packed(&[0, 1, 2, 1, 0]);
    let view = owner.as_packed_str();
    assert_eq!(
        view.slice_from(0).iter().collect::<Vec<_>>(),
        owner.to_vec()
    );
    assert_eq!(
        view.slice_from(1).iter().collect::<Vec<_>>(),
        vec![
            super::Symbol::One,
            super::Symbol::Two,
            super::Symbol::One,
            super::Symbol::Zero
        ]
    );
    assert_eq!(
        view.slice_from(4).iter().collect::<Vec<_>>(),
        vec![super::Symbol::Zero]
    );
    assert!(view.slice_from(5).is_empty());
    assert!(view.slice_from(6).is_empty());
    assert!(view.slice_from(usize::MAX).is_empty());

    let empty_owner = PackedString::<PackedSymbol, 1>::new();
    assert!(empty_owner.as_packed_str().slice_from(0).is_empty());
    assert!(
        empty_owner
            .as_packed_str()
            .slice_from(usize::MAX)
            .is_empty()
    );

    let nested = view
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap())
        .slice_from(1);
    assert_eq!(
        nested.iter().collect::<Vec<_>>(),
        vec![super::Symbol::Two, super::Symbol::One]
    );

    let binary = packed_as::<PackedSymbol, 1>(&[0, 1, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    let binary_suffix = binary.as_packed_str().slice_from(1);
    assert_eq!(
        binary_suffix.iter().collect::<Vec<_>>(),
        vec![PackedSymbol::One, PackedSymbol::One]
    );
    assert_eq!(binary_suffix.to_packed_string().bits().bit_len(), 2);

    let bytes = packed_as::<SparseByte, 8>(&[0, 255, 3], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        3 => SparseByte::Middle,
        _ => unreachable!(),
    });
    let byte_suffix = bytes.as_packed_str().slice_from(1);
    assert_eq!(
        byte_suffix.iter().collect::<Vec<_>>(),
        vec![SparseByte::Maximum, SparseByte::Middle]
    );
    assert_eq!(byte_suffix.to_packed_string().bits().bit_len(), 16);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_suffix = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap())
        .slice_from(1);
    assert_eq!(
        oct_suffix
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![5, 6, 7]
    );
    assert_eq!(oct_suffix.to_packed_string().bits().bit_len(), 9);

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_suffix = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap())
        .slice_from(1);
    assert_eq!(
        wide_suffix
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![99, 110, 121]
    );
    assert_eq!(wide_suffix.to_packed_string().bits().bit_len(), 21);

    assert_eq!(view.char_len(), 5);
    assert_eq!(view.get(0), Some(super::Symbol::Zero));
    assert_eq!(view.get(4), Some(super::Symbol::Zero));
}

#[test]
fn packed_str_slice_until_clamps_at_the_view_start() {
    let owner = super::packed(&[0, 1, 2, 1, 0]);
    let view = owner.as_packed_str();
    assert!(view.slice_until(0).is_empty());
    assert_eq!(
        view.slice_until(1).iter().collect::<Vec<_>>(),
        vec![super::Symbol::Zero]
    );
    assert_eq!(
        view.slice_until(4).iter().collect::<Vec<_>>(),
        vec![
            super::Symbol::Zero,
            super::Symbol::One,
            super::Symbol::Two,
            super::Symbol::One,
        ]
    );
    assert_eq!(
        view.slice_until(5).iter().collect::<Vec<_>>(),
        owner.to_vec()
    );
    assert_eq!(
        view.slice_until(usize::MAX).iter().collect::<Vec<_>>(),
        owner.to_vec()
    );

    let empty_owner = PackedString::<PackedSymbol, 1>::new();
    assert!(empty_owner.as_packed_str().slice_until(0).is_empty());
    assert!(
        empty_owner
            .as_packed_str()
            .slice_until(usize::MAX)
            .is_empty()
    );

    let offset_owner = super::packed(&[0, 1, 2, 1, 0]);
    let offset_view = offset_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    assert!(offset_view.slice_until(0).is_empty());
    assert_eq!(
        offset_view.slice_until(1).iter().collect::<Vec<_>>(),
        vec![super::Symbol::One]
    );
    let nested = offset_view.slice_until(2).slice_until(1);
    assert_eq!(nested.iter().collect::<Vec<_>>(), vec![super::Symbol::One]);

    let binary = packed_as::<PackedSymbol, 1>(&[0, 1, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    let binary_prefix = binary.as_packed_str().slice_until(2);
    assert_eq!(
        binary_prefix.iter().collect::<Vec<_>>(),
        vec![PackedSymbol::Zero, PackedSymbol::One]
    );
    assert_eq!(binary_prefix.to_packed_string().bits().bit_len(), 2);

    let bytes = packed_as::<SparseByte, 8>(&[0, 255, 3], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        3 => SparseByte::Middle,
        _ => unreachable!(),
    });
    let byte_prefix = bytes.as_packed_str().slice_until(2);
    assert_eq!(
        byte_prefix.to_packed_string().to_vec(),
        vec![SparseByte::Zero, SparseByte::Maximum]
    );
    assert_eq!(byte_prefix.to_packed_string().bits().bit_len(), 16);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_prefix = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap())
        .slice_until(3);
    assert_eq!(
        oct_prefix
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![4, 5, 6]
    );
    assert_eq!(oct_prefix.to_packed_string().bits().bit_len(), 9);

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_prefix = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap())
        .slice_until(3);
    assert_eq!(
        wide_prefix
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![88, 99, 110]
    );
    assert_eq!(wide_prefix.to_packed_string().bits().bit_len(), 21);

    assert_eq!(view.char_len(), 5);
    assert_eq!(view.get(0), Some(super::Symbol::Zero));
    assert_eq!(view.get(4), Some(super::Symbol::Zero));
}

#[test]
fn packed_string_is_empty_tracks_zero_packed_bits() {
    let new = PackedString::<super::Symbol, 2>::new();
    let default = PackedString::<super::Symbol, 2>::default();
    let from_empty = PackedString::<super::Symbol, 2>::from_chars(core::iter::empty());
    for value in [&new, &default, &from_empty] {
        assert!(value.is_empty());
        assert_eq!(value.char_len(), 0);
        assert_eq!(value.bits().bit_len(), 0);
    }

    let one_zero = super::packed(&[0]);
    assert!(!one_zero.is_empty());
    assert_eq!(one_zero.char_len(), 1);
    assert_eq!(one_zero.bits().bit_len(), 2);

    let all_zero = super::packed(&[0; 64]);
    assert!(!all_zero.is_empty());
    assert_eq!(all_zero.char_len(), 64);
    assert_eq!(all_zero.bits().bit_len(), 128);

    let binary = packed_as::<PackedSymbol, 1>(&[0], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    let oct = packed_as::<Oct, 3>(&[0; 22], super::oct);
    let wide = packed_as::<WideCode, 7>(&[0; 10], super::wide);
    let bytes = packed_as::<SparseByte, 8>(&[0], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        3 => SparseByte::Middle,
        _ => unreachable!(),
    });
    assert!(!binary.is_empty());
    assert!(!oct.is_empty());
    assert!(!wide.is_empty());
    assert!(!bytes.is_empty());

    let mut popped = super::packed(&[0]);
    assert!(!popped.is_empty());
    assert_eq!(popped.pop(), Some(super::Symbol::Zero));
    assert!(popped.is_empty());

    let mut cleared = super::packed(&[0, 1, 2]);
    cleared.clear();
    assert!(cleared.is_empty());

    let mut truncated = super::packed(&[0, 1, 2]);
    truncated.truncate(0);
    assert!(truncated.is_empty());

    let mut drained = super::packed(&[0, 1, 2]);
    drained.drain_interval_assign(UsizeCO::checked_from_start_len(0, 3).unwrap());
    assert!(drained.is_empty());
}

fn assert_access_slice_order<C, const BITS: u8>(
    left: &[u8],
    right: &[u8],
    start: usize,
    len: usize,
    decode: fn(u8) -> C,
) where
    C: PackedChar<BITS> + core::fmt::Debug,
{
    let string = packed_as(left, decode);
    assert_eq!(string.char_len(), left.len());
    let view = string.as_packed_str();
    for index in 0..=left.len() + 1 {
        assert_eq!(string.get(index), left.get(index).copied().map(decode));
        assert_eq!(view.get(index), left.get(index).copied().map(decode));
    }

    let oracle_start = start.min(left.len());
    let oracle_end = start.saturating_add(len).min(left.len()).max(oracle_start);
    let slice = string.slice(UsizeCO::checked_from_start_len(start, len).unwrap());
    assert_eq!(
        slice.to_vec(),
        left[oracle_start..oracle_end]
            .iter()
            .copied()
            .map(decode)
            .collect::<Vec<_>>()
    );

    let right_string = packed_as(right, decode);
    assert_eq!(string.cmp(&right_string), left.cmp(&right));
    assert_eq!(
        string.as_packed_str().cmp(&right_string.as_packed_str()),
        left.cmp(&right)
    );
}

proptest! {
    #[test]
    fn packed_access_slice_and_order_match_vec_oracles(
        left in prop::collection::vec(0u8..=2, 0..=32),
        right in prop::collection::vec(0u8..=2, 0..=32),
        start in 0usize..40,
        len in 1usize..40,
    ) {
        let string = super::packed(&left);
        prop_assert_eq!(string.char_len(), left.len());
        let view = string.as_packed_str();
        for index in 0..=left.len() + 1 {
            prop_assert_eq!(string.get(index), left.get(index).copied().map(super::symbol));
            prop_assert_eq!(view.get(index), left.get(index).copied().map(super::symbol));
        }

        let oracle_start = start.min(left.len());
        let oracle_end = start.saturating_add(len).min(left.len()).max(oracle_start);
        let slice = string.slice(UsizeCO::checked_from_start_len(start, len).unwrap());
        prop_assert_eq!(slice.to_vec(), left[oracle_start..oracle_end].iter().copied().map(super::symbol).collect::<Vec<_>>());

        let right_string = super::packed(&right);
        prop_assert_eq!(string.cmp(&right_string), left.cmp(&right));
        prop_assert_eq!(string.as_packed_str().cmp(&right_string.as_packed_str()), left.cmp(&right));
    }

    #[test]
    fn three_and_seven_bit_access_slice_and_order_cross_word_boundaries(
        left3 in prop::collection::vec(0u8..=7, 22..=128),
        right3 in prop::collection::vec(0u8..=7, 22..=128),
        left7 in prop::collection::vec(0u8..=127, 10..=128),
        right7 in prop::collection::vec(0u8..=127, 10..=128),
        start in 0usize..160,
        len in 1usize..80,
    ) {
        assert_access_slice_order::<Oct, 3>(&left3, &right3, start, len, oct);
        assert_access_slice_order::<WideCode, 7>(&left7, &right7, start, len, wide);
    }
}
