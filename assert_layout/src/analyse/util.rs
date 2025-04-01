use std::collections::HashMap;

use syn::{Generics, Ident, Path, TypeParamBound};

/// Collect all container generics which are only bound by a single trait in order to disambiguate
/// them.
pub fn get_bound_generics(generics: &Generics) -> HashMap<&Ident, &Path> {
    generics
        .type_params()
        .filter_map(|param| {
            if param.bounds.len() != 1 {
                return None;
            };

            let Some(TypeParamBound::Trait(bound)) = param.bounds.first() else {
                return None;
            };

            Some((&param.ident, &bound.path))
        })
        .collect()
}

#[cfg(test)]
mod test {
    use syn::parse_quote;

    use super::*;

    fn test_bound_generics(generics: Generics, expected: impl Into<HashMap<Ident, Path>>) {
        let output = get_bound_generics(&generics);
        assert_eq!(
            output
                .into_iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<HashMap<_, _>>(),
            expected.into()
        );
    }

    #[test]
    fn no_bound() {
        test_bound_generics(parse_quote!(<T>), []);
    }

    #[test]
    fn simple_bound() {
        test_bound_generics(
            parse_quote!(<T: SomeTrait>),
            [(parse_quote!(T), parse_quote!(SomeTrait))],
        );
    }

    #[test]
    fn multiple_bounds() {
        test_bound_generics(parse_quote!(<T: SomeTrait + OtherTrait>), []);
    }

    #[test]
    fn path_bound() {
        test_bound_generics(
            parse_quote!(<T: some::path::SomeTrait>),
            [(parse_quote!(T), parse_quote!(some::path::SomeTrait))],
        );
    }

    #[test]
    fn multiple() {
        test_bound_generics(
            parse_quote!(<T: SomeTrait, U, V: OtherTrait>),
            [
                (parse_quote!(T), parse_quote!(SomeTrait)),
                (parse_quote!(V), parse_quote!(OtherTrait)),
            ],
        );
    }
}
