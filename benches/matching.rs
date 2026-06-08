use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

#[derive(Clone, Copy)]
enum CaseKind {
    Prefix,
    Suffix,
    Front,
    Middle,
    End,
    Miss,
}

struct NeedleCase {
    haystack_bits: BitString,
    needle_bits: BitString,
    haystack_string: String,
    needle_string: String,
}

macro_rules! pair_case {
    ($name:ident, $len:expr, $kind:expr, $bit_fn:ident, $string_fn:ident) => {
        #[divan::bench_group]
        mod $name {
            use super::*;

            #[divan::bench]
            fn bit_string(bencher: Bencher) {
                $bit_fn(bencher, $len, $kind);
            }

            #[divan::bench]
            fn string(bencher: Bencher) {
                $string_fn(bencher, $len, $kind);
            }
        }
    };
}

macro_rules! prefix_cases {
    ($bit_fn:ident, $string_fn:ident) => {
        pair_case!(len_65_hit, 65, CaseKind::Prefix, $bit_fn, $string_fn);
        pair_case!(len_65_miss, 65, CaseKind::Miss, $bit_fn, $string_fn);

        pair_case!(len_65536_hit, 65_536, CaseKind::Prefix, $bit_fn, $string_fn);
        pair_case!(len_65536_miss, 65_536, CaseKind::Miss, $bit_fn, $string_fn);
    };
}

macro_rules! suffix_cases {
    ($bit_fn:ident, $string_fn:ident) => {
        pair_case!(len_65_hit, 65, CaseKind::Suffix, $bit_fn, $string_fn);
        pair_case!(len_65_miss, 65, CaseKind::Miss, $bit_fn, $string_fn);

        pair_case!(len_65536_hit, 65_536, CaseKind::Suffix, $bit_fn, $string_fn);
        pair_case!(len_65536_miss, 65_536, CaseKind::Miss, $bit_fn, $string_fn);
    };
}

macro_rules! search_cases {
    ($bit_fn:ident, $string_fn:ident) => {
        pair_case!(len_65_front, 65, CaseKind::Front, $bit_fn, $string_fn);
        pair_case!(len_65_middle, 65, CaseKind::Middle, $bit_fn, $string_fn);
        pair_case!(len_65_end, 65, CaseKind::End, $bit_fn, $string_fn);
        pair_case!(len_65_miss, 65, CaseKind::Miss, $bit_fn, $string_fn);

        pair_case!(
            len_65536_front,
            65_536,
            CaseKind::Front,
            $bit_fn,
            $string_fn
        );
        pair_case!(
            len_65536_middle,
            65_536,
            CaseKind::Middle,
            $bit_fn,
            $string_fn
        );
        pair_case!(len_65536_end, 65_536, CaseKind::End, $bit_fn, $string_fn);
        pair_case!(len_65536_miss, 65_536, CaseKind::Miss, $bit_fn, $string_fn);
    };
}

#[divan::bench_group]
mod starts_with {
    use super::*;
    prefix_cases!(bench_bit_string_starts_with, bench_string_starts_with);
}

#[divan::bench_group]
mod ends_with {
    use super::*;
    suffix_cases!(bench_bit_string_ends_with, bench_string_ends_with);
}

#[divan::bench_group]
mod find {
    use super::*;
    search_cases!(bench_bit_string_find, bench_string_find);
}

#[divan::bench_group]
mod rfind {
    use super::*;
    search_cases!(bench_bit_string_rfind, bench_string_rfind);
}

#[divan::bench_group]
mod strip_prefix {
    use super::*;
    prefix_cases!(bench_bit_string_strip_prefix, bench_string_strip_prefix);
}

#[divan::bench_group]
mod strip_suffix {
    use super::*;
    suffix_cases!(bench_bit_string_strip_suffix, bench_string_strip_suffix);
}

fn bench_bit_string_starts_with(bencher: Bencher, len: usize, kind: CaseKind) {
    let case = make_case(len, kind);
    bench_bits(bencher, case, |haystack, needle| {
        haystack.starts_with(needle)
    });
}

fn bench_string_starts_with(bencher: Bencher, len: usize, kind: CaseKind) {
    let case = make_case(len, kind);
    bench_string(bencher, case, |haystack, needle| {
        haystack.starts_with(needle)
    });
}

fn bench_bit_string_ends_with(bencher: Bencher, len: usize, kind: CaseKind) {
    let case = make_case(len, kind);
    bench_bits(bencher, case, |haystack, needle| haystack.ends_with(needle));
}

fn bench_string_ends_with(bencher: Bencher, len: usize, kind: CaseKind) {
    let case = make_case(len, kind);
    bench_string(bencher, case, |haystack, needle| haystack.ends_with(needle));
}

