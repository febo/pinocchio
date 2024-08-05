mod declare_id;
mod pubkey;

pub(crate) use declare_id::DeclareId;
pub(crate) use pubkey::Pubkey;

use proc_macro2::Span;
use quote::quote;
use syn::{parse::ParseStream, Expr, LitByte, LitStr, Result};

fn parse_id(input: ParseStream) -> Result<proc_macro2::TokenStream> {
    let id = if input.peek(syn::LitStr) {
        let id_literal: LitStr = input.parse()?;
        parse_pubkey(&id_literal)?
    } else {
        let expr: Expr = input.parse()?;
        quote! { #expr }
    };

    if !input.is_empty() {
        let stream: proc_macro2::TokenStream = input.parse()?;
        return Err(syn::Error::new_spanned(stream, "unexpected token"));
    }
    Ok(id)
}

fn parse_pubkey(id_literal: &LitStr) -> Result<proc_macro2::TokenStream> {
    let id_vec = bs58::decode(id_literal.value())
        .into_vec()
        .map_err(|_| syn::Error::new_spanned(id_literal, "failed to decode base58 string"))?;
    let id_array = <[u8; 32]>::try_from(<&[u8]>::clone(&&id_vec[..])).map_err(|_| {
        syn::Error::new_spanned(
            id_literal,
            format!("pubkey array is not 32 bytes long: len={}", id_vec.len()),
        )
    })?;
    let bytes = id_array.iter().map(|b| LitByte::new(*b, Span::call_site()));
    Ok(quote! {
        [#(#bytes,)*]
    })
}
