use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;

#[proc_macro_attribute]
pub fn register(args: TokenStream, mut input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return syn::Error::new(Span::call_site(), "`register` does not take any arguments")
            .to_compile_error()
            .into();
    }

    let name = match function_name(&input.clone().into()) {
        Ok(name) => name,
        Err(err) => return err.to_compile_error().into(),
    };

    input.extend([TokenStream::from(quote! {
        const _: () = {
            ::quest_helper::register_part!(#name);
        };
    })]);
    input
}

/// Returns the name of the function declared by `item`.
///
/// Only the tokens up to the function name are looked at: the rest of the item
/// is never interpreted, so the attribute keeps working for signatures and
/// bodies it would not otherwise understand.
fn function_name(item: &TokenStream2) -> syn::Result<Ident> {
    let mut tokens = item.clone().into_iter();

    while let Some(token) = tokens.next() {
        if let TokenTree::Ident(keyword) = &token {
            if keyword == "fn" {
                return match tokens.next() {
                    Some(TokenTree::Ident(name)) => Ok(name),
                    _ => Err(syn::Error::new_spanned(token, "expected a function name")),
                };
            }
        }
    }

    Err(syn::Error::new_spanned(
        item,
        "`register` can only be applied to a function",
    ))
}
