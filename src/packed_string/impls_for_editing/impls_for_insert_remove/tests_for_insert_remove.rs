use crate::{PackedString, code_mask, traits::PackedChar};
use alloc::vec::Vec;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Code(u8);

impl<const BITS: u8> PackedChar<BITS> for Code {
    fn code(self) -> u8 {
        self.0
    }
    fn from_code(code: u8) -> Option<Self> {
        (code <= code_mask::<BITS>()).then_some(Self(code))
    }
}

fn check<const BITS: u8>() {
    for len in [0, 1, 7, 8, 15, 16, 31, 32, 63, 64, 65, 127, 128, 129] {
        let mut state = 0x1234_5678_9abc_def0u64;
        let expected = (0..len)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                Code(state as u8 & code_mask::<BITS>())
            })
            .collect::<Vec<_>>();
        let source = PackedString::<Code, BITS>::from_chars(expected.iter().copied());
        for index in 0..=len + 1 {
            for code in [Code(0), Code(code_mask::<BITS>())] {
                let mut actual = source.clone();
                let mut model = expected.clone();
                let at = index.min(len);
                model.insert(at, code);
                actual.insert(index, code);
                assert_eq!(actual.iter().collect::<Vec<_>>(), model);
                assert_eq!(
                    actual.bits(),
                    PackedString::<Code, BITS>::from_chars(model).bits()
                );
                assert_eq!(actual.remove(at), code);
                assert_eq!(actual.bits(), source.bits());
            }
            if index < len {
                let mut actual = source.clone();
                let mut model = expected.clone();
                assert_eq!(actual.remove(index), model.remove(index));
                assert_eq!(
                    actual.bits(),
                    PackedString::<Code, BITS>::from_chars(model).bits()
                );
            }
        }
    }
}

#[test]
fn fixed_width_edits_match_character_model() {
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
fn remove_out_of_bounds_preserves_payload() {
    extern crate std;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let mut actual = PackedString::<Code, 5>::from_chars([Code(7), Code(31)]);
    let original = actual.bits().clone();
    for index in [2, usize::MAX] {
        assert!(catch_unwind(AssertUnwindSafe(|| actual.remove(index))).is_err());
        assert_eq!(actual.bits(), &original);
    }
}
