use std::fmt;

use serde::Deserialize;

/// Represents all possible errors that can occur when interacting with the Upbit API
#[derive(Debug)]
pub enum Errors {
    /// Invalid API key or signature
    InvalidApiKey(),

    /// Network / HTTP level error (abstracted, no direct reqwest dependency)
    HttpError(String),

    /// An error returned by the Upbit API
    ApiError(ApiError),

    /// A general error with a descriptive message
    Error(String),
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Errors::InvalidApiKey() => write!(f, "Invalid API key or signature"),
            Errors::HttpError(err) => write!(f, "HTTP error: {err}"),
            Errors::ApiError(err) => write!(f, "API error: {err}"),
            Errors::Error(msg) => write!(f, "Error: {msg}"),
        }
    }
}

impl std::error::Error for Errors {}

impl From<rest::HttpError> for Errors {
    fn from(err: rest::HttpError) -> Self {
        let msg = match err {
            rest::HttpError::Network(e) => format!("network: {e}"),
            rest::HttpError::Timeout => "timeout".to_string(),
            rest::HttpError::InvalidUrl(u) => format!("invalid url: {u}"),
            rest::HttpError::Decode(e) => format!("decode: {e}"),
            rest::HttpError::Http { status, body } => format!("status {status}: {body}"),
            rest::HttpError::Unknown(e) => format!("unknown: {e}"),
            #[allow(unreachable_patterns)]
            _ => "unclassified http error".to_string(),
        };
        Errors::HttpError(msg)
    }
}

impl From<serde_json::Error> for Errors {
    fn from(err: serde_json::Error) -> Self {
        Errors::Error(format!("JSON error: {err}"))
    }
}

/// Upbit API error response
#[derive(Debug, Deserialize, Clone)]
pub struct ApiError {
    pub name: String,
    pub message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.message)
    }
}

impl std::error::Error for ApiError {}

/// Error response wrapper
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: ApiError,
}
