use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Result,
};

use super::parse_id;

pub(crate) struct Pubkey(proc_macro2::TokenStream);

impl Parse for Pubkey {
    fn parse(input: ParseStream) -> Result<Self> {
        parse_id(input, quote! { ::pubkey::Pubkey }).map(Self)
    }
}

impl ToTokens for Pubkey {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let id = &self.0;
        tokens.extend(quote! {#id})
    }
}
