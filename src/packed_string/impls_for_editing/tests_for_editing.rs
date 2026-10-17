use crate::packed_string::tests_for_support::{Letter, LetterString};
use crate::traits::PackedChar;

#[test]
fn push_set_pop_and_truncate_preserve_values() {
    let mut value = LetterString::from_chars([Letter::A, Letter::B, Letter::C]);
    value.push(Letter::D);
    assert_eq!(value.set(1, Letter::C), Some(Letter::B));
    assert_eq!(value.pop(), Some(Letter::D));
    value.truncate(2);
    assert_eq!(
        value.iter().collect::<alloc::vec::Vec<_>>(),
        [Letter::A, Letter::C]
    );
    value.clear();
    assert!(value.is_empty());
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Code<const BITS: u8>(u8);

impl<const BITS: u8> PackedChar<BITS> for Code<BITS> {
    fn code(self) -> u8 {
        self.0
    }

    fn from_code(code: u8) -> Option<Self> {
        (code <= crate::code_mask::<BITS>()).then_some(Self(code))
    }
}

fn assert_extend_preserves_codes<const BITS: u8>() {
    for prefix_len in [0, 1, 7, 8, 15, 16, 31, 32, 63, 64, 65] {
        for extension_len in [0, 1, 7, 8, 15, 16, 31, 32, 63, 64, 65, 127, 128, 129, 193] {
            let code_at = |index: usize| {
                (index.wrapping_mul(37).wrapping_add(13) as u8) & crate::code_mask::<BITS>()
            };
            let prefix = (0..prefix_len).map(|index| Code::<BITS>(code_at(index)));
            let extension =
                (prefix_len..prefix_len + extension_len).map(|index| Code::<BITS>(code_at(index)));
            let expected = prefix
                .clone()
                .chain(extension.clone())
                .collect::<alloc::vec::Vec<_>>();

            let mut actual = super::super::PackedString::<Code<BITS>, BITS>::from_chars(prefix);
            actual.extend(extension);

            assert_eq!(actual.iter().collect::<alloc::vec::Vec<_>>(), expected);
        }
    }
}

#[test]
fn extend_batches_preserve_codes_at_unaligned_tails_for_every_width() {
    assert_extend_preserves_codes::<1>();
    assert_extend_preserves_codes::<2>();
    assert_extend_preserves_codes::<3>();
    assert_extend_preserves_codes::<4>();
    assert_extend_preserves_codes::<5>();
    assert_extend_preserves_codes::<6>();
    assert_extend_preserves_codes::<7>();
    assert_extend_preserves_codes::<8>();
}

fn assert_extend_with_zero_size_hint<const BITS: u8>() {
    let prefix_len = 31;
    let extension_len = 193;
    let code_at =
        |index: usize| (index.wrapping_mul(37).wrapping_add(13) as u8) & crate::code_mask::<BITS>();
    let prefix = (0..prefix_len).map(|index| Code::<BITS>(code_at(index)));
    let extension = (prefix_len..prefix_len + extension_len)
        .map(|index| Code::<BITS>(code_at(index)))
        .collect::<alloc::vec::Vec<_>>();
    let expected = prefix
        .clone()
        .chain(extension.iter().copied())
        .collect::<alloc::vec::Vec<_>>();
    let mut actual = super::super::PackedString::<Code<BITS>, BITS>::from_chars(prefix);
    let filtered = extension.iter().copied().filter(|_| true);
    assert_eq!(filtered.size_hint().0, 0);
    actual.extend(filtered);

    assert_eq!(actual.iter().collect::<alloc::vec::Vec<_>>(), expected);
}

#[test]
fn extend_batches_do_not_depend_on_size_hint_lower_bound() {
    assert_extend_with_zero_size_hint::<1>();
    assert_extend_with_zero_size_hint::<2>();
    assert_extend_with_zero_size_hint::<3>();
    assert_extend_with_zero_size_hint::<4>();
    assert_extend_with_zero_size_hint::<5>();
    assert_extend_with_zero_size_hint::<6>();
    assert_extend_with_zero_size_hint::<7>();
    assert_extend_with_zero_size_hint::<8>();
}
