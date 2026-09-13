#[path = "packed_support/mod.rs"]
mod support;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use bit_string::PackedString;
use divan::{Bencher, black_box};
use int_intervals::UsizeCO;
use support::{
    codes, cross_word_index, cross_word_index_for_offset, mix64, packed, unaligned_index,
};

const NEEDLE_LENGTH: usize = 8;

fn main() {
    divan::main();
}

fn find_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input, start) = present_case::<BITS>(len, false);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&needle_input);
    assert_eq!(value.find(&needle), Some(start));
    bencher.bench(|| black_box(value.find(&needle)));
}

fn find_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input, start) = present_case::<BITS>(len, false);
    let value_owner = offset_packed::<BITS>(&input);
    let needle_owner = packed::<BITS>(&needle_input);
    let value = value_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    assert_eq!(value.find(needle), Some(start));
    bencher.bench(|| black_box(value.find(needle)));
}

fn find_absent_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input) = absent_case::<BITS>(len);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&needle_input);
    assert_eq!(value.find(&needle), None);
    bencher.bench(|| black_box(value.find(&needle)));
}

fn find_absent_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input) = absent_case::<BITS>(len);
    let value_owner = offset_packed::<BITS>(&input);
    let needle_owner = packed::<BITS>(&needle_input);
    let value = value_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    assert_eq!(value.find(needle), None);
    bencher.bench(|| black_box(value.find(needle)));
}

fn rfind_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input, start) = present_case::<BITS>(len, true);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&needle_input);
    assert_eq!(value.rfind(&needle), Some(start));
    bencher.bench(|| black_box(value.rfind(&needle)));
}

fn rfind_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input, start) = present_case::<BITS>(len, true);
    let value_owner = offset_packed::<BITS>(&input);
    let needle_owner = packed::<BITS>(&needle_input);
    let value = value_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    assert_eq!(value.rfind(needle), Some(start));
    bencher.bench(|| black_box(value.rfind(needle)));
}

fn rfind_absent_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input) = absent_case::<BITS>(len);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&needle_input);
    assert_eq!(value.rfind(&needle), None);
    bencher.bench(|| black_box(value.rfind(&needle)));
}

fn rfind_absent_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input) = absent_case::<BITS>(len);
    let value_owner = offset_packed::<BITS>(&input);
    let needle_owner = packed::<BITS>(&needle_input);
    let value = value_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    assert_eq!(value.rfind(needle), None);
    bencher.bench(|| black_box(value.rfind(needle)));
}

fn contains_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input, start) = present_case::<BITS>(len, false);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&needle_input);
    assert_eq!(value.find(&needle), Some(start));
    assert!(value.contains(&needle));
    bencher.bench(|| black_box(value.contains(&needle)));
}

fn contains_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input, start) = present_case::<BITS>(len, false);
    let value_owner = offset_packed::<BITS>(&input);
    let needle_owner = packed::<BITS>(&needle_input);
    let value = value_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    assert_eq!(value.find(needle), Some(start));
    assert!(value.contains(needle));
    bencher.bench(|| black_box(value.contains(needle)));
}

fn contains_absent_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input) = absent_case::<BITS>(len);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&needle_input);
    assert!(!value.contains(&needle));
    bencher.bench(|| black_box(value.contains(&needle)));
}

fn contains_absent_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input) = absent_case::<BITS>(len);
    let value_owner = offset_packed::<BITS>(&input);
    let needle_owner = packed::<BITS>(&needle_input);
    let value = value_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    assert!(!value.contains(needle));
    bencher.bench(|| black_box(value.contains(needle)));
}

fn matches_at_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let start = unaligned_index::<BITS>(len);
    let needle_len = NEEDLE_LENGTH.min(len - start);
    let needle = packed::<BITS>(&input[start..start + needle_len]);
    assert!(value.matches_at(start, &needle));
    bencher.bench(|| black_box(value.matches_at(start, &needle)));
}

fn matches_at_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value_owner = offset_packed::<BITS>(&input);
    let start = unaligned_index::<BITS>(len);
    let needle_len = NEEDLE_LENGTH.min(len - start);
    let needle_owner = packed::<BITS>(&input[start..start + needle_len]);
    let value = value_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    assert!(value.matches_at(start, needle));
    bencher.bench(|| black_box(value.matches_at(start, needle)));
}

fn starts_with_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let prefix = packed::<BITS>(&input[..NEEDLE_LENGTH]);
    bencher.bench(|| black_box(value.starts_with(&prefix)));
}

fn starts_with_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value_owner = offset_packed::<BITS>(&input);
    let prefix_owner = packed::<BITS>(&input[..NEEDLE_LENGTH]);
    let value = value_owner.as_packed_str().slice_from(1);
    let prefix = prefix_owner.as_packed_str();
    bencher.bench(|| black_box(value.starts_with(prefix)));
}

fn ends_with_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let suffix = packed::<BITS>(&input[len - NEEDLE_LENGTH..]);
    bencher.bench(|| black_box(value.ends_with(&suffix)));
}

