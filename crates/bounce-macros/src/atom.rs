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
    let notion_apply_impls = create_notion_apply_impls(&notion_ident, &notion_idents);

    let ident = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics ::bounce::Atom for #ident #ty_generics #where_clause {
            fn apply(self: ::std::rc::Rc<Self>, #notion_ident: ::std::rc::Rc<dyn ::std::any::Any>) -> ::std::rc::Rc<Self> {
                #(#notion_apply_impls)*

                self
            }

            fn notion_ids(&self) -> &'static [::std::any::TypeId] {
                static NOTION_IDS: ::bounce::__vendored::once_cell::sync::OnceCell<::std::vec::Vec<::std::any::TypeId>> =
                    ::bounce::__vendored::once_cell::sync::OnceCell::new();

                NOTION_IDS.get_or_init(
                    || {
                        ::std::vec![#(::std::any::TypeId::of::<#notion_idents>(),)*]
                    }
                )
            }
        }
    }
}
