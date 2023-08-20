use std::{process::Command, borrow::Cow, io::Read, fs::File};

fn test_python_interpreter(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_or(false, |mut child| {
            let exit_status = child.wait().map_or(false, |status| status.success());
            if let Some(mut stdout) = child.stdout {
                let mut python_version = String::new();
                if stdout.read_to_string(&mut python_version).is_err() {
                    return false;
                }
                python_version.starts_with("Python 3") && exit_status
            } else {
                false
            }
        })
}
fn find_python_interpreter() -> Option<Cow<'static, str>> {
    if let Ok(python) = std::env::var("PYTHON") {
        if test_python_interpreter(&python) {
            return Some(Cow::Owned(python));
        }
    }
    let python_interpreter_candidates = ["python3", "python", "python.exe", "python3.exe"];
    for py in python_interpreter_candidates {
        if test_python_interpreter(py) {
            return Some(Cow::Borrowed(py));
        }
    }
    None
}
fn main() {
    let py_interpreter = find_python_interpreter().expect("Cannot find python3 interpreter");

    for file in std::fs::read_dir("../lsdl/examples/").expect("Unable to read ldsl example dir") {
        if let Ok(file) = file {
            if file.path().extension().map(|s| s.to_str()).flatten() == Some("py") && file.file_type().map_or(false, |t| t.is_file()) {
                let mut file_out = file.path();
                file_out.set_extension("json");
                let mut py_instance = Command::new(py_interpreter.as_ref())
                    .arg(file.path())
                    .env("PYTHONPATH", format!("{}:{}", std::env::var("PYTHONPATH").unwrap_or("".to_string()), "../lsdl/"))
                    .stdout(File::create(file_out).unwrap())
                    .spawn()
                    .expect("Unable to execute LDSL source");
                let child = py_instance.wait().unwrap();
                if !child.success() {
                    panic!("Unable to execute LDSL source {}", file.file_name().to_string_lossy());
                }
            }
        }
    }
}