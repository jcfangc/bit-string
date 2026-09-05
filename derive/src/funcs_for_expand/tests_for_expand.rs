use super::*;
use quote::quote;
use syn::parse2;

fn error(input: TokenStream2) -> String {
    expand(parse2(input).unwrap()).unwrap_err().to_string()
}

#[test]
fn accepts_strict_enum_encoding() {
    let result = expand(
        parse2(quote! {
            #[repr(u8)]
            #[packed(bits = 2)]
            enum Letter { A = 0, B = 1, C = 2, D = 3 }
        })
        .unwrap(),
    );
    assert!(result.is_ok());
}

#[test]
fn requires_explicit_discriminants() {
    assert!(
        error(quote! {
            #[repr(u8)]
            #[packed(bits = 1)]
            enum Letter { A = 0, B }
        })
        .contains("explicit integer discriminant")
    );
}

#[test]
fn rejects_codes_outside_the_width() {
    assert!(
        error(quote! {
            #[repr(u8)]
            #[packed(bits = 2)]
            enum Letter { A = 4 }
        })
        .contains("does not fit in 2 bits")
    );
}

#[test]
fn rejects_widths_above_u8_capacity() {
    assert!(
        error(quote! {
            #[repr(u8)]
            #[packed(bits = 9)]
            enum Letter { A = 0 }
        })
        .contains("must be between 1 and 8")
    );
}

#[test]
fn rejects_zero_width() {
    assert!(
        error(quote! {
            #[repr(u8)]
            #[packed(bits = 0)]
            enum Letter { A = 0 }
        })
        .contains("must be between 1 and 8")
    );
}

#[test]
fn rejects_variants_with_fields() {
    assert!(
        error(quote! {
            #[repr(u8)]
            #[packed(bits = 1)]
            enum Letter { A(u8) = 0 }
        })
        .contains("must not contain fields")
    );
}

#[test]
fn rejects_duplicate_codes() {
    assert!(
        error(quote! {
            #[repr(u8)]
            #[packed(bits = 1)]
            enum Letter { A = 0, B = 0 }
        })
        .contains("duplicate packed discriminant")
    );
}

#[test]
fn rejects_non_literal_discriminants() {
    assert!(
        error(quote! {
            #[repr(u8)]
            #[packed(bits = 2)]
            enum Letter { A = 1 + 1 }
        })
        .contains("explicit integer discriminant")
    );
}

#[test]
fn requires_repr_u8() {
    assert!(
        error(quote! {
            #[packed(bits = 1)]
            enum Letter { A = 0 }
        })
        .contains("requires #[repr(u8)]")
    );
}
