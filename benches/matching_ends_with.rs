use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

struct Case {
    haystack_bits: BitString,
    suffix_bits: BitString,
    haystack_string: String,
    suffix_string: String,
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
fn make_case(len: usize, sfx: usize) -> Case {
    let h = make_bits(len);
    let s = make_bits(sfx);
    Case {
        haystack_string: h.to_string(),
        suffix_string: s.to_string(),
        haystack_bits: h,
        suffix_bits: s,
    }
}
fn no_case(len: usize, sfx: usize) -> Case {
    let mut h = make_bits(len);
    let s = make_bits(sfx);
    let last = h.bit_len() - 1;
    h.set(last, !h.get(last).unwrap());
    Case {
        haystack_string: h.to_string(),
        suffix_string: s.to_string(),
        haystack_bits: h,
        suffix_bits: s,
    }
}

#[divan::bench(name = "ends_with/len_65/yes/ours_str")]
fn e65y_str(b: Bencher) {
    b_str(b, make_case(65, 4));
}
#[divan::bench(name = "ends_with/len_65/yes/ours_string")]
fn e65y_string(b: Bencher) {
    b_string(b, make_case(65, 4));
}
#[divan::bench(name = "ends_with/len_65/yes/string")]
fn e65y_native(b: Bencher) {
    b_native(b, make_case(65, 4));
}

#[divan::bench(name = "ends_with/len_65/no/ours_str")]
fn e65n_str(b: Bencher) {
    b_str(b, no_case(65, 4));
}
#[divan::bench(name = "ends_with/len_65/no/ours_string")]
fn e65n_string(b: Bencher) {
    b_string(b, no_case(65, 4));
}
#[divan::bench(name = "ends_with/len_65/no/string")]
fn e65n_native(b: Bencher) {
    b_native(b, no_case(65, 4));
}

#[divan::bench(name = "ends_with/len_65536/yes/ours_str")]
fn e6y_str(b: Bencher) {
    b_str(b, make_case(65_536, 128));
}
#[divan::bench(name = "ends_with/len_65536/yes/ours_string")]
fn e6y_string(b: Bencher) {
    b_string(b, make_case(65_536, 128));
}
#[divan::bench(name = "ends_with/len_65536/yes/string")]
fn e6y_native(b: Bencher) {
    b_native(b, make_case(65_536, 128));
}

#[divan::bench(name = "ends_with/len_65536/no/ours_str")]
fn e6n_str(b: Bencher) {
    b_str(b, no_case(65_536, 128));
}
#[divan::bench(name = "ends_with/len_65536/no/ours_string")]
fn e6n_string(b: Bencher) {
    b_string(b, no_case(65_536, 128));
}
#[divan::bench(name = "ends_with/len_65536/no/string")]
fn e6n_native(b: Bencher) {
    b_native(b, no_case(65_536, 128));
}

fn b_str(b: Bencher, c: Case) {
    b.bench(|| black_box(&c.haystack_bits).ends_with_str(black_box(c.suffix_bits.as_bit_str())));
}
fn b_string(b: Bencher, c: Case) {
    b.bench(|| black_box(&c.haystack_bits).ends_with_string(black_box(&c.suffix_bits)));
}
fn b_native(b: Bencher, c: Case) {
    b.bench(|| black_box(&c.haystack_string).ends_with(black_box(&c.suffix_string)));
}
