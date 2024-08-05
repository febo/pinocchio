use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Result,
};

use super::parse_id;

pub(crate) struct DeclareId(proc_macro2::TokenStream);

impl Parse for DeclareId {
    fn parse(input: ParseStream) -> Result<Self> {
        parse_id(input).map(Self)
    }
}

impl ToTokens for DeclareId {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        id_to_tokens(&self.0, quote! { ::pinocchio::pubkey::Pubkey }, tokens)
    }
}

fn id_to_tokens(
    id: &proc_macro2::TokenStream,
    pubkey_type: proc_macro2::TokenStream,
    tokens: &mut proc_macro2::TokenStream,
) {
    tokens.extend(quote! {
        /// The const program ID.
        pub const ID: #pubkey_type = #id;

        /// Returns `true` if given pubkey is the program ID.
        pub fn check_id(id: &#pubkey_type) -> bool {
            id == &ID
        }

        /// Returns the program ID.
        pub const fn id() -> #pubkey_type {
            ID
        }
    });
}
