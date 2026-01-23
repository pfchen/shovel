use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::field::Opts;

pub(crate) fn derive_opts(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let opts = match Opts::parse(input) {
        Ok(opts) => opts,
        Err(err) => {
            let compile_error = err.to_compile_error();
            return quote! {
                impl Opts for #name {
                    fn parse<'a, 'b: 'a>(
                        opts: &'a ::shovel::RawOpts<'a, 'b>,
                    ) -> Result<(Self, usize), ::shovel::ParseError<'b>> {
                        #compile_error
                        let _ = opts;
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
    let required_stmts = opts.required_stmts();
    let fields: Vec<proc_macro2::Ident> =
        opts.fields.iter().map(|f| f.ident.clone()).collect();

    quote! {
        impl Opts for #name {
            fn parse<'a, 'b: 'a>(
                opts: &'a ::shovel::RawOpts<'a, 'b>,
            ) -> Result<(Self, usize), ::shovel::ParseError<'b>> {
                #(#init_fields)*

                for opt in opts {
                    match opt.0 {
                        #(#matches),*
                        "help" | "version" => {},
                        _ => Err(::shovel::ParseError::UnknownOption(opt.0))?,
                    }
                }

                #(#required_stmts)*

                Ok((Self { #(#fields),* }, opts.len_consumed()))
            }

            fn specs() -> &'static [::shovel::Opt] {
                #opts
            }
        }
    }
    .into()
}
