
#[derive(Clone, Copy)]
enum LeadingPattern {
    Zeros,
    Alternating,
    Dense,
}

#[divan::bench(name = "leading_zeros/len_65/all_zeros/ours_str")]
fn leading_65_zeros_str(b: Bencher) {
    leading_bench_str(b, 65, LeadingPattern::Zeros);
}
#[divan::bench(name = "leading_zeros/len_65/all_zeros/ours_string")]
fn leading_65_zeros_string(b: Bencher) {
    leading_bench_string(b, 65, LeadingPattern::Zeros);
}
#[divan::bench(name = "leading_zeros/len_65/all_zeros/bitvec_simd")]
fn leading_65_zeros_bitvec(b: Bencher) {
    leading_bench_bitvec(b, 65, LeadingPattern::Zeros);
}

#[divan::bench(name = "leading_zeros/len_65/alternating/ours_str")]
fn leading_65_alternating_str(b: Bencher) {
    leading_bench_str(b, 65, LeadingPattern::Alternating);
}
#[divan::bench(name = "leading_zeros/len_65/alternating/ours_string")]
fn leading_65_alternating_string(b: Bencher) {
    leading_bench_string(b, 65, LeadingPattern::Alternating);
}
#[divan::bench(name = "leading_zeros/len_65/alternating/bitvec_simd")]
fn leading_65_alternating_bitvec(b: Bencher) {
    leading_bench_bitvec(b, 65, LeadingPattern::Alternating);
}

#[divan::bench(name = "leading_zeros/len_65/dense/ours_str")]
fn leading_65_dense_str(b: Bencher) {
    leading_bench_str(b, 65, LeadingPattern::Dense);
}
#[divan::bench(name = "leading_zeros/len_65/dense/ours_string")]
fn leading_65_dense_string(b: Bencher) {
    leading_bench_string(b, 65, LeadingPattern::Dense);
}
#[divan::bench(name = "leading_zeros/len_65/dense/bitvec_simd")]
fn leading_65_dense_bitvec(b: Bencher) {
    leading_bench_bitvec(b, 65, LeadingPattern::Dense);
}

#[divan::bench(name = "leading_zeros/len_4096/all_zeros/ours_str")]
fn leading_4096_zeros_str(b: Bencher) {
    leading_bench_str(b, 4096, LeadingPattern::Zeros);
}
#[divan::bench(name = "leading_zeros/len_4096/all_zeros/ours_string")]
fn leading_4096_zeros_string(b: Bencher) {
    leading_bench_string(b, 4096, LeadingPattern::Zeros);
}
#[divan::bench(name = "leading_zeros/len_4096/all_zeros/bitvec_simd")]
fn leading_4096_zeros_bitvec(b: Bencher) {
    leading_bench_bitvec(b, 4096, LeadingPattern::Zeros);
}

#[divan::bench(name = "leading_zeros/len_4096/dense/ours_str")]
fn leading_4096_dense_str(b: Bencher) {
    leading_bench_str(b, 4096, LeadingPattern::Dense);
}
#[divan::bench(name = "leading_zeros/len_4096/dense/ours_string")]
fn leading_4096_dense_string(b: Bencher) {
    leading_bench_string(b, 4096, LeadingPattern::Dense);
}
#[divan::bench(name = "leading_zeros/len_4096/dense/bitvec_simd")]
fn leading_4096_dense_bitvec(b: Bencher) {
    leading_bench_bitvec(b, 4096, LeadingPattern::Dense);
}

#[divan::bench(name = "leading_zeros/len_65536/all_zeros/ours_str")]
fn leading_65536_zeros_str(b: Bencher) {
    leading_bench_str(b, 65536, LeadingPattern::Zeros);
}
#[divan::bench(name = "leading_zeros/len_65536/all_zeros/ours_string")]
fn leading_65536_zeros_string(b: Bencher) {
    leading_bench_string(b, 65536, LeadingPattern::Zeros);
}
#[divan::bench(name = "leading_zeros/len_65536/all_zeros/bitvec_simd")]
fn leading_65536_zeros_bitvec(b: Bencher) {
    leading_bench_bitvec(b, 65536, LeadingPattern::Zeros);
}

#[divan::bench(name = "leading_zeros/unaligned_3/len_4096/all_zeros/ours_str")]
fn leading_unaligned_3_4096_zeros_str(b: Bencher) {
    leading_bench_unaligned_str(b, 4096, 3, LeadingPattern::Zeros);
}
#[divan::bench(name = "leading_zeros/unaligned_31/len_4096/all_zeros/ours_str")]
fn leading_unaligned_31_4096_zeros_str(b: Bencher) {
    leading_bench_unaligned_str(b, 4096, 31, LeadingPattern::Zeros);
}
#[divan::bench(name = "leading_zeros/unaligned_63/len_4096/all_zeros/ours_str")]
fn leading_unaligned_63_4096_zeros_str(b: Bencher) {
    leading_bench_unaligned_str(b, 4096, 63, LeadingPattern::Zeros);
}

fn leading_bench_str(b: Bencher, len: usize, p: LeadingPattern) {
    let bits: BitString = (0..len).map(|i| leading_bit(i, p)).collect();
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).leading_zeros());
}
fn leading_bench_string(b: Bencher, len: usize, p: LeadingPattern) {
    let bits: BitString = (0..len).map(|i| leading_bit(i, p)).collect();
    b.bench(|| black_box(&bits).leading_zeros());
}
fn leading_bench_bitvec(b: Bencher, len: usize, p: LeadingPattern) {
    let bv = BitVec::from_bool_iterator((0..len).map(|i| leading_bit(i, p)));
    b.bench(|| black_box(&bv).leading_zeros());
}
fn leading_bench_unaligned_str(b: Bencher, len: usize, skip: usize, p: LeadingPattern) {
    let pad = skip + len + 10;
    let mut bits = BitString::zeros(pad);
    for i in skip..skip + len {
        bits.set(i, leading_bit(i - skip, p));
    }
    let sub = bits
        .as_bit_str()
        .slice(UsizeCO::try_new(skip, skip + len).unwrap());
    b.bench(|| black_box(&sub).leading_zeros());
}

fn leading_bit(i: usize, p: LeadingPattern) -> bool {
    match p {
        LeadingPattern::Zeros => false,
        LeadingPattern::Alternating => i % 2 != 0,
        LeadingPattern::Dense => leading_mix64(i as u64) & 1 != 0,
    }
}
fn leading_mix64(mut v: u64) -> u64 {
    v = v.wrapping_add(0x9e37_79b9_7f4a_7c15);
    v = (v ^ (v >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    v = (v ^ (v >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    v ^ (v >> 31)
}
