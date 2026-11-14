struct StartsWithCase {
    h: BitString,
    p: BitString,
    hs: String,
    ps: String,
}

fn starts_with_mk(len: usize) -> BitString {
    let mut b = BitString::zeros(len);
    for i in 0..len {
        if (i as u64 * 17 + 3) % 7 == 0 {
            b.set(i, true);
        }
    }
    b
}
fn starts_with_hit(len: usize, pfx: usize) -> StartsWithCase {
    let h = starts_with_mk(len);
    let p = starts_with_mk(pfx);
    StartsWithCase {
        hs: h.to_string(),
        ps: p.to_string(),
        h,
        p,
    }
}
fn starts_with_no(len: usize, pfx: usize) -> StartsWithCase {
    let mut h = starts_with_mk(len);
    let p = starts_with_mk(pfx);
    h.set(0, !h.get(0).unwrap());
    StartsWithCase {
        hs: h.to_string(),
        ps: p.to_string(),
        h,
        p,
    }
}

#[divan::bench(name = "starts_with/len_65/hit/ours_str_str")]
fn starts_65_hit_str_str(b: Bencher) {
    starts_with_bench_str_str(b, &starts_with_hit(65, 4));
}
#[divan::bench(name = "starts_with/len_65/hit/ours_str_string")]
fn starts_65_hit_str_string(b: Bencher) {
    starts_with_bench_str_string(b, &starts_with_hit(65, 4));
}
#[divan::bench(name = "starts_with/len_65/hit/ours_string_str")]
fn starts_65_hit_string_str(b: Bencher) {
    starts_with_bench_string_str(b, &starts_with_hit(65, 4));
}
#[divan::bench(name = "starts_with/len_65/hit/ours_string_string")]
fn starts_65_hit_string_string(b: Bencher) {
    starts_with_bench_string_string(b, &starts_with_hit(65, 4));
}
#[divan::bench(name = "starts_with/len_65/hit/string")]
fn starts_65_hit_native(b: Bencher) {
    starts_with_bench_native(b, &starts_with_hit(65, 4));
}

#[divan::bench(name = "starts_with/len_65/miss/ours_str_str")]
fn starts_65_miss_str_str(b: Bencher) {
    starts_with_bench_str_str(b, &starts_with_no(65, 4));
}
#[divan::bench(name = "starts_with/len_65/miss/ours_str_string")]
fn starts_65_miss_str_string(b: Bencher) {
    starts_with_bench_str_string(b, &starts_with_no(65, 4));
}
#[divan::bench(name = "starts_with/len_65/miss/ours_string_str")]
fn starts_65_miss_string_str(b: Bencher) {
    starts_with_bench_string_str(b, &starts_with_no(65, 4));
}
#[divan::bench(name = "starts_with/len_65/miss/ours_string_string")]
fn starts_65_miss_string_string(b: Bencher) {
    starts_with_bench_string_string(b, &starts_with_no(65, 4));
}
#[divan::bench(name = "starts_with/len_65/miss/string")]
fn starts_65_miss_native(b: Bencher) {
    starts_with_bench_native(b, &starts_with_no(65, 4));
}

#[divan::bench(name = "starts_with/len_65536/hit/ours_str_str")]
fn starts_65536_hit_str_str(b: Bencher) {
    starts_with_bench_str_str(b, &starts_with_hit(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/hit/ours_str_string")]
fn starts_65536_hit_str_string(b: Bencher) {
    starts_with_bench_str_string(b, &starts_with_hit(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/hit/ours_string_str")]
fn starts_65536_hit_string_str(b: Bencher) {
    starts_with_bench_string_str(b, &starts_with_hit(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/hit/ours_string_string")]
fn starts_65536_hit_string_string(b: Bencher) {
    starts_with_bench_string_string(b, &starts_with_hit(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/hit/string")]
fn starts_65536_hit_native(b: Bencher) {
    starts_with_bench_native(b, &starts_with_hit(65536, 128));
}

#[divan::bench(name = "starts_with/len_65536/miss/ours_str_str")]
fn starts_65536_miss_str_str(b: Bencher) {
    starts_with_bench_str_str(b, &starts_with_no(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/miss/ours_str_string")]
fn starts_65536_miss_str_string(b: Bencher) {
    starts_with_bench_str_string(b, &starts_with_no(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/miss/ours_string_str")]
fn starts_65536_miss_string_str(b: Bencher) {
    starts_with_bench_string_str(b, &starts_with_no(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/miss/ours_string_string")]
fn starts_65536_miss_string_string(b: Bencher) {
    starts_with_bench_string_string(b, &starts_with_no(65536, 128));
}
#[divan::bench(name = "starts_with/len_65536/miss/string")]
fn starts_65536_miss_native(b: Bencher) {
    starts_with_bench_native(b, &starts_with_no(65536, 128));
}

fn starts_with_bench_str_str(b: Bencher, c: &StartsWithCase) {
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).starts_with_str(black_box(c.p.as_bit_str())));
}
fn starts_with_bench_str_string(b: Bencher, c: &StartsWithCase) {
    let v = c.h.as_bit_str();
    b.bench(|| black_box(&v).starts_with_string(black_box(&c.p)));
}
fn starts_with_bench_string_str(b: Bencher, c: &StartsWithCase) {
    b.bench(|| black_box(&c.h).starts_with_str(black_box(c.p.as_bit_str())));
}
fn starts_with_bench_string_string(b: Bencher, c: &StartsWithCase) {
    b.bench(|| black_box(&c.h).starts_with_string(black_box(&c.p)));
}
fn starts_with_bench_native(b: Bencher, c: &StartsWithCase) {
    b.bench(|| black_box(&c.hs).starts_with(black_box(&c.ps)));
}
