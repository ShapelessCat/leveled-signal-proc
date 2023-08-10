use std::{env::args, fs::File, error::Error};

use lsp_ir::LspIr;

fn main() -> Result<(), Box<dyn Error>>{
    for ir_path in args().skip(1) {
        let input = File::open(&ir_path)?;
        let parse_result: Result<LspIr, _>= serde_json::from_reader(input);

        match parse_result {
            Ok(ir) => {
                println!("LSPIR in {} is well-formed.", ir_path);
                println!("Number of Nodes: {}", ir.nodes.len());
            }
            Err(err) => {
                println!("LSPIR in {} is malformed", ir_path);
                println!("Error: {}", err);
            }
        }
    }
    Ok(())
}