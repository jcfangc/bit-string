const NEEDLE_LENGTH: usize = 8;

fn find_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input, start) = present_case::<BITS>(len, false);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&needle_input);
    assert_eq!(value.find(&needle), Some(start));
    bencher.bench(|| black_box(black_box(&value).find(black_box(&needle))));
}

fn find_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input, start) = present_case::<BITS>(len, false);
    let value_owner = offset_packed::<BITS>(&input);
    let needle_owner = packed::<BITS>(&needle_input);
    let value = value_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    assert_eq!(value.find(needle), Some(start));
    bencher.bench(|| black_box(black_box(&value).find(black_box(needle))));
}

fn find_absent_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input) = absent_case::<BITS>(len);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&needle_input);
    assert_eq!(value.find(&needle), None);
    bencher.bench(|| black_box(black_box(&value).find(black_box(&needle))));
}

fn find_absent_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input) = absent_case::<BITS>(len);
    let value_owner = offset_packed::<BITS>(&input);
    let needle_owner = packed::<BITS>(&needle_input);
    let value = value_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    assert_eq!(value.find(needle), None);
    bencher.bench(|| black_box(black_box(&value).find(black_box(needle))));
}

fn rfind_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input, start) = present_case::<BITS>(len, true);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&needle_input);
    assert_eq!(value.rfind(&needle), Some(start));
    bencher.bench(|| black_box(black_box(&value).rfind(black_box(&needle))));
}

fn rfind_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input, start) = present_case::<BITS>(len, true);
    let value_owner = offset_packed::<BITS>(&input);
    let needle_owner = packed::<BITS>(&needle_input);
    let value = value_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    assert_eq!(value.rfind(needle), Some(start));
    bencher.bench(|| black_box(black_box(&value).rfind(black_box(needle))));
}

fn rfind_absent_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input) = absent_case::<BITS>(len);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&needle_input);
    assert_eq!(value.rfind(&needle), None);
    bencher.bench(|| black_box(black_box(&value).rfind(black_box(&needle))));
}

fn rfind_absent_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input) = absent_case::<BITS>(len);
    let value_owner = offset_packed::<BITS>(&input);
    let needle_owner = packed::<BITS>(&needle_input);
    let value = value_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    assert_eq!(value.rfind(needle), None);
    bencher.bench(|| black_box(black_box(&value).rfind(black_box(needle))));
}

fn contains_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input, start) = present_case::<BITS>(len, false);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&needle_input);
    assert_eq!(value.find(&needle), Some(start));
    assert!(value.contains(&needle));
    bencher.bench(|| black_box(black_box(&value).contains(black_box(&needle))));
}

fn contains_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input, start) = present_case::<BITS>(len, false);
    let value_owner = offset_packed::<BITS>(&input);
    let needle_owner = packed::<BITS>(&needle_input);
    let value = value_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    assert_eq!(value.find(needle), Some(start));
    assert!(value.contains(needle));
    bencher.bench(|| black_box(black_box(&value).contains(black_box(needle))));
}

fn contains_absent_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input) = absent_case::<BITS>(len);
    let value = packed::<BITS>(&input);
    let needle = packed::<BITS>(&needle_input);
    assert!(!value.contains(&needle));
    bencher.bench(|| black_box(black_box(&value).contains(black_box(&needle))));
}

fn contains_absent_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let (input, needle_input) = absent_case::<BITS>(len);
    let value_owner = offset_packed::<BITS>(&input);
    let needle_owner = packed::<BITS>(&needle_input);
    let value = value_owner.as_packed_str().slice_from(1);
    let needle = needle_owner.as_packed_str();
    assert!(!value.contains(needle));
    bencher.bench(|| black_box(black_box(&value).contains(black_box(needle))));
}

fn matches_at_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let start = unaligned_index::<BITS>(len);
    let needle_len = NEEDLE_LENGTH.min(len - start);
    let needle = packed::<BITS>(&input[start..start + needle_len]);
    assert!(value.matches_at(start, &needle));
    bencher.bench(|| black_box(black_box(&value).matches_at(black_box(start), black_box(&needle))));
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
    bencher.bench(|| black_box(black_box(&value).matches_at(black_box(start), black_box(needle))));
}

fn starts_with_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let prefix = packed::<BITS>(&input[..NEEDLE_LENGTH]);
    bencher.bench(|| black_box(black_box(&value).starts_with(black_box(&prefix))));
}

fn starts_with_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value_owner = offset_packed::<BITS>(&input);
    let prefix_owner = packed::<BITS>(&input[..NEEDLE_LENGTH]);
    let value = value_owner.as_packed_str().slice_from(1);
    let prefix = prefix_owner.as_packed_str();
    bencher.bench(|| black_box(black_box(&value).starts_with(black_box(prefix))));
}

