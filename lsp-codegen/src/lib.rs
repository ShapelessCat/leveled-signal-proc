use context::MacroContext;
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod context;
mod node;
mod schema;

#[proc_macro]
pub fn define_input_schema(input: TokenStream) -> TokenStream {
    let ctx = parse_macro_input!(input as MacroContext);
    match ctx.expand_input_state_bag() {
        Ok(state_bag) => state_bag.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn define_data_logic_nodes(input: TokenStream) -> TokenStream {
    let ctx = parse_macro_input!(input as MacroContext);
    match ctx.define_lsp_nodes() {
        Ok(res) => res.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn impl_data_logic_updates(input: TokenStream) -> TokenStream {
    let ctx = parse_macro_input!(input as MacroContext);
    match ctx.impl_nodes_update() {
        Ok(res) => res.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
