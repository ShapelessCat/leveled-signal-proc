use lsdl_build::LsdlSource;

fn main() {
    let mut metrics_src : LsdlSource = "metrics/metrics-def.py".into();
    metrics_src.set_output_dir("src").lower_to_ir().expect("Unable to generate IR from LSDL");
}