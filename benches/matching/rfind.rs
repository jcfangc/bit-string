struct RfindNeedleCase {
    haystack_bits: BitString,
    needle_bits: BitString,
    haystack_string: String,
    needle_string: String,
}

// -- rfind/len_65 -----------------------------------------------------------

#[divan::bench(name = "rfind/len_65/front/ours_string_str")]
fn rfind_65_front_str(b: Bencher) {
    rfind_b_str(b, rfind_make_case(65, 0));
}
#[divan::bench(name = "rfind/len_65/front/ours_string_string")]
fn rfind_65_front_string(b: Bencher) {
    rfind_b_string(b, rfind_make_case(65, 0));
}
#[divan::bench(name = "rfind/len_65/front/string")]
fn rfind_65_front_native(b: Bencher) {
    rfind_b_native(b, rfind_make_case(65, 0));
}

#[divan::bench(name = "rfind/len_65/middle/ours_string_str")]
fn rfind_65_middle_str(b: Bencher) {
    rfind_b_str(b, rfind_middle_case(65));
}
#[divan::bench(name = "rfind/len_65/middle/ours_string_string")]
fn rfind_65_middle_string(b: Bencher) {
    rfind_b_string(b, rfind_middle_case(65));
}
#[divan::bench(name = "rfind/len_65/middle/string")]
fn rfind_65_middle_native(b: Bencher) {
    rfind_b_native(b, rfind_middle_case(65));
}

#[divan::bench(name = "rfind/len_65/end/ours_string_str")]
fn rfind_65_end_str(b: Bencher) {
    rfind_b_str(b, rfind_end_case(65));
}
#[divan::bench(name = "rfind/len_65/end/ours_string_string")]
fn rfind_65_end_string(b: Bencher) {
    rfind_b_string(b, rfind_end_case(65));
}
#[divan::bench(name = "rfind/len_65/end/string")]
fn rfind_65_end_native(b: Bencher) {
    rfind_b_native(b, rfind_end_case(65));
}

#[divan::bench(name = "rfind/len_65/miss/ours_string_str")]
fn rfind_65_miss_str(b: Bencher) {
    rfind_b_str(b, rfind_miss_case(65));
}
#[divan::bench(name = "rfind/len_65/miss/ours_string_string")]
fn rfind_65_miss_string(b: Bencher) {
    rfind_b_string(b, rfind_miss_case(65));
}
#[divan::bench(name = "rfind/len_65/miss/string")]
fn rfind_65_miss_native(b: Bencher) {
    rfind_b_native(b, rfind_miss_case(65));
}

// -- rfind/len_65536 --------------------------------------------------------

#[divan::bench(name = "rfind/len_65536/front/ours_string_str")]
fn rfind_65536_front_str(b: Bencher) {
    rfind_b_str(b, rfind_make_case(65536, 0));
}
#[divan::bench(name = "rfind/len_65536/front/ours_string_string")]
fn rfind_65536_front_string(b: Bencher) {
    rfind_b_string(b, rfind_make_case(65536, 0));
}
#[divan::bench(name = "rfind/len_65536/front/string")]
fn rfind_65536_front_native(b: Bencher) {
    rfind_b_native(b, rfind_make_case(65536, 0));
}

#[divan::bench(name = "rfind/len_65536/middle/ours_string_str")]
fn rfind_65536_middle_str(b: Bencher) {
    rfind_b_str(b, rfind_middle_case(65536));
}
#[divan::bench(name = "rfind/len_65536/middle/ours_string_string")]
fn rfind_65536_middle_string(b: Bencher) {
    rfind_b_string(b, rfind_middle_case(65536));
}
#[divan::bench(name = "rfind/len_65536/middle/string")]
fn rfind_65536_middle_native(b: Bencher) {
    rfind_b_native(b, rfind_middle_case(65536));
}

#[divan::bench(name = "rfind/len_65536/end/ours_string_str")]
fn rfind_65536_end_str(b: Bencher) {
    rfind_b_str(b, rfind_end_case(65536));
}
#[divan::bench(name = "rfind/len_65536/end/ours_string_string")]
fn rfind_65536_end_string(b: Bencher) {
    rfind_b_string(b, rfind_end_case(65536));
}
#[divan::bench(name = "rfind/len_65536/end/string")]
fn rfind_65536_end_native(b: Bencher) {
    rfind_b_native(b, rfind_end_case(65536));
}

#[divan::bench(name = "rfind/len_65536/miss/ours_string_str")]
fn rfind_65536_miss_str(b: Bencher) {
    rfind_b_str(b, rfind_miss_case(65536));
}
#[divan::bench(name = "rfind/len_65536/miss/ours_string_string")]
fn rfind_65536_miss_string(b: Bencher) {
    rfind_b_string(b, rfind_miss_case(65536));
}
#[divan::bench(name = "rfind/len_65536/miss/string")]
fn rfind_65536_miss_native(b: Bencher) {
    rfind_b_native(b, rfind_miss_case(65536));
}

// -- helpers ----------------------------------------------------------------

fn rfind_b_str(b: Bencher, c: RfindNeedleCase) {
    b.bench(|| black_box(&c.haystack_bits).rfind_str(black_box(c.needle_bits.as_bit_str())));
}
fn rfind_b_string(b: Bencher, c: RfindNeedleCase) {
    b.bench(|| black_box(&c.haystack_bits).rfind_string(black_box(&c.needle_bits)));
}
fn rfind_b_native(b: Bencher, c: RfindNeedleCase) {
    b.bench(|| black_box(&c.haystack_string).rfind(black_box(&c.needle_string)));
}

fn rfind_middle_case(len: usize) -> RfindNeedleCase {
    let n = rfind_chunk_len(len);
    rfind_make_case(len, (len - n) / 2)
}
fn rfind_end_case(len: usize) -> RfindNeedleCase {
    let n = rfind_chunk_len(len);
    rfind_make_case(len, len - n)
}
fn rfind_miss_case(len: usize) -> RfindNeedleCase {
    let haystack = vec![false; len];
    let needle = vec![true; rfind_chunk_len(len)];
    RfindNeedleCase {
        haystack_bits: haystack.iter().copied().collect(),
        needle_bits: needle.iter().copied().collect(),
        haystack_string: rfind_bools_to_string(&haystack),
        needle_string: rfind_bools_to_string(&needle),
    }
}
fn rfind_make_case(len: usize, position: usize) -> RfindNeedleCase {
    let needle = rfind_make_needle_bits(rfind_chunk_len(len));
    let mut haystack = vec![false; len];
    for (offset, &value) in needle.iter().enumerate() {
        haystack[position + offset] = value;
    }
    RfindNeedleCase {
        haystack_bits: haystack.iter().copied().collect(),
        needle_bits: needle.iter().copied().collect(),
        haystack_string: rfind_bools_to_string(&haystack),
        needle_string: rfind_bools_to_string(&needle),
    }
}
fn rfind_chunk_len(len: usize) -> usize {
    (len / 8).max(1)
}
fn rfind_make_needle_bits(len: usize) -> Vec<bool> {
    let mut out = Vec::with_capacity(len);
    for index in 0..len {
        out.push(rfind_mix64((index as u64).wrapping_add(0x9e37_79b9_7f4a_7c15)) & 7 <= 2);
    }
    out[0] = true;
    out[len - 1] = true;
    out
}
fn rfind_bools_to_string(values: &[bool]) -> String {
    let mut out = String::with_capacity(values.len());
    for &value in values {
        out.push(if value { '1' } else { '0' });
    }
    out
}
fn rfind_mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
