use super::{Oct, PackedSymbol, SparseByte, Symbol, WideCode, oct, packed_as, wide};
use bit_string::{BitString, PackedString, traits::PackedChar};

fn assert_packed_owner<C, const BITS: u8>(codes: &[u8], decode: fn(u8) -> C)
where
    C: PackedChar<BITS> + core::fmt::Debug,
{
    let owner = packed_as(codes, decode);
    let expected = codes.iter().copied().map(decode).collect::<Vec<_>>();
    assert_eq!(owner.char_len(), codes.len());
    assert_eq!(owner.bits_per_char(), usize::from(BITS));
    assert_eq!(owner.bits().bit_len(), codes.len() * usize::from(BITS));
    assert_eq!(owner.to_vec(), expected);
    for (index, character) in expected.iter().copied().enumerate() {
        assert_eq!(owner.get(index), Some(character));
    }
    assert_eq!(owner.get(codes.len()), None);
    assert!(owner.as_packed_str().iter().collect::<Vec<_>>() == expected);
    assert!(owner.clone().into_bits() == owner.bits().clone());
}

#[test]
fn generated_sparse_code_round_trip_is_executable() {
    assert_eq!(SparseByte::Zero.code(), 0);
    assert_eq!(SparseByte::Middle.code(), 3);
    assert_eq!(SparseByte::Maximum.code(), u8::MAX);

    for code in 0..=u8::MAX {
        let expected = match code {
            0 => Some(SparseByte::Zero),
            3 => Some(SparseByte::Middle),
            u8::MAX => Some(SparseByte::Maximum),
            _ => None,
        };
        assert_eq!(SparseByte::from_code(code), expected);
    }
}

#[test]
fn packed_attribute_encoding_round_trips_through_packed_string() {
    let string = PackedString::<Symbol, 2>::from_chars([Symbol::Two, Symbol::Zero, Symbol::One]);
    assert_eq!(string.get(0), Some(Symbol::Two));
    assert_eq!(string.get(1), Some(Symbol::Zero));
    assert_eq!(string.get(2), Some(Symbol::One));
}

#[test]
fn bits_per_char_is_the_type_level_width() {
    let binary_empty = PackedString::<PackedSymbol, 1>::new();
    assert_eq!(binary_empty.bits_per_char(), 1);

    let oct_string = packed_as::<Oct, 3>(&[0; 22], oct);
    assert_eq!(oct_string.bits_per_char(), 3);
    assert_eq!(oct_string.bits().bit_len(), 22 * oct_string.bits_per_char());

    let wide_string = packed_as::<WideCode, 7>(&[0, 127], wide);
    assert_eq!(wide_string.bits_per_char(), 7);
    assert_eq!(
        wide_string.bits().bit_len(),
        2 * wide_string.bits_per_char()
    );

    let byte_string = packed_as::<SparseByte, 8>(&[0, 255, 3], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        3 => SparseByte::Middle,
        _ => unreachable!(),
    });
    assert_eq!(byte_string.bits_per_char(), 8);
    assert_eq!(
        byte_string.bits().bit_len(),
        3 * byte_string.bits_per_char()
    );
}

#[test]
fn bits_exposes_the_owned_bitstring_layout() {
    let empty = PackedString::<Symbol, 2>::new();
    assert!(empty.bits().words().is_empty());
    assert_eq!(empty.bits().bit_len(), 0);
    assert!(core::ptr::eq(empty.bits(), empty.bits()));

    let zero = packed_as::<SparseByte, 8>(&[0], |code| match code {
        0 => SparseByte::Zero,
        _ => unreachable!(),
    });
    assert_eq!(zero.bits().bit_len(), 8);
    assert_eq!(zero.bits().words(), &[0]);
    assert!(!zero.is_empty());

    let oct_codes: Vec<_> = (0..22).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_expected = BitString::from_iter(
        oct_codes
            .iter()
            .flat_map(|&code| (0..3).map(move |offset| code & (1 << offset) != 0)),
    );
    assert_eq!(oct_owner.bits().bit_len(), oct_expected.bit_len());
    assert_eq!(oct_owner.bits().words(), oct_expected.words());

    let wide_codes: Vec<_> = (0..10).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_expected = BitString::from_iter(
        wide_codes
            .iter()
            .flat_map(|&code| (0..7).map(move |offset| code & (1 << offset) != 0)),
    );
    assert_eq!(wide_owner.bits().bit_len(), wide_expected.bit_len());
    assert_eq!(wide_owner.bits().words(), wide_expected.words());
}

fn assert_default_matches_new<C, const BITS: u8>()
where
    C: PackedChar<BITS>,
{
    let default = PackedString::<C, BITS>::default();
    let new = PackedString::<C, BITS>::new();
    assert!(default == new);
    assert!(default.is_empty());
    assert_eq!(default.char_len(), 0);
    assert_eq!(default.bits().bit_len(), 0);
    assert!(default.bits().words().is_empty());
    assert_eq!(default.bits_per_char(), usize::from(BITS));
}

