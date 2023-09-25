use lsdl_build::LsdlSourceDirectory;

fn main() {
    LsdlSourceDirectory::new("metrics")
        .set_output_dir("src")
        .for_each_lsdl_source(|src| {
            src.lower_to_ir()?;
            Ok(())
        })
        .expect("Unable to generate IR file from LSDL");
}