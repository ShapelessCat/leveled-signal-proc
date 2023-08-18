use context::MacroContext;
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod context;
mod node;
mod schema;
mod metrics;

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

#[proc_macro]
pub fn define_output_schema(input: TokenStream) -> TokenStream {
    let ctx = parse_macro_input!(input as MacroContext);
    match ctx.define_output_schema() {
        Ok(res) => res.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn impl_metrics_measurement(input: TokenStream) -> TokenStream {
    let ctx = parse_macro_input!(input as MacroContext);
    ctx.impl_metrics_measuring().into()
}

struct MainFnMeta {
    id: syn::Ident,
    path: syn::LitStr,
}

impl syn::parse::Parse for MainFnMeta {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let id : syn::Ident = input.parse()?;
        let _at: syn::Token![@] = input.parse()?;
        let path : syn::LitStr = input.parse()?;
        Ok(Self {
            id,
            path
        })
    }
}

#[proc_macro]
pub fn include_lsp_ir(input: TokenStream) -> TokenStream {
    let MainFnMeta{id, path} = parse_macro_input!(input as MainFnMeta);
    let real_ir_path = match context::MacroContext::normalize_ir_path(&path.value()) {
        Ok(path) => path,
        Err(e) => panic!("{}", e.to_string()),
    };
    let real_ir_path = real_ir_path.to_str();
    quote::quote! {
        const _ : () = { include_str!(#real_ir_path); };
        lsp_codegen::define_input_schema!(#path);
        lsp_codegen::define_output_schema!(#path);
        pub fn #id<InputIter, OutputHandler>(input_iter: InputIter, mut out_handle: OutputHandler) -> Result<(), anyhow::Error> 
        where
            InputIter: Iterator<Item = InputSignalBagPatch>,
            OutputHandler: FnMut(MetricsBag) -> Result<(), anyhow::Error>
        {
            use lsp_runtime::LspContext;
            use serde_json::Deserializer;
            let mut input_state = Default::default();
            lsp_codegen::define_data_logic_nodes!(#path);
            let mut ctx = LspContext::<_, InputSignalBag>::new(input_iter);
            while let Some(moment) = ctx.next_event(&mut input_state) {
                let mut update_context = ctx.borrow_update_context();
                if moment.should_update_signals() {
                    lsp_codegen::impl_data_logic_updates!(#path);
                }
                if moment.should_take_measurements() {
                    lsp_codegen::impl_metrics_measurement!(#path);
                    out_handle(_metrics_bag)?;
                }
            }
            Ok(())
        }
    }.into()
}