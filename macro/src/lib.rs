mod parse;
use parse::{DeclareId, Pubkey};

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn pubkey(input: TokenStream) -> TokenStream {
    let id = parse_macro_input!(input as Pubkey);
    TokenStream::from(quote! {#id})
}

#[proc_macro]
pub fn declare_id(input: TokenStream) -> TokenStream {
    let id = parse_macro_input!(input as DeclareId);
    TokenStream::from(quote! {#id})
}
