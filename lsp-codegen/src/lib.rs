use std::{path::{Path, PathBuf}, fs::File};

use anyhow::Error;
use lsp_ir::{LspIr, Schema, SchemaField, Node, NodeInput};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::{abort, proc_macro_error};
use syn::{parse_macro_input, LitStr};
use quote::{quote, ToTokens};

fn load_ir_from_file(input: &LitStr, full_path: &mut String) -> Result<LspIr, Error> {
    let ir_path_str = input.value();
    let ir_path: &Path = ir_path_str.as_ref();

    let manifest_dir_str = std::env::var("CARGO_MANIFEST_DIR")?;
    
    let ir_path = if ir_path.is_relative() {
        let mut ir_path_buf = PathBuf::from(manifest_dir_str);
        ir_path_buf.extend(ir_path.components());
        ir_path_buf
    } else {
        ir_path.to_path_buf()
    };

    full_path.clear();
    full_path.push_str(ir_path.to_str().unwrap());

    let input = File::open(&ir_path)?;
    Ok(serde_json::from_reader(input)?)
}

fn expand_input_state_item(id: &str, schema: &SchemaField, span: &Span) -> Result<TokenStream2, Error> {
    let field_id = syn::Ident::new(id, span.clone());
    let type_name : syn::Type = syn::parse_str(&schema.type_name)?;
    let clock_companion = syn::Ident::new(&schema.clock_companion, span.clone());
    let item_impl = quote! {
        pub #field_id: #type_name,
        pub #clock_companion: u64,
    };
    Ok(item_impl.into())
}

fn expand_input_patch_item(id: &str, schema: &SchemaField, span: &Span) -> Result<TokenStream2, Error> {
    let field_id = syn::Ident::new(id, span.clone());
    let type_name : syn::Type = syn::parse_str(&schema.type_name)?;
    let input_key = schema.input_key.as_str();
    let item_impl = quote! {
        #[serde(rename = #input_key)]
        pub #field_id : Option<#type_name>,
    };
    Ok(item_impl.into())
}

fn expand_input_patch_code(id: &str, schema: &SchemaField, span: &Span) -> Result<TokenStream2, Error> {
    let field_id = syn::Ident::new(id, span.clone());
    let clock_companion = syn::Ident::new(&schema.clock_companion, span.clone());
    let item_impl = quote! {
        if let Some(value) = patch.#field_id {
            self.#clock_companion += 1;
            self.#field_id = value;
        }
    };
    Ok(item_impl.into())
}

fn expand_input_state_bag(schema: &Schema, span: &Span) -> Result<TokenStream2, Error> {
    let mut item_impls = Vec::new();
    let mut diff_item_impls = Vec::new();
    let mut patch_code_impls = Vec::new();
    let type_name = syn::Ident::new(&schema.type_name, span.clone());
    let patch_type_name = syn::Ident::new(&format!("{}Patch", schema.type_name), span.clone());
    for (id, field) in schema.members.iter() {
        item_impls.push(expand_input_state_item(id, field, span)?);
        diff_item_impls.push(expand_input_patch_item(id, field, span)?);
        patch_code_impls.push(expand_input_patch_code(id, field, span)?);
    } 
    let item_impl = quote! {
        #[derive(Clone, Default)]
        pub struct #type_name {
            #(#item_impls)*
        }
        #[derive(serde::Deserialize, Clone)]
        pub struct #patch_type_name {
            timestamp: chrono::DateTime<chrono::Utc>,
            #(#diff_item_impls)*
        }
        impl lsp_runtime::WithTimestamp for #patch_type_name {
            fn timestamp(&self) -> lsp_runtime::Timestamp {
                self.timestamp.timestamp_nanos() as u64
            }
        }
        impl lsp_runtime::InputState for #type_name {
            type Event = #patch_type_name;
            fn patch(&mut self, patch: #patch_type_name) {
                #(#patch_code_impls)*
            }
            fn should_measure(&mut self) -> bool {
                //TODO
                true
            }
        }
    };
    Ok(item_impl.into())
}

#[proc_macro]
#[proc_macro_error]
pub fn define_lsp_input_schema(input: TokenStream) -> TokenStream {
    let ir_path_lit = parse_macro_input!(input as LitStr);
    let mut full_ir_path = String::new();
    let LspIr {schema, .. } = match load_ir_from_file(&ir_path_lit, &mut full_ir_path) {
        Ok(ir) => ir,
        Err(err) => {
            abort!{
                ir_path_lit.span(),
                "Failed to load LSPIR from file {} : {}",
                full_ir_path,
                err
            }
        }
    };

    let state_bag = match expand_input_state_bag(&schema, &ir_path_lit.span()) {
        Ok(state_bag) => state_bag,
        Err(err) => {
            abort!{
                ir_path_lit.span(),
                "Failed to expand input state bag: {}",
                err
            }
        }
    };

    let output = quote! {
        #state_bag
    };

    output.into()
}

