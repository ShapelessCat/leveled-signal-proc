use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use lsp_ir::{EnumVariantInfo, SchemaField, SignalBehavior};

use heck::ToUpperCamelCase;

use crate::MacroContext;

impl MacroContext {
    fn expand_input_state_item(
        &self,
        id: &str,
        schema: &SchemaField,
    ) -> Result<TokenStream2, syn::Error> {
        let field_id = syn::Ident::new(id, self.span());
        let type_name: syn::Type = if schema.enum_variants.is_empty() {
            syn::parse_str(&schema.type_name).map_err(self.map_lsdl_error(schema))?
        } else {
            syn::parse_str(&id.to_upper_camel_case()).map_err(self.map_lsdl_error(schema))?
        };
        let clock_companion = syn::Ident::new(&schema.clock_companion, self.span());
        let item_impl = quote! {
            pub #field_id: #type_name,
            pub #clock_companion: u64,
        };
        Ok(item_impl)
    }

    fn expand_input_patch_item(
        &self,
        id: &str,
        schema: &SchemaField,
    ) -> Result<TokenStream2, syn::Error> {
        let field_id = syn::Ident::new(id, self.span());
        let type_name: syn::Type = if schema.enum_variants.is_empty() {
            syn::parse_str(&schema.type_name).map_err(self.map_lsdl_error(schema))?
        } else {
            syn::parse_str(&id.to_upper_camel_case()).map_err(self.map_lsdl_error(schema))?
        };
        let input_key = schema.input_key.as_str();
        let item_impl = quote! {
            #[serde(rename = #input_key)]
            pub #field_id : Option<#type_name>,
        };
        Ok(item_impl)
    }

    fn expand_input_patch_code(
        &self,
        id: &str,
        schema: &SchemaField,
    ) -> Result<TokenStream2, syn::Error> {
        let field_id = syn::Ident::new(id, self.span());
        let clock_companion = syn::Ident::new(&schema.clock_companion, self.span());
        let if_arm = quote! {
            if let Some(value) = patch.#field_id {
                self.#clock_companion += 1;
                self.#field_id = value;
            }
        };

        let item_impl: TokenStream2 = match &schema.signal_behavior {
            SignalBehavior::Persist => if_arm,
            SignalBehavior::Reset { default_expr } => {
                let default_expr: syn::Expr = syn::parse_str(default_expr)?;
                quote! {
                    #if_arm
                    else {
                        self.#clock_companion += 1;
                        self.#field_id = #default_expr;
                    }
                }
            }
        };
        Ok(item_impl)
    }

    fn expand_enum_types(&self) -> Result<TokenStream2, syn::Error> {
        let schema = &self.get_ir_data().schema;

        let enum_defs = schema
            .members
            .iter()
            .filter(|(_, v)| !v.enum_variants.is_empty())
            .map(|(k, v)| self.build_enum(k, &v.enum_variants));

        Ok(quote! {
            #(#enum_defs)*
        })
    }

    fn build_enum(&self, name: &str, variants_info: &[EnumVariantInfo]) -> TokenStream2 {
        let name = syn::Ident::new(&name.to_upper_camel_case(), self.span());
        let attributes = variants_info.iter().map(
            |EnumVariantInfo {
                 input_value, ..
             }| {
                quote! { #[serde(rename = #input_value)] }
            },
        );
        let variants = variants_info.iter().map(
            |EnumVariantInfo {
                 variant_name, ..
             }| { syn::Ident::new(variant_name, self.span()) },
        );
        let match_branches = variants_info.iter().map(
            |EnumVariantInfo {
                 variant_name,
                 input_value,
             }| {
                let variant = syn::Ident::new(variant_name, self.span());
                quote! { #name::#variant => #input_value }
            },
        );
        quote! {
            #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
            pub enum #name {
                #[default]
                #(#attributes #variants),*
            }

            impl std::fmt::Display for #name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let repr = match self {
                        #(#match_branches),*
                    };
                    write!(f, "{}", repr)
                }
            }
        }
    }

    pub(super) fn expand_input_state_bag(&self) -> Result<TokenStream2, syn::Error> {
        let schema = &self.get_ir_data().schema;
        let span = &self.span();
        let mut item_impls = Vec::new();
        let mut diff_item_impls = Vec::new();
        let mut patch_code_impls = Vec::new();
        let type_name = syn::Ident::new(&schema.type_name, *span);
        let patch_type_name = syn::Ident::new(&format!("{}Patch", schema.type_name), *span);

        // Add a companion clock signal to the input_state_bag itself, rather than to a specific field of the input_state_bag.
        let input_state_bag_clock = syn::Ident::new("_clock", self.span());
        let input_state_bag_clock_item = quote! { pub #input_state_bag_clock: u64, };
        item_impls.push(input_state_bag_clock_item);
        let input_state_bag_clock_update_item = quote! { self.#input_state_bag_clock += 1; };
        patch_code_impls.push(input_state_bag_clock_update_item);

        for (id, field) in &schema.members {
            item_impls.push(self.expand_input_state_item(id, field)?);
            diff_item_impls.push(self.expand_input_patch_item(id, field)?);
            patch_code_impls.push(self.expand_input_patch_code(id, field)?);
        }
        let timestamp_key = &schema.patch_timestamp_key;
        let measure_at_event_filter: syn::Expr = {
            let predicate = &self
                .get_ir_data()
                .measurement_policy
                .measure_at_event_filter;
            syn::parse_str(predicate)?
        };
        let enum_defs = self.expand_enum_types()?;
        let item_impl = quote! {
            #enum_defs

            #[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
            pub struct #type_name {
                #(#item_impls)*
            }
            #[derive(serde::Deserialize, Clone, Debug)]
            pub struct #patch_type_name {
                #[serde(rename = #timestamp_key)]
                timestamp: chrono::DateTime<chrono::Utc>,
                #(#diff_item_impls)*
            }
            impl lsp_runtime::context::WithTimestamp for #patch_type_name {
                fn timestamp(&self) -> lsp_runtime::Timestamp {
                    self.timestamp
                        .timestamp_nanos_opt()
                        .expect("value can not be represented in a timestamp with nanosecond precision.")
                        as lsp_runtime::Timestamp
                }
            }
            impl lsp_runtime::context::InputSignalBag for #type_name {
                type Input = #patch_type_name;
                fn patch(&mut self, patch: #patch_type_name) {
                    #(#patch_code_impls)*
                }
                fn should_measure(&mut self) -> bool {
                    (#measure_at_event_filter)(self)
                }
            }
        };
        Ok(item_impl)
    }
}
