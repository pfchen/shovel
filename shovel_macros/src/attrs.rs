use syn::{
    Attribute, Expr, ExprLit, Lit, Meta, MetaNameValue, spanned::Spanned,
};

pub(crate) fn doc_parse(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| match &attr.meta {
            Meta::NameValue(MetaNameValue {
                value:
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(str), ..
                    }),
                ..
            }) => Some(str.value()),
            _ => None,
        })
        .map(|s| s.trim().replace("\n", " "))
        .skip_while(|s| s.trim().is_empty())
        .fold(String::new(), |acc, s| {
            if acc.is_empty() { s } else { acc + " " + &s }
        })
}

pub(crate) fn short_parse(attr: &Attribute) -> syn::Result<Option<char>> {
    match &attr.meta {
        Meta::NameValue(MetaNameValue {
            value:
                Expr::Lit(ExprLit {
                    lit: Lit::Char(c), ..
                }),
            ..
        }) => Ok(Some(c.value())),
        Meta::Path(path) => {
            if path.is_ident("short") {
                Ok(None)
            } else {
                Err(syn::Error::new(attr.span(), "Invalid attribute 'format'"))
            }
        }
        _ => Err(syn::Error::new(attr.span(), "Invalid attribute 'format'")),
    }
}

pub(crate) fn long_parse(attr: &Attribute) -> syn::Result<String> {
    match &attr.meta {
        Meta::NameValue(MetaNameValue {
            value:
                Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }),
            ..
        }) => Ok(s.value().trim().to_owned()),
        _ => Err(syn::Error::new(
            attr.span(),
            "'long' should be a name-value attribute",
        )),
    }
}

pub(crate) fn format_parse(attr: &Attribute) -> syn::Result<String> {
    match &attr.meta {
        Meta::NameValue(MetaNameValue {
            value:
                Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }),
            ..
        }) => Ok(s.value().trim().to_owned()),
        _ => Err(syn::Error::new(
            attr.span(),
            "'format' should be a name-value attribute",
        )),
    }
}
