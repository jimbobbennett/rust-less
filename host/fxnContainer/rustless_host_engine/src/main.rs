use actix_web::{get, post, App, HttpServer, Responder, HttpResponse, web, web::Json};
use colored::Colorize;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use rusqlite::Error;
use tempfile::tempdir;
use uuid::Uuid;

use rustless_shared::{FunctionAppStatus, FunctionAppStatusResult, FunctionAppNameRequest};

mod docker;
mod function_app_builder;
mod storage;

// Interface
// ✅ GET hello - test that the server is running
// ❌ GET/POST api/{appname}/{approute} - route request to function app
// ❌ GET api/{appname}/ - list all routes for the app
// ✅ GET function-apps - list all apps
// ✅ GET function-apps/{appname}/id - Get the ID for the app
// ✅ POST function-apps - adds a new function app to the server. This is a multi-stage process. This stage returns a unique ID for the function app
// ✅ POST function-apps/{id}/code - uploads the code for the function app for the given ID (registered with a post to api/function-apps), and this kicks off the build and registration of the docker container. If the app is running, it will be stopped
// ❌ GET function-apps/{id}/status - gets the status of the function app, Not found, registered, building, ready, running, error
// ❌ POST function-apps/{id}/start - starts the function app if it is ready or error
// ❌ POST function-apps/{id}/stop - stops the function app if it is started
// ❌ DELETE function-apps/{id} - deletes the function app, stopping it if it is running
//
// ❌ Check status before adding code
// ❌ Check status before updating code, and stop the app if it is running
// ❌ Poll every few seconds for status updates

/// This route is used as a test to ensure the server is running. It will return "Hello!"
#[get("/hello")]
async fn greet() -> impl Responder {
    format!("Hello from rustless!")
}

#[get("/function-apps/{id}/status")]
async fn get_function_app_status(info: web::Path<String>) -> HttpResponse {
    let conn = storage::create_connection_fast();

    let id = Uuid::parse_str(&info);
    let id = match id {
        Ok(id) => id,
        Err(e) => {
            println!("Error parsing ID: {}", e);
            return HttpResponse::BadRequest().body(e.to_string())
        }
    };

    let status = function_app_builder::get_function_app_status(&conn, &id);
    let status = match status {
        Ok(status) => status,
        Err(e) => {
            println!("Error getting function app status: {}", e);
            return HttpResponse::InternalServerError().body(e.to_string())
        }
    };

    let _ = storage::set_function_app_status(&conn, &id, &status);

    // Return the status
    let result = FunctionAppStatusResult {
        id,
        status,
    };

    HttpResponse::Ok().json(result)
}

#[post("/function-apps/{id}/start")]
async fn start_function_app(info: web::Path<String>) -> HttpResponse {
    let conn = storage::create_connection_fast();

    let id = Uuid::parse_str(&info);
    let id = match id {
        Ok(id) => id,
        Err(e) => {
            println!("Error parsing ID: {}", e);
            return HttpResponse::BadRequest().body(e.to_string())
        }
    };

    let status = function_app_builder::get_function_app_status(&conn, &id);
    let status = match status {
        Ok(status) => status,
        Err(e) => {
            println!("Error getting function app status: {}", e);
            return HttpResponse::InternalServerError().body(e.to_string())
        }
    };

    let _ = storage::set_function_app_status(&conn, &id, &status);

    match status {
        FunctionAppStatus::Ready => {
            // Get the function app name to prove we have an app registered with this ID
            let function_app_name = storage::get_function_app_name(&conn, &id);
            let function_app_name = match function_app_name {
                Ok(n) => n,
                Err(e) => {
                    return HttpResponse::BadRequest().body(format!("Cannot get function app name from ID: {}", e));
                }
            };

            // Start the function app
            let start_result = docker::start_function_app(&function_app_name);
            let port = match start_result {
                Ok(port) => port,
                Err(e) => {
                    return HttpResponse::InternalServerError().body(format!("Error starting function app: {}", e));
                }
            };

            // Update the status and port in the database
            match storage::set_function_app_running(&conn, &id, port){
                Ok(_) => HttpResponse::Ok().body("Function app is already running"),
                Err(e) => HttpResponse::InternalServerError().body(format!("Error updating function app status: {}", e))
            }            
        },
        FunctionAppStatus::Running => HttpResponse::Ok().body("Function app is already running"),
        FunctionAppStatus::Building => HttpResponse::InternalServerError().body("Cannot start function app, it is currently building"),
        FunctionAppStatus::Error => HttpResponse::InternalServerError().body("Cannot start function app, it is in an error state"),
        FunctionAppStatus::Registered => HttpResponse::InternalServerError().body("Cannot start function app, it doesn't have any code yet"),
        FunctionAppStatus::NotRegistered => HttpResponse::InternalServerError().body("Cannot start function app, it doesn't exist"),
    }
}

#[get("/function-apps")]
async fn list_function_apps() -> impl Responder {
    let result = storage::get_all_apps();

    match result {
        Ok(apps) => {
            HttpResponse::Ok().json(apps)
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string())
    }
}

#[get("/function-apps/{name}/id")]
async fn get_function_app_id(name: web::Path<String>) -> impl Responder {
    let conn = storage::create_connection_fast();
    let name = name.to_string();

    let result = storage::get_function_id_from_name(&conn, &name);

    match result {
        Ok(id) => HttpResponse::Ok().body(id.to_string()),
        Err(Error::QueryReturnedNoRows) => HttpResponse::NotFound().body(format!("No function app with name {} found", name)),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string())
    }
}

