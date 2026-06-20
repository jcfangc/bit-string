use bit_string::BitString;
use divan::{Bencher, black_box};
use int_interval::UsizeCO;

fn main() {
    divan::main();
}

// Drain a ~20% interval from the middle — compare allocating vs in-place.
#[divan::bench(name = "drain/len_4096_mid/allocating")]
fn drain_len_4096_mid_allocating(bencher: Bencher) {
    bench_drain_interval(bencher, 4096);
}

#[divan::bench(name = "drain/len_4096_mid/in_place")]
fn drain_len_4096_mid_in_place(bencher: Bencher) {
    bench_drain_interval_assign(bencher, 4096);
}

// Drain a word-sized chunk from a 65-bit string — triggers the shiftable
// witness fast path (gap >= 64 && non-empty tail).
#[divan::bench(name = "drain/len_65536_mid/allocating")]
fn drain_len_65536_mid_allocating(bencher: Bencher) {
    bench_drain_interval(bencher, 65_536);
}

#[divan::bench(name = "drain/len_65536_mid/in_place")]
fn drain_len_65536_mid_in_place(bencher: Bencher) {
    bench_drain_interval_assign(bencher, 65_536);
}

// Drain a small interval (< 64 bits) — falls back to allocate path.
#[divan::bench(name = "drain/len_4096_small/allocating")]
fn drain_len_4096_small_allocating(bencher: Bencher) {
    bench_drain_small_interval(bencher, 4096);
}

#[divan::bench(name = "drain/len_4096_small/in_place")]
fn drain_len_4096_small_in_place(bencher: Bencher) {
    bench_drain_small_interval_assign(bencher, 4096);
}

fn bench_drain_interval(bencher: Bencher, len: usize) {
    let input = make_bit_string(len);
    let mid = len / 5;
    let interval = UsizeCO::try_new(mid, mid * 2).unwrap();

    bencher.bench(|| {
        let bits = black_box(input.clone());
        black_box(bits.drain_interval(interval))
    });
}

fn bench_drain_interval_assign(bencher: Bencher, len: usize) {
    let input = make_bit_string(len);
    let mid = len / 5;
    let interval = UsizeCO::try_new(mid, mid * 2).unwrap();

    bencher.bench(|| {
        let mut bits = black_box(input.clone());
        bits.drain_interval_assign(interval);
        black_box(bits)
    });
}

fn bench_drain_small_interval(bencher: Bencher, len: usize) {
    let input = make_bit_string(len);
    let mid = len / 5;
    let interval = UsizeCO::try_new(mid, mid + 7).unwrap();

    bencher.bench(|| {
        let bits = black_box(input.clone());
        black_box(bits.drain_interval(interval))
    });
}

fn bench_drain_small_interval_assign(bencher: Bencher, len: usize) {
    let input = make_bit_string(len);
    let mid = len / 5;
    let interval = UsizeCO::try_new(mid, mid + 7).unwrap();

    bencher.bench(|| {
        let mut bits = black_box(input.clone());
        bits.drain_interval_assign(interval);
        black_box(bits)
    });
}

#[inline]
fn make_bit_string(len: usize) -> BitString {
    (0..len).map(|index| bit_at(index)).collect()
}

#[inline]
fn bit_at(index: usize) -> bool {
    mix64(index as u64) & 1 != 0
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
