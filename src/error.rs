//! Error types for multi-agent-config

use std::path::PathBuf;
use thiserror::Error;

/// Configuration error types
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found: {0}")]
    FileNotFound(PathBuf),

    /// Permission denied accessing configuration file
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    /// TOML parsing error
    #[error("Parse error at line {line}: {message}")]
    ParseError {
        /// Error message
        message: String,
        /// Line number where error occurred
        line: usize,
    },

    /// Schema validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// General I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// TOML deserialization error
    #[error("TOML error: {0}")]
    TomlError(#[from] toml::de::Error),
}

impl ConfigError {
    /// Create a parse error with line number
    pub fn parse_error(message: impl Into<String>, line: usize) -> Self {
        ConfigError::ParseError {
            message: message.into(),
            line,
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        ConfigError::ValidationError(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_file_not_found_display() {
        let err = ConfigError::FileNotFound(PathBuf::from("/path/to/config.toml"));
        assert_eq!(
            format!("{}", err),
            "Configuration file not found: /path/to/config.toml"
        );
    }

    #[test]
    fn test_permission_denied_display() {
        let err = ConfigError::PermissionDenied(PathBuf::from("/etc/config.toml"));
        assert_eq!(format!("{}", err), "Permission denied: /etc/config.toml");
    }

    #[test]
    fn test_parse_error_display() {
        let err = ConfigError::parse_error("invalid syntax", 42);
        assert_eq!(format!("{}", err), "Parse error at line 42: invalid syntax");
    }

    #[test]
    fn test_validation_error_display() {
        let err = ConfigError::validation("missing required field");
        assert_eq!(format!("{}", err), "Validation error: missing required field");
    }

    #[test]
    fn test_parse_error_constructor() {
        let err = ConfigError::parse_error("test message", 10);
        match err {
            ConfigError::ParseError { message, line } => {
                assert_eq!(message, "test message");
                assert_eq!(line, 10);
            }
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_validation_constructor() {
        let err = ConfigError::validation("test validation");
        match err {
            ConfigError::ValidationError(msg) => {
                assert_eq!(msg, "test validation");
            }
            _ => panic!("Expected ValidationError variant"),
        }
    }
}
