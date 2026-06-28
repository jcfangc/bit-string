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
fn s65h_va(b: Bencher) {
    bench_va(b, &hit(65, 4));
}
#[divan::bench(name = "starts_with/len_65/hit/ours_str_string")]
fn s65h_vb(b: Bencher) {
    bench_vb(b, &hit(65, 4));
}
#[divan::bench(name = "starts_with/len_65/hit/ours_string_str")]
fn s65h_ba(b: Bencher) {
    bench_ba(b, &hit(65, 4));
}
#[divan::bench(name = "starts_with/len_65/hit/ours_string_string")]
fn s65h_bb(b: Bencher) {
    bench_bb(b, &hit(65, 4));
}
#[divan::bench(name = "starts_with/len_65/hit/string")]
fn s65h_vn(b: Bencher) {
    bench_vn(b, &hit(65, 4));
}

#[divan::bench(name = "starts_with/len_65/miss/ours_str_str")]
fn s65m_va(b: Bencher) {
    bench_va(b, &no(65, 4));
}
#[divan::bench(name = "starts_with/len_65/miss/ours_str_string")]
fn s65m_vb(b: Bencher) {
    bench_vb(b, &no(65, 4));
}
#[divan::bench(name = "starts_with/len_65/miss/ours_string_str")]
fn s65m_ba(b: Bencher) {
    bench_ba(b, &no(65, 4));
}
#[divan::bench(name = "starts_with/len_65/miss/ours_string_string")]
fn s65m_bb(b: Bencher) {
    bench_bb(b, &no(65, 4));
}
#[divan::bench(name = "starts_with/len_65/miss/string")]
fn s65m_vn(b: Bencher) {
    bench_vn(b, &no(65, 4));
}

#[divan::bench(name = "starts_with/len_65536/hit/ours_str_str")]
fn s6h_va(b: Bencher) {
    bench_va(b, &hit(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/hit/ours_str_string")]
fn s6h_vb(b: Bencher) {
    bench_vb(b, &hit(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/hit/ours_string_str")]
fn s6h_ba(b: Bencher) {
    bench_ba(b, &hit(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/hit/ours_string_string")]
fn s6h_bb(b: Bencher) {
    bench_bb(b, &hit(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/hit/string")]
fn s6h_vn(b: Bencher) {
    bench_vn(b, &hit(65536, 128));
}

#[divan::bench(name = "starts_with/len_65536/miss/ours_str_str")]
fn s6m_va(b: Bencher) {
    bench_va(b, &no(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/miss/ours_str_string")]
fn s6m_vb(b: Bencher) {
    bench_vb(b, &no(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/miss/ours_string_str")]
fn s6m_ba(b: Bencher) {
    bench_ba(b, &no(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/miss/ours_string_string")]
fn s6m_bb(b: Bencher) {
    bench_bb(b, &no(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/miss/string")]
fn s6m_vn(b: Bencher) {
    bench_vn(b, &no(65536, 128));
}

fn bench_va(b: Bencher, c: &Case) {
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).starts_with_str(black_box(c.p.as_bit_str())));
}
fn bench_vb(b: Bencher, c: &Case) {
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).starts_with_string(black_box(&c.p)));
}
fn bench_ba(b: Bencher, c: &Case) {
    b.bench(|| black_box(&c.h).starts_with_str(black_box(c.p.as_bit_str())));
}
fn bench_bb(b: Bencher, c: &Case) {
    b.bench(|| black_box(&c.h).starts_with_string(black_box(&c.p)));
}
fn bench_vn(b: Bencher, c: &Case) {
    b.bench(|| black_box(&c.hs).starts_with(black_box(&c.ps)));
}
