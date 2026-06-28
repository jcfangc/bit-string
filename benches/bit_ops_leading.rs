use bit_string::BitString;
use bitvec_simd::BitVec;
use divan::{Bencher, black_box};
use int_interval::UsizeCO;

fn main() {
    divan::main();
}

#[derive(Clone, Copy)]
enum P {
    Z,
    A,
    D,
}

#[divan::bench(name = "leading_zeros/len_65/all_zeros/ours_str")]
fn l65z_v(b: Bencher) {
    let bits = bs(65, P::Z);
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).leading_zeros());
}
#[divan::bench(name = "leading_zeros/len_65/all_zeros/ours_string")]
fn l65z_o(b: Bencher) {
    let bits = bs(65, P::Z);
    b.bench(|| black_box(&bits).leading_zeros());
}
#[divan::bench(name = "leading_zeros/len_65/all_zeros/bitvec_simd")]
fn l65z_b(b: Bencher) {
    let bv = BitVec::from_bool_iterator((0..65).map(|i| bit(i, P::Z)));
    b.bench(|| black_box(&bv).leading_zeros());
}

#[divan::bench(name = "leading_zeros/len_65/alternating/ours_str")]
fn l65a_v(b: Bencher) {
    let bits = bs(65, P::A);
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).leading_zeros());
}
#[divan::bench(name = "leading_zeros/len_65/alternating/ours_string")]
fn l65a_o(b: Bencher) {
    let bits = bs(65, P::A);
    b.bench(|| black_box(&bits).leading_zeros());
}
#[divan::bench(name = "leading_zeros/len_65/alternating/bitvec_simd")]
fn l65a_b(b: Bencher) {
    let bv = BitVec::from_bool_iterator((0..65).map(|i| bit(i, P::A)));
    b.bench(|| black_box(&bv).leading_zeros());
}

#[divan::bench(name = "leading_zeros/len_65/dense/ours_str")]
fn l65d_v(b: Bencher) {
    let bits = bs(65, P::D);
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).leading_zeros());
}
#[divan::bench(name = "leading_zeros/len_65/dense/ours_string")]
fn l65d_o(b: Bencher) {
    let bits = bs(65, P::D);
    b.bench(|| black_box(&bits).leading_zeros());
}
#[divan::bench(name = "leading_zeros/len_65/dense/bitvec_simd")]
fn l65d_b(b: Bencher) {
    let bv = BitVec::from_bool_iterator((0..65).map(|i| bit(i, P::D)));
    b.bench(|| black_box(&bv).leading_zeros());
}

#[divan::bench(name = "leading_zeros/len_4096/all_zeros/ours_str")]
fn l4z_v(b: Bencher) {
    let bits = bs(4096, P::Z);
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).leading_zeros());
}
#[divan::bench(name = "leading_zeros/len_4096/all_zeros/ours_string")]
fn l4z_o(b: Bencher) {
    let bits = bs(4096, P::Z);
    b.bench(|| black_box(&bits).leading_zeros());
}
#[divan::bench(name = "leading_zeros/len_4096/all_zeros/bitvec_simd")]
fn l4z_b(b: Bencher) {
    let bv = BitVec::from_bool_iterator((0..4096).map(|i| bit(i, P::Z)));
    b.bench(|| black_box(&bv).leading_zeros());
}

#[divan::bench(name = "leading_zeros/len_4096/dense/ours_str")]
fn l4d_v(b: Bencher) {
    let bits = bs(4096, P::D);
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).leading_zeros());
}
#[divan::bench(name = "leading_zeros/len_4096/dense/ours_string")]
fn l4d_o(b: Bencher) {
    let bits = bs(4096, P::D);
    b.bench(|| black_box(&bits).leading_zeros());
}
#[divan::bench(name = "leading_zeros/len_4096/dense/bitvec_simd")]
fn l4d_b(b: Bencher) {
    let bv = BitVec::from_bool_iterator((0..4096).map(|i| bit(i, P::D)));
    b.bench(|| black_box(&bv).leading_zeros());
}

#[divan::bench(name = "leading_zeros/len_65536/all_zeros/ours_str")]
fn l6z_v(b: Bencher) {
    let bits = bs(65536, P::Z);
    let v = bits.as_bit_str();
    b.bench(|| black_box(&v).leading_zeros());
}
#[divan::bench(name = "leading_zeros/len_65536/all_zeros/ours_string")]
fn l6z_o(b: Bencher) {
    let bits = bs(65536, P::Z);
    b.bench(|| black_box(&bits).leading_zeros());
}
#[divan::bench(name = "leading_zeros/len_65536/all_zeros/bitvec_simd")]
fn l6z_b(b: Bencher) {
    let bv = BitVec::from_bool_iterator((0..65536).map(|i| bit(i, P::Z)));
    b.bench(|| black_box(&bv).leading_zeros());
}

#[divan::bench(name = "leading_zeros/unaligned_3/len_4096/all_zeros/ours_str")]
fn lu3_v(b: Bencher) {
    let bits = bs_unaligned(4096, 3, P::Z);
    let sub = slice_sub(&bits, 3, 4096);
    b.bench(|| black_box(&sub).leading_zeros());
}
#[divan::bench(name = "leading_zeros/unaligned_31/len_4096/all_zeros/ours_str")]
fn lu31_v(b: Bencher) {
    let bits = bs_unaligned(4096, 31, P::Z);
    let sub = slice_sub(&bits, 31, 4096);
    b.bench(|| black_box(&sub).leading_zeros());
}
#[divan::bench(name = "leading_zeros/unaligned_63/len_4096/all_zeros/ours_str")]
fn lu63_v(b: Bencher) {
    let bits = bs_unaligned(4096, 63, P::Z);
    let sub = slice_sub(&bits, 63, 4096);
    b.bench(|| black_box(&sub).leading_zeros());
}

fn bs(len: usize, p: P) -> BitString {
    (0..len).map(|i| bit(i, p)).collect()
}
fn bs_unaligned(len: usize, skip: usize, p: P) -> BitString {
    let pad = skip + len + 10;
    let mut bits = BitString::zeros(pad);
    for i in skip..skip + len {
        bits.set(i, bit(i - skip, p));
    }
    bits
}
fn slice_sub(bits: &BitString, skip: usize, len: usize) -> bit_string::BitStr<'_> {
    bits.as_bit_str()
        .slice(UsizeCO::try_new(skip, skip + len).unwrap())
}
fn bit(i: usize, p: P) -> bool {
    match p {
        P::Z => false,
        P::A => i % 2 != 0,
        P::D => mix64(i as u64) & 1 != 0,
    }
}
fn mix64(mut v: u64) -> u64 {
    v = v.wrapping_add(0x9e37_79b9_7f4a_7c15);
    v = (v ^ (v >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    v = (v ^ (v >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    v ^ (v >> 31)
}
