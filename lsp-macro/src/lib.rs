use proc_macro::TokenStream;

use syn::{self, DeriveInput};

mod patch;

#[proc_macro_derive(Patchable, attributes(patchable))]
pub fn derive_state_and_patchable_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(input as DeriveInput);
    patch::create_state_and_patchable_impl(&input).into()
}
