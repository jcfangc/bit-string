use super::*;
use proc_macro2::{Delimiter, Spacing, TokenTree};
use quote::quote;
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use syn::parse2;

fn error(input: TokenStream2) -> String {
    expand(parse2(input).unwrap()).unwrap_err().to_string()
}

fn repr_error(input: TokenStream2) -> String {
    require_repr_u8(&parse2(input).unwrap())
        .unwrap_err()
        .to_string()
}

fn parsed_bits(input: TokenStream2) -> u8 {
    parse_bits(&parse2(input).unwrap()).unwrap().0
}

fn bits_error(input: TokenStream2) -> String {
    parse_bits(&parse2(input).unwrap()).unwrap_err().to_string()
}

#[derive(Debug, PartialEq, Eq)]
enum TokenShape {
    Group(u8, Vec<TokenShape>),
    Ident(String),
    Literal(String),
    Punct(char, bool),
}

fn token_shapes(tokens: TokenStream2) -> Vec<TokenShape> {
    tokens
        .into_iter()
        .map(|token| match token {
            TokenTree::Group(group) => TokenShape::Group(
                match group.delimiter() {
                    Delimiter::Parenthesis => 0,
                    Delimiter::Brace => 1,
                    Delimiter::Bracket => 2,
                    Delimiter::None => 3,
                },
                token_shapes(group.stream()),
            ),
            TokenTree::Ident(ident) => TokenShape::Ident(ident.to_string()),
            TokenTree::Literal(literal) => TokenShape::Literal(literal.to_string()),
            TokenTree::Punct(punct) => {
                TokenShape::Punct(punct.as_char(), punct.spacing() == Spacing::Joint)
            }
        })
        .collect()
}

struct FixtureGuard(std::path::PathBuf);

