use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The status of the function app
#[derive(Debug)]
#[derive(Serialize)]
#[derive(Deserialize)]
pub enum FunctionAppStatus {
    /// Not registered - this is returned if the Uuid is not recognized
    NotRegistered,

    /// Registered - the function app has been registered, but the code has not been uploaded
    Registered,

    /// Building - the function app is currently being built from the uploaded code
    Building,

    /// Ready - the function app is built and ready to be used, but is not running
    Ready,

    /// Running - the function app is running
    Running,

    /// Error - the function app has encountered an error either building or running
    Error,
}

/// The function app details to store in the database
#[derive(Debug)]
#[derive(Serialize)]
#[derive(Deserialize)]
pub struct FunctionApp {
    // The app name
    pub name: String,

    // The app ID
    pub id: Uuid,

    // The status of the app
    pub status: FunctionAppStatus,

    // The date/time the app was created
    pub created_at: u64,
}

/// The contents of the request sent to create a new function app
#[derive(Deserialize)]
#[derive(Serialize)]
pub struct FunctionAppNameRequest {
    pub name: String,
}

// The status of the function app
#[derive(Deserialize)]
#[derive(Serialize)]
pub struct FunctionAppStatusResult {
    pub id: Uuid,
    pub status: FunctionAppStatus,
}