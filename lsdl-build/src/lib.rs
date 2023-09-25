use std::{
    process::Command, 
    borrow::Cow, 
    io::Read, 
    fs::{File, read_dir}, 
    path::{PathBuf, Path}, 
    sync::OnceLock, 
    env
};

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

fn get_python_interpreter() -> &'static str {
    static PYTHON_INTERPRETER : OnceLock<String> = OnceLock::new();
    PYTHON_INTERPRETER.get_or_init(||{
        find_python_interpreter().map_or_else(|| "python".to_string(), |x| x.to_string())
    }).as_str()
}

pub struct LsdlSourceDirectory{
    source_dir: PathBuf,
    ir_dir: PathBuf,
}

impl LsdlSourceDirectory {
    pub fn new<P: AsRef<Path> + ?Sized>(p: &P) -> LsdlSourceDirectory {
        LsdlSourceDirectory {
            source_dir: p.as_ref().to_owned(),
            ir_dir: p.as_ref().to_owned(),
        }
    }
    pub fn set_output_dir<P: AsRef<Path>>(mut self, path: &P) -> Self {
        self.ir_dir = path.as_ref().to_owned();
        self
    }
    pub fn for_each_lsdl_source<Handle>(&self, mut source_callback: Handle) -> Result<usize, anyhow::Error> 
    where
        Handle: FnMut(LsdlSource) -> Result<(), anyhow::Error>,
    {
        let source_iter = read_dir(&self.source_dir)?
            .filter_map(|entry_result| entry_result.ok())
            .filter(|entry| entry.file_type().map_or(false, |t| t.is_file()))
            .map(|entry| entry.path())
            .filter(|path| path.extension().map_or(false, |ext| ext == "py"));
        let mut count = 0;
        for source_file in source_iter {
            let source_file_name = source_file.file_name().unwrap();
            let mut ir_file_buf = self.ir_dir.to_path_buf();
            ir_file_buf.push(source_file_name);
            ir_file_buf.set_extension("json");
            let source_obj = LsdlSource {
                src_path: source_file,
                out_path: ir_file_buf,
                lsdl_runtime_dir: None,
            };
            source_callback(source_obj)?;
            count += 1;
        }
        Ok(count)
    }
}

pub struct LsdlSource {
    src_path: PathBuf,
    out_path: PathBuf,
    lsdl_runtime_dir: Option<PathBuf>,
}

impl LsdlSource {
    pub fn set_output_path<P: AsRef<Path> + ?Sized>(&mut self, p: &P) {
        self.out_path = p.as_ref().to_owned();
    }
    pub fn set_lsdl_runtime_path<P: AsRef<Path> + ?Sized>(&mut self, p: &P) {
        self.lsdl_runtime_dir = Some(p.as_ref().to_owned());
    }
    pub fn lower_to_ir(&self) -> Result<&Path, anyhow::Error> {
        if let Ok(metadata) = self.out_path.metadata() {
            let json_file_modify_time = metadata.modified()?;
            let lsdl_file_modify_time = self.src_path.metadata()?.modified()?;
            if lsdl_file_modify_time < json_file_modify_time {
                return Ok(&self.out_path);
            }
        }
        eprintln!("Lowering LSDL to LSPIR: {}", self.src_path.display());
        let mut py_instance = Command::new(get_python_interpreter());
        py_instance.arg(self.src_path.as_path())
            .stdout(File::create(self.out_path.as_path())?);
        if let Some(lsdl_runtime_dir) = self.lsdl_runtime_dir.as_ref() {
            let mut python_path = env::var("PYTHONPATH").unwrap_or_else(|_| "".to_string());
            python_path.push(':');
            python_path.push_str(lsdl_runtime_dir.to_string_lossy().as_ref());
            py_instance.env("PYTHONPATH", python_path);
        }
        let child_handle = py_instance.spawn()?.wait()?;
        println!("cargo:rerun-if-changed={}", self.src_path.display());
        if !child_handle.success() {
            std::fs::remove_file(self.out_path.as_path())?;
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Unable to lower LSDL source to LSPIR. (lsdl_file_name: {})", self.src_path.display())
            ))?
        }
        Ok(&self.out_path)
    }
}
