use std::io;
use thiserror::Error;

/// Application-wide error type, consolidating all possible errors into a single enum.
#[derive(Debug, Error)]
pub enum AppError {
    /// Represents errors originating from the database, typically from `sqlx`.
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Represents standard input/output errors.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Represents errors specific to the actor system, such as communication failures.
    #[error("Actor error: {0}")]
    Actor(#[from] crate::actors::messages::ActorError),

    /// Represents data validation errors (e.g., invalid input format).
    #[error("Validation error: {0}")]
    Validation(String),

    /// Represents configuration-related errors (e.g., missing environment variables).
    #[error("Configuration error: {0}")]
    Config(String),

    /// Represents unexpected internal errors that indicate a bug.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Represents errors from operations that did not complete in time.
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Represents an error indicating that a rate limit has been exceeded.
    #[error("Rate limit exceeded")]
    RateLimited,
}

impl Clone for AppError {
    fn clone(&self) -> Self {
        match self {
            AppError::Database(e) => AppError::Database(sqlx::Error::Protocol(e.to_string())),
            AppError::Io(e) => AppError::Io(io::Error::new(e.kind(), e.to_string())),
            AppError::Actor(e) => AppError::Actor(e.clone()),
            AppError::Validation(s) => AppError::Validation(s.clone()),
            AppError::Config(s) => AppError::Config(s.clone()),
            AppError::Internal(s) => AppError::Internal(s.clone()),
            AppError::Timeout(s) => AppError::Timeout(s.clone()),
            AppError::RateLimited => AppError::RateLimited,
        }
    }
}

impl From<tokio::time::error::Elapsed> for AppError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        AppError::Timeout(format!("Operation timed out: {}", err))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Validation(format!("JSON error: {}", err))
    }
}

impl From<url::ParseError> for AppError {
    fn from(err: url::ParseError) -> Self {
        AppError::Validation(format!("URL parse error: {}", err))
    }
}

impl From<uuid::Error> for AppError {
    fn from(err: uuid::Error) -> Self {
        AppError::Validation(format!("UUID error: {}", err))
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        AppError::Validation(format!("Validation errors: {}", err))
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        AppError::Validation(format!("Date parse error: {}", err))
    }
}

impl From<which::Error> for AppError {
    fn from(err: which::Error) -> Self {
        AppError::Config(format!("Command not found: {}", err))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Io(io::Error::other(format!("HTTP error: {}", err)))
    }
}

impl From<fastembed::Error> for AppError {
    fn from(err: fastembed::Error) -> Self {
        AppError::Internal(format!("Embedding error: {}", err))
    }
}

impl From<lancedb::Error> for AppError {
    fn from(err: lancedb::Error) -> Self {
        AppError::Database(sqlx::Error::Protocol(format!("LanceDB error: {}", err)))
    }
}

impl From<arrow::error::ArrowError> for AppError {
    fn from(err: arrow::error::ArrowError) -> Self {
        AppError::Internal(format!("Arrow error: {}", err))
    }
}
