use alloc::vec::Vec;

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

fn codes<const BITS: u8>(mask: usize, len: usize) -> Vec<Code> {
    (0..len)
        .map(|index| Code(((mask >> index) & 1) as u8))
        .collect()
}

fn expected_find(haystack: &[Code], needle: &[Code], reverse: bool) -> Option<usize> {
    if needle.is_empty() {
        return Some(if reverse { haystack.len() } else { 0 });
    }
    if needle.len() > haystack.len() {
        return None;
    }

    let mut starts = 0..=haystack.len() - needle.len();
    if reverse {
        starts
            .rev()
            .find(|&start| haystack[start..].starts_with(needle))
    } else {
        starts.find(|&start| haystack[start..].starts_with(needle))
    }
}

fn check_exhaustive<const BITS: u8>() {
    for haystack_len in 0..=8 {
        for haystack_mask in 0..(1usize << haystack_len) {
            let haystack_codes = codes::<BITS>(haystack_mask, haystack_len);
            let haystack = PackedString::<Code, BITS>::from_chars(haystack_codes.iter().copied());

            // Put both views at a nonzero, character-aligned bit offset. The
            // offset is deliberately not a u64 boundary for BITS=1..=8.
            let mut haystack_owner = PackedString::<Code, BITS>::from_chars(
                core::iter::once(Code(1)).chain(haystack_codes.iter().copied()),
            );
            haystack_owner.push(Code(1));
            let haystack_view = haystack_owner
                .as_packed_str()
                .slice_from(1)
                .slice_until(haystack_len);

            for needle_len in 0..=5 {
                for needle_mask in 0..(1usize << needle_len) {
                    let needle_codes = codes::<BITS>(needle_mask, needle_len);
                    let needle =
                        PackedString::<Code, BITS>::from_chars(needle_codes.iter().copied());
                    let mut needle_owner = PackedString::<Code, BITS>::from_chars(
                        core::iter::once(Code(1)).chain(needle_codes.iter().copied()),
                    );
                    needle_owner.push(Code(1));
                    let needle_view = needle_owner
                        .as_packed_str()
                        .slice_from(1)
                        .slice_until(needle_len);

                    let expected_first = expected_find(&haystack_codes, &needle_codes, false);
                    let expected_last = expected_find(&haystack_codes, &needle_codes, true);
                    assert_eq!(haystack.find(&needle), expected_first);
                    assert_eq!(haystack.rfind(&needle), expected_last);
                    assert_eq!(
                        haystack.contains(&needle),
                        expected_first.is_some() || needle_len == 0
                    );
                    assert_eq!(haystack_view.find(needle_view), expected_first);
                    assert_eq!(haystack_view.rfind(needle_view), expected_last);
                    assert_eq!(
                        haystack_view.contains(needle_view),
                        expected_first.is_some() || needle_len == 0
                    );
                }
            }
        }
    }
}

#[test]
fn contains_handles_simd_threshold_slice_without_a_guard_word() {
    let haystack = PackedString::<Code, 8>::from_chars(core::iter::repeat_n(Code(0), 32));
    let needle = PackedString::<Code, 8>::from_chars([Code(1)]);

    assert!(!haystack.contains(&needle));
    assert!(!haystack.as_packed_str().contains(needle.as_packed_str()));
}

macro_rules! test_widths {
    ($($name:ident: $bits:literal),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                check_exhaustive::<$bits>();
            }
        )+
    };
}

test_widths!(
    bits_1: 1,
    bits_2: 2,
    bits_3: 3,
    bits_4: 4,
    bits_5: 5,
    bits_6: 6,
    bits_7: 7,
    bits_8: 8,
);
