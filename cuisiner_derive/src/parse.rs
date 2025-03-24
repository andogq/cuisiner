use syn::{Attribute, DeriveInput, Error, ItemEnum, ItemStruct};

/// From the provided [`DeriveInput`], assert that it's a [`Struct` definition](ItemStruct), and
/// return it.
pub fn parse(derive_input: DeriveInput) -> Result<Ast, Error> {
    Ok(match derive_input.data {
        syn::Data::Struct(data_struct) => Ast::Struct(ItemStruct {
            attrs: derive_input.attrs,
            vis: derive_input.vis,
            struct_token: data_struct.struct_token,
            ident: derive_input.ident,
            generics: derive_input.generics,
            fields: data_struct.fields,
            semi_token: data_struct.semi_token,
        }),
        syn::Data::Enum(data_enum) => Ast::Enum(ItemEnum {
            attrs: derive_input.attrs,
            vis: derive_input.vis,
            enum_token: data_enum.enum_token,
            ident: derive_input.ident,
            generics: derive_input.generics,
            brace_token: data_enum.brace_token,
            variants: data_enum.variants,
        }),
        syn::Data::Union(_) => {
            return Err(Error::new_spanned(derive_input, "`union` is not supported"));
        }
    })
}

/// AST that the macro is able to process.
pub enum Ast {
    /// Struct definition that the macro was called on.
    Struct(ItemStruct),
    /// Enum definition that the macro was called on.
    Enum(ItemEnum),
}

impl Ast {
    /// Fetch the attributes for the AST node.
    pub fn attrs(&self) -> &[Attribute] {
        match self {
            Ast::Struct(item_struct) => &item_struct.attrs,
            Ast::Enum(item_enum) => &item_enum.attrs,
        }
    }
}

#[cfg(test)]
mod test {
    use syn::parse_quote;

    use super::*;

    fn test_struct(
        input: DeriveInput,
        expected_ident: impl AsRef<str>,
        expected_field_count: Option<usize>,
    ) {
        let Ast::Struct(input) = parse(input).unwrap() else {
            panic!("expected struct");
        };

        assert_eq!(input.ident, expected_ident.as_ref());

        assert_eq!(
            match input.fields {
                syn::Fields::Named(fields_named) => Some(fields_named.named.len()),
                syn::Fields::Unnamed(fields_unnamed) => Some(fields_unnamed.unnamed.len()),
                syn::Fields::Unit => None,
            },
            expected_field_count
        )
    }

    fn test_enum(
        input: DeriveInput,
        expected_ident: impl AsRef<str>,
        expected_variant_count: usize,
    ) {
        let Ast::Enum(input) = parse(input).unwrap() else {
            panic!("expected enum");
        };

        assert_eq!(input.ident, expected_ident.as_ref());

        assert_eq!(input.variants.len(), expected_variant_count)
    }

    #[test]
    fn parse_unit_struct() {
        test_struct(
            parse_quote! {
                struct MyStruct;
            },
            "MyStruct",
            None,
        );
    }

    #[test]
    fn parse_tuple_struct_empty() {
        test_struct(
            parse_quote! {
                struct MyStruct();
            },
            "MyStruct",
            Some(0),
        );
    }

    #[test]
    fn parse_tuple_struct() {
        test_struct(
            parse_quote! {
                struct MyStruct(usize, isize, f32);
            },
            "MyStruct",
            Some(3),
        );
    }

    #[test]
    fn parse_struct_empty() {
        test_struct(
            parse_quote! {
                struct MyStruct {}
            },
            "MyStruct",
            Some(0),
        );
    }

    #[test]
    fn parse_struct() {
        test_struct(
            parse_quote! {
                struct MyStruct {
                    a: usize,
                    b: isize,
                    c: f32
                }
            },
            "MyStruct",
            Some(3),
        );
    }

    #[test]
    fn parse_empty_enum() {
        test_enum(
            parse_quote! {
                enum MyEnum {}
            },
            "MyEnum",
            0,
        );
    }

    #[test]
    fn parse_single_variant_enum() {
        test_enum(
            parse_quote! {
                enum MyEnum { First }
            },
            "MyEnum",
            1,
        );
    }

    #[test]
    fn parse_multi_variant_enum() {
        test_enum(
            parse_quote! {
                enum MyEnum { First, Second, Third }
            },
            "MyEnum",
            3,
        );
    }

    #[test]
    fn parse_field_enum() {
        test_enum(
            parse_quote! {
                enum MyEnum { First(u16), Second(bool), Third(String) }
            },
            "MyEnum",
            3,
        );
    }

    #[test]
    fn error_on_union() {
        assert!(
            parse(parse_quote! {
                union MyUnion { }
            })
            .is_err()
        )
    }
}
