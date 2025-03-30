use std::collections::HashMap;

use proc_macro2::Span;
use syn::{
    Attribute, Error, Expr, ExprLit, GenericArgument, Generics, Ident, Lit, LitStr, Meta, Token,
    Visibility, meta::ParseNestedMeta, punctuated::Punctuated,
};

use crate::{Ast, Fields};

/// Analyse the struct, and produce a model for future usage.
pub fn analyse(ast: Ast) -> Result<DeriveModel, Error> {
    // Parse the attributes to pull out the config.
    let config = DeriveConfig::try_from(ast.attrs())?;

    Ok(match ast {
        Ast::Struct(item_struct) => DeriveModel {
            name: item_struct.ident.clone(),
            visibility: item_struct.vis,
            item: DeriveModelItem::Struct {
                fields: Fields::try_from(&item_struct.fields)?,
                generics: item_struct.generics,
                default_assert: config.default_assert,
                namespace_assert: config.namespace_assert,
            },
        },
        Ast::Enum(item_enum) => DeriveModel {
            name: item_enum.ident.clone(),
            visibility: item_enum.vis,
            item: DeriveModelItem::Enum {
                repr: config.repr.ok_or(Error::new(
                    Span::call_site(),
                    "'repr = ...' attribute is missing",
                ))?,
                variants: item_enum
                    .variants
                    .into_iter()
                    .map(|variant| {
                        if !matches!(variant.fields, syn::Fields::Unit) {
                            return Err(Error::new_spanned(
                                variant.fields,
                                "enum variants must be unit",
                            ));
                        }

                        let value = variant
                            .discriminant
                            .as_ref()
                            // Extract the literal
                            .and_then(|(_, discriminant)| {
                                if let Expr::Lit(ExprLit { lit, .. }) = discriminant {
                                    Some(lit)
                                } else {
                                    None
                                }
                            })
                            .ok_or_else(|| Error::new_spanned(&variant, "discriminant required"))
                            // Parse the literal
                            .and_then(|lit| match lit {
                                Lit::Int(value) => value.base10_parse().map_err(|_| {
                                    Error::new_spanned(value, "cannot parse discriminant")
                                }),
                                Lit::Byte(value) => Ok(value.value() as usize),
                                _ => Err(Error::new_spanned(
                                    lit,
                                    "only int or byte literal discriminants are supported",
                                )),
                            })?;

                        Ok((variant.ident, value))
                    })
                    .collect::<Result<_, _>>()?,
            },
        },
    })
}

/// All information required to be pulled from the AST to implement the derive macro.
#[derive(Clone)]
pub struct DeriveModel {
    /// Original name of the struct.
    pub name: Ident,
    /// Visibility of the original struct.
    pub visibility: Visibility,
    /// Additional information specific to the variant of model.
    pub item: DeriveModelItem,
}

#[derive(Clone)]
pub enum DeriveModelItem {
    Struct {
        /// Collection of fields present in the original struct.
        fields: Fields,
        /// Generics present on the original struct.
        generics: Generics,
        default_assert: AssertConfig,
        namespace_assert: HashMap<String, AssertConfig>,
    },
    Enum {
        /// All variants and their discriminant values.
        variants: Vec<(Ident, usize)>,
        /// Internal enum representation.
        repr: Repr,
    },
}

/// Configuration provided via attributes.
#[derive(Clone, Default)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
struct DeriveConfig {
    repr: Option<Repr>,
    default_assert: AssertConfig,
    namespace_assert: HashMap<String, AssertConfig>,
}

/// Container assertions that can be provided via an attribute.
#[derive(Clone, Default)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct AssertConfig {
    pub size: Option<Expr>,
    pub generics: Vec<GenericArgument>,
}

impl AssertConfig {
    fn try_parse_meta(&mut self, meta: &ParseNestedMeta) -> Result<bool, Error> {
        if meta.path.is_ident("assert_size") {
            self.size = Some(meta.value()?.parse()?);

            return Ok(true);
        }

        if meta.path.is_ident("assert_generics") {
            let assert_generics_str = meta.value()?.parse::<LitStr>()?;
            let assert_generics = assert_generics_str
                .parse_with(Punctuated::<GenericArgument, Token![,]>::parse_terminated)?;

            self.generics = assert_generics.into_iter().collect();

            return Ok(true);
        }

        Ok(false)
    }
}

impl TryFrom<&[Attribute]> for DeriveConfig {
    type Error = Error;

