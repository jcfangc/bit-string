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
fn starts_65_hit_str_str(b: Bencher) {
    bench_str_str(b, &hit(65, 4));
}
#[divan::bench(name = "starts_with/len_65/hit/ours_str_string")]
fn starts_65_hit_str_string(b: Bencher) {
    bench_str_string(b, &hit(65, 4));
}
#[divan::bench(name = "starts_with/len_65/hit/ours_string_str")]
fn starts_65_hit_string_str(b: Bencher) {
    bench_string_str(b, &hit(65, 4));
}
#[divan::bench(name = "starts_with/len_65/hit/ours_string_string")]
fn starts_65_hit_string_string(b: Bencher) {
    bench_string_string(b, &hit(65, 4));
}
#[divan::bench(name = "starts_with/len_65/hit/string")]
fn starts_65_hit_native(b: Bencher) {
    bench_native(b, &hit(65, 4));
}

#[divan::bench(name = "starts_with/len_65/miss/ours_str_str")]
fn starts_65_miss_str_str(b: Bencher) {
    bench_str_str(b, &no(65, 4));
}
#[divan::bench(name = "starts_with/len_65/miss/ours_str_string")]
fn starts_65_miss_str_string(b: Bencher) {
    bench_str_string(b, &no(65, 4));
}
#[divan::bench(name = "starts_with/len_65/miss/ours_string_str")]
fn starts_65_miss_string_str(b: Bencher) {
    bench_string_str(b, &no(65, 4));
}
#[divan::bench(name = "starts_with/len_65/miss/ours_string_string")]
fn starts_65_miss_string_string(b: Bencher) {
    bench_string_string(b, &no(65, 4));
}
#[divan::bench(name = "starts_with/len_65/miss/string")]
fn starts_65_miss_native(b: Bencher) {
    bench_native(b, &no(65, 4));
}

#[divan::bench(name = "starts_with/len_65536/hit/ours_str_str")]
fn starts_65536_hit_str_str(b: Bencher) {
    bench_str_str(b, &hit(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/hit/ours_str_string")]
fn starts_65536_hit_str_string(b: Bencher) {
    bench_str_string(b, &hit(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/hit/ours_string_str")]
fn starts_65536_hit_string_str(b: Bencher) {
    bench_string_str(b, &hit(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/hit/ours_string_string")]
fn starts_65536_hit_string_string(b: Bencher) {
    bench_string_string(b, &hit(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/hit/string")]
fn starts_65536_hit_native(b: Bencher) {
    bench_native(b, &hit(65536, 128));
}

#[divan::bench(name = "starts_with/len_65536/miss/ours_str_str")]
fn starts_65536_miss_str_str(b: Bencher) {
    bench_str_str(b, &no(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/miss/ours_str_string")]
fn starts_65536_miss_str_string(b: Bencher) {
    bench_str_string(b, &no(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/miss/ours_string_str")]
fn starts_65536_miss_string_str(b: Bencher) {
    bench_string_str(b, &no(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/miss/ours_string_string")]
fn starts_65536_miss_string_string(b: Bencher) {
    bench_string_string(b, &no(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/miss/string")]
fn starts_65536_miss_native(b: Bencher) {
    bench_native(b, &no(65536, 128));
}

fn bench_str_str(b: Bencher, c: &Case) {
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).starts_with_str(black_box(c.p.as_bit_str())));
}
fn bench_str_string(b: Bencher, c: &Case) {
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).starts_with_string(black_box(&c.p)));
}
fn bench_string_str(b: Bencher, c: &Case) {
    b.bench(|| black_box(&c.h).starts_with_str(black_box(c.p.as_bit_str())));
}
fn bench_string_string(b: Bencher, c: &Case) {
    b.bench(|| black_box(&c.h).starts_with_string(black_box(&c.p)));
}
fn bench_native(b: Bencher, c: &Case) {
    b.bench(|| black_box(&c.hs).starts_with(black_box(&c.ps)));
}
