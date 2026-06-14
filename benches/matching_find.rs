use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

struct NeedleCase {
    haystack_bits: BitString,
    needle_bits: BitString,
    haystack_string: String,
    needle_string: String,
}

#[divan::bench(name = "find/len_65/front/bit_string")]
fn find_len_65_front_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, make_case(65, 0));
}

#[divan::bench(name = "find/len_65/front/string")]
fn find_len_65_front_string(bencher: Bencher) {
    bench_string(bencher, make_case(65, 0));
}

#[divan::bench(name = "find/len_65/middle/bit_string")]
fn find_len_65_middle_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, middle_case(65));
}

#[divan::bench(name = "find/len_65/middle/string")]
fn find_len_65_middle_string(bencher: Bencher) {
    bench_string(bencher, middle_case(65));
}

#[divan::bench(name = "find/len_65/end/bit_string")]
fn find_len_65_end_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, end_case(65));
}

#[divan::bench(name = "find/len_65/end/string")]
fn find_len_65_end_string(bencher: Bencher) {
    bench_string(bencher, end_case(65));
}

#[divan::bench(name = "find/len_65/miss/bit_string")]
fn find_len_65_miss_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, miss_case(65));
}

#[divan::bench(name = "find/len_65/miss/string")]
fn find_len_65_miss_string(bencher: Bencher) {
    bench_string(bencher, miss_case(65));
}

#[divan::bench(name = "find/len_65536/front/bit_string")]
fn find_len_65536_front_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, make_case(65_536, 0));
}

#[divan::bench(name = "find/len_65536/front/string")]
fn find_len_65536_front_string(bencher: Bencher) {
    bench_string(bencher, make_case(65_536, 0));
}

#[divan::bench(name = "find/len_65536/middle/bit_string")]
fn find_len_65536_middle_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, middle_case(65_536));
}

#[divan::bench(name = "find/len_65536/middle/string")]
fn find_len_65536_middle_string(bencher: Bencher) {
    bench_string(bencher, middle_case(65_536));
}

#[divan::bench(name = "find/len_65536/end/bit_string")]
fn find_len_65536_end_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, end_case(65_536));
}

#[divan::bench(name = "find/len_65536/end/string")]
fn find_len_65536_end_string(bencher: Bencher) {
    bench_string(bencher, end_case(65_536));
}

#[divan::bench(name = "find/len_65536/miss/bit_string")]
fn find_len_65536_miss_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, miss_case(65_536));
}

#[divan::bench(name = "find/len_65536/miss/string")]
fn find_len_65536_miss_string(bencher: Bencher) {
    bench_string(bencher, miss_case(65_536));
}

fn bench_bit_string(bencher: Bencher, case: NeedleCase) {
    bencher.bench(|| black_box(&case.haystack_bits).find(black_box(&case.needle_bits)));
}

fn bench_string(bencher: Bencher, case: NeedleCase) {
    bencher.bench(|| black_box(&case.haystack_string).find(black_box(&case.needle_string)));
}

fn middle_case(len: usize) -> NeedleCase {
    let needle_len = chunk_len(len);
    make_case(len, (len - needle_len) / 2)
}

fn end_case(len: usize) -> NeedleCase {
    let needle_len = chunk_len(len);
    make_case(len, len - needle_len)
}

fn miss_case(len: usize) -> NeedleCase {
    let haystack = vec![false; len];
    let needle = vec![true; chunk_len(len)];

    NeedleCase {
        haystack_bits: haystack.iter().copied().collect(),
        needle_bits: needle.iter().copied().collect(),
        haystack_string: bools_to_string(&haystack),
        needle_string: bools_to_string(&needle),
    }
}

fn make_case(len: usize, position: usize) -> NeedleCase {
    let needle = make_needle_bits(chunk_len(len));
    let mut haystack = vec![false; len];

    for (offset, &value) in needle.iter().enumerate() {
        haystack[position + offset] = value;
    }

    NeedleCase {
        haystack_bits: haystack.iter().copied().collect(),
        needle_bits: needle.iter().copied().collect(),
        haystack_string: bools_to_string(&haystack),
        needle_string: bools_to_string(&needle),
    }
}

#[inline]
fn chunk_len(len: usize) -> usize {
    (len / 8).max(1)
}

fn make_needle_bits(len: usize) -> Vec<bool> {
    let mut out = Vec::with_capacity(len);

    for index in 0..len {
        out.push(mix64((index as u64).wrapping_add(0x9e37_79b9_7f4a_7c15)) & 7 <= 2);
    }

    out[0] = true;
    out[len - 1] = true;

    out
}

fn bools_to_string(values: &[bool]) -> String {
    let mut out = String::with_capacity(values.len());

    for &value in values {
        out.push(if value { '1' } else { '0' });
    }

    out
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
