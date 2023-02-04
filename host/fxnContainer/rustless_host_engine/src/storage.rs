use std::time::SystemTime;

use rusqlite::{Connection, Result, Error};
use uuid::Uuid;
use rustless_shared::{FunctionApp, FunctionAppStatus};

/// The function app details to store in the database
#[derive(Debug)]
struct SqliteFunctionApp {
    // The app name
    pub name: String,

    // The app ID
    pub id: String,

    // The status of the app
    pub status: u8,

    // The date/time the app was created
    pub created_at: u64,

    // The port the container is running on, if it is running
    pub port: u16
}

const DB_FILE: &str = "rustless_host.db";

/// Create the database connection assuming it already exists. Only call this if create_connection() has already been called once
/// create_connection() will be called at the start of the server, so this should be ok. It will panic if the database does not exist
pub fn create_connection_fast() -> Connection {
    let conn = Connection::open(DB_FILE);
    match conn {
        Ok(conn) => conn,
        Err(e) => panic!("Error opening database: {}", e),
    }
}

/// Gets all the registered function apps
pub fn get_all_apps() -> Result<Vec<FunctionApp>, String> {
    let conn = create_connection_fast();

    // Prepare the SQL statement
    let stmt = conn.prepare("SELECT name, id, status, created_at, port FROM function_apps");
    let mut stmt = match stmt {
        Ok(stmt) => stmt,
        Err(e) => return Err(e.to_string()),
    };

    // Run the query
    let function_apps = stmt.query_map([], |row| {
        Ok(SqliteFunctionApp {
            name: row.get(0)?,
            id: row.get(1)?,
            status: row.get(2)?,
            created_at: row.get(3)?,
            port: row.get(4)?
        })
    });

    let function_apps = match function_apps {
        Ok(function_apps) => function_apps,
        Err(e) => return Err(e.to_string()),
    };

    // Build a vector for the response
    let mut response = Vec::new();

    // Add all the items to the vector, mapping the status to the enum
    for function_app in function_apps {
        let function_app = match function_app {
            Ok(function_app) => function_app,
            Err(e) => return Err(e.to_string()),
        };

        let id = Uuid::parse_str(&function_app.id);
        let id = match id {
            Ok(id) => id,
            Err(e) => return Err(e.to_string()),
        };
        
        response.push(FunctionApp {
            name: function_app.name,
            id: id,
            status: match function_app.status {
                0 => FunctionAppStatus::NotRegistered,
                1 => FunctionAppStatus::Registered,
                2 => FunctionAppStatus::Building,
                3 => FunctionAppStatus::Ready,
                4 => FunctionAppStatus::Running,
                5 => FunctionAppStatus::Error,
                _ => panic!("Unknown status"),
            },
            created_at: function_app.created_at
        });
    }

    Ok(response)
}

/// Checks if the given function app name is already in use
pub fn is_name_in_use(conn: &Connection, name: &str) -> Result<bool, Error> {
    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM function_apps WHERE name = ?")?;
    
    let mut rows = stmt.query(&[name])?;
    match rows.next()? {
        Some(row) => {
            let count: i64 = row.get(0)?;
            Ok(count > 0)
        },
        None => Ok(false),
    }
}

/// Gets the function ID from the app name
pub fn get_function_id_from_name(conn: &Connection, name: &String) -> Result<Uuid, Error> {
    let mut stmt = conn
        .prepare("SELECT id FROM function_apps WHERE name = ?")?;
    let mut rows = stmt.query([name])?;

    match rows.next()? {
        Some(row) => {
            let id: String = row.get(0)?;
            let id = Uuid::parse_str(&id);
            match id {
                Ok(id) => Ok(id),
                Err(e) => Err(Error::ToSqlConversionFailure(e.into())),
            }
        },
        None => Err(Error::QueryReturnedNoRows),
    }
}

/// Gets the function app name from the ID
pub fn get_function_app_name(conn: &Connection, id: &Uuid) -> Result<String, Error> {
    let mut stmt = conn
        .prepare("SELECT name FROM function_apps WHERE id = ?")?;
    let mut rows = stmt.query([id.to_string()])?;

    match rows.next()? {
        Some(row) => {
            let name: String = row.get(0)?;
            Ok(name)
        },
        None => Err(Error::QueryReturnedNoRows),
    }
}

/// Adds a new function app to the database and returns the ID
pub fn add_new_function_app(conn: &Connection, name: &str) -> Result<Uuid> {
    // Generate the ID
    let id = Uuid::new_v4();

    // The function app starts with a status of registered
    let status = FunctionAppStatus::Registered as u8;
    
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as u64;

    // Insert the new row
    match conn.execute(
        format!("INSERT INTO function_apps (name, id, status, created_at, port) VALUES (?1, ?2, {}, {}, 0)", status, time).as_str(),
        &[name, &id.to_string()],
    ) {
        Ok(_) => Ok(id),
        Err(e) => Err(e),
    }
}

/// Sets the status of the given app to building
pub fn set_function_app_status(conn: &Connection, id: &Uuid, status: &FunctionAppStatus) -> Result<()> {
    let status = (*status) as u8;

    match conn.execute(
        format!("UPDATE function_apps SET status = {} WHERE id = ?", status).as_str(),
        &[&id.to_string()],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

/// Sets a function app as running
pub fn set_function_app_running(conn: &Connection, id: &Uuid, port: u16) -> Result<()> {
    match conn.execute(
        "UPDATE function_apps SET status = 4, port = ? WHERE id = ?",
        &[&port.to_string(), &id.to_string()],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

/// Creates a connection to the database
pub fn create_connection() -> Result<Connection, String> {
    // Open the database file
    let conn_result = Connection::open(DB_FILE);

    // Check if the open actually worked
    let conn = match conn_result {
        Ok(conn) => conn,
        Err(_) => {
            return Err("Error connecting to database".to_string());
        }
    };

    // We need a table to store the function app details. Create it if it doesn't exist
    match conn.execute(
        "CREATE TABLE IF NOT EXISTS function_apps (
                  id          TEXT PRIMARY KEY,
                  name        TEXT NOT NULL UNIQUE,
                  status      INTEGER NOT NULL,
                  created_at  INTEGER NOT NULL,
                  port        INTEGER NOT NULL
                  )",
        [],
    ) {
        Ok(_) => {},
        Err(_) => {
            return Err("Error creating table".to_string());
        }
    };

    // Return the connection
    Ok(conn)
}