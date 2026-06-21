use bit_string::BitString;
use divan::{Bencher, black_box};
use int_interval::UsizeCO;

fn main() {
    divan::main();
}

struct Case {
    haystack_bits: BitString,
    pattern_bits: BitString,
    haystack_string: String,
    pattern_string: String,
    index: usize,
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

fn iv(start: usize, len: usize) -> UsizeCO {
    UsizeCO::checked_from_start_len(start, len).unwrap()
}

fn make_case(len: usize, pat_len: usize, index: usize) -> Case {
    let h = make_bits(len);
    let p = h.slice(iv(index, pat_len));
    Case {
        haystack_string: h.to_string(),
        pattern_string: p.to_string(),
        haystack_bits: h,
        pattern_bits: p,
        index,
    }
}

fn no_case(len: usize, pat_len: usize, index: usize) -> Case {
    let h = make_bits(len);
    let mut p = h.slice(iv(index, pat_len));
    p.set(0, !p.get(0).unwrap());
    Case {
        haystack_string: h.to_string(),
        pattern_string: p.to_string(),
        haystack_bits: h,
        pattern_bits: p,
        index,
    }
}

// ---------------------------------------------------------------------------
// 65-bit haystack (small — scalar path)
// ---------------------------------------------------------------------------

#[divan::bench(name = "matches_at/len_65/yes/aligned/bit_string")]
fn m65ya(b: Bencher) {
    b_bit(b, make_case(65, 4, 64));
}
#[divan::bench(name = "matches_at/len_65/yes/aligned/string")]
fn m65yas(b: Bencher) {
    b_str(b, make_case(65, 4, 64));
}
#[divan::bench(name = "matches_at/len_65/yes/unaligned/bit_string")]
fn m65yu(b: Bencher) {
    b_bit(b, make_case(65, 4, 3));
}
#[divan::bench(name = "matches_at/len_65/yes/unaligned/string")]
fn m65yus(b: Bencher) {
    b_str(b, make_case(65, 4, 3));
}
#[divan::bench(name = "matches_at/len_65/no/aligned/bit_string")]
fn m65na(b: Bencher) {
    b_bit(b, no_case(65, 4, 64));
}
#[divan::bench(name = "matches_at/len_65/no/aligned/string")]
fn m65nas(b: Bencher) {
    b_str(b, no_case(65, 4, 64));
}
#[divan::bench(name = "matches_at/len_65/no/unaligned/bit_string")]
fn m65nu(b: Bencher) {
    b_bit(b, no_case(65, 4, 3));
}
#[divan::bench(name = "matches_at/len_65/no/unaligned/string")]
fn m65nus(b: Bencher) {
    b_str(b, no_case(65, 4, 3));
}

// ---------------------------------------------------------------------------
// 65536-bit haystack (large — SIMD path)
// ---------------------------------------------------------------------------

#[divan::bench(name = "matches_at/len_65536/yes/aligned/bit_string")]
fn m6ya(b: Bencher) {
    b_bit(b, make_case(65_536, 128, 64));
}
#[divan::bench(name = "matches_at/len_65536/yes/aligned/string")]
fn m6yas(b: Bencher) {
    b_str(b, make_case(65_536, 128, 64));
}
#[divan::bench(name = "matches_at/len_65536/yes/unaligned/bit_string")]
fn m6yu(b: Bencher) {
    b_bit(b, make_case(65_536, 128, 3));
}
#[divan::bench(name = "matches_at/len_65536/yes/unaligned/string")]
fn m6yus(b: Bencher) {
    b_str(b, make_case(65_536, 128, 3));
}
#[divan::bench(name = "matches_at/len_65536/no/aligned/bit_string")]
fn m6na(b: Bencher) {
    b_bit(b, no_case(65_536, 128, 64));
}
#[divan::bench(name = "matches_at/len_65536/no/aligned/string")]
fn m6nas(b: Bencher) {
    b_str(b, no_case(65_536, 128, 64));
}
#[divan::bench(name = "matches_at/len_65536/no/unaligned/bit_string")]
fn m6nu(b: Bencher) {
    b_bit(b, no_case(65_536, 128, 3));
}
#[divan::bench(name = "matches_at/len_65536/no/unaligned/string")]
fn m6nus(b: Bencher) {
    b_str(b, no_case(65_536, 128, 3));
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn b_bit(b: Bencher, c: Case) {
    b.bench(|| black_box(&c.haystack_bits).matches_at(c.index, black_box(&c.pattern_bits)));
}

fn b_str(b: Bencher, c: Case) {
    b.bench(|| {
        let h = black_box(&c.haystack_string);
        let p = black_box(&c.pattern_string);
        h.as_bytes()[c.index..].starts_with(p.as_bytes())
    });
}
