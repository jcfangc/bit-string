use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

struct Case {
    haystack_bits: BitString,
    prefix_bits: BitString,
    haystack_string: String,
    prefix_string: String,
}

fn make_bits(len: usize) -> BitString {
    let mut bits = BitString::zeros(len);
    for i in 0..len {
        if (i as u64 * 17 + 3) % 7 == 0 {
            bits.set(i, true);
        }
    }
    bits
}

fn make_case(len: usize, pfx: usize) -> Case {
    let h = make_bits(len);
    let p = make_bits(pfx);
    Case {
        haystack_string: h.to_string(),
        prefix_string: p.to_string(),
        haystack_bits: h,
        prefix_bits: p,
    }
}
fn no_case(len: usize, pfx: usize) -> Case {
    let mut h = make_bits(len);
    let p = make_bits(pfx);
    h.set(0, !h.get(0).unwrap());
    Case {
        haystack_string: h.to_string(),
        prefix_string: p.to_string(),
        haystack_bits: h,
        prefix_bits: p,
    }
}

#[divan::bench(name = "starts_with/len_65/yes/bit_string")]
fn s65y(b: Bencher) {
    b_bit(b, make_case(65, 4));
}
#[divan::bench(name = "starts_with/len_65/yes/string")]
fn s65ys(b: Bencher) {
    b_str(b, make_case(65, 4));
}
#[divan::bench(name = "starts_with/len_65/no/bit_string")]
fn s65n(b: Bencher) {
    b_bit(b, no_case(65, 4));
}
#[divan::bench(name = "starts_with/len_65/no/string")]
fn s65ns(b: Bencher) {
    b_str(b, no_case(65, 4));
}
#[divan::bench(name = "starts_with/len_65536/yes/bit_string")]
fn s6y(b: Bencher) {
    b_bit(b, make_case(65_536, 128));
}
#[divan::bench(name = "starts_with/len_65536/yes/string")]
fn s6ys(b: Bencher) {
    b_str(b, make_case(65_536, 128));
}
#[divan::bench(name = "starts_with/len_65536/no/bit_string")]
fn s6n(b: Bencher) {
    b_bit(b, no_case(65_536, 128));
}
#[divan::bench(name = "starts_with/len_65536/no/string")]
fn s6ns(b: Bencher) {
    b_str(b, no_case(65_536, 128));
}

fn b_bit(b: Bencher, c: Case) {
    b.bench(|| black_box(&c.haystack_bits).starts_with(black_box(&c.prefix_bits)));
}
fn b_str(b: Bencher, c: Case) {
    b.bench(|| black_box(&c.haystack_string).starts_with(black_box(&c.prefix_string)));
}
