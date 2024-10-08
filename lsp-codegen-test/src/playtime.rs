lsp_codegen::include_lsp_ir!(lsp_main @ "../lsdl/examples/playtime.json");

fn main() -> Result<(), anyhow::Error> {
    use lsp_codegen_test::{create_instrument_ctx, input_iter, print_metrics_to_stdout};
    let mut instr = create_instrument_ctx!();
    lsp_main(
        input_iter()?,
        print_metrics_to_stdout,
        &mut instr,
        std::path::Path::new("."),
    )?;
    eprintln!("{}", instr);
    Ok(())
}
