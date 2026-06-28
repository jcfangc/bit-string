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

// -- find/len_65 ------------------------------------------------------------

#[divan::bench(name = "find_str/len_65/front/ours_string_str")]
fn find_65_front_str(b: Bencher) {
    bench_find_str(b, make_case(65, 0));
}
#[divan::bench(name = "find_str/len_65/front/ours_string_string")]
fn find_65_front_string(b: Bencher) {
    bench_find_string(b, make_case(65, 0));
}
#[divan::bench(name = "find_str/len_65/front/string")]
fn find_65_front_native(b: Bencher) {
    bench_native_find(b, make_case(65, 0));
}

#[divan::bench(name = "find_str/len_65/middle/ours_string_str")]
fn find_65_middle_str(b: Bencher) {
    bench_find_str(b, middle_case(65));
}
#[divan::bench(name = "find_str/len_65/middle/ours_string_string")]
fn find_65_middle_string(b: Bencher) {
    bench_find_string(b, middle_case(65));
}
#[divan::bench(name = "find_str/len_65/middle/string")]
fn find_65_middle_native(b: Bencher) {
    bench_native_find(b, middle_case(65));
}

#[divan::bench(name = "find_str/len_65/end/ours_string_str")]
fn find_65_end_str(b: Bencher) {
    bench_find_str(b, end_case(65));
}
#[divan::bench(name = "find_str/len_65/end/ours_string_string")]
fn find_65_end_string(b: Bencher) {
    bench_find_string(b, end_case(65));
}
#[divan::bench(name = "find_str/len_65/end/string")]
fn find_65_end_native(b: Bencher) {
    bench_native_find(b, end_case(65));
}

#[divan::bench(name = "find_str/len_65/miss/ours_string_str")]
fn find_65_miss_str(b: Bencher) {
    bench_find_str(b, miss_case(65));
}
#[divan::bench(name = "find_str/len_65/miss/ours_string_string")]
fn find_65_miss_string(b: Bencher) {
    bench_find_string(b, miss_case(65));
}
#[divan::bench(name = "find_str/len_65/miss/string")]
fn find_65_miss_native(b: Bencher) {
    bench_native_find(b, miss_case(65));
}

// -- find/len_65536 ---------------------------------------------------------

#[divan::bench(name = "find_str/len_65536/front/ours_string_str")]
fn find_65536_front_str(b: Bencher) {
    bench_find_str(b, make_case(65536, 0));
}
#[divan::bench(name = "find_str/len_65536/front/ours_string_string")]
fn find_65536_front_string(b: Bencher) {
    bench_find_string(b, make_case(65536, 0));
}
#[divan::bench(name = "find_str/len_65536/front/string")]
fn find_65536_front_native(b: Bencher) {
    bench_native_find(b, make_case(65536, 0));
}

#[divan::bench(name = "find_str/len_65536/middle/ours_string_str")]
fn find_65536_middle_str(b: Bencher) {
    bench_find_str(b, middle_case(65536));
}
#[divan::bench(name = "find_str/len_65536/middle/ours_string_string")]
fn find_65536_middle_string(b: Bencher) {
    bench_find_string(b, middle_case(65536));
}
#[divan::bench(name = "find_str/len_65536/middle/string")]
fn find_65536_middle_native(b: Bencher) {
    bench_native_find(b, middle_case(65536));
}

#[divan::bench(name = "find_str/len_65536/end/ours_string_str")]
fn find_65536_end_str(b: Bencher) {
    bench_find_str(b, end_case(65536));
}
#[divan::bench(name = "find_str/len_65536/end/ours_string_string")]
fn find_65536_end_string(b: Bencher) {
    bench_find_string(b, end_case(65536));
}
#[divan::bench(name = "find_str/len_65536/end/string")]
fn find_65536_end_native(b: Bencher) {
    bench_native_find(b, end_case(65536));
}

