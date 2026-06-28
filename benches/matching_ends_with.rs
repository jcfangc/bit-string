use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

struct Case {
    h: BitString,
    s: BitString,
    hs: String,
    ss: String,
}

fn mk(len: usize) -> BitString {
    let mut b = BitString::zeros(len);
    for i in 0..len {
        if (i as u64 * 17 + 3) % 7 == 0 {
            b.set(i, true);
        }
    }
    b
}
fn hit(len: usize, sfx: usize) -> Case {
    let h = mk(len);
    let s = mk(sfx);
    Case {
        hs: h.to_string(),
        ss: s.to_string(),
        h,
        s,
    }
}
fn no(len: usize, sfx: usize) -> Case {
    let mut h = mk(len);
    let s = mk(sfx);
    h.set(h.bit_len() - 1, !h.get(h.bit_len() - 1).unwrap());
    Case {
        hs: h.to_string(),
        ss: s.to_string(),
        h,
        s,
    }
}

#[divan::bench(name = "ends_with/len_65/hit/ours_str_str")]
fn ends_65h_a(b: Bencher) {
    let c = hit(65, 4);
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).ends_with_str(black_box(c.s.as_bit_str())));
}
#[divan::bench(name = "ends_with/len_65/hit/ours_str_string")]
fn ends_65h_b(b: Bencher) {
    let c = hit(65, 4);
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).ends_with_string(black_box(&c.s)));
}
#[divan::bench(name = "ends_with/len_65/hit/ours_string_str")]
fn ends_65h_c(b: Bencher) {
    let c = hit(65, 4);
    b.bench(|| black_box(&c.h).ends_with_str(black_box(c.s.as_bit_str())));
}
#[divan::bench(name = "ends_with/len_65/hit/ours_string_string")]
fn ends_65h_d(b: Bencher) {
    let c = hit(65, 4);
    b.bench(|| black_box(&c.h).ends_with_string(black_box(&c.s)));
}
#[divan::bench(name = "ends_with/len_65/hit/string")]
fn ends_65h_e(b: Bencher) {
    let c = hit(65, 4);
    b.bench(|| black_box(&c.hs).ends_with(black_box(&c.ss)));
}

#[divan::bench(name = "ends_with/len_65/miss/ours_str_str")]
fn ends_65m_a(b: Bencher) {
    let c = no(65, 4);
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).ends_with_str(black_box(c.s.as_bit_str())));
}
#[divan::bench(name = "ends_with/len_65/miss/ours_str_string")]
fn ends_65m_b(b: Bencher) {
    let c = no(65, 4);
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).ends_with_string(black_box(&c.s)));
}
#[divan::bench(name = "ends_with/len_65/miss/ours_string_str")]
fn ends_65m_c(b: Bencher) {
    let c = no(65, 4);
    b.bench(|| black_box(&c.h).ends_with_str(black_box(c.s.as_bit_str())));
}
#[divan::bench(name = "ends_with/len_65/miss/ours_string_string")]
fn ends_65m_d(b: Bencher) {
    let c = no(65, 4);
    b.bench(|| black_box(&c.h).ends_with_string(black_box(&c.s)));
}
#[divan::bench(name = "ends_with/len_65/miss/string")]
fn ends_65m_e(b: Bencher) {
    let c = no(65, 4);
    b.bench(|| black_box(&c.hs).ends_with(black_box(&c.ss)));
}

#[divan::bench(name = "ends_with/len_65536/hit/ours_str_str")]
fn ends_6h_a(b: Bencher) {
    let c = hit(65536, 128);
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).ends_with_str(black_box(c.s.as_bit_str())));
}
#[divan::bench(name = "ends_with/len_65536/hit/ours_str_string")]
fn ends_6h_b(b: Bencher) {
    let c = hit(65536, 128);
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).ends_with_string(black_box(&c.s)));
}
#[divan::bench(name = "ends_with/len_65536/hit/ours_string_str")]
fn ends_6h_c(b: Bencher) {
    let c = hit(65536, 128);
    b.bench(|| black_box(&c.h).ends_with_str(black_box(c.s.as_bit_str())));
}
#[divan::bench(name = "ends_with/len_65536/hit/ours_string_string")]
fn ends_6h_d(b: Bencher) {
    let c = hit(65536, 128);
    b.bench(|| black_box(&c.h).ends_with_string(black_box(&c.s)));
}
#[divan::bench(name = "ends_with/len_65536/hit/string")]
fn ends_6h_e(b: Bencher) {
    let c = hit(65536, 128);
    b.bench(|| black_box(&c.hs).ends_with(black_box(&c.ss)));
}

#[divan::bench(name = "ends_with/len_65536/miss/ours_str_str")]
fn ends_6m_a(b: Bencher) {
    let c = no(65536, 128);
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).ends_with_str(black_box(c.s.as_bit_str())));
}
#[divan::bench(name = "ends_with/len_65536/miss/ours_str_string")]
fn ends_6m_b(b: Bencher) {
    let c = no(65536, 128);
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).ends_with_string(black_box(&c.s)));
}
#[divan::bench(name = "ends_with/len_65536/miss/ours_string_str")]
fn ends_6m_c(b: Bencher) {
    let c = no(65536, 128);
    b.bench(|| black_box(&c.h).ends_with_str(black_box(c.s.as_bit_str())));
}
#[divan::bench(name = "ends_with/len_65536/miss/ours_string_string")]
fn ends_6m_d(b: Bencher) {
    let c = no(65536, 128);
    b.bench(|| black_box(&c.h).ends_with_string(black_box(&c.s)));
}
#[divan::bench(name = "ends_with/len_65536/miss/string")]
fn ends_6m_e(b: Bencher) {
    let c = no(65536, 128);
    b.bench(|| black_box(&c.hs).ends_with(black_box(&c.ss)));
}
