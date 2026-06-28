use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

#[derive(Clone, Copy)]
enum Pattern {
    Dense,
    Sparse,
    Alternating,
}

// ---------------------------------------------------------------------------
// BitString
// ---------------------------------------------------------------------------

#[divan::bench(name = "hash/len_64/dense/ours_string")]
fn hash_len_64_dense_bit_string(b: Bencher) {
    bench_bit_string(b, 64, Pattern::Dense);
}

#[divan::bench(name = "hash/len_64/dense/ours_str")]
fn hash_len_64_dense_bit_str(b: Bencher) {
    bench_bit_str(b, 64, Pattern::Dense);
}

#[divan::bench(name = "hash/len_64/dense/string")]
fn hash_len_64_dense_string(b: Bencher) {
    bench_string(b, 64, Pattern::Dense);
}

#[divan::bench(name = "hash/len_64/dense/str")]
fn hash_len_64_dense_str(b: Bencher) {
    bench_str(b, 64, Pattern::Dense);
}

#[divan::bench(name = "hash/len_64/sparse/ours_string")]
fn hash_len_64_sparse_bit_string(b: Bencher) {
    bench_bit_string(b, 64, Pattern::Sparse);
}

#[divan::bench(name = "hash/len_64/sparse/ours_str")]
fn hash_len_64_sparse_bit_str(b: Bencher) {
    bench_bit_str(b, 64, Pattern::Sparse);
}

#[divan::bench(name = "hash/len_64/sparse/string")]
fn hash_len_64_sparse_string(b: Bencher) {
    bench_string(b, 64, Pattern::Sparse);
}

#[divan::bench(name = "hash/len_64/sparse/str")]
fn hash_len_64_sparse_str(b: Bencher) {
    bench_str(b, 64, Pattern::Sparse);
}

#[divan::bench(name = "hash/len_64/alternating/ours_string")]
fn hash_len_64_alternating_bit_string(b: Bencher) {
    bench_bit_string(b, 64, Pattern::Alternating);
}

#[divan::bench(name = "hash/len_64/alternating/ours_str")]
fn hash_len_64_alternating_bit_str(b: Bencher) {
    bench_bit_str(b, 64, Pattern::Alternating);
}

#[divan::bench(name = "hash/len_64/alternating/string")]
fn hash_len_64_alternating_string(b: Bencher) {
    bench_string(b, 64, Pattern::Alternating);
}

#[divan::bench(name = "hash/len_64/alternating/str")]
fn hash_len_64_alternating_str(b: Bencher) {
    bench_str(b, 64, Pattern::Alternating);
}

// ---------------------------------------------------------------------------
// len = 4096
// ---------------------------------------------------------------------------

#[divan::bench(name = "hash/len_4096/dense/ours_string")]
fn hash_len_4096_dense_bit_string(b: Bencher) {
    bench_bit_string(b, 4096, Pattern::Dense);
}

#[divan::bench(name = "hash/len_4096/dense/ours_str")]
fn hash_len_4096_dense_bit_str(b: Bencher) {
    bench_bit_str(b, 4096, Pattern::Dense);
}

#[divan::bench(name = "hash/len_4096/dense/string")]
fn hash_len_4096_dense_string(b: Bencher) {
    bench_string(b, 4096, Pattern::Dense);
}

#[divan::bench(name = "hash/len_4096/dense/str")]
fn hash_len_4096_dense_str(b: Bencher) {
    bench_str(b, 4096, Pattern::Dense);
}

#[divan::bench(name = "hash/len_4096/sparse/ours_string")]
fn hash_len_4096_sparse_bit_string(b: Bencher) {
    bench_bit_string(b, 4096, Pattern::Sparse);
}

#[divan::bench(name = "hash/len_4096/sparse/ours_str")]
fn hash_len_4096_sparse_bit_str(b: Bencher) {
    bench_bit_str(b, 4096, Pattern::Sparse);
}

#[divan::bench(name = "hash/len_4096/sparse/string")]
fn hash_len_4096_sparse_string(b: Bencher) {
    bench_string(b, 4096, Pattern::Sparse);
}

#[divan::bench(name = "hash/len_4096/sparse/str")]
fn hash_len_4096_sparse_str(b: Bencher) {
    bench_str(b, 4096, Pattern::Sparse);
}

#[divan::bench(name = "hash/len_4096/alternating/ours_string")]
fn hash_len_4096_alternating_bit_string(b: Bencher) {
    bench_bit_string(b, 4096, Pattern::Alternating);
}

#[divan::bench(name = "hash/len_4096/alternating/ours_str")]
fn hash_len_4096_alternating_bit_str(b: Bencher) {
    bench_bit_str(b, 4096, Pattern::Alternating);
}

