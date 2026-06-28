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

// -- existing ----------------------------------------------------------------

#[divan::bench(name = "starts_with/len_65/yes/ours_str")]
fn s65y_str(b: Bencher) {
    b_str(b, make_case(65, 4));
}
#[divan::bench(name = "starts_with/len_65/yes/ours_string")]
fn s65y_string(b: Bencher) {
    b_string(b, make_case(65, 4));
}
#[divan::bench(name = "starts_with/len_65/yes/string")]
fn s65y_native(b: Bencher) {
    b_native(b, make_case(65, 4));
}

#[divan::bench(name = "starts_with/len_65/no/ours_str")]
fn s65n_str(b: Bencher) {
    b_str(b, no_case(65, 4));
}
#[divan::bench(name = "starts_with/len_65/no/ours_string")]
fn s65n_string(b: Bencher) {
    b_string(b, no_case(65, 4));
}
#[divan::bench(name = "starts_with/len_65/no/string")]
fn s65n_native(b: Bencher) {
    b_native(b, no_case(65, 4));
}

#[divan::bench(name = "starts_with/len_65536/yes/ours_str")]
fn s6y_str(b: Bencher) {
    b_str(b, make_case(65_536, 128));
}
#[divan::bench(name = "starts_with/len_65536/yes/ours_string")]
fn s6y_string(b: Bencher) {
    b_string(b, make_case(65_536, 128));
}
#[divan::bench(name = "starts_with/len_65536/yes/string")]
fn s6y_native(b: Bencher) {
    b_native(b, make_case(65_536, 128));
}

#[divan::bench(name = "starts_with/len_65536/no/ours_str")]
fn s6n_str(b: Bencher) {
    b_str(b, no_case(65_536, 128));
}
#[divan::bench(name = "starts_with/len_65536/no/ours_string")]
fn s6n_string(b: Bencher) {
    b_string(b, no_case(65_536, 128));
}
#[divan::bench(name = "starts_with/len_65536/no/string")]
fn s6n_native(b: Bencher) {
    b_native(b, no_case(65_536, 128));
}

// -- helpers ----------------------------------------------------------------

fn b_str(b: Bencher, c: Case) {
    b.bench(|| black_box(&c.haystack_bits).starts_with_str(black_box(c.prefix_bits.as_bit_str())));
}
fn b_string(b: Bencher, c: Case) {
    b.bench(|| black_box(&c.haystack_bits).starts_with_string(black_box(&c.prefix_bits)));
}
fn b_native(b: Bencher, c: Case) {
    b.bench(|| black_box(&c.haystack_string).starts_with(black_box(&c.prefix_string)));
}
