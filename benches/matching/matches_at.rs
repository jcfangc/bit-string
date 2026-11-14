struct MatchesAtCase {
    haystack_bits: BitString,
    pattern_bits: BitString,
    haystack_string: String,
    pattern_string: String,
    index: usize,
}

fn matches_at_make_bits(len: usize) -> BitString {
    let mut bits = BitString::zeros(len);
    for i in 0..len {
        if (i as u64 * 17 + 3) % 7 == 0 {
            bits.set(i, true);
        }
    }
    bits
}

fn matches_at_iv(start: usize, len: usize) -> UsizeCO {
    UsizeCO::checked_from_start_len(start, len).unwrap()
}

fn matches_at_make_case(len: usize, pat_len: usize, index: usize) -> MatchesAtCase {
    let h = matches_at_make_bits(len);
    let p = h.slice(matches_at_iv(index, pat_len));
    MatchesAtCase {
        haystack_string: h.to_string(),
        pattern_string: p.to_string(),
        haystack_bits: h,
        pattern_bits: p,
        index,
    }
}

fn matches_at_no_case(len: usize, pat_len: usize, index: usize) -> MatchesAtCase {
    let h = matches_at_make_bits(len);
    let mut p = h.slice(matches_at_iv(index, pat_len));
    p.set(0, !p.get(0).unwrap());
    MatchesAtCase {
        haystack_string: h.to_string(),
        pattern_string: p.to_string(),
        haystack_bits: h,
        pattern_bits: p,
        index,
    }
}

// ---------------------------------------------------------------------------
// 65-bit haystack (small — scalar path)
// ---------------------------------------------------------------------------

#[divan::bench(name = "matches_at_str/len_65/yes/aligned/ours_string_str")]
fn m65ya(b: Bencher) {
    matches_at_b_bit(b, matches_at_make_case(65, 4, 64));
}
#[divan::bench(name = "matches_at_str/len_65/yes/aligned/string")]
fn m65yas(b: Bencher) {
    matches_at_b_str(b, matches_at_make_case(65, 4, 64));
}
#[divan::bench(name = "matches_at_str/len_65/yes/unaligned/ours_string_str")]
fn m65yu(b: Bencher) {
    matches_at_b_bit(b, matches_at_make_case(65, 4, 3));
}
#[divan::bench(name = "matches_at_str/len_65/yes/unaligned/string")]
fn m65yus(b: Bencher) {
    matches_at_b_str(b, matches_at_make_case(65, 4, 3));
}
#[divan::bench(name = "matches_at_str/len_65/no/aligned/ours_string_str")]
fn m65na(b: Bencher) {
    matches_at_b_bit(b, matches_at_no_case(65, 4, 64));
}
#[divan::bench(name = "matches_at_str/len_65/no/aligned/string")]
fn m65nas(b: Bencher) {
    matches_at_b_str(b, matches_at_no_case(65, 4, 64));
}
#[divan::bench(name = "matches_at_str/len_65/no/unaligned/ours_string_str")]
fn m65nu(b: Bencher) {
    matches_at_b_bit(b, matches_at_no_case(65, 4, 3));
}
#[divan::bench(name = "matches_at_str/len_65/no/unaligned/string")]
fn m65nus(b: Bencher) {
    matches_at_b_str(b, matches_at_no_case(65, 4, 3));
}

// ---------------------------------------------------------------------------
// 65536-bit haystack (large — SIMD path)
// ---------------------------------------------------------------------------

#[divan::bench(name = "matches_at_str/len_65536/yes/aligned/ours_string_str")]
fn m6ya(b: Bencher) {
    matches_at_b_bit(b, matches_at_make_case(65_536, 128, 64));
}
#[divan::bench(name = "matches_at_str/len_65536/yes/aligned/string")]
fn m6yas(b: Bencher) {
    matches_at_b_str(b, matches_at_make_case(65_536, 128, 64));
}
#[divan::bench(name = "matches_at_str/len_65536/yes/unaligned/ours_string_str")]
fn m6yu(b: Bencher) {
    matches_at_b_bit(b, matches_at_make_case(65_536, 128, 3));
}
#[divan::bench(name = "matches_at_str/len_65536/yes/unaligned/string")]
fn m6yus(b: Bencher) {
    matches_at_b_str(b, matches_at_make_case(65_536, 128, 3));
}
#[divan::bench(name = "matches_at_str/len_65536/no/aligned/ours_string_str")]
fn m6na(b: Bencher) {
    matches_at_b_bit(b, matches_at_no_case(65_536, 128, 64));
}
#[divan::bench(name = "matches_at_str/len_65536/no/aligned/string")]
fn m6nas(b: Bencher) {
    matches_at_b_str(b, matches_at_no_case(65_536, 128, 64));
}
#[divan::bench(name = "matches_at_str/len_65536/no/unaligned/ours_string_str")]
fn m6nu(b: Bencher) {
    matches_at_b_bit(b, matches_at_no_case(65_536, 128, 3));
}
#[divan::bench(name = "matches_at_str/len_65536/no/unaligned/string")]
fn m6nus(b: Bencher) {
    matches_at_b_str(b, matches_at_no_case(65_536, 128, 3));
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn matches_at_b_bit(b: Bencher, c: MatchesAtCase) {
    b.bench(|| {
        black_box(&c.haystack_bits).matches_at_str(c.index, black_box(c.pattern_bits.as_bit_str()))
    });
}

fn matches_at_b_str(b: Bencher, c: MatchesAtCase) {
    b.bench(|| {
        let h = black_box(&c.haystack_string);
        let p = black_box(&c.pattern_string);
        h.as_bytes()[c.index..].starts_with(p.as_bytes())
    });
}
