use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod atom;

#[proc_macro_derive(Atom)]
#[proc_macro_error]
pub fn atom(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    atom::macro_fn(input).into()
}
