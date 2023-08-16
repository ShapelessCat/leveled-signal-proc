use crate::MacroContext;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

impl MacroContext {
    pub(crate) fn define_output_schema(&self) -> Result<TokenStream2, syn::Error> {
        let mut item_list = Vec::new();
        let derive_lines = match self.get_ir_data().measurement_policy.metrics_drain {
            lsp_ir::MetricsDrainType::Json => quote! {
                #[derive(serde::Serialize)]
            }
        };
        for (name, data_spec) in self.get_ir_data().measurement_policy.output_schema.iter() {
            let id = syn::Ident::new(name, self.span());
            let ty : syn::Type = syn::parse_str(&data_spec.typename)?;
            item_list.push(quote!{
                #id: #ty
            });
        }
        Ok(quote! {
            #derive_lines
            #[allow(non_snake_case)]
            pub struct MetricsBag {
                #(#item_list,)*
            }
        }.into())
    }

    pub(crate) fn impl_metrics_measuring(&self) -> TokenStream2 {
        let mut item_list = Vec::new();
        for (name, data_spec) in self.get_ir_data().measurement_policy.output_schema.iter() {
            let id = syn::Ident::new(name, self.span());
            let node_ref = match data_spec.source {
                lsp_ir::NodeInput::Component { id } => self.get_node_ident(id),
                _ => panic!("Unsupported measurement input")
            };
            item_list.push(quote!{
                #id: { 
                    use lsp_runtime::measurement::Measurement;
                    #node_ref . measure(&mut update_context)
                }
            });
        }
        quote!{
            let _metrics_bag = MetricsBag {
                #(#item_list,)*
            };
        }.into()
    }
}