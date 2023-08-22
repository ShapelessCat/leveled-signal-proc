use std::{env::args, fs::File};
use anyhow::Error;

use lsp_ir::LspIr;

fn main() -> Result<(), Error> {
    for ir_path in args().skip(1) {
        let input = File::open(&ir_path)?;
        let parse_result: Result<LspIr, _> = serde_json::from_reader(input);

        match parse_result {
            Ok(ir) => {
                eprintln!(
                    "LSPIR in {} is valid. (# of nodes: {}, # of metrics: {}).",
                    ir_path,
                    ir.nodes.len(),
                    ir.measurement_policy.output_schema.len()
                );
            }
            Err(err) => {
                eprintln!("LSPIR in {} is malformed: {}", ir_path, err);
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
