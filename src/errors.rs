// errors.rs — All possible errors in our app
//
// Instead of letting random errors crash the server,
// we define exactly what can go wrong and what HTTP status to return.

use axum::{
    http::StatusCode,                   // HTTP status codes like 200, 404, 401
    response::{IntoResponse, Response}, // lets us return errors as HTTP responses
    Json,                               // lets us return JSON
};
use serde_json::json; // lets us write JSON like: json!({ "error": "..." })

// Our custom error type
// Each variant represents a different kind of error
pub enum AppError {
    NotFound(String),     // 404 - resource doesn't exist
    BadRequest(String),   // 400 - client sent bad data
    Unauthorized(String), // 401 - not logged in or bad token
    Conflict(String),     // 409 - e.g. email already exists
    Internal(String),     // 500 - something unexpected went wrong
}

// This tells Axum how to convert our AppError into an HTTP response
// Axum calls this automatically when a handler returns Err(AppError::...)
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Match the error type to get the right status code and message
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        // Return a JSON response like: { "error": "Email already registered" }
        (status, Json(json!({ "error": message }))).into_response()
    }
}

// This lets us use ? operator with sqlx database errors
// Instead of: db_query().map_err(|e| AppError::Internal(e.to_string()))?
// We can just write: db_query()?
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}
