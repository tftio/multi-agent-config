//! Error types for multi-agent-config

use std::path::PathBuf;
use thiserror::Error;

/// Exit codes matching specification
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_VALIDATION_ERROR: i32 = 1;
pub const EXIT_FILE_ERROR: i32 = 2;
pub const EXIT_PARTIAL_FAILURE: i32 = 3;
pub const EXIT_LOCK_ERROR: i32 = 4;

/// Main error type for multi-agent-config
#[derive(Debug, Error)]
pub enum MultiAgentError {
    /// Configuration errors
    #[error("{0}")]
    Config(#[from] ConfigError),

    /// Environment variable expansion error
    #[error("Environment variable error: {0}")]
    EnvError(String),

    /// Transformation error
    #[error("Transformation error: {0}")]
    TransformError(String),

    /// File operation error
    #[error("File operation error: {0}")]
    FileOpError(String),

    /// CLI argument error
    #[error("CLI error: {0}")]
    CliError(String),
}

impl MultiAgentError {
    /// Get the exit code for this error
    pub fn exit_code(&self) -> i32 {
        match self {
            MultiAgentError::Config(config_err) => match config_err {
                ConfigError::FileNotFound(_) | ConfigError::PermissionDenied(_) => EXIT_FILE_ERROR,
                ConfigError::ParseError { .. }
                | ConfigError::ValidationError(_)
                | ConfigError::TomlError(_) => EXIT_VALIDATION_ERROR,
                ConfigError::IoError(_) => EXIT_FILE_ERROR,
            },
            MultiAgentError::EnvError(_) => EXIT_VALIDATION_ERROR,
            MultiAgentError::TransformError(_) => EXIT_VALIDATION_ERROR,
            MultiAgentError::FileOpError(_) => EXIT_FILE_ERROR,
            MultiAgentError::CliError(_) => EXIT_VALIDATION_ERROR,
        }
    }

    /// Format error with suggestion
    pub fn format_with_suggestion(&self) -> String {
        match self {
            MultiAgentError::Config(ConfigError::FileNotFound(path)) => {
                format!(
                    "Error: Configuration file not found: {}\n\n\
                     Suggestion: Run 'multi-agent-config init' to create a template configuration.",
                    path.display()
                )
            }
            MultiAgentError::Config(ConfigError::PermissionDenied(path)) => {
                format!(
                    "Error: Permission denied: {}\n\n\
                     Suggestion: Check file permissions and ensure you have read access.",
                    path.display()
                )
            }
            MultiAgentError::Config(ConfigError::ParseError { message, line }) => {
                format!(
                    "Error: Parse error at line {}: {}\n\n\
                     Suggestion: Check TOML syntax at the indicated line.",
                    line, message
                )
            }
            MultiAgentError::Config(ConfigError::ValidationError(msg)) => {
                format!(
                    "Error: Validation error: {}\n\n\
                     Suggestion: Run 'multi-agent-config validate' to see all validation errors.",
                    msg
                )
            }
            _ => format!("Error: {}", self),
        }
    }
}

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

    #[test]
    fn test_exit_codes() {
        assert_eq!(EXIT_SUCCESS, 0);
        assert_eq!(EXIT_VALIDATION_ERROR, 1);
        assert_eq!(EXIT_FILE_ERROR, 2);
        assert_eq!(EXIT_PARTIAL_FAILURE, 3);
        assert_eq!(EXIT_LOCK_ERROR, 4);
    }

    #[test]
    fn test_multi_agent_error_exit_code_file_not_found() {
        let err = MultiAgentError::Config(ConfigError::FileNotFound(PathBuf::from("/test")));
        assert_eq!(err.exit_code(), EXIT_FILE_ERROR);
    }

    #[test]
    fn test_multi_agent_error_exit_code_parse_error() {
        let err = MultiAgentError::Config(ConfigError::parse_error("test", 1));
        assert_eq!(err.exit_code(), EXIT_VALIDATION_ERROR);
    }

    #[test]
    fn test_multi_agent_error_exit_code_env_error() {
        let err = MultiAgentError::EnvError("test".to_string());
        assert_eq!(err.exit_code(), EXIT_VALIDATION_ERROR);
    }

    #[test]
    fn test_multi_agent_error_format_with_suggestion_file_not_found() {
        let err = MultiAgentError::Config(ConfigError::FileNotFound(PathBuf::from("/test")));
        let formatted = err.format_with_suggestion();
        assert!(formatted.contains("Configuration file not found"));
        assert!(formatted.contains("Suggestion"));
        assert!(formatted.contains("init"));
    }

    #[test]
    fn test_multi_agent_error_format_with_suggestion_permission_denied() {
        let err = MultiAgentError::Config(ConfigError::PermissionDenied(PathBuf::from("/test")));
        let formatted = err.format_with_suggestion();
        assert!(formatted.contains("Permission denied"));
        assert!(formatted.contains("Suggestion"));
        assert!(formatted.contains("permissions"));
    }

    #[test]
    fn test_multi_agent_error_format_with_suggestion_parse_error() {
        let err = MultiAgentError::Config(ConfigError::parse_error("syntax", 42));
        let formatted = err.format_with_suggestion();
        assert!(formatted.contains("Parse error"));
        assert!(formatted.contains("line 42"));
        assert!(formatted.contains("Suggestion"));
    }

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
        assert_eq!(
            format!("{}", err),
            "Validation error: missing required field"
        );
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

    #[test]
    fn test_multi_agent_error_display() {
        let err = MultiAgentError::EnvError("test env error".to_string());
        assert_eq!(
            format!("{}", err),
            "Environment variable error: test env error"
        );
    }
}
