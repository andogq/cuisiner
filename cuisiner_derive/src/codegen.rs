use proc_macro2::TokenStream;
use quote::quote;
use syn::Error;

use crate::{Fields, Ir};

pub fn codegen(ir: Ir) -> Result<TokenStream, Error> {
    let Ir {
        crate_name,
        base_ident,
        raw_ident,
        raw_derives,
        fields,
    } = ir;

    let (field_definitions, from_raws, to_raws) = match fields {
        Fields::Named(fields) => {
            let (names, tys): (Vec<_>, Vec<_>) = fields.into_iter().unzip();

            (
                quote! {
                    { #(#names: <#tys as #crate_name::Cuisiner>::Raw::<B>),* }
                },
                quote! {
                    { #(#names: <#tys as #crate_name::Cuisiner>::try_from_raw::<B>(raw.#names)?),* }
                },
                quote! {
                    { #(#names: <#tys as #crate_name::Cuisiner>::try_to_raw::<B>(self.#names)?),* }
                },
            )
        }
        Fields::Unnamed(fields) => {
            let (names, tys): (Vec<_>, Vec<_>) = fields.into_iter().enumerate().unzip();

            (
                quote! {
                    (#(#tys),*);
                },
                quote! {
                    (#(<#tys as #crate_name::Cuisiner>::try_from_raw(raw.#names)?),*);
                },
                quote! {
                    (#(<#tys as #crate_name::Cuisiner>::try_to_raw(self.#names)?),*);
                },
            )
        }
        Fields::Unit => (quote!(;), quote!(;), quote!(;)),
    };

    // WARN: Not sure if there's a better way to get this wrapped in quotes...
    let zerocopy_crate = quote!(#crate_name::zerocopy).to_string();

    Ok(quote! {
        #[derive(#(#raw_derives),*)]
        #[repr(C)]
        #[zerocopy(crate = #zerocopy_crate)]
        #[automatically_derived]
        struct #raw_ident<B: #crate_name::zerocopy::ByteOrder> #field_definitions

        #[automatically_derived]
        impl #crate_name::Cuisiner for #base_ident {
            type Raw<B: #crate_name::zerocopy::ByteOrder> = #raw_ident<B>;

            fn try_from_raw<B: #crate_name::zerocopy::ByteOrder>(raw: Self::Raw<B>) -> ::core::result::Result<Self, #crate_name::CuisinerError> {
                Ok(Self #from_raws)
            }

            fn try_to_raw<B: #crate_name::zerocopy::ByteOrder>(self) -> ::core::result::Result<Self::Raw<B>, #crate_name::CuisinerError> {
                Ok(Self::Raw #to_raws)
            }
        }
    })
}
