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

pub(crate) use funcs_for_unpack_codes_core::prefer_bulk_unpack;

#[cfg(test)]
mod tests_for_words_unpack;

#[cfg(test)]
mod tests_for_bulk_preference {
    use super::prefer_bulk_unpack;

    #[test]
    fn preference_matches_available_avx2_widths() {
        let has_avx2 = cfg!(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        ));

        for (bits, preferred) in [
            (1, prefer_bulk_unpack::<1>()),
            (2, prefer_bulk_unpack::<2>()),
            (3, prefer_bulk_unpack::<3>()),
            (4, prefer_bulk_unpack::<4>()),
            (5, prefer_bulk_unpack::<5>()),
            (6, prefer_bulk_unpack::<6>()),
            (7, prefer_bulk_unpack::<7>()),
            (8, prefer_bulk_unpack::<8>()),
        ] {
            let has_kernel = matches!(bits, 1 | 2 | 4 | 6 | 8);
            assert_eq!(preferred, has_avx2 && has_kernel, "BITS={bits}");
        }
    }
}
