use std::{fs::File, io::{BufWriter, BufReader}};

use lsp_codegen::{define_lsp_input_schema, generate_lsp_node_declarations, generate_lsp_nodes_update_code};
use lsp_runtime::LspContext;
use serde_json::Deserializer;

define_lsp_input_schema!("../lsdl/examples/cidr.json");

fn main() {
    let fin = File::open("../input.json").unwrap();
    let mut _fout = BufWriter::new(File::open("/dev/null").unwrap());
    let reader = BufReader::new(fin);
    let mut ctx = LspContext::<_, InputSignalBag>::new(Deserializer::from_reader(reader).into_iter::<InputSignalBagPatch>().filter_map(Result::ok));

    let mut input_state = Default::default();

    generate_lsp_node_declarations!("../lsdl/examples/cidr.json");

    while let Some(moment) = ctx.next_event(&mut input_state) {
        let mut update_context = ctx.borrow_update_context();

        if moment.should_update_signals() {
            generate_lsp_nodes_update_code!("../lsdl/examples/cidr.json");
        }
    }
}