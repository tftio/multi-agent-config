//! Configuration schema validation

use crate::config::types::{HttpServerConfig, MultiAgentConfig, ServerConfig, StdioServerConfig};
use regex::Regex;
use std::collections::HashSet;

/// Validation error with context
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Error message
    pub message: String,
    /// Context (e.g., server name, field name)
    pub context: Option<String>,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            context: None,
        }
    }

    /// Create a validation error with context
    pub fn with_context(message: impl Into<String>, context: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            context: Some(context.into()),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ctx) = &self.context {
            write!(f, "{}: {}", ctx, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

/// Validate configuration against schema requirements
///
/// Collects all validation errors instead of failing on first error.
///
/// # Arguments
///
/// * `config` - Configuration to validate
///
/// # Returns
///
/// * `Ok(())` - Configuration is valid
/// * `Err(Vec<ValidationError>)` - List of validation errors found
///
/// # Errors
///
/// Returns `Err` with list of validation errors if configuration is invalid
pub fn validate_config(config: &MultiAgentConfig) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Validate settings section
    if let Some(settings) = &config.settings {
        validate_settings(settings, &mut errors);
    }

    // Validate MCP servers section
    validate_mcp_servers(config, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate settings section
fn validate_settings(settings: &crate::config::types::Settings, errors: &mut Vec<ValidationError>) {
    // Validate version format
    let version_regex = Regex::new(r"^\d+\.\d+(\.\d+)?$").unwrap();
    if !version_regex.is_match(&settings.version) {
        errors.push(ValidationError::with_context(
            format!(
                "Invalid version format '{}', expected semver (e.g., '1.0' or '1.0.0')",
                settings.version
            ),
            "settings.version",
        ));
    }

    // Currently only version 1.0 is supported
    if !settings.version.starts_with("1.0") {
        errors.push(ValidationError::with_context(
            format!(
                "Unsupported version '{}', only '1.0' is currently supported",
                settings.version
            ),
            "settings.version",
        ));
    }

    // Validate default_targets
    let valid_tools = ["claude-code", "cursor", "opencode", "codex", "all"];
    let mut seen = HashSet::new();
    for target in &settings.default_targets {
        if !valid_tools.contains(&target.as_str()) {
            errors.push(ValidationError::with_context(
                format!(
                    "Invalid tool name '{}', must be one of: {}",
                    target,
                    valid_tools.join(", ")
                ),
                "settings.default_targets",
            ));
        }
        if !seen.insert(target) {
            // Duplicate found - this is a warning in the spec but we'll include it
            errors.push(ValidationError::with_context(
                format!("Duplicate target '{target}'"),
                "settings.default_targets",
            ));
        }
    }
}

/// Validate MCP servers section
fn validate_mcp_servers(config: &MultiAgentConfig, errors: &mut Vec<ValidationError>) {
    // Must have at least one server
    if config.mcp.servers.is_empty() {
        errors.push(ValidationError::new(
            "At least one MCP server must be defined in [mcp.servers]",
        ));
        return;
    }

    // Validate each server
    for (name, server) in &config.mcp.servers {
        validate_server(name, server, errors);
    }
}

/// Validate individual server configuration
fn validate_server(name: &str, server: &ServerConfig, errors: &mut Vec<ValidationError>) {
    match server {
        ServerConfig::Stdio(stdio) => validate_stdio_server(name, stdio, errors),
        ServerConfig::Http(http) => validate_http_server(name, http, errors),
    }
}

/// Validate STDIO server configuration
fn validate_stdio_server(
    name: &str,
    server: &StdioServerConfig,
    errors: &mut Vec<ValidationError>,
) {
    let ctx = format!("mcp.servers.{name}");

    // Command must not be empty
    if server.command.trim().is_empty() {
        errors.push(ValidationError::with_context(
            "command cannot be empty",
            &ctx,
        ));
    }

    // Validate targets
    validate_targets(name, &server.targets, errors);

    // Check if command executable exists (warning only)
    if !server.command.contains('/') && !server.command.contains('\\') {
        // It's a command name, not a path - we could check PATH but that's expensive
        // For now, we'll just validate it's not empty (done above)
    }

    // Warn about bearer_token on STDIO server (should only be on HTTP)
    // This is checked implicitly by the type system - STDIO doesn't have bearer_token field
}

/// Validate HTTP server configuration
fn validate_http_server(name: &str, server: &HttpServerConfig, errors: &mut Vec<ValidationError>) {
    let ctx = format!("mcp.servers.{name}");

    // URL must start with http:// or https://
    if !server.url.starts_with("http://") && !server.url.starts_with("https://") {
        errors.push(ValidationError::with_context(
            format!(
                "URL must start with 'http://' or 'https://', got '{}'",
                server.url
            ),
            &ctx,
        ));
    }

    // Validate targets
    validate_targets(name, &server.targets, errors);
}

/// Validate targets array
fn validate_targets(server_name: &str, targets: &[String], errors: &mut Vec<ValidationError>) {
    let ctx = format!("mcp.servers.{server_name}.targets");
    let valid_tools = ["claude-code", "cursor", "opencode", "codex", "all"];

    for target in targets {
        if !valid_tools.contains(&target.as_str()) {
            errors.push(ValidationError::with_context(
                format!(
                    "Invalid tool name '{}', must be one of: {}",
                    target,
                    valid_tools.join(", ")
                ),
                &ctx,
            ));
        }
    }
}

/// Check if an executable exists in PATH
///
/// Returns true if the executable can be found, false otherwise.
/// This is a best-effort check and may have false negatives.
#[allow(dead_code)]
fn validate_executable(command: &str) -> bool {
    // Simple check - see if command exists
    // In a real implementation, we'd check PATH
    which::which(command).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{McpConfig, Settings};
    use std::collections::HashMap;

    fn create_minimal_valid_config() -> MultiAgentConfig {
        let mut servers = HashMap::new();
        servers.insert(
            "test".to_string(),
            ServerConfig::Stdio(StdioServerConfig {
                command: "npx".to_string(),
                args: vec![],
                enabled: true,
                targets: vec!["all".to_string()],
                env: None,
                disabled: None,
                auto_approve: None,
                startup_timeout_sec: None,
                tool_timeout_sec: None,
            }),
        );

        MultiAgentConfig {
            settings: Some(Settings {
                version: "1.0".to_string(),
                default_targets: vec!["cursor".to_string()],
            }),
            env: None,
            mcp: McpConfig { servers },
        }
    }

    #[test]
    fn test_validate_valid_config() {
        let config = create_minimal_valid_config();
        let result = validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_version_format() {
        let mut config = create_minimal_valid_config();
        config.settings.as_mut().unwrap().version = "invalid".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("Invalid version format"))
        );
    }

    #[test]
    fn test_validate_unsupported_version() {
        let mut config = create_minimal_valid_config();
        config.settings.as_mut().unwrap().version = "2.0".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("Unsupported version"))
        );
    }

    #[test]
    fn test_validate_invalid_tool_name_in_settings() {
        let mut config = create_minimal_valid_config();
        config.settings.as_mut().unwrap().default_targets = vec!["invalid-tool".to_string()];

        let result = validate_config(&config);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("Invalid tool name"))
        );
    }

    #[test]
    fn test_validate_duplicate_targets_in_settings() {
        let mut config = create_minimal_valid_config();
        config.settings.as_mut().unwrap().default_targets =
            vec!["cursor".to_string(), "cursor".to_string()];

        let result = validate_config(&config);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("Duplicate target"))
        );
    }

    #[test]
    fn test_validate_no_servers() {
        let mut config = create_minimal_valid_config();
        config.mcp.servers.clear();

        let result = validate_config(&config);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("At least one MCP server"))
        );
    }

    #[test]
    fn test_validate_empty_command() {
        let mut servers = HashMap::new();
        servers.insert(
            "test".to_string(),
            ServerConfig::Stdio(StdioServerConfig {
                command: String::new(),
                args: vec![],
                enabled: true,
                targets: vec!["all".to_string()],
                env: None,
                disabled: None,
                auto_approve: None,
                startup_timeout_sec: None,
                tool_timeout_sec: None,
            }),
        );

        let config = MultiAgentConfig {
            settings: Some(Settings {
                version: "1.0".to_string(),
                default_targets: vec!["cursor".to_string()],
            }),
            env: None,
            mcp: McpConfig { servers },
        };

        let result = validate_config(&config);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("command cannot be empty"))
        );
    }

    #[test]
    fn test_validate_invalid_http_url() {
        let mut servers = HashMap::new();
        servers.insert(
            "test".to_string(),
            ServerConfig::Http(HttpServerConfig {
                url: "ftp://example.com".to_string(),
                bearer_token: None,
                enabled: true,
                targets: vec!["all".to_string()],
            }),
        );

        let config = MultiAgentConfig {
            settings: Some(Settings {
                version: "1.0".to_string(),
                default_targets: vec!["cursor".to_string()],
            }),
            env: None,
            mcp: McpConfig { servers },
        };

        let result = validate_config(&config);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("must start with 'http://'"))
        );
    }

    #[test]
    fn test_validate_invalid_targets_in_server() {
        let mut servers = HashMap::new();
        servers.insert(
            "test".to_string(),
            ServerConfig::Stdio(StdioServerConfig {
                command: "npx".to_string(),
                args: vec![],
                enabled: true,
                targets: vec!["invalid-target".to_string()],
                env: None,
                disabled: None,
                auto_approve: None,
                startup_timeout_sec: None,
                tool_timeout_sec: None,
            }),
        );

        let config = MultiAgentConfig {
            settings: Some(Settings {
                version: "1.0".to_string(),
                default_targets: vec!["cursor".to_string()],
            }),
            env: None,
            mcp: McpConfig { servers },
        };

        let result = validate_config(&config);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| { e.context.as_ref().is_some_and(|c| c.contains("targets")) })
        );
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::new("test message");
        assert_eq!(format!("{err}"), "test message");

        let err = ValidationError::with_context("test message", "context");
        assert_eq!(format!("{err}"), "context: test message");
    }

    #[test]
    fn test_validation_error_equality() {
        let err1 = ValidationError::new("msg");
        let err2 = ValidationError::new("msg");
        assert_eq!(err1, err2);

        let err3 = ValidationError::with_context("msg", "ctx");
        let err4 = ValidationError::with_context("msg", "ctx");
        assert_eq!(err3, err4);
    }
}
