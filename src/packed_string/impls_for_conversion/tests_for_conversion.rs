use crate::PackedString;
use crate::traits::PackedChar;

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
fn to_vec_matches_codes_across_batches_and_tails() {
    fn check<const BITS: u8>() {
        let mask = crate::code_mask::<BITS>();
        for len in [0, 1, 15, 16, 31, 32, 63, 64, 65, 127, 128, 129, 257] {
            let expected = (0..len)
                .map(|index| Code(((index * 29 + 5) as u8) & mask))
                .collect::<alloc::vec::Vec<_>>();
            let value = PackedString::<Code, BITS>::from_chars(expected.iter().copied());
            assert_eq!(value.to_vec(), expected, "BITS={BITS}, len={len}");
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
