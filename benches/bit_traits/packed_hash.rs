fn packed_hash<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = packed::<BITS>(&codes(BITS, len));
    bencher.bench(|| {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        black_box(hasher.finish())
    });
}

mod hash_cases {
    use super::*;

    macro_rules! define_case {
        ($case:ident, $bits:literal, $len:literal) => {
            mod $case {
                use super::*;

                #[divan::bench(name = concat!("packed_hash/hash/", stringify!($case), "/ours_packed_string"))]
                fn ours_packed_string(bencher: Bencher) {
                    super::super::packed_hash::<$bits>(bencher, $len);
                }
            }
        };
    }

    crate::for_each_packed_trait_bench_case!(define_case);
}