fn ends_with_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value_owner = offset_packed::<BITS>(&input);
    let suffix_owner = packed::<BITS>(&input[len - NEEDLE_LENGTH..]);
    let value = value_owner.as_packed_str().slice_from(1);
    let suffix = suffix_owner.as_packed_str();
    bencher.bench(|| black_box(value.ends_with(suffix)));
}

fn strip_prefix_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let prefix = packed::<BITS>(&input[..NEEDLE_LENGTH]);
    bencher.bench(|| black_box(value.strip_prefix(&prefix)));
}

fn strip_prefix_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value_owner = offset_packed::<BITS>(&input);
    let prefix_owner = packed::<BITS>(&input[..NEEDLE_LENGTH]);
    let value = value_owner.as_packed_str().slice_from(1);
    let prefix = prefix_owner.as_packed_str();
    bencher.bench(|| black_box(value.strip_prefix(prefix)));
}

fn strip_suffix_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let suffix = packed::<BITS>(&input[len - NEEDLE_LENGTH..]);
    bencher.bench(|| black_box(value.strip_suffix(&suffix)));
}

fn strip_suffix_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value_owner = offset_packed::<BITS>(&input);
    let suffix_owner = packed::<BITS>(&input[len - NEEDLE_LENGTH..]);
    let value = value_owner.as_packed_str().slice_from(1);
    let suffix = suffix_owner.as_packed_str();
    bencher.bench(|| black_box(value.strip_suffix(suffix)));
}

fn cmp_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let left_codes = codes(BITS, len);
    let mut right_codes = left_codes.clone();
    right_codes[cross_word_index::<BITS>(len)] ^= 1;
    let left = packed::<BITS>(&left_codes);
    let right = packed::<BITS>(&right_codes);
    bencher.bench(|| black_box(left.cmp(&right)));
}

fn cmp_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let left_codes = codes(BITS, len);
    let mut right_codes = left_codes.clone();
    right_codes[cross_word_index_for_offset(usize::from(BITS), len, usize::from(BITS))] ^= 1;
    let left_owner = offset_packed::<BITS>(&left_codes);
    let right_owner = offset_packed::<BITS>(&right_codes);
    let left = left_owner.as_packed_str().slice_from(1);
    let right = right_owner.as_packed_str().slice_from(1);
    bencher.bench(|| black_box(left.cmp(&right)));
}

fn hash_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = packed::<BITS>(&codes(BITS, len));
    bencher.bench(|| {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        black_box(hasher.finish())
    });
}

fn slice_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = packed::<BITS>(&codes(BITS, len));
    bencher.bench(|| black_box(value.slice(interval(len / 4, len / 2))));
}

fn slice_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let owner = offset_packed::<BITS>(&codes(BITS, len));
    let value = owner.as_packed_str().slice_from(1);
    bencher.bench(|| black_box(value.slice(interval(len / 4, len / 2))));
}

fn to_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let owner = offset_packed::<BITS>(&codes(BITS, len));
    let value = owner.as_packed_str().slice_from(1);
    bencher.bench(|| black_box(value.to_packed_string()));
}

fn replace_interval<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = packed::<BITS>(&codes(BITS, len));
    let replacement = packed::<BITS>(&codes(BITS, len / 4));
    bencher.bench(|| black_box(value.replace_interval(interval(len / 4, len / 4), &replacement)));
}

macro_rules! define_two_case_benchmarks {
    ($module:ident, $scenario:literal, $string_fn:ident, $str_fn:ident) => {
        mod $module {
            macro_rules! define_case {
                ($case:ident, $bits:literal, $len:literal) => {
                    mod $case {
                        #[divan::bench(
                                                                    name = concat!(
                                                                        $scenario,
                                                                        "/",
                                                                        stringify!($case),
                                                                        "/ours_packed_string_aligned"
                                                                    )
                                                                )]
                        fn ours_packed_string(bencher: divan::Bencher) {
                            crate::$string_fn::<$bits>(bencher, $len);
                        }

                        #[divan::bench(
                                                                    name = concat!(
                                                                        $scenario,
                                                                        "/",
                                                                        stringify!($case),
                                                                        "/ours_packed_str_unaligned"
                                                                    )
                                                                )]
                        fn ours_packed_str(bencher: divan::Bencher) {
                            crate::$str_fn::<$bits>(bencher, $len);
                        }
                    }
                };
            }

            crate::for_each_packed_bench_case!(define_case);
        }
    };
}

macro_rules! define_one_case_benchmark {
    ($module:ident, $scenario:literal, $fn_name:ident, $legend:literal) => {
        mod $module {
            macro_rules! define_case {
                ($case:ident, $bits:literal, $len:literal) => {
                    mod $case {
                        #[divan::bench(
                                                                            name = concat!(
                                                                                $scenario,
                                                                                "/",
                                                                                stringify!($case),
                                                                                "/",
                                                                                $legend
                                                                            )
                                                                        )]
                                fn benchmark(bencher: divan::Bencher) {
                                    crate::$fn_name::<$bits>(bencher, $len);
                                }
                            }
                        };
                    }

            crate::for_each_packed_bench_case!(define_case);
        }
    };
}

