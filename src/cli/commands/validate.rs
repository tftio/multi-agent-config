//! Validate command implementation

use multi_agent_config::{
    config::{parse_and_expand_config, validate_config},
    error::MultiAgentError,
};
use std::path::Path;

/// Validate configuration file
///
/// # Arguments
///
/// * `config_path` - Path to configuration file
/// * `verbose` - Enable verbose output
///
/// # Returns
///
/// * `Ok(())` - Configuration is valid
/// * `Err(MultiAgentError)` - Configuration is invalid or cannot be read
///
/// # Errors
///
/// Returns error if config cannot be read, parsed, or is invalid
pub fn validate_command(config_path: &Path, verbose: bool) -> Result<(), MultiAgentError> {
    if verbose {
        println!("Validating configuration: {}", config_path.display());
    }

    // Parse and expand configuration
    let config = parse_and_expand_config(config_path)?;

    // Validate schema
    if let Err(errors) = validate_config(&config) {
        eprintln!("Validation failed with {} error(s):", errors.len());
        for (i, error) in errors.iter().enumerate() {
            eprintln!("  {}. {}", i + 1, error);
        }
        return Err(MultiAgentError::Config(
            multi_agent_config::error::ConfigError::ValidationError(format!(
                "{} validation error(s) found",
                errors.len()
            )),
        ));
    }

    // Count servers by tool
    let total_servers = config.mcp.servers.len();

    if verbose {
        println!("Configuration valid!");
        println!("  Total servers: {total_servers}");
        println!(
            "  Settings version: {}",
            config
                .settings
                .as_ref()
                .map_or("none", |s| s.version.as_str())
        );
    } else {
        println!("Configuration is valid ({total_servers} server(s))");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_command_valid_config() {
        let toml_content = r#"
[settings]
version = "1.0"

[mcp.servers.test]
command = "npx"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let result = validate_command(temp_file.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_command_invalid_version() {
        let toml_content = r#"
[settings]
version = "invalid"

[mcp.servers.test]
command = "npx"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let result = validate_command(temp_file.path(), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_command_file_not_found() {
        let result = validate_command(Path::new("/nonexistent/config.toml"), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_command_verbose() {
        let toml_content = r#"
[settings]
version = "1.0"

[mcp.servers.test]
command = "npx"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let result = validate_command(temp_file.path(), true);
        assert!(result.is_ok());
    }
}
