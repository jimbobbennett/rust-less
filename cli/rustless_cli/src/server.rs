use colored::Colorize;
use reqwest::{Client, Error};
use rusqlite::{Connection, Result};
use uuid::Uuid;

use rustless_shared::{FunctionApp, FunctionAppStatus, FunctionAppStatusResult, FunctionAppNameRequest};

use crate::storage;

/// Builds a HTTPS request client. In debug mode, this ignores invalid certs so it can be run locally
#[cfg(debug_assertions)]
fn get_builder() -> Result<Client, Error> {
    Client::builder().danger_accept_invalid_certs(true).build()
}

/// Builds a HTTPS request client. In release mode, this does not invalid certs so it can't be run locally
#[cfg(not(debug_assertions))]
fn get_builder() -> Result<Client, Error> {
    Client::builder().build()
}

/// Test the server to see if it is available
///
/// The server will respond on a request to url:port/hello with Hello from rustless!
/// and a 200 status code if it is a valid server
pub async fn test_server(hostname: &String, port: u16) -> Result<(), String> {
    // Create the url from the hostname and port
    let url = format!("https://{}:{}/hello", hostname, port);

    // Get the request client
    let builder = get_builder();
    let client = match builder {
        Ok(client) => client,
        Err(e) => return Err(format!("Error creating client: {}", e)),
    };

    // Make the request
    let res = client.get(url).send().await;

    // Check the response
    match res {
        Ok(res) => {
            // If the server is correct, we should get a 200 status code
            if res.status() != 200 {
                return Err(format!("Server returned status code: {}", res.status()));
            }

            // If we got a 200, check the test we get back to see if it matches what is expected
            // If not, return an error
            match res.text().await {
                Ok(text) => {
                    if text != "Hello from rustless!" {
                        return Err(format!("Server returned unexpected text: {}", text));
                    }
                }
                Err(e) => {
                    return Err(format!("Error reading response text: {}", e));
                }
            }

            // If everything works, return Ok.
            Ok(())
        }
        Err(err) => Err(format!("Error: {}", err)),
    }
}

/// Registers a function app with the server
pub async fn register_function_app(conn: &Connection, name: &String) -> Uuid {
    let result = call_post_function_app(conn, name).await;

    match result {
        Ok(id) => id,
        Err(e) => {
            let error_message = format!("Error adding function app: {}", e).red().bold();
            println!("{}", error_message);
            std::process::exit(-1);
        }
    }
}

/// Calls the server to add a function app
async fn call_post_function_app(conn: &Connection, name: &String) -> Result<Uuid, String> {
    // Get the server from the database
    let server = match storage::get_server(&conn) {
        Ok(server) => server,
        Err(_) => {
            return Err(
                "No server set. Use the 'set-server' command to set the server.".to_string(),
            )
        }
    };

    let test_result = test_server(&server.hostname, server.port).await;
    if test_result.is_err() {
        return Err(format!(
            "Error testing server: {}. Is the server set correctly",
            test_result.err().unwrap()
        ));
    }

    // Create the url from the hostname and port
    let url = format!("https://{}:{}/function-apps", server.hostname, server.port);

    let builder = get_builder();
    let client = match builder {
        Ok(client) => client,
        Err(e) => return Err(format!("Error creating HTTPS client: {}", e)),
    };

    // Build some JSON containing the function app name
    let json = FunctionAppNameRequest{ 
        name: name.to_string() 
    };

    // Make the request
    let res = client.post(url).json(&json).send().await;

    // Check the response
    match res {
        Ok(res) => {
            // If the server is correct, we should get a 200 status code
            if res.status() != 200 {
                if res.status() == 409 {
                    return Err(
                        format!("A function app already exists that is named '{}'", name)
                            .to_string(),
                    );
                }

                return Err(format!("Server returned status code: {}", res.status()));
            }

            // We are expecting an ID back if this works
            match res.text().await {
                Ok(id) => match Uuid::parse_str(&id) {
                    Ok(id) => Ok(id),
                    Err(e) => Err(format!("Error parsing ID: {}", e)),
                },
                Err(e) => Err(format!("Error reading response text: {}", e)),
            }
        }
        Err(err) => Err(format!("Error: {}", err)),
    }
}

/// Uploads the code to the server
pub async fn post_app_code(conn: &Connection, id: &Uuid, zip_file_buffer: &String) {
    // Get the server
    let server = match storage::get_server(&conn) {
        Ok(server) => server,
        Err(_) => {
            let error_message = format!("No server set. Use the 'set-server' command to set the server.").red().bold();
            println!("{}", error_message);
            std::process::exit(-1);
        }
    };

    // Create the url from the hostname and port
    let url = format!("https://{}:{}/function-apps/{}/code", server.hostname, server.port, id.to_string());

    let builder = get_builder();
    let client = match builder {
        Ok(client) => client,
        Err(e) => {
            let error_message = format!("Error creating HTTPS client: {}", e).red().bold();
            println!("{}", error_message);
            std::process::exit(-1);
        }
    };

    // Make the request
    let res = client.post(url).body(zip_file_buffer.to_string()).send().await;

    // Check the response
    match res {
        Ok(res) => {
            // If the server is correct, we should get a 200 status code
            if res.status() != 200 {
                let error_message = format!("Server returned status code: {}", res.status()).red().bold();
                println!("{}", error_message);
                let error_message = format!("Server returned error: {}", res.text().await.unwrap()).red().bold();
                println!("{}", error_message);
                std::process::exit(-1);
            }
        }
        Err(e) => {
            let error_message = format!("Error: {}", e).red().bold();
            println!("{}", error_message);
            std::process::exit(-1);
        }
    };
}

/// Get the ID for the function app with the given name
pub async fn get_id_for_function_app(conn: &Connection, name: &String) -> Uuid {
    // Get the server
    let server = match storage::get_server(&conn) {
        Ok(server) => server,
        Err(_) => {
            println!("{}", format!("No server set. Use the 'set-server' command to set the server.").red().bold());
            std::process::exit(-1);
        }
    };

    // Create the url from the hostname and port
    let url = format!("https://{}:{}/function-apps/{}/id", server.hostname, server.port, name);

    let builder = get_builder();
    let client = match builder {
        Ok(client) => client,
        Err(e) => {
            println!("{}", format!("Error creating HTTPS client: {}", e).red().bold());
            std::process::exit(-1);
        }
    };

    // Make the request
    let res = client.get(url).send().await;

    match res {
        Ok(res) => {
            if res.status() == 404 {
                println!("{}", format!("No function app with the name '{}' exists", name).red().bold());
                std::process::exit(-1);
            }

            // If the server is correct, we should get a 200 status code
            if res.status() != 200 {
                println!("{}", format!("Server returned status code: {}", res.status()).red().bold());
                std::process::exit(-1);
            }

            // We are expecting an ID back if this works
            let id = match res.text().await {
                Ok(id) => id,
                Err(e) => {
                    println!("{}", format!("Error reading response text: {}", e).red().bold());
                    std::process::exit(-1);
                }
            };

            match Uuid::parse_str(&id) {
                Ok(id) => id,
                Err(e) => {
                    println!("{}", format!("Error parsing ID: {}", e).red().bold());
                    std::process::exit(-1);
                }
            }
        }
        Err(e) => {
            println!("{}", format!("Error: {}", e).red().bold());
            std::process::exit(-1);
        }
    }
}

