mod lsp_eventcount {
    lsp_codegen::include_lsp_ir!(lsp_main @ "../lsdl/examples/eventcount.json");
}

mod lsp_playtime {
    lsp_codegen::include_lsp_ir!(lsp_main @ "../lsdl/examples/playtime.json");
}

mod lsp_cidr {
    lsp_codegen::include_lsp_ir!(lsp_main @ "../lsdl/examples/cidr.json");
}

fn main() {
    use lsp_playtime::*;
    let fin = std::fs::File::open(std::env::args().skip(1).take(1).next().unwrap()).unwrap();
    let reader = std::io::BufReader::new(fin);
    let input_iter = serde_json::Deserializer::from_reader(reader)
        .into_iter::<InputSignalBagPatch>()
        .filter_map(Result::ok);
    lsp_main(input_iter, |m| {
       println!("{}", serde_json::to_string(&m).unwrap());
       Ok(())
    }).unwrap();
}