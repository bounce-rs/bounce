use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_quote, FnArg, Generics, Ident, ItemFn, ReturnType, Type, Visibility};

#[derive(Debug)]
pub struct FutureNotionAttr {
    name: Option<Ident>,
}

impl Parse for FutureNotionAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
        })
    }
}

pub struct AsyncFnProps {
    input: Type,
    output: Type,
    with_state: bool,
    vis: Visibility,
    name: Ident,
    generics: Generics,
}

impl AsyncFnProps {
    fn extract(item: &ItemFn) -> syn::Result<Self> {
        let vis = item.vis.clone();
        let name = item.sig.ident.clone();
        let generics = item.sig.generics.clone();

        if item.sig.asyncness.is_none() {
            return Err(syn::Error::new_spanned(
                name,
                "future notions must be async functions",
            ));
        }

        let output = match item.sig.output {
            ReturnType::Default => {
                // Unit Type is Output.
                parse_quote! { () }
            }
            ReturnType::Type(_, ref ty) => *ty.clone(),
        };

        let mut fn_args = item.sig.inputs.iter();

        let (input_arg, with_state) = match (fn_args.next(), fn_args.next()) {
            (Some(_), Some(n)) => (n.clone(), true),
            (Some(m), None) => (m.clone(), false),
            _ => {
                return Err(syn::Error::new_spanned(
                    item.sig.inputs.clone(),
                    "future notions must accept at least 1 argument",
                ))
            }
        };

        let input_type = match input_arg {
            FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(
                    item.sig.inputs.clone(),
                    "future notions do not accept self argument",
                ))
            }
            FnArg::Typed(m) => m,
        }
        .ty;

        let input = match *input_type {
            Type::Reference(m) => *m.elem,
            arg => return Err(syn::Error::new_spanned(arg, "input must be a reference")),
        };
        Ok(Self {
            input,
            output,
            with_state,
            vis,
            name,
            generics,
        })
    }
}

pub(crate) fn macro_fn(attr: FutureNotionAttr, mut item: ItemFn) -> TokenStream {
    let async_fn_props = match AsyncFnProps::extract(&item) {
        Ok(m) => m,
        Err(e) => return e.into_compile_error(),
    };

    let AsyncFnProps {
        input,
        output,
        with_state,
        vis,
        name: fn_name,
        generics,
    } = async_fn_props;

    let (notion_name, fn_name) = match attr.name {
        Some(m) => (m, fn_name),
        None => (fn_name, Ident::new("inner", Span::mixed_site())),
    };

    if notion_name == fn_name {
        return syn::Error::new_spanned(
            item.sig.ident,
            "notions must not have the same name as the function",
        )
        .into_compile_error();
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let fn_generics = ty_generics.as_turbofish();

    let fn_call = if with_state {
        quote! {
            #fn_name #fn_generics(states, input)
        }
    } else {
        quote! {
            #fn_name #fn_generics(input)
        }
    };

    item.sig.ident = fn_name;

    let phantom_generics = generics
        .type_params()
        .map(|ty_param| ty_param.ident.clone())
        .collect::<Punctuated<_, Comma>>();

    quote! {

        #vis struct #notion_name #generics {
            _marker: ::std::marker::PhantomData<(#phantom_generics)>
        }

        #[automatically_derived]
        impl #impl_generics ::bounce::FutureNotion for #notion_name #ty_generics #where_clause {
            type Input = #input;
            type Output = #output;

            fn run<'a>(
                states: &'a ::bounce::BounceStates,
                input: &'a #input,
            ) -> ::bounce::__vendored::futures::future::LocalBoxFuture<'a, #output> {
                #item

                ::std::boxed::Box::pin(#fn_call)
            }
        }
    }
}