fn bench_bit_string_find(bencher: Bencher, len: usize, kind: CaseKind) {
    let case = make_case(len, kind);
    bench_bits(bencher, case, |haystack, needle| haystack.find_bits(needle));
}

fn bench_string_find(bencher: Bencher, len: usize, kind: CaseKind) {
    let case = make_case(len, kind);
    bench_string(bencher, case, |haystack, needle| haystack.find(needle));
}

fn bench_bit_string_rfind(bencher: Bencher, len: usize, kind: CaseKind) {
    let case = make_case(len, kind);
    bench_bits(bencher, case, |haystack, needle| {
        haystack.rfind_bits(needle)
    });
}

fn bench_string_rfind(bencher: Bencher, len: usize, kind: CaseKind) {
    let case = make_case(len, kind);
    bench_string(bencher, case, |haystack, needle| haystack.rfind(needle));
}

fn bench_bit_string_strip_prefix(bencher: Bencher, len: usize, kind: CaseKind) {
    let case = make_case(len, kind);
    bench_bits(bencher, case, |haystack, needle| {
        haystack.strip_prefix(needle)
    });
}

fn bench_string_strip_prefix(bencher: Bencher, len: usize, kind: CaseKind) {
    let case = make_case(len, kind);
    bench_string(bencher, case, |haystack, needle| {
        haystack.strip_prefix(needle).map(str::to_owned)
    });
}

fn bench_bit_string_strip_suffix(bencher: Bencher, len: usize, kind: CaseKind) {
    let case = make_case(len, kind);
    bench_bits(bencher, case, |haystack, needle| {
        haystack.strip_suffix(needle)
    });
}

fn bench_string_strip_suffix(bencher: Bencher, len: usize, kind: CaseKind) {
    let case = make_case(len, kind);
    bench_string(bencher, case, |haystack, needle| {
        haystack.strip_suffix(needle).map(str::to_owned)
    });
}

fn bench_bits<R>(
    bencher: Bencher,
    case: NeedleCase,
    f: impl Fn(&BitString, &BitString) -> R + Sync,
) {
    bencher.bench(|| {
        black_box(f(
            black_box(&case.haystack_bits),
            black_box(&case.needle_bits),
        ))
    });
}

fn bench_string<R>(bencher: Bencher, case: NeedleCase, f: impl Fn(&String, &str) -> R + Sync) {
    bencher.bench(|| {
        black_box(f(
            black_box(&case.haystack_string),
            black_box(case.needle_string.as_str()),
        ))
    });
}

fn make_case(len: usize, kind: CaseKind) -> NeedleCase {
    let needle_len = chunk_len(len);

    match kind {
        CaseKind::Prefix | CaseKind::Front => make_needle(len, 0, needle_len),
        CaseKind::Suffix | CaseKind::End => make_needle(len, len - needle_len, needle_len),
        CaseKind::Middle => make_needle(len, (len - needle_len) / 2, needle_len),
        CaseKind::Miss => miss_case(len, needle_len),
    }
}

fn make_needle(len: usize, position: usize, needle_len: usize) -> NeedleCase {
    assert!(
        position <= len,
        "needle position out of bounds: position={position}, len={len}",
    );

    assert!(
        needle_len <= len - position,
        "needle does not fit: position={position}, needle_len={needle_len}, len={len}",
    );

    let needle = make_needle_bits(needle_len);
    let mut haystack = vec![false; len];

    for (offset, &value) in needle.iter().enumerate() {
        haystack[position + offset] = value;
    }

    NeedleCase {
        haystack_string: bools_to_string(&haystack),
        needle_string: bools_to_string(&needle),
        haystack_bits: haystack.into_iter().collect(),
        needle_bits: needle.into_iter().collect(),
    }
}

fn miss_case(len: usize, needle_len: usize) -> NeedleCase {
    NeedleCase {
        haystack_bits: BitString::zeros(len),
        needle_bits: BitString::ones(needle_len),
        haystack_string: "0".repeat(len),
        needle_string: "1".repeat(needle_len),
    }
}

fn make_needle_bits(len: usize) -> Vec<bool> {
    let mut out = Vec::with_capacity(len);

    for index in 0..len {
        let seed = (index as u64).wrapping_add(0x9e37_79b9_7f4a_7c15);
        out.push(mix64(seed) & 7 <= 2);
    }

    if let Some(first) = out.first_mut() {
        *first = true;
    }

    if let Some(last) = out.last_mut() {
        *last = true;
    }

    out
}

fn bools_to_string(values: &[bool]) -> String {
    values.iter().copied().map(bit_char).collect()
}

#[inline]
fn bit_char(value: bool) -> char {
    if value { '1' } else { '0' }
}

#[inline]
fn chunk_len(len: usize) -> usize {
    (len / 8).max(1)
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
