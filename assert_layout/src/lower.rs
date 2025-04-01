use syn::{Error, Expr, Item, ItemStruct, parse_quote, parse_quote_spanned, spanned::Spanned};

use crate::analyse::{Assertion, AssertionItem, LayoutModel};

pub fn lower(model: LayoutModel) -> Result<Ir, Error> {
    Ok(Ir {
        item: model.item,
        assertions: model
            .assertions
            .into_iter()
            .map(|(generics, assertions)|  {
                (
                    generics.into_iter().map(|item| match item {
                        AssertionItem::Type { ty, ident } => {
                            parse_quote!(type #ident = #ty;)
                        },
                        AssertionItem::Const { ident, ty, expr } => {
                            parse_quote!(const #ident: #ty = #expr;)
                        }
                    }).collect(),
                    assertions.into_iter().map(|assertion|
                        match assertion {
                            Assertion::Size { ty, size } => {
                                parse_quote_spanned! { size.span() => ::core::mem::size_of::<#ty>() == #size }
                            }
                            Assertion::Offset {
                                container,
                                field,
                                offset,
                            } => {
                                parse_quote_spanned! { offset.span() => ::core::mem::offset_of!(#container, #field) == #offset }
                            }
                        }).collect()
                )
            })
            .collect(),
    })
}

pub struct Ir {
    pub item: ItemStruct,
    pub assertions: Vec<(Vec<Item>, Vec<Expr>)>,
}
