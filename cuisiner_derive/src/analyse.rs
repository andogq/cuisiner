use std::str::FromStr;

use proc_macro2::Span;
use syn::{Attribute, Error, Ident, ItemStruct, Meta};

use crate::{Endian, Fields};

/// Analyse the struct, and produce a model for future usage.
pub fn analyse(item_struct: ItemStruct) -> Result<DeriveModel, Error> {
    // Parse the attributes to pull out the config.
    let config = DeriveConfig::try_from(item_struct.attrs.as_slice())?;

    Ok(DeriveModel {
        name: item_struct.ident.clone(),
        endian: config.endian,
        fields: Fields::from(&item_struct.fields),
    })
}

/// All information required to be pulled from the AST to implement the derive macro.
#[derive(Clone)]
pub struct DeriveModel {
    /// Original name of the struct.
    pub name: Ident,
    /// The configured endianness to generate the raw struct with.
    pub endian: Endian,
    /// Collection of fields present in the original struct.
    pub fields: Fields,
}

/// Configuration provided via attributes.
#[derive(Clone, Default)]
struct DeriveConfig {
    endian: Endian,
}

impl TryFrom<&[Attribute]> for DeriveConfig {
    type Error = Error;

    fn try_from(attrs: &[Attribute]) -> Result<Self, Self::Error> {
        // Search for relevant attributes.
        let mut attrs = attrs.iter().filter(|attr| attr.path().is_ident("cuisiner"));

        let mut config = Self::default();

        let Some(attr) = attrs.next() else {
            // No attributes provided.
            return Ok(config);
        };

        // Make sure only one attribute is provided.
        if attrs.next().is_some() {
            return Err(Error::new(
                Span::call_site(),
                "only a single `cuisiner` attribute is supported",
            ));
        }

        match &attr.meta {
            // Accept attibute with no arguments, although it's useless.
            Meta::Path(_) => {},
            // Parse out arguments from list.
            Meta::List(_) => attr.parse_nested_meta(|meta| {
                // Attempt to read the endianness attribute.
                if let Some(endian) = meta
                    .path
                    .get_ident()
                    .and_then(|ident| Endian::from_str(&ident.to_string()).ok())
                {
                    config.endian = endian;

                    return Ok(());
                }

                Err(Error::new_spanned(meta.path, "unknown attribute argument, expected endianness (`big_endian`, `little_endian`, etc)"))
            })?,
            // Reject all other formats
            _ => {
                return Err(Error::new_spanned(attr, "attribute must be in list format (eg `#[cuisiner(argument)]`)"));
            },
        }

        Ok(config)
    }
}

#[cfg(test)]
mod test {
    use syn::parse_quote;

    use super::*;

    fn test_analyse(
        s: ItemStruct,
        expected_name: impl AsRef<str>,
        expected_endian: Endian,
        expected_field_count: Option<usize>,
    ) {
        let model = analyse(s).unwrap();

        assert_eq!(model.name, expected_name.as_ref());
        assert_eq!(model.endian, expected_endian);
        assert_eq!(
            match model.fields {
                Fields::Named(fields) => Some(fields.len()),
                Fields::Unnamed(fields) => Some(fields.len()),
                Fields::Unit => None,
            },
            expected_field_count
        );
    }

    #[test]
    fn analyse_valid_unit_struct() {
        test_analyse(
            parse_quote! {
                struct MyStruct;
            },
            "MyStruct",
            Endian::default(),
            None,
        );
    }

    #[test]
    fn analyse_valid_tuple_struct() {
        test_analyse(
            parse_quote! {
                struct MyStruct(u32);
            },
            "MyStruct",
            Endian::default(),
            Some(1),
        );
    }

    #[test]
    fn analyse_valid_struct() {
        test_analyse(
            parse_quote! {
                struct MyStruct {
                    a: u32,
                    b: bool,
                }
            },
            "MyStruct",
            Endian::default(),
            Some(2),
        );
    }

    #[test]
    fn analyse_valid_struct_with_empty_attribute() {
        test_analyse(
            parse_quote! {
                #[cuisiner]
                struct MyStruct {
                    a: u32,
                    b: bool,
                }
            },
            "MyStruct",
            Endian::default(),
            Some(2),
        );
    }

    #[test]
    fn analyse_valid_struct_with_endian() {
        test_analyse(
            parse_quote! {
                #[cuisiner(little_endian)]
                struct MyStruct {
                    a: u32,
                    b: bool,
                }
            },
            "MyStruct",
            Endian::LittleEndian,
            Some(2),
        );
    }

    #[test]
    fn invalid_endian() {
        assert!(
            analyse(parse_quote! {
                #[cuisiner(some_endian)]
                struct MyStruct {
                    a: u32,
                }
            })
            .is_err()
        );
    }

    mod derive_config {
        use syn::parse_quote;

        use super::*;

        fn test_config(attrs: &[Attribute], expected_endian: Endian) {
            let config = DeriveConfig::try_from(attrs).unwrap();

            assert_eq!(config.endian, expected_endian);
        }

        #[test]
        fn from_empty_attributes() {
            test_config(&[], Endian::default());
        }

        #[test]
        fn single_attribute_path() {
            test_config(&[parse_quote!(#[cuisiner])], Endian::default());
        }

        #[test]
        fn single_attribute_empty_list() {
            test_config(&[parse_quote!(#[cuisiner()])], Endian::default());
        }

        #[test]
        fn single_attribute_valid_endian() {
            test_config(
                &[parse_quote!(#[cuisiner(little_endian)])],
                Endian::LittleEndian,
            );
        }

        #[test]
        fn extra_attributes() {
            test_config(
                &[
                    parse_quote!(#[repr(C)]),
                    parse_quote!(#[some = attribute]),
                    parse_quote!(#[cuisiner(little_endian)]),
                ],
                Endian::LittleEndian,
            );
        }

        #[test]
        fn only_extra_attributes() {
            test_config(
                &[parse_quote!(#[repr(C)]), parse_quote!(#[some = attribute])],
                Endian::default(),
            );
        }

        #[test]
        fn multiple_attributes() {
            assert!(
                DeriveConfig::try_from(&[
                    parse_quote!(#[cuisiner]),
                    parse_quote!(#[cuisiner(big_endian)]),
                ] as &[Attribute])
                .is_err()
            );
        }

        #[test]
        fn unknown_attribute_argument() {
            assert!(
                DeriveConfig::try_from(&[parse_quote!(#[cuisiner(some_endian)]),] as &[Attribute])
                    .is_err()
            );
        }
    }
}
