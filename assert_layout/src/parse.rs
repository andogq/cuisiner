use std::ops::{Deref, DerefMut};

use proc_macro2::Span;
use syn::{
    Error, Expr, ExprLit, GenericArgument, ItemStruct, Lit, Member, Meta, Token, Type,
    punctuated::Punctuated, spanned::Spanned,
};

pub fn parse(attrs: impl IntoIterator<Item = Meta>, item: ItemStruct) -> Result<Ast, Error> {
    let mut ast = Ast {
        item,
        size: None,
        generics: Vec::new(),
        field_assertions: Vec::new(),
    };

    // Process attributes on fields.
    ast.item
        .fields
        .iter_mut()
        .enumerate()
        .try_for_each(|(i, field)| {
            let field_ident = field
                .ident
                .as_ref()
                .map(|ident| Member::from(ident.clone()))
                .unwrap_or_else(|| Member::from(i));

            field.attrs = std::mem::take(&mut field.attrs).into_iter().try_fold(
                Vec::new(),
                |mut attrs, attr| {
                    if !attr.path().is_ident("assert_layout") {
                        attrs.push(attr);
                        return Ok::<_, Error>(attrs);
                    }

                    let mut assertion = FieldAssertion {
                        field: field_ident.clone(),
                        ty: field.ty.clone(),
                        size: None,
                        offset: None,
                    };

                    attr.parse_nested_meta(|meta| {
                        match meta.path.require_ident()?.to_string().as_str() {
                            "size" => {
                                assertion.size = Some(meta.value()?.parse()?);
                            }
                            "offset" => {
                                assertion.offset = Some(meta.value()?.parse()?);
                            }
                            attr_name => {
                                return Err(Error::new_spanned(
                                    meta.path,
                                    format!("unknown attribute: {attr_name}"),
                                ));
                            }
                        }

                        Ok(())
                    })?;

                    ast.field_assertions.push(assertion);

                    Ok(attrs)
                },
            )?;

            Ok::<_, Error>(())
        })?;

    for attr in attrs.into_iter() {
        let attr = attr.require_name_value()?;

        match attr.path.require_ident()?.to_string().as_str() {
            "size" => {
                ast.size = Some(attr.value.clone());
            }
            "generics" => {
                let Expr::Lit(ExprLit {
                    lit: Lit::Str(ref lit_str),
                    ..
                }) = attr.value
                else {
                    return Err(Error::new_spanned(
                        &attr.value,
                        "expected generics in string",
                    ));
                };

                ast.generics.push(WithSpan::new(
                    lit_str
                        .parse_with(Punctuated::<GenericArgument, Token![,]>::parse_terminated)?
                        .into_iter()
                        .collect(),
                    lit_str,
                ));
            }
            attr_name => {
                return Err(Error::new_spanned(
                    &attr.path,
                    format!("unknown attribute: {attr_name}"),
                ));
            }
        }
    }

    Ok(ast)
}

pub struct Ast {
    /// Item that the macro was called on.
    pub item: ItemStruct,
    /// Expected size of the item.
    pub size: Option<Expr>,
    /// Generics provided for assertions.
    pub generics: Vec<WithSpan<Vec<GenericArgument>>>,
    pub field_assertions: Vec<FieldAssertion>,
}

pub struct FieldAssertion {
    pub field: Member,
    pub ty: Type,
    pub size: Option<Expr>,
    pub offset: Option<Expr>,
}

pub struct WithSpan<T> {
    value: T,
    span: Span,
}

impl<T> WithSpan<T> {
    pub fn new(value: T, span: impl Spanned) -> Self {
        Self {
            value,
            span: span.span(),
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl<T> Deref for WithSpan<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for WithSpan<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