impl Drop for FixtureGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn parses_bits_literals_and_defers_range_validation() {
    for (input, expected) in [
        (quote! { #[packed(bits = 0)] enum Letter {} }, 0),
        (quote! { #[packed(bits = 1)] enum Letter {} }, 1),
        (quote! { #[packed(bits = 8)] enum Letter {} }, 8),
        (quote! { #[packed(bits = 9)] enum Letter {} }, 9),
        (quote! { #[packed(bits = 0xff)] enum Letter {} }, 255),
        (quote! { #[packed(bits = 0b1)] enum Letter {} }, 1),
        (quote! { #[packed(bits = 0o10)] enum Letter {} }, 8),
        (quote! { #[packed(bits = 8u8)] enum Letter {} }, 8),
    ] {
        assert_eq!(parsed_bits(input), expected);
    }
}

#[test]
fn rejects_missing_unknown_duplicate_and_invalid_bits_metadata() {
    for input in [
        quote! { enum Letter {} },
        quote! { #[derive(Clone)] enum Letter {} },
        quote! { #[packed()] enum Letter {} },
    ] {
        assert!(bits_error(input).contains("packed requires #[packed(bits = N)]"));
    }

    for input in [
        quote! { #[packed(width = 2)] enum Letter {} },
        quote! { #[packed(bits = 2, width = 3)] enum Letter {} },
    ] {
        assert!(bits_error(input).contains("expected `bits = N`"));
    }

    for input in [
        quote! { #[packed(bits = 1, bits = 2)] enum Letter {} },
        quote! { #[packed(bits = 1)] #[packed(bits = 2)] enum Letter {} },
    ] {
        assert!(bits_error(input).contains("duplicate packed bit width"));
    }

    for input in [
        quote! { #[packed(bits = "2")] enum Letter {} },
        quote! { #[packed(bits = true)] enum Letter {} },
        quote! { #[packed(bits = 1 + 1)] enum Letter {} },
        quote! { #[packed(bits = -1)] enum Letter {} },
        quote! { #[packed(bits = 256)] enum Letter {} },
        quote! { #[packed(bits)] enum Letter {} },
    ] {
        assert!(parse_bits(&parse2(input).unwrap()).is_err());
    }
}

#[test]
fn returns_span_of_bits_literal() {
    let input = syn::parse_str::<syn::DeriveInput>("#[packed(bits = 17)]\nenum Letter {}").unwrap();
    let (_, span) = parse_bits(&input).unwrap();

    let mut expected = None;
    input.attrs[0]
        .parse_nested_meta(|meta| {
            if meta.path.is_ident("bits") {
                expected = Some(meta.value()?.parse::<syn::LitInt>()?.span());
            }
            Ok(())
        })
        .unwrap();
    let expected = expected.unwrap();

    assert_eq!(span.start(), expected.start());
    assert_eq!(span.end(), expected.end());
}

#[test]
fn accepts_top_level_u8_in_repr_metadata() {
    for input in [
        quote! {
            #[repr(u8)]
            enum Letter { A = 0 }
        },
        quote! {
            #[repr(C, u8)]
            enum Letter { A = 0 }
        },
        quote! {
            #[repr(u8, C)]
            enum Letter { A = 0 }
        },
        quote! {
            #[repr(C)]
            #[repr(u8)]
            enum Letter { A = 0 }
        },
    ] {
        assert!(require_repr_u8(&parse2(input).unwrap()).is_ok());
    }
}

#[test]
fn falls_back_to_conventional_bit_string_path_when_unresolved() {
    assert_eq!(
        token_shapes(bit_string_path()),
        token_shapes(quote!(::bit_string))
    );
}

#[test]
fn resolves_renamed_bit_string_dependency_in_isolated_fixture() {
    let fixture = std::env::temp_dir().join(format!(
        "bit-string-derive-renamed-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _cleanup = FixtureGuard(fixture.clone());
    fs::create_dir_all(fixture.join("src")).unwrap();

    let bit_string = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .unwrap();
    fs::write(
        fixture.join("Cargo.toml"),
        format!(
            "[package]\nname = \"renamed-bit-string-fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[workspace]\n\n[dependencies]\nbit_string_alias = {{ package = \"bit-string\", path = {:?} }}\n",
            bit_string,
        ),
    )
    .unwrap();
    fs::write(
        fixture.join("src/main.rs"),
        r#"
use bit_string_alias::{packed, traits::PackedChar};

#[packed(bits = 2)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Symbol {
    Zero = 0,
    Two = 2,
}

fn main() {
    assert_eq!(Symbol::Zero.code(), 0);
    assert_eq!(Symbol::Two.code(), 2);
    assert_eq!(Symbol::from_code(0), Some(Symbol::Zero));
    assert_eq!(Symbol::from_code(1), None);
    assert_eq!(Symbol::from_code(2), Some(Symbol::Two));
}
"#,
    )
    .unwrap();

    let lockfile = Command::new("cargo")
        .args(["generate-lockfile", "--offline", "--manifest-path"])
        .arg(fixture.join("Cargo.toml"))
        .output()
        .unwrap();
    assert!(
        lockfile.status.success(),
        "failed to generate fixture lockfile:\n{}",
        String::from_utf8_lossy(&lockfile.stderr)
    );

    let run = Command::new("cargo")
        .args(["run", "--locked", "--offline", "--manifest-path"])
        .arg(fixture.join("Cargo.toml"))
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "renamed dependency fixture failed:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn rejects_repr_without_top_level_u8() {
    for input in [
        quote! { enum Letter { A = 0 } },
        quote! {
            #[repr(C)]
            enum Letter { A = 0 }
        },
        quote! {
            #[repr(usize)]
            enum Letter { A = 0 }
        },
        quote! {
            #[repr(transparent)]
            enum Letter { A = 0 }
        },
        quote! {
            #[repr(align(8))]
            enum Letter { A = 0 }
        },
    ] {
        assert!(require_repr_u8(&parse2(input).unwrap()).is_err());
    }

    for input in [
        quote! {
            #[repr(align(8))]
            enum Letter { A = 0 }
        },
        quote! {
            #[repr(C, align(8))]
            enum Letter { A = 0 }
        },
        quote! {
            #[repr(align(u8))]
            enum Letter { A = 0 }
        },
    ] {
        assert_eq!(repr_error(input), "packed requires #[repr(u8)]");
    }
}

#[test]
fn propagates_malformed_repr_metadata() {
    let input = parse2(quote! {
        #[repr(C,, u8)]
        enum Letter { A = 0 }
    })
    .unwrap();
    let error = require_repr_u8(&input).unwrap_err().to_string();

    assert_ne!(error, "packed requires #[repr(u8)]");
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
fn accepts_width_boundaries_and_rejects_first_out_of_range_code() {
    assert!(
        expand(
            parse2(quote! {
                #[repr(u8)]
                #[packed(bits = 1)]
                enum Binary { Zero = 0, One = 1 }
            })
            .unwrap(),
        )
        .is_ok()
    );

    assert!(
        expand(
            parse2(quote! {
                #[repr(u8)]
                #[packed(bits = 8)]
                enum Byte { Zero = 0, Maximum = 255 }
            })
            .unwrap(),
        )
        .is_ok()
    );

    assert!(
        error(quote! {
            #[repr(u8)]
            #[packed(bits = 8)]
            enum Byte { TooLarge = 256 }
        })
        .contains("does not fit in 8 bits")
    );
}

#[test]
fn emits_code_wise_decoding_for_sparse_alphabet() {
    let output = expand(
        parse2(quote! {
            #[repr(u8)]
            #[packed(bits = 3)]
            enum Sparse { High = 7, Low = 0, Middle = 3 }
        })
        .unwrap(),
    )
    .unwrap()
    .to_string();

    assert!(output.contains("PackedChar < 3 > for Sparse"));
    for (code, variant) in [(0, "Low"), (3, "Middle"), (7, "High")] {
        let code = syn::LitInt::new(&code.to_string(), proc_macro2::Span::call_site());
        let variant = syn::Ident::new(variant, proc_macro2::Span::call_site());
        let expected = quote!(
            #code => ::core::option::Option::Some(Self::#variant)
        )
        .to_string();
        assert!(
            output.contains(&expected),
            "missing `{expected}` in `{output}`"
        );
    }
    assert!(output.contains(&quote!(_ => ::core::option::Option::None).to_string()));
}

#[test]
fn emits_empty_enum_decoding_fallback() {
    let output = expand(
        parse2(quote! {
            #[repr(u8)]
            #[packed(bits = 2)]
            enum Empty {}
        })
        .unwrap(),
    )
    .unwrap()
    .to_string();

    assert!(output.contains("PackedChar < 2 > for Empty"));
    assert!(output.contains(&quote!(_ => ::core::option::Option::None).to_string()));
    assert!(!output.contains("Option :: Some"));
}

#[test]
fn preserves_generics_and_where_clause() {
    let output = expand(
        parse2(quote! {
            #[repr(u8)]
            #[packed(bits = 1)]
            enum Generic<T> where T: Copy { Zero = 0, One = 1 }
        })
        .unwrap(),
    )
    .unwrap()
    .to_string();

    assert!(output.contains("impl < T >"));
    assert!(output.contains("for Generic < T > where T : Copy"));
}

#[test]
fn rejects_non_enum_inputs() {
    for input in [
        quote! {
            #[repr(u8)]
            #[packed(bits = 1)]
            struct NotAnEnum;
        },
        quote! {
            #[repr(u8)]
            #[packed(bits = 1)]
            union NotAnEnum { value: u8 }
        },
    ] {
        let message = error(input);
        assert!(
            message.contains("packed can only be applied to enums"),
            "unexpected error: {message}"
        );
    }
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
