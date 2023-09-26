use std::{io::BufReader, fs::File};

use anyhow::Error;
use lsp_runtime::instrument::NoInstrument;

lsp_codegen::include_lsp_ir!(lsp_main @ "src/metrics-def.json");

fn print_to_stdout(metrics: MetricsBag) -> Result<(), Error> {
    println!("{}", serde_json::to_string(&metrics)?);
    Ok(())
}

fn main() -> Result<(), Error> {
    let path = std::env::args().nth(1).expect("Missing path argument");
    let fp = BufReader::new(File::open(path)?);
    let input_stream = serde_json::Deserializer::from_reader(fp).into_iter().filter_map(|r| r.ok());
    let mut instr_ctx = NoInstrument::default();
    lsp_main(input_stream, print_to_stdout, &mut instr_ctx)?;
    eprintln!("{}", instr_ctx);
    Ok(())
}