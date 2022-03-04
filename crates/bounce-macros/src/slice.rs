use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Ident, Type};

pub(crate) fn parse_with_notion_attrs(input: DeriveInput) -> syn::Result<Vec<Type>> {
    let mut notion_idents = Vec::new();

    for attr in input.attrs.iter() {
        if !attr.path.is_ident("with_notion") {
            continue;
        }

        let ident = attr.parse_args::<Type>()?;

        notion_idents.push(ident);
    }

    Ok(notion_idents)
}

pub(crate) fn parse_find_observed(input: DeriveInput) -> syn::Result<bool> {
    let mut observed = false;

    for attr in input.attrs.iter() {
        if !attr.path.is_ident("observed") {
            continue;
        }

        if !attr.tokens.is_empty() {
            return Err(syn::Error::new_spanned(
                &attr.tokens,
                "observed attribute accepts no argument",
            ));
        }

        if observed {
            return Err(syn::Error::new_spanned(
                &attr.path,
                "you can only have 1 observed attribute",
            ));
        }

        observed = true;
    }

    Ok(observed)
}

pub(crate) fn create_notion_apply_impls(notion_ident: &Ident, idents: &[Type]) -> Vec<TokenStream> {
    let mut notion_apply_impls = Vec::new();

    for ident in idents {
        let notion_apply_impl = quote! {
            let #notion_ident = match <::std::rc::Rc::<dyn std::any::Any>>::downcast::<#ident>(#notion_ident) {
                ::std::result::Result::Ok(m) => return ::bounce::WithNotion::<#ident>::apply(::std::clone::Clone::clone(&self), m),
                ::std::result::Result::Err(e) => e,
            };
        };

        notion_apply_impls.push(notion_apply_impl);
    }

    notion_apply_impls
}

pub(crate) fn macro_fn(input: DeriveInput) -> TokenStream {
    let notion_idents = match parse_with_notion_attrs(input.clone()) {
        Ok(m) => m,
        Err(e) => return e.into_compile_error(),
    };
    let observed = match parse_find_observed(input.clone()) {
        Ok(m) => m,
        Err(e) => return e.into_compile_error(),
    };

    let notion_ident = Ident::new("notion", Span::mixed_site());
    let notion_apply_impls = create_notion_apply_impls(&notion_ident, &notion_idents);

    let type_ident = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let impl_observed = observed.then(|| {
        quote! {
            fn changed(self: ::std::rc::Rc<Self>) {
                ::bounce::Observed::changed(self);
            }
        }
    });

    quote! {
        #[automatically_derived]
        impl #impl_generics ::bounce::Slice for #type_ident #ty_generics #where_clause {
            type Action = <Self as ::bounce::__vendored::yew::functional::Reducible>::Action;

            fn reduce(self: ::std::rc::Rc<Self>, action: Self::Action) -> ::std::rc::Rc<Self> {
                ::bounce::__vendored::yew::functional::Reducible::reduce(self, action)
            }

            fn apply(self: ::std::rc::Rc<Self>, #notion_ident: ::std::rc::Rc<dyn ::std::any::Any>) -> ::std::rc::Rc<Self> {
                #(#notion_apply_impls)*

                self
            }

            #impl_observed
        }
    }
}