#[divan::bench(name = "find_str/len_65536/miss/ours_string_str")]
fn find_65536_miss_str(b: Bencher) {
    bench_find_str(b, miss_case(65536));
}
#[divan::bench(name = "find_str/len_65536/miss/ours_string_string")]
fn find_65536_miss_string(b: Bencher) {
    bench_find_string(b, miss_case(65536));
}
#[divan::bench(name = "find_str/len_65536/miss/string")]
fn find_65536_miss_native(b: Bencher) {
    bench_native_find(b, miss_case(65536));
}

// -- contains/len_65 --------------------------------------------------------

#[divan::bench(name = "contains_str/len_65/yes/ours_string_str")]
fn contains_65_yes_str(b: Bencher) {
    bench_contains_str(b, make_case(65, 0));
}
#[divan::bench(name = "contains_str/len_65/yes/ours_string_string")]
fn contains_65_yes_string(b: Bencher) {
    bench_contains_string(b, make_case(65, 0));
}
#[divan::bench(name = "contains_str/len_65/yes/string")]
fn contains_65_yes_native(b: Bencher) {
    bench_native_contains(b, make_case(65, 0));
}

#[divan::bench(name = "contains_str/len_65/no/ours_string_str")]
fn contains_65_no_str(b: Bencher) {
    bench_contains_str(b, miss_case(65));
}
#[divan::bench(name = "contains_str/len_65/no/ours_string_string")]
fn contains_65_no_string(b: Bencher) {
    bench_contains_string(b, miss_case(65));
}
#[divan::bench(name = "contains_str/len_65/no/string")]
fn contains_65_no_native(b: Bencher) {
    bench_native_contains(b, miss_case(65));
}

#[divan::bench(name = "contains_str/len_65536/yes/ours_string_str")]
fn contains_65536_yes_str(b: Bencher) {
    bench_contains_str(b, make_case(65536, 0));
}
#[divan::bench(name = "contains_str/len_65536/yes/ours_string_string")]
fn contains_65536_yes_string(b: Bencher) {
    bench_contains_string(b, make_case(65536, 0));
}
#[divan::bench(name = "contains_str/len_65536/yes/string")]
fn contains_65536_yes_native(b: Bencher) {
    bench_native_contains(b, make_case(65536, 0));
}

#[divan::bench(name = "contains_str/len_65536/no/ours_string_str")]
fn contains_65536_no_str(b: Bencher) {
    bench_contains_str(b, miss_case(65536));
}
#[divan::bench(name = "contains_str/len_65536/no/ours_string_string")]
fn contains_65536_no_string(b: Bencher) {
    bench_contains_string(b, miss_case(65536));
}
#[divan::bench(name = "contains_str/len_65536/no/string")]
fn contains_65536_no_native(b: Bencher) {
    bench_native_contains(b, miss_case(65536));
}

// -- helpers ----------------------------------------------------------------

fn bench_find_str(b: Bencher, c: NeedleCase) {
    b.bench(|| black_box(&c.haystack_bits).find_str(black_box(c.needle_bits.as_bit_str())));
}
fn bench_find_string(b: Bencher, c: NeedleCase) {
    b.bench(|| black_box(&c.haystack_bits).find_string(black_box(&c.needle_bits)));
}
fn bench_contains_str(b: Bencher, c: NeedleCase) {
    b.bench(|| black_box(&c.haystack_bits).contains_str(black_box(c.needle_bits.as_bit_str())));
}
fn bench_contains_string(b: Bencher, c: NeedleCase) {
    b.bench(|| black_box(&c.haystack_bits).contains_string(black_box(&c.needle_bits)));
}
fn bench_native_find(b: Bencher, c: NeedleCase) {
    b.bench(|| black_box(&c.haystack_string).find(black_box(&c.needle_string)));
}
fn bench_native_contains(b: Bencher, c: NeedleCase) {
    b.bench(|| black_box(&c.haystack_string).contains(black_box(&c.needle_string)));
}

fn middle_case(len: usize) -> NeedleCase {
    let n = chunk_len(len);
    make_case(len, (len - n) / 2)
}
fn end_case(len: usize) -> NeedleCase {
    let n = chunk_len(len);
    make_case(len, len - n)
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
fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
