#[cfg(debug_assertions)]
use crate::BitString;
use crate::PackedString;
use crate::packed_string::tests_for_support::{Letter, LetterString};

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
