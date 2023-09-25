use lsdl_build::LsdlSourceDirectory;

fn main() {
    LsdlSourceDirectory::new("metrics")
        .for_each_lsdl_source(|src| {
            eprintln!("{}", src.get_lsdl_runtime_path().display());
            src.lower_to_ir()?;
            Ok(())
        })
        .expect("Unable to generate IR file from LSDL");
}