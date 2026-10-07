use crate::{PackedStr, PackedString, traits::PackedChar};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TestChar(u8);

impl<const BITS: u8> PackedChar<BITS> for TestChar {
    fn code(self) -> u8 {
        self.0
    }

    fn from_code(code: u8) -> Option<Self> {
        Some(Self(code))
    }
}

fn check_access<const BITS: u8>() {
    let mask = (1u16 << BITS) - 1;
    let values: alloc::vec::Vec<_> = (0..96)
        .map(|index| TestChar(((index * 29 + 7) as u16 & mask) as u8))
        .collect();
    let packed = PackedString::<TestChar, BITS>::from_chars(values.iter().copied());
    let view: PackedStr<'_, TestChar, BITS> = packed.as_packed_str().slice_from(1);
    assert_eq!(view.bits.start() % usize::from(BITS), 0);
    assert_ne!(view.bits.start() % 64, 0);

    for (index, expected) in values.iter().copied().skip(1).enumerate() {
        assert_eq!(packed.get(index + 1), Some(expected));
        assert_eq!(view.get(index), Some(expected));
    }

    assert_eq!(view.get(view.char_len()), None);
    assert_eq!(packed.get(packed.char_len()), None);
}

#[test]
fn checked_get_supports_all_widths_and_offset_views() {
    check_access::<1>();
    check_access::<2>();
    check_access::<3>();
    check_access::<4>();
    check_access::<5>();
    check_access::<6>();
    check_access::<7>();
    check_access::<8>();
}
