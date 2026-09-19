/// Fixed-width code packing operations on `[u64]` backing storage.
///
/// The destination is expected to contain exactly the words produced by the
/// complete layout blocks in `codes`. Partial blocks and bit offsets are
/// handled by the packed-domain caller instead.
pub(crate) trait WordsPack {
    fn pack_codes<const BITS: u8>(&mut self, codes: &[u8]);
}

pub(crate) mod funcs_for_pack_codes_core;
mod impls_for_u64_slice;

pub(crate) use funcs_for_pack_codes_core::layout_block_len;
