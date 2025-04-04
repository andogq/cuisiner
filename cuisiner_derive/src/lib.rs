mod analyse;
mod codegen;
mod lower;
mod parse;

use proc_macro2::TokenStream;
use syn::{DeriveInput, Error, Ident, Meta, Token, Type, parenthesized, punctuated::Punctuated};

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
    Named(Vec<(Ident, Type, Option<Vec<Meta>>)>),
    /// Unnamed fields ([`syn::FieldsUnnamed`]).
    Unnamed(Vec<(Type, Option<Vec<Meta>>)>),
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

                        let mut assert_layout = None;
                        for attr in &field.attrs {
                            if !attr.path().is_ident("cuisiner") {
                                continue;
                            }

                            attr.parse_nested_meta(|meta| {
                                if meta.path.is_ident("assert") {
                                    // Remove the parenthesis.
                                    let args;
                                    parenthesized!(args in meta.input);

                                    // Fetch the meta items from the attributes.
                                    assert_layout = Some(
                                        Punctuated::<Meta, Token![,]>::parse_terminated(&args)?
                                            .into_iter()
                                            .collect(),
                                    );

                                    return Ok(());
                                }

                                Err(Error::new_spanned(&meta.path, "unknown attribute"))
                            })?;
                        }

                        Ok((ident, ty, assert_layout))
                    })
                    .collect::<Result<_, Error>>()?,
            ),
            syn::Fields::Unnamed(fields_unnamed) => Fields::Unnamed(
                fields_unnamed
                    .unnamed
                    .iter()
                    .map(|field| {
                        Ok((field.ty.clone(), {
                            let mut assert_layout = None;
                            for attr in &field.attrs {
                                if !attr.path().is_ident("cuisiner") {
                                    continue;
                                }

                                attr.parse_nested_meta(|meta| {
                                    if meta.path.is_ident("assert") {
                                        // Remove the parenthesis.
                                        let args;
                                        parenthesized!(args in meta.input);

                                        // Fetch the meta items from the attributes.
                                        assert_layout = Some(
                                            Punctuated::<Meta, Token![,]>::parse_terminated(&args)?
                                                .into_iter()
                                                .collect(),
                                        );

                                        return Ok(());
                                    }

                                    Err(Error::new_spanned(&meta.path, "unknown attribute"))
                                })?;
                            }
                            assert_layout
                        }))
                    })
                    .collect::<Result<_, Error>>()?,
            ),
            syn::Fields::Unit => Fields::Unit,
        })
    }
}
