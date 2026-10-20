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

fn assert_front_window<const BITS: u8>(iter: &super::Iter<'_, Code, BITS>, front: usize) {
    assert_eq!(
        iter.front_next_word_index * crate::WORD_BITS,
        front * usize::from(BITS) + iter.front_pending_bit_count
    );
    assert!(iter.front_pending_bit_count <= crate::WORD_BITS);
}

#[test]
fn mixed_iteration_preserves_order_across_word_boundaries() {
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
        assert_front_window(&iter, 0);

        for (front, &code) in expected[..25].iter().enumerate() {
            assert_eq!(iter.next(), Some(code));
            assert_front_window(&iter, front + 1);
        }
        for &code in expected[75..].iter().rev() {
            assert_eq!(iter.next_back(), Some(code));
            assert_front_window(&iter, 25);
        }
        for (front, &code) in expected[25..75].iter().enumerate() {
            assert_eq!(iter.next(), Some(code));
            assert_front_window(&iter, front + 26);
        }

        assert_front_window(&iter, 75);
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

#[test]
fn rolling_window_handles_word_boundaries_and_mixed_iteration() {
    fn check<const BITS: u8>() {
        let mask = if BITS == 8 {
            u8::MAX
        } else {
            (1u16 << BITS) as u8 - 1
        };
        let codes_per_word = crate::WORD_BITS / usize::from(BITS);
        let codes_per_two_words = (crate::WORD_BITS * 2).div_ceil(usize::from(BITS));
        let lengths = [
            0,
            1,
            codes_per_word.saturating_sub(1),
            codes_per_word,
            codes_per_word + 1,
            codes_per_word * 2 - 1,
            codes_per_word * 2,
            codes_per_word * 2 + 1,
            codes_per_two_words - 1,
            codes_per_two_words,
            codes_per_two_words + 1,
        ];

        for len in lengths {
            let expected = (0..len)
                .map(|index| Code(((index * 17 + 3) as u8) & mask))
                .collect::<alloc::vec::Vec<_>>();
            let value = PackedString::<Code, BITS>::from_chars(expected.iter().copied());
            let mut iter = value.iter();
            let mut front = 0;
            let mut back = len;
            assert_front_window(&iter, front);

            while front < back {
                assert_eq!(iter.next(), Some(expected[front]));
                front += 1;
                assert_front_window(&iter, front);
                if front < back {
                    back -= 1;
                    assert_eq!(iter.next_back(), Some(expected[back]));
                    assert_front_window(&iter, front);
                }
            }

            assert_eq!(iter.next(), None);
            assert_eq!(iter.next_back(), None);
        }
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
