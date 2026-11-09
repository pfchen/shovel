use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::field::{Opts, parser_ident};

pub(crate) fn derive_opts(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let raw_opts = parser_ident("opts");
    let env = parser_ident("env");
    let opt = parser_ident("opt");
    let opts = match Opts::parse(input) {
        Ok(opts) => opts,
        Err(err) => {
            let compile_error = err.to_compile_error();
            return quote! {
                impl Opts for #name {
                    fn parse<'a, 'b: 'a>(
                        #raw_opts: &'a ::shovel::RawOpts<'a, 'b>,
                    ) -> Result<(Self, usize), ::shovel::ParseError<'b>> {
                        #compile_error
                        let _ = #raw_opts;
                        Err(::shovel::ParseError::UnknownOption(""))
                    }

                    fn specs() -> &'static [::shovel::Opt] {
                        &[]
                    }
                }
            }
            .into();
        }
    };
    let init_fields = opts.init_fields();
    let matches = opts.matches();
    let env_stmts = opts.env_stmts();
    let required_stmts = opts.required_stmts();
    let fields = opts.fields.iter().map(|field| {
        let name = &field.ident;
        let binding = field.binding();
        quote! { #name: #binding }
    });

    quote! {
        impl Opts for #name {
            fn parse<'a, 'b: 'a>(
                #raw_opts: &'a ::shovel::RawOpts<'a, 'b>,
            ) -> Result<(Self, usize), ::shovel::ParseError<'b>> {
                Self::parse_with_env(#raw_opts, &[])
            }

            fn parse_with_env<'a, 'b: 'a>(
                #raw_opts: &'a ::shovel::RawOpts<'a, 'b>,
                #env: &'b [(&'b str, &'b str)],
            ) -> Result<(Self, usize), ::shovel::ParseError<'b>> {
                #(#init_fields)*

                for #opt in #raw_opts {
                    match #opt.0 {
                        #(#matches),*
                        "help" | "version" => {},
                        _ => Err(::shovel::ParseError::UnknownOption(#opt.0))?,
                    }
                }

                #(#env_stmts)*
                #(#required_stmts)*

                Ok((Self { #(#fields),* }, #raw_opts.len_consumed()))
            }

            fn specs() -> &'static [::shovel::Opt] {
                #opts
            }
        }
    }
    .into()
}
