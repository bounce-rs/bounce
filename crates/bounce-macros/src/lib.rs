use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput, ItemFn};

mod atom;
mod future_notion;
mod slice;

#[proc_macro_derive(Atom, attributes(with_notion))]
#[proc_macro_error]
pub fn atom(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    atom::macro_fn(input).into()
}

#[proc_macro_derive(Slice, attributes(with_notion))]
#[proc_macro_error]
pub fn slice(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    slice::macro_fn(input).into()
}

#[proc_macro_attribute]
pub fn future_notion(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);
    let attr = parse_macro_input!(attr as future_notion::FutureNotionAttr);

    future_notion::macro_fn(attr, item).into()
}
