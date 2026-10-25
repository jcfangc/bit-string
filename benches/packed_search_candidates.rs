#[path = "packed_support/mod.rs"]
mod support;

use bit_string::{PackedStr, PackedString};
use divan::{Bencher, black_box};
use support::{Code, mix64, packed};

const NEEDLE_LEN: usize = 32;

fn main() {
    divan::main();
}

#[derive(Clone, Copy)]
enum Position {
    Head,
    Middle,
    Tail,
    Absent,
}

fn input<const BITS: u8>(
    len: usize,
    position: Position,
) -> (PackedString<Code, BITS>, PackedString<Code, BITS>) {
    let mask = if BITS == 8 {
        u8::MAX
    } else {
        ((1u16 << BITS) - 1) as u8
    };
    let mut haystack_codes: Vec<u8> = (0..len)
        .map(|index| mix64(index as u64 ^ 0xd6e8_feb8_6659_fd93) as u8 & mask)
        .collect();
    let needle_start = match position {
        Position::Head => 0,
        Position::Middle => len / 2 - NEEDLE_LEN / 2,
        Position::Tail => len - NEEDLE_LEN,
        Position::Absent => 0,
    };
    let mut needle_seed = 0xa076_1d64_78bd_642fu64;
    let needle_codes = loop {
        let candidate: Vec<u8> = (0..NEEDLE_LEN)
            .map(|index| mix64(needle_seed.wrapping_add(index as u64)) as u8 & mask)
            .collect();
        if !matches!(position, Position::Absent)
            || !haystack_codes
                .windows(NEEDLE_LEN)
                .any(|window| window == candidate)
        {
            break candidate;
        }
        needle_seed = needle_seed.wrapping_add(NEEDLE_LEN as u64);
    };
    if !matches!(position, Position::Absent) {
        haystack_codes[needle_start..needle_start + NEEDLE_LEN].copy_from_slice(&needle_codes);
    }
    (
        packed::<BITS>(&haystack_codes),
        packed::<BITS>(&needle_codes),
    )
}

fn reference_find<const BITS: u8>(
    haystack: &PackedString<Code, BITS>,
    needle: &PackedString<Code, BITS>,
) -> Option<usize> {
    (0..=haystack.char_len().saturating_sub(needle.char_len()))
        .find(|&index| haystack.matches_at(index, needle))
}

fn reference_rfind<const BITS: u8>(
    haystack: &PackedString<Code, BITS>,
    needle: &PackedString<Code, BITS>,
) -> Option<usize> {
    (0..=haystack.char_len().saturating_sub(needle.char_len()))
        .rev()
        .find(|&index| haystack.matches_at(index, needle))
}

fn reference_contains<const BITS: u8>(
    haystack: &PackedString<Code, BITS>,
    needle: &PackedString<Code, BITS>,
) -> bool {
    reference_find(haystack, needle).is_some()
}

fn offset_owners<const BITS: u8>(
    len: usize,
    position: Position,
) -> (PackedString<Code, BITS>, PackedString<Code, BITS>) {
    let (haystack, needle) = input::<BITS>(len, position);
    let mut prefixed_codes = Vec::with_capacity(len + 1);
    prefixed_codes.push(0);
    prefixed_codes.extend(haystack.iter().map(|code| code.0));
    (packed::<BITS>(&prefixed_codes), needle)
}

fn reference_find_view<const BITS: u8>(
    haystack: PackedStr<'_, Code, BITS>,
    needle: PackedStr<'_, Code, BITS>,
) -> Option<usize> {
    (0..=haystack.char_len().saturating_sub(needle.char_len()))
        .find(|&index| haystack.matches_at(index, needle))
}

fn reference_rfind_view<const BITS: u8>(
    haystack: PackedStr<'_, Code, BITS>,
    needle: PackedStr<'_, Code, BITS>,
) -> Option<usize> {
    (0..=haystack.char_len().saturating_sub(needle.char_len()))
        .rev()
        .find(|&index| haystack.matches_at(index, needle))
}

fn bench_find_view<const BITS: u8, const CANDIDATE: bool>(
    bencher: Bencher,
    len: usize,
    position: Position,
) {
    let (haystack_owner, needle_owner) = offset_owners::<BITS>(len, position);
    let haystack = haystack_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    let expected = expected_start(len, position);
    assert_eq!(reference_find_view(haystack, needle), expected);
    assert_eq!(haystack.find(needle), expected);
    bencher.bench(|| {
        let haystack = black_box(haystack);
        let needle = black_box(needle);
        black_box(if CANDIDATE {
            haystack.find(needle)
        } else {
            reference_find_view(haystack, needle)
        })
    });
}

