mod lsp {
    lsp_codegen::lsp_data_logic_main!(lsp_main @ "../lsdl/examples/cidr.json");
}

fn main() {
    let fin = std::fs::File::open("../input.json").unwrap();
    let reader = std::io::BufReader::new(fin);
    let input_iter = serde_json::Deserializer::from_reader(reader)
        .into_iter::<lsp::InputSignalBagPatch>()
        .filter_map(Result::ok);
    lsp::lsp_main(input_iter, |_| {Ok(())}).unwrap();
}