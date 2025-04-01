use std::{collections::HashMap, ops::Deref};

use proc_macro2::Span;
use syn::{
    Error, Expr, GenericArgument, GenericParam, Ident, ItemStruct, Member, Path, PathArguments,
    PathSegment, Token, Type, TypeParamBound, TypePath, parse_quote, parse_quote_spanned,
    punctuated::Punctuated, spanned::Spanned,
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
    let container_generics = ast.item.generics.split_for_impl().1;
    let container_type: Type = parse_quote!(#container_ident #container_generics);

    // Collect all container generics which are only bound by a single trait in order to
    // disambiguate them.
    let bound_generics = ast
        .item
        .generics
        .type_params()
        .filter_map(|param| {
            if param.bounds.len() != 1 {
                return None;
            };

            let Some(TypeParamBound::Trait(bound)) = param.bounds.first() else {
                return None;
            };

            Some((param.ident.clone(), bound.path.clone()))
        })
        .collect::<HashMap<_, _>>();

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
                            ty: qualify_generic(&bound_generics, assertion.ty),
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

/// Traverse all of the path segments, and qualify any nested generics.
fn qualify_path_segments<'a>(
    bound_generics: &'a HashMap<Ident, Path>,
    segments: impl Iterator<Item = &'a mut PathSegment>,
) {
    segments.for_each(|segment| {
        match &mut segment.arguments {
            PathArguments::None => {}
            PathArguments::AngleBracketed(args) => {
                args.args
                    .iter_mut()
                    .filter_map(|arg| {
                        if let GenericArgument::Type(ty) = arg {
                            Some(ty)
                        } else {
                            None
                        }
                    })
                    .for_each(|ty| {
                        *ty = qualify_generic(
                            bound_generics,
                            // Sad clone :(
                            ty.clone(),
                        );
                    });
            }
            PathArguments::Parenthesized(args) => {
                args.inputs.iter_mut().for_each(|ty| {
                    *ty = qualify_generic(
                        bound_generics,
                        // Sad clone :(
                        ty.clone(),
                    );
                });
            }
        };
    });
}

/// Attempt to qualify the provided type if it's detected to be a generic.
fn qualify_generic(bound_generics: &HashMap<Ident, Path>, mut ty: Type) -> Type {
    let segments = match ty {
        // Only accept `Path` types which aren't already qualified and don't have a leading `::`.
        Type::Path(TypePath {
            qself: None,
            path:
                Path {
                    leading_colon: None,
                    ref mut segments,
                },
        }) => segments,
        // Try and recurse to resolve `qself`'s type.
        Type::Path(TypePath {
            qself: Some(mut qself),
            mut path,
        }) => {
            qself.ty = Box::new(qualify_generic(bound_generics, *qself.ty));
            qualify_path_segments(bound_generics, &mut path.segments.iter_mut());
            return TypePath {
                qself: Some(qself),
                path,
            }
            .into();
        }
        // Return everything else as-is.
        _ => {
            return ty;
        }
    };

    qualify_path_segments(bound_generics, segments.iter_mut());

    // There must be two or more segments, otherwise it's just an identifier.
    if segments.len() <= 1 {
        return ty;
    }

    // Pull out the first ident, and see if it's bound to a trait.
    let Some((ident, bound_trait)) = segments
        .first()
        .map(|segment| segment.ident.clone())
        .and_then(|ident| {
            let bound = bound_generics.get(&ident)?;
            Some((ident, bound))
        })
    else {
        return ty;
    };

    // Contruct a new punctuated list of segments, with the first removed.
    let segments: Punctuated<_, Token![::]> =
        std::mem::take(segments).into_iter().skip(1).collect();

    parse_quote_spanned! { ty.span() => <#ident as #bound_trait>::#segments }
}

#[cfg(test)]
mod test {
    use quote::ToTokens;

    use super::*;

    fn test_qualify_generic(ty: Type, expected: Type) {
        let bound_generics = [
            (parse_quote!(T), parse_quote!(MyTrait)),
            (parse_quote!(U), parse_quote!(some_module::OtherTrait)),
        ]
        .into();
        let output = qualify_generic(&bound_generics, ty);
        println!(
            "output: {}, expected: {}",
            output.to_token_stream(),
            expected.to_token_stream()
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn single_ident() {
        test_qualify_generic(parse_quote!(T), parse_quote!(T));
    }

    #[test]
    fn associated_type() {
        test_qualify_generic(parse_quote!(T::Item), parse_quote!(<T as MyTrait>::Item));
    }

    #[test]
    fn nested_associated_type() {
        test_qualify_generic(
            parse_quote!(<T::Item as OtherTrait>::Item),
            parse_quote!(<<T as MyTrait>::Item as OtherTrait>::Item),
        );
    }

    #[test]
    fn super_nested_associated_type() {
        test_qualify_generic(
            parse_quote!(<<T::Item as OtherTrait>::Item as OtherOtherTrait>::Item),
            parse_quote!(<<<T as MyTrait>::Item as OtherTrait>::Item as OtherOtherTrait>::Item),
        );
    }

    #[test]
    fn as_other_generic() {
        test_qualify_generic(
            parse_quote!(SomeStruct<T::Item>),
            parse_quote!(SomeStruct<<T as MyTrait>::Item>),
        );
    }

    #[test]
    fn associated_type_generic() {
        test_qualify_generic(
            parse_quote!(<K as TraitWithGeneric>::Item<T::Item>),
            parse_quote!(<K as TraitWithGeneric>::Item<<T as MyTrait>::Item>),
        );
    }
}
