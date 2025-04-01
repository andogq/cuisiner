use std::collections::HashMap;

use syn::{
    GenericArgument, Ident, Path, PathArguments, PathSegment, Token, Type, TypePath,
    parse_quote_spanned, punctuated::Punctuated, spanned::Spanned,
};

/// Traverse all of the path segments, and qualify any nested generics.
pub fn qualify_path_segments<'a>(
    bound_generics: &HashMap<&Ident, &Path>,
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
pub fn qualify_generic(bound_generics: &HashMap<&Ident, &Path>, mut ty: Type) -> Type {
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
    use syn::parse_quote;

    use super::*;

    #[allow(non_snake_case)]
    fn test_qualify_generic(ty: Type, expected: Type) {
        let T = parse_quote!(T);
        let U = parse_quote!(U);
        let MyTrait = parse_quote!(MyTrait);
        let OtherTrait = parse_quote!(some_module::OtherTrait);

        let bound_generics = [(&T, &MyTrait), (&U, &OtherTrait)].into();
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
