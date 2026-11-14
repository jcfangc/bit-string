
#[derive(Clone, Copy)]
enum TrailingPattern {
    Zeros,
    Alternating,
    Dense,
}

// ═══════════════════════════════════════════════════════════════════════
// Leading references are named under the same case hierarchy as their
// trailing counterparts, so Divan prints each comparison together.
// ═══════════════════════════════════════════════════════════════════════

#[divan::bench(name = "count_zeros/len_65/all_zeros/leading/ours_str")]
fn lead_65_zeros_str_ref(b: Bencher) {
    trailing_bench_lead_str(b, 65, TrailingPattern::Zeros);
}
#[divan::bench(name = "count_zeros/len_65/all_zeros/leading/ours_string")]
fn lead_65_zeros_string_ref(b: Bencher) {
    trailing_bench_lead_string(b, 65, TrailingPattern::Zeros);
}
#[divan::bench(name = "count_zeros/len_65/dense/leading/ours_str")]
fn lead_65_dense_str_ref(b: Bencher) {
    trailing_bench_lead_str(b, 65, TrailingPattern::Dense);
}
#[divan::bench(name = "count_zeros/len_65/dense/leading/ours_string")]
fn lead_65_dense_string_ref(b: Bencher) {
    trailing_bench_lead_string(b, 65, TrailingPattern::Dense);
}
#[divan::bench(name = "count_zeros/len_4096/all_zeros/leading/ours_str")]
fn lead_4096_zeros_str_ref(b: Bencher) {
    trailing_bench_lead_str(b, 4096, TrailingPattern::Zeros);
}
#[divan::bench(name = "count_zeros/len_4096/all_zeros/leading/ours_string")]
fn lead_4096_zeros_string_ref(b: Bencher) {
    trailing_bench_lead_string(b, 4096, TrailingPattern::Zeros);
}
#[divan::bench(name = "count_zeros/len_65536/all_zeros/leading/ours_str")]
fn lead_65536_zeros_str_ref(b: Bencher) {
    trailing_bench_lead_str(b, 65536, TrailingPattern::Zeros);
}
#[divan::bench(name = "count_zeros/len_65536/all_zeros/leading/ours_string")]
fn lead_65536_zeros_string_ref(b: Bencher) {
    trailing_bench_lead_string(b, 65536, TrailingPattern::Zeros);
}
#[divan::bench(name = "count_ones/len_65/all_zeros/leading/ours_str")]
fn lead_ones_65_zeros_str_ref(b: Bencher) {
    trailing_bench_lead_ones_str(b, 65, TrailingPattern::Zeros);
}
#[divan::bench(name = "count_ones/len_65/all_zeros/leading/ours_string")]
fn lead_ones_65_zeros_string_ref(b: Bencher) {
    trailing_bench_lead_ones_string(b, 65, TrailingPattern::Zeros);
}
#[divan::bench(name = "count_ones/len_4096/all_zeros/leading/ours_str")]
fn lead_ones_4096_zeros_str_ref(b: Bencher) {
    trailing_bench_lead_ones_str(b, 4096, TrailingPattern::Zeros);
}
#[divan::bench(name = "count_ones/len_4096/all_zeros/leading/ours_string")]
fn lead_ones_4096_zeros_string_ref(b: Bencher) {
    trailing_bench_lead_ones_string(b, 4096, TrailingPattern::Zeros);
}

// ═══════════════════════════════════════════════════════════════════════
// trailing_zeros
// ═══════════════════════════════════════════════════════════════════════

#[divan::bench(name = "count_zeros/len_65/all_zeros/trailing/ours_str")]
fn trailing_65_zeros_str(b: Bencher) {
    trailing_bench_str(b, 65, TrailingPattern::Zeros);
}
#[divan::bench(name = "count_zeros/len_65/all_zeros/trailing/ours_string")]
fn trailing_65_zeros_string(b: Bencher) {
    trailing_bench_string(b, 65, TrailingPattern::Zeros);
}

