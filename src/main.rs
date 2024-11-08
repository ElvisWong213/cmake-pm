mod cmakelists;

use std::{env, fs::{self, File}, io::Write, path::{Path, PathBuf}};
use clap::{arg, Command};

use cmakelists::{CMakeListsFunction, CMakeListsVal};

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
            if let Some(text) = cmakelists::read_cmakelists_file(&cmake_file_path) {
                let mut cmake_vals: Vec<CMakeListsVal> = vec![];
                cmakelists::phrase_cmakelists_file(&mut cmake_vals, &text);
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
                cmakelists::write_to_cmakelists_file(&cmake_file_path, &text);
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
