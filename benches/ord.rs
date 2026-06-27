use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

// ---------------------------------------------------------------------------
// len = 64
// ---------------------------------------------------------------------------

#[divan::bench(name = "cmp/len_64/identical/bit_string")]
fn cmp_len_64_identical_bit_string(b: Bencher) {
    bench_bit_string(b, 64, CmpCase::Identical);
}

#[divan::bench(name = "cmp/len_64/identical/bit_str_unaligned")]
fn cmp_len_64_identical_bit_str_unaligned(b: Bencher) {
    bench_bit_str_unaligned(b, 64, CmpCase::Identical);
}

#[divan::bench(name = "cmp/len_64/identical/string")]
fn cmp_len_64_identical_string(b: Bencher) {
    bench_string(b, 64, CmpCase::Identical);
}

#[divan::bench(name = "cmp/len_64/diff_first/bit_string")]
fn cmp_len_64_diff_first_bit_string(b: Bencher) {
    bench_bit_string(b, 64, CmpCase::DifferAtFirst);
}

#[divan::bench(name = "cmp/len_64/diff_first/bit_str_unaligned")]
fn cmp_len_64_diff_first_bit_str_unaligned(b: Bencher) {
    bench_bit_str_unaligned(b, 64, CmpCase::DifferAtFirst);
}

#[divan::bench(name = "cmp/len_64/diff_first/string")]
fn cmp_len_64_diff_first_string(b: Bencher) {
    bench_string(b, 64, CmpCase::DifferAtFirst);
}

#[divan::bench(name = "cmp/len_64/diff_last/bit_string")]
fn cmp_len_64_diff_last_bit_string(b: Bencher) {
    bench_bit_string(b, 64, CmpCase::DifferAtLast);
}

#[divan::bench(name = "cmp/len_64/diff_last/bit_str_unaligned")]
fn cmp_len_64_diff_last_bit_str_unaligned(b: Bencher) {
    bench_bit_str_unaligned(b, 64, CmpCase::DifferAtLast);
}

#[divan::bench(name = "cmp/len_64/diff_last/string")]
fn cmp_len_64_diff_last_string(b: Bencher) {
    bench_string(b, 64, CmpCase::DifferAtLast);
}

// ---------------------------------------------------------------------------
// len = 4096
// ---------------------------------------------------------------------------

#[divan::bench(name = "cmp/len_4096/identical/bit_string")]
fn cmp_len_4096_identical_bit_string(b: Bencher) {
    bench_bit_string(b, 4096, CmpCase::Identical);
}

#[divan::bench(name = "cmp/len_4096/identical/bit_str_unaligned")]
fn cmp_len_4096_identical_bit_str_unaligned(b: Bencher) {
    bench_bit_str_unaligned(b, 4096, CmpCase::Identical);
}

#[divan::bench(name = "cmp/len_4096/identical/string")]
fn cmp_len_4096_identical_string(b: Bencher) {
    bench_string(b, 4096, CmpCase::Identical);
}

#[divan::bench(name = "cmp/len_4096/diff_first/bit_string")]
fn cmp_len_4096_diff_first_bit_string(b: Bencher) {
    bench_bit_string(b, 4096, CmpCase::DifferAtFirst);
}

#[divan::bench(name = "cmp/len_4096/diff_first/bit_str_unaligned")]
fn cmp_len_4096_diff_first_bit_str_unaligned(b: Bencher) {
    bench_bit_str_unaligned(b, 4096, CmpCase::DifferAtFirst);
}

#[divan::bench(name = "cmp/len_4096/diff_first/string")]
fn cmp_len_4096_diff_first_string(b: Bencher) {
    bench_string(b, 4096, CmpCase::DifferAtFirst);
}

#[divan::bench(name = "cmp/len_4096/diff_last/bit_string")]
fn cmp_len_4096_diff_last_bit_string(b: Bencher) {
    bench_bit_string(b, 4096, CmpCase::DifferAtLast);
}

#[divan::bench(name = "cmp/len_4096/diff_last/bit_str_unaligned")]
fn cmp_len_4096_diff_last_bit_str_unaligned(b: Bencher) {
    bench_bit_str_unaligned(b, 4096, CmpCase::DifferAtLast);
}

#[divan::bench(name = "cmp/len_4096/diff_last/string")]
fn cmp_len_4096_diff_last_string(b: Bencher) {
    bench_string(b, 4096, CmpCase::DifferAtLast);
}

// ---------------------------------------------------------------------------
// len = 65536
// ---------------------------------------------------------------------------

#[divan::bench(name = "cmp/len_65536/identical/bit_string")]
fn cmp_len_65536_identical_bit_string(b: Bencher) {
    bench_bit_string(b, 65536, CmpCase::Identical);
}

