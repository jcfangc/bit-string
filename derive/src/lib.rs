use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, parse_macro_input, parse_quote};

mod funcs_for_expand;

use funcs_for_expand::expand;

/// Marks a fieldless enum as a packed alphabet and supplies its implementation.
///
/// This is the preferred spelling for new code:
/// `#[packed(bits = 2)] enum Symbol { ... }`.
#[proc_macro_attribute]
pub fn packed(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as DeriveInput);
    let bits: proc_macro2::TokenStream = args.into();

    let packed: Attribute = parse_quote!(#[packed(#bits)]);
    item.attrs.push(packed);
    if !item
        .attrs
        .iter()
        .any(|attribute| attribute.path().is_ident("repr"))
    {
        item.attrs.push(parse_quote!(#[repr(u8)]));
    }

    let implementation = match expand(item.clone()) {
        Ok(tokens) => tokens,
        Err(error) => error.into_compile_error(),
    };
    item.attrs
        .retain(|attribute| !attribute.path().is_ident("packed"));
    quote!(#item #implementation).into()
}
