use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod atom;
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
