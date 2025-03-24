use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, LitInt};

use crate::{Fields, Ir, ItemIr, Repr};

pub fn codegen(ir: Ir) -> Result<TokenStream, Error> {
    let Ir {
        crate_name,
        base_ident,
        item,
    } = ir;

    // WARN: Not sure if there's a better way to get this wrapped in quotes...
    let zerocopy_crate = quote!(#crate_name::zerocopy).to_string();

    match item {
        ItemIr::Struct {
            fields,
            raw_ident,
            raw_derives,
            assert_size,
        } => {
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

            let assert_size = assert_size.map(|assert_size| {
                quote! {
                    #crate_name::static_assertions::const_assert_eq!(
                        ::core::mem::size_of::<#raw_ident<#crate_name::zerocopy::BigEndian>>(),
                        #assert_size
                    );
                }
            });

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

                #assert_size
            })
        }
        ItemIr::Enum { variants, repr } => {
            let repr_ty = match repr {
                Repr::U8 => quote!(u8),
                Repr::U16 => quote!(u16),
                Repr::U32 => quote!(u32),
                Repr::U64 => quote!(u64),
                Repr::U128 => quote!(u128),
                Repr::Usize => quote!(usize),
                Repr::I8 => quote!(i8),
                Repr::I16 => quote!(i16),
                Repr::I32 => quote!(i32),
                Repr::I64 => quote!(i64),
                Repr::I128 => quote!(i128),
                Repr::Isize => quote!(isize),
            };
            let raw_repr = quote!(<#repr_ty as #crate_name::Cuisiner>::Raw::<B>);
            let (raw_value, raw_constructor) = match repr {
                Repr::U8 | Repr::I8 => (quote!(raw), None),
                _ => (quote!(raw.get()), Some(quote!(#raw_repr::new))),
            };

            let (variants, discriminants): (Vec<_>, Vec<_>) = variants
                .into_iter()
                .map(|(variant, discriminant)| {
                    (
                        variant,
                        // To prevent `usize` being included in the value, turn it into a literal
                        // value.
                        LitInt::new(&discriminant.to_string(), Span::call_site()),
                    )
                })
                .unzip();

            let invalid_discriminant_message =
                format!("invalid discriminant for {base_ident}: {{}}");

            Ok(quote! {
                #[automatically_derived]
                impl #crate_name::Cuisiner for #base_ident {
                    type Raw<B: #crate_name::zerocopy::ByteOrder> = #raw_repr;

                    fn try_from_raw<B: #crate_name::zerocopy::ByteOrder>(raw: Self::Raw<B>) -> ::core::result::Result<Self, #crate_name::CuisinerError> {
                        match #raw_value {
                            #(#discriminants => ::core::result::Result::Ok(Self::#variants),)*
                            n => ::core::result::Result::Err(#crate_name::CuisinerError::Validation(::std::format!(#invalid_discriminant_message, n))),
                        }
                    }

                    fn try_to_raw<B: #crate_name::zerocopy::ByteOrder>(self) -> ::core::result::Result<Self::Raw<B>, #crate_name::CuisinerError> {
                        ::core::result::Result::Ok(#raw_constructor(match self {
                            #(Self::#variants => #discriminants,)*
                        }))
                    }
                }
            })
        }
    }
}
