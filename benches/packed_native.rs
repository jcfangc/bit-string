const NEEDLE_LENGTH: usize = 8;

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
