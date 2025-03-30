use proc_macro2::TokenStream;
use syn::{
    Error, Expr, GenericArgument, GenericParam, Generics, Ident, Path, Type, Visibility,
    parse_quote, parse2,
};

use crate::{DeriveModel, DeriveModelItem, FieldAssertions, Fields, Repr};

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
                default_assert,
                namespace_assert,
            } => {
                let raw_ident = format!("___Cuisiner{}Raw", model.name);
                let raw_ident_tokens: TokenStream = raw_ident.parse()?;
                let raw_ident: Ident = parse2(raw_ident_tokens.clone())?;

                let mut assertions = Vec::new();

                let container_ty = |generics: &[GenericArgument]| {
                    let generics = generics.iter().cloned().chain(std::iter::once(
                        parse_quote!(#crate_name::zerocopy::byteorder::BigEndian),
                    ));

                    parse_quote!(#raw_ident::<#(#generics,)*>)
                };

                let raw_ty =
                    |ty| parse_quote!(<#ty as #crate_name::Cuisiner>::Raw<#crate_name::BigEndian>);

                if let Some(assert_size) = &default_assert.size {
                    assertions.push(Assertion::Size {
                        ty: container_ty(&default_assert.generics),
                        size: assert_size.clone(),
                    });
                }

                fn add_assertions(
                    assertions: &mut Vec<Assertion>,
                    container: Type,
                    field: Ident,
                    ty: Type,
                    field_assertions: &FieldAssertions,
                ) {
                    if let Some(size) = &field_assertions.size {
                        assertions.push(Assertion::Size {
                            ty,
                            size: size.clone(),
                        });
                    }

                    if let Some(offset) = &field_assertions.offset {
                        assertions.push(Assertion::Offset {
                            container,
                            field,
                            offset: offset.clone(),
                        });
                    }
                }

                match &fields {
                    Fields::Named(fields) => {
                        for (ident, ty, field_assertions) in fields {
                            add_assertions(
                                &mut assertions,
                                container_ty(&default_assert.generics),
                                ident.clone(),
                                raw_ty(ty),
                                field_assertions,
                            );
                        }
                    }
                    Fields::Unnamed(fields) => {
                        for (i, (ty, field_assertions)) in fields.iter().enumerate() {
                            add_assertions(
                                &mut assertions,
                                container_ty(&default_assert.generics),
                                parse_quote!(#i),
                                raw_ty(ty),
                                field_assertions,
                            );
                        }
                    }
                    _ => {}
                };

                ItemIr::Struct {
                    fields,
                    raw_ident,
                    raw_derives: vec![
                        parse_quote!(#crate_name::zerocopy::FromBytes),
                        parse_quote!(#crate_name::zerocopy::IntoBytes),
                        parse_quote!(#crate_name::zerocopy::Immutable),
                        parse_quote!(#crate_name::zerocopy::Unaligned),
                    ],
                    assertions,
                    generics: StructGenerics::new(generics, &crate_name),
                }
            }
            DeriveModelItem::Enum { variants, repr } => ItemIr::Enum { repr, variants },
        },
        crate_name,
    })
}

pub enum Assertion {
    Size {
        ty: Type,
        size: Expr,
    },
    Offset {
        container: Type,
        field: Ident,
        offset: Expr,
    },
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
        /// Assertions to apply to output.
        assertions: Vec<Assertion>,
        /// Required generics.
        generics: StructGenerics,
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

#[cfg(test)]
mod test {
    use proc_macro2::Span;
    use syn::Visibility;

    use crate::FieldAssertions;

    use super::*;

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
                    fields: Fields::Named(vec![(
                        parse_quote!(a),
                        parse_quote!(u64),
                        FieldAssertions::default(),
                    )]),
                    generics: Default::default(),
                    default_assert: Default::default(),
                    namespace_assert: Default::default(),
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
}
