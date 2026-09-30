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

#[cfg(test)]
mod tests_for_words_unpack;
