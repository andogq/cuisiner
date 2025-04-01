use std::ops::Deref;

use proc_macro2::Span;
use syn::{
    Error, Expr, GenericArgument, GenericParam, Ident, ItemStruct, Member, Type, parse_quote,
};

use crate::parse::Ast;

pub fn analyse(ast: Ast) -> Result<LayoutModel, Error> {
    // Make sure that the provided generics match the number on the container.
    let provided_generic_count = ast
        .generics
        .as_ref()
        .map(|generics| generics.len())
        .unwrap_or(0);
    if provided_generic_count != ast.item.generics.params.len() {
        return Err(Error::new(
            ast.generics
                .map(|generics| generics.span())
                .unwrap_or(Span::call_site()),
            format!(
                "expected {} generic arguments provided, but found {provided_generic_count}",
                ast.item.generics.params.len()
            ),
        ));
    }

    // Build the container type.
    let container_ident = ast.item.ident.clone();
    let container_generics = ast.generics.iter().flat_map(|generic| generic.deref());
    let container_type: Type = parse_quote!(#container_ident<#(#container_generics,)*>);

    // Zip up provided generics with container generics.
    let generics = ast
        .item
        .generics
        .params
        .iter()
        .zip(ast.generics.iter().flat_map(|generics| generics.deref()))
        .map(|(param, ty)| {
            // Expand the generics.
            Ok(match (param, ty) {
                (GenericParam::Lifetime(_), GenericArgument::Lifetime(_)) => unimplemented!(),
                (GenericParam::Type(type_param), GenericArgument::Type(ty)) => {
                    AssertionItem::Type {
                        ident: type_param.ident.clone(),
                        ty: ty.clone(),
                    }
                }
                (GenericParam::Const(const_param), GenericArgument::Const(const_expr)) => {
                    AssertionItem::Const {
                        ident: const_param.ident.clone(),
                        ty: const_param.ty.clone(),
                        expr: const_expr.clone(),
                    }
                }
                (param, ty) => {
                    return Err(Error::new_spanned(
                        ty,
                        format!(
                            "{} required as generic argument",
                            match param {
                                GenericParam::Lifetime(_) => "lifetime",
                                GenericParam::Type(_) => "type",
                                GenericParam::Const(_) => "const",
                            }
                        ),
                    ));
                }
            })
        })
        .collect::<Result<_, _>>()?;

    Ok(LayoutModel {
        item: ast.item,
        assertions: vec![(
            generics,
            ast.field_assertions
                .into_iter()
                // Expand field assertions
                .flat_map(|assertion| {
                    [
                        assertion.size.map(|size| Assertion::Size {
                            ty: assertion.ty,
                            size,
                        }),
                        assertion.offset.map(|offset| Assertion::Offset {
                            container: container_type.clone(),
                            field: assertion.field,
                            offset,
                        }),
                    ]
                })
                // Add container assertion
                .chain(std::iter::once(ast.size.map(|size| Assertion::Size {
                    ty: container_type.clone(),
                    size,
                })))
                .flatten()
                .collect(),
        )],
    })
}

pub struct LayoutModel {
    pub item: ItemStruct,
    pub assertions: Vec<(Vec<AssertionItem>, Vec<Assertion>)>,
}

pub enum Assertion {
    Size {
        ty: Type,
        size: Expr,
    },
    Offset {
        container: Type,
        field: Member,
        offset: Expr,
    },
}

/// Items which are allowed to be expanded in an assertion.
pub enum AssertionItem {
    Const { ident: Ident, ty: Type, expr: Expr },
    Type { ident: Ident, ty: Type },
}