#[divan::bench(name = "cmp/len_65536/identical/bit_str_unaligned")]
fn cmp_len_65536_identical_bit_str_unaligned(b: Bencher) {
    bench_bit_str_unaligned(b, 65536, CmpCase::Identical);
}

#[divan::bench(name = "cmp/len_65536/identical/string")]
fn cmp_len_65536_identical_string(b: Bencher) {
    bench_string(b, 65536, CmpCase::Identical);
}

#[divan::bench(name = "cmp/len_65536/diff_first/bit_string")]
fn cmp_len_65536_diff_first_bit_string(b: Bencher) {
    bench_bit_string(b, 65536, CmpCase::DifferAtFirst);
}

#[divan::bench(name = "cmp/len_65536/diff_first/bit_str_unaligned")]
fn cmp_len_65536_diff_first_bit_str_unaligned(b: Bencher) {
    bench_bit_str_unaligned(b, 65536, CmpCase::DifferAtFirst);
}

#[divan::bench(name = "cmp/len_65536/diff_first/string")]
fn cmp_len_65536_diff_first_string(b: Bencher) {
    bench_string(b, 65536, CmpCase::DifferAtFirst);
}

#[divan::bench(name = "cmp/len_65536/diff_last/bit_string")]
fn cmp_len_65536_diff_last_bit_string(b: Bencher) {
    bench_bit_string(b, 65536, CmpCase::DifferAtLast);
}

#[divan::bench(name = "cmp/len_65536/diff_last/bit_str_unaligned")]
fn cmp_len_65536_diff_last_bit_str_unaligned(b: Bencher) {
    bench_bit_str_unaligned(b, 65536, CmpCase::DifferAtLast);
}

#[divan::bench(name = "cmp/len_65536/diff_last/string")]
fn cmp_len_65536_diff_last_string(b: Bencher) {
    bench_string(b, 65536, CmpCase::DifferAtLast);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum CmpCase {
    Identical,
    DifferAtFirst,
    DifferAtLast,
}

fn bench_bit_string(bencher: Bencher, len: usize, case: CmpCase) {
    let (a, b) = make_pair(len, case);
    let va = a.as_bit_str();
    let vb = b.as_bit_str();
    bencher.bench(|| black_box(va).cmp(&black_box(vb)));
}

/// Unaligned view (start=3) vs aligned `as_bit_str()`.
fn bench_bit_str_unaligned(bencher: Bencher, len: usize, case: CmpCase) {
    let base: BitString = (0..len + 3).map(|i| mix64(i as u64) & 1 != 0).collect();
    let va = base.as_bit_str().slice_from(3).slice_until(3 + len);
    // Exact aligned copy of the unaligned view's bits, then flip one
    // bit if needed.
    let mut vb = va.to_bit_string();
    match case {
        CmpCase::Identical => {}
        CmpCase::DifferAtFirst => _ = vb.set(0, !vb.get(0).unwrap()),
        CmpCase::DifferAtLast => _ = vb.set(len - 1, !vb.get(len - 1).unwrap()),
    }
    bencher.bench(|| black_box(va).cmp(&black_box(vb.as_bit_str())));
}

fn bench_string(bencher: Bencher, len: usize, case: CmpCase) {
    let (a, b) = make_string_pair(len, case);
    bencher.bench(|| black_box(a.as_str()).cmp(black_box(b.as_str())));
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

/// Build two aligned `BitString`s for comparison.
fn make_pair(len: usize, case: CmpCase) -> (BitString, BitString) {
    let base: BitString = (0..len).map(|i| mix64(i as u64) & 1 != 0).collect();
    match case {
        CmpCase::Identical => (base.clone(), base),
        CmpCase::DifferAtFirst => {
            let mut b = base.clone();
            b.set(0, !base.get(0).unwrap());
            (base, b)
        }
        CmpCase::DifferAtLast => {
            let mut b = base.clone();
            b.set(len - 1, !base.get(len - 1).unwrap());
            (base, b)
        }
    }
}

fn make_string_pair(len: usize, case: CmpCase) -> (String, String) {
    let a: String = (0..len)
        .map(|i| if mix64(i as u64) & 1 != 0 { '1' } else { '0' })
        .collect();
    let b = match case {
        CmpCase::Identical => a.clone(),
        _ => (0..len)
            .map(|i| {
                let base = mix64(i as u64) & 1 != 0;
                let flip = match case {
                    CmpCase::DifferAtFirst => i == 0,
                    CmpCase::DifferAtLast => i == len - 1,
                    _ => unreachable!(),
                };
                if base ^ flip { '1' } else { '0' }
            })
            .collect(),
    };
    (a, b)
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
