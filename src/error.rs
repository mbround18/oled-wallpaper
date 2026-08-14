/// Error handling types for the application
/// 
/// Provides custom error enum and Result type alias for consistent error management.

use thiserror::Error;

/// Application error type
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Physics simulation error: {0}")]
    Physics(String),

    #[error("Rendering error: {0}")]
    Render(String),

    #[error("Wallpaper integration error: {0}")]
    Wallpaper(String),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::error::Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias for application errors
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Config("test error".to_string());
        assert_eq!(err.to_string(), "Configuration error: test error");
    }

    #[test]
    fn test_result_type() {
        let ok_result: Result<i32> = Ok(42);
        assert!(ok_result.is_ok());
        
        let err_result: Result<i32> = Err(Error::Config("test".to_string()));
        assert!(err_result.is_err());
    }
}
