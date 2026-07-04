//! Application Error Types
//!
//! Centralizes error handling across the codebase with `AppError` enum
//! and `AppResult<T>` type alias, replacing `Result<T, String>`.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::fmt;

/// Application-level error type.
///
/// Centralizes error handling across the codebase, replacing `Result<T, String>`.
#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    /// I/O error (file operations, network, etc.)
    Io(std::io::Error),
    /// JSON serialization/deserialization error
    Serde(serde_json::Error),
    /// Validation error (invalid input, path, etc.)
    Validation(String),
    /// ffmpeg/ffprobe execution error
    Ffmpeg(String),
    /// Generic error with a message
    Message(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "Erro de I/O: {}", e),
            AppError::Serde(e) => write!(f, "Erro de JSON: {}", e),
            AppError::Validation(msg) => write!(f, "Validação: {}", msg),
            AppError::Ffmpeg(msg) => write!(f, "ffmpeg: {}", msg),
            AppError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(e) => Some(e),
            AppError::Serde(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Serde(e)
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Message(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Message(s.to_string())
    }
}

/// Convenience type alias for Results using AppError.
#[allow(dead_code)]
pub type AppResult<T> = Result<T, AppError>;