    fn try_from(attrs: &[Attribute]) -> Result<Self, Self::Error> {
        let mut config = Self::default();

        for attr in attrs {
            if !attr.path().is_ident("cuisiner") {
                continue;
            }

            let Meta::List(attr) = &attr.meta else {
                return Err(Error::new_spanned(
                    attr,
                    "attribute must be in list format (eg `#[cuisiner(argument)]`)",
                ));
            };

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("repr") {
                    config.repr = Some(Repr::try_from(
                        meta.value()?.parse::<Ident>()?.to_string().as_str(),
                    )?);

                    return Ok(());
                }

                if config.default_assert.try_parse_meta(&meta)? {
                    return Ok(());
                }

                // Try to parse as a namespaced assertion
                let nested_name = meta.path.require_ident()?.to_string();
                let mut nested_assert_config = AssertConfig::default();
                meta.parse_nested_meta(|meta| {
                    if nested_assert_config.try_parse_meta(&meta)? {
                        return Ok(());
                    }

                    Err(Error::new_spanned(meta.path, "unknown attribute argument"))
                })?;
                config
                    .namespace_assert
                    .insert(nested_name, nested_assert_config);

                Ok(())
            })?;
        }

        Ok(config)
    }
}

#[derive(Clone)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub enum Repr {
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
}

impl TryFrom<&str> for Repr {
    type Error = Error;