#[test]
fn packed_default_is_canonical_and_editable() {
    assert_default_matches_new::<PackedSymbol, 1>();
    assert_default_matches_new::<Symbol, 2>();
    assert_default_matches_new::<Oct, 3>();
    assert_default_matches_new::<WideCode, 7>();
    assert_default_matches_new::<SparseByte, 8>();

    let mut default = PackedString::<Oct, 3>::default();
    let mut new = PackedString::<Oct, 3>::new();
    for code in [
        Oct::V0,
        Oct::V7,
        Oct::V3,
        Oct::V4,
        Oct::V1,
        Oct::V6,
        Oct::V2,
        Oct::V5,
    ] {
        default.push(code);
        new.push(code);
    }
    assert!(default == new);
    assert_eq!(default.char_len(), 8);

    default.clear();
    new.clear();
    assert!(default == new);
    assert!(default.is_empty());
    assert_eq!(default.pop(), None);
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ManualSymbol;

impl PackedChar<1> for ManualSymbol {
    fn code(self) -> u8 {
        0
    }

    fn from_code(code: u8) -> Option<Self> {
        (code == 0).then_some(Self)
    }
}

#[test]
fn manual_packed_char_implementations_remain_supported() {
    assert_eq!(
        PackedString::<ManualSymbol, 1>::from_chars([ManualSymbol]).char_len(),
        1
    );
}

#[test]
fn attribute_macro_generates_packed_char_impl() {
    let string =
        PackedString::<PackedSymbol, 1>::from_chars([PackedSymbol::One, PackedSymbol::Zero]);
    assert_eq!(string.get(0), Some(PackedSymbol::One));
}

#[test]
fn from_bits_rejects_misaligned_and_unknown_codes() {
    assert!(PackedString::<Symbol, 2>::from_bits(BitString::from_iter([true])).is_none());
    assert!(PackedString::<Symbol, 2>::from_bits(BitString::from_iter([true, true])).is_none());
}

#[test]
fn packed_string_clone_copies_storage_and_preserves_invariants() {
    let oct_codes: Vec<u8> = (0..22).map(|index| index as u8 % 8).collect();
    let original = packed_as::<Oct, 3>(&oct_codes, oct);
    let original_bits = original.bits().clone();
    let original_values = original.to_vec();
    let mut clone = original.clone();
    assert!(clone.bits() == &original_bits);
    assert_eq!(clone.to_vec(), original_values);
    assert!(clone.as_packed_str() == original.as_packed_str());
    assert_eq!(
        clone.as_packed_str().iter().collect::<Vec<_>>(),
        original_values
    );

    clone.push(Oct::V7);
    assert_eq!(original.to_vec(), original_values);
    assert_eq!(clone.to_vec().len(), original_values.len() + 1);
    clone.pop();
    assert!(clone == original);

    let mut edited_original = original.clone();
    edited_original.clear();
    assert!(original == clone);
    assert!(edited_original.is_empty());

    let binary = packed_as::<PackedSymbol, 1>(&[0, 1, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    let mut binary_clone = binary.clone();
    binary_clone.push(PackedSymbol::Zero);
    assert_eq!(binary.to_vec().len(), 3);
    assert_eq!(binary_clone.to_vec().len(), 4);
    assert_eq!(binary.bits().bit_len(), 3);

    let bytes = packed_as::<SparseByte, 8>(&[0, 255, 3], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        3 => SparseByte::Middle,
        _ => unreachable!(),
    });
    let mut bytes_clone = bytes.clone();
    assert_eq!(
        bytes_clone.set(0, SparseByte::Maximum),
        Some(SparseByte::Zero)
    );
    assert_eq!(bytes.get(0), Some(SparseByte::Zero));
    assert_eq!(bytes_clone.get(0), Some(SparseByte::Maximum));
    assert_eq!(bytes.bits().bit_len(), 24);

    let wide_codes: Vec<u8> = (0..10).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_clone = wide_owner.clone();
    assert!(wide_clone.bits() == wide_owner.bits());
    assert_eq!(
        wide_clone.to_vec(),
        wide_codes.iter().copied().map(wide).collect::<Vec<_>>()
    );
    assert_eq!(wide_clone.as_packed_str().char_len(), 10);
}

#[test]
fn packed_string_representation_invariant_is_width_derived() {
    assert_eq!(
        core::mem::size_of::<PackedString<Symbol, 2>>(),
        core::mem::size_of::<BitString>()
    );
    assert_packed_owner::<PackedSymbol, 1>(&[0, 1, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_packed_owner::<Symbol, 2>(&[0, 1, 2, 1], super::symbol);
    assert_packed_owner::<Oct, 3>(&[0, 1, 6, 7, 2, 5, 4, 3], oct);
    assert_packed_owner::<WideCode, 7>(&[0, 1, 63, 64, 126, 127], wide);
    assert_packed_owner::<SparseByte, 8>(&[0, 255, 3], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        3 => SparseByte::Middle,
        _ => unreachable!(),
    });

    let empty = PackedString::<Symbol, 2>::new();
    assert_eq!(empty.char_len(), 0);
    assert_eq!(empty.bits().bit_len(), 0);
    assert!(empty.as_packed_str().is_empty());

    let invalid_sparse_code =
        BitString::from_iter([true, false, false, false, false, false, false, false]);
    assert!(PackedString::<SparseByte, 8>::from_bits(invalid_sparse_code).is_none());
}
