//! opencode.ai JSON transformer

use crate::config::types::{HttpServerConfig, ServerConfig, StdioServerConfig, ToolName};
use crate::transform::filter::filter_servers_for_tool;
use serde::Serialize;
use std::collections::HashMap;

/// opencode.ai configuration output structure
#[derive(Debug, Serialize)]
struct OpencodeConfig {
    /// MCP servers for opencode.ai
    mcp: HashMap<String, OpencodeServer>,
}

/// opencode.ai server configuration (STDIO or HTTP)
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum OpencodeServer {
    /// Local STDIO server
    Local(OpencodeLocalServer),
    /// Remote HTTP server
    Remote(OpencodeRemoteServer),
}

/// opencode.ai local STDIO server
#[derive(Debug, Serialize)]
pub struct OpencodeLocalServer {
    /// Server type (always "local")
    #[serde(rename = "type")]
    server_type: String,

    /// Command as array: [executable, ...args]
    command: Vec<String>,

    /// Environment variables (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<HashMap<String, String>>,

    /// Whether server is enabled
    enabled: bool,
}

/// opencode.ai remote HTTP server
#[derive(Debug, Serialize)]
pub struct OpencodeRemoteServer {
    /// Server type (always "remote")
    #[serde(rename = "type")]
    server_type: String,

    /// Server URL
    url: String,

    /// Headers (optional, for bearer token)
    #[serde(skip_serializing_if = "Option::is_none")]
    headers: Option<HashMap<String, String>>,

    /// Whether server is enabled
    enabled: bool,
}

/// Transform servers to opencode.ai JSON format
///
/// Implements transformation rules from specification Section 7.4.
///
/// # Arguments
///
/// * `servers` - All servers from configuration
/// * `default_targets` - Default targets from settings
///
/// # Returns
///
/// * `Ok(String)` - JSON string formatted for opencode.ai
/// * `Err(String)` - Transformation error
///
/// # Errors
///
/// Returns error if JSON serialization fails
pub fn transform_for_opencode(
    servers: &HashMap<String, ServerConfig>,
    default_targets: &[String],
) -> Result<String, String> {
    // Filter servers for opencode
    let filtered = filter_servers_for_tool(servers, ToolName::Opencode, default_targets);

    let mut opencode_servers = HashMap::new();

    for (name, server) in filtered {
        let opencode_server = match server {
            ServerConfig::Stdio(stdio) => {
                OpencodeServer::Local(transform_stdio_server(&stdio))
            }
            ServerConfig::Http(http) => {
                OpencodeServer::Remote(transform_http_server(&http))
            }
        };
        opencode_servers.insert(name, opencode_server);
    }

    let opencode_config = OpencodeConfig {
        mcp: opencode_servers,
    };

    // Serialize to JSON with 2-space indentation
    serde_json::to_string_pretty(&opencode_config)
        .map_err(|e| format!("JSON serialization error: {e}"))
}

/// Transform a STDIO server to opencode.ai local format
pub fn transform_stdio_server(stdio: &StdioServerConfig) -> OpencodeLocalServer {
    // Combine command and args into single array
    let mut command = vec![stdio.command.clone()];
    command.extend(stdio.args.clone());

    OpencodeLocalServer {
        server_type: "local".to_string(),
        command,
        env: stdio.env.clone(),
        enabled: stdio.enabled,
    }
}