#[divan::bench(name = "hash/len_4096/alternating/string")]
fn hash_len_4096_alternating_string(b: Bencher) {
    bench_string(b, 4096, Pattern::Alternating);
}

#[divan::bench(name = "hash/len_4096/alternating/str")]
fn hash_len_4096_alternating_str(b: Bencher) {
    bench_str(b, 4096, Pattern::Alternating);
}

// ---------------------------------------------------------------------------
// len = 65536
// ---------------------------------------------------------------------------

#[divan::bench(name = "hash/len_65536/dense/ours_string")]
fn hash_len_65536_dense_bit_string(b: Bencher) {
    bench_bit_string(b, 65536, Pattern::Dense);
}

#[divan::bench(name = "hash/len_65536/dense/ours_str")]
fn hash_len_65536_dense_bit_str(b: Bencher) {
    bench_bit_str(b, 65536, Pattern::Dense);
}

#[divan::bench(name = "hash/len_65536/dense/string")]
fn hash_len_65536_dense_string(b: Bencher) {
    bench_string(b, 65536, Pattern::Dense);
}

#[divan::bench(name = "hash/len_65536/dense/str")]
fn hash_len_65536_dense_str(b: Bencher) {
    bench_str(b, 65536, Pattern::Dense);
}

#[divan::bench(name = "hash/len_65536/sparse/ours_string")]
fn hash_len_65536_sparse_bit_string(b: Bencher) {
    bench_bit_string(b, 65536, Pattern::Sparse);
}

#[divan::bench(name = "hash/len_65536/sparse/ours_str")]
fn hash_len_65536_sparse_bit_str(b: Bencher) {
    bench_bit_str(b, 65536, Pattern::Sparse);
}

#[divan::bench(name = "hash/len_65536/sparse/string")]
fn hash_len_65536_sparse_string(b: Bencher) {
    bench_string(b, 65536, Pattern::Sparse);
}

#[divan::bench(name = "hash/len_65536/sparse/str")]
fn hash_len_65536_sparse_str(b: Bencher) {
    bench_str(b, 65536, Pattern::Sparse);
}

#[divan::bench(name = "hash/len_65536/alternating/ours_string")]
fn hash_len_65536_alternating_bit_string(b: Bencher) {
    bench_bit_string(b, 65536, Pattern::Alternating);
}

#[divan::bench(name = "hash/len_65536/alternating/ours_str")]
fn hash_len_65536_alternating_bit_str(b: Bencher) {
    bench_bit_str(b, 65536, Pattern::Alternating);
}

#[divan::bench(name = "hash/len_65536/alternating/string")]
fn hash_len_65536_alternating_string(b: Bencher) {
    bench_string(b, 65536, Pattern::Alternating);
}

#[divan::bench(name = "hash/len_65536/alternating/str")]
fn hash_len_65536_alternating_str(b: Bencher) {
    bench_str(b, 65536, Pattern::Alternating);
}

// ---------------------------------------------------------------------------
// Bench helpers
// ---------------------------------------------------------------------------

fn bench_bit_string(bencher: Bencher, len: usize, pattern: Pattern) {
    let bits = make_bit_string(len, pattern);
    bencher.bench(|| {
        let mut h = DefaultHasher::new();
        black_box(&bits).hash(&mut h);
        black_box(h.finish())
    });
}

fn bench_bit_str(bencher: Bencher, len: usize, pattern: Pattern) {
    let bits = make_bit_string(len, pattern);
    let view = bits.as_bit_str();
    bencher.bench(|| {
        let mut h = DefaultHasher::new();
        black_box(&view).hash(&mut h);
        black_box(h.finish())
    });
}

fn bench_string(bencher: Bencher, len: usize, pattern: Pattern) {
    let s = make_string(len, pattern);
    bencher.bench(|| {
        let mut h = DefaultHasher::new();
        black_box(&s).hash(&mut h);
        black_box(h.finish())
    });
}

fn bench_str(bencher: Bencher, len: usize, pattern: Pattern) {
    let s = make_string(len, pattern);
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
fn make_bit_string(len: usize, pattern: Pattern) -> BitString {
    (0..len).map(|index| bit_at(index, pattern)).collect()
}

#[inline]
fn make_string(len: usize, pattern: Pattern) -> String {
    (0..len)
        .map(|index| if bit_at(index, pattern) { '1' } else { '0' })
        .collect()
}

#[inline]
fn bit_at(index: usize, pattern: Pattern) -> bool {
    match pattern {
        Pattern::Dense => mix64(index as u64) & 1 != 0,
        Pattern::Sparse => mix64(index as u64) & 63 == 0,
        Pattern::Alternating => index % 2 != 0,
    }
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
