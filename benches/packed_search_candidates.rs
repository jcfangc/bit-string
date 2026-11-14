const NEEDLE_LEN: usize = 32;
const HIGH_REPEAT_ABSENT_NEEDLE_LEN: usize = NEEDLE_LEN;

#[derive(Clone, Copy)]
enum Position {
    Head,
    Middle,
    Tail,
    Absent,
    HighRepeatAbsent,
}

fn input<const BITS: u8>(
    len: usize,
    needle_len: usize,
    position: Position,
) -> (PackedString<Code, BITS>, PackedString<Code, BITS>) {
    let needle_len = if matches!(position, Position::HighRepeatAbsent) {
        assert!(
            HIGH_REPEAT_ABSENT_NEEDLE_LEN * usize::from(BITS) > u64::BITS as usize,
            "high-repeat absent needle must cross the 64-bit prefix filter"
        );
        HIGH_REPEAT_ABSENT_NEEDLE_LEN
    } else {
        needle_len
    };
    let mask = if BITS == 8 {
        u8::MAX
    } else {
        ((1u16 << BITS) - 1) as u8
    };
    let mut haystack_codes: Vec<u8> = if matches!(position, Position::HighRepeatAbsent) {
        vec![0; len]
    } else {
        (0..len)
            .map(|index| mix64(index as u64 ^ 0xd6e8_feb8_6659_fd93) as u8 & mask)
            .collect()
    };
    let needle_start = match position {
        Position::Head => 0,
        Position::Middle => len / 2 - needle_len / 2,
        Position::Tail => len - needle_len,
        Position::Absent | Position::HighRepeatAbsent => 0,
    };
    let mut needle_seed = 0xa076_1d64_78bd_642fu64;
    let needle_codes = if matches!(position, Position::HighRepeatAbsent) {
        let mut needle_codes = vec![0; needle_len];
        needle_codes[needle_len - 1] = mask;
        needle_codes
    } else {
        loop {
            let candidate: Vec<u8> = (0..needle_len)
            .map(|index| mix64(needle_seed.wrapping_add(index as u64)) as u8 & mask)
            .collect();
            if !matches!(position, Position::Absent)
            || !haystack_codes
                .windows(needle_len)
                .any(|window| window == candidate)
            {
                break candidate;
            }
            needle_seed = needle_seed.wrapping_add(needle_len as u64);
        }
    };
    if !matches!(position, Position::Absent | Position::HighRepeatAbsent) {
        haystack_codes[needle_start..needle_start + needle_len].copy_from_slice(&needle_codes);
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
    needle_len: usize,
    position: Position,
) -> (PackedString<Code, BITS>, PackedString<Code, BITS>) {
    let (haystack, needle) = input::<BITS>(len, needle_len, position);
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
    let (haystack_owner, needle_owner) = offset_owners::<BITS>(len, NEEDLE_LEN, position);
    let haystack = haystack_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    let expected = expected_start(len, NEEDLE_LEN, position);
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
    let (haystack_owner, needle_owner) = offset_owners::<BITS>(len, NEEDLE_LEN, position);
    let haystack = haystack_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    let expected = expected_start(len, NEEDLE_LEN, position);
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
    let (haystack_owner, needle_owner) = offset_owners::<BITS>(len, NEEDLE_LEN, position);
    let haystack = haystack_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    let expected = !matches!(position, Position::Absent | Position::HighRepeatAbsent);
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
    needle_len: usize,
    position: Position,
) {
    let (haystack, needle) = input::<BITS>(len, needle_len, position);
    let expected = expected_start(len, needle_len, position);
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
    needle_len: usize,
    position: Position,
) {
    let (haystack, needle) = input::<BITS>(len, needle_len, position);
    let expected = expected_start(len, needle_len, position);
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
    needle_len: usize,
    position: Position,
) {
    let (haystack, needle) = input::<BITS>(len, needle_len, position);
    let expected = !matches!(position, Position::Absent | Position::HighRepeatAbsent);
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

fn expected_start(len: usize, needle_len: usize, position: Position) -> Option<usize> {
    match position {
        Position::Head => Some(0),
        Position::Middle => Some(len / 2 - needle_len / 2),
        Position::Tail => Some(len - needle_len),
        Position::Absent | Position::HighRepeatAbsent => None,
    }
}

const fn length_dependent_needle_len(bits: u8, haystack_len: usize) -> usize {
    let haystack_len = if haystack_len == 0 { 1 } else { haystack_len };
    let logarithm = (usize::BITS - (haystack_len - 1).leading_zeros()) as usize;
    let bits = bits as usize;
    let length = (logarithm + 10 + bits - 1) / bits;
    if length < 1 {
        1
    } else if length > haystack_len {
        haystack_len
    } else {
        length
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
                            bench_find::<$bits, false>(bencher, $len, NEEDLE_LEN, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates/find/", stringify!($case), "/", $label, "/ours_packed_string"))]
                        fn find_candidate(bencher: Bencher) {
                            bench_find::<$bits, true>(bencher, $len, NEEDLE_LEN, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates/rfind/", stringify!($case), "/", $label, "/direct_reference"))]
                        fn rfind_direct(bencher: Bencher) {
                            bench_rfind::<$bits, false>(bencher, $len, NEEDLE_LEN, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates/rfind/", stringify!($case), "/", $label, "/ours_packed_string"))]
                        fn rfind_candidate(bencher: Bencher) {
                            bench_rfind::<$bits, true>(bencher, $len, NEEDLE_LEN, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates/contains/", stringify!($case), "/", $label, "/direct_reference"))]
                        fn contains_direct(bencher: Bencher) {
                            bench_contains::<$bits, false>(bencher, $len, NEEDLE_LEN, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates/contains/", stringify!($case), "/", $label, "/ours_packed_string"))]
                        fn contains_candidate(bencher: Bencher) {
                            bench_contains::<$bits, true>(bencher, $len, NEEDLE_LEN, $position);
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

macro_rules! define_long_cases {
    ($case:ident, $bits:literal) => {
        mod $case {
            use super::*;

            const LEN: usize = 1_048_576;
            const NEEDLE_LEN: usize = length_dependent_needle_len($bits, LEN);

            macro_rules! define_position {
                ($name:ident, $position:expr, $label:literal) => {
                    mod $name {
                        use super::*;

                        #[divan::bench(name = concat!("packed_search_candidates/find/", stringify!($case), "/", $label, "/direct_reference"))]
                        fn find_direct(bencher: Bencher) {
                            bench_find::<$bits, false>(bencher, LEN, NEEDLE_LEN, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates/find/", stringify!($case), "/", $label, "/ours_packed_string"))]
                        fn find_candidate(bencher: Bencher) {
                            bench_find::<$bits, true>(bencher, LEN, NEEDLE_LEN, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates/rfind/", stringify!($case), "/", $label, "/direct_reference"))]
                        fn rfind_direct(bencher: Bencher) {
                            bench_rfind::<$bits, false>(bencher, LEN, NEEDLE_LEN, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates/rfind/", stringify!($case), "/", $label, "/ours_packed_string"))]
                        fn rfind_candidate(bencher: Bencher) {
                            bench_rfind::<$bits, true>(bencher, LEN, NEEDLE_LEN, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates/contains/", stringify!($case), "/", $label, "/direct_reference"))]
                        fn contains_direct(bencher: Bencher) {
                            bench_contains::<$bits, false>(bencher, LEN, NEEDLE_LEN, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates/contains/", stringify!($case), "/", $label, "/ours_packed_string"))]
                        fn contains_candidate(bencher: Bencher) {
                            bench_contains::<$bits, true>(bencher, LEN, NEEDLE_LEN, $position);
                        }
                    }
                };
            }

            define_position!(head, Position::Head, "head");
            define_position!(middle, Position::Middle, "middle");
            define_position!(tail, Position::Tail, "tail");
            define_position!(absent, Position::Absent, "absent");
            define_position!(
                high_repeat_absent,
                Position::HighRepeatAbsent,
                "high_repeat_absent_needle_32_chars"
            );
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

                        #[divan::bench(name = concat!("packed_search_candidates_offset/find/", stringify!($case), "/", $label, "/ours_packed_str_unaligned"))]
                        fn find_candidate(bencher: Bencher) {
                            bench_find_view::<$bits, true>(bencher, 65_536, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates_offset/rfind/", stringify!($case), "/", $label, "/direct_reference"))]
                        fn rfind_direct(bencher: Bencher) {
                            bench_rfind_view::<$bits, false>(bencher, 65_536, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates_offset/rfind/", stringify!($case), "/", $label, "/ours_packed_str_unaligned"))]
                        fn rfind_candidate(bencher: Bencher) {
                            bench_rfind_view::<$bits, true>(bencher, 65_536, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates_offset/contains/", stringify!($case), "/", $label, "/direct_reference"))]
                        fn contains_direct(bencher: Bencher) {
                            bench_contains_view::<$bits, false>(bencher, 65_536, $position);
                        }

                        #[divan::bench(name = concat!("packed_search_candidates_offset/contains/", stringify!($case), "/", $label, "/ours_packed_str_unaligned"))]
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

crate::for_each_packed_search_bench_case!(define_cases);
crate::for_each_packed_search_offset_bench_case!(define_offset_cases);
define_long_cases!(bits_3_len_1048576_length_dependent_needle, 3);
define_long_cases!(bits_8_len_1048576_length_dependent_needle, 8);
