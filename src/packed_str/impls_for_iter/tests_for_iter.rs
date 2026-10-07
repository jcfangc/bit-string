use crate::{PackedStr, PackedString, WORD_BITS, traits::PackedChar};

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
        iter.view.bits.start() + front * usize::from(BITS) + iter.front_cursor.pending_bit_count,
        iter.front_cursor.next_word_index * WORD_BITS
    );
    assert!(iter.front_cursor.pending_bit_count <= WORD_BITS);
}

fn check_mixed_iteration<const BITS: u8>(word_unaligned: bool) {
    let width = usize::from(BITS);
    let mask = if BITS == 8 {
        u8::MAX
    } else {
        (1u16 << BITS) as u8 - 1
    };
    let codes_per_word = WORD_BITS / width;
    let lengths = [
        0,
        1,
        codes_per_word.saturating_sub(1),
        codes_per_word,
        codes_per_word + 1,
        codes_per_word * 2 - 1,
        codes_per_word * 2,
        codes_per_word * 2 + 1,
        95,
    ];

    for len in lengths {
        let expected = (0..len)
            .map(|index| Code(((index * 17 + 3) as u8) & mask))
            .collect::<alloc::vec::Vec<_>>();
        let mut source = alloc::vec::Vec::with_capacity(len + usize::from(word_unaligned));
        if word_unaligned {
            source.push(Code(0));
        }
        source.extend_from_slice(&expected);
        let owner = PackedString::<Code, BITS>::from_chars(source);
        let packed_str = owner.as_packed_str();
        let view: PackedStr<'_, Code, BITS> = if word_unaligned {
            packed_str.slice_from(1)
        } else {
            packed_str
        };
        let mut iter = view.iter();
        let mut front = 0;
        let mut back = len;

        if !word_unaligned || len > 0 {
            assert_eq!(view.bits.start() % width, 0);
            if word_unaligned {
                assert_ne!(view.bits.start() % WORD_BITS, 0);
            } else {
                assert_eq!(view.bits.start(), 0);
            }
            assert_front_window(&iter, front);
        }

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
        assert_eq!(iter.len(), 0);
    }
}

#[test]
fn rolling_window_handles_aligned_and_offset_views_with_mixed_iteration() {
    fn check_width<const BITS: u8>() {
        check_mixed_iteration::<BITS>(false);
        check_mixed_iteration::<BITS>(true);
    }

    check_width::<1>();
    check_width::<2>();
    check_width::<3>();
    check_width::<4>();
    check_width::<5>();
    check_width::<6>();
    check_width::<7>();
    check_width::<8>();
}
