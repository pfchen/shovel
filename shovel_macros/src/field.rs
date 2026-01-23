use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Field, Fields,
    FieldsNamed, FieldsUnnamed, GenericArgument, Ident, LitChar, LitStr, Path,
    PathArguments, Type, Variant, spanned::Spanned,
};

use crate::attrs::{doc_parse, format_parse, long_parse, short_parse};

pub(crate) struct TypeInfo<'a> {
    pub(crate) ty: &'a Type,
    pub(crate) required: bool,
    pub(crate) flag: bool,
    pub(crate) name: String,
    pub(crate) is_list: bool,
}

pub(crate) struct OptField<'a> {
    pub(crate) ident: Ident,
    pub(crate) ty_info: TypeInfo<'a>,
    pub(crate) short: Option<char>,
    pub(crate) long: String,
    pub(crate) format: Option<String>,
    pub(crate) description: String,
}

pub(crate) struct ArgField<'a> {
    pub(crate) ident: Option<Ident>,
    pub(crate) format: Option<String>,
    pub(crate) ty_info: TypeInfo<'a>,
    pub(crate) description: String,
}

pub(crate) struct ArgGroup<'a> {
    pub(crate) ident: Ident,
    pub(crate) fields: Vec<ArgField<'a>>,
    pub(crate) description: String,
    pub(crate) format: Option<String>,
    pub(crate) variadic: bool,
}

pub(crate) enum ArgGroups<'a> {
    Struct(ArgGroup<'a>),
    Enum {
        ident: Ident,
        groups: Vec<ArgGroup<'a>>,
    },
}

pub(crate) struct Opts<'a> {
    pub(crate) fields: Vec<OptField<'a>>,
}

impl<'a> OptField<'a> {
    pub(crate) fn parse(field: &'a Field) -> syn::Result<Self> {
        let ident = field.ident.clone().unwrap();
        let description = doc_parse(&field.attrs);
        let mut short = None;
        let mut long = None;
        let mut format = None;

        for attr in &field.attrs {
            if attr.path().is_ident("short") {
                short = match short_parse(attr) {
                    Ok(v) => match v {
                        None => Some(ident.to_string().chars().next().unwrap()),
                        Some(c) => Some(c),
                    },
                    Err(e) => Err(e)?,
                }
            } else if attr.path().is_ident("format") {
                format = Some(format_parse(attr)?);
            } else if attr.path().is_ident("long") {
                long = Some(long_parse(attr)?);
            }
        }

        let ty_info = type_parse(&field.ty)?;
        let long = long.unwrap_or_else(|| to_kebab_case(&ident.to_string()));

        Ok(Self {
            ident,
            ty_info,
            short,
            long,
            format,
            description,
        })
    }
}

impl<'a> Opts<'a> {
    pub(crate) fn parse(input: &'a DeriveInput) -> syn::Result<Self> {
        match &input.data {
            Data::Struct(DataStruct {
                fields: Fields::Named(fields),
                ..
            }) => {
                let mut v = Vec::new();
                let mut short_chars: std::collections::HashMap<char, Ident> =
                    std::collections::HashMap::new();
                let mut long_names: std::collections::HashMap<String, Ident> =
                    std::collections::HashMap::new();
                for f in fields.named.iter() {
                    let field = OptField::parse(f)?;
                    // Check for duplicate short options
                    let field_name = field.ident.to_string();
                    if let Some(existing) = long_names.get(&field.long) {
                        let mut err = syn::Error::new_spanned(
                            &field.ident,
                            format!(
                                "duplicate long option '--{}' for fields '{}' and '{}'.\n\
                                 Note: Set an explicit long option on one of them, e.g. #[long = \"...\"]",
                                field.long, existing, field_name
                            ),
                        );
                        err.combine(syn::Error::new_spanned(
                            existing,
                            "first use of this long option here",
                        ));
                        return Err(err);
                    }
                    long_names.insert(field.long.clone(), field.ident.clone());
                    if let Some(short) = field.short {
                        if let Some(existing) = short_chars.get(&short) {
                            let mut err = syn::Error::new_spanned(
                                &field.ident,
                                format!(
                                    "duplicate short option '-{}' for fields '{}' and '{}'.\n\
                                     Note: Set an explicit short option on one of them, e.g. #[short = 'x']",
                                    short, existing, field_name
                                ),
                            );
                            err.combine(syn::Error::new_spanned(
                                existing,
                                "first use of this short option here",
                            ));
                            return Err(err);
                        }
                        short_chars.insert(short, field.ident.clone());
                    }
                    v.push(field);
                }
                Ok(Self { fields: v })
            }
            _ => Err(syn::Error::new(
                input.span(),
                "`#[derive(Opts)]` only supports named structs",
            ))?,
        }
    }

