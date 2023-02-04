use std::{process::Command, io::Write};
use std::fs::{self, File};
use std::path::Path;

use rusqlite::Connection;
use tempfile::TempDir;
use uuid::Uuid;

use rustless_shared::FunctionAppStatus;

use crate::docker;
use crate::storage;

/// Creates a zip file from the binary data and unzips it in the temporary directory
pub fn unzip_file_in_temp_dir(temp_dir: &TempDir, zip_file_data: &Vec<u8>) -> Result<(), String> {
    // Create a zip file in the temporary directory
    let zip_file_path = temp_dir.path().join("code.zip");
    let zip_file = File::create(&zip_file_path);
    let mut zip_file = match zip_file {
        Ok(file) => file,
        Err(e) => return Err(format!("Error creating zip file: {}", e))
    };

    // Write the zip file to the temporary directory
    if zip_file.write_all(&zip_file_data).is_err() {
        return Err("Error writing zip file".to_string());
    }

    // Unzip the file
    let unzip_result = Command::new("unzip")
        .arg("code.zip")
        .current_dir(temp_dir.path())
        .output();

    match unzip_result {
        Ok(_) => {},
        Err(e) => return Err(format!("Error unzipping file: {}", e))
    };

    // Delete the zip file
    let remove_result = fs::remove_file(&zip_file_path);
    match remove_result {
        Ok(_) => {},
        Err(e) => return Err(format!("Error deleting file: {}", e))
    };

    let paths = fs::read_dir(temp_dir.path());
    let paths = match paths {
        Ok(paths) => paths,
        Err(e) => return Err(format!("Error reading directory: {}", e))
    };
    
    if paths.count() != 1 {
        return Err("Zip file must contain exactly one folder".to_string());
    }

    // Get the output path
    let paths = fs::read_dir(temp_dir.path());
    let mut paths = match paths {
        Ok(paths) => paths,
        Err(e) => return Err(format!("Error reading directory: {}", e))
    };

    let path = paths.next();
    let path = match path {
        Some(path) => path,
        None => return Err("Error reading directory".to_string())
    };
    let path = match path {
        Ok(path) => path.path(),
        Err(e) => return Err(format!("Error reading directory: {}", e))
    };

    // Build the new output path
    let new_path = Path::join(temp_dir.path(), "code");

    let rename_result = fs::rename(path, new_path);
    match rename_result {
        Ok(_) => {},
        Err(e) => return Err(format!("Error renaming output folder: {}", e))
    };

    Ok(())
}

/// Gets if the function app is running under docker
pub fn get_function_app_status(conn: &Connection, id: &Uuid) -> Result<FunctionAppStatus, String> {
    // Get the function app name to prove we have an app registered with this ID
    let function_app_name = storage::get_function_app_name(&conn, &id);
    let function_app_name = match function_app_name {
        Ok(n) => n,
        Err(e) => {
            return Err(format!("Cannot get function app name from ID: {}. Does this function app exist?", e));
        }
    };

    // Check if the function app is running under docker
    let is_running = docker::is_container_running(&function_app_name);

    // Update the status in the database
    if is_running {
        Ok(FunctionAppStatus::Running)
    } else {
        Ok(FunctionAppStatus::Ready)
    }
}