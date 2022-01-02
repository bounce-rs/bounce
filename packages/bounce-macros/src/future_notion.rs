use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{FnArg, Generics, Ident, ItemFn, ReturnType, Type, TypePath, Visibility};

#[derive(Debug)]
pub struct FutureNotionAttr {
    name: Ident,
}

impl Parse for FutureNotionAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(input.error("a type identifier is required for future notions"));
        }

        Ok(Self {
            name: input.parse()?,
        })
    }
}

pub struct AsyncFnProps {
    input: TypePath,
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
                return Err(syn::Error::new_spanned(
                    item.sig.output.clone(),
                    "future notions must return an Rc'ed type",
                ))
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
            Type::Path(m) => m,
            arg => return Err(syn::Error::new_spanned(arg, "input must be an Rc'ed type.")),
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

pub(crate) fn macro_fn(attr: FutureNotionAttr, item: ItemFn) -> TokenStream {
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

    let notion_name = attr.name;

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

    let phantom_generics = generics
        .type_params()
        .map(|ty_param| ty_param.ident.clone())
        .collect::<Punctuated<_, Comma>>();

    quote! {
        #item

        #vis struct #notion_name #generics {
            _marker: ::std::marker::PhantomData<(#phantom_generics)>
        }

        #[automatically_derived]
        impl #impl_generics ::bounce::FutureNotion for #notion_name #ty_generics #where_clause {
            type Input = #input;
            type Output = #output;

            fn run(
                states: ::bounce::BounceStates,
                input: #input,
            ) -> ::bounce::__vendored::futures::future::LocalBoxFuture<'static, #output> {
                ::std::boxed::Box::pin(#fn_call)
            }
        }
    }
}