/// Transform an HTTP server to opencode.ai remote format
pub fn transform_http_server(http: &HttpServerConfig) -> OpencodeRemoteServer {
    // Convert bearer_token to Authorization header if present
    let headers = http.bearer_token.as_ref().map(|token| {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {token}"));
        headers
    });

    OpencodeRemoteServer {
        server_type: "remote".to_string(),
        url: http.url.clone(),
        headers,
        enabled: http.enabled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_stdio_server(
        command: &str,
        args: Vec<String>,
        targets: Vec<String>,
    ) -> ServerConfig {
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

    fn create_http_server(url: &str, bearer_token: Option<String>, targets: Vec<String>) -> ServerConfig {
        ServerConfig::Http(HttpServerConfig {
            url: url.to_string(),
            bearer_token,
            enabled: true,
            targets,
        })
    }

    #[test]
    fn test_transform_opencode_stdio_server() {
        let mut servers = HashMap::new();
        servers.insert(
            "test".to_string(),
            create_stdio_server("npx", vec!["-y".to_string(), "package".to_string()], vec!["opencode".to_string()]),
        );

        let result = transform_for_opencode(&servers, &[]);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json.contains("mcp"));
        assert!(json.contains("test"));
        assert!(json.contains("\"type\": \"local\""));
        assert!(json.contains("npx"));
        assert!(json.contains("-y"));
        assert!(json.contains("package"));
        assert!(json.contains("\"enabled\": true"));
    }

    #[test]
    fn test_transform_opencode_http_server() {
        let mut servers = HashMap::new();
        servers.insert(
            "remote".to_string(),
            create_http_server("https://example.com/mcp", Some("token123".to_string()), vec!["all".to_string()]),
        );

        let result = transform_for_opencode(&servers, &[]);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json.contains("\"type\": \"remote\""));
        assert!(json.contains("https://example.com/mcp"));
        assert!(json.contains("Authorization"));
        assert!(json.contains("Bearer token123"));
    }

    #[test]
    fn test_transform_opencode_with_env() {
        let mut env_vars = HashMap::new();
        env_vars.insert("API_KEY".to_string(), "secret".to_string());

        let mut servers = HashMap::new();
        servers.insert(
            "test".to_string(),
            ServerConfig::Stdio(StdioServerConfig {
                command: "node".to_string(),
                args: vec![],
                enabled: true,
                targets: vec!["opencode".to_string()],
                env: Some(env_vars),
                disabled: None,
                auto_approve: None,
                startup_timeout_sec: None,
                tool_timeout_sec: None,
            }),
        );

        let result = transform_for_opencode(&servers, &[]);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json.contains("env"));
        assert!(json.contains("API_KEY"));
        assert!(json.contains("secret"));
    }

    #[test]
    fn test_transform_opencode_command_array() {
        let mut servers = HashMap::new();
        servers.insert(
            "test".to_string(),
            create_stdio_server("node", vec!["server.js".to_string()], vec!["all".to_string()]),
        );

        let result = transform_for_opencode(&servers, &[]);
        assert!(result.is_ok());

        let json = result.unwrap();
        // Parse to verify command structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let command = &parsed["mcp"]["test"]["command"];
        assert!(command.is_array());
        assert_eq!(command.as_array().unwrap().len(), 2);
        assert_eq!(command[0], "node");
        assert_eq!(command[1], "server.js");
    }

    #[test]
    fn test_transform_opencode_mixed_servers() {
        let mut servers = HashMap::new();
        servers.insert(
            "local".to_string(),
            create_stdio_server("npx", vec![], vec!["all".to_string()]),
        );
        servers.insert(
            "remote".to_string(),
            create_http_server("https://api.example.com", None, vec!["all".to_string()]),
        );

        let result = transform_for_opencode(&servers, &[]);
        assert!(result.is_ok());

        let json = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Both servers should be present
        assert!(parsed["mcp"]["local"].is_object());
        assert!(parsed["mcp"]["remote"].is_object());

        // Check types
        assert_eq!(parsed["mcp"]["local"]["type"], "local");
        assert_eq!(parsed["mcp"]["remote"]["type"], "remote");
    }

    #[test]
    fn test_transform_opencode_empty_servers() {
        let servers = HashMap::new();
        let result = transform_for_opencode(&servers, &[]);
        assert!(result.is_ok());

        let json = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["mcp"].as_object().unwrap().is_empty());
    }
}
