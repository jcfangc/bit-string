use crate::PackedString;
use crate::packed_string::tests_for_support::{Letter, LetterString};
use crate::traits::PackedChar;

#[test]
fn iter_is_double_ended_and_exact_size() {
    let value = LetterString::from_chars([Letter::A, Letter::B, Letter::C]);
    let mut iter = value.iter();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.next(), Some(Letter::A));
    assert_eq!(iter.next_back(), Some(Letter::C));
    assert_eq!(iter.next(), Some(Letter::B));
    assert_eq!(iter.next(), None);
}

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
fn mixed_iteration_preserves_order_across_word_boundaries() {
    fn assert_front_cursor<const BITS: u8>(iter: &super::Iter<'_, Code, BITS>, front: usize) {
        assert_eq!(
            front * usize::from(BITS),
            iter.front_word * crate::WORD_BITS + iter.front_bit_offset
        );
    }

    fn check<const BITS: u8>() {
        let mask = if BITS == 8 {
            u8::MAX
        } else {
            (1u16 << BITS) as u8 - 1
        };
        let expected = (0..100)
            .map(|index| Code(((index * 13 + 7) as u8) & mask))
            .collect::<alloc::vec::Vec<_>>();
        let value = PackedString::<Code, BITS>::from_chars(expected.iter().copied());
        let mut iter = value.iter();
        assert_front_cursor(&iter, 0);

        for (front, &code) in expected[..25].iter().enumerate() {
            assert_eq!(iter.next(), Some(code));
            assert_front_cursor(&iter, front + 1);
        }
        for &code in expected[75..].iter().rev() {
            assert_eq!(iter.next_back(), Some(code));
            assert_front_cursor(&iter, 25);
        }
        for (front, &code) in expected[25..75].iter().enumerate() {
            assert_eq!(iter.next(), Some(code));
            assert_front_cursor(&iter, front + 26);
        }

        assert_front_cursor(&iter, 75);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
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