fn get_node_ident(id: usize, span: &Span) -> syn::Ident {
    let node_id = syn::Ident::new(&format!("__lsp_node_{}", id), span.clone());
    node_id
}

fn get_output_ident(id: usize, span: &Span) -> syn::Ident {
    let output_var = syn::Ident::new(&format!("__lsp_output_buffer_{}", id), span.clone());
    output_var
}


fn generate_lsp_node_declaration(node: &Node, span: &Span) -> Result<TokenStream2, Error> {
    let node_id = get_node_ident(node.id, span);
    let output_var = get_output_ident(node.id, span);
    let decl_namespace : syn::Path = syn::parse_str(&node.namespace)?;
    let decl_expr : syn::Expr = syn::parse_str(&node.node_decl)?;
    let decl_code = quote! {
        let mut #node_id = {
            use #decl_namespace;
            #decl_expr
        };
        let mut #output_var;
    };
    Ok(decl_code.into())
}

#[proc_macro]
#[proc_macro_error]
pub fn generate_lsp_node_declarations(input: TokenStream) -> TokenStream {
    let ir_path_lit = parse_macro_input!(input as LitStr);
    let mut full_ir_path = String::new();
    let LspIr {nodes, .. } = match load_ir_from_file(&ir_path_lit, &mut full_ir_path) {
        Ok(ir) => ir,
        Err(err) => {
            abort!{
                ir_path_lit.span(),
                "Failed to load LSPIR from file {} : {}",
                full_ir_path,
                err
            }
        }
    };

    let mut decl_codes = Vec::new();

    for node in nodes.iter() {
        match generate_lsp_node_declaration(node, &ir_path_lit.span()) {
            Ok(code) => decl_codes.push(code),
            Err(err) => {
                abort!{
                    ir_path_lit.span(),
                    "Failed to declare LSP node: {}",
                    err
                }
            }
        }
    }

    let output = quote! {
        #(#decl_codes)*
    };

    output.into()
}

fn generate_downstream_ref(reference: &NodeInput, span: &Span) -> Result<TokenStream2, Error> {
    Ok(match reference {
        NodeInput::InputBag => syn::Ident::new("input_state", span.clone()).into_token_stream(),
        NodeInput::InputSignal { id } => {
            let id = syn::Ident::new(id.as_str(), span.clone());
            quote! {
                input_state.#id
            }.into()
        },
        NodeInput::Constant { value, type_name } => {
            let type_name : syn::Type = syn::parse_str(type_name)?;
            let value : syn::Expr = syn::parse_str(value)?;
            quote! {
                {
                    let _temp : #type_name = #value;
                    _temp
                }
            }.into()
        },
        NodeInput::Component { id } => get_output_ident(*id, span).into_token_stream(),
        NodeInput::Tuple { values } => {
            let mut value_code = Vec::new();
            for value in values {
                value_code.push(generate_downstream_ref(value, span)?);
            }
            quote!{
                (
                    #(#value_code,)*
                )
            }.into()
        }
    })
}

fn generate_node_update_code(node: &Node, span: &Span) -> Result<TokenStream2, Error> {
    let node_ident = get_node_ident(node.id, span);
    let out_ident = get_output_ident(node.id, span);

    let mut upstream_refs = Vec::new();

    for up in node.upstreams.iter() {
        upstream_refs.push(generate_downstream_ref(up, span)?);
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
    }.into())
}

#[proc_macro]
#[proc_macro_error]
pub fn generate_lsp_nodes_update_code(input: TokenStream) -> TokenStream {
    let ir_path_lit = parse_macro_input!(input as LitStr);
    let mut full_ir_path = String::new();
    let LspIr {nodes, .. } = match load_ir_from_file(&ir_path_lit, &mut full_ir_path) {
        Ok(ir) => ir,
        Err(err) => {
            abort!{
                ir_path_lit.span(),
                "Failed to load LSPIR from file {} : {}",
                full_ir_path,
                err
            }
        }
    };

    let mut update_code_vec = Vec::new();

    for node in nodes.iter() {
        match generate_node_update_code(node, &ir_path_lit.span()) {
            Ok(code) => update_code_vec.push(code),
            Err(err) => {
                abort!{
                    ir_path_lit.span(),
                    "Failed to generate update code for LSP node: {}",
                    err
                }
            }
        }
    }

    let out = quote! {
        #(#update_code_vec)*
    };

    out.into()
}