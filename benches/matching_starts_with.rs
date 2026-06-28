use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

struct Case {
    h: BitString,
    p: BitString,
    hs: String,
    ps: String,
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
fn hit(len: usize, pfx: usize) -> Case {
    let h = mk(len);
    let p = mk(pfx);
    Case {
        hs: h.to_string(),
        ps: p.to_string(),
        h,
        p,
    }
}
fn no(len: usize, pfx: usize) -> Case {
    let mut h = mk(len);
    let p = mk(pfx);
    h.set(0, !h.get(0).unwrap());
    Case {
        hs: h.to_string(),
        ps: p.to_string(),
        h,
        p,
    }
}

#[divan::bench(name = "starts_with/len_65/hit/ours_str_str")]
fn starts_65h_a(b: Bencher) {
    let c = hit(65, 4);
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).starts_with_str(black_box(c.p.as_bit_str())));
}
#[divan::bench(name = "starts_with/len_65/hit/ours_str_string")]
fn starts_65h_b(b: Bencher) {
    let c = hit(65, 4);
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).starts_with_string(black_box(&c.p)));
}
#[divan::bench(name = "starts_with/len_65/hit/ours_string_str")]
fn starts_65h_c(b: Bencher) {
    let c = hit(65, 4);
    b.bench(|| black_box(&c.h).starts_with_str(black_box(c.p.as_bit_str())));
}
#[divan::bench(name = "starts_with/len_65/hit/ours_string_string")]
fn starts_65h_d(b: Bencher) {
    let c = hit(65, 4);
    b.bench(|| black_box(&c.h).starts_with_string(black_box(&c.p)));
}
#[divan::bench(name = "starts_with/len_65/hit/string")]
fn starts_65h_e(b: Bencher) {
    let c = hit(65, 4);
    b.bench(|| black_box(&c.hs).starts_with(black_box(&c.ps)));
}

#[divan::bench(name = "starts_with/len_65/miss/ours_str_str")]
fn starts_65m_a(b: Bencher) {
    let c = no(65, 4);
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).starts_with_str(black_box(c.p.as_bit_str())));
}
#[divan::bench(name = "starts_with/len_65/miss/ours_str_string")]
fn starts_65m_b(b: Bencher) {
    let c = no(65, 4);
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).starts_with_string(black_box(&c.p)));
}
#[divan::bench(name = "starts_with/len_65/miss/ours_string_str")]
fn starts_65m_c(b: Bencher) {
    let c = no(65, 4);
    b.bench(|| black_box(&c.h).starts_with_str(black_box(c.p.as_bit_str())));
}
#[divan::bench(name = "starts_with/len_65/miss/ours_string_string")]
fn starts_65m_d(b: Bencher) {
    let c = no(65, 4);
    b.bench(|| black_box(&c.h).starts_with_string(black_box(&c.p)));
}
#[divan::bench(name = "starts_with/len_65/miss/string")]
fn starts_65m_e(b: Bencher) {
    let c = no(65, 4);
    b.bench(|| black_box(&c.hs).starts_with(black_box(&c.ps)));
}

#[divan::bench(name = "starts_with/len_65536/hit/ours_str_str")]
fn starts_6h_a(b: Bencher) {
    let c = hit(65536, 128);
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).starts_with_str(black_box(c.p.as_bit_str())));
}
#[divan::bench(name = "starts_with/len_65536/hit/ours_str_string")]
fn starts_6h_b(b: Bencher) {
    let c = hit(65536, 128);
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).starts_with_string(black_box(&c.p)));
}
#[divan::bench(name = "starts_with/len_65536/hit/ours_string_str")]
fn starts_6h_c(b: Bencher) {
    let c = hit(65536, 128);
    b.bench(|| black_box(&c.h).starts_with_str(black_box(c.p.as_bit_str())));
}
#[divan::bench(name = "starts_with/len_65536/hit/ours_string_string")]
fn starts_6h_d(b: Bencher) {
    let c = hit(65536, 128);
    b.bench(|| black_box(&c.h).starts_with_string(black_box(&c.p)));
}
#[divan::bench(name = "starts_with/len_65536/hit/string")]
fn starts_6h_e(b: Bencher) {
    let c = hit(65536, 128);
    b.bench(|| black_box(&c.hs).starts_with(black_box(&c.ps)));
}

#[divan::bench(name = "starts_with/len_65536/miss/ours_str_str")]
fn starts_6m_a(b: Bencher) {
    let c = no(65536, 128);
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).starts_with_str(black_box(c.p.as_bit_str())));
}
#[divan::bench(name = "starts_with/len_65536/miss/ours_str_string")]
fn starts_6m_b(b: Bencher) {
    let c = no(65536, 128);
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).starts_with_string(black_box(&c.p)));
}
#[divan::bench(name = "starts_with/len_65536/miss/ours_string_str")]
fn starts_6m_c(b: Bencher) {
    let c = no(65536, 128);
    b.bench(|| black_box(&c.h).starts_with_str(black_box(c.p.as_bit_str())));
}
#[divan::bench(name = "starts_with/len_65536/miss/ours_string_string")]
fn starts_6m_d(b: Bencher) {
    let c = no(65536, 128);
    b.bench(|| black_box(&c.h).starts_with_string(black_box(&c.p)));
}
#[divan::bench(name = "starts_with/len_65536/miss/string")]
fn starts_6m_e(b: Bencher) {
    let c = no(65536, 128);
    b.bench(|| black_box(&c.hs).starts_with(black_box(&c.ps)));
}
