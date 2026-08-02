use std::collections::BTreeMap;

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Data, DeriveInput, Error, Expr, Fields, Lit, LitInt, Result};

pub(super) fn expand(input: DeriveInput) -> Result<TokenStream2> {
    require_repr_u8(&input)?;
    let (bits, bits_span) = parse_bits(&input)?;
    if bits > 8 {
        return Err(Error::new(bits_span, "packed bit width must not exceed 8"));
    }

    let Data::Enum(data) = &input.data else {
        return Err(Error::new_spanned(
            &input.ident,
            "PackedChar can only be derived for enums",
        ));
    };

    let limit = 1u16 << bits;
    let mut codes = BTreeMap::new();
    let mut variants = Vec::with_capacity(data.variants.len());

    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(Error::new_spanned(
                &variant.fields,
                "PackedChar variants must not contain fields",
            ));
        }

        let Some((_, Expr::Lit(expression))) = &variant.discriminant else {
            return Err(Error::new_spanned(
                &variant.ident,
                "PackedChar variants require an explicit integer discriminant",
            ));
        };
        let Lit::Int(discriminant) = &expression.lit else {
            return Err(Error::new_spanned(
                &expression.lit,
                "PackedChar discriminants must be integer literals",
            ));
        };
        let code = discriminant.base10_parse::<u16>()?;
        if code >= limit {
            return Err(Error::new_spanned(
                discriminant,
                format!("discriminant {code} does not fit in {bits} bits"),
            ));
        }
        if let Some(previous) = codes.insert(code, discriminant.span()) {
            let mut error = Error::new_spanned(discriminant, "duplicate packed discriminant");
            error.combine(Error::new(previous, "first used here"));
            return Err(error);
        }

        variants.push((&variant.ident, code));
    }

    let crate_path = bit_string_path();
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let bits = LitInt::new(&bits.to_string(), bits_span);
    let variant_names = variants.iter().map(|(variant, _)| variant);
    let discriminants = variants
        .iter()
        .map(|(_, code)| LitInt::new(&code.to_string(), Span::call_site()));

    Ok(quote! {
        impl #impl_generics #crate_path::traits::PackedChar<#bits>
            for #name #type_generics #where_clause
        {
            #[inline]
            fn code(self) -> u8 {
                self as u8
            }

            #[inline]
            fn from_code(code: u8) -> ::core::option::Option<Self> {
                match code {
                    #(#discriminants => ::core::option::Option::Some(Self::#variant_names),)*
                    _ => ::core::option::Option::None,
                }
            }
        }
    })
}

fn require_repr_u8(input: &DeriveInput) -> Result<()> {
    let mut found = false;
    for attribute in &input.attrs {
        if attribute.path().is_ident("repr") {
            attribute.parse_nested_meta(|meta| {
                found |= meta.path.is_ident("u8");
                Ok(())
            })?;
        }
    }
    if found {
        Ok(())
    } else {
        Err(Error::new_spanned(
            &input.ident,
            "PackedChar requires #[repr(u8)]",
        ))
    }
}

fn parse_bits(input: &DeriveInput) -> Result<(u8, Span)> {
    let mut bits = None;
    for attribute in &input.attrs {
        if !attribute.path().is_ident("packed") {
            continue;
        }
        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("bits") {
                return Err(meta.error("expected `bits = N`"));
            }
            if bits.is_some() {
                return Err(meta.error("duplicate packed bit width"));
            }
            let literal: LitInt = meta.value()?.parse()?;
            bits = Some((literal.base10_parse::<u8>()?, literal.span()));
            Ok(())
        })?;
    }
    bits.ok_or_else(|| Error::new_spanned(&input.ident, "PackedChar requires #[packed(bits = N)]"))
}

fn bit_string_path() -> TokenStream2 {
    match crate_name("bit-string") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let name = syn::Ident::new(&name, Span::call_site());
            quote!(::#name)
        }
        Err(_) => quote!(::bit_string),
    }
}

#[cfg(test)]
mod tests_for_expand;
