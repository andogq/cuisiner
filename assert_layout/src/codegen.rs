use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{Error, spanned::Spanned};

use crate::lower::Ir;

pub fn codegen(ir: Ir) -> Result<TokenStream, Error> {
    let item = ir.item;

    let assertions = ir.assertions.into_iter().map(|(generics, assertions)| {
        // Convert the const expression into a static assertion.
        let assertions = assertions.into_iter().map(|assertion| {
            quote_spanned! {
                assertion.span() =>
                    const _: [(); 0 - !{ const ASSERT: bool = #assertion; ASSERT } as usize] = [];
            }
        });

        quote! {
            const _: () = {
                #(#generics)* #(#assertions)*
            };
        }
    });

    Ok(quote! {
        #item

        #(#assertions)*
    })
}
