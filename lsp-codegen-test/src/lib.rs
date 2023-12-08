use std::env::args;

use serde::{Deserialize, Serialize};

pub fn print_metrics_to_stdout<T: Serialize>(metrics: T) -> Result<(), anyhow::Error> {
    println!("{}", serde_json::to_string(&metrics)?);
    Ok(())
}

pub fn input_iter<InputTy>() -> Result<impl Iterator<Item = InputTy>, anyhow::Error>
where
    InputTy: Deserialize<'static>,
{
    let fin = std::fs::File::open(args().nth(1).unwrap()).unwrap();
    let reader = std::io::BufReader::new(fin);
    let input_iter = serde_json::Deserializer::from_reader(reader)
        .into_iter::<InputTy>()
        .filter_map(Result::ok);
    Ok(input_iter)
}

#[macro_export]
macro_rules! create_instrument_ctx {
    () => {
        {lsp_runtime::instrument::NoInstrument::default()}
    }
    //{lsp_runtime::instrument::InstrumentDataLogicRunningTime::default()}};
}