fn bench_rfind_view<const BITS: u8, const CANDIDATE: bool>(
    bencher: Bencher,
    len: usize,
    position: Position,
) {
    let (haystack_owner, needle_owner) = offset_owners::<BITS>(len, position);
    let haystack = haystack_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    let expected = expected_start(len, position);
    assert_eq!(reference_rfind_view(haystack, needle), expected);
    assert_eq!(haystack.rfind(needle), expected);
    bencher.bench(|| {
        let haystack = black_box(haystack);
        let needle = black_box(needle);
        black_box(if CANDIDATE {
            haystack.rfind(needle)
        } else {
            reference_rfind_view(haystack, needle)
        })
    });
}

fn bench_contains_view<const BITS: u8, const CANDIDATE: bool>(
    bencher: Bencher,
    len: usize,
    position: Position,
) {
    let (haystack_owner, needle_owner) = offset_owners::<BITS>(len, position);
    let haystack = haystack_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    let expected = !matches!(position, Position::Absent);
    assert_eq!(reference_find_view(haystack, needle).is_some(), expected);
    assert_eq!(haystack.contains(needle), expected);
    bencher.bench(|| {
        let haystack = black_box(haystack);
        let needle = black_box(needle);
        black_box(if CANDIDATE {
            haystack.contains(needle)
        } else {
            reference_find_view(haystack, needle).is_some()
        })
    });
}

fn bench_find<const BITS: u8, const CANDIDATE: bool>(
    bencher: Bencher,
    len: usize,
    position: Position,
) {
    let (haystack, needle) = input::<BITS>(len, position);
    let expected = expected_start(len, position);
    assert_eq!(reference_find(&haystack, &needle), expected);
    assert_eq!(haystack.find(&needle), expected);
    bencher.bench(|| {
        let haystack = black_box(&haystack);
        let needle = black_box(&needle);
        black_box(if CANDIDATE {
            haystack.find(needle)
        } else {
            reference_find(haystack, needle)
        })
    });
}

fn bench_rfind<const BITS: u8, const CANDIDATE: bool>(
    bencher: Bencher,
    len: usize,
    position: Position,
) {
    let (haystack, needle) = input::<BITS>(len, position);
    let expected = expected_start(len, position);
    assert_eq!(reference_rfind(&haystack, &needle), expected);
    assert_eq!(haystack.rfind(&needle), expected);
    bencher.bench(|| {
        let haystack = black_box(&haystack);
        let needle = black_box(&needle);
        black_box(if CANDIDATE {
            haystack.rfind(needle)
        } else {
            reference_rfind(haystack, needle)
        })
    });
}

fn bench_contains<const BITS: u8, const CANDIDATE: bool>(
    bencher: Bencher,
    len: usize,
    position: Position,
) {
    let (haystack, needle) = input::<BITS>(len, position);
    let expected = !matches!(position, Position::Absent);
    assert_eq!(reference_contains(&haystack, &needle), expected);
    assert_eq!(haystack.contains(&needle), expected);
    bencher.bench(|| {
        let haystack = black_box(&haystack);
        let needle = black_box(&needle);
        black_box(if CANDIDATE {
            haystack.contains(needle)
        } else {
            reference_contains(haystack, needle)
        })
    });
}

fn expected_start(len: usize, position: Position) -> Option<usize> {
    match position {
        Position::Head => Some(0),
        Position::Middle => Some(len / 2 - NEEDLE_LEN / 2),
        Position::Tail => Some(len - NEEDLE_LEN),
        Position::Absent => None,
    }
}

