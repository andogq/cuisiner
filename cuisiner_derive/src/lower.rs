use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    Error, Expr, ExprLit, GenericArgument, GenericParam, Generics, Ident, Lit, Meta, Path, Token,
    Visibility, parse_quote, parse_quote_spanned, parse2, punctuated::Punctuated, spanned::Spanned,
};

use crate::{DeriveModel, DeriveModelItem, Fields, Repr};

/// From the provided [`DeriveModel`], generate an [`Ir`] representing it.
pub fn lower(model: DeriveModel) -> Result<Ir, Error> {
    let crate_name = parse_quote!(::cuisiner);

    Ok(Ir {
        base_ident: model.name.clone(),
        visibility: model.visibility,
        item: match model.item {
            DeriveModelItem::Struct {
                fields,
                generics,
                container_assert_layout,
            } => {
                let raw_ident = format!("___Cuisiner{}Raw", model.name);
                let raw_ident_tokens: TokenStream = raw_ident.parse()?;
                let raw_ident: Ident = parse2(raw_ident_tokens.clone())?;
                let extra_generic: GenericArgument = parse_quote!(#crate_name::BigEndian);

                ItemIr::Struct {
                    fields,
                    raw_ident,
                    raw_derives: vec![
                        parse_quote!(#crate_name::zerocopy::FromBytes),
                        parse_quote!(#crate_name::zerocopy::IntoBytes),
                        parse_quote!(#crate_name::zerocopy::Immutable),
                        parse_quote!(#crate_name::zerocopy::Unaligned),
                    ],
                    container_assert_layout: container_assert_layout.map(
                        |container_assert_layout| {
                            extend_assert_generics(
                                container_assert_layout,
                                extra_generic.clone(),
                                generics.params.len(),
                            )
                        },
                    ),
                    generics: StructGenerics::new(generics, &crate_name),
                }
            }
            DeriveModelItem::Enum { variants, repr } => ItemIr::Enum { repr, variants },
        },
        crate_name,
    })
}

/// Intermediate representation of the output of the derive macro.
pub struct Ir {
    /// Path of the crate that everything is exported from.
    pub crate_name: Path,
    /// Identifier of the base struct.
    pub base_ident: Ident,
    /// Item specific information.
    pub item: ItemIr,
    /// Visibility of the output item.
    pub visibility: Visibility,
}

/// IR specific to the item.
pub enum ItemIr {
    /// Struct IR.
    Struct {
        /// Fields present in the original struct.
        fields: Fields,
        /// Identifier of the raw struct.
        raw_ident: Ident,
        /// Derives to be added to the raw struct.
        raw_derives: Vec<Path>,
        /// Required generics.
        generics: StructGenerics,
        container_assert_layout: Option<Vec<Meta>>,
    },
    /// Enum IR.
    Enum {
        repr: Repr,
        variants: Vec<(Ident, usize)>,
    },
}

/// Generics required for lowering a struct.
pub struct StructGenerics {
    pub base: Generics,
    pub raw: Generics,
    pub b_ident: Ident,
    pub b_generic: GenericParam,
}

impl StructGenerics {
    pub fn new(base: Generics, crate_name: &Path) -> Self {
        // Ident for the `ByteOrder` generic.
        let b_ident = parse_quote!(___Cuisiner_Generic_B);
        let b_generic: GenericParam = parse_quote!(#b_ident: #crate_name::zerocopy::ByteOrder);

        let mut raw = base.clone();
        raw.params.push(b_generic.clone());

        Self {
            base,
            raw,
            b_ident,
            b_generic,
        }
    }
}

/// Traverses the provided attributes, to try find a `generics` attribute. If it doesn't exist, one
/// will be created. The `extra_generic` will be added to the generics list, and the modified
/// attributes will be returned.
fn extend_assert_generics(
    mut attrs: Vec<Meta>,
    extra_generic: GenericArgument,
    container_generic_count: usize,
) -> Vec<Meta> {
    let generics_count = attrs
        .iter_mut()
        // Pull out all `generics` attributes
        .filter_map(|attr| {
            if let Meta::List(list) = attr {
                let path = &list.path;
                let metas = extend_assert_generics(
                    list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                        .ok()?
                        .into_iter()
                        .collect(),
                    extra_generic.clone(),
                    container_generic_count,
                );

                *list = parse_quote_spanned! { list.span() => #path(#(#metas),*) };

                return None;
            }

            let Meta::NameValue(attr) = attr else {
                return None;
            };

            if !attr.path.is_ident("generics") {
                return None;
            }

            let Expr::Lit(ExprLit {
                lit: Lit::Str(generics_lit),
                ..
            }) = &mut attr.value
            else {
                return None;
            };

            let generics = generics_lit
                .parse_with(Punctuated::<GenericArgument, Token![,]>::parse_terminated)
                .ok()?;

            Some((generics_lit, generics))
        })
        // Update the literal with the updated generics
        .map(|(generics_lit, mut generics)| {
            generics.push(extra_generic.clone());

            let generics_str = generics.to_token_stream().to_string();
            *generics_lit = parse_quote_spanned! { generics_lit.span() => #generics_str };
        })
        .count();

    // If no generic was found, create a new attribute and insert it.
    if generics_count == 0 && container_generic_count == 0 {
        let extra_generic_str = extra_generic.to_token_stream().to_string();
        attrs.push(parse_quote!(generics = #extra_generic_str));
    }

    attrs
}

#[cfg(test)]
mod test {
    use super::*;

    use proc_macro2::Span;
    use syn::Visibility;

    fn test_struct_ir(model: DeriveModel, expected_raw_ident: impl AsRef<str>) {
        let ir = lower(model).unwrap();
        let ItemIr::Struct { raw_ident, .. } = ir.item else {
            panic!("expected struct item");
        };

        assert_eq!(raw_ident, expected_raw_ident.as_ref());
    }

    #[test]
    fn valid_struct_model() {
        test_struct_ir(
            DeriveModel {
                name: Ident::new("MyStruct", Span::call_site()),
                visibility: Visibility::Inherited,
                item: DeriveModelItem::Struct {
                    fields: Fields::Named(vec![(parse_quote!(a), parse_quote!(u64), None)]),
                    generics: Default::default(),
                    container_assert_layout: None,
                },
            },
            "___CuisinerMyStructRaw",
        );
    }

    #[test]
    fn valid_enum_model() {
        let ir = lower(DeriveModel {
            name: Ident::new("MyEnum", Span::call_site()),
            visibility: Visibility::Inherited,
            item: DeriveModelItem::Enum {
                variants: vec![
                    (parse_quote!(First), 1),
                    (parse_quote!(Second), 2),
                    (parse_quote!(Third), 3),
                ],
                repr: Repr::U32,
            },
        })
        .unwrap();

        let ItemIr::Enum { repr, variants } = ir.item else {
            panic!("expected enum item");
        };

        assert_eq!(repr, Repr::U32);
        assert_eq!(variants.len(), 3);
    }
    mod extend_assert_generics {
        use syn::parse_quote;

        use super::*;

        #[test]
        fn single_generic() {
            assert_eq!(
                extend_assert_generics(
                    vec![parse_quote!(generics = "A")],
                    parse_quote!(SomeIdent),
                    0,
                ),
                [parse_quote!(generics = "A , SomeIdent")]
            );
        }

        #[test]
        fn multi_generic() {
            assert_eq!(
                extend_assert_generics(
                    vec![parse_quote!(generics = "A, u32, Something")],
                    parse_quote!(SomeIdent),
                    0,
                ),
                [parse_quote!(generics = "A , u32 , Something , SomeIdent")]
            );
        }

        #[test]
        fn multi_with_const_generic() {
            assert_eq!(
                extend_assert_generics(
                    vec![parse_quote!(generics = "123, 'a', A, u32, Something")],
                    parse_quote!(SomeIdent),
                    0,
                ),
                [parse_quote!(
                    generics = "123 , 'a' , A , u32 , Something , SomeIdent"
                )]
            );
        }

        mod no_container_generics {
            use super::*;
            #[test]
            fn no_attrs() {
                assert_eq!(
                    extend_assert_generics(vec![], parse_quote!(SomeIdent), 0),
                    [parse_quote!(generics = "SomeIdent")]
                );
            }

            #[test]
            fn other_attrs() {
                assert_eq!(
                    extend_assert_generics(
                        vec![parse_quote!(some_path), parse_quote!(some_key = "value"),],
                        parse_quote!(SomeIdent),
                        0,
                    ),
                    [
                        parse_quote!(some_path),
                        parse_quote!(some_key = "value"),
                        parse_quote!(generics = "SomeIdent")
                    ]
                );
            }

            #[test]
            fn namespaced_generics() {
                assert_eq!(
                    extend_assert_generics(
                        vec![
                            parse_quote!(generics = "T"),
                            parse_quote!(namespace(generics = "T")),
                        ],
                        parse_quote!(SomeIdent),
                        0,
                    ),
                    [
                        parse_quote!(generics = "T , SomeIdent"),
                        parse_quote!(namespace(generics = "T , SomeIdent")),
                    ]
                );
            }

            #[test]
            fn namespaced_no_generics() {
                assert_eq!(
                    extend_assert_generics(
                        vec![parse_quote!(namespace(size = 0))],
                        parse_quote!(SomeIdent),
                        0,
                    ),
                    [
                        parse_quote!(namespace(size = 0, generics = "SomeIdent")),
                        parse_quote!(generics = "SomeIdent"),
                    ]
                );
            }
        }

        mod with_container_generics {
            use super::*;
            #[test]
            fn no_attrs() {
                assert_eq!(
                    extend_assert_generics(vec![], parse_quote!(SomeIdent), 1),
                    []
                );
            }

            #[test]
            fn other_attrs() {
                assert_eq!(
                    extend_assert_generics(
                        vec![parse_quote!(some_path), parse_quote!(some_key = "value"),],
                        parse_quote!(SomeIdent),
                        1,
                    ),
                    [parse_quote!(some_path), parse_quote!(some_key = "value"),]
                );
            }

            #[test]
            fn namespaced_generics() {
                assert_eq!(
                    extend_assert_generics(
                        vec![
                            parse_quote!(generics = "T"),
                            parse_quote!(namespace(generics = "T")),
                        ],
                        parse_quote!(SomeIdent),
                        1,
                    ),
                    [
                        parse_quote!(generics = "T , SomeIdent"),
                        parse_quote!(namespace(generics = "T , SomeIdent")),
                    ]
                );
            }

            #[test]
            fn namespaced_no_generics() {
                assert_eq!(
                    extend_assert_generics(
                        vec![parse_quote!(namespace(size = 0))],
                        parse_quote!(SomeIdent),
                        1,
                    ),
                    [parse_quote!(namespace(size = 0)),]
                );
            }
        }
    }
}
