mod qualify;
mod util;

use syn::{
    Error, Expr, GenericArgument, GenericParam, Ident, ItemStruct, Member, Type, parse_quote,
};

use self::{qualify::*, util::*};
use crate::parse::Ast;

pub fn analyse(ast: Ast) -> Result<LayoutModel, Error> {
    // Build the container type.
    let container_ident = ast.item.ident.clone();
    let container_generics = ast.item.generics.split_for_impl().1;
    let container_type: Type = parse_quote!(#container_ident #container_generics);

    let assertions = ast
        .generics
        .into_iter()
        .map(|generics| {
            // Make sure that the provided generics match with the definition's.
            if generics.len() != ast.item.generics.params.len() {
                return Err(Error::new(
                    generics.span(),
                    format!(
                        "expected {} generic arguments, but found {}",
                        ast.item.generics.params.len(),
                        generics.len()
                    ),
                ));
            }

            let bound_generics = get_bound_generics(&ast.item.generics);

            // Convert the generics into a list of top-level items (`type` and `const` statements) used
            // during assertions.
            let assert_items: Vec<AssertionItem> = ast
                .item
                .generics
                .params
                .iter()
                .zip(generics.iter())
                .map(AssertionItem::try_from)
                .collect::<Result<_, _>>()?;

            let assertions = ast
                .field_assertions
                .iter()
                .flat_map(|assertion| {
                    [
                        assertion.size.as_ref().map(|size| Assertion::Size {
                            ty: qualify_generic(&bound_generics, assertion.ty.clone()),
                            size: size.clone(),
                        }),
                        assertion.offset.as_ref().map(|offset| Assertion::Offset {
                            container: container_type.clone(),
                            field: assertion.field.clone(),
                            offset: offset.clone(),
                        }),
                    ]
                })
                // Add container assertion
                .chain(std::iter::once(ast.size.as_ref().map(|size| {
                    Assertion::Size {
                        ty: container_type.clone(),
                        size: size.clone(),
                    }
                })))
                .flatten()
                .collect();

            Ok((assert_items, assertions))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(LayoutModel {
        item: ast.item,
        assertions,
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
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub enum AssertionItem {
    Const { ident: Ident, ty: Type, expr: Expr },
    Type { ident: Ident, ty: Type },
}

impl TryFrom<(&GenericParam, &GenericArgument)> for AssertionItem {
    type Error = Error;

    fn try_from((param, ty): (&GenericParam, &GenericArgument)) -> Result<Self, Self::Error> {
        Ok(match (param, ty) {
            (GenericParam::Lifetime(_), GenericArgument::Lifetime(_)) => {
                unimplemented!()
            }
            (GenericParam::Type(type_param), GenericArgument::Type(ty)) => AssertionItem::Type {
                ident: type_param.ident.clone(),
                ty: ty.clone(),
            },
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
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn type_item() {
        assert_eq!(
            AssertionItem::try_from((&parse_quote!(T), &parse_quote!(MyType))).unwrap(),
            AssertionItem::Type {
                ident: parse_quote!(T),
                ty: parse_quote!(MyType),
            }
        );
    }

    #[test]
    fn const_item() {
        assert_eq!(
            AssertionItem::try_from((&parse_quote!(const N: usize), &parse_quote!(128))).unwrap(),
            AssertionItem::Const {
                ident: parse_quote!(N),
                ty: parse_quote!(usize),
                expr: parse_quote!(128),
            }
        );
    }

    #[test]
    fn mismatch() {
        assert!(AssertionItem::try_from((&parse_quote!(T), &parse_quote!(128))).is_err());
    }
}
