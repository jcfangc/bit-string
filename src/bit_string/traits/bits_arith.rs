/// SIMD-accelerated word-level operations on `[u64]` backing storage.
pub(crate) trait BitsArith {
    // TODO: and, or, xor, not, shl, shr — assign and owned variants
}

pub(crate) mod impls_for_u64_slice;