    pub(crate) fn init_fields(&self) -> Vec<proc_macro2::TokenStream> {
        let mut stmts = Vec::new();
        for f in &self.fields {
            let ident = f.ident.clone();
            if f.ty_info.flag {
                stmts.push(quote! {
                    let mut #ident = false;
                });
            } else {
                stmts.push(quote! {
                    let mut #ident = None;
                });
            }
        }
        stmts
    }

    pub(crate) fn matches(&self) -> Vec<proc_macro2::TokenStream> {
        let mut matches = Vec::new();
        for f in &self.fields {
            let ident = f.ident.clone();
            let lit = LitStr::new(&f.long, f.ident.span());
            let ty = f.ty_info.ty;
            let match_arm = match f.short {
                Some(c) if f.long != c.to_string() => {
                    let short_lit =
                        LitStr::new(&c.to_string(), Span::call_site());
                    quote! { #lit | #short_lit }
                }
                _ => {
                    quote! { #lit }
                }
            };

            if f.ty_info.is_list {
                // Handle list types like Vec<String> or Option<Vec<String>>
                matches.push(quote! {
                    #match_arm => {
                        match opt.1 {
                            None => {
                                Err(::shovel::ParseError::MissingValueForOption { name: #lit })?;
                            },
                            Some(s) => {
                                // For list types, we need to parse comma-separated values
                                let mut v = Vec::new();
                                for part in s.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                                    v.push(part.parse::<#ty>().map_err(|_| {
                                        ::shovel::ParseError::InvalidOptionValue {
                                            name: #lit,
                                            value: part,
                                            expected: stringify!(#ty)
                                        }
                                    })?);
                                }
                                match &mut #ident {
                                    None => {
                                        #ident = Some(v);
                                    },
                                    Some(existing) => {
                                        existing.extend(v);
                                    }
                                }
                            }
                        }
                    }
                });
            } else if !f.ty_info.required {
                matches.push(quote! {
                    #match_arm => {
                        match opt.1 {
                            None => {
                                Err(::shovel::ParseError::MissingValueForOption { name: #lit })?;
                            },
                            Some(s) => {
                                #ident = Some(s.parse::<#ty>().map_err(|_| {
                                    ::shovel::ParseError::InvalidOptionValue {
                                        name: #lit,
                                        value: s,
                                        expected: stringify!(#ty)
                                    }
                                })?);
                            }
                        }
                    }
                });
            } else if f.ty_info.flag {
                matches.push(quote! {
                    #match_arm => {
                        match opt.1 {
                            None => {
                                #ident = true;
                            },
                            Some(s) => {
                                Err(::shovel::ParseError::UnexpectedFlagValue{
                                    name: #lit,
                                    value: s
                                })?;
                            }
                        }
                    }
                });
            } else {
                matches.push(quote! {
                    #match_arm => {
                        match opt.1 {
                            None => {
                                Err(::shovel::ParseError::MissingValueForOption { name: #lit })?;
                            },
                            Some(s) => {
                                #ident = Some(s.parse::<#ty>().map_err(|_| {
                                    ::shovel::ParseError::InvalidOptionValue{
                                        name: #lit,
                                        value: s,
                                        expected: stringify!(#ty)
                                    }
                                })?);
                            }
                        }
                    }
                });
            }
        }

        matches
    }

    pub(crate) fn required_stmts(&self) -> Vec<proc_macro2::TokenStream> {
        let mut stmts = Vec::new();
        for f in &self.fields {
            let ident = f.ident.clone();
            let lit = LitStr::new(&f.long, Span::call_site());
            if !f.ty_info.flag && f.ty_info.required {
                stmts.push(quote! {
                    let #ident = match #ident {
                        None => Err(::shovel::ParseError::MissingRequiredOption { name: #lit })?,
                        Some(v) => v,
                    };
                })
            }
        }
        stmts
    }
}

