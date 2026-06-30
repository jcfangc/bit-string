use bit_string::BitString;
use divan::{Bencher, black_box};
use int_interval::UsizeCO;

fn main() {
    divan::main();
}

#[derive(Clone, Copy)]
enum Pattern {
    Zeros,
    Alternating,
    Dense,
}

// ═══════════════════════════════════════════════════════════════════════
// trailing_zeros
// ═══════════════════════════════════════════════════════════════════════

#[divan::bench(name = "trailing_zeros/len_65/all_zeros/ours_str")]
fn trailing_65_zeros_str(b: Bencher) {
    bench_str(b, 65, Pattern::Zeros);
}
#[divan::bench(name = "trailing_zeros/len_65/all_zeros/ours_string")]
fn trailing_65_zeros_string(b: Bencher) {
    bench_string(b, 65, Pattern::Zeros);
}

#[divan::bench(name = "trailing_zeros/len_65/alternating/ours_str")]
fn trailing_65_alternating_str(b: Bencher) {
    bench_str(b, 65, Pattern::Alternating);
}
#[divan::bench(name = "trailing_zeros/len_65/alternating/ours_string")]
fn trailing_65_alternating_string(b: Bencher) {
    bench_string(b, 65, Pattern::Alternating);
}

#[divan::bench(name = "trailing_zeros/len_65/dense/ours_str")]
fn trailing_65_dense_str(b: Bencher) {
    bench_str(b, 65, Pattern::Dense);
}
#[divan::bench(name = "trailing_zeros/len_65/dense/ours_string")]
fn trailing_65_dense_string(b: Bencher) {
    bench_string(b, 65, Pattern::Dense);
}

#[divan::bench(name = "trailing_zeros/len_4096/all_zeros/ours_str")]
fn trailing_4096_zeros_str(b: Bencher) {
    bench_str(b, 4096, Pattern::Zeros);
}
#[divan::bench(name = "trailing_zeros/len_4096/all_zeros/ours_string")]
fn trailing_4096_zeros_string(b: Bencher) {
    bench_string(b, 4096, Pattern::Zeros);
}

#[divan::bench(name = "trailing_zeros/len_4096/dense/ours_str")]
fn trailing_4096_dense_str(b: Bencher) {
    bench_str(b, 4096, Pattern::Dense);
}
#[divan::bench(name = "trailing_zeros/len_4096/dense/ours_string")]
fn trailing_4096_dense_string(b: Bencher) {
    bench_string(b, 4096, Pattern::Dense);
}

#[divan::bench(name = "trailing_zeros/len_65536/all_zeros/ours_str")]
fn trailing_65536_zeros_str(b: Bencher) {
    bench_str(b, 65536, Pattern::Zeros);
}
#[divan::bench(name = "trailing_zeros/len_65536/all_zeros/ours_string")]
fn trailing_65536_zeros_string(b: Bencher) {
    bench_string(b, 65536, Pattern::Zeros);
}

#[divan::bench(name = "trailing_zeros/len_65536/dense/ours_str")]
fn trailing_65536_dense_str(b: Bencher) {
    bench_str(b, 65536, Pattern::Dense);
}
#[divan::bench(name = "trailing_zeros/len_65536/dense/ours_string")]
fn trailing_65536_dense_string(b: Bencher) {
    bench_string(b, 65536, Pattern::Dense);
}

#[divan::bench(name = "trailing_zeros/unaligned_3/len_4096/all_zeros/ours_str")]
fn trailing_unaligned_3_4096_zeros_str(b: Bencher) {
    bench_unaligned_str(b, 4096, 3, Pattern::Zeros);
}
#[divan::bench(name = "trailing_zeros/unaligned_63/len_4096/all_zeros/ours_str")]
fn trailing_unaligned_63_4096_zeros_str(b: Bencher) {
    bench_unaligned_str(b, 4096, 63, Pattern::Zeros);
}

// ═══════════════════════════════════════════════════════════════════════
// trailing_ones
// ═══════════════════════════════════════════════════════════════════════

#[divan::bench(name = "trailing_ones/len_65/all_zeros/ours_str")]
fn trailing_ones_65_zeros_str(b: Bencher) {
    bench_str_trailing_ones(b, 65, Pattern::Zeros);
}
#[divan::bench(name = "trailing_ones/len_65/all_zeros/ours_string")]
fn trailing_ones_65_zeros_string(b: Bencher) {
    bench_string_trailing_ones(b, 65, Pattern::Zeros);
}

#[divan::bench(name = "trailing_ones/len_65/dense/ours_str")]
fn trailing_ones_65_dense_str(b: Bencher) {
    bench_str_trailing_ones(b, 65, Pattern::Dense);
}
#[divan::bench(name = "trailing_ones/len_65/dense/ours_string")]
fn trailing_ones_65_dense_string(b: Bencher) {
    bench_string_trailing_ones(b, 65, Pattern::Dense);
}

#[divan::bench(name = "trailing_ones/len_4096/all_zeros/ours_str")]
fn trailing_ones_4096_zeros_str(b: Bencher) {
    bench_str_trailing_ones(b, 4096, Pattern::Zeros);
}
#[divan::bench(name = "trailing_ones/len_4096/all_zeros/ours_string")]
fn trailing_ones_4096_zeros_string(b: Bencher) {
    bench_string_trailing_ones(b, 4096, Pattern::Zeros);
}

// ── trailing_zeros helpers ────────────────────────────────────────────

fn bench_str(b: Bencher, len: usize, p: Pattern) {
    let bits: BitString = (0..len).map(|i| bit(i, p)).collect();
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).trailing_zeros());
}
fn bench_string(b: Bencher, len: usize, p: Pattern) {
    let bits: BitString = (0..len).map(|i| bit(i, p)).collect();
    b.bench(|| black_box(&bits).trailing_zeros());
}
fn bench_unaligned_str(b: Bencher, len: usize, skip: usize, p: Pattern) {
    let pad = skip + len + 10;
    let mut bits = BitString::zeros(pad);
    for i in skip..skip + len {
        bits.set(i, bit(i - skip, p));
    }
    let sub = bits
        .as_bit_str()
        .slice(UsizeCO::try_new(skip, skip + len).unwrap());
    b.bench(|| black_box(&sub).trailing_zeros());
}

// ── trailing_ones helpers ─────────────────────────────────────────────

fn bench_str_trailing_ones(b: Bencher, len: usize, p: Pattern) {
    let bits: BitString = (0..len).map(|i| bit(i, p)).collect();
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).trailing_ones());
}
fn bench_string_trailing_ones(b: Bencher, len: usize, p: Pattern) {
    let bits: BitString = (0..len).map(|i| bit(i, p)).collect();
    b.bench(|| black_box(&bits).trailing_ones());
}

// ── data helpers ──────────────────────────────────────────────────────

fn bit(i: usize, p: Pattern) -> bool {
    match p {
        Pattern::Zeros => false,
        Pattern::Alternating => i % 2 != 0,
        Pattern::Dense => mix64(i as u64) & 1 != 0,
    }
}
fn mix64(mut v: u64) -> u64 {
    v = v.wrapping_add(0x9e37_79b9_7f4a_7c15);
    v = (v ^ (v >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    v = (v ^ (v >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    v ^ (v >> 31)
}
