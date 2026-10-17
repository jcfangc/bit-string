use alloc::{vec, vec::Vec};

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

fn deterministic_codes<const BITS: u8>(len: usize, mut state: u64) -> Vec<Code> {
    let mask = if BITS == 8 {
        u8::MAX
    } else {
        ((1u16 << BITS) - 1) as u8
    };
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            Code(state as u8 & mask)
        })
        .collect()
}

fn assert_searches<const BITS: u8>(haystack_codes: &[Code], needle_codes: &[Code]) {
    let haystack = PackedString::<Code, BITS>::from_chars(haystack_codes.iter().copied());
    let needle = PackedString::<Code, BITS>::from_chars(needle_codes.iter().copied());
    let expected_first = expected_find(haystack_codes, needle_codes, false);
    let expected_last = expected_find(haystack_codes, needle_codes, true);

    assert_eq!(haystack.find(&needle), expected_first);
    assert_eq!(haystack.rfind(&needle), expected_last);
    assert_eq!(haystack.contains(&needle), expected_first.is_some());

    for prefix_len in [1, 7, 31, 65] {
        let owner = PackedString::<Code, BITS>::from_chars(
            deterministic_codes::<BITS>(prefix_len, 0x9e37_79b9 ^ prefix_len as u64)
                .into_iter()
                .chain(haystack_codes.iter().copied()),
        );
        let view = owner
            .as_packed_str()
            .slice_from(prefix_len)
            .slice_until(haystack_codes.len());

        let needle_owner = PackedString::<Code, BITS>::from_chars(
            deterministic_codes::<BITS>(3, 0xd1b5_4a32 ^ prefix_len as u64)
                .into_iter()
                .chain(needle_codes.iter().copied()),
        );
        let needle_view = needle_owner
            .as_packed_str()
            .slice_from(3)
            .slice_until(needle_codes.len());

        assert_eq!(
            view.find(needle_view),
            expected_first,
            "prefix={prefix_len}"
        );
        assert_eq!(
            view.rfind(needle_view),
            expected_last,
            "prefix={prefix_len}"
        );
        assert_eq!(
            view.contains(needle_view),
            expected_first.is_some(),
            "prefix={prefix_len}"
        );
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

fn check_long_multivalue_needles<const BITS: u8>() {
    const HAYSTACK_LEN: usize = 320;
    let alphabet_len = 1usize << BITS;
    let alphabet: Vec<_> = (0..alphabet_len).map(|code| Code(code as u8)).collect();
    let mut observed = vec![false; alphabet_len];
    for code in &alphabet {
        observed[usize::from(code.0)] = true;
    }
    assert!(observed.into_iter().all(core::convert::identity));
    assert_searches::<BITS>(&alphabet, &alphabet[..alphabet.len().min(3)]);

    let code_mask = if BITS == 8 {
        u8::MAX
    } else {
        ((1u16 << BITS) - 1) as u8
    };

    for needle_bit_len in [63usize, 64, 65, 127, 128, 129] {
        let needle_len = needle_bit_len.div_ceil(usize::from(BITS));
        let needle_codes =
            deterministic_codes::<BITS>(needle_len, 0xa076_1d64_78bd_642f ^ needle_bit_len as u64);

        for start in [
            0,
            (HAYSTACK_LEN - needle_len) / 2,
            HAYSTACK_LEN - needle_len,
        ] {
            let mut haystack_codes =
                deterministic_codes::<BITS>(HAYSTACK_LEN, 0xe703_7ed1_a0b4_28db);
            haystack_codes[start..start + needle_len].copy_from_slice(&needle_codes);
            assert_searches::<BITS>(&haystack_codes, &needle_codes);
        }

        // All-zero haystack and all-maximum needle guarantee an absent match
        // while exercising the full code alphabet for every supported width.
        let absent_haystack = vec![Code(0); HAYSTACK_LEN];
        let absent_needle = vec![Code(code_mask); needle_len];
        assert_searches::<BITS>(&absent_haystack, &absent_needle);
    }
}

macro_rules! test_long_multivalue_widths {
    ($($name:ident: $bits:literal),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                check_long_multivalue_needles::<$bits>();
            }
        )+
    };
}

test_long_multivalue_widths!(
    long_multivalue_needles_bits_1: 1,
    long_multivalue_needles_bits_2: 2,
    long_multivalue_needles_bits_3: 3,
    long_multivalue_needles_bits_4: 4,
    long_multivalue_needles_bits_5: 5,
    long_multivalue_needles_bits_6: 6,
    long_multivalue_needles_bits_7: 7,
    long_multivalue_needles_bits_8: 8,
);

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
