use proc_macro2::Span;
use syn::{Error, Ident, Path, parse_quote};

use crate::{DeriveModel, DeriveModelItem, Fields, Repr};

/// From the provided [`DeriveModel`], generate an [`Ir`] representing it.
pub fn lower(model: DeriveModel) -> Result<Ir, Error> {
    let crate_name = parse_quote!(::cuisiner);

    Ok(Ir {
        base_ident: model.name.clone(),
        item: match model.item {
            DeriveModelItem::Struct {
                fields,
                assert_size,
            } => ItemIr::Struct {
                fields,
                raw_ident: Ident::new(&format!("___Cuisiner{}Raw", model.name), Span::call_site()),
                raw_derives: vec![
                    parse_quote!(#crate_name::zerocopy::FromBytes),
                    parse_quote!(#crate_name::zerocopy::IntoBytes),
                    parse_quote!(#crate_name::zerocopy::Immutable),
                    parse_quote!(#crate_name::zerocopy::Unaligned),
                ],
                assert_size,
            },
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
        /// Size to assert in the output.
        assert_size: Option<usize>,
    },
    /// Enum IR.
    Enum {
        repr: Repr,
        variants: Vec<(Ident, usize)>,
    },
}

#[cfg(test)]
mod test {
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
                item: DeriveModelItem::Struct {
                    fields: Fields::Named(vec![(parse_quote!(a), parse_quote!(u64))]),
                    assert_size: None,
                },
            },
            "___CuisinerMyStructRaw",
        );
    }

    #[test]
    fn valid_enum_model() {
        let ir = lower(DeriveModel {
            name: Ident::new("MyEnum", Span::call_site()),
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