fn ends_with_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let suffix = packed::<BITS>(&input[len - NEEDLE_LENGTH..]);
    bencher.bench(|| black_box(black_box(&value).ends_with(black_box(&suffix))));
}

fn ends_with_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value_owner = offset_packed::<BITS>(&input);
    let suffix_owner = packed::<BITS>(&input[len - NEEDLE_LENGTH..]);
    let value = value_owner.as_packed_str().slice_from(1);
    let suffix = suffix_owner.as_packed_str();
    bencher.bench(|| black_box(black_box(&value).ends_with(black_box(suffix))));
}

fn strip_prefix_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let prefix = packed::<BITS>(&input[..NEEDLE_LENGTH]);
    bencher.bench(|| black_box(black_box(&value).strip_prefix(black_box(&prefix))));
}

fn strip_prefix_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value_owner = offset_packed::<BITS>(&input);
    let prefix_owner = packed::<BITS>(&input[..NEEDLE_LENGTH]);
    let value = value_owner.as_packed_str().slice_from(1);
    let prefix = prefix_owner.as_packed_str();
    bencher.bench(|| black_box(black_box(&value).strip_prefix(black_box(prefix))));
}

fn strip_suffix_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let suffix = packed::<BITS>(&input[len - NEEDLE_LENGTH..]);
    bencher.bench(|| black_box(black_box(&value).strip_suffix(black_box(&suffix))));
}

fn strip_suffix_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value_owner = offset_packed::<BITS>(&input);
    let suffix_owner = packed::<BITS>(&input[len - NEEDLE_LENGTH..]);
    let value = value_owner.as_packed_str().slice_from(1);
    let suffix = suffix_owner.as_packed_str();
    bencher.bench(|| black_box(black_box(&value).strip_suffix(black_box(suffix))));
}

macro_rules! define_two_case_benchmarks {
    ($module:ident, $scenario:literal, $case_macro:ident, $string_fn:ident, $str_fn:ident) => {
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

            crate::$case_macro!(define_case);
        }
    };
}

define_two_case_benchmarks!(
    find_cases,
    "packed_matching/find_present_late",
    for_each_packed_matching_search_bench_case,
    find_packed_string,
    find_packed_str
);
define_two_case_benchmarks!(
    find_absent_cases,
    "packed_matching/find_absent_early_reject",
    for_each_packed_matching_search_bench_case,
    find_absent_packed_string,
    find_absent_packed_str
);
define_two_case_benchmarks!(
    rfind_cases,
    "packed_matching/rfind_present_scan_late",
    for_each_packed_matching_search_bench_case,
    rfind_packed_string,
    rfind_packed_str
);
define_two_case_benchmarks!(
    rfind_absent_cases,
    "packed_matching/rfind_absent_early_reject",
    for_each_packed_matching_search_bench_case,
    rfind_absent_packed_string,
    rfind_absent_packed_str
);
define_two_case_benchmarks!(
    contains_cases,
    "packed_matching/contains_present_late",
    for_each_packed_matching_search_bench_case,
    contains_packed_string,
    contains_packed_str
);
define_two_case_benchmarks!(
    contains_absent_cases,
    "packed_matching/contains_absent_early_reject",
    for_each_packed_matching_search_bench_case,
    contains_absent_packed_string,
    contains_absent_packed_str
);
define_two_case_benchmarks!(
    matches_at_cases,
    "packed_matching/matches_at/unaligned",
    for_each_packed_matching_exact_bench_case,
    matches_at_packed_string,
    matches_at_packed_str
);
define_two_case_benchmarks!(
    starts_with_cases,
    "packed_matching/starts_with",
    for_each_packed_matching_boundary_bench_case,
    starts_with_packed_string,
    starts_with_packed_str
);
define_two_case_benchmarks!(
    ends_with_cases,
    "packed_matching/ends_with",
    for_each_packed_matching_boundary_bench_case,
    ends_with_packed_string,
    ends_with_packed_str
);
define_two_case_benchmarks!(
    strip_prefix_cases,
    "packed_matching/strip_prefix",
    for_each_packed_matching_boundary_bench_case,
    strip_prefix_packed_string,
    strip_prefix_packed_str
);
define_two_case_benchmarks!(
    strip_suffix_cases,
    "packed_matching/strip_suffix",
    for_each_packed_matching_boundary_bench_case,
    strip_suffix_packed_string,
    strip_suffix_packed_str
);
fn offset_packed<const BITS: u8>(input: &[u8]) -> PackedString<crate::support::Code, BITS> {
    let mut source = Vec::with_capacity(input.len() + 1);
    source.push(0);
    source.extend_from_slice(input);
    packed::<BITS>(&source)
}

fn search_needle_length(bits: u8, len: usize) -> usize {
    let len = len.max(1);
    let logarithm = (usize::BITS - (len - 1).leading_zeros()) as usize;
    let needed_bits = logarithm + 10;
    needed_bits.div_ceil(usize::from(bits)).clamp(1, len)
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
