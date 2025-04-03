use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use proc_macro2::Span;
use syn::{
    Error, Expr, ExprLit, GenericArgument, ItemStruct, Lit, Member, Meta, Token, Type,
    meta::ParseNestedMeta, punctuated::Punctuated, spanned::Spanned,
};

pub fn parse(attrs: impl IntoIterator<Item = Meta>, item: ItemStruct) -> Result<Ast, Error> {
    let mut ast = Ast {
        item,
        namespaces: HashMap::new(),
    };

    let (default_namespace, nested) = parse_namespace(attrs)?;

    ast.namespaces.insert("".to_string(), default_namespace);
    ast.namespaces.extend(nested);

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

                    let mut default_assertion = FieldAssertion {
                        field: field_ident.clone(),
                        ty: field.ty.clone(),
                        size: None,
                        offset: None,
                    };

                    fn parse_attr(
                        assertion: &mut FieldAssertion,
                        meta: &ParseNestedMeta,
                    ) -> Result<(), Error> {
                        match meta.path.require_ident()?.to_string().as_str() {
                            "size" => {
                                assertion.size = Some(meta.value()?.parse()?);
                            }
                            "offset" => {
                                assertion.offset = Some(meta.value()?.parse()?);
                            }
                            attr_name => {
                                return Err(Error::new_spanned(
                                    &meta.path,
                                    format!("unknown attribute: {attr_name}"),
                                ));
                            }
                        }

                        Ok(())
                    }

                    attr.parse_nested_meta(|meta| {
                        // Try parse default assertion
                        if parse_attr(&mut default_assertion, &meta).is_ok() {
                            return Ok(());
                        }

                        // Attempt to pull out the namespace
                        let namespace_key = meta.path.require_ident()?.to_string();
                        let namespace =
                            ast.namespaces
                                .get_mut(&namespace_key)
                                .ok_or(Error::new_spanned(
                                    &meta.path,
                                    format!("unknown namespace: {namespace_key}"),
                                ))?;

                        let mut assertion = FieldAssertion {
                            field: default_assertion.field.clone(),
                            ty: default_assertion.ty.clone(),
                            size: None,
                            offset: None,
                        };

                        meta.parse_nested_meta(|meta| parse_attr(&mut assertion, &meta))?;

                        namespace.field_assertions.push(assertion);

                        Ok(())
                    })?;

                    ast.namespaces
                        .entry("".to_string())
                        .or_default()
                        .field_assertions
                        .push(default_assertion);

                    Ok(attrs)
                },
            )?;

            Ok::<_, Error>(())
        })?;

    Ok(ast)
}

fn parse_namespace(
    attrs: impl IntoIterator<Item = Meta>,
) -> Result<(Namespace, Vec<(String, Namespace)>), Error> {
    let mut namespace = Namespace::default();
    let mut nested = Vec::new();

    for attr in attrs {
        match attr {
            Meta::Path(path) => todo!(),
            Meta::List(attr) => {
                let (namespace, deeply_nested) = parse_namespace(
                    attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?,
                )?;

                if !deeply_nested.is_empty() {
                    return Err(Error::new_spanned(attr, "cannot deeply nest namespaces"));
                };

                nested.push((attr.path.require_ident()?.to_string(), namespace));
            }
            Meta::NameValue(attr) => match attr.path.require_ident()?.to_string().as_str() {
                "size" => {
                    namespace.size = Some(attr.value.clone());
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

                    namespace.generics.push(WithSpan::new(
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
            },
        }
    }

    Ok((namespace, nested))
}

pub struct Ast {
    /// Item that the macro was called on.
    pub item: ItemStruct,
    pub namespaces: HashMap<String, Namespace>,
}

#[derive(Default)]
pub struct Namespace {
    /// Expected size of the item.
    pub size: Option<Expr>,
    /// Generics provided for assertions.
    pub generics: Vec<WithSpan<Vec<GenericArgument>>>,
    /// Field assertions.
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
