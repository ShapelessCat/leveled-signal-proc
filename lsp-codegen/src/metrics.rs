use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Error;

use crate::MacroContext;

impl MacroContext {
    pub(crate) fn define_output_schema(&self) -> Result<TokenStream2, syn::Error> {
        let mut item_list = Vec::new();
        let derive_lines = match self.get_ir_data().measurement_policy.metrics_drain {
            lsp_ir::MetricsDrainType::Json => quote! {
                #[derive(Default, serde::Serialize)]
            },
        };
        for (name, data_spec) in &self.get_ir_data().measurement_policy.output_schema {
            let id = syn::Ident::new(name, self.span());
            let ty: syn::Type = syn::parse_str(&data_spec.typename)?;
            item_list.push(quote! {
                #id: #ty
            });
        }
        if let Some(conf) = &self
            .get_ir_data()
            .measurement_policy
            .complementary_output_config
        {
            for (name, data_spec) in &conf.schema {
                let id = syn::Ident::new(name, self.span());
                let ty: syn::Type = syn::parse_str(&data_spec.typename)?;
                item_list.push(quote! {
                    #id: #ty
                });
            }
        }
        Ok(quote! {
            #derive_lines
            #[allow(non_snake_case)]
            pub struct MetricsBag {
                #(#item_list,)*
            }
        })
    }

    pub(crate) fn define_previous_metrics_bag(&self) -> Result<TokenStream2, syn::Error> {
        let mut definition = quote! {
            let mut _previous_metrics_bag = MetricsBag::default();
        };
        if let Some(conf) = &self
            .get_ir_data()
            .measurement_policy
            .complementary_output_config
        {
            if let Some(switch) = &conf.reset_switch {
                let metric_name = syn::Ident::new(&switch.metric_name, self.span());
                let initial_value: syn::Expr = syn::parse_str(&switch.initial_value)?;
                definition.extend(quote! {
                    _previous_metrics_bag . #metric_name = #initial_value;
                })
            }
        }
        Ok(definition)
    }

    pub(crate) fn set_previous_metrics_bag_value(&self) -> Result<TokenStream2, syn::Error> {
        let code = if self
            .get_ir_data()
            .measurement_policy
            .complementary_output_config
            .is_some()
        {
            quote! {
                _previous_metrics_bag = _metrics_bag;
            }
        } else {
            quote! {()}
        };
        Ok(code)
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
            .unwrap_or_else(Error::into_compile_error);
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
            .unwrap_or_else(Error::into_compile_error);
        quote! {
            let __should_measure_left_side_limit : bool = (#signal_ref).clone();
        }
    }

    pub(crate) fn impl_should_output(&self) -> TokenStream2 {
        let measurement_node_ids = &self
            .get_ir_data()
            .measurement_policy
            .output_control_measurement_ids;

        match &measurement_node_ids[..] {
            [] => quote! { true },
            ids => {
                let conjunction = ids
                    .iter()
                    .map(|id| {
                        let measurement = self.get_node_ident(*id);
                        quote! { #measurement . measure(&mut update_context) }
                    })
                    .reduce(|cond0, cond1| {
                        quote! { #cond0 && #cond1 }
                    });

                quote! {
                    {
                        use lsp_runtime::signal_api::SignalMeasurement;
                        #conjunction
                    }
                }
            }
        }
    }

    pub(crate) fn impl_metrics_measuring(&self) -> TokenStream2 {
        let mut item_list = Vec::new();
        for (name, data_spec) in &self.get_ir_data().measurement_policy.output_schema {
            let id = syn::Ident::new(name, self.span());
            let node_ref = match data_spec.source {
                lsp_ir::NodeInput::Component { id } => self.get_node_ident(id),
                _ => panic!("Unsupported measurement input"),
            };
            item_list.push(quote! {
                #id: {
                    use lsp_runtime::signal_api::SignalMeasurement;
                    #node_ref . measure(&mut update_context)
                }
            });
        }

        let mut complementary_candidates_item_list = Vec::new();
        if let Some(conf) = &self
            .get_ir_data()
            .measurement_policy
            .complementary_output_config
        {
            for (name, data_spec) in &conf.schema {
                let id = syn::Ident::new(name, self.span());
                let node_ref = match data_spec.source {
                    lsp_ir::NodeInput::Component { id } => self.get_node_ident(id),
                    _ => panic!("Unsupported measurement input"),
                };
                let source_id = syn::Ident::new(&data_spec.source_metric_name, self.span());
                let candidates = (
                    quote! {
                        #id: {
                            use lsp_runtime::signal_api::SignalMeasurement;
                            #node_ref . measure(&mut update_context) - _previous_metrics_bag . #source_id
                        }
                    },
                    quote! {
                        #id: {
                            use lsp_runtime::signal_api::SignalMeasurement;
                            #node_ref . measure(&mut update_context)
                        }
                    },
                );
                complementary_candidates_item_list.push(candidates);
            }
            let (sub_previous, no_sub): (Vec<TokenStream2>, Vec<TokenStream2>) =
                complementary_candidates_item_list.into_iter().unzip();
            match &conf.reset_switch {
                None => quote! {
                    let _metrics_bag = MetricsBag {
                        #(#item_list,)*
                        #(#sub_previous,)*
                    };
                },
                Some(reset_switch) => {
                    let reset = syn::Ident::new(&reset_switch.metric_name, self.span());
                    let node_ref = match reset_switch.source {
                        lsp_ir::NodeInput::Component { id } => self.get_node_ident(id),
                        _ => panic!("Unsupported measurement input"),
                    };
                    quote! {
                        let #reset = {
                            use lsp_runtime::signal_api::SignalMeasurement;
                            #node_ref . measure(&mut update_context)
                        };
                        let _metrics_bag = if _previous_metrics_bag . #reset == #reset {
                            MetricsBag {
                                #(#item_list,)*
                                #(#sub_previous,)*
                            }
                        } else {
                            MetricsBag {
                                #(#item_list,)*
                                #(#no_sub,)*
                            }
                        };
                    }
                }
            }
        } else {
            quote! {
                let _metrics_bag = MetricsBag {
                    #(#item_list,)*
                };
            }
        }
    }
}
