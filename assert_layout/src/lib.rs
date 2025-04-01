mod analyse;
mod codegen;
mod lower;
mod parse;

use self::{analyse::analyse, codegen::codegen, lower::lower, parse::parse};
use proc_macro::TokenStream;
use syn::{Error, ItemStruct, Meta, Token, punctuated::Punctuated};

#[proc_macro_attribute]
pub fn assert_layout(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = syn::parse_macro_input!(attr with Punctuated<Meta, Token![,]>::parse_terminated);
    let item = syn::parse_macro_input!(item as ItemStruct);
    match assert_layout_inner(attrs, item) {
        Ok(ts) => ts,
        Err(e) => e.into_compile_error(),
    }
    .into()
}

fn assert_layout_inner(
    attrs: impl IntoIterator<Item = Meta>,
    item: ItemStruct,
) -> Result<proc_macro2::TokenStream, Error> {
    let ast = parse(attrs, item)?;
    let model = analyse(ast)?;
    let ir = lower(model)?;
    codegen(ir)
}
