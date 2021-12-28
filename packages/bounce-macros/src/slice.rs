use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Ident};

pub(crate) fn parse_with_notion_attrs(input: DeriveInput) -> syn::Result<Vec<Ident>> {
    let mut notion_idents = Vec::new();

    for attr in input.attrs.iter() {
        if !attr.path.is_ident("with_notion") {
            continue;
        }

        let ident = attr.parse_args::<Ident>()?;

        notion_idents.push(ident);
    }

    Ok(notion_idents)
}

pub(crate) fn macro_fn(input: DeriveInput) -> TokenStream {
    let notion_idents = match parse_with_notion_attrs(input.clone()) {
        Ok(m) => m,
        Err(e) => return e.into_compile_error(),
    };

    let mut notion_apply_impls = Vec::new();
    let notion_ident = Ident::new("notion", Span::mixed_site());

    for ident in notion_idents {
        let notion_apply_impl = quote! {
            if let Ok(m) = <::std::rc::Rc::<dyn std::any::Any>>::downcast::<#ident>(::std::clone::Clone::clone(&#notion_ident)) {
                return ::bounce::WithNotion::<#ident>::apply(::std::clone::Clone::clone(&self), m);
            }
        };

        notion_apply_impls.push(notion_apply_impl);
    }

    let type_ident = input.ident;

    quote! {
        #[automatically_derived]
        impl ::bounce::Slice for #type_ident {
            type Action = <Self as ::bounce::__vendored::yew::functional::Reducible>::Action;

            fn reduce(self: ::std::rc::Rc<Self>, action: Self::Action) -> ::std::rc::Rc<Self> {
                ::bounce::__vendored::yew::functional::Reducible::reduce(self, action)
            }

            fn apply(self: ::std::rc::Rc<Self>, #notion_ident: ::std::rc::Rc<dyn ::std::any::Any>) -> ::std::rc::Rc<Self> {
                #(#notion_apply_impls)*

                self
            }
        }
    }
}