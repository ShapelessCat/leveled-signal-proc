use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::MacroContext;

impl MacroContext {

    pub(crate) fn merge_simultaneous_moments(&self) -> TokenStream2 {
        let msm = self.get_ir_data().processing_policy.merge_simultaneous_moments;
        quote! { #msm }.into()
    }

}