impl<'a> ArgField<'a> {
    pub(crate) fn parse(field: &'a Field) -> syn::Result<Self> {
        let ty_info = type_parse(&field.ty)?;
        let ident = field.ident.clone();
        let mut format = None;
        let description = doc_parse(&field.attrs);
        for attr in &field.attrs {
            if attr.path().is_ident("format") {
                format = Some(format_parse(attr)?);
                break;
            }
        }
        Ok(Self {
            ident,
            format,
            ty_info,
            description,
        })
    }
}

impl<'a> ArgGroup<'a> {
    pub(crate) fn parse_from_variant(
        variant: &'a Variant,
        is_last: bool,
    ) -> syn::Result<Self> {
        let ident = variant.ident.clone();
        let mut description = doc_parse(&variant.attrs);
        let mut fields = Vec::new();
        for f in &variant.fields {
            fields.push(ArgField::parse(f)?);
        }
        let format = Self::parse_format(&variant.attrs)?;

        let variadic = is_last
            && !fields.is_empty()
            && fields[fields.len() - 1].ty_info.is_list;

        // Generate description from fields if empty
        if description.is_empty() {
            description = Self::get_description(&fields);
        }

        Ok(Self {
            ident,
            fields,
            description,
            format,
            variadic,
        })
    }

    pub(crate) fn parse_from_struct(
        data: &'a DataStruct,
        attrs: &'a [Attribute],
        ident: &Ident,
    ) -> syn::Result<Self> {
        let mut description = doc_parse(attrs);
        let ident = ident.clone();

        let mut fields = Vec::new();
        match &data.fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                for f in named {
                    fields.push(ArgField::parse(f)?);
                }
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                for f in unnamed {
                    fields.push(ArgField::parse(f)?);
                }
            }
            _ => {}
        }

        let format = Self::parse_format(attrs)?;
        let variadic =
            !fields.is_empty() && fields[fields.len() - 1].ty_info.is_list;

        // Generate description from fields if empty
        if description.is_empty() {
            description = Self::get_description(&fields);
        }

        Ok(Self {
            ident,
            fields,
            description,
            format,
            variadic,
        })
    }

    fn parse_format(attrs: &[Attribute]) -> syn::Result<Option<String>> {
        let mut format = None;
        for attr in attrs {
            if attr.path().is_ident("format") {
                format = Some(format_parse(attr)?);
                break;
            }
        }
        Ok(format)
    }

    /// Generate a description by merging field descriptions.
    /// Format: "<field1> <field2> ..." like command-line syntax
    fn get_description(fields: &[ArgField<'_>]) -> String {
        let parts: Vec<String> = fields
            .iter()
            .map(|f| {
                if !f.description.is_empty() {
                    format!("<{}>", f.description)
                } else if let Some(ref ident) = f.ident {
                    format!("<{}>", ident)
                } else {
                    format!("<{}>", f.ty_info.name)
                }
            })
            .collect();

        if parts.is_empty() {
            String::new()
        } else {
            format!("[{}]", parts.join(" "))
        }
    }

    fn get_format(&self) -> String {
        if !self.fields.is_empty() {
            if self.fields[0].ident.is_some() {
                "[".to_owned()
                    + &self
                        .fields
                        .iter()
                        .map(|f| match &f.format {
                            None => {
                                f.ident.clone().unwrap().to_string()
                                    + ":"
                                    + &f.ty_info.name
                            }
                            Some(s) => s.to_string(),
                        })
                        .fold(String::new(), |acc, s| {
                            if acc.is_empty() {
                                "<".to_owned() + &s + ">"
                            } else {
                                acc + ", <" + &s + ">"
                            }
                        })
                    + "]"
            } else {
                "[".to_owned()
                    + &self.fields.iter().map(|f| &f.ty_info.name).fold(
                        String::new(),
                        |acc, s| {
                            if acc.is_empty() {
                                "<".to_owned() + s + ">"
                            } else {
                                acc + " <" + s + ">"
                            }
                        },
                    )
                    + "]"
            }
        } else {
            "[]".to_owned()
        }
    }
}

