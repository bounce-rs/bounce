use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Ident};

use super::slice::{create_notion_apply_impls, parse_with_notion_attrs};

pub(crate) fn macro_fn(input: DeriveInput) -> TokenStream {
    let notion_idents = match parse_with_notion_attrs(input.clone()) {
        Ok(m) => m,
        Err(e) => return e.into_compile_error(),
    };

    let notion_ident = Ident::new("notion", Span::mixed_site());
    let notion_apply_impls = create_notion_apply_impls(&notion_ident, notion_idents);

    let ident = input.ident;

    quote! {
        #[automatically_derived]
        impl ::bounce::Atom for #ident {
            fn apply(self: ::std::rc::Rc<Self>, #notion_ident: ::std::rc::Rc<dyn ::std::any::Any>) -> ::std::rc::Rc<Self> {
                #(#notion_apply_impls)*

                self
            }
        }
    }
}
