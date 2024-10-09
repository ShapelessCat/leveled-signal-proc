use context::MacroContext;
use proc_macro::TokenStream;

mod context;
mod metrics;
mod node;
mod processing;
mod schema;

#[proc_macro]
pub fn should_merge_simultaneous_moments(input: TokenStream) -> TokenStream {
    let ctx = syn::parse_macro_input!(input as MacroContext);
    ctx.merge_simultaneous_moments().into()
}

#[proc_macro]
pub fn define_input_schema(input: TokenStream) -> TokenStream {
    let ctx = syn::parse_macro_input!(input as MacroContext);
    match ctx.expand_input_state_bag() {
        Ok(state_bag) => state_bag.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn define_data_logic_nodes(input: TokenStream) -> TokenStream {
    let ctx = syn::parse_macro_input!(input as MacroContext);
    match ctx.define_lsp_nodes() {
        Ok(res) => res.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn patch_lsp_nodes(input: TokenStream) -> TokenStream {
    let ctx = syn::parse_macro_input!(input as MacroContext);
    match ctx.patch_lsp_nodes() {
        Ok(res) => res.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn impl_data_logic_updates(input: TokenStream) -> TokenStream {
    let ctx = syn::parse_macro_input!(input as MacroContext);
    match ctx.impl_nodes_update() {
        Ok(res) => res.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn define_output_schema(input: TokenStream) -> TokenStream {
    let ctx = syn::parse_macro_input!(input as MacroContext);
    match ctx.define_output_schema() {
        Ok(res) => res.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn define_previous_metrics_bag(input: TokenStream) -> TokenStream {
    let ctx = syn::parse_macro_input!(input as MacroContext);
    match ctx.define_previous_metrics_bag() {
        Ok(res) => res.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn set_previous_metrics_bag_value(input: TokenStream) -> TokenStream {
    let ctx = syn::parse_macro_input!(input as MacroContext);
    match ctx.set_previous_metrics_bag_value() {
        Ok(res) => res.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn impl_metrics_measurement(input: TokenStream) -> TokenStream {
    let ctx = syn::parse_macro_input!(input as MacroContext);
    ctx.impl_metrics_measuring().into()
}

#[proc_macro]
pub fn define_measurement_trigger(input: TokenStream) -> TokenStream {
    let ctx = syn::parse_macro_input!(input as MacroContext);
    ctx.define_signal_trigger_measurement_ctx().into()
}

#[proc_macro]
pub fn impl_signal_measurement_trigger(input: TokenStream) -> TokenStream {
    let ctx = syn::parse_macro_input!(input as MacroContext);
    ctx.impl_signal_triggered_measurement().into()
}

#[proc_macro]
pub fn impl_signal_measurement_limit_side_control(input: TokenStream) -> TokenStream {
    let ctx = syn::parse_macro_input!(input as MacroContext);
    ctx.impl_measurement_limit_side_control().into()
}

#[proc_macro]
pub fn impl_should_output(input: TokenStream) -> TokenStream {
    let ctx = syn::parse_macro_input!(input as MacroContext);
    ctx.impl_should_output().into()
}

#[proc_macro]
pub fn build_checkpoint(input: TokenStream) -> TokenStream {
    let ctx = syn::parse_macro_input!(input as MacroContext);
    ctx.build_checkpoint(quote::quote! { update_context })
        .into()
}

// // For debugging, output the final checkpoint after the whole iteration.
// #[proc_macro]
// pub fn build_debug_final_checkpoint(input: TokenStream) -> TokenStream {
//     let ctx = parse_macro_input!(input as MacroContext);
//     ctx.build_checkpoint(quote::quote! { ctx }).into()
// }

struct MainFnMeta {
    id: syn::Ident,
    path: syn::LitStr,
}

impl syn::parse::Parse for MainFnMeta {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let id: syn::Ident = input.parse()?;
        let _at: syn::Token![@] = input.parse()?;
        let path: syn::LitStr = input.parse()?;
        Ok(Self { id, path })
    }
}

#[proc_macro]
pub fn include_lsp_ir(input: TokenStream) -> TokenStream {
    let MainFnMeta { id, path } = syn::parse_macro_input!(input as MainFnMeta);
    let real_ir_path = match MacroContext::normalize_ir_path(&path.value()) {
        Ok(path) => path,
        Err(e) => panic!("{}", e.to_string()),
    };
    let real_ir_path = real_ir_path.to_str();

    if let Err(e) = MacroContext::parse_ir_file(&path) {
        return e.to_compile_error().into();
    }

    quote::quote! {
        const _ : () = { include_str!(#real_ir_path); };
        lsp_codegen::define_input_schema!(#path);
        lsp_codegen::define_output_schema!(#path);

        pub fn #id<InputIter, OutputHandler, Inst>(
            input_iter: InputIter,
            mut out_handle: OutputHandler,
            instrument_ctx: &mut Inst,
            checkpoint_home: &std::path::Path,
        ) -> Result<(), anyhow::Error>
        where
            InputIter: Iterator<Item = InputSignalBagPatch>,
            OutputHandler: FnMut(&MetricsBag) -> Result<(), anyhow::Error>,
            Inst: lsp_runtime::instrument::LspDataLogicInstrument,
        {
            use lsp_runtime::context::LspContext;
            use lsp_runtime::checkpoint::Checkpoint;
            let path2checkpoint = checkpoint_home.join("checkpoint.json");
            let (context_state, input_state, entries) = std::fs::read_to_string(path2checkpoint.as_path())
                .ok()
                .and_then(|s| serde_json::from_str::<Checkpoint>(&s).ok())
                .map_or(Default::default(), |c| {
                    let Checkpoint {
                        context_state,
                        input_state,
                        entries,
                    } = c;
                    (Some(context_state), Some(input_state), Some(entries))
                });
            lsp_codegen::define_data_logic_nodes!(#path);
            lsp_codegen::patch_lsp_nodes!(#path);

            lsp_codegen::define_measurement_trigger!(#path);

            // Setup for interval metrics computation
            lsp_codegen::define_previous_metrics_bag!(#path);

            // Setup for input signal state
            let mut input_state: InputSignalBag = input_state
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or(Default::default());

            // Setup for computation context
            let mut ctx = LspContext::<_, InputSignalBag>::new(
                input_iter,
                lsp_codegen::should_merge_simultaneous_moments!(#path)
            );
            if let Some(context_state) = context_state {
                use lsp_runtime::signal_api::Patchable;
                ctx.patch(&context_state);
            };

            // Main iteration
            while let Some(moment) = ctx.next_event(&mut input_state) {
                instrument_ctx.data_logic_update_begin();
                let mut update_context = ctx.borrow_update_context();
                let mut should_measure = moment.should_take_measurements();
                let mut should_use_left_limit = false;

                let left_limit_measurements = {
                    lsp_codegen::impl_metrics_measurement!(#path);
                    _metrics_bag
                };

                if moment.should_update_signals() {
                    lsp_codegen::impl_data_logic_updates!(#path , instrument_ctx);
                    lsp_codegen::impl_signal_measurement_trigger!(#path);
                    should_measure = should_measure || __signal_trigger_fired;
                    lsp_codegen::impl_signal_measurement_limit_side_control!(#path);
                    should_use_left_limit = __should_measure_left_side_limit;
                }
                let should_output = lsp_codegen::impl_should_output!(#path);
                if should_measure && should_output {
                    let _metrics_bag = if should_use_left_limit {
                        left_limit_measurements
                    } else {
                        lsp_codegen::impl_metrics_measurement!(#path);
                        _metrics_bag
                    };
                    instrument_ctx.data_logic_update_end();
                    out_handle(&_metrics_bag)?;
                    lsp_codegen::set_previous_metrics_bag_value!(#path);
                } else {
                    instrument_ctx.data_logic_update_end();
                }
                // Write checkpoint
                if update_context.offset() % 200 == 0 {
                    let checkpoint = lsp_codegen::build_checkpoint!(#path);
                    std::fs::write(
                        path2checkpoint.as_path(),
                        &serde_json::to_string(&checkpoint)?,
                    )?;
                }
            }
            // // Debug
            // let final_checkpoint = lsp_codegen::build_debug_final_checkpoint!(#path);
            // std::fs::write(path2checkpoint.as_path(), &serde_json::to_string(&final_checkpoint)?)?;
            Ok(())
        }
    }
    .into()
}
