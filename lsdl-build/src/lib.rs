use std::{
    borrow::Cow,
    env,
    fs::{read_dir, File},
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
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
    static PYTHON_INTERPRETER: OnceLock<String> = OnceLock::new();
    PYTHON_INTERPRETER
        .get_or_init(|| {
            find_python_interpreter().map_or_else(|| "python".to_string(), |x| x.to_string())
        })
        .as_str()
}

pub struct LsdlSourceDirectory {
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
    pub fn set_output_dir<P: AsRef<Path> + ?Sized>(mut self, path: &P) -> Self {
        self.ir_dir = path.as_ref().to_owned();
        self
    }
    pub fn for_each_lsdl_source<Handle>(
        &self,
        mut source_callback: Handle,
    ) -> Result<usize, anyhow::Error>
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
            let mut source_obj: LsdlSource = (&source_file).into();
            source_obj.out_path = self.ir_dir.to_path_buf();
            source_obj.out_path.push(source_file_name);
            source_obj.out_path.set_extension("json");
            source_callback(source_obj)?;
            count += 1;
        }
        Ok(count)
    }
}

pub struct LsdlSource {
    src_path: PathBuf,
    out_path: PathBuf,
    lsdl_runtime_dir: PathBuf,
}

impl<'a, T: AsRef<Path> + ?Sized> From<&'a T> for LsdlSource {
    fn from(value: &'a T) -> Self {
        let source_file_name = value.as_ref().to_owned();
        let mut ir_file_name = source_file_name.clone();
        ir_file_name.set_extension("json");
        Self {
            src_path: source_file_name,
            out_path: ir_file_name,
            lsdl_runtime_dir: env!("CARGO_MANIFEST_DIR").into(),
        }
    }
}

impl LsdlSource {
    pub fn from_path<P: AsRef<Path> + ?Sized>(p: &P) -> Self {
        p.into()
    }
    pub fn get_lsdl_runtime_path(&self) -> &Path {
        &self.lsdl_runtime_dir
    }
    pub fn set_output_dir<P: AsRef<Path> + ?Sized>(&mut self, p: &P) -> &mut Self {
        if let Some(filename) = self.out_path.file_name() {
            let filename = filename.to_owned();
            self.out_path = p.as_ref().to_owned();
            self.out_path.push(filename)
        }
        self
    }
    pub fn set_output_path<P: AsRef<Path> + ?Sized>(&mut self, p: &P) -> &mut Self {
        self.out_path = p.as_ref().to_owned();
        self
    }
    pub fn set_lsdl_runtime_path<P: AsRef<Path> + ?Sized>(&mut self, p: &P) -> &mut Self {
        self.lsdl_runtime_dir = p.as_ref().to_owned();
        self
    }
    fn trigger_rebuild_for_extra_src(&self) -> Result<(), anyhow::Error> {
        let fp = BufReader::new(File::open(self.src_path.as_path())?);
        let mut src_prefix = self.src_path.clone();
        src_prefix.pop();
        for line in fp.lines().map_while(Result::ok) {
            if let Some(stripped) = line.strip_prefix('#') {
                if let Some(comment_body) = stripped.strip_prefix(|c| c == ' ' || c == '\t') {
                    const EXTRA_SRC_LIT: &str = "extra-src:";
                    if !comment_body.starts_with(EXTRA_SRC_LIT) {
                        continue;
                    }
                    let list = comment_body[EXTRA_SRC_LIT.len()..].split(|c| c == ' ' || c == '\t');
                    for item in list {
                        if !item.is_empty() {
                            src_prefix.push(item);
                            println!("cargo:rerun-if-changed={}", src_prefix.display());
                            src_prefix.pop();
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn trigger_rebuild_for_lsdl_package(&self, root: Option<PathBuf>) -> Result<(), anyhow::Error> {
        let root = if let Some(root) = root {
            root
        } else {
            let mut lsdl_package_path = self.lsdl_runtime_dir.clone();
            lsdl_package_path.push("lsdl");
            lsdl_package_path
        };
        for entry in read_dir(root)?.filter_map(|e| e.ok()) {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() && !metadata.is_symlink() {
                    self.trigger_rebuild_for_lsdl_package(Some(entry.path()))
                        .ok();
                }
                if metadata.is_file() && entry.path().extension().map_or(false, |e| e == "py") {
                    println!("cargo:rerun-if-changed={}", entry.path().display());
                }
            }
        }
        Ok(())
    }
    pub fn lower_to_ir(&self) -> Result<&Path, anyhow::Error> {
        eprintln!("Lowering LSDL to LSPIR: {}", self.src_path.display());
        let mut py_instance = Command::new(get_python_interpreter());
        py_instance
            .arg(self.src_path.as_path())
            .stdout(File::create(self.out_path.as_path())?);
        let mut python_path = env::var("PYTHONPATH").unwrap_or_else(|_| "".to_string());
        python_path.push(':');
        python_path.push_str(self.lsdl_runtime_dir.to_string_lossy().as_ref());
        py_instance.env("PYTHONPATH", python_path);
        let child_handle = py_instance.spawn()?.wait()?;
        println!("cargo:rerun-if-changed={}", self.src_path.display());
        self.trigger_rebuild_for_lsdl_package(None)?;
        self.trigger_rebuild_for_extra_src()?;
        if !child_handle.success() {
            std::fs::remove_file(self.out_path.as_path())?;
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Unable to lower LSDL source to LSPIR. (lsdl_file_name: {})",
                    self.src_path.display()
                ),
            ))?
        }
        Ok(&self.out_path)
    }
}
