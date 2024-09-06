use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};

use lsp_ir::{Node, NodeInput};

use crate::{context::LsdlDebugInfo, MacroContext};

const NODE_ID_PREFIX: &str = "__lsp_node_";
const NODE_OUTPUT_BUFFER_PREFIX: &str = "__lsp_output_buffer_";
const ID_UPSTREAM_TEMPLATE: &str = "$";

impl MacroContext {
    pub(crate) fn get_node_ident(&self, id: usize) -> syn::Ident {
        syn::Ident::new(&format!("{}{}", NODE_ID_PREFIX, id), self.span())
    }

    fn get_output_ident(&self, id: usize) -> syn::Ident {
        syn::Ident::new(&format!("{}{}", NODE_OUTPUT_BUFFER_PREFIX, id), self.span())
    }

    fn get_decl_expr(node: &Node) -> Result<syn::Expr, syn::Error> {
        let processed_node_decl = node.node_decl.replace(ID_UPSTREAM_TEMPLATE, NODE_ID_PREFIX);
        syn::parse_str(&processed_node_decl)
    }

    fn generate_lsp_node_declaration(&self, node: &Node) -> Result<TokenStream2, syn::Error> {
        let node_id = self.get_node_ident(node.id);
        let output_var = self.get_output_ident(node.id);
        let decl_namespace: syn::Path =
            syn::parse_str(&node.namespace).map_err(self.map_lsdl_error(node))?;
        let decl_expr: syn::Expr = MacroContext::get_decl_expr(node)?;
        let decl_code = quote! {
            let mut #node_id = {
                use #decl_namespace;
                #decl_expr
            };
            let mut #output_var;
        };
        Ok(decl_code)
    }

    fn generate_lsp_node_state_loading(&self, node: &Node) -> Result<TokenStream2, syn::Error> {
        let node_id = self.get_node_ident(node.id);
        let key = syn::Index::from(node.id);
        let load_state = quote! {
            if let Some(state) = entries.get(& #key) {
                #node_id . patch(state);
            }
        };
        Ok(load_state)
    }

    pub(crate) fn define_lsp_nodes(&self) -> Result<TokenStream2, syn::Error> {
        let nodes = &self.get_ir_data().nodes;
        let mut decl_codes = Vec::new();
        for node in nodes {
            decl_codes.push(self.generate_lsp_node_declaration(node)?);
        }
        Ok(quote! {
            #(#decl_codes)*
        })
    }

    pub(crate) fn patch_lsp_nodes(&self) -> Result<TokenStream2, syn::Error> {
        let nodes = &self.get_ir_data().nodes;
        let mut load_state_stmts = Vec::new();
        for node in nodes {
            load_state_stmts.push(self.generate_lsp_node_state_loading(node)?);
        }
        Ok(quote! {
            if let Some(entries) = entries {
                use lsp_runtime::signal_api::Patchable;
                #(#load_state_stmts)*
            }
        })
    }

    pub(crate) fn generate_downstream_ref<T: LsdlDebugInfo>(
        &self,
        reference: &NodeInput,
        node: &T,
    ) -> Result<TokenStream2, syn::Error> {
        let ret = match reference {
            NodeInput::InputBag => syn::Ident::new("input_state", self.span()).into_token_stream(),
            NodeInput::InputSignal { id } => {
                let id = syn::Ident::new(id.as_str(), self.span());
                quote! {
                    input_state.#id
                }
            }
            NodeInput::Constant { value, type_name } => {
                let type_name: syn::Type =
                    syn::parse_str(type_name).map_err(self.map_lsdl_error(node))?;
                let value: syn::Expr = syn::parse_str(value).map_err(self.map_lsdl_error(node))?;
                quote! {
                    {
                        let _temp : #type_name = #value;
                        _temp
                    }
                }
            }
            NodeInput::Component { id } => self.get_output_ident(*id).into_token_stream(),
            NodeInput::Tuple { values } => {
                let mut value_code = Vec::new();
                for value in values {
                    value_code.push(self.generate_downstream_ref(value, node)?);
                }
                quote! {
                    (
                        #(#value_code.clone(),)*
                    )
                }
            }
        };
        Ok(ret)
    }

    fn generate_node_update_code(&self, node: &Node) -> Result<TokenStream2, syn::Error> {
        let node_ident = self.get_node_ident(node.id);
        let out_ident = self.get_output_ident(node.id);

        let mut upstream_refs = Vec::new();

        for up in &node.upstreams {
            upstream_refs.push(self.generate_downstream_ref(up, node)?);
        }

        let input_expr = if upstream_refs.len() == 1 {
            let upstream = &upstream_refs[0];
            quote! {
                &#upstream
            }
        } else {
            quote! {
                &(
                    #(#upstream_refs . clone(),)*
                )
            }
        };
        let mut before_node_update = quote!();
        let mut after_node_update = quote!();
        let node_id = node.id;
        if let Some(inst_id) = self.get_instrument_var() {
            before_node_update = quote! {
                #inst_id . node_update_begin(#node_id);
            };
            after_node_update = quote! {
                #inst_id . node_update_end(#node_id);
                #inst_id . handle_node_output(&#out_ident);
            }
        }
        let use_stmt = if node.is_measurement {
            quote! {
                use lsp_runtime::signal_api::SignalMeasurement;
            }
        } else {
            quote! {
                use lsp_runtime::signal_api::SignalProcessor;
            }
        };
        Ok(quote! {
            {
                #use_stmt;
                #before_node_update
                #out_ident = #node_ident . update(&mut update_context, #input_expr);
                #after_node_update
            }
        })
    }

    pub(super) fn impl_nodes_update(&self) -> Result<TokenStream2, syn::Error> {
        let nodes = &self.get_ir_data().nodes;

        let mut update_code_vec = Vec::new();
        for node in nodes {
            update_code_vec.push(self.generate_node_update_code(node)?);
        }

        let out = quote! {
            #(#update_code_vec)*
        };
        Ok(out)
    }

    // The second parameter is used for debugging.
    // In generated code it can be `ctx` or `update_context`, depends on this function call site.
    pub(crate) fn build_checkpoint(&self, context: TokenStream2) -> TokenStream2 {
        let mut insert_statements = Vec::new();
        let nodes = &self.get_ir_data().nodes;
        for node in nodes {
            let key = node.id;
            let node_ident = self.get_node_ident(node.id);
            insert_statements.push(quote! {
                let _ = entries.insert(#key, #node_ident . to_state());
            });
        }

        quote! {
            {
                use lsp_runtime::signal_api::Patchable;
                let mut entries = std::collections::HashMap::new();
                #(#insert_statements)*
                Checkpoint {
                    context_state: serde_json::to_string(& #context)?,
                    input_state: serde_json::to_string(&input_state)?,
                    entries,
                }
            }
        }
    }
}
