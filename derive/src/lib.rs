use proc_macro::TokenStream;
use syn::{DeriveInput, Error, parse_macro_input};

mod funcs_for_expand;

use funcs_for_expand::expand;

/// Derives a fixed-width packed encoding from explicit `#[repr(u8)]` enum
/// discriminants.
///
/// The enum must use `#[packed(bits = N)]`, contain only fieldless variants,
/// and give every variant an integer-literal discriminant that fits in `N`
/// bits.
#[proc_macro_derive(PackedChar, attributes(packed))]
pub fn derive_packed_char(input: TokenStream) -> TokenStream {
    expand(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
