#[cfg(debug_assertions)]
use crate::BitString;
use crate::packed_string::tests_for_support::{Letter, LetterString};
use crate::{PackedString, traits::PackedChar};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Code(u8);

impl<const BITS: u8> PackedChar<BITS> for Code {
    fn code(self) -> u8 {
        self.0
    }

    fn from_code(code: u8) -> Option<Self> {
        Some(Self(code))
    }
}

#[test]
fn enum_discriminants_are_stored_directly() {
    let value = LetterString::from_chars([Letter::A, Letter::B, Letter::C, Letter::D]);
    assert_eq!(value.bits_per_char(), 2);
    assert_eq!(value.bits().bit_len(), 8);
    assert_eq!(value.bits().get_chunk(0), 0b11_10_01_00);
}

#[test]
fn collect_constructs_a_packed_string() {
    let value: LetterString = [Letter::A, Letter::C].into_iter().collect();
    assert_eq!(value.char_len(), 2);
    assert_eq!(value.get(0), Some(Letter::A));
}

#[test]
fn construction_and_extend_preserve_values_across_word_boundaries() {
    fn check<const BITS: u8>() {
        let mask = if BITS == 8 {
            u8::MAX
        } else {
            (1u16 << BITS) as u8 - 1
        };
        let initial = (0..7)
            .map(|index| Code(((index * 3 + 1) as u8) & mask))
            .collect::<alloc::vec::Vec<_>>();
        let suffix = (7..30)
            .map(|index| Code(((index * 5 + 2) as u8) & mask))
            .collect::<alloc::vec::Vec<_>>();
        let expected = initial
            .iter()
            .chain(&suffix)
            .map(|code| code.0)
            .collect::<alloc::vec::Vec<_>>();

        let mut value = PackedString::<Code, BITS>::from_chars(initial.iter().copied());
        value.extend(suffix.iter().copied());

        assert_eq!(value.bits().bit_len(), expected.len() * usize::from(BITS));
        assert_eq!(
            value
                .iter()
                .map(|code| code.0)
                .collect::<alloc::vec::Vec<_>>(),
            expected
        );
    }

    check::<1>();
    check::<2>();
    check::<3>();
    check::<4>();
    check::<5>();
    check::<6>();
    check::<7>();
    check::<8>();
}

#[test]
fn trusted_constructor_adopts_aligned_payload_unchanged() {
    let source = LetterString::from_chars([Letter::A, Letter::D, Letter::B, Letter::C]);
    let source_bits = source.bits().clone();
    let adopted = PackedString::<Letter, 2>::from_valid_bits(source_bits.clone());

    assert_eq!(adopted.bits().bit_len(), source_bits.bit_len());
    assert_eq!(adopted.bits().words(), source_bits.words());
    assert_eq!(adopted.to_vec(), source.to_vec());
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn trusted_constructor_rejects_misaligned_payload_in_debug() {
    PackedString::<Letter, 2>::from_valid_bits(BitString::from_iter([true]));
}
