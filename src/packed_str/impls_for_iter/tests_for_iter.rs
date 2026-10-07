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

fn check_mixed_iteration<const BITS: u8>() {
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
        let mut source = alloc::vec::Vec::with_capacity(len + 1);
        source.push(Code(0));
        source.extend_from_slice(&expected);
        let owner = PackedString::<Code, BITS>::from_chars(source);
        let view: PackedStr<'_, Code, BITS> = owner.as_packed_str().slice_from(1);
        let mut iter = view.iter();
        let mut front = 0;
        let mut back = len;

        if len > 0 {
            assert_eq!(view.bits.start() % width, 0);
            assert_ne!(view.bits.start() % WORD_BITS, 0);
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
fn rolling_window_handles_offset_views_and_mixed_iteration() {
    check_mixed_iteration::<1>();
    check_mixed_iteration::<2>();
    check_mixed_iteration::<3>();
    check_mixed_iteration::<4>();
    check_mixed_iteration::<5>();
    check_mixed_iteration::<6>();
    check_mixed_iteration::<7>();
    check_mixed_iteration::<8>();
}
