use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::MacroContext;

impl MacroContext {
    pub(crate) fn define_output_schema(&self) -> Result<TokenStream2, syn::Error> {
        let mut item_list = Vec::new();
        let derive_lines = match self.get_ir_data().measurement_policy.metrics_drain {
            lsp_ir::MetricsDrainType::Json => quote! {
                #[derive(serde::Serialize)]
            },
        };
        for (name, data_spec) in self.get_ir_data().measurement_policy.output_schema.iter() {
            let id = syn::Ident::new(name, self.span());
            let ty: syn::Type = syn::parse_str(&data_spec.typename)?;
            item_list.push(quote! {
                #id: #ty
            });
        }
        Ok(quote! {
            #derive_lines
            #[allow(non_snake_case)]
            pub struct MetricsBag {
                #(#item_list,)*
            }
        })
    }

    pub(crate) fn define_signal_trigger_measurement_ctx(&self) -> TokenStream2 {
        quote! {
            let mut __lsp_measurement_trigger_state = Default::default();
        }
    }

    pub(crate) fn impl_signal_triggered_measurement(&self) -> TokenStream2 {
        let signal_node = &self.get_ir_data().measurement_policy.measure_trigger_signal;
        let signal_ref = self
            .generate_downstream_ref(signal_node, &())
            .unwrap_or_else(|e| e.into_compile_error());
        quote! {
            let __signal_trigger_fired = {
                let next_state = (#signal_ref).clone();
                let ret = next_state != __lsp_measurement_trigger_state;
                __lsp_measurement_trigger_state = next_state;
                ret
            };
        }
    }

    pub(crate) fn impl_measurement_limit_side_control(&self) -> TokenStream2 {
        let signal_node = &self
            .get_ir_data()
            .measurement_policy
            .measure_left_side_limit_signal;
        let signal_ref = self
            .generate_downstream_ref(signal_node, &())
            .unwrap_or_else(|e| e.into_compile_error());
        quote! {
            let __should_measure_left_side_limit : bool = (#signal_ref).clone();
        }
    }

    pub(crate) fn impl_metrics_measuring(&self) -> TokenStream2 {
        let mut item_list = Vec::new();
        for (name, data_spec) in self.get_ir_data().measurement_policy.output_schema.iter() {
            let id = syn::Ident::new(name, self.span());
            let node_ref = match data_spec.source {
                lsp_ir::NodeInput::Component { id } => self.get_node_ident(id),
                _ => panic!("Unsupported measurement input"),
            };
            item_list.push(quote! {
                #id: {
                    use lsp_runtime::measurement::Measurement;
                    #node_ref . measure(&mut update_context)
                }
            });
        }
        quote! {
            let _metrics_bag = MetricsBag {
                #(#item_list,)*
            };
        }
    }
}
