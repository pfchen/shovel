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
                Err(syn::Error::new(attr.span(), "Invalid attribute 'short'"))
            }
        }
        _ => Err(syn::Error::new(attr.span(), "Invalid attribute 'short'")),
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

pub(crate) fn env_parse(attr: &Attribute) -> syn::Result<String> {
    let name = match &attr.meta {
        Meta::NameValue(MetaNameValue {
            value:
                Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }),
            ..
        }) => s.value(),
        _ => {
            return Err(syn::Error::new(
                attr.span(),
                "'env' should be a name-value string attribute, e.g. #[env = \"NAME\"]",
            ));
        }
    };

    if name.is_empty() {
        return Err(syn::Error::new(
            attr.span(),
            "environment variable name must not be empty",
        ));
    }
    if !name.is_ascii() {
        return Err(syn::Error::new(
            attr.span(),
            "environment variable name must be ASCII",
        ));
    }
    if name.contains('=') || name.contains('\0') {
        return Err(syn::Error::new(
            attr.span(),
            "environment variable name must not contain '=' or NUL",
        ));
    }
    if name.trim() != name {
        return Err(syn::Error::new(
            attr.span(),
            "environment variable name must not have leading or trailing whitespace",
        ));
    }

    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn env_parse_accepts_valid_name() {
        let attr: Attribute = parse_quote!(#[env = "PORT"]);
        assert_eq!(env_parse(&attr).unwrap(), "PORT");
    }

    #[test]
    fn env_parse_rejects_invalid_names() {
        for attr in [
            parse_quote!(#[env = ""]),
            parse_quote!(#[env = "PÖRT"]),
            parse_quote!(#[env = "A=B"]),
            parse_quote!(#[env = "A\0B"]),
            parse_quote!(#[env = " PORT"]),
        ] {
            assert!(env_parse(&attr).is_err());
        }
    }

    #[test]
    fn env_parse_rejects_wrong_shape() {
        let attr: Attribute = parse_quote!(#[env = 123]);
        assert!(env_parse(&attr).is_err());
    }
}