define_two_case_benchmarks!(
    find_cases,
    "packed_matching/find_present_late",
    find_packed_string,
    find_packed_str
);
define_two_case_benchmarks!(
    find_absent_cases,
    "packed_matching/find_absent",
    find_absent_packed_string,
    find_absent_packed_str
);
define_two_case_benchmarks!(
    rfind_cases,
    "packed_matching/rfind_present_scan_late",
    rfind_packed_string,
    rfind_packed_str
);
define_two_case_benchmarks!(
    rfind_absent_cases,
    "packed_matching/rfind_absent",
    rfind_absent_packed_string,
    rfind_absent_packed_str
);
define_two_case_benchmarks!(
    contains_cases,
    "packed_matching/contains_present_late",
    contains_packed_string,
    contains_packed_str
);
define_two_case_benchmarks!(
    contains_absent_cases,
    "packed_matching/contains_absent",
    contains_absent_packed_string,
    contains_absent_packed_str
);
define_two_case_benchmarks!(
    matches_at_cases,
    "packed_matching/matches_at/unaligned",
    matches_at_packed_string,
    matches_at_packed_str
);
define_two_case_benchmarks!(
    starts_with_cases,
    "packed_matching/starts_with",
    starts_with_packed_string,
    starts_with_packed_str
);
define_two_case_benchmarks!(
    ends_with_cases,
    "packed_matching/ends_with",
    ends_with_packed_string,
    ends_with_packed_str
);
define_two_case_benchmarks!(
    strip_prefix_cases,
    "packed_matching/strip_prefix",
    strip_prefix_packed_string,
    strip_prefix_packed_str
);
define_two_case_benchmarks!(
    strip_suffix_cases,
    "packed_matching/strip_suffix",
    strip_suffix_packed_string,
    strip_suffix_packed_str
);
define_two_case_benchmarks!(
    cmp_cases,
    "packed_ordering/cmp_difference/word_boundary_or_fallback",
    cmp_packed_string,
    cmp_packed_str
);
define_two_case_benchmarks!(
    slice_cases,
    "packed_slice/slice_middle/unaligned_view",
    slice_packed_string,
    slice_packed_str
);

define_one_case_benchmark!(
    hash_cases,
    "packed_hash/hash",
    hash_packed_string,
    "ours_packed_string"
);
define_one_case_benchmark!(
    conversion_cases,
    "packed_conversion/to_packed_string",
    to_packed_string,
    "ours_packed_str"
);
define_one_case_benchmark!(
    replacement_cases,
    "packed_editing/replace_interval",
    replace_interval,
    "ours_packed_string"
);

fn interval(start: usize, len: usize) -> UsizeCO {
    UsizeCO::checked_from_start_len(start, len).unwrap()
}

fn offset_packed<const BITS: u8>(input: &[u8]) -> PackedString<support::Code, BITS> {
    let mut source = Vec::with_capacity(input.len() + 1);
    source.push(0);
    source.extend_from_slice(input);
    packed::<BITS>(&source)
}

fn search_needle_length(bits: u8, len: usize) -> usize {
    let len = len.max(1);
    let logarithm = (usize::BITS - (len - 1).leading_zeros()) as usize;
    let needed_bits = logarithm + 10;
    (needed_bits + usize::from(bits) - 1)
        .div_ceil(usize::from(bits))
        .clamp(1, len)
}

fn present_case<const BITS: u8>(len: usize, from_right: bool) -> (Vec<u8>, Vec<u8>, usize) {
    let needle_len = search_needle_length(BITS, len);
    let start = if from_right { 0 } else { len - needle_len };
    let mask = if BITS == 8 {
        u8::MAX
    } else {
        ((1u16 << BITS) - 1) as u8
    };

    for attempt in 0..1024u64 {
        let needle: Vec<_> = (0..needle_len)
            .map(|index| {
                (mix64(index as u64 ^ attempt.wrapping_mul(0x9e37_79b9_7f4a_7c15)) as u8) & mask
            })
            .collect();
        let mut input = codes(BITS, len);
        input[start..start + needle_len].copy_from_slice(&needle);
        let collision = if from_right {
            (start + 1..=len - needle_len).any(|index| input[index..index + needle_len] == needle)
        } else {
            (0..start).any(|index| input[index..index + needle_len] == needle)
        };
        if !collision {
            return (input, needle, start);
        }
    }
    panic!("could not create a directional unique search needle");
}

fn absent_case<const BITS: u8>(len: usize) -> (Vec<u8>, Vec<u8>) {
    let needle_len = search_needle_length(BITS, len);
    let max_code = if BITS == 8 {
        u8::MAX
    } else {
        ((1u16 << BITS) - 1) as u8
    };
    (vec![0; len], vec![max_code; needle_len])
}
