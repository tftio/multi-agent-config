//! TOML configuration file parsing

use crate::config::types::MultiAgentConfig;
use crate::error::ConfigError;
use std::fs;
use std::path::Path;

/// Parse configuration from a TOML file
///
/// # Arguments
///
/// * `path` - Path to the TOML configuration file
///
/// # Returns
///
/// * `Ok(MultiAgentConfig)` - Successfully parsed configuration
/// * `Err(ConfigError)` - Error reading or parsing the file
///
/// # Errors
///
/// * `ConfigError::FileNotFound` - File does not exist
/// * `ConfigError::PermissionDenied` - Cannot read file due to permissions
/// * `ConfigError::ParseError` - TOML syntax error
/// * `ConfigError::IoError` - Other I/O error
pub fn parse_config_file(path: &Path) -> Result<MultiAgentConfig, ConfigError> {
    // Read the file contents
    let contents = read_file_utf8(path)?;

    // Parse TOML
    let config: MultiAgentConfig = toml::from_str(&contents).map_err(|e| {
        // Extract line number from toml error if available
        let line = e
            .span()
            .map(|span| {
                // Count newlines up to the error position
                contents[..span.start].lines().count()
            })
            .unwrap_or(0);

        ConfigError::parse_error(e.message(), line)
    })?;

    Ok(config)
}

/// Read a file as UTF-8 string with appropriate error handling
///
/// # Arguments
///
/// * `path` - Path to read
///
/// # Returns
///
/// * `Ok(String)` - File contents as UTF-8 string
/// * `Err(ConfigError)` - Error reading the file
///
/// # Errors
///
/// * `ConfigError::FileNotFound` - File does not exist
/// * `ConfigError::PermissionDenied` - Permission denied
/// * `ConfigError::IoError` - Other I/O errors
pub fn read_file_utf8(path: &Path) -> Result<String, ConfigError> {
    fs::read_to_string(path).map_err(|e| {
        use std::io::ErrorKind;
        match e.kind() {
            ErrorKind::NotFound => ConfigError::FileNotFound(path.to_path_buf()),
            ErrorKind::PermissionDenied => ConfigError::PermissionDenied(path.to_path_buf()),
            _ => ConfigError::IoError(e),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_minimal_config() {
        let toml_content = r#"
[settings]
version = "1.0"

[mcp.servers.example]
command = "npx"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let result = parse_config_file(temp_file.path());
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.settings.is_some());
        assert_eq!(config.settings.as_ref().unwrap().version, "1.0");
        assert_eq!(config.mcp.servers.len(), 1);
        assert!(config.mcp.servers.contains_key("example"));
    }

    #[test]
    fn test_parse_valid_complete_config() {
        let toml_content = r#"
[settings]
version = "1.0"
default_targets = ["cursor", "codex"]

[env]
MY_VAR = "value"
TOKEN = "${GITHUB_TOKEN}"

[mcp.servers.stdio-server]
command = "npx"
args = ["-y", "package"]
enabled = true
targets = ["cursor"]

[mcp.servers.stdio-server.env]
API_KEY = "{MY_VAR}"

[mcp.servers.http-server]
url = "https://example.com/mcp"
bearer_token = "{TOKEN}"
enabled = true
targets = ["all"]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let result = parse_config_file(temp_file.path());
        assert!(result.is_ok());

        let config = result.unwrap();

        // Check settings
        let settings = config.settings.as_ref().unwrap();
        assert_eq!(settings.version, "1.0");
        assert_eq!(settings.default_targets.len(), 2);

        // Check env section
        let env = config.env.as_ref().unwrap();
        assert_eq!(env.get("MY_VAR").unwrap(), "value");
        assert_eq!(env.get("TOKEN").unwrap(), "${GITHUB_TOKEN}");

        // Check servers
        assert_eq!(config.mcp.servers.len(), 2);
        assert!(config.mcp.servers.contains_key("stdio-server"));
        assert!(config.mcp.servers.contains_key("http-server"));
    }

    #[test]
    fn test_parse_file_not_found() {
        let path = Path::new("/nonexistent/config.toml");
        let result = parse_config_file(path);

        assert!(result.is_err());
        match result {
            Err(ConfigError::FileNotFound(p)) => {
                assert_eq!(p, path);
            }
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_parse_invalid_toml() {
        let invalid_toml = r#"
[settings
version = "1.0"  # Missing closing bracket
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(invalid_toml.as_bytes()).unwrap();

        let result = parse_config_file(temp_file.path());

        assert!(result.is_err());
        match result {
            Err(ConfigError::ParseError { message, line }) => {
                assert!(!message.is_empty());
                assert!(line > 0);
            }
            Err(e) => panic!("Expected ParseError, got: {}", e),
            Ok(_) => panic!("Expected error, got success"),
        }
    }

    #[test]
    fn test_parse_missing_required_field() {
        let toml_content = r#"
[settings]
version = "1.0"

# Missing mcp section entirely
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let result = parse_config_file(temp_file.path());

        // Should fail during deserialization because mcp is required
        assert!(result.is_err());
    }

    #[test]
    fn test_read_file_utf8_success() {
        let content = "test content";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();

        let result = read_file_utf8(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);
    }

    #[test]
    fn test_read_file_utf8_not_found() {
        let path = Path::new("/nonexistent/file.txt");
        let result = read_file_utf8(path);

        assert!(result.is_err());
        matches!(result, Err(ConfigError::FileNotFound(_)));
    }
}
