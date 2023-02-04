use colored::Colorize;
use rusqlite::{Connection, Result, Error};

use crate::server;

/// The server details to store in the database
#[derive(Debug)]
pub struct Server {
    // The server hostname
    pub hostname: String,

    // The server port
    pub port: u16
}

/// Creates a connection to the database
pub fn create_connection() -> Result<Connection, String> {
    // Open the database file
    let conn_result = Connection::open("rustless_cli.db");

    // Check if the open actually worked
    let conn = match conn_result {
        Ok(conn) => conn,
        Err(_) => {
            return Err("Error connecting to database".to_string());
        }
    };

    // We need a table to store the server details. Create one if it doesn't exist
    match conn.execute(
        "CREATE TABLE IF NOT EXISTS servers (
                  id              INTEGER PRIMARY KEY,
                  hostname        TEXT NOT NULL,
                  port            INTEGER NOT NULL
                  )",
        [],
    ) {
        Ok(_) => {},
        Err(_ ) => {
            return Err("Error creating table".to_string());
        }
    };

    // Return the connection
    Ok(conn)
}

/// Adds a server to the database
/// 
/// We only store a single server in the database. This starts by deleting any existing servers
/// then adds the new one.
fn add_server(conn: &Connection, hostname: &String, port: u16) -> Result<(), Error> { 
    // Delete all the entries in the servers table
    let delete_sql = "DELETE FROM servers"; 
    let delete_result = conn.execute(
        &delete_sql,
        [],
    );

    // Check if the delete worked
    match delete_result {
        Ok(_) => {}
        Err(e) => return Err(e)
    };

    // Insert the new server
    let sql = format!("INSERT INTO servers (hostname, port) VALUES (?1, {})", port);
    let insert_result = conn.execute(
        &sql,
        &[&hostname],
    );

    // Check if the insert worked
    match insert_result {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}

/// Sets the server
/// 
/// This starts by testing the connection to the server, making sure it is valid. If so
/// the server is stored in the database. There can be only one server, so adding one deletes any
/// previous entry.
pub async fn set_server(conn: Connection, hostname: &String, port: u16) -> Result<(), String> {
    // Write to the console that we are testing the server
    let message = format !("Testing server: {}:{}...", hostname, port).blue();
    print!("{}", message);

    // TODO - add a spinner here for long running tests

    // Test the connection to the server
    let result = server::test_server(hostname, port).await;

    // Check if the test worked. If it did, write the server details to the database
    match result {
        Ok(_) => {
            // Write a message to the console to show it worked
            println!("✅");

            // Add the server to the database
            match add_server(&conn, hostname, port) {
                Ok(_) => {
                    let ok_message = format!("Server set!").green().bold();
                    println!("{}", ok_message);

                    Ok(())
                },
                Err(e) => Err(format!("Error adding server to storage: {}", e))
            }
        },
        Err(_) => {
            // If the server is not found, report back to the user
            println!("❌");
            let error_message = format!("Server {}:{} not found.\n", hostname, port).red().bold();
            println!("{}",error_message);

            // If there is a server already set, report this so the user knows which server will be used
            // If no server is set, also report this back to the user
            let current_message = match get_server(&conn) {
                Ok(server) => format!("Current server: {}:{}\n", server.hostname, server.port).bold().blue().to_string(),
                Err(_) => "No server set".bold().blue().to_string()
            };
            println!("{}", current_message);

            // Return an error
            Err("Server not found".to_string())
        }
    }
}

/// Gets the server from the database
pub fn get_server(conn: &Connection) -> Result<Server, Error> {
    // Create a statement to select the single server from the database
    let mut stmt = conn.prepare("SELECT hostname, port FROM servers LIMIT 1")?;
    let server_iter_result = stmt.query_map([], |row| {
        Ok(Server {
            hostname: row.get(0)?,
            port: row.get(1)?,
        })
    });

    // Check if the query worked
    let server_iter = match server_iter_result {
        Ok(server_iter) => server_iter,
        Err(e) => return Err(e)
    };

    // Get the first server from the iterator
    for server in server_iter {
        return Ok(server?);
    }

    // If there is no server, return an error
    Err(Error::QueryReturnedNoRows)
}