use std::{
    env::VarError,
    fs::File,
    path::{Path as FilePath, PathBuf},
};

use proc_macro2::{Ident, Span};
use syn::{parse::Parse, LitStr, Token};

use lsp_ir::{DebugInfo, LspIr, Node, SchemaField};

pub(crate) trait LsdlDebugInfo {
    fn debug_info(&self) -> Option<&DebugInfo>;
}

impl LsdlDebugInfo for () {
    fn debug_info(&self) -> Option<&DebugInfo> {
        None
    }
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
    instrument_var: Option<Ident>,
}

impl MacroContext {
    pub fn get_instrument_var(&self) -> Option<&Ident> {
        self.instrument_var.as_ref()
    }

    pub fn get_ir_data(&self) -> &LspIr {
        &self.ir_data
    }

    pub fn span(&self) -> Span {
        self.ir_path_span
    }

    pub fn map_lsdl_error<T: LsdlDebugInfo>(
        &self,
        ir_obj: &T,
    ) -> impl FnOnce(syn::Error) -> syn::Error {
        let debug_info = ir_obj.debug_info().cloned();
        move |err: syn::Error| {
            let message = if let Some(DebugInfo { file, line }) = debug_info {
                format!("{}\n{}:{}:{}", err, file, line, 1)
            } else {
                err.to_string()
            };
            syn::Error::new(err.span(), message)
        }
    }

    pub fn normalize_ir_path<P: AsRef<FilePath>>(ir_path: &P) -> Result<PathBuf, VarError> {
        let manifest_dir_str = std::env::var("CARGO_MANIFEST_DIR")?;
        let ir_path = ir_path.as_ref();

        Ok(if ir_path.is_relative() {
            let mut ir_path_buf = PathBuf::from(manifest_dir_str);
            ir_path_buf.extend(ir_path.components());
            ir_path_buf
        } else {
            ir_path.to_path_buf()
        })
    }

    pub(super) fn parse_ir_file(path_lit: &LitStr) -> syn::Result<LspIr> {
        let ir_path_str = path_lit.value();
        let ir_path = Self::normalize_ir_path(&ir_path_str)
            .map_err(|e| syn::Error::new_spanned(path_lit, e.to_string()))?;

        let input =
            File::open(ir_path).map_err(|e| syn::Error::new_spanned(path_lit, e.to_string()))?;
        let mut input_ir_obj = serde_json::from_reader::<_, LspIr>(input).map_err(|e| {
            let error_message = format!(
                "IR parsing error: {msg}\nnote: Originate site {file}:{line}:{col}",
                msg = e,
                file = ir_path_str,
                line = e.line(),
                col = e.column(),
            );
            syn::Error::new_spanned(path_lit, error_message)
        })?;
        input_ir_obj.normalize();
        Ok(input_ir_obj)
    }
}

impl Parse for MacroContext {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path_lit: LitStr = input.parse()?;
        let instrument_var: Option<Ident> = if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };
        let ir_obj = Self::parse_ir_file(&path_lit)?;
        Ok(Self {
            ir_path_span: path_lit.span(),
            ir_data: ir_obj,
            instrument_var,
        })
    }
}
