#[path = "packed_support/mod.rs"]
mod support;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use divan::{Bencher, black_box};
use int_intervals::UsizeCO;
use support::{codes, packed};

const NEEDLE_LENGTH: usize = 8;

fn main() {
    divan::main();
}

fn find_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let start = len / 2;
    let needle = packed::<BITS>(&input[start..start + NEEDLE_LENGTH]);
    bencher.bench(|| black_box(value.find(&needle)));
}

fn find_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value_owner = packed::<BITS>(&input);
    let needle_owner = packed::<BITS>(&input[len / 2..len / 2 + NEEDLE_LENGTH]);
    let value = value_owner.as_packed_str();
    let needle = needle_owner.as_packed_str();
    bencher.bench(|| black_box(value.find(needle)));
}

fn rfind_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let start = len / 2;
    let needle = packed::<BITS>(&input[start..start + NEEDLE_LENGTH]);
    bencher.bench(|| black_box(value.rfind(&needle)));
}

fn rfind_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value_owner = packed::<BITS>(&input);
    let needle_owner = packed::<BITS>(&input[len / 2..len / 2 + NEEDLE_LENGTH]);
    let value = value_owner.as_packed_str();
    let needle = needle_owner.as_packed_str();
    bencher.bench(|| black_box(value.rfind(needle)));
}

fn contains_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let start = len / 2;
    let needle = packed::<BITS>(&input[start..start + NEEDLE_LENGTH]);
    bencher.bench(|| black_box(value.contains(&needle)));
}

fn contains_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value_owner = packed::<BITS>(&input);
    let needle_owner = packed::<BITS>(&input[len / 2..len / 2 + NEEDLE_LENGTH]);
    let value = value_owner.as_packed_str();
    let needle = needle_owner.as_packed_str();
    bencher.bench(|| black_box(value.contains(needle)));
}

fn matches_at_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let start = len / 2;
    let needle = packed::<BITS>(&input[start..start + NEEDLE_LENGTH]);
    bencher.bench(|| black_box(value.matches_at(start, &needle)));
}

fn matches_at_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value_owner = packed::<BITS>(&input);
    let start = len / 2;
    let needle_owner = packed::<BITS>(&input[start..start + NEEDLE_LENGTH]);
    let value = value_owner.as_packed_str();
    let needle = needle_owner.as_packed_str();
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
    let value_owner = packed::<BITS>(&input);
    let prefix_owner = packed::<BITS>(&input[..NEEDLE_LENGTH]);
    let value = value_owner.as_packed_str();
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
    let value_owner = packed::<BITS>(&input);
    let suffix_owner = packed::<BITS>(&input[len - NEEDLE_LENGTH..]);
    let value = value_owner.as_packed_str();
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
    let value_owner = packed::<BITS>(&input);
    let prefix_owner = packed::<BITS>(&input[..NEEDLE_LENGTH]);
    let value = value_owner.as_packed_str();
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
    let value_owner = packed::<BITS>(&input);
    let suffix_owner = packed::<BITS>(&input[len - NEEDLE_LENGTH..]);
    let value = value_owner.as_packed_str();
    let suffix = suffix_owner.as_packed_str();
    bencher.bench(|| black_box(value.strip_suffix(suffix)));
}

fn cmp_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let left_codes = codes(BITS, len);
    let mut right_codes = left_codes.clone();
    right_codes[len / 2] ^= 1;
    let left = packed::<BITS>(&left_codes);
    let right = packed::<BITS>(&right_codes);
    bencher.bench(|| black_box(left.cmp(&right)));
}

fn cmp_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let left_codes = codes(BITS, len);
    let mut right_codes = left_codes.clone();
    right_codes[len / 2] ^= 1;
    let left_owner = packed::<BITS>(&left_codes);
    let right_owner = packed::<BITS>(&right_codes);
    let left = left_owner.as_packed_str();
    let right = right_owner.as_packed_str();
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
    let owner = packed::<BITS>(&codes(BITS, len));
    let value = owner.as_packed_str();
    bencher.bench(|| black_box(value.slice(interval(len / 4, len / 2))));
}

fn to_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let owner = packed::<BITS>(&codes(BITS, len));
    let value = owner.as_packed_str();
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
                                                                        "/ours_packed_string"
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
                                                                        "/ours_packed_str"
                                                                    )
                                                                )]
                        fn ours_packed_str(bencher: divan::Bencher) {
                            crate::$str_fn::<$bits>(bencher, $len);
                        }
                    }
                };
            }

            crate::for_each_packed_case!(define_case);
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

            crate::for_each_packed_case!(define_case);
        }
    };
}

define_two_case_benchmarks!(
    find_cases,
    "packed_matching/find_present_middle",
    find_packed_string,
    find_packed_str
);
define_two_case_benchmarks!(
    rfind_cases,
    "packed_matching/rfind_present_middle",
    rfind_packed_string,
    rfind_packed_str
);
define_two_case_benchmarks!(
    contains_cases,
    "packed_matching/contains_present_middle",
    contains_packed_string,
    contains_packed_str
);
define_two_case_benchmarks!(
    matches_at_cases,
    "packed_matching/matches_at_middle",
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
    "packed_ordering/cmp_middle_difference",
    cmp_packed_string,
    cmp_packed_str
);
define_two_case_benchmarks!(
    slice_cases,
    "packed_slice/slice_middle",
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
