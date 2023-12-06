use lsdl_build::LsdlSourceDirectory;

fn main() {
    LsdlSourceDirectory::new("../lsdl/examples")
        .for_each_lsdl_source(|mut lsdl_src| {
            lsdl_src.set_lsdl_runtime_path("../lsdl/");
            Ok(lsdl_src
                .lower_to_ir()
                .map(|p| eprintln!("Built LSPIR {}", p.display()))?)
        })
        .expect("Unable to build example LSDL source");
}
