use std::collections::HashSet;

use bimap::BiMap;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::visit::Visit;
use syn::{Field, PathArguments, Type, TypePath};

use super::has_patchable_attr;

pub(crate) struct DerivationInfo {
    pub kept_types: HashSet<syn::Ident>,
    pub patchable_type_params: BiMap<syn::Ident, syn::Ident>,
    pub state_struct_fields: Vec<TokenStream2>,
}

impl DerivationInfo {
    pub fn new(stateful_fields: &[&Field]) -> Self {
        let mut kept_types: HashSet<syn::Ident> = HashSet::new();
        let mut patchable_type_params: BiMap<syn::Ident, syn::Ident> = BiMap::new();
        let mut state_struct_fields = vec![];

        stateful_fields.iter().for_each(|f| {
            let field_type = &f.ty;
            let is_patchable_field = has_patchable_attr(f);

            if !is_patchable_field {
                kept_types.extend(collect_used_generics(field_type));
                state_struct_fields.push(quote! { #f });
            } else if let syn::Type::Path(tp) = field_type {
                let type_path = to_ident(tp).expect("Only a generic type that can represent a measurement can have a `#[patchable]` attribute");
                let state_type: syn::Ident = format_ident!("{}State", type_path);
                let field = if let Some(field_name) = f.ident.as_ref() {
                    quote! { #field_name: #state_type }
                } else {
                    quote! { #state_type }
                };
                state_struct_fields.push(field);
                patchable_type_params.insert(type_path, state_type);
            } else {
                panic!("Only accept a type path");
            }
        });

        DerivationInfo {
            kept_types,
            patchable_type_params,
            state_struct_fields,
        }
    }
}

struct GenericsCollector {
    used_generics: Vec<syn::Ident>,
}

impl<'ast> Visit<'ast> for GenericsCollector {
    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        if node.qself.is_none() {
            if let Some(segment) = node.path.segments.first() {
                self.used_generics.push(segment.ident.clone());
            }
        }
        syn::visit::visit_type_path(self, node);
    }
}

fn collect_used_generics(ty: &Type) -> Vec<syn::Ident> {
    let mut collector = GenericsCollector {
        used_generics: Vec::new(),
    };
    collector.visit_type(ty);
    collector.used_generics
}

fn to_ident(type_path: &TypePath) -> Option<syn::Ident> {
    if type_path.qself.is_none() && type_path.path.segments.len() == 1 {
        let last_segment = type_path.path.segments.last()?;
        // Ensure the path segment has no arguments (e.g., it's not Vec<T> or Option<T>)
        if matches!(last_segment.arguments, PathArguments::None) {
            return Some(last_segment.ident.clone());
        }
    }
    None
}