#[divan::bench(name = "count_zeros/len_65/alternating/trailing/ours_str")]
fn trailing_65_alternating_str(b: Bencher) {
    trailing_bench_str(b, 65, TrailingPattern::Alternating);
}
#[divan::bench(name = "count_zeros/len_65/alternating/trailing/ours_string")]
fn trailing_65_alternating_string(b: Bencher) {
    trailing_bench_string(b, 65, TrailingPattern::Alternating);
}

#[divan::bench(name = "count_zeros/len_65/dense/trailing/ours_str")]
fn trailing_65_dense_str(b: Bencher) {
    trailing_bench_str(b, 65, TrailingPattern::Dense);
}
#[divan::bench(name = "count_zeros/len_65/dense/trailing/ours_string")]
fn trailing_65_dense_string(b: Bencher) {
    trailing_bench_string(b, 65, TrailingPattern::Dense);
}

#[divan::bench(name = "count_zeros/len_4096/all_zeros/trailing/ours_str")]
fn trailing_4096_zeros_str(b: Bencher) {
    trailing_bench_str(b, 4096, TrailingPattern::Zeros);
}
#[divan::bench(name = "count_zeros/len_4096/all_zeros/trailing/ours_string")]
fn trailing_4096_zeros_string(b: Bencher) {
    trailing_bench_string(b, 4096, TrailingPattern::Zeros);
}

#[divan::bench(name = "count_zeros/len_4096/dense/trailing/ours_str")]
fn trailing_4096_dense_str(b: Bencher) {
    trailing_bench_str(b, 4096, TrailingPattern::Dense);
}
#[divan::bench(name = "count_zeros/len_4096/dense/trailing/ours_string")]
fn trailing_4096_dense_string(b: Bencher) {
    trailing_bench_string(b, 4096, TrailingPattern::Dense);
}

#[divan::bench(name = "count_zeros/len_65536/all_zeros/trailing/ours_str")]
fn trailing_65536_zeros_str(b: Bencher) {
    trailing_bench_str(b, 65536, TrailingPattern::Zeros);
}
#[divan::bench(name = "count_zeros/len_65536/all_zeros/trailing/ours_string")]
fn trailing_65536_zeros_string(b: Bencher) {
    trailing_bench_string(b, 65536, TrailingPattern::Zeros);
}

#[divan::bench(name = "count_zeros/len_65536/dense/trailing/ours_str")]
fn trailing_65536_dense_str(b: Bencher) {
    trailing_bench_str(b, 65536, TrailingPattern::Dense);
}
#[divan::bench(name = "count_zeros/len_65536/dense/trailing/ours_string")]
fn trailing_65536_dense_string(b: Bencher) {
    trailing_bench_string(b, 65536, TrailingPattern::Dense);
}

#[divan::bench(name = "count_zeros/len_4096/all_zeros/trailing_unaligned_3/ours_str")]
fn trailing_unaligned_3_4096_zeros_str(b: Bencher) {
    trailing_bench_unaligned_str(b, 4096, 3, TrailingPattern::Zeros);
}
#[divan::bench(name = "count_zeros/len_4096/all_zeros/trailing_unaligned_63/ours_str")]
fn trailing_unaligned_63_4096_zeros_str(b: Bencher) {
    trailing_bench_unaligned_str(b, 4096, 63, TrailingPattern::Zeros);
}

// ═══════════════════════════════════════════════════════════════════════
// trailing_ones
// ═══════════════════════════════════════════════════════════════════════

#[divan::bench(name = "count_ones/len_65/all_zeros/trailing/ours_str")]
fn trailing_ones_65_zeros_str(b: Bencher) {
    trailing_bench_str_trailing_ones(b, 65, TrailingPattern::Zeros);
}
#[divan::bench(name = "count_ones/len_65/all_zeros/trailing/ours_string")]
fn trailing_ones_65_zeros_string(b: Bencher) {
    trailing_bench_string_trailing_ones(b, 65, TrailingPattern::Zeros);
}

