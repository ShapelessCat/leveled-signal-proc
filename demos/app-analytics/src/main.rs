use std::{
    fs::File,
    io::{BufReader, Write},
};

use anyhow::Error;
use lsp_runtime::instrument::NoInstrument;

lsp_codegen::include_lsp_ir!(lsp_main @ "src/metrics-def.json");

fn main() -> Result<(), Error> {
    let path = std::env::args().nth(1).expect("Missing path argument");
    let fp = BufReader::new(File::open(path)?);
    let input_stream = serde_json::Deserializer::from_reader(fp)
        .into_iter()
        .filter_map(Result::ok);
    let mut instr_ctx = NoInstrument;
    let mut output = std::io::BufWriter::new(std::io::stdout());
    lsp_main(
        input_stream,
        move |metric| {
            output.write_all(serde_json::to_string(&metric)?.as_bytes())?;
            output.write_all(b"\n")?;
            Ok(())
        },
        &mut instr_ctx,
    )?;
    eprintln!("{}", instr_ctx);
    Ok(())
}
