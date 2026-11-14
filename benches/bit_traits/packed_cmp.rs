fn packed_cmp_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let left_codes = codes(BITS, len);
    let mut right_codes = left_codes.clone();
    right_codes[cross_word_index::<BITS>(len)] ^= 1;
    let left = packed::<BITS>(&left_codes);
    let right = packed::<BITS>(&right_codes);
    bencher.bench(|| black_box(left.cmp(&right)));
}

fn packed_cmp_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let left_codes = codes(BITS, len);
    let mut right_codes = left_codes.clone();
    right_codes[cross_word_index_for_offset(usize::from(BITS), len, usize::from(BITS))] ^= 1;
    let left_owner = packed_cmp_offset::<BITS>(&left_codes);
    let right_owner = packed_cmp_offset::<BITS>(&right_codes);
    let left = left_owner.as_packed_str().slice_from(1);
    let right = right_owner.as_packed_str().slice_from(1);
    bencher.bench(|| black_box(left.cmp(&right)));
}

fn packed_cmp_offset<const BITS: u8>(input: &[u8]) -> PackedString<support::Code, BITS> {
    let mut source = Vec::with_capacity(input.len() + 1);
    source.push(0);
    source.extend_from_slice(input);
    packed::<BITS>(&source)
}

mod cmp_cases {
    use super::*;

    macro_rules! define_case {
        ($case:ident, $bits:literal, $len:literal) => {
            mod $case {
                use super::*;

                #[divan::bench(name = concat!("packed_ordering/cmp_difference/word_boundary_or_fallback/", stringify!($case), "/ours_packed_string_aligned"))]
                fn ours_packed_string(bencher: Bencher) {
                    super::super::packed_cmp_string::<$bits>(bencher, $len);
                }

                #[divan::bench(name = concat!("packed_ordering/cmp_difference/word_boundary_or_fallback/", stringify!($case), "/ours_packed_str_unaligned"))]
                fn ours_packed_str(bencher: Bencher) {
                    super::super::packed_cmp_str::<$bits>(bencher, $len);
                }
            }
        };
    }

    crate::for_each_packed_trait_bench_case!(define_case);
}
