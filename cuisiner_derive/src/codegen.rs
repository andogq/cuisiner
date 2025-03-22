use proc_macro2::TokenStream;
use quote::quote;
use syn::Error;

use crate::{Endian, Fields, Ir};

pub fn codegen(ir: Ir) -> Result<TokenStream, Error> {
    let Ir {
        crate_name,
        base_ident,
        raw_ident,
        endian,
        raw_derives,
        fields,
    } = ir;

    let endian = match endian {
        Endian::BigEndian => quote! { #crate_name::BigEndian },
        Endian::LittleEndian => quote! { #crate_name::LittleEndian },
        Endian::NetworkEndian => quote! { #crate_name::NetworkEndian },
        Endian::NativeEndian => quote! { #crate_name::NativeEndian },
    };

    let (field_definitions, from_raws, to_raws) = match fields {
        Fields::Named(fields) => {
            let (names, tys): (Vec<_>, Vec<_>) = fields.into_iter().unzip();

            (
                quote! {
                    { #(#names: <#tys as #crate_name::Cuisiner<#endian>>::Raw),* }
                },
                quote! {
                    { #(#names: <#tys as #crate_name::Cuisiner<#endian>>::try_from_raw(raw.#names)?),* }
                },
                quote! {
                    { #(#names: <#tys as #crate_name::Cuisiner<#endian>>::try_to_raw(self.#names)?),* }
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
                    (#(<#tys as #crate_name::Cuisiner<#endian>>::try_from_raw(raw.#names)?),*);
                },
                quote! {
                    (#(<#tys as #crate_name::Cuisiner<#endian>>::try_to_raw(self.#names)?),*);
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
        struct #raw_ident #field_definitions

        impl #crate_name::Cuisiner<#endian> for #base_ident {
            type Raw = #raw_ident;

            fn try_from_raw(raw: Self::Raw) -> ::core::result::Result<Self, #crate_name::CuisinerError> {
                Ok(Self #from_raws)
            }

            fn try_to_raw(self) -> ::core::result::Result<Self::Raw, #crate_name::CuisinerError> {
                Ok(Self::Raw #to_raws)
            }
        }
    })
}