macro_rules! define_cases {
    ($case:ident, $bits:literal, $len:literal) => {
        mod $case {
            use super::*;

            macro_rules! define_position {
                ($name:ident, $position:expr, $label:literal) => {
                    mod $name {
                        use super::*;

                        #[divan::bench(name = concat!("packed_search_candidates/find/", stringify!($case), "/", $label, "/direct_reference"))]
                        fn find_direct(bencher: Bencher) {
                            bench_find::<$bits, false>(bencher, $len, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates/find/", stringify!($case), "/", $label, "/candidate_scan"))]
                        fn find_candidate(bencher: Bencher) {
                            bench_find::<$bits, true>(bencher, $len, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates/rfind/", stringify!($case), "/", $label, "/direct_reference"))]
                        fn rfind_direct(bencher: Bencher) {
                            bench_rfind::<$bits, false>(bencher, $len, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates/rfind/", stringify!($case), "/", $label, "/candidate_scan"))]
                        fn rfind_candidate(bencher: Bencher) {
                            bench_rfind::<$bits, true>(bencher, $len, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates/contains/", stringify!($case), "/", $label, "/direct_reference"))]
                        fn contains_direct(bencher: Bencher) {
                            bench_contains::<$bits, false>(bencher, $len, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates/contains/", stringify!($case), "/", $label, "/candidate_scan"))]
                        fn contains_candidate(bencher: Bencher) {
                            bench_contains::<$bits, true>(bencher, $len, $position);
                        }
                    }
                };
            }

            define_position!(head, Position::Head, "head");
            define_position!(middle, Position::Middle, "middle");
            define_position!(tail, Position::Tail, "tail");
            define_position!(absent, Position::Absent, "absent");
        }
    };
}

macro_rules! define_offset_cases {
    ($case:ident, $bits:literal) => {
        mod $case {
            use super::*;

            macro_rules! define_position {
                ($name:ident, $position:expr, $label:literal) => {
                    mod $name {
                        use super::*;

                        #[divan::bench(name = concat!("packed_search_candidates_offset/find/", stringify!($case), "/", $label, "/direct_reference"))]
                        fn find_direct(bencher: Bencher) {
                            bench_find_view::<$bits, false>(bencher, 65_536, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates_offset/find/", stringify!($case), "/", $label, "/candidate_scan"))]
                        fn find_candidate(bencher: Bencher) {
                            bench_find_view::<$bits, true>(bencher, 65_536, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates_offset/rfind/", stringify!($case), "/", $label, "/direct_reference"))]
                        fn rfind_direct(bencher: Bencher) {
                            bench_rfind_view::<$bits, false>(bencher, 65_536, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates_offset/rfind/", stringify!($case), "/", $label, "/candidate_scan"))]
                        fn rfind_candidate(bencher: Bencher) {
                            bench_rfind_view::<$bits, true>(bencher, 65_536, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates_offset/contains/", stringify!($case), "/", $label, "/direct_reference"))]
                        fn contains_direct(bencher: Bencher) {
                            bench_contains_view::<$bits, false>(bencher, 65_536, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates_offset/contains/", stringify!($case), "/", $label, "/candidate_scan"))]
                        fn contains_candidate(bencher: Bencher) {
                            bench_contains_view::<$bits, true>(bencher, 65_536, $position);
                        }
                    }
                };
            }

            define_position!(head, Position::Head, "head");
            define_position!(middle, Position::Middle, "middle");
            define_position!(tail, Position::Tail, "tail");
            define_position!(absent, Position::Absent, "absent");
        }
    };
}

define_cases!(bits_1_len_64, 1, 64);
define_cases!(bits_2_len_64, 2, 64);
define_cases!(bits_3_len_64, 3, 64);
define_cases!(bits_4_len_64, 4, 64);
define_cases!(bits_5_len_64, 5, 64);
define_cases!(bits_6_len_64, 6, 64);
define_cases!(bits_7_len_64, 7, 64);
define_cases!(bits_8_len_64, 8, 64);
define_cases!(bits_1_len_4096, 1, 4_096);
define_cases!(bits_2_len_4096, 2, 4_096);
define_cases!(bits_3_len_4096, 3, 4_096);
define_cases!(bits_4_len_4096, 4, 4_096);
define_cases!(bits_5_len_4096, 5, 4_096);
define_cases!(bits_6_len_4096, 6, 4_096);
define_cases!(bits_7_len_4096, 7, 4_096);
define_cases!(bits_8_len_4096, 8, 4_096);
define_cases!(bits_1_len_65536, 1, 65_536);
define_cases!(bits_2_len_65536, 2, 65_536);
define_cases!(bits_3_len_65536, 3, 65_536);
define_cases!(bits_4_len_65536, 4, 65_536);
define_cases!(bits_5_len_65536, 5, 65_536);
define_cases!(bits_6_len_65536, 6, 65_536);
define_cases!(bits_7_len_65536, 7, 65_536);
define_cases!(bits_8_len_65536, 8, 65_536);

define_offset_cases!(offset_bits_1_len_65536, 1);
define_offset_cases!(offset_bits_2_len_65536, 2);
define_offset_cases!(offset_bits_3_len_65536, 3);
define_offset_cases!(offset_bits_4_len_65536, 4);
define_offset_cases!(offset_bits_5_len_65536, 5);
define_offset_cases!(offset_bits_6_len_65536, 6);
define_offset_cases!(offset_bits_7_len_65536, 7);
define_offset_cases!(offset_bits_8_len_65536, 8);
