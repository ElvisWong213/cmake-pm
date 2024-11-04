use std::{env, fs::{self, File}, io::Write};

use clap::{arg, Command};

fn main() {
    let matches = cli().get_matches();
    const CMAKE_VERSION: &str = "3.30";

    match matches.subcommand() {
        Some(("new", sub_matches)) => {
            let project_name  = sub_matches
                .get_one::<String>("PROJECTNAME")
                .map(|s| s.as_str())
                .expect("project name is required");
            let mut path = match env::current_dir() {
                Ok(p) => p,
                Err(_) => {
                    println!("Unable to get current directory");
                    return;
                }
            };
            path.push(project_name);
            if !path.exists() {
                match fs::create_dir(&path) {
                    Ok(_) => {
                        println!("Project directory created");
                        // Create main file
                        let mut cpp_main_file_path = path.clone();
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
                        let mut cmake_file_path = path.clone();
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
                                println!("Data wrote in main file")
                            }
                            Err(_) => {
                                println!("Unable write data to main file")
                            }
                        };
                    },
                    Err(_) => {
                        println!("Unable create project directory")
                    }
                };
            } else {
                println!("Project: {} is exists", project_name);
            }
        },
        Some(("reload", sub_matches)) => {
            println!("");
        },
        _ => unreachable!()
    }
}

fn cli() -> Command {
    Command::new("cmake-pm")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("new")
            .arg_required_else_help(true)
            .arg(arg!(<PROJECTNAME> "Project name"))
        )
        .subcommand(
            Command::new("reload")
        )
}
