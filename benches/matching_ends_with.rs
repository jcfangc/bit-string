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
fn e65h_va(b: Bencher) {
    bench_va(b, &hit(65, 4));
}
#[divan::bench(name = "ends_with/len_65/hit/ours_str_string")]
fn e65h_vb(b: Bencher) {
    bench_vb(b, &hit(65, 4));
}
#[divan::bench(name = "ends_with/len_65/hit/ours_string_str")]
fn e65h_ba(b: Bencher) {
    bench_ba(b, &hit(65, 4));
}
#[divan::bench(name = "ends_with/len_65/hit/ours_string_string")]
fn e65h_bb(b: Bencher) {
    bench_bb(b, &hit(65, 4));
}
#[divan::bench(name = "ends_with/len_65/hit/string")]
fn e65h_vn(b: Bencher) {
    bench_vn(b, &hit(65, 4));
}

#[divan::bench(name = "ends_with/len_65/miss/ours_str_str")]
fn e65m_va(b: Bencher) {
    bench_va(b, &no(65, 4));
}
#[divan::bench(name = "ends_with/len_65/miss/ours_str_string")]
fn e65m_vb(b: Bencher) {
    bench_vb(b, &no(65, 4));
}
#[divan::bench(name = "ends_with/len_65/miss/ours_string_str")]
fn e65m_ba(b: Bencher) {
    bench_ba(b, &no(65, 4));
}
#[divan::bench(name = "ends_with/len_65/miss/ours_string_string")]
fn e65m_bb(b: Bencher) {
    bench_bb(b, &no(65, 4));
}
#[divan::bench(name = "ends_with/len_65/miss/string")]
fn e65m_vn(b: Bencher) {
    bench_vn(b, &no(65, 4));
}

#[divan::bench(name = "ends_with/len_65536/hit/ours_str_str")]
fn e6h_va(b: Bencher) {
    bench_va(b, &hit(65536, 128));
}
#[divan::bench(name = "ends_with/len_65536/hit/ours_str_string")]
fn e6h_vb(b: Bencher) {
    bench_vb(b, &hit(65536, 128));
}
#[divan::bench(name = "ends_with/len_65536/hit/ours_string_str")]
fn e6h_ba(b: Bencher) {
    bench_ba(b, &hit(65536, 128));
}
#[divan::bench(name = "ends_with/len_65536/hit/ours_string_string")]
fn e6h_bb(b: Bencher) {
    bench_bb(b, &hit(65536, 128));
}
#[divan::bench(name = "ends_with/len_65536/hit/string")]
fn e6h_vn(b: Bencher) {
    bench_vn(b, &hit(65536, 128));
}

#[divan::bench(name = "ends_with/len_65536/miss/ours_str_str")]
fn e6m_va(b: Bencher) {
    bench_va(b, &no(65536, 128));
}
#[divan::bench(name = "ends_with/len_65536/miss/ours_str_string")]
fn e6m_vb(b: Bencher) {
    bench_vb(b, &no(65536, 128));
}
#[divan::bench(name = "ends_with/len_65536/miss/ours_string_str")]
fn e6m_ba(b: Bencher) {
    bench_ba(b, &no(65536, 128));
}
#[divan::bench(name = "ends_with/len_65536/miss/ours_string_string")]
fn e6m_bb(b: Bencher) {
    bench_bb(b, &no(65536, 128));
}
#[divan::bench(name = "ends_with/len_65536/miss/string")]
fn e6m_vn(b: Bencher) {
    bench_vn(b, &no(65536, 128));
}

fn bench_va(b: Bencher, c: &Case) {
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).ends_with_str(black_box(c.s.as_bit_str())));
}
fn bench_vb(b: Bencher, c: &Case) {
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).ends_with_string(black_box(&c.s)));
}
fn bench_ba(b: Bencher, c: &Case) {
    b.bench(|| black_box(&c.h).ends_with_str(black_box(c.s.as_bit_str())));
}
fn bench_bb(b: Bencher, c: &Case) {
    b.bench(|| black_box(&c.h).ends_with_string(black_box(&c.s)));
}
fn bench_vn(b: Bencher, c: &Case) {
    b.bench(|| black_box(&c.hs).ends_with(black_box(&c.ss)));
}
