//! Cursor JSON transformer

use crate::{
    config::types::{ServerConfig, StdioServerConfig, ToolName},
    transform::filter::filter_servers_for_tool,
};
use serde::Serialize;
use std::collections::HashMap;

/// Cursor configuration output structure
#[derive(Debug, Serialize)]
struct CursorConfig {
    /// MCP servers for Cursor
    #[serde(rename = "mcpServers")]
    mcp_servers: HashMap<String, CursorServer>,
}

/// Cursor server configuration
#[derive(Debug, Serialize)]
struct CursorServer {
    /// Command to execute
    command: String,

    /// Command arguments
    args: Vec<String>,

    /// Environment variables (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<HashMap<String, String>>,

    /// Whether server is disabled (optional, Cursor-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    disabled: Option<bool>,

    /// Tools to auto-approve (optional, Cursor-specific)
    #[serde(rename = "autoApprove", skip_serializing_if = "Option::is_none")]
    auto_approve: Option<Vec<String>>,
}

/// Transform servers to Cursor JSON format
///
/// Implements transformation rules from specification Section 7.3.
///
/// # Arguments
///
/// * `servers` - All servers from configuration
/// * `default_targets` - Default targets from settings
///
/// # Returns
///
/// * `Ok(String)` - JSON string formatted for Cursor
/// * `Err(String)` - Transformation error
///
/// # Errors
///
/// Returns error if JSON serialization fails
#[allow(clippy::implicit_hasher)]
pub fn transform_for_cursor(
    servers: &HashMap<String, ServerConfig>,
    default_targets: &[String],
) -> Result<String, String> {
    // Filter servers for Cursor
    let filtered = filter_servers_for_tool(servers, ToolName::Cursor, default_targets);

    let mut cursor_servers = HashMap::new();

    for (name, server) in filtered {
        // Cursor only supports STDIO servers, skip HTTP
        if let ServerConfig::Stdio(stdio) = server {
            let cursor_server = transform_stdio_server(&stdio);
            cursor_servers.insert(name, cursor_server);
        }
        // HTTP servers silently skipped for Cursor
    }

    let cursor_config = CursorConfig {
        mcp_servers: cursor_servers,
    };

    // Serialize to JSON with 2-space indentation
    serde_json::to_string_pretty(&cursor_config)
        .map_err(|e| format!("JSON serialization error: {e}"))
}

/// Transform a STDIO server to Cursor format
fn transform_stdio_server(stdio: &StdioServerConfig) -> CursorServer {
    CursorServer {
        command: stdio.command.clone(),
        args: stdio.args.clone(),
        env: stdio.env.clone(),
        disabled: stdio.disabled,
        auto_approve: stdio.auto_approve.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_stdio_server(command: &str, args: Vec<String>, targets: Vec<String>) -> ServerConfig {
        ServerConfig::Stdio(StdioServerConfig {
            command: command.to_string(),
            args,
            enabled: true,
            targets,
            env: None,
            disabled: None,
            auto_approve: None,
            startup_timeout_sec: None,
            tool_timeout_sec: None,
        })
    }

    #[test]
    fn test_transform_cursor_single_server() {
        let mut servers = HashMap::new();
        servers.insert(
            "test".to_string(),
            create_stdio_server("npx", vec!["-y".to_string()], vec!["cursor".to_string()]),
        );

        let result = transform_for_cursor(&servers, &[]);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json.contains("mcpServers"));
        assert!(json.contains("test"));
        assert!(json.contains("npx"));
        assert!(json.contains("-y"));
    }

    #[test]
    fn test_transform_cursor_with_env() {
        let mut servers = HashMap::new();
        let mut env_vars = HashMap::new();
        env_vars.insert("API_KEY".to_string(), "secret".to_string());

        servers.insert(
            "test".to_string(),
            ServerConfig::Stdio(StdioServerConfig {
                command: "npx".to_string(),
                args: vec![],
                enabled: true,
                targets: vec!["all".to_string()],
                env: Some(env_vars),
                disabled: None,
                auto_approve: None,
                startup_timeout_sec: None,
                tool_timeout_sec: None,
            }),
        );

        let result = transform_for_cursor(&servers, &[]);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json.contains("env"));
        assert!(json.contains("API_KEY"));
        assert!(json.contains("secret"));
    }

    #[test]
    fn test_transform_cursor_with_cursor_fields() {
        let mut servers = HashMap::new();
        servers.insert(
            "test".to_string(),
            ServerConfig::Stdio(StdioServerConfig {
                command: "npx".to_string(),
                args: vec![],
                enabled: true,
                targets: vec!["cursor".to_string()],
                env: None,
                disabled: Some(false),
                auto_approve: Some(vec!["tool1".to_string()]),
                startup_timeout_sec: None,
                tool_timeout_sec: None,
            }),
        );

        let result = transform_for_cursor(&servers, &[]);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json.contains("disabled"));
        assert!(json.contains("autoApprove"));
        assert!(json.contains("tool1"));
        // Should NOT contain Codex-specific fields
        assert!(!json.contains("startup_timeout_sec"));
        assert!(!json.contains("tool_timeout_sec"));
    }

    #[test]
    fn test_transform_cursor_excludes_http() {
        let mut servers = HashMap::new();
        servers.insert(
            "stdio".to_string(),
            create_stdio_server("npx", vec![], vec!["all".to_string()]),
        );
        servers.insert(
            "http".to_string(),
            ServerConfig::Http(crate::config::types::HttpServerConfig {
                url: "https://example.com".to_string(),
                bearer_token: None,
                enabled: true,
                targets: vec!["all".to_string()],
            }),
        );

        let result = transform_for_cursor(&servers, &[]);
        assert!(result.is_ok());

        let json = result.unwrap();
        // Should only contain STDIO server
        assert!(json.contains("stdio"));
        assert!(!json.contains("http"));
        assert!(!json.contains("https://example.com"));
    }

    #[test]
    fn test_transform_cursor_empty_servers() {
        let servers = HashMap::new();
        let result = transform_for_cursor(&servers, &[]);
        assert!(result.is_ok());

        let json = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["mcpServers"].as_object().unwrap().is_empty());
    }

    #[test]
    fn test_transform_cursor_disabled_servers() {
        let mut servers = HashMap::new();
        servers.insert(
            "disabled".to_string(),
            ServerConfig::Stdio(StdioServerConfig {
                command: "npx".to_string(),
                args: vec![],
                enabled: false, // Disabled
                targets: vec!["cursor".to_string()],
                env: None,
                disabled: None,
                auto_approve: None,
                startup_timeout_sec: None,
                tool_timeout_sec: None,
            }),
        );

        let result = transform_for_cursor(&servers, &[]);
        assert!(result.is_ok());

        let json = result.unwrap();
        // Should be empty because server is disabled
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["mcpServers"].as_object().unwrap().is_empty());
    }
}