/// Create a new function app in the server
/// 
/// This registers a new function app by name in the database and returns the new ID
/// The name MUST be unique
#[post("/function-apps")]
async fn create_function_app(body: Json<FunctionAppNameRequest>) -> HttpResponse {
    let conn = storage::create_connection_fast();

    // Check if the name is already in use
    let in_use = storage::is_name_in_use(&conn, &body.name);
    match in_use {
        Ok(in_use) => {
            if in_use {
                return HttpResponse::BadRequest().body("Name is already in use");
            }
        },
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    }

    // Register the function app in the database
    let res = storage::add_new_function_app(&conn, &body.name);
    match res {
        Ok(id) => HttpResponse::Ok().body(id.to_string()),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Handles code upload for the function app
/// 
/// The body is a base64 encoded string containing a zip file with all the code for the function app
#[post("/function-apps/{id}/code")]
async fn post_function_app_code(info: web::Path<String>, body: String) -> HttpResponse {
    let conn = storage::create_connection_fast();

    let id = Uuid::parse_str(&info);
    let id = match id {
        Ok(id) => id,
        Err(e) => {
            println!("Error parsing ID: {}", e);
            return HttpResponse::BadRequest().body(e.to_string())
        }
    };

    // Get the function app name to prove we have an app registered with this ID
    let function_app_name = storage::get_function_app_name(&conn, &id);
    let function_app_name = match function_app_name {
        Ok(n) => n,
        Err(e) => {
            return HttpResponse::BadRequest().body(format!("Cannot get function app name from ID: {}", e));
        }
    };

    let status_update = storage::set_function_app_status(&conn, &id, &FunctionAppStatus::Building);
    match status_update {
        Ok(_) => (),
        Err(e) => {
            let _ = storage::set_function_app_status(&conn, &id, &FunctionAppStatus::Error);
            println!("Error updating status: {}", e);
            return HttpResponse::InternalServerError().body(e.to_string())
        }
    }

    // Decode the base64 string
    let decoded = base64::decode(&body);
    let decoded = match decoded {
        Ok(d) => d,
        Err(e) => {
            let _ = storage::set_function_app_status(&conn, &id, &FunctionAppStatus::Error);
            println!("Error decoding base64: {}", e);
            return HttpResponse::BadRequest().body(e.to_string())
        }
    };

    let temp_dir = tempdir();
    let temp_dir = match temp_dir {
        Ok(dir) => {
            // print the directory path
            println!("Created temporary directory at {}", dir.path().display());
            dir
        },
        Err(e) => {
            let _ = storage::set_function_app_status(&conn, &id, &FunctionAppStatus::Error);
            println!("Error creating temporary directory: {}", e);
            return HttpResponse::BadRequest().body(format!("Error creating temporary directory: {}", e));
        }
    };

    // Write the decoded string to a temporary zip file
    let zip_file = function_app_builder::unzip_file_in_temp_dir(&temp_dir, &decoded);
    match zip_file {
        Ok(_) => (),
        Err(e) => {
            let _ = storage::set_function_app_status(&conn, &id, &FunctionAppStatus::Error);
            println!("Error writing zip file: {}", e);
            return HttpResponse::InternalServerError().body(format!("Could not write zip file: {}", e));
        }
    }

    println!("{}", temp_dir.path().to_string_lossy().to_string());

    // Build the Docker container for the function app
    let result = docker::build_function_app_container(&temp_dir, &function_app_name);
    match result {
        Ok(_) => {},
        Err(e) => {
            let _ = storage::set_function_app_status(&conn, &id, &FunctionAppStatus::Error);
            return HttpResponse::BadRequest().body(format!("Error: {}", e));
        }
    };

    // Finally set the status to ready
    let status_update = storage::set_function_app_status(&conn, &id, &FunctionAppStatus::Ready);
    match status_update {
        Ok(_) => (),
        Err(e) => {
            let _ = storage::set_function_app_status(&conn, &id, &FunctionAppStatus::Error);
            println!("Error updating status: {}", e);
            return HttpResponse::InternalServerError().body(e.to_string())
        }
    }

    HttpResponse::Ok().body("")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create the connection
    let conn_result = storage::create_connection();
    match conn_result {
        Ok(conn) => conn,
        Err(_) => {
            let error_message = format!("Error connecting to database.").red().bold();
            println!("{}", error_message);
            std::process::exit(-1);
        }
    };
    
    // Set up HTTPS
    let builder = SslAcceptor::mozilla_intermediate(SslMethod::tls());
    let mut builder = match builder {
        Ok(builder) => builder,
        Err(e) => {
            let error_message = format!("Error creating SSL builder: {}", e).red().bold();
            println!("{}", error_message);
            std::process::exit(-1);
        }
    };

    if builder.set_private_key_file("key.pem", SslFiletype::PEM).is_err() {
        let error_message = format!("Error setting private key file").red().bold();
        println!("{}", error_message);
        std::process::exit(-1);
    }

    if builder.set_certificate_chain_file("cert.pem").is_err() {
        let error_message = format!("Error setting certificate chain file").red().bold();
        println!("{}", error_message);
        std::process::exit(-1);
    }

    // Create and start the server
    HttpServer::new(|| {
        App::new().service(greet)
                  .service(create_function_app)
                  .service(post_function_app_code)
                  .service(list_function_apps)
                  .service(get_function_app_id)
                  .service(start_function_app)
                  .service(get_function_app_status)
    })
    .bind_openssl("0.0.0.0:8080", builder)?
    .run()
    .await
}