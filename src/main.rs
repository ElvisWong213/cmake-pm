use std::{env, ffi::OsStr, fmt::Display, fs::{self, File}, io::{Read, Write}, path::{Path, PathBuf}, str::FromStr};

use clap::{arg, Command};

fn main() {
    let matches = cli().get_matches();
    const CMAKE_VERSION: &str = "3.30";

    match matches.subcommand() {
        Some(("new", sub_matches)) => {
            let project_name  = sub_matches
                .get_one::<String>("PROJECT_NAME")
                .map(|s| s.as_str())
                .expect("project name is required");
            let mut current_dir = match env::current_dir() {
                Ok(p) => p,
                Err(_) => {
                    println!("Unable to get current directory");
                    return;
                }
            };
            current_dir.push(project_name);
            if !current_dir.exists() {
                match fs::create_dir(&current_dir) {
                    Ok(_) => {
                        println!("Project directory created");
                        // Create main file
                        let mut cpp_main_file_path = current_dir.clone();
                        cpp_main_file_path.push("main.cpp");
                        let mut cpp_main_file = match File::create(cpp_main_file_path) {
                            Ok(file) => {
                                println!("main file created");
                                file
                            },
                            Err(_) => {
                                println!("Unable create main file");
                                return;
                            }
                        };
                        let cpp_data = b"#include <iostream>\nint main () {\n\tstd::cout << \"HelloWorld!\" << std::endl;\n\treturn 0;\n}";
                        match cpp_main_file.write_all(cpp_data) {
                            Ok(_) => {
                                println!("Data wrote in main file")
                            }
                            Err(_) => {
                                println!("Unable write data to main file")
                            }
                        };
                        // Create CMakeList file
                        let mut cmake_file_path = current_dir.clone();
                        cmake_file_path.push("CMakeLists.txt");
                        let mut cmake_file = match File::create(cmake_file_path) {
                            Ok(file) => {
                                println!("CMakeList file created");
                                file
                            },
                            Err(_) => {
                                println!("Unable create CMakeList file");
                                return;
                            }
                        };
                        let cmake_data = format!("cmake_minimum_required(VERSION {})\nproject({})\nadd_executable({} main.cpp)", CMAKE_VERSION, project_name, project_name);
                        match cmake_file.write_all(cmake_data.as_bytes()) {
                            Ok(_) => {
                                println!("Data wrote in main file");
                            }
                            Err(_) => {
                                println!("Unable write data to main file");
                            }
                        };
                        // Create build directory
                        let mut buid_path = current_dir.clone();
                        buid_path.push("build");
                        match fs::create_dir(&buid_path) {
                            Ok(_) => {
                                println!("Build directory created");
                            },
                            Err(_) => {
                                println!("Unable create build directory");
                            }
                        }

                    },
                    Err(_) => {
                        println!("Unable create project directory")
                    }
                };
            } else {
                println!("Project: {} is exists", project_name);
            }
        },
        Some(("reload", _)) => {
            println!("relad");
        },
        Some(("add-class", sub_matches)) => {
            let new_file_name  = sub_matches
                .get_one::<String>("NEW_FILE_NAME")
                .map(|s| s.as_str())
                .expect("file name is required");
            let current_dir = match env::current_dir() {
                Ok(p) => p,
                Err(_) => {
                    println!("Unable to get current directory");
                    return;
                }
            };
            if !is_working_directory(&current_dir) {
                println!("{} is not working directory", current_dir.display());
                return;
            }
            let cpp_file = new_file_name.to_string() + ".cpp";
            let h_file = new_file_name.to_string() + ".h";
            if file_exists(&current_dir, &cpp_file) || file_exists(&current_dir, &h_file) {
                println!("Cpp or header file exists");
                return;
            }
            match File::create(&cpp_file) {
                Ok(mut file) => {
                    println!("Cpp file created");
                    let cpp_data = format!("#include \"{}.h\"", new_file_name);
                    match file.write_all(cpp_data.as_bytes()) {
                        Ok(_) => {
                            println!("Data wrote to cpp file");
                        },
                        Err(_) => {
                            println!("Unable to write data to cpp file");
                        }
                    };
                },
                Err(_) => {
                    println!("Unable create cpp file");
                }
            }
            match File::create(&h_file) {
                Ok(_) => {
                    println!("Header file created");
                },
                Err(_) => {
                    println!("Unable create header file");
                }
            }
            let mut cmake_file_path = current_dir.clone();
            cmake_file_path.push("CMakeLists.txt");
            if let Some(text) = read_cmakelists_file(&cmake_file_path) {
                let mut cmake_vals: Vec<CMakeListsVal> = vec![];
                phrase_cmakelists_file(&mut cmake_vals, &text);
                for val in &mut cmake_vals {
                    if val.function == CMakeListsFunction::AddExecutable {
                        val.values.push(cpp_file.clone());
                        val.values.push(h_file.clone());
                    }
                }
                let mut text = String::new();
                let size = cmake_vals.len();
                for (index, val) in cmake_vals.iter().enumerate() {
                    text.push_str(&val.to_string());
                    if index < size - 1 {
                        text.push('\n');
                    }
                }
                write_to_cmakelists_file(&cmake_file_path, &text);
            }
        },
        _ => unreachable!()
    }
}

fn is_working_directory(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }
    match dir.read_dir() {
        Ok(files) => {
            for file in files.flatten() {
                if file.file_name() == "CMakeLists.txt" {
                    return true;
                }
            }
            false
        },
        Err(_) => {
            println!("Unable to read directory");
            false
        }
    }
}

fn file_exists(path: &Path, file_name: &str) -> bool {
    let mut file_path: PathBuf = path.to_path_buf();
    file_path.push(file_name);
    file_path.exists()
}

fn read_cmakelists_file(path: &Path) -> Option<String> {
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

fn write_to_cmakelists_file(path: &Path, data: &str) {
    match fs::write(path, data) {
        Ok(_) => {
            println!("New values is wrote to cmakelists file");
        },
        Err(_) => {
            println!("Unable to write cmakelists file");
        }
    }
}

fn phrase_cmakelists_file(cmake_vals: &mut Vec<CMakeListsVal>, text: &str) {
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

#[derive(Clone)]
struct CMakeListsVal {
    function: CMakeListsFunction,
    values: Vec<String>
}

impl CMakeListsVal {
    fn new() -> Self {
        Self { function: CMakeListsFunction::None, values: vec![] }
    }

    fn clear(&mut self) {
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
enum CMakeListsFunction {
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

fn cli() -> Command {
    Command::new("cmake-pm")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("new")
            .arg(arg!(<PROJECT_NAME> "Project name"))
        )
        .subcommand(
            Command::new("reload")
        )
        .subcommand(
            Command::new("add-class")
            .arg(arg!(<NEW_FILE_NAME> "New File Name"))
        )
}
