use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(crate) fn macro_fn(input: DeriveInput) -> TokenStream {
    let ident = input.ident;

    quote! {
        #[automatically_derived]
        impl ::bounce::Atom for #ident {}
    }
}
