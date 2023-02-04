use std::{process::Command};

use portpicker::pick_unused_port;
use rust_embed::RustEmbed;
use tempfile::TempDir;

/// Files from the Container folder
#[derive(RustEmbed)]
#[folder = "container/"]
struct ContainerFolder;

/// Gets if a docker container is running
pub fn is_container_running(function_app_name: &String) -> bool {
    let tag = get_container_tag(function_app_name);

    let output = Command::new("docker")
        .arg("ps")
        .output();
    
    let output = match output {
        Ok(output) => output,
        Err(_) => return false
    };

    let output = String::from_utf8(output.stdout);
    let output = match output {
        Ok(output) => output,
        Err(_) => return false
    };

    let output = output.split("\n");

    for line in output {
        if line.contains(&tag) {
            return true;
        }
    }

    false
}

/// Gets the next free port
fn get_next_free_port() -> Result<u16, String> {
    let port = pick_unused_port();
    match port {
        Some(port) => Ok(port),
        None => Err("Error getting next free port".to_string())
    }
}

/// Starts a docker container
pub fn start_function_app(function_app_name: &String) -> Result<u16, String> {
    let tag = get_container_tag(function_app_name);

    // get the next free port
    let port = get_next_free_port();
    let port = match port {
        Ok(port) => port,
        Err(e) => return Err(e)
    };

    // Start the container running
    let output = Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("-p")
        .arg(format!("{}:8080/tcp", port))
        .arg(tag)
        .output();
    
    // Check for any errors
    let output = match output {
        Ok(output) => output,
        Err(e) => return Err(format!("Error starting container: {}", e))
    };

    let output = String::from_utf8(output.stdout);
    let output = match output {
        Ok(output) => output,
        Err(e) => return Err(format!("Error starting container: {}", e))
    };

    if output.contains("Error") {
        return Err(format!("Error starting container: {}", output));
    }

    // Return the port
    Ok(port)
}

/// Creates a docker container tag from a function app name
fn get_container_tag(function_app_name: &String) -> String {
    format!("{}-container", function_app_name.replace(" ", "-").to_lowercase())
}

/// Builds a function app container.
/// 
/// This takes the source code that is uploaded, and builds a container
/// with docker that installs Rust, and then compiles the code that is sent
pub fn build_function_app_container(temp_dir: &TempDir, function_app_name: &String) -> Result<(), String> {
    // Create a Dockerfile in the temporary folder
    let dockerfile_path = temp_dir.path().join("Dockerfile");

    // Get the Dockerfile content from the embedded folder
    let dockerfile_source = match ContainerFolder::get("Dockerfile") {
        Some(dockerfile_source) => dockerfile_source,
        None => return Err("Error getting Dockerfile from container folder".to_string())
    };

    // Get the Dockerfile content as a string
    let dockerfile_content = std::str::from_utf8(dockerfile_source.data.as_ref());
    let dockerfile_content = match dockerfile_content {
        Ok(content) => content,
        Err(e) => return Err(format!("Error converting Dockerfile to string: {}", e))
    };

    // Write the Dockerfile to the temporary folder
    let dockerfile_result = std::fs::write(dockerfile_path, dockerfile_content);
    match dockerfile_result {
        Ok(_) => (),
        Err(e) => return Err(format!("Error writing Dockerfile: {}", e))
    };

    println!("Dockerfile created in {}", temp_dir.path().display());

    // Build the correct docker tag
    let tag = get_container_tag(function_app_name);

    // Build the Dockerfile and tag it with the name of the function app
    let dockerfile_command = format!("docker build -t {} .", tag);
    println!("Running command: {}", dockerfile_command);
    let dockerfile_command_result = Command::new("sh")
        .arg("-c")
        .arg(dockerfile_command)
        .current_dir(temp_dir.path())
        .output();

    match dockerfile_command_result {
        Ok(output) => {
            let std_out = String::from_utf8(output.stdout);
            let std_out = match std_out {
                Ok(std_out) => std_out,
                Err(e) => return Err(format!("Error converting Dockerfile output to string: {}", e))
            };

            println!("Dockerfile output: {}", std_out);
            
            if output.status.success() {
                println!("Dockerfile built successfully");
            } else {
                return Err(format!("Error building Dockerfile: {}", String::from_utf8_lossy(&output.stderr)))
            }
        },
        Err(e) => return Err(format!("Error building Dockerfile: {}", e))
    };

    Ok(())
}
