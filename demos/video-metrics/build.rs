fn main() {
    lsdl_build::LsdlSource::from_path("metrics/metrics-def.py")
        .set_output_dir("src")
        .lower_to_ir()
        .expect("Unable to generate IR file from LSDL");
}
