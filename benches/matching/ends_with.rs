struct EndsWithCase {
    h: BitString,
    s: BitString,
    hs: String,
    ss: String,
}

fn ends_with_mk(len: usize) -> BitString {
    let mut b = BitString::zeros(len);
    for i in 0..len {
        if (i as u64 * 17 + 3) % 7 == 0 {
            b.set(i, true);
        }
    }
    b
}

fn ends_with_suffix(h: &BitString, sfx: usize) -> BitString {
    let start = h.bit_len() - sfx;
    h.slice(UsizeCO::checked_from_start_len(start, sfx).unwrap())
}

fn ends_with_hit(len: usize, sfx: usize) -> EndsWithCase {
    let h = ends_with_mk(len);
    let s = ends_with_suffix(&h, sfx);
    assert!(h.ends_with_string(&s));
    EndsWithCase {
        hs: h.to_string(),
        ss: s.to_string(),
        h,
        s,
    }
}
fn ends_with_no(len: usize, sfx: usize) -> EndsWithCase {
    let mut h = ends_with_mk(len);
    let s = ends_with_suffix(&h, sfx);
    h.set(h.bit_len() - 1, !h.get(h.bit_len() - 1).unwrap());
    assert!(!h.ends_with_string(&s));
    EndsWithCase {
        hs: h.to_string(),
        ss: s.to_string(),
        h,
        s,
    }
}

#[divan::bench(name = "ends_with/len_65/hit/ours_str_str")]
fn ends_65_hit_str_str(b: Bencher) {
    ends_with_bench_str_str(b, &ends_with_hit(65, 4));
}
#[divan::bench(name = "ends_with/len_65/hit/ours_str_string")]
fn ends_65_hit_str_string(b: Bencher) {
    ends_with_bench_str_string(b, &ends_with_hit(65, 4));
}
#[divan::bench(name = "ends_with/len_65/hit/ours_string_str")]
fn ends_65_hit_string_str(b: Bencher) {
    ends_with_bench_string_str(b, &ends_with_hit(65, 4));
}
#[divan::bench(name = "ends_with/len_65/hit/ours_string_string")]
fn ends_65_hit_string_string(b: Bencher) {
    ends_with_bench_string_string(b, &ends_with_hit(65, 4));
}
#[divan::bench(name = "ends_with/len_65/hit/string")]
fn ends_65_hit_native(b: Bencher) {
    ends_with_bench_native(b, &ends_with_hit(65, 4));
}

#[divan::bench(name = "ends_with/len_65/miss/ours_str_str")]
fn ends_65_miss_str_str(b: Bencher) {
    ends_with_bench_str_str(b, &ends_with_no(65, 4));
}
#[divan::bench(name = "ends_with/len_65/miss/ours_str_string")]
fn ends_65_miss_str_string(b: Bencher) {
    ends_with_bench_str_string(b, &ends_with_no(65, 4));
}
#[divan::bench(name = "ends_with/len_65/miss/ours_string_str")]
fn ends_65_miss_string_str(b: Bencher) {
    ends_with_bench_string_str(b, &ends_with_no(65, 4));
}
#[divan::bench(name = "ends_with/len_65/miss/ours_string_string")]
fn ends_65_miss_string_string(b: Bencher) {
    ends_with_bench_string_string(b, &ends_with_no(65, 4));
}
#[divan::bench(name = "ends_with/len_65/miss/string")]
fn ends_65_miss_native(b: Bencher) {
    ends_with_bench_native(b, &ends_with_no(65, 4));
}

#[divan::bench(name = "ends_with/len_65536/hit/ours_str_str")]
fn ends_65536_hit_str_str(b: Bencher) {
    ends_with_bench_str_str(b, &ends_with_hit(65536, 128));
}
#[divan::bench(name = "ends_with/len_65536/hit/ours_str_string")]
fn ends_65536_hit_str_string(b: Bencher) {
    ends_with_bench_str_string(b, &ends_with_hit(65536, 128));
}
#[divan::bench(name = "ends_with/len_65536/hit/ours_string_str")]
fn ends_65536_hit_string_str(b: Bencher) {
    ends_with_bench_string_str(b, &ends_with_hit(65536, 128));
}
#[divan::bench(name = "ends_with/len_65536/hit/ours_string_string")]
fn ends_65536_hit_string_string(b: Bencher) {
    ends_with_bench_string_string(b, &ends_with_hit(65536, 128));
}
#[divan::bench(name = "ends_with/len_65536/hit/string")]
fn ends_65536_hit_native(b: Bencher) {
    ends_with_bench_native(b, &ends_with_hit(65536, 128));
}

#[divan::bench(name = "ends_with/len_65536/miss/ours_str_str")]
fn ends_65536_miss_str_str(b: Bencher) {
    ends_with_bench_str_str(b, &ends_with_no(65536, 128));
}
#[divan::bench(name = "ends_with/len_65536/miss/ours_str_string")]
fn ends_65536_miss_str_string(b: Bencher) {
    ends_with_bench_str_string(b, &ends_with_no(65536, 128));
}
#[divan::bench(name = "ends_with/len_65536/miss/ours_string_str")]
fn ends_65536_miss_string_str(b: Bencher) {
    ends_with_bench_string_str(b, &ends_with_no(65536, 128));
}
#[divan::bench(name = "ends_with/len_65536/miss/ours_string_string")]
fn ends_65536_miss_string_string(b: Bencher) {
    ends_with_bench_string_string(b, &ends_with_no(65536, 128));
}
#[divan::bench(name = "ends_with/len_65536/miss/string")]
fn ends_65536_miss_native(b: Bencher) {
    ends_with_bench_native(b, &ends_with_no(65536, 128));
}

fn ends_with_bench_str_str(b: Bencher, c: &EndsWithCase) {
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).ends_with_str(black_box(c.s.as_bit_str())));
}
fn ends_with_bench_str_string(b: Bencher, c: &EndsWithCase) {
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).ends_with_string(black_box(&c.s)));
}
fn ends_with_bench_string_str(b: Bencher, c: &EndsWithCase) {
    b.bench(|| black_box(&c.h).ends_with_str(black_box(c.s.as_bit_str())));
}
fn ends_with_bench_string_string(b: Bencher, c: &EndsWithCase) {
    b.bench(|| black_box(&c.h).ends_with_string(black_box(&c.s)));
}
fn ends_with_bench_native(b: Bencher, c: &EndsWithCase) {
    b.bench(|| black_box(&c.hs).ends_with(black_box(&c.ss)));
}
