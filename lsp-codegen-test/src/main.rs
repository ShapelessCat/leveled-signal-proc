use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

use lsp_codegen::{define_data_logic_nodes, define_input_schema, impl_data_logic_updates};
use lsp_runtime::LspContext;
use serde_json::Deserializer;

define_input_schema!("../lsdl/examples/cidr.json");

fn main() {
    let fin = File::open("../input.json").unwrap();
    let mut _fout = BufWriter::new(File::open("/dev/null").unwrap());
    let reader = BufReader::new(fin);
    let mut ctx = LspContext::<_, InputSignalBag>::new(
        Deserializer::from_reader(reader)
            .into_iter::<InputSignalBagPatch>()
            .filter_map(Result::ok),
    );

    let mut input_state = Default::default();

    define_data_logic_nodes!("../lsdl/examples/cidr.json");

    while let Some(moment) = ctx.next_event(&mut input_state) {
        let mut update_context = ctx.borrow_update_context();

        if moment.should_update_signals() {
            impl_data_logic_updates!("../lsdl/examples/cidr.json");
        }
    }
}
