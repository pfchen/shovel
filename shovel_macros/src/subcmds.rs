use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::{LitInt, Token, parse_macro_input};

mod kw {
    syn::custom_keyword!(number);
}

#[allow(dead_code)]
pub struct MacroInput {
    hint: kw::number,
    eq_token: Token![=],
    number: u16,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let hint = input.parse::<kw::number>()?;
        let eq_token = input.parse()?;
        let lit: LitInt = input.parse()?;
        let number = lit.base10_parse::<u16>()?;
        Ok(MacroInput {
            hint,
            eq_token,
            number,
        })
    }
}

pub(crate) fn impl_subcmds(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as MacroInput);
    let mut all_impls = quote! {};

    // Generate implementations from 3 up to the specified number
    for number in 3..=input.number {
        let generics = gen_generics(number);
        let tuple = gen_tuple(number);
        let cmd_count = gen_cmd_count(number);
        let run_expr = gen_run_expr(number);
        let help_stmts = gen_help_stmts(number);

        all_impls = quote! {
            #all_impls

            impl #generics crate::Subcmds<G> for #tuple {
                #cmd_count

                fn run<'b>(
                    &'b self,
                    name: &'b str,
                    raw_args: &'b crate::RawArgs<'b>,
                    global_opts: G,
                ) -> crate::ActionResult {
                    #run_expr
                }

                fn get_help_list<'b>(
                    &'b self,
                    pool: crate::Pool<'b, crate::CmdHelp<'b>>
                ) -> Option<&'b [crate::CmdHelp<'b>]> {
                    #help_stmts
                }
            }
        };
    }
    all_impls.into()
}

pub(crate) fn gen_generics(number: u16) -> proc_macro2::TokenStream {
    let mut generics = quote! { 'a, G: crate::Opts };
    for i in 0..number {
        let (o, a, c, r) = (
            format_ident!("O{}", i.to_string()),
            format_ident!("A{}", i.to_string()),
            format_ident!("C{}", i.to_string()),
            format_ident!("R{}", i.to_string()),
        );
        generics = quote! {
            #generics, #o: crate::Opts, #a: crate::Args, #c: crate::Subcmds<G>, #r: crate::ActionReturn
        }
    }
    quote! { < #generics > }
}

pub(crate) fn gen_tuple(number: u16) -> proc_macro2::TokenStream {
    let number = number as usize;
    let mut tys = quote!();
    for i in 0..number {
        let (o, a, c, r) = (
            format_ident!("O{}", i.to_string()),
            format_ident!("A{}", i.to_string()),
            format_ident!("C{}", i.to_string()),
            format_ident!("R{}", i.to_string()),
        );
        tys = quote! { #tys Cmd<'a, #o, #a, G, #c, #r>, };
    }
    quote! { ( #tys ) }
}

pub(crate) fn gen_cmd_count(number: u16) -> proc_macro2::TokenStream {
    let number = number as usize;
    let mut count = quote! { #number };
    for i in 0..number {
        let c = format_ident!("C{}", i.to_string());
        count = quote! { #count + #c::CMD_COUNT }
    }
    quote! { const CMD_COUNT: usize = #count; }
}

pub(crate) fn gen_run_expr(number: u16) -> proc_macro2::TokenStream {
    let mut expr = quote!();
    let mut names = quote!();
    for i in 0..number {
        let i_lit = LitInt::new(&i.to_string(), Span::call_site());
        if i == 0 {
            expr = quote! {
                if name == self.#i_lit.name {
                    self.#i_lit.run(raw_args, global_opts)
                }
            }
        } else {
            expr = quote! {
                #expr
                else if name == self.#i_lit.name {
                    self.#i_lit.run(raw_args, global_opts)
                }
            }
        }
        names = quote! { #names self.#i_lit.name, };
    }

    quote! {
        #expr
        else {
            crate::ActionResult::Suggestion(crate::Suggestion::find(
                name,
                &[ #names ],
            ))
        }
    }
}

pub(crate) fn gen_help_stmts(number: u16) -> proc_macro2::TokenStream {
    let number = number as usize;
    let mut stmts = quote!();
    let mut helps = quote!();
    for i in 0..number - 1 {
        let subcmd_pool = format_ident!("subcmd{}_pool", i.to_string());
        let c = format_ident!("C{}", i.to_string());
        let i_lit = LitInt::new(&i.to_string(), Span::call_site());
        stmts = quote! {
            #stmts
            let (#subcmd_pool, right) = right.alloc(#c::CMD_COUNT);
        };
        helps = quote! { #helps self.#i_lit.help(#subcmd_pool), };
    }

    let i = number - 1;
    let i_lit = LitInt::new(&i.to_string(), Span::call_site());
    let helps = quote! { #helps self.#i_lit.help(right), };

    quote! {
        let (mut left, right) = pool.alloc(#number);
        #stmts
        Some(left.assign(&[#helps]))
    }
}
