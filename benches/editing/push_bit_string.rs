
#[divan::bench(name = "push_bit_string/len_65/ours_string")]
fn push_bits_len_65_bit_string(bencher: Bencher) {
    push_bit_string_bench_bit_string(bencher, 65);
}

#[divan::bench(name = "push_bit_string/len_65/string")]
fn push_bits_len_65_string(bencher: Bencher) {
    push_bit_string_bench_string(bencher, 65);
}

#[divan::bench(name = "push_bit_string/len_65536/ours_string")]
fn push_bits_len_65536_bit_string(bencher: Bencher) {
    push_bit_string_bench_bit_string(bencher, 65_536);
}

#[divan::bench(name = "push_bit_string/len_65536/string")]
fn push_bits_len_65536_string(bencher: Bencher) {
    push_bit_string_bench_string(bencher, 65_536);
}

macro_rules! define_aligned_append_cases {
    ($($name:ident: $lhs_len:literal, $rhs_words:literal, $rhs_tail:literal),* $(,)?) => {
        $(
            #[cfg(feature = "characterization")]
            #[divan::bench(name = concat!(
                "push_bit_string/lhs_bits_", stringify!($lhs_len),
                "/rhs_words_", stringify!($rhs_words),
                "/tail_bits_", stringify!($rhs_tail),
                "/ours_string"
            ))]
            fn $name(bencher: Bencher) {
                push_bit_string_bench_aligned(bencher, $lhs_len, $rhs_words, $rhs_tail);
            }
        )*
    };
}

macro_rules! define_default_aligned_append_case {
    ($name:ident, $lhs_len:literal, $rhs_words:literal, $rhs_tail:literal) => {
        #[cfg(not(feature = "characterization"))]
        #[divan::bench(name = concat!(
            "push_bit_string/lhs_bits_", stringify!($lhs_len),
            "/rhs_words_", stringify!($rhs_words),
            "/tail_bits_", stringify!($rhs_tail),
            "/ours_string"
        ))]
        fn $name(bencher: Bencher) {
            push_bit_string_bench_aligned(bencher, $lhs_len, $rhs_words, $rhs_tail);
        }
    };
}

define_default_aligned_append_case!(aligned_append_default_small, 1, 4, 0);
define_default_aligned_append_case!(aligned_append_default_simd, 1, 64, 0);
define_default_aligned_append_case!(aligned_append_default_tail, 63, 64, 1);
define_default_aligned_append_case!(aligned_append_default_large, 31, 1024, 0);

define_aligned_append_cases!(
    aligned_append_l1_w4_t0: 1, 4, 0,
    aligned_append_l1_w4_t1: 1, 4, 1,
    aligned_append_l1_w5_t0: 1, 5, 0,
    aligned_append_l1_w5_t1: 1, 5, 1,
    aligned_append_l1_w16_t0: 1, 16, 0,
    aligned_append_l1_w16_t1: 1, 16, 1,
    aligned_append_l1_w64_t0: 1, 64, 0,
    aligned_append_l1_w64_t1: 1, 64, 1,
    aligned_append_l1_w1024_t0: 1, 1024, 0,
    aligned_append_l1_w1024_t1: 1, 1024, 1,
    aligned_append_l31_w4_t0: 31, 4, 0,
    aligned_append_l31_w4_t1: 31, 4, 1,
    aligned_append_l31_w5_t0: 31, 5, 0,
    aligned_append_l31_w5_t1: 31, 5, 1,
    aligned_append_l31_w16_t0: 31, 16, 0,
    aligned_append_l31_w16_t1: 31, 16, 1,
    aligned_append_l31_w64_t0: 31, 64, 0,
    aligned_append_l31_w64_t1: 31, 64, 1,
    aligned_append_l31_w1024_t0: 31, 1024, 0,
    aligned_append_l31_w1024_t1: 31, 1024, 1,
    aligned_append_l63_w4_t0: 63, 4, 0,
    aligned_append_l63_w4_t1: 63, 4, 1,
    aligned_append_l63_w5_t0: 63, 5, 0,
    aligned_append_l63_w5_t1: 63, 5, 1,
    aligned_append_l63_w16_t0: 63, 16, 0,
    aligned_append_l63_w16_t1: 63, 16, 1,
    aligned_append_l63_w64_t0: 63, 64, 0,
    aligned_append_l63_w64_t1: 63, 64, 1,
    aligned_append_l63_w1024_t0: 63, 1024, 0,
    aligned_append_l63_w1024_t1: 63, 1024, 1,
);

fn push_bit_string_bench_aligned(
    bencher: Bencher,
    lhs_len: usize,
    rhs_words: usize,
    rhs_tail_bits: usize,
) {
    let rhs_len = rhs_words * 64 + rhs_tail_bits;
    let rhs = push_bit_string_make_bit_string(rhs_len);

    bencher
        .with_inputs(|| push_bit_string_make_bit_string(lhs_len))
        .bench_refs(|lhs| {
            lhs.push_bit_string(black_box(&rhs));
            black_box(&*lhs);
        });
}

fn push_bit_string_bench_bit_string(bencher: Bencher, len: usize) {
    let input = push_bit_string_make_bit_string(len);
    let rhs = push_bit_string_make_bit_string(push_bit_string_chunk_len(len));

    bencher.bench(|| {
        let mut bits = black_box(input.clone());
        bits.push_bit_string(black_box(&rhs));
        black_box(bits)
    });
}

fn push_bit_string_bench_string(bencher: Bencher, len: usize) {
    let input = push_bit_string_make_string(len);
    let rhs = push_bit_string_make_string(push_bit_string_chunk_len(len));

    bencher.bench(|| {
        let mut string = black_box(input.clone());
        string.push_str(black_box(&rhs));
        black_box(string)
    });
}

#[inline]
fn push_bit_string_chunk_len(len: usize) -> usize {
    (len / 8).max(1)
}

#[inline]
fn push_bit_string_make_bit_string(len: usize) -> BitString {
    (0..len).map(|index| push_bit_string_bit_at(index)).collect()
}

fn push_bit_string_make_string(len: usize) -> String {
    let mut out = String::with_capacity(len);

    for index in 0..len {
        out.push(push_bit_string_bit_char(push_bit_string_bit_at(index)));
    }

    out
}

#[inline]
fn push_bit_string_bit_char(value: bool) -> char {
    if value { '1' } else { '0' }
}

#[inline]
fn push_bit_string_bit_at(index: usize) -> bool {
    push_bit_string_mix64(index as u64) & 1 != 0
}

#[inline]
fn push_bit_string_mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
