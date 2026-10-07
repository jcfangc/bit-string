/// Fixed-width code unpacking operations on `[u64]` backing storage.
///
/// The input must contain complete layout blocks and the output length must
/// equal the number of codes represented by those words. Every output code is
/// fully overwritten and contains only its low `BITS` bits.
pub(crate) trait WordsUnpack {
    fn unpack_codes<const BITS: u8>(&self, codes: &mut [u8]);
}

pub(crate) mod funcs_for_unpack_codes_core;
mod impls_for_u64_slice;

pub(crate) use funcs_for_unpack_codes_core::has_accelerated_bulk_unpack;

#[cfg(test)]
mod tests_for_words_unpack;

#[cfg(test)]
mod tests_for_bulk_backend_availability {
    use super::has_accelerated_bulk_unpack;

    #[test]
    fn availability_matches_compiled_avx2_widths() {
        let has_avx2 = cfg!(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        ));

        for (bits, has_backend) in [
            (1, has_accelerated_bulk_unpack::<1>()),
            (2, has_accelerated_bulk_unpack::<2>()),
            (3, has_accelerated_bulk_unpack::<3>()),
            (4, has_accelerated_bulk_unpack::<4>()),
            (5, has_accelerated_bulk_unpack::<5>()),
            (6, has_accelerated_bulk_unpack::<6>()),
            (7, has_accelerated_bulk_unpack::<7>()),
            (8, has_accelerated_bulk_unpack::<8>()),
        ] {
            let has_kernel = matches!(bits, 1 | 2 | 4 | 6 | 8);
            assert_eq!(has_backend, has_avx2 && has_kernel, "BITS={bits}");
        }
    }
}
