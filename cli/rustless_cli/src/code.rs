use std::path::PathBuf;
use std::{process::Command, path::Path};
use std::fs::{self, File};
use std::io::{BufReader, Read};

use colored::Colorize;

/// Compiles the code in the given path to verify it is valid
pub fn compile_code(code_path: &String) {
    // Create a new process to run the build command
    let compile_process = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(code_path)
        .output()
        .expect("Failed to run cargo build");

    // Check the result
    match compile_process.status.code() {
        Some(code) => {
            if code != 0 {
                println!(
                    "{}",
                    format!("Error compiling the function app code. Is the code valid?")
                        .red()
                        .bold()
                );
                std::process::exit(-1);
            }
        }
        None => {
            println!(
                "{}",
                format!("Error compiling the function app code. Is the code valid?")
                    .red()
                    .bold()
            );
            std::process::exit(-1);
        }
    };

    // Clean the code if everything worked so it is ready to zip and upload
    let compile_process = Command::new("cargo")
        .arg("clean")
        .current_dir(code_path)
        .output()
        .expect("Failed to run cargo clean");

    // Check the result
    match compile_process.status.code() {
        Some(code) => {
            if code != 0 {
                println!(
                    "{}",
                    format!("Error compiling the function app code. Is the code valid?")
                        .red()
                        .bold()
                );
                std::process::exit(-1);
            }
        }
        None => {
            println!(
                "{}",
                format!("Error compiling the function app code. Is the code valid?")
                    .red()
                    .bold()
            );
            std::process::exit(-1);
        }
    };
}

/// Uploads code to the server as a zip file
pub async fn zip_function_app_code(code_path: &String) -> PathBuf {
    // Get the folder to run this in - the parent folder of the path to the code
    let run_dir = Path::new(code_path).parent();
    let run_dir = match run_dir {
        Some(z) => z,
        None => {
            let error_message = format!("Error getting the parent directory of the code path").red().bold();
            println!("{}", error_message);
            std::process::exit(-1);
        }
    };

    // Get the folder in the run directory that contains the code
    let zip_dir = Path::new(code_path).strip_prefix(run_dir);
    let zip_dir = match zip_dir {
        Ok(z) => z,
        Err(_) => {
            let error_message = format!("Error getting the parent directory of the code path").red().bold();
            println!("{}", error_message);
            std::process::exit(-1);
        }
    };

    // Delete the existing zip file if it exists
    let zip_file = Path::new(run_dir).join("code.zip");
    let _ = fs::remove_file(&zip_file);
    
    let zip_result = Command::new("zip")
        .arg("-r")
        .arg("code.zip")
        .arg(zip_dir)
        .current_dir(run_dir)
        .output();

    match zip_result {
        Ok(zip_result) => {
            if zip_result.status.code() != Some(0) {
                let error_message = format!("Error zipping the code").red().bold();
                println!("{}", error_message);
                std::process::exit(-1);
            }
        }
        Err(e) => {
            let error_message = format!("Error zipping the code: {}", e).red().bold();
            println!("{}", error_message);
            std::process::exit(-1);
        }
    };

    zip_file
}

/// Converts the zip file to a base64 encoded string
pub fn zip_file_to_base64(zip_file: &PathBuf) -> String {
    // Open the zip file
    let file = File::open(zip_file).unwrap();

    // Read the file into a buffer
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();

    let result = reader.read_to_end(&mut buffer);
    match result {
        Ok(_) => (),
        Err(e) => {
            let error_message = format!("Error reading the zip file: {}", e).red().bold();
            println!("{}", error_message);
            std::process::exit(-1);
        }
    };

    // return the string as a bae64 encoded string
    base64::encode(&buffer)
}