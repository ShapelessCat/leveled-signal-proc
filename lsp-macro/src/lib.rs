use patch::create_state_and_patchable_impl;
use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput};

mod patch;

#[proc_macro_derive(Patchable, attributes(patchable))]
pub fn derive_state_and_patchable_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    create_state_and_patchable_impl(&input).into()
}