#[divan::bench(name = "count_ones/len_65/dense/trailing/ours_str")]
fn trailing_ones_65_dense_str(b: Bencher) {
    trailing_bench_str_trailing_ones(b, 65, TrailingPattern::Dense);
}
#[divan::bench(name = "count_ones/len_65/dense/trailing/ours_string")]
fn trailing_ones_65_dense_string(b: Bencher) {
    trailing_bench_string_trailing_ones(b, 65, TrailingPattern::Dense);
}

#[divan::bench(name = "count_ones/len_4096/all_zeros/trailing/ours_str")]
fn trailing_ones_4096_zeros_str(b: Bencher) {
    trailing_bench_str_trailing_ones(b, 4096, TrailingPattern::Zeros);
}
#[divan::bench(name = "count_ones/len_4096/all_zeros/trailing/ours_string")]
fn trailing_ones_4096_zeros_string(b: Bencher) {
    trailing_bench_string_trailing_ones(b, 4096, TrailingPattern::Zeros);
}

// ── leading reference helpers (BitStr) ──────────────────────────────

fn trailing_bench_lead_str(b: Bencher, len: usize, p: TrailingPattern) {
    let bits: BitString = (0..len).map(|i| trailing_bit(i, p)).collect();
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).leading_zeros());
}
fn trailing_bench_lead_string(b: Bencher, len: usize, p: TrailingPattern) {
    let bits: BitString = (0..len).map(|i| trailing_bit(i, p)).collect();
    b.bench(|| black_box(&bits).leading_zeros());
}
fn trailing_bench_lead_ones_str(b: Bencher, len: usize, p: TrailingPattern) {
    let bits: BitString = (0..len).map(|i| trailing_bit(i, p)).collect();
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).leading_ones());
}
fn trailing_bench_lead_ones_string(b: Bencher, len: usize, p: TrailingPattern) {
    let bits: BitString = (0..len).map(|i| trailing_bit(i, p)).collect();
    b.bench(|| black_box(&bits).leading_ones());
}

// ── trailing_zeros helpers ────────────────────────────────────────────

fn trailing_bench_str(b: Bencher, len: usize, p: TrailingPattern) {
    let bits: BitString = (0..len).map(|i| trailing_bit(i, p)).collect();
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).trailing_zeros());
}
fn trailing_bench_string(b: Bencher, len: usize, p: TrailingPattern) {
    let bits: BitString = (0..len).map(|i| trailing_bit(i, p)).collect();
    b.bench(|| black_box(&bits).trailing_zeros());
}
fn trailing_bench_unaligned_str(b: Bencher, len: usize, skip: usize, p: TrailingPattern) {
    let pad = skip + len + 10;
    let mut bits = BitString::zeros(pad);
    for i in skip..skip + len {
        bits.set(i, trailing_bit(i - skip, p));
    }
    let sub = bits
        .as_bit_str()
        .slice(UsizeCO::try_new(skip, skip + len).unwrap());
    b.bench(|| black_box(&sub).trailing_zeros());
}

// ── trailing_ones helpers ─────────────────────────────────────────────

fn trailing_bench_str_trailing_ones(b: Bencher, len: usize, p: TrailingPattern) {
    let bits: BitString = (0..len).map(|i| trailing_bit(i, p)).collect();
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).trailing_ones());
}
fn trailing_bench_string_trailing_ones(b: Bencher, len: usize, p: TrailingPattern) {
    let bits: BitString = (0..len).map(|i| trailing_bit(i, p)).collect();
    b.bench(|| black_box(&bits).trailing_ones());
}

// ── data helpers ──────────────────────────────────────────────────────

fn trailing_bit(i: usize, p: TrailingPattern) -> bool {
    match p {
        TrailingPattern::Zeros => false,
        TrailingPattern::Alternating => i % 2 != 0,
        TrailingPattern::Dense => trailing_mix64(i as u64) & 1 != 0,
    }
}
fn trailing_mix64(mut v: u64) -> u64 {
    v = v.wrapping_add(0x9e37_79b9_7f4a_7c15);
    v = (v ^ (v >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    v = (v ^ (v >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    v ^ (v >> 31)
}