    fn try_from(repr: &str) -> Result<Self, Self::Error> {
        match repr {
            "u8" => Ok(Self::U8),
            "u16" => Ok(Self::U16),
            "u32" => Ok(Self::U32),
            "u64" => Ok(Self::U64),
            "u128" => Ok(Self::U128),
            "usize" => Ok(Self::Usize),
            "i8" => Ok(Self::I8),
            "i16" => Ok(Self::I16),
            "i32" => Ok(Self::I32),
            "i64" => Ok(Self::I64),
            "i128" => Ok(Self::I128),
            "isize" => Ok(Self::Isize),
            repr => Err(Error::new(
                Span::call_site(),
                format!("unknown repr: {repr}"),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use syn::parse_quote;

    use super::*;

    fn test_analyse_struct(
        ast: Ast,
        expected_name: impl AsRef<str>,
        expected_field_count: Option<usize>,
        expected_assert_size: Option<Expr>,
    ) {
        let model = analyse(ast).unwrap();

        let DeriveModelItem::Struct {
            fields,
            generics: _,
            default_assert,
            namespace_assert: _,
        } = &model.item
        else {
            panic!("expected struct derive model item");
        };

        assert_eq!(model.name, expected_name.as_ref());
        assert_eq!(
            match fields {
                Fields::Named(fields) => Some(fields.len()),
                Fields::Unnamed(fields) => Some(fields.len()),
                Fields::Unit => None,
            },
            expected_field_count
        );
        assert_eq!(default_assert.size, expected_assert_size);
    }

    fn test_analyse_enum(ast: Ast, expected_repr: Repr, expected_variants: &[(Ident, usize)]) {
        let model = analyse(ast).unwrap();
        let DeriveModelItem::Enum { variants, repr } = model.item else {
            panic!("expected enum derive model item");
        };

        assert_eq!(repr, expected_repr);
        assert_eq!(variants, expected_variants);
    }

    #[test]
    fn analyse_valid_unit_struct() {
        test_analyse_struct(
            Ast::Struct(parse_quote! {
                struct MyStruct;
            }),
            "MyStruct",
            None,
            None,
        );
    }

    #[test]
    fn analyse_valid_tuple_struct() {
        test_analyse_struct(
            Ast::Struct(parse_quote! {
                struct MyStruct(u32);
            }),
            "MyStruct",
            Some(1),
            None,
        );
    }

    #[test]
    fn analyse_valid_struct() {
        test_analyse_struct(
            Ast::Struct(parse_quote! {
                struct MyStruct {
                    a: u32,
                    b: bool,
                }
            }),
            "MyStruct",
            Some(2),
            None,
        );
    }

    #[test]
    fn analyse_valid_struct_with_size_assert() {
        test_analyse_struct(
            Ast::Struct(parse_quote! {
                #[cuisiner(assert_size = 5)]
                struct MyStruct {
                    a: u32,
                    b: bool,
                }
            }),
            "MyStruct",
            Some(2),
            Some(parse_quote!(5)),
        );
    }

    #[test]
    fn invalid_attribute() {
        assert!(
            analyse(Ast::Struct(parse_quote! {
                #[cuisiner(some_attribute)]
                struct MyStruct {
                    a: u32,
                }
            }))
            .is_err()
        );
    }

    #[test]
    fn analyse_valid_enum() {
        test_analyse_enum(
            Ast::Enum(parse_quote! {
                #[cuisiner(repr = u32)]
                enum MyEnum {
                    First = 1,
                    Second = 2,
                    Third = 3,
                }
            }),
            Repr::U32,
            &[
                (parse_quote!(First), 1),
                (parse_quote!(Second), 2),
                (parse_quote!(Third), 3),
            ],
        );
    }

    #[test]
    fn enum_missing_repr() {
        assert!(
            analyse(Ast::Enum(parse_quote! {
                enum MyEnum {
                    First = 1,
                    Second = 2,
                    Third = 3,
                }
            }))
            .is_err()
        );
    }

    #[test]
    fn enum_missing_discriminant() {
        assert!(
            analyse(Ast::Enum(parse_quote! {
                #[cuisiner(repr = u32)]
                enum MyEnum {
                    First,
                    Second,
                    Third,
                }
            }))
            .is_err()
        );
    }

    #[test]
    fn enum_some_discriminants() {
        assert!(
            analyse(Ast::Enum(parse_quote! {
                #[cuisiner(repr = u32)]
                enum MyEnum {
                    First = 1,
                    Second,
                    Third,
                }
            }))
            .is_err()
        );
    }

    mod derive_config {
        use syn::parse_quote;

        use super::*;

        #[test]
        fn from_empty_attributes() {
            assert_eq!(
                DeriveConfig::try_from([].as_slice()).unwrap(),
                DeriveConfig::default()
            );
        }

        #[test]
        fn single_attribute_path() {
            assert!(DeriveConfig::try_from([parse_quote!(#[cuisiner])].as_slice()).is_err());
        }

        #[test]
        fn single_attribute_empty_list() {
            assert_eq!(
                DeriveConfig::try_from([parse_quote!(#[cuisiner()])].as_slice()).unwrap(),
                DeriveConfig::default()
            );
        }

        #[test]
        fn with_repr() {
            assert_eq!(
                DeriveConfig::try_from([parse_quote!(#[cuisiner(repr = i64)])].as_slice()).unwrap(),
                DeriveConfig {
                    repr: Some(Repr::I64),
                    ..Default::default()
                }
            )
        }

        #[test]
        fn extra_attributes() {
            assert_eq!(
                DeriveConfig::try_from(
                    [parse_quote!(#[repr(C)]), parse_quote!(#[some = attribute])].as_slice()
                )
                .unwrap(),
                DeriveConfig::default()
            );
        }

        mod assert_generics {
            use super::*;

            #[test]
            fn none() {
                assert_eq!(
                    DeriveConfig::try_from(
                        [parse_quote!(#[cuisiner(assert_generics = "")])].as_slice()
                    )
                    .unwrap(),
                    DeriveConfig {
                        default_assert: AssertConfig {
                            generics: vec![],
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                )
            }

            #[test]
            fn one() {
                assert_eq!(
                    DeriveConfig::try_from(
                        [parse_quote!(#[cuisiner(assert_generics = "SomeType")])].as_slice()
                    )
                    .unwrap(),
                    DeriveConfig {
                        default_assert: AssertConfig {
                            generics: vec![parse_quote!(SomeType)],
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                )
            }

            #[test]
            fn many() {
                assert_eq!(
                    DeriveConfig::try_from(
                        [parse_quote!(#[cuisiner(assert_generics = "'static, SomeType, u64")])]
                            .as_slice()
                    )
                    .unwrap(),
                    DeriveConfig {
                        default_assert: AssertConfig {
                            generics: vec![
                                parse_quote!('static),
                                parse_quote!(SomeType),
                                parse_quote!(u64)
                            ],
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                )
            }

            #[test]
            fn namespaced() {
                assert_eq!(
                    DeriveConfig::try_from(
                        [parse_quote!(#[cuisiner(some_namespace(assert_generics = "'static, SomeType, u64"))])]
                            .as_slice()
                    )
                    .unwrap(),
                    DeriveConfig {
                        namespace_assert: [("some_namespace".into(), AssertConfig {
                            generics: vec![
                                parse_quote!('static),
                                parse_quote!(SomeType),
                                parse_quote!(u64)
                            ],
                            ..Default::default()
                        })].into_iter().collect(),
                        ..Default::default()
                    }
                )
            }

            #[test]
            fn multiple_namespaced() {
                assert_eq!(
                    DeriveConfig::try_from(
                        [
                            parse_quote!(#[cuisiner(some_namespace(assert_generics = "'static, SomeType, u64"))]),
                            parse_quote!(#[cuisiner(something(assert_generics = "bool, T", assert_size = 123))])
                        ]
                            .as_slice()
                    )
                        .unwrap(),
                    DeriveConfig {
                        namespace_assert: [
                            ("some_namespace".into(), AssertConfig {
                                generics: vec![
                                    parse_quote!('static),
                                    parse_quote!(SomeType),
                                    parse_quote!(u64)
                                ],
                                ..Default::default()
                            }),
                            ("something".into(), AssertConfig {
                                generics: vec![
                                    parse_quote!(bool),
                                    parse_quote!(T)
                                ],
                                size: Some(parse_quote!(123))
                            })].into_iter().collect(),
                        ..Default::default()
                    }
                )
            }
        }

        #[test]
        fn multiple_attributes() {
            assert!(
                DeriveConfig::try_from(
                    [
                        parse_quote!(#[cuisiner]),
                        parse_quote!(#[cuisiner(another_attribute)]),
                    ]
                    .as_slice()
                )
                .is_err()
            );
        }

        #[test]
        fn unknown_attribute_argument() {
            assert!(
                DeriveConfig::try_from([parse_quote!(#[cuisiner(another_attribute)]),].as_slice())
                    .is_err()
            );
        }
    }
}
