use syn::{DeriveInput, Error, ItemStruct};

/// From the provided [`DeriveInput`], assert that it's a [`Struct` definition](ItemStruct), and
/// return it.
pub fn parse(derive_input: DeriveInput) -> Result<ItemStruct, Error> {
    let syn::Data::Struct(data_struct) = derive_input.data else {
        return Err(Error::new_spanned(
            derive_input,
            "only `struct` is supported",
        ));
    };

    Ok(ItemStruct {
        attrs: derive_input.attrs,
        vis: derive_input.vis,
        struct_token: data_struct.struct_token,
        ident: derive_input.ident,
        generics: derive_input.generics,
        fields: data_struct.fields,
        semi_token: data_struct.semi_token,
    })
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
        let input = parse(input).unwrap();

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
    fn error_on_enum() {
        assert!(
            parse(parse_quote! {
                enum MyEnum { A, B, C }
            })
            .is_err()
        )
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
