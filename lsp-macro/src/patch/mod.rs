use std::collections::HashSet;

use bimap::BiMap;
use derivation_info::DerivationInfo;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{
    Data, DataStruct, DeriveInput, Expr, Field, Fields, GenericParam, Generics, Ident, Index,
    Token, Type, WhereClause,
};

mod derivation_info;

pub fn create_state_and_patchable_impl(input: &DeriveInput) -> TokenStream2 {
    if let Data::Struct(DataStruct { ref fields, .. }) = input.data {
        let all_owned_fields = input
            .generics
            .params
            .iter()
            .all(|gp| !matches!(gp, GenericParam::Lifetime(_)));
        assert!(all_owned_fields, "This `Patchable` derive macro doesn't support struct definition who has borrowed field(s)");
        let input_struct_name = &input.ident;
        let stateful_fields: Vec<&Field> = fields
            .iter()
            .filter(|f| !has_serde_skip_attr(f) && !is_phantom_data(f))
            .collect();
        let DerivationInfo {
            kept_types,
            patchable_type_params,
            state_struct_fields,
        } = DerivationInfo::new(&stateful_fields);
        let state_struct_type_params =
            collect_state_struct_type_params(&input.generics, &kept_types, &patchable_type_params);
        let state_name = quote::format_ident!("{}State", input_struct_name);

        // Define the state struct of LSP processors/measurements:
        let state_struct_def = build_state_struct(
            &state_name,
            fields,
            &state_struct_fields,
            &state_struct_type_params,
        );
        // Implement `Patchable` for the state struct of LSP processors/measurements:
        let patchable_impl = build_patch_impl(
            input_struct_name,
            &stateful_fields,
            &state_name,
            &input.generics,
            &state_struct_type_params,
            &patchable_type_params,
        );

        quote! {
            const _: () = {
                use serde as _serde;
                use lsp_runtime as _lsp_runtime;

                #state_struct_def
                #patchable_impl
            };
        }
    } else {
        panic!("This `Patchable` derive macro can only be applied on structs");
    }
}

fn build_state_struct(
    state_name: &Ident,
    fields: &Fields,
    state_struct_fields: &[TokenStream2],
    state_struct_type_params: &[&syn::Ident],
) -> TokenStream2 {
    let state_struct_body = if state_struct_fields.is_empty() {
        quote! {;}
    } else if matches!(fields, &Fields::Unnamed(_)) {
        quote! { <#(#state_struct_type_params),*>(#(#state_struct_fields),*); }
    } else {
        quote! { <#(#state_struct_type_params),*> { #(#state_struct_fields),* } }
    };
    quote! {
        #[derive(_serde::Deserialize)]
        pub struct #state_name #state_struct_body
    }
}

fn build_patch_impl(
    input_struct_name: &Ident,
    stateful_fields: &[&Field],
    state_name: &Ident,
    input_generics: &Generics,
    state_struct_type_params: &[&syn::Ident],
    patchable_type_params: &BiMap<syn::Ident, syn::Ident>,
) -> TokenStream2 {
    let (impl_generics, type_generics, _) = input_generics.split_for_impl();
    let where_clause = build_where_clause(
        input_generics.where_clause.as_ref(),
        state_struct_type_params,
        patchable_type_params,
    );
    let assoc_type_decl = build_associate_type_declaration(
        state_name,
        state_struct_type_params,
        patchable_type_params,
    );
    let patch_method_def = impl_patch_method(stateful_fields);
    quote! {
        impl #impl_generics _lsp_runtime::signal_api::Patchable
            for #input_struct_name #type_generics
        #where_clause
        {
            #assoc_type_decl
            #patch_method_def
        }
    }
}

fn has_serde_skip_attr(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        attr.path().is_ident("serde") && {
            attr.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
                .unwrap()
                .iter()
                .any(|expr| {
                    if let Expr::Path(ep) = expr {
                        ep.path.is_ident("skip")
                    } else {
                        false
                    }
                })
        }
    })
}

fn is_phantom_data(field: &Field) -> bool {
    if let Type::Path(p) = &field.ty {
        p.path
            .segments
            .first()
            .iter()
            .any(|s| s.ident == "PhantomData")
    } else {
        false
    }
}

fn collect_state_struct_type_params<'a>(
    input_generics: &'a Generics,
    kept_types: &HashSet<syn::Ident>,
    patchable_type_params: &'a BiMap<syn::Ident, syn::Ident>,
) -> Vec<&'a Ident> {
    let mut state_struct_type_params = vec![];
    input_generics.type_params().for_each(|tp| {
        if kept_types.contains(&tp.ident) {
            state_struct_type_params.push(&tp.ident);
        } else if let Some(state_type) = patchable_type_params.get_by_left(&tp.ident) {
            state_struct_type_params.push(state_type)
        }
    });
    state_struct_type_params
}

fn build_associate_type_declaration(
    state_name: &syn::Ident,
    state_struct_type_params: &[&syn::Ident],
    patchable_type_params: &BiMap<syn::Ident, syn::Ident>,
) -> TokenStream2 {
    let mut states = vec![];

    state_struct_type_params.iter().for_each(|&tpe| {
        if let Some(t) = patchable_type_params.get_by_right(tpe) {
            states.push(quote! { #t::State });
        } else {
            states.push(quote! { #tpe });
        }
    });

    quote! {
        type State = #state_name <#(#states),*>;
    }
}

fn build_where_clause(
    input_where_clause: Option<&WhereClause>,
    state_struct_type_params: &[&syn::Ident],
    patchable_type_params: &BiMap<syn::Ident, syn::Ident>,
) -> TokenStream2 {
    let bounds = state_struct_type_params.iter().map(|&tpe| {
        if let Some(t) = patchable_type_params.get_by_right(tpe) {
            quote! { #t: _serde::Serialize + Patchable }
        } else {
            quote! { #tpe: _serde::Serialize + _serde::de::DeserializeOwned }
        }
    });
    let normalized_input_where_clause = input_where_clause
        .map(|w| {
            if w.predicates.empty_or_trailing() {
                quote! { #w }
            } else {
                quote! { #w, }
            }
        })
        .unwrap_or(quote! { where });

    quote! {
        #normalized_input_where_clause
        #(#bounds),*
    }
}

fn impl_patch_method(stateful_fields: &[&Field]) -> TokenStream2 {
    if stateful_fields.is_empty() {
        quote! {
            fn patch(&mut self, _state: &str) {} // Empty state, nothing to `patch`

            fn patch_from(&mut self, _state: Self::State) {}
        }
    } else {
        let state_update = stateful_fields.iter().enumerate().map(|(idx, f)| {
            let name = f.ident.as_ref().map(|id| quote! { #id }).unwrap_or({
                let i = Index::from(idx);
                quote! { #i }
            });
            if has_patchable_attr(f) {
                quote! { self . #name . patch_from ( state . #name ) }
            } else {
                quote! { self . #name = state . #name }
            }
        });

        quote! {
            fn patch_from(&mut self, state: Self::State) {
                #(#state_update);*;
            }
        }
    }
}

fn has_patchable_attr(field: &syn::Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("patchable"))
}