/// Gets all the function apps from the server
pub async fn list_function_apps(conn: &Connection) -> Vec<FunctionApp> {
    // Get the server
    let server = match storage::get_server(&conn) {
        Ok(server) => server,
        Err(_) => {
            println!("{}", format!("No server set. Use the 'set-server' command to set the server.").red().bold());
            std::process::exit(-1);
        }
    };

    // Create the url from the hostname and port
    let url = format!("https://{}:{}/function-apps", server.hostname, server.port);

    let builder = get_builder();
    let client = match builder {
        Ok(client) => client,
        Err(e) => {
            println!("{}", format!("Error creating HTTPS client: {}", e).red().bold());
            std::process::exit(-1);
        }
    };

    // Make the request
    let res = client.get(url).send().await;

    // Check the response
    match res {
        Ok(res) => {
            // If the server is correct, we should get a 200 status code
            if res.status() != 200 {
                let error_message = format!("Server returned status code: {}", res.status()).red().bold();
                println!("{}", error_message);
                let error_message = format!("Server returned error: {}", res.text().await.unwrap()).red().bold();
                println!("{}", error_message);
                std::process::exit(-1);
            }

            let response_json = res.json::<Vec<FunctionApp>>().await;
            match response_json {
                Ok(response_json) => response_json,
                Err(e) => {
                    let error_message = format!("Error parsing JSON: {}", e).red().bold();
                    println!("{}", error_message);
                    std::process::exit(-1);
                }
            }
        }
        Err(e) => {
            let error_message = format!("Error: {}", e).red().bold();
            println!("{}", error_message);
            std::process::exit(-1);
        }
    }
}

/// Starts a function app running
pub async fn start_function_app(conn: &Connection, id: &Uuid) {
    // Get the server
    let server = match storage::get_server(&conn) {
        Ok(server) => server,
        Err(_) => {
            println!("{}", format!("No server set. Use the 'set-server' command to set the server.").red().bold());
            std::process::exit(-1);
        }
    };

    // Create the url from the hostname and port
    let url = format!("https://{}:{}/function-apps/{}/start", server.hostname, server.port, id.to_string());

    let builder = get_builder();
    let client = match builder {
        Ok(client) => client,
        Err(e) => {
            println!("{}", format!("Error creating HTTPS client: {}", e).red().bold());
            std::process::exit(-1);
        }
    };

    // Make the request
    let res = client.post(url).send().await;

    // Check the response
    match res {
        Ok(res) => {
            // If the server is correct, we should get a 200 status code
            if res.status() != 200 {
                let error_message = format!("Server returned status code: {}", res.status()).red().bold();
                println!("{}", error_message);
                let error_message = format!("Server returned error: {}", res.text().await.unwrap()).red().bold();
                println!("{}", error_message);
                std::process::exit(-1);
            }
        }
        Err(e) => {
            let error_message = format!("Error: {}", e).red().bold();
            println!("{}", error_message);
            std::process::exit(-1);
        }
    };
}

/// Get the status for the function app with the given Id
pub async fn get_status_for_function_app(conn: &Connection, id: &Uuid) -> FunctionAppStatus {
    // Get the server
    let server = match storage::get_server(&conn) {
        Ok(server) => server,
        Err(_) => {
            println!("{}", format!("No server set. Use the 'set-server' command to set the server.").red().bold());
            std::process::exit(-1);
        }
    };

    // Create the url from the hostname and port
    let url = format!("https://{}:{}/function-apps/{}/status", server.hostname, server.port, id);

    let builder = get_builder();
    let client = match builder {
        Ok(client) => client,
        Err(e) => {
            println!("{}", format!("Error creating HTTPS client: {}", e).red().bold());
            std::process::exit(-1);
        }
    };

    // Make the request
    let res = client.get(url).send().await;

    match res {
        Ok(res) => {
            // If the server is correct, we should get a 200 status code
            if res.status() != 200 {
                println!("{}", format!("Server returned status code: {}", res.status()).red().bold());
                std::process::exit(-1);
            }

            // Get the response JSON
            let json = res.json::<FunctionAppStatusResult>().await;

            match json {
                Ok(json) => json.status,
                Err(e) => {
                    println!("{}", format!("Error parsing JSON: {}", e).red().bold());
                    std::process::exit(-1);
                }
            }
        }
        Err(e) => {
            println!("{}", format!("Error: {}", e).red().bold());
            std::process::exit(-1);
        }
    }
}