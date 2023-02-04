use clap::{Parser, Subcommand};
use colored::Colorize;

mod cli;
mod code;
mod server;
mod storage;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds a function app to the rustless host
    AddFunctionApp { name: String, code_path: String },

    /// Updates the code of a function app
    UpdateFunctionApp { name: String, code_path: String },

    /// Sets the server to use when running commands
    SetServer {
        hostname: String,

        #[arg(default_value_t = 80)]
        port: u16,
    },

    /// Shows the current server
    ShowServer,

    /// Lists all the function apps on the current server
    List,

    /// Starts a function app
    Start { name: String },

    /// Gets the status of a function app
    Status { name: String },

    // /// Stops a function app
    // Stop { name: String },

    // /// Restarts a function app
    // Restart { name: String },

    // /// Deletes a function app
    // Delete { name: String },
}

/// Shows the CLI header
fn show_header() {
    println!("{}", format!(
        "\n
    ______          _   _                 _____  _     _____ 
    | ___ \\        | | | |               /  __ \\| |   |_   _|
    | |_/ /   _ ___| |_| | ___  ___ ___  | /  \\/| |     | |  
    |    / | | / __| __| |/ _ \\/ __/ __| | |    | |     | |  
    | |\\ \\ |_| \\__ \\ |_| |  __/\\__ \\__ \\ | \\__/\\| |_____| |_ 
    \\_| \\_\\__,_|___/\\__|_|\\___||___/___/  \\____/\\_____/\\___/ \n\n"
    )
    .bold()
    .blue());
}

#[tokio::main]
async fn main() {
    // Show the header
    show_header();

    // Parse the command line arguments
    let cli = Cli::parse();

    // Create the connection
    let conn = storage::create_connection();
    let conn = match conn {
        Ok(conn) => conn,
        Err(_) => {
            println!("{}", format!("Error connecting to database.").red().bold());
            std::process::exit(-1);
        }
    };

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::AddFunctionApp { name, code_path } => {
            cli::add_function_app(&conn, name, code_path).await;
        }

        Commands::UpdateFunctionApp { name, code_path } => {
            cli::update_function_app(&conn, name, code_path).await;
        }

        // Set the server
        Commands::SetServer { hostname, port } => {
            // Message the user
            println!("{}", format!("Setting server: {}:{}", hostname, port).green());

            if storage::set_server(conn, hostname, *port).await.is_err() {
                std::process::exit(-1)
            }
        }

        // Show the server that we have set. If this fails, report that no server is set
        Commands::ShowServer => match storage::get_server(&conn) {
            Ok(server) => println!("{}", format!("Server: {}:{}", server.hostname, server.port).green()),
            Err(_) => println!("{}", format!("No server set.").red())
        },

        // List out all the function apps on the server
        Commands::List => {
            cli::list_function_apps(&conn).await;
        }

        // Start a function app
        Commands::Start { name } => {
            cli::start_function_app(&conn, name).await;
        }

        Commands::Status { name } => {
            cli::get_function_app_status(&conn, name).await;
        }
    }
}
