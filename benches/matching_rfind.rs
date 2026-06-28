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

// -- rfind/len_65 -----------------------------------------------------------

#[divan::bench(name = "rfind/len_65/front/ours_str")]
fn rf65f_str(b: Bencher) {
    b_str(b, make_case(65, 0));
}
#[divan::bench(name = "rfind/len_65/front/ours_string")]
fn rf65f_string(b: Bencher) {
    b_string(b, make_case(65, 0));
}
#[divan::bench(name = "rfind/len_65/front/string")]
fn rf65f_native(b: Bencher) {
    b_native(b, make_case(65, 0));
}

#[divan::bench(name = "rfind/len_65/middle/ours_str")]
fn rf65m_str(b: Bencher) {
    b_str(b, middle_case(65));
}
#[divan::bench(name = "rfind/len_65/middle/ours_string")]
fn rf65m_string(b: Bencher) {
    b_string(b, middle_case(65));
}
#[divan::bench(name = "rfind/len_65/middle/string")]
fn rf65m_native(b: Bencher) {
    b_native(b, middle_case(65));
}

#[divan::bench(name = "rfind/len_65/end/ours_str")]
fn rf65e_str(b: Bencher) {
    b_str(b, end_case(65));
}
#[divan::bench(name = "rfind/len_65/end/ours_string")]
fn rf65e_string(b: Bencher) {
    b_string(b, end_case(65));
}
#[divan::bench(name = "rfind/len_65/end/string")]
fn rf65e_native(b: Bencher) {
    b_native(b, end_case(65));
}

#[divan::bench(name = "rfind/len_65/miss/ours_str")]
fn rf65x_str(b: Bencher) {
    b_str(b, miss_case(65));
}
#[divan::bench(name = "rfind/len_65/miss/ours_string")]
fn rf65x_string(b: Bencher) {
    b_string(b, miss_case(65));
}
#[divan::bench(name = "rfind/len_65/miss/string")]
fn rf65x_native(b: Bencher) {
    b_native(b, miss_case(65));
}

// -- rfind/len_65536 --------------------------------------------------------

#[divan::bench(name = "rfind/len_65536/front/ours_str")]
fn rf6f_str(b: Bencher) {
    b_str(b, make_case(65536, 0));
}
#[divan::bench(name = "rfind/len_65536/front/ours_string")]
fn rf6f_string(b: Bencher) {
    b_string(b, make_case(65536, 0));
}
#[divan::bench(name = "rfind/len_65536/front/string")]
fn rf6f_native(b: Bencher) {
    b_native(b, make_case(65536, 0));
}

#[divan::bench(name = "rfind/len_65536/middle/ours_str")]
fn rf6m_str(b: Bencher) {
    b_str(b, middle_case(65536));
}
#[divan::bench(name = "rfind/len_65536/middle/ours_string")]
fn rf6m_string(b: Bencher) {
    b_string(b, middle_case(65536));
}
#[divan::bench(name = "rfind/len_65536/middle/string")]
fn rf6m_native(b: Bencher) {
    b_native(b, middle_case(65536));
}

#[divan::bench(name = "rfind/len_65536/end/ours_str")]
fn rf6e_str(b: Bencher) {
    b_str(b, end_case(65536));
}
#[divan::bench(name = "rfind/len_65536/end/ours_string")]
fn rf6e_string(b: Bencher) {
    b_string(b, end_case(65536));
}
#[divan::bench(name = "rfind/len_65536/end/string")]
fn rf6e_native(b: Bencher) {
    b_native(b, end_case(65536));
}

#[divan::bench(name = "rfind/len_65536/miss/ours_str")]
fn rf6x_str(b: Bencher) {
    b_str(b, miss_case(65536));
}
#[divan::bench(name = "rfind/len_65536/miss/ours_string")]
fn rf6x_string(b: Bencher) {
    b_string(b, miss_case(65536));
}
#[divan::bench(name = "rfind/len_65536/miss/string")]
fn rf6x_native(b: Bencher) {
    b_native(b, miss_case(65536));
}

// -- helpers ----------------------------------------------------------------

fn b_str(b: Bencher, c: NeedleCase) {
    b.bench(|| black_box(&c.haystack_bits).rfind_str(black_box(c.needle_bits.as_bit_str())));
}
fn b_string(b: Bencher, c: NeedleCase) {
    b.bench(|| black_box(&c.haystack_bits).rfind_string(black_box(&c.needle_bits)));
}
fn b_native(b: Bencher, c: NeedleCase) {
    b.bench(|| black_box(&c.haystack_string).rfind(black_box(&c.needle_string)));
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
