use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Ident};

use super::slice::BounceAttrs;

pub(crate) fn macro_fn(input: DeriveInput) -> TokenStream {
    let bounce_attrs = match BounceAttrs::parse(&input.attrs) {
        Ok(m) => m,
        Err(e) => return e.into_compile_error(),
    };

    let notion_ident = Ident::new("notion", Span::mixed_site());
    let notion_apply_impls = bounce_attrs.create_notion_apply_impls(&notion_ident);
    let notion_ids_impls = bounce_attrs.create_notion_id_impls();

    let ident = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let impl_observed = bounce_attrs.observed.is_some().then(|| {
        quote! {
            fn changed(self: ::std::rc::Rc<Self>) {
                ::bounce::Observed::changed(self);
            }
        }
    });

    quote! {
        #[automatically_derived]
        impl #impl_generics ::bounce::Atom for #ident #ty_generics #where_clause {
            fn apply(self: ::std::rc::Rc<Self>, #notion_ident: ::std::rc::Rc<dyn ::std::any::Any>) -> ::std::rc::Rc<Self> {
                #(#notion_apply_impls)*

                self
            }

            fn notion_ids(&self) -> ::std::vec::Vec<::std::any::TypeId> {
                ::std::vec![#(#notion_ids_impls,)*]
            }

            #impl_observed
        }
    }
}
