use crate::MacroContext;
use lsp_ir::{Node, NodeInput};
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};

impl MacroContext {
    fn get_node_ident(&self, id: usize) -> syn::Ident {
        let node_id = syn::Ident::new(&format!("__lsp_node_{}", id), self.span());
        node_id
    }

    fn get_output_ident(&self, id: usize) -> syn::Ident {
        let output_var = syn::Ident::new(&format!("__lsp_output_buffer_{}", id), self.span());
        output_var
    }

    fn generate_lsp_node_declaration(&self, node: &Node) -> Result<TokenStream2, syn::Error> {
        let node_id = self.get_node_ident(node.id);
        let output_var = self.get_output_ident(node.id);
        let decl_namespace: syn::Path =
            syn::parse_str(&node.namespace).map_err(self.map_lsdl_error(node))?;
        let decl_expr: syn::Expr = syn::parse_str(&node.node_decl)?;
        let decl_code = quote! {
            let mut #node_id = {
                use #decl_namespace;
                #decl_expr
            };
            let mut #output_var;
        };
        Ok(decl_code.into())
    }

    pub(crate) fn define_lsp_nodes(&self) -> Result<TokenStream2, syn::Error> {
        let nodes = &self.get_ir_data().nodes;

        let mut decl_codes = Vec::new();

        for node in nodes.iter() {
            decl_codes.push(self.generate_lsp_node_declaration(node)?);
        }
        Ok(quote! {
            #(#decl_codes)*
        }
        .into())
    }

    fn generate_downstream_ref(
        &self,
        reference: &NodeInput,
        node: &Node,
    ) -> Result<TokenStream2, syn::Error> {
        let ret = match reference {
            NodeInput::InputBag => syn::Ident::new("input_state", self.span()).into_token_stream(),
            NodeInput::InputSignal { id } => {
                let id = syn::Ident::new(id.as_str(), self.span());
                quote! {
                    input_state.#id
                }
                .into()
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
                .into()
            }
            NodeInput::Component { id } => self.get_output_ident(*id).into_token_stream(),
            NodeInput::Tuple { values } => {
                let mut value_code = Vec::new();
                for value in values {
                    value_code.push(self.generate_downstream_ref(value, node)?);
                }
                quote! {
                    (
                        #(#value_code,)*
                    )
                }
                .into()
            }
        };
        Ok(ret)
    }

    fn generate_node_update_code(&self, node: &Node) -> Result<TokenStream2, syn::Error> {
        let node_ident = self.get_node_ident(node.id);
        let out_ident = self.get_output_ident(node.id);

        let mut upstream_refs = Vec::new();

        for up in node.upstreams.iter() {
            upstream_refs.push(self.generate_downstream_ref(up, node)?);
        }

        let input_expr = if upstream_refs.len() == 1 {
            let upstream = &upstream_refs[0];
            quote! {
                &#upstream
            }
        } else {
            quote! {
                (
                    #(&#upstream_refs,)*
                )
            }
        };
        Ok(quote! {
            {
                use lsp_runtime::signal::SignalProcessor;
                use lsp_runtime::measurement::Measurement;
                #out_ident = #node_ident . update(&mut update_context, #input_expr);
            }
        }
        .into())
    }

    pub(super) fn impl_nodes_update(&self) -> Result<TokenStream2, syn::Error> {
        let nodes = &self.get_ir_data().nodes;

        let mut update_code_vec = Vec::new();
        for node in nodes.iter() {
            update_code_vec.push(self.generate_node_update_code(node)?);
        }

        let out = quote! {
            #(#update_code_vec)*
        };
        Ok(out)
    }
}