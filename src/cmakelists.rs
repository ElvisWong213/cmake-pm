use std::{ffi::OsStr, fmt::Display, fs::{self, File}, io::Read, path::Path, str::FromStr};

#[derive(Clone)]
pub(crate) struct CMakeListsVal {
    pub(crate) function: CMakeListsFunction,
    pub(crate) values: Vec<String>
}

impl CMakeListsVal {
    pub(crate)  fn new() -> Self {
        Self { function: CMakeListsFunction::None, values: vec![] }
    }

    pub(crate) fn clear(&mut self) {
        self.function = CMakeListsFunction::None;
        self.values.clear();
    }
}

impl Display for CMakeListsVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output: String = String::new();
        output.push_str(&self.function.to_string());
        output.push('(');
        let size = self.values.len();
        for (index, value) in self.values.iter().enumerate() {
            output.push_str(value);
            if index < size - 1 {
                output.push(' ');
            } else {
                output.push(')');
            }
        }
        write!(f, "{}", output)
    }
}

#[derive(Clone, PartialEq)]
pub(crate) enum CMakeListsFunction {
    CmakeMinimumRequired,
    Project,
    AddExecutable,
    None,
}

impl FromStr for CMakeListsFunction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cmake_minimum_required" => Ok(CMakeListsFunction::CmakeMinimumRequired),
            "project" => Ok(CMakeListsFunction::Project),
            "add_executable" => Ok(CMakeListsFunction::AddExecutable),
            _ => Err(())
        }
    }
}

impl Display for CMakeListsFunction  {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CMakeListsFunction::CmakeMinimumRequired => write!(f, "cmake_minimum_required"),
            CMakeListsFunction::Project => write!(f, "project"),
            CMakeListsFunction::AddExecutable => write!(f, "add_executable"),
            CMakeListsFunction::None => write!(f, ""),
        }
    }
}

pub(crate) fn read_cmakelists_file(path: &Path) -> Option<String> {
    if !path.is_file() || !path.exists() {
        println!("{} is not a file or does not exists", path.to_str().unwrap());
        return None;
    }
    let file_name = match path.file_name() {
        Some(val) => val,
        None => OsStr::new("")
    };
    if file_name != "CMakeLists.txt" {
        println!("{:?}", file_name);
        println!("CMakeLists file not found");
        return None
    }
    let mut text = String::new();
    let mut data = match File::open(path) {
        Ok(data) => data,
        Err(_) => {
            println!("Unable to open cmakelists file");
            return None
        }
    };
    if data.read_to_string(&mut text).is_ok() {
        return Some(text)
    }
    println!("Unable to read cmakelists file");
    None
}

pub(crate) fn write_to_cmakelists_file(path: &Path, data: &str) {
    match fs::write(path, data) {
        Ok(_) => {
            println!("New values is wrote to cmakelists file");
        },
        Err(_) => {
            println!("Unable to write cmakelists file");
        }
    }
}

pub(crate) fn phrase_cmakelists_file(cmake_vals: &mut Vec<CMakeListsVal>, text: &str) {
    let mut buffer = String::new();
    let mut cmake_lists_val = CMakeListsVal::new();
    let mut is_values = false;
    for c in text.chars() {
        match c {
            '(' => {
                is_values = true;
                match CMakeListsFunction::from_str(&buffer) {
                    Ok(function) => {
                        cmake_lists_val.function = function;
                        buffer.clear();
                    },
                    Err(_) => {
                        println!("Unable to phrase cmakelists file");
                        return;
                    }
                }
            },
            ')' => {
                if !is_values {
                    println!("Unable to phrase cmakelists file syntax error");
                    return;
                }
                if !buffer.is_empty() {
                    cmake_lists_val.values.push(buffer.clone());
                    buffer.clear();
                }
                cmake_vals.push(cmake_lists_val.clone());
                cmake_lists_val.clear();
                is_values = false;
            },
            '\n' | '\t' | '\r' => {
                continue;
            },
            ' ' => {
                if !buffer.is_empty() {
                    cmake_lists_val.values.push(buffer.clone());
                    buffer.clear();
                }
            }
            _ => {
                buffer.push(c);
            }
        }
    }
}
