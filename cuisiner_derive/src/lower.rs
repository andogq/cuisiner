use proc_macro2::Span;
use syn::{Error, Ident, Path, parse_quote};

use crate::{DeriveModel, Fields};

/// From the provided [`DeriveModel`], generate an [`Ir`] representing it.
pub fn lower(model: DeriveModel) -> Result<Ir, Error> {
    let crate_name = parse_quote!(::cuisiner);

    Ok(Ir {
        base_ident: model.name.clone(),
        raw_ident: Ident::new(&format!("___Cuisiner{}Raw", model.name), Span::call_site()),
        raw_derives: vec![
            parse_quote!(#crate_name::zerocopy::FromBytes),
            parse_quote!(#crate_name::zerocopy::IntoBytes),
            parse_quote!(#crate_name::zerocopy::Immutable),
            parse_quote!(#crate_name::zerocopy::Unaligned),
        ],
        fields: model.fields.clone(),
        crate_name,
    })
}

/// Intermediate representation of the output of the derive macro.
pub struct Ir {
    /// Path of the crate that everything is exported from.
    pub crate_name: Path,
    /// Identifier of the base struct.
    pub base_ident: Ident,
    /// Identifier of the raw struct.
    pub raw_ident: Ident,
    /// Derives to be added to the raw struct.
    pub raw_derives: Vec<Path>,
    /// Fields present in the original struct.
    pub fields: Fields,
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_ir(model: DeriveModel, expected_raw_ident: impl AsRef<str>) {
        let ir = lower(model).unwrap();

        assert_eq!(ir.raw_ident, expected_raw_ident.as_ref());
    }

    #[test]
    fn valid_model() {
        test_ir(
            DeriveModel {
                name: Ident::new("MyStruct", Span::call_site()),
                fields: Fields::Named(vec![(parse_quote!(a), parse_quote!(u64))]),
            },
            "___CuisinerMyStructRaw",
        );
    }
}
