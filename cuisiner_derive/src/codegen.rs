use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, Index, LitInt};

use crate::{Fields, Ir, ItemIr, Repr, StructGenerics};

pub fn codegen(ir: Ir) -> Result<TokenStream, Error> {
    let Ir {
        crate_name,
        base_ident,
        visibility,
        item,
    } = ir;

    // WARN: Not sure if there's a better way to get this wrapped in quotes...
    let zerocopy_crate = quote!(#crate_name::zerocopy).to_string();

    match item {
        ItemIr::Struct {
            fields,
            raw_ident,
            raw_derives,
            generics,
            container_assert_layout,
        } => {
            let StructGenerics {
                base: base_generics,
                raw: raw_generics,
                b_ident: b_generic_ident,
                b_generic,
            } = &generics;
            let (impl_generics, ty_generics, where_clause) = base_generics.split_for_impl();
            let (_, raw_ty_generics, raw_where_clause) = raw_generics.split_for_impl();

            let container_assert_layout = container_assert_layout.map(|metas| {
                quote! { #[#crate_name::assert_layout(#(#metas,)*)] }
            });

            let (field_definitions, from_raws, to_raws) = match fields {
                Fields::Named(fields) => {
                    let fields_len = fields.len();
                    let (names, tys, assertions) = fields.into_iter().fold(
                        (
                            Vec::with_capacity(fields_len),
                            Vec::with_capacity(fields_len),
                            Vec::with_capacity(fields_len),
                        ),
                        |(mut names, mut tys, mut assertions), (name, ty, assertion)| {
                            names.push(name);
                            tys.push(ty);
                            assertions.push(assertion.map(|metas| {
                                quote! { #[assert_layout(#(#metas,)*)] }
                            }));

                            (names, tys, assertions)
                        },
                    );

                    (
                        quote! {
                            { #(#assertions #names: <#tys as #crate_name::Cuisiner>::Raw::<#b_generic_ident>),* }
                        },
                        quote! {
                            { #(#names: <#tys as #crate_name::Cuisiner>::try_from_raw::<#b_generic_ident>(raw.#names)?),* }
                        },
                        quote! {
                            { #(#names: <#tys as #crate_name::Cuisiner>::try_to_raw::<#b_generic_ident>(self.#names)?),* }
                        },
                    )
                }
                Fields::Unnamed(fields) => {
                    let fields_len = fields.len();
                    let (names, tys, assertions) = fields.iter().enumerate().fold(
                        (
                            Vec::with_capacity(fields_len),
                            Vec::with_capacity(fields_len),
                            Vec::with_capacity(fields_len),
                        ),
                        |(mut names, mut tys, mut assertions), (name, (ty, assertion))| {
                            names.push(Index::from(name));
                            tys.push(ty);
                            assertions.push(assertion.as_ref().map(|metas| {
                                quote! { #[assert_layout(#(#metas,)*)] }
                            }));

                            (names, tys, assertions)
                        },
                    );

                    (
                        quote! {
                            (#(#assertions #tys),*);
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

            Ok(quote! {
                #[derive(#(#raw_derives),*)]
                #[repr(C)]
                #[zerocopy(crate = #zerocopy_crate)]
                #[automatically_derived]
                #container_assert_layout
                #visibility struct #raw_ident #raw_generics #raw_where_clause #field_definitions

                #[automatically_derived]
                impl #impl_generics #crate_name::Cuisiner for #base_ident #ty_generics #where_clause {
                    type Raw<#b_generic> = #raw_ident #raw_ty_generics;

                    fn try_from_raw<#b_generic>(raw: Self::Raw<#b_generic_ident>) -> ::core::result::Result<Self, #crate_name::CuisinerError> {
                        Ok(Self #from_raws)
                    }

                    fn try_to_raw<#b_generic>(self) -> ::core::result::Result<Self::Raw<#b_generic_ident>, #crate_name::CuisinerError> {
                        Ok(Self::Raw #to_raws)
                    }
                }
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
