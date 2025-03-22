mod analyse;
mod codegen;
mod lower;
mod parse;

use proc_macro2::TokenStream;
use syn::{DeriveInput, Error, Ident, Type};

use self::{analyse::*, codegen::*, lower::*, parse::*};

#[proc_macro_derive(Cuisiner)]
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
    Named(Vec<(Ident, Type)>),
    /// Unnamed fields ([`syn::FieldsUnnamed`]).
    Unnamed(Vec<Type>),
    /// No fields ([`syn::Fields::Unit`]).
    Unit,
}

impl From<&syn::Fields> for Fields {
    fn from(fields: &syn::Fields) -> Self {
        match fields {
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

                        (ident, ty)
                    })
                    .collect(),
            ),
            syn::Fields::Unnamed(fields_unnamed) => Fields::Unnamed(
                fields_unnamed
                    .unnamed
                    .iter()
                    .map(|field| field.ty.clone())
                    .collect(),
            ),
            syn::Fields::Unit => Fields::Unit,
        }
    }
}
