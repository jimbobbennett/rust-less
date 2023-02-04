use std::time::SystemTime;
use std::{path::PathBuf, time::Duration};

use chrono::prelude::{DateTime, Local, Utc};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use rusqlite::Connection;
use tokio::sync::mpsc::channel;
use tokio::time::sleep;
use uuid::Uuid;

use rustless_shared::FunctionAppStatus;

use crate::code;
use crate::server;
use crate::storage;

/// Formats a time into a string
fn format_date(date_time: SystemTime) -> String
{
    let dt: DateTime<Utc> = date_time.clone().into();
    format!("{}", dt.with_timezone(&Local).format("%d-%m-%Y %H:%M:%S"))
}

/// Creates a progress bar
fn create_progress_bar() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&["◜", "◠", "◝", "◞", "◡", "◟"]),
    );
    pb
}

/// Test that the code compiles
async fn test_compile_code(code_path: &String) {
    // Create a message channel to send messages to the progress bar
    let (tx, mut rx) = channel(1);

    let handle = tokio::spawn(async move {
        let pb = create_progress_bar();
        pb.set_message("Compiling function app...");

        while rx.try_recv().is_err() {
            pb.tick();
            sleep(Duration::from_millis(120)).await;
        }

        pb.finish_and_clear();
    });

    code::compile_code(code_path);

    tx.send(true).await.unwrap();

    handle.await.unwrap();
}

async fn get_new_id_for_function_app(conn: &Connection, name: &String) -> Uuid {
    // Create a message channel to send messages to the progress bar
    let (tx, mut rx) = channel(1);

    let handle = tokio::spawn(async move {
        let pb = create_progress_bar();
        pb.set_message("Registering app...");

        while rx.try_recv().is_err() {
            pb.tick();
            sleep(Duration::from_millis(120)).await;
        }

        pb.finish_and_clear();
    });

    // Check we have a server set
    let server = storage::get_server(&conn);

    // If not, report an error and exit
    if server.is_err() {
        let error_message =
            format!("No server set. Use the 'set-server' command to set the server.")
                .red()
                .bold();
        println!("{}", error_message);
        std::process::exit(-1);
    }

    // Construct the function app and get it's ID
    let id = server::register_function_app(conn, name).await;

    // Send a message to stop the spinner
    tx.send(id.to_string()).await.unwrap();

    handle.await.unwrap();

    id
}

/// Test that the code compiles
async fn get_base64_zip_file(zip_file: PathBuf) -> String {
    // Create a message channel to send messages to the progress bar
    let (tx, mut rx) = channel(1);

    let handle = tokio::spawn(async move {
        let pb = create_progress_bar();
        pb.set_message("Building packet to send to server...");

        while rx.try_recv().is_err() {
            pb.tick();
            sleep(Duration::from_millis(120)).await;
        }

        pb.finish_and_clear();
    });

    let zip_file_base64 = code::zip_file_to_base64(&zip_file);

    tx.send(true).await.unwrap();

    handle.await.unwrap();

    zip_file_base64
}

/// Test that the code compiles
async fn zip_code(code_path: &String) -> PathBuf {
    // Create a message channel to send messages to the progress bar
    let (tx, mut rx) = channel(1);

    let handle = tokio::spawn(async move {
        let pb = create_progress_bar();
        pb.set_message("Zipping function app...");

        while rx.try_recv().is_err() {
            pb.tick();
            sleep(Duration::from_millis(120)).await;
        }

        pb.finish_and_clear();
    });

    let zip_file = code::zip_function_app_code(code_path).await;

    tx.send(true).await.unwrap();

    handle.await.unwrap();

    zip_file
}

/// Sends the code to the server as a base64 encoded zip file
async fn send_zip_file_to_server(conn: &Connection, id: &Uuid, zip_file_base_64: &String) {
    // Create a message channel to send messages to the progress bar
    let (tx, mut rx) = channel(1);

    let handle = tokio::spawn(async move {
        let pb = create_progress_bar();
        pb.set_message("Sending function app code to server...");

        while rx.try_recv().is_err() {
            pb.tick();
            sleep(Duration::from_millis(120)).await;
        }

        pb.finish_and_clear();
    });

    // Send the app code
    server::post_app_code(conn, id, zip_file_base_64).await;

    tx.send(true).await.unwrap();

    handle.await.unwrap();
}

/// Gets the ID for the function app
async fn get_function_app_id(conn: &Connection, name: &String) -> Uuid {
    // Create a message channel to send messages to the progress bar
    let (tx, mut rx) = channel(1);

    let handle = tokio::spawn(async move {
        let pb = create_progress_bar();
        pb.set_message("Sending function app code to server...");

        while rx.try_recv().is_err() {
            pb.tick();
            sleep(Duration::from_millis(120)).await;
        }

        pb.finish_and_clear();
    });

    // Get the ID for the function app
    let id = server::get_id_for_function_app(conn, name).await;

    tx.send(true).await.unwrap();

    handle.await.unwrap();

    id
}

