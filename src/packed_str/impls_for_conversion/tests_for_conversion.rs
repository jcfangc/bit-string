use crate::{BitString, PackedStr, traits::PackedChar};
use int_intervals::UsizeCO;

#[derive(Clone, Copy, Eq, PartialEq)]
struct TestChar(u8);

impl<const BITS: u8> PackedChar<BITS> for TestChar {
    fn code(self) -> u8 {
        self.0
    }

    fn from_code(code: u8) -> Option<Self> {
        Some(Self(code))
    }
}

fn bit_view<const N: usize>(bits: &BitString, start: usize) -> crate::BitStr<'_> {
    bits.as_bit_str()
        .slice(UsizeCO::checked_from_start_len(start, N).unwrap())
}

#[test]
fn trusted_constructor_preserves_valid_aligned_descriptors() {
    let bits = BitString::zeros(80);
    let source = bit_view::<21>(&bits, 3);
    let packed: PackedStr<'_, TestChar, 3> = PackedStr::from_valid_bit_str(source);

    assert_eq!(packed.bits.start(), 3);
    assert_eq!(packed.bits.bit_len(), 21);
    assert_eq!(packed.char_len(), 7);
    assert!(packed.bits == source);
}

#[test]
#[should_panic(expected = "packed character width must be between 1 and 8")]
fn trusted_constructor_rejects_zero_width() {
    let bits = BitString::new();
    let source = bits.as_bit_str();
    let _: PackedStr<'_, TestChar, 0> = PackedStr::from_valid_bit_str(source);
}

#[test]
#[should_panic(expected = "packed character width must be between 1 and 8")]
fn trusted_constructor_rejects_width_above_eight() {
    let bits = BitString::new();
    let source = bits.as_bit_str();
    let _: PackedStr<'_, TestChar, 9> = PackedStr::from_valid_bit_str(source);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn trusted_constructor_rejects_misaligned_start_in_debug() {
    let bits = BitString::zeros(16);
    let source = bit_view::<3>(&bits, 1);
    let _: PackedStr<'_, TestChar, 3> = PackedStr::from_valid_bit_str(source);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn trusted_constructor_rejects_non_divisible_length_in_debug() {
    let bits = BitString::zeros(16);
    let source = bit_view::<4>(&bits, 0);
    let _: PackedStr<'_, TestChar, 3> = PackedStr::from_valid_bit_str(source);
}
