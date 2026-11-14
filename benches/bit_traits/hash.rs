#[cfg_attr(not(feature = "characterization"), allow(dead_code))]
#[derive(Clone, Copy)]
enum HashPattern {
    Dense,
    Sparse,
    Alternating,
}

// ---------------------------------------------------------------------------
// BitString
// ---------------------------------------------------------------------------

#[divan::bench(name = "hash/len_64/dense/ours_string")]
fn hash_len_64_dense_bit_string(b: Bencher) {
    hash_bench_bit_string(b, 64, HashPattern::Dense);
}

#[divan::bench(name = "hash/len_64/dense/ours_str")]
fn hash_len_64_dense_bit_str(b: Bencher) {
    hash_bench_bit_str(b, 64, HashPattern::Dense);
}

#[divan::bench(name = "hash/len_64/dense/string")]
fn hash_len_64_dense_string(b: Bencher) {
    hash_bench_string(b, 64, HashPattern::Dense);
}

#[divan::bench(name = "hash/len_64/dense/str")]
fn hash_len_64_dense_str(b: Bencher) {
    hash_bench_str(b, 64, HashPattern::Dense);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_64/sparse/ours_string")]
fn hash_len_64_sparse_bit_string(b: Bencher) {
    hash_bench_bit_string(b, 64, HashPattern::Sparse);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_64/sparse/ours_str")]
fn hash_len_64_sparse_bit_str(b: Bencher) {
    hash_bench_bit_str(b, 64, HashPattern::Sparse);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_64/sparse/string")]
fn hash_len_64_sparse_string(b: Bencher) {
    hash_bench_string(b, 64, HashPattern::Sparse);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_64/sparse/str")]
fn hash_len_64_sparse_str(b: Bencher) {
    hash_bench_str(b, 64, HashPattern::Sparse);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_64/alternating/ours_string")]
fn hash_len_64_alternating_bit_string(b: Bencher) {
    hash_bench_bit_string(b, 64, HashPattern::Alternating);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_64/alternating/ours_str")]
fn hash_len_64_alternating_bit_str(b: Bencher) {
    hash_bench_bit_str(b, 64, HashPattern::Alternating);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_64/alternating/string")]
fn hash_len_64_alternating_string(b: Bencher) {
    hash_bench_string(b, 64, HashPattern::Alternating);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_64/alternating/str")]
fn hash_len_64_alternating_str(b: Bencher) {
    hash_bench_str(b, 64, HashPattern::Alternating);
}

// ---------------------------------------------------------------------------
// len = 65 (one full word plus a partial-word tail)
// ---------------------------------------------------------------------------

#[divan::bench(name = "hash/len_65/dense/ours_string")]
fn hash_len_65_dense_bit_string(b: Bencher) {
    hash_bench_bit_string(b, 65, HashPattern::Dense);
}

#[divan::bench(name = "hash/len_65/dense/ours_str")]
fn hash_len_65_dense_bit_str(b: Bencher) {
    hash_bench_bit_str(b, 65, HashPattern::Dense);
}

#[divan::bench(name = "hash/len_65/dense/ours_str_unaligned")]
fn hash_len_65_dense_bit_str_unaligned(b: Bencher) {
    hash_bench_bit_str_unaligned(b, 65, HashPattern::Dense);
}

// ---------------------------------------------------------------------------
// len = 4096
// ---------------------------------------------------------------------------

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_4096/dense/ours_string")]
fn hash_len_4096_dense_bit_string(b: Bencher) {
    hash_bench_bit_string(b, 4096, HashPattern::Dense);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_4096/dense/ours_str")]
fn hash_len_4096_dense_bit_str(b: Bencher) {
    hash_bench_bit_str(b, 4096, HashPattern::Dense);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_4096/dense/string")]
fn hash_len_4096_dense_string(b: Bencher) {
    hash_bench_string(b, 4096, HashPattern::Dense);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_4096/dense/str")]
fn hash_len_4096_dense_str(b: Bencher) {
    hash_bench_str(b, 4096, HashPattern::Dense);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_4096/sparse/ours_string")]
fn hash_len_4096_sparse_bit_string(b: Bencher) {
    hash_bench_bit_string(b, 4096, HashPattern::Sparse);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_4096/sparse/ours_str")]
fn hash_len_4096_sparse_bit_str(b: Bencher) {
    hash_bench_bit_str(b, 4096, HashPattern::Sparse);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_4096/sparse/string")]
fn hash_len_4096_sparse_string(b: Bencher) {
    hash_bench_string(b, 4096, HashPattern::Sparse);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_4096/sparse/str")]
fn hash_len_4096_sparse_str(b: Bencher) {
    hash_bench_str(b, 4096, HashPattern::Sparse);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_4096/alternating/ours_string")]
fn hash_len_4096_alternating_bit_string(b: Bencher) {
    hash_bench_bit_string(b, 4096, HashPattern::Alternating);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_4096/alternating/ours_str")]
fn hash_len_4096_alternating_bit_str(b: Bencher) {
    hash_bench_bit_str(b, 4096, HashPattern::Alternating);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_4096/alternating/string")]
fn hash_len_4096_alternating_string(b: Bencher) {
    hash_bench_string(b, 4096, HashPattern::Alternating);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_4096/alternating/str")]
fn hash_len_4096_alternating_str(b: Bencher) {
    hash_bench_str(b, 4096, HashPattern::Alternating);
}

// ---------------------------------------------------------------------------
// len = 65536
// ---------------------------------------------------------------------------

#[divan::bench(name = "hash/len_65536/dense/ours_string")]
fn hash_len_65536_dense_bit_string(b: Bencher) {
    hash_bench_bit_string(b, 65536, HashPattern::Dense);
}

#[divan::bench(name = "hash/len_65536/dense/ours_str")]
fn hash_len_65536_dense_bit_str(b: Bencher) {
    hash_bench_bit_str(b, 65536, HashPattern::Dense);
}

#[divan::bench(name = "hash/len_65536/dense/string")]
fn hash_len_65536_dense_string(b: Bencher) {
    hash_bench_string(b, 65536, HashPattern::Dense);
}

#[divan::bench(name = "hash/len_65536/dense/str")]
fn hash_len_65536_dense_str(b: Bencher) {
    hash_bench_str(b, 65536, HashPattern::Dense);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_65536/sparse/ours_string")]
fn hash_len_65536_sparse_bit_string(b: Bencher) {
    hash_bench_bit_string(b, 65536, HashPattern::Sparse);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_65536/sparse/ours_str")]
fn hash_len_65536_sparse_bit_str(b: Bencher) {
    hash_bench_bit_str(b, 65536, HashPattern::Sparse);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_65536/sparse/string")]
fn hash_len_65536_sparse_string(b: Bencher) {
    hash_bench_string(b, 65536, HashPattern::Sparse);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_65536/sparse/str")]
fn hash_len_65536_sparse_str(b: Bencher) {
    hash_bench_str(b, 65536, HashPattern::Sparse);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_65536/alternating/ours_string")]
fn hash_len_65536_alternating_bit_string(b: Bencher) {
    hash_bench_bit_string(b, 65536, HashPattern::Alternating);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_65536/alternating/ours_str")]
fn hash_len_65536_alternating_bit_str(b: Bencher) {
    hash_bench_bit_str(b, 65536, HashPattern::Alternating);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_65536/alternating/string")]
fn hash_len_65536_alternating_string(b: Bencher) {
    hash_bench_string(b, 65536, HashPattern::Alternating);
}

#[cfg(feature = "characterization")]
#[divan::bench(name = "hash/len_65536/alternating/str")]
fn hash_len_65536_alternating_str(b: Bencher) {
    hash_bench_str(b, 65536, HashPattern::Alternating);
}

// ---------------------------------------------------------------------------
// Bench helpers
// ---------------------------------------------------------------------------

fn hash_bench_bit_string(bencher: Bencher, len: usize, pattern: HashPattern) {
    let bits = hash_make_bit_string(len, pattern);
    bencher.bench(|| {
        let mut h = DefaultHasher::new();
        black_box(&bits).hash(&mut h);
        black_box(h.finish())
    });
}

fn hash_bench_bit_str(bencher: Bencher, len: usize, pattern: HashPattern) {
    let bits = hash_make_bit_string(len, pattern);
    let view = bits.as_bit_str();
    bencher.bench(|| {
        let mut h = DefaultHasher::new();
        black_box(&view).hash(&mut h);
        black_box(h.finish())
    });
}

fn hash_bench_bit_str_unaligned(bencher: Bencher, len: usize, pattern: HashPattern) {
    let bits = hash_make_bit_string(len + 3, pattern);
    let view = bits.as_bit_str().slice_from(3).slice_until(3 + len);
    bencher.bench(|| {
        let mut h = DefaultHasher::new();
        black_box(&view).hash(&mut h);
        black_box(h.finish())
    });
}

fn hash_bench_string(bencher: Bencher, len: usize, pattern: HashPattern) {
    let s = hash_make_string(len, pattern);
    bencher.bench(|| {
        let mut h = DefaultHasher::new();
        black_box(&s).hash(&mut h);
        black_box(h.finish())
    });
}

fn hash_bench_str(bencher: Bencher, len: usize, pattern: HashPattern) {
    let s = hash_make_string(len, pattern);
    bencher.bench(|| {
        let mut h = DefaultHasher::new();
        black_box(s.as_str()).hash(&mut h);
        black_box(h.finish())
    });
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

#[inline]
fn hash_make_bit_string(len: usize, pattern: HashPattern) -> BitString {
    (0..len).map(|index| hash_bit_at(index, pattern)).collect()
}

#[inline]
fn hash_make_string(len: usize, pattern: HashPattern) -> String {
    (0..len)
        .map(|index| if hash_bit_at(index, pattern) { '1' } else { '0' })
        .collect()
}

#[inline]
fn hash_bit_at(index: usize, pattern: HashPattern) -> bool {
    match pattern {
        HashPattern::Dense => hash_mix64(index as u64) & 1 != 0,
        HashPattern::Sparse => hash_mix64(index as u64) & 63 == 0,
        HashPattern::Alternating => index % 2 != 0,
    }
}

#[inline]
fn hash_mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
