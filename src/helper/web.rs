use axum::{http::StatusCode, response::Json};
use serde::Serialize;
use std::fmt::{Display, Formatter, Result as fmtResult};

/// Represents a response returned by the server.
///
/// The `Response` type is a tuple that consists of the following components:
///
/// - `StatusCode`: An enumeration representing the HTTP status code of the response.
/// - `Result<Json<T>, String>`: A result type that contains either a JSON payload of
///                              type `T` or an error message as a string.
pub type Response<T> = (StatusCode, Result<Json<T>, String>);

/// Represents server errors that can occur during the execution of the application.
///
/// The `ServerErr` enum provides different variants to represent various server errors.
/// Currently, the following variant is available:
///
/// - `Internal`: Represents an internal server error.
#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerErr {
    Internal,
}

impl Display for ServerErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmtResult {
        match self {
            ServerErr::Internal => write!(f, "Internal server error"),
        }
    }
}

/// Represents an internal server error response.
pub async fn internal_server_error<T>() -> Response<T> {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Err(ServerErr::Internal.to_string()),
    )
}