impl<'a> ArgGroups<'a> {
    pub(crate) fn parse(input: &'a syn::DeriveInput) -> syn::Result<Self> {
        match &input.data {
            Data::Struct(data) => {
                let group = ArgGroup::parse_from_struct(
                    data,
                    &input.attrs,
                    &input.ident,
                )?;
                Ok(ArgGroups::Struct(group))
            }
            Data::Enum(DataEnum { variants, .. }) => {
                let mut groups = Vec::new();
                let ident = input.ident.clone();
                for (i, variant) in variants.iter().enumerate() {
                    groups.push(ArgGroup::parse_from_variant(
                        variant,
                        i == variants.len() - 1,
                    )?);
                }
                Ok(ArgGroups::Enum { ident, groups })
            }
            _ => Err(syn::Error::new(
                input.span(),
                "`#[derive(Args)]` only supports structs and enums",
            )),
        }
    }

    pub(crate) fn matches(&self) -> Vec<proc_macro2::TokenStream> {
        match self {
            ArgGroups::Struct(group) => {
                let ident = &group.ident;
                let mut fields = Vec::new();
                let mut variadic = false;
                for (i, f) in group.fields.iter().enumerate() {
                    let name = f.ident.clone().unwrap();
                    let ty = &f.ty_info.ty;
                    // Last field
                    if i == group.fields.len() - 1 && f.ty_info.is_list {
                        variadic = true;
                        fields.push(quote! {
                            #name: {
                                let mut v = Vec::new();
                                for (idx, arg) in args[#i..].iter().enumerate() {
                                    v.push(arg.parse::<#ty>().map_err(|_| {
                                        ::shovel::ParseError::InvalidArgumentValue {
                                            index: #i + idx,
                                            value: arg,
                                            expected: stringify!(#ty)
                                        }
                                    })?);
                                }
                                v
                            }
                        });
                    } else {
                        fields.push(quote! {
                            #name: args[#i].parse::<#ty>().map_err(|_| {
                                ::shovel::ParseError::InvalidArgumentValue {
                                    index: #i,
                                    value: args[#i],
                                    expected: stringify!(#ty)
                                }
                            })?
                        });
                    }
                }

                let n = group.fields.len();
                if variadic {
                    vec![quote! {
                        l if l >= (#n - 1) => #ident {
                            #(#fields),*
                        },
                    }]
                } else {
                    vec![quote! {
                        #n => #ident {
                            #(#fields),*
                        },
                    }]
                }
            }
            ArgGroups::Enum { ident, groups } => {
                Self::enum_matches(ident, groups)
            }
        }
    }

    fn enum_matches(
        enum_name: &Ident,
        groups: &[ArgGroup],
    ) -> Vec<proc_macro2::TokenStream> {
        groups
            .iter()
            .enumerate()
            .map(|(n, group)| {
                let is_last_group = n == groups.len() - 1;
                let mut variadic = false;
                let fields = match group.fields.is_empty() {
                    true => quote!(),
                    false => match group.fields[0].ident.is_some() {
                        true => {
                            let mut fields = Vec::new();
                            for (i, f) in group.fields.iter().enumerate() {
                                let ty = &f.ty_info.ty;
                                let name = &f.ident.clone().unwrap();
                                if is_last_group && i == group.fields.len() - 1 && f.ty_info.is_list {
                                    variadic = true;
                                    fields.push(quote! {
                                        #name: {
                                            let mut v = Vec::new();
                                            for (idx, arg) in args[#i..].iter().enumerate() {
                                                v.push(arg.parse::<#ty>().map_err(|_| {
                                                    ::shovel::ParseError::InvalidArgumentValue {
                                                        index: #i + idx,
                                                        value: arg,
                                                        expected: stringify!(#ty)
                                                    }
                                                })?);
                                            }
                                            v
                                        }
                                    });
                                } else {
                                    fields.push(
                                        quote! {
                                            #name: args[#i].parse::<#ty>().map_err(|_| {
                                                ::shovel::ParseError::InvalidArgumentValue {
                                                    index: #i,
                                                    value: args[#i],
                                                    expected: stringify!(#ty)
                                                }
                                            })?
                                        },
                                    );
                                }
                            }
                            quote! { {#(#fields),*} }
                        }
                        false => {
                            let mut fields = Vec::new();
                            for (i, f) in group.fields.iter().enumerate() {
                                let ty = f.ty_info.ty;
                                if is_last_group && i == group.fields.len() - 1 && f.ty_info.is_list {
                                    variadic = true;
                                    fields.push(quote! {
                                        {
                                            let mut v = Vec::new();
                                            for (idx, arg) in args[#i..].iter().enumerate() {
                                                v.push(arg.parse::<#ty>().map_err(|_| {
                                                    ::shovel::ParseError::InvalidArgumentValue {
                                                        index: #i + idx,
                                                        value: arg,
                                                        expected: stringify!(#ty)
                                                    }
                                                })?);
                                            }
                                            v
                                        }
                                    });
                                } else {
                                    fields.push(quote! {
                                        args[#i].parse::<#ty>().map_err(|_| {
                                            ::shovel::ParseError::InvalidArgumentValue {
                                                index: #i,
                                                value: args[#i],
                                                expected: stringify!(#ty)
                                            }
                                        })?
                                    });
                                }
                            }
                            quote! { (#(#fields),*) }
                        }
                    },
                };

                let n = group.fields.len();
                let variant = &group.ident;
                if variadic {
                    quote! {
                        l if l >= (#n - 1) => {
                            #enum_name::#variant #fields
                        }
                    }
                } else {
                    quote! {
                        #n => {
                            #enum_name::#variant #fields
                        }
                    }
                }
            })
            .collect()
    }
}

pub(crate) fn type_parse(ty: &'_ Type) -> syn::Result<TypeInfo<'_>> {
    let path = match ty {
        Type::Path(tp) if tp.qself.is_none() => &tp.path,
        _ => {
            return Err(syn::Error::new(
                ty.span(),
                format!("Unsupported type: {}", type_str(ty)),
            ));
        }
    };

    let last_segment = path.segments.last().ok_or_else(|| {
        syn::Error::new(path.span(), "Type path cannot be empty")
    })?;
    let type_args = &last_segment.arguments;

    if is_option(path) {
        if let PathArguments::AngleBracketed(params) = type_args
            && params.args.len() == 1
            && let Some(GenericArgument::Type(inner_ty)) = params.args.first()
        {
            // It's Option<T>. Check if T is Vec<U>.
            if let Type::Path(inner_tp) = inner_ty
                && is_vec(&inner_tp.path)
            {
                if let PathArguments::AngleBracketed(vec_params) =
                    &inner_tp.path.segments.last().unwrap().arguments
                    && vec_params.args.len() == 1
                    && let Some(GenericArgument::Type(vec_inner_ty)) =
                        vec_params.args.first()
                {
                    // Option<Vec<U>>
                    return Ok(TypeInfo {
                        ty: vec_inner_ty,
                        required: false,
                        flag: false,
                        name: type_str(inner_ty), // name is Vec<U>
                        is_list: true,
                    });
                }
                // Invalid Option<Vec<...>>
                return Err(syn::Error::new(
                    inner_ty.span(),
                    format!("Unsupported type: {}", type_str(inner_ty)),
                ));
            }
            // Regular Option<T>
            return Ok(TypeInfo {
                ty: inner_ty,
                required: false,
                flag: false, // Option<T> is never a flag.
                name: type_str(inner_ty),
                is_list: false,
            });
        }
        // Invalid Option<...>
        Err(syn::Error::new(
            ty.span(),
            format!("Unsupported type: {}", type_str(ty)),
        ))
    } else if is_vec(path) {
        if let PathArguments::AngleBracketed(params) = type_args
            && params.args.len() == 1
            && let Some(GenericArgument::Type(inner_ty)) = params.args.first()
        {
            // Vec<T>
            return Ok(TypeInfo {
                ty: inner_ty,
                required: true,
                flag: false,
                name: type_str(ty), // name is Vec<T>
                is_list: true,
            });
        }
        // Invalid Vec<...>
        Err(syn::Error::new(
            ty.span(),
            format!("Unsupported type: {}", type_str(ty)),
        ))
    } else {
        // Bare type
        Ok(TypeInfo {
            ty,
            required: true,
            flag: path.is_ident("bool"),
            name: last_segment.ident.to_string(),
            is_list: false,
        })
    }
}

pub(crate) fn type_str(ty: &Type) -> String {
    quote!(#ty).to_string().replace(' ', "")
}

fn to_kebab_case(input: &str) -> String {
    let name = input.strip_prefix("r#").unwrap_or(input);
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch == '_' {
            out.push('-');
        } else {
            out.push(ch);
        }
    }
    out
}

fn is_option(path: &Path) -> bool {
    let name = path.segments.iter().fold(String::new(), |mut acc, s| {
        acc.push_str(&s.ident.to_string());
        acc.push('|');
        acc
    });

    ["Option|", "std|option|Option|", "core|option|Option|"]
        .into_iter()
        .any(|s| name == s)
}

fn is_vec(path: &Path) -> bool {
    // Check for Vec patterns more efficiently
    let segments: Vec<String> =
        path.segments.iter().map(|s| s.ident.to_string()).collect();

    let segment_strs: Vec<&str> = segments.iter().map(|s| s.as_str()).collect();

    matches!(
        segment_strs.as_slice(),
        ["Vec"] | ["std", "vec", "Vec"] | ["alloc", "vec", "Vec"]
    )
}

impl ToTokens for Opts<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut exprs = Vec::new();
        for f in &self.fields {
            let name = LitStr::new(&f.long, f.ident.span());
            let ty = LitStr::new(&f.ty_info.name, f.ty_info.ty.span());
            let required = f.ty_info.required;
            let flag = f.ty_info.flag;
            let description = LitStr::new(&f.description, f.ident.span());
            let short = match f.short {
                None => quote! {None},
                Some(c) => {
                    let short = LitChar::new(c, f.ident.span());
                    quote! {Some(#short)}
                }
            };
            let format = match &f.format {
                Some(s) => LitStr::new(s, f.ident.span()),
                None => match f.short {
                    None => LitStr::new(
                        &format!("    --{}", f.long),
                        f.ident.span(),
                    ),
                    Some(c) => {
                        if f.long == c.to_string() {
                            LitStr::new(&format!("-{c}"), f.ident.span())
                        } else {
                            LitStr::new(
                                &format!("-{}, --{}", c, f.long),
                                f.ident.span(),
                            )
                        }
                    }
                },
            };

            exprs.push(quote! {
                ::shovel::Opt {
                    name: #name,
                    ty: #ty,
                    required: #required,
                    flag: #flag,
                    description: #description,
                    short: #short,
                    format: #format,
                }
            });
        }

        quote! {
            &[#(#exprs),*]
        }
        .to_tokens(tokens)
    }
}

impl ToTokens for ArgGroup<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let tys = self
            .fields
            .iter()
            .map(|f| LitStr::new(&f.ty_info.name, f.ty_info.ty.span()))
            .collect::<Vec<_>>();

        let format = match &self.format {
            Some(s) => {
                let format = LitStr::new(s, self.ident.span());
                quote! { #format }
            }
            None => {
                let format = LitStr::new(&self.get_format(), self.ident.span());
                quote! { #format }
            }
        };
        let description = LitStr::new(&self.description, self.ident.span());
        let variadic = self.variadic;

        quote! {
            ::shovel::ArgGroup {
                tys: &[#(#tys),*],
                format: #format,
                variadic: #variadic,
                description: #description
            }
        }
        .to_tokens(tokens);
    }
}

impl ToTokens for ArgGroups<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ArgGroups::Struct(group) => {
                quote! {&[#group]}.to_tokens(tokens);
            }
            ArgGroups::Enum { groups, .. } => {
                quote! { &[#(#groups),*] }.to_tokens(tokens);
            }
        }
    }
}
