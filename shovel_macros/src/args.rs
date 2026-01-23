use proc_macro::TokenStream;
use quote::quote;

use crate::field::ArgGroups;

pub(crate) fn derive_args(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let groups = match ArgGroups::parse(input) {
        Ok(groups) => groups,
        Err(err) => {
            let compile_error = err.to_compile_error();
            return quote! {
                impl ::shovel::Args for #name {
                    fn parse<'a>(
                        args: &'a [&'a str],
                    ) -> Result<Self, ::shovel::ParseError<'a>> {
                        #compile_error
                        let _ = args;
                        Err(::shovel::ParseError::ArgumentCountMismatch)
                    }

                    fn specs() -> &'static [::shovel::ArgGroup] {
                        &[]
                    }
                }
            }
            .into();
        }
    };
    let matches = groups.matches();

    quote! {
        impl ::shovel::Args for #name {
            fn parse<'a>(args: &'a [&'a str]) -> Result<Self, ::shovel::ParseError<'a>> {
                Ok(
                    match args.len() {
                        #(#matches),*
                        _ => Err(::shovel::ParseError::ArgumentCountMismatch)?
                    }
                )
            }

            fn specs() -> &'static [::shovel::ArgGroup] {
                #groups
            }
        }
    }
    .into()
}