/// Start the function app
pub async fn start_function_app_on_server(conn: &Connection, name: &String) {
    // Create a message channel to send messages to the progress bar
    let (tx, mut rx) = channel(1);

    let handle = tokio::spawn(async move {
        let pb = create_progress_bar();
        pb.set_message("Starting the function app...");

        while rx.try_recv().is_err() {
            pb.tick();
            sleep(Duration::from_millis(120)).await;
        }

        pb.finish_and_clear();
    });

    // Get the function app ID
    let id = server::get_id_for_function_app(conn, name).await;

    // start the function app
    server::start_function_app(conn, &id).await;

    tx.send(true).await.unwrap();

    handle.await.unwrap();
}

async fn add_function_app_impl(conn: &Connection, name: &String, code_path: &String, id: Option<Uuid>) {
    // Compile the code to ensure it is valid before we start
    test_compile_code(code_path).await;
    println!("{}", format!("✅ Function app code compiled successfully").green());

    // Get the ID for the function app
    let id = match id {
        Some(id) => id,
        None => get_new_id_for_function_app(conn, name).await,
    };
    println!("{}", format!("✅ App registered with ID {}", id).green());

    // Upload the code for the app
    let zip_file = zip_code(code_path).await;
    println!("{}", format!("✅ Function app zipped").green());

    // Convert the Zip file to a base64 string
    let zip_file_base64 = get_base64_zip_file(zip_file).await;
    println!("{}", format!("✅ Function app packet built").green());

    // Send the request to the server
    send_zip_file_to_server(&conn, &id, &zip_file_base64).await;
    println!("{}", format!("✅ Function app code sent").green());
}

/// Adds a function app to the host
pub async fn add_function_app(conn: &Connection, name: &String, code_path: &String) {
    println!("{}", format!("Adding new function app '{}'", name).blue());

    add_function_app_impl(conn, name, code_path, None).await;

    println!("{}", format!("✅ Function app '{}' registered!", name).green());
}

/// Adds a function app to the host
pub async fn update_function_app(conn: &Connection, name: &String, code_path: &String) {
    println!("{}", format!("Adding new function app '{}'", name).blue());

    // get the ID for the function app
    let id = get_function_app_id(conn, name).await;
    println!("{}", format!("✅ Retrieved app id").green());

    // upload the code for the app
    add_function_app_impl(conn, name, code_path, Some(id)).await;

    println!("{}", format!("✅ Function app '{}' updated!", name).green());
}

/// Lists the function apps on the server
pub async fn list_function_apps(conn: &Connection) {
    // Get the function apps
    let function_apps = server::list_function_apps(&conn).await;

    if function_apps.is_empty() {
        println!("{}", format!("No function apps registered").blue());
        return;
    };

    // Build the table
    // First we need the size of the larges name
    let mut max_name_length = 0;
    for function_app in &function_apps {
        if function_app.name.len() > max_name_length {
            max_name_length = function_app.name.len();
        }
    }

    // Now we can build the table
    // The table is ID } Name | Status
    println!(
        "┌-{}-┬--------------------------------------┬----------------┬---------------------┐",
        "-".repeat(max_name_length)
    );
    println!(
        "| {}{} | {}                                   | {}         | {}        |",
        "Name".bold(),
        " ".repeat(max_name_length - 4),
        "ID".bold(),
        "Status".bold(),
        "Created date".bold()
    );
    println!(
        "|-{}-┼--------------------------------------┼----------------┼---------------------|",
        "-".repeat(max_name_length)
    );
    for function_app in &function_apps {
        let status_string = match function_app.status {
            FunctionAppStatus::NotRegistered => "Not registered".red(),
            FunctionAppStatus::Registered => "Registered".blue(),
            FunctionAppStatus::Running => "Running".green(),
            FunctionAppStatus::Ready => "Ready".blue(),
            FunctionAppStatus::Error => "Error".red(),
            FunctionAppStatus::Building => "Building".blue(),
        };
        let created_at = SystemTime::from(SystemTime::UNIX_EPOCH + Duration::from_secs(function_app.created_at));
        let created_at = format_date(created_at);

        println!(
            "| {}{} | {} | {}{} | {} |",
            function_app.name.blue().bold(),
            " ".repeat(max_name_length - function_app.name.len()),
            function_app.id,
            status_string,
            " ".repeat(14 - status_string.len()),
            created_at
        );
    }
    println!(
        "└-{}-┴--------------------------------------┴----------------┴---------------------┘",
        "-".repeat(max_name_length)
    );
}

/// Calls the server to start a function app
pub async fn start_function_app(conn: &Connection, name: &String) {
    println!("{}", format!("Adding new function app '{}'", name).blue());

    // Start the function app
    start_function_app_on_server(conn, name).await;

    println!("{}", format!("Function app '{}' running!", name).blue());
}

/// Calls the server to get the status of a function app
pub async fn get_function_app_status(conn: &Connection, name: &String) {
    let id = server::get_id_for_function_app(conn, name).await;
    let status = server::get_status_for_function_app(conn, &id).await;

    let status_string = match status {
        FunctionAppStatus::NotRegistered => "Not registered".red(),
        FunctionAppStatus::Registered => "Registered".blue(),
        FunctionAppStatus::Running => "Running".green(),
        FunctionAppStatus::Ready => "Ready".blue(),
        FunctionAppStatus::Error => "Error".red(),
        FunctionAppStatus::Building => "Building".blue(),
    };

    println!("Function app {} is {}", name, status_string);
}