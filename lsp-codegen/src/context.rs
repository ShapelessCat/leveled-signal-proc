use std::{
    fs::File,
    path::{Path as FilePath, PathBuf},
};

use lsp_ir::{DebugInfo, LspIr, Node, SchemaField};
use proc_macro2::Span;
use syn::{parse::Parse, LitStr};

pub(crate) trait LsdlDebugInfo {
    fn debug_info(&self) -> Option<&DebugInfo>;
}

impl LsdlDebugInfo for Node {
    fn debug_info(&self) -> Option<&DebugInfo> {
        self.debug_info.as_ref()
    }
}

impl LsdlDebugInfo for SchemaField {
    fn debug_info(&self) -> Option<&DebugInfo> {
        self.debug_info.as_ref()
    }
}

pub(crate) struct MacroContext {
    ir_path_span: Span,
    ir_data: LspIr,
}

impl MacroContext {
    pub fn get_ir_data(&self) -> &LspIr {
        &self.ir_data
    }
    pub fn span(&self) -> Span {
        self.ir_path_span.clone()
    }
    pub fn map_lsdl_error<T: LsdlDebugInfo>(
        &self,
        ir_obj: &T,
    ) -> impl FnOnce(syn::Error) -> syn::Error {
        let debug_info = ir_obj.debug_info().cloned();
        move |err: syn::Error| {
            let message = if let Some(DebugInfo { file, line }) = debug_info {
                format!("{}\n{}:{}:{}", err.to_string(), file, line, 1)
            } else {
                err.to_string()
            };
            syn::Error::new(err.span(), message)
        }
    }
}

impl Parse for MacroContext {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path_lit: LitStr = input.parse()?;
        let ir_path_str = path_lit.value();
        let ir_path: &FilePath = ir_path_str.as_ref();

        let manifest_dir_str = std::env::var("CARGO_MANIFEST_DIR")
            .map_err(|e| syn::Error::new_spanned(&path_lit, e.to_string()))?;

        let ir_path = if ir_path.is_relative() {
            let mut ir_path_buf = PathBuf::from(manifest_dir_str);
            ir_path_buf.extend(ir_path.components());
            ir_path_buf
        } else {
            ir_path.to_path_buf()
        };

        let input =
            File::open(&ir_path).map_err(|e| syn::Error::new_spanned(&path_lit, e.to_string()))?;
        let ir_obj = serde_json::from_reader::<_, LspIr>(input).map_err(|e| {
            let error_message = format!(
                "IR parsing error: {msg}\nnote: Originate site {file}:{line}:{col}",
                msg = e.to_string(),
                file = ir_path_str,
                line = e.line(),
                col = e.column(),
            );
            syn::Error::new_spanned(&path_lit, error_message)
        })?;
        Ok(Self {
            ir_path_span: path_lit.span().clone(),
            ir_data: ir_obj,
        })
    }
}
