mod analyse;
mod codegen;
mod lower;
mod parse;

use proc_macro2::TokenStream;
use syn::{Attribute, DeriveInput, Error, Expr, Ident, Meta, Type};

use self::{analyse::*, codegen::*, lower::*, parse::*};

#[proc_macro_derive(Cuisiner, attributes(cuisiner))]
pub fn derive_cuisiner(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the token stream.
    let derive_input = syn::parse_macro_input!(ts as DeriveInput);

    // Run the inner implementation, and handle `Error` cases.
    match derive_cuisiner_inner(derive_input) {
        Ok(ts) => ts,
        Result::Err(e) => e.into_compile_error(),
    }
    .into()
}

/// Inner implementation of the derive, which simply glues together each of the different stages of
/// the macro.
fn derive_cuisiner_inner(derive_input: DeriveInput) -> Result<TokenStream, Error> {
    let ast = parse(derive_input)?;
    let model = analyse(ast)?;
    let ir = lower(model)?;
    codegen(ir)
}

/// All availble field representations. Similar to [`syn::Fields`].
#[derive(Clone)]
enum Fields {
    /// Named fields ([`syn::FieldsNamed`]).
    Named(Vec<(Ident, Type, FieldAssertions)>),
    /// Unnamed fields ([`syn::FieldsUnnamed`]).
    Unnamed(Vec<(Type, FieldAssertions)>),
    /// No fields ([`syn::Fields::Unit`]).
    Unit,
}

impl TryFrom<&syn::Fields> for Fields {
    type Error = Error;

    fn try_from(fields: &syn::Fields) -> Result<Self, Self::Error> {
        Ok(match fields {
            syn::Fields::Named(fields_named) => Fields::Named(
                fields_named
                    .named
                    .iter()
                    .map(|field| {
                        let ident = field
                            .ident
                            .clone()
                            .expect("named struct field must have ident");
                        let ty = field.ty.clone();
                        let field_assertions = FieldAssertions::try_from(field.attrs.as_slice())?;

                        Ok((ident, ty, field_assertions))
                    })
                    .collect::<Result<_, Error>>()?,
            ),
            syn::Fields::Unnamed(fields_unnamed) => Fields::Unnamed(
                fields_unnamed
                    .unnamed
                    .iter()
                    .map(|field| {
                        Ok((
                            field.ty.clone(),
                            FieldAssertions::try_from(field.attrs.as_slice())?,
                        ))
                    })
                    .collect::<Result<_, Error>>()?,
            ),
            syn::Fields::Unit => Fields::Unit,
        })
    }
}

/// Optional assertions that can be applied to a field.
#[derive(Clone, Default)]
struct FieldAssertions {
    /// Size of the field.
    size: Option<Expr>,
    /// Offset of the field.
    offset: Option<Expr>,
}

impl TryFrom<&[Attribute]> for FieldAssertions {
    type Error = Error;

    fn try_from(attrs: &[Attribute]) -> Result<Self, Self::Error> {
        let mut assertions = Self::default();

        for attr in attrs {
            if !attr.path().is_ident("cuisiner") {
                continue;
            }

            let Meta::List(list) = &attr.meta else {
                continue;
            };

            list.parse_nested_meta(|meta| {
                if meta.path.is_ident("size") {
                    assertions.size = Some(meta.value()?.parse()?);
                    return Ok(());
                }

                if meta.path.is_ident("offset") {
                    assertions.offset = Some(meta.value()?.parse()?);
                    return Ok(());
                }

                Err(Error::new_spanned(meta.path, "unknown attribute"))
            })?;
        }

        Ok(assertions)
    }
}
