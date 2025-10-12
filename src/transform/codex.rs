//! Codex TOML transformer

use crate::{
    config::types::{HttpServerConfig, ServerConfig, StdioServerConfig, ToolName},
    transform::filter::filter_servers_for_tool,
};
use serde::Serialize;
use std::collections::HashMap;

/// Codex configuration output structure
#[derive(Debug, Serialize)]
struct CodexConfig {
    /// MCP servers for Codex
    mcp_servers: HashMap<String, CodexServer>,
}

/// Codex server configuration (STDIO or HTTP)
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum CodexServer {
    /// STDIO server
    Stdio(CodexStdioServer),
    /// HTTP server
    Http(CodexHttpServer),
}

/// Codex STDIO server
#[derive(Debug, Serialize)]
struct CodexStdioServer {
    /// Command to execute
    command: String,

    /// Command arguments (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<Vec<String>>,

    /// Startup timeout in seconds (optional, Codex-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    startup_timeout_sec: Option<u32>,

    /// Tool timeout in seconds (optional, Codex-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_timeout_sec: Option<u32>,

    /// Environment variables (optional, as separate table in TOML)
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<HashMap<String, String>>,
}

/// Codex HTTP server
#[derive(Debug, Serialize)]
struct CodexHttpServer {
    /// Server URL
    url: String,

    /// Bearer token for authentication (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    bearer_token: Option<String>,
}

/// Transform servers to Codex TOML format
///
/// Implements transformation rules from specification Section 7.5.
///
/// # Arguments
///
/// * `servers` - All servers from configuration
/// * `default_targets` - Default targets from settings
///
/// # Returns
///
/// * `Ok(String)` - TOML string formatted for Codex
/// * `Err(String)` - Transformation error
///
/// # Errors
///
/// Returns error if TOML serialization fails
#[allow(clippy::implicit_hasher)]
pub fn transform_for_codex(
    servers: &HashMap<String, ServerConfig>,
    default_targets: &[String],
) -> Result<String, String> {
    // Filter servers for Codex
    let filtered = filter_servers_for_tool(servers, ToolName::Codex, default_targets);

    let mut codex_servers = HashMap::new();

    for (name, server) in filtered {
        let codex_server = match server {
            ServerConfig::Stdio(stdio) => CodexServer::Stdio(transform_stdio_server(&stdio)),
            ServerConfig::Http(http) => CodexServer::Http(transform_http_server(&http)),
        };
        codex_servers.insert(name, codex_server);
    }

    let codex_config = CodexConfig {
        mcp_servers: codex_servers,
    };

    // Serialize to TOML
    toml::to_string_pretty(&codex_config).map_err(|e| format!("TOML serialization error: {e}"))
}

/// Transform a STDIO server to Codex format
fn transform_stdio_server(stdio: &StdioServerConfig) -> CodexStdioServer {
    CodexStdioServer {
        command: stdio.command.clone(),
        args: if stdio.args.is_empty() {
            None
        } else {
            Some(stdio.args.clone())
        },
        startup_timeout_sec: stdio.startup_timeout_sec,
        tool_timeout_sec: stdio.tool_timeout_sec,
        env: stdio.env.clone(),
    }
}

/// Transform an HTTP server to Codex format
fn transform_http_server(http: &HttpServerConfig) -> CodexHttpServer {
    CodexHttpServer {
        url: http.url.clone(),
        bearer_token: http.bearer_token.clone(),
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

    fn create_http_server(
        url: &str,
        bearer_token: Option<String>,
        targets: Vec<String>,
    ) -> ServerConfig {
        ServerConfig::Http(HttpServerConfig {
            url: url.to_string(),
            bearer_token,
            enabled: true,
            targets,
        })
    }

    #[test]
    fn test_transform_codex_stdio_server() {
        let mut servers = HashMap::new();
        servers.insert(
            "test".to_string(),
            create_stdio_server(
                "npx",
                vec!["-y".to_string(), "package".to_string()],
                vec!["codex".to_string()],
            ),
        );

        let result = transform_for_codex(&servers, &[]);
        assert!(result.is_ok());

        let toml_str = result.unwrap();
        println!("Generated TOML:\n{toml_str}");
        assert!(toml_str.contains("[mcp_servers.test]"));
        assert!(toml_str.contains("command = \"npx\""));
        // TOML arrays might have different whitespace
        assert!(toml_str.contains("-y"));
        assert!(toml_str.contains("package"));
    }

    #[test]
    fn test_transform_codex_with_timeouts() {
        let mut servers = HashMap::new();
        servers.insert(
            "test".to_string(),
            ServerConfig::Stdio(StdioServerConfig {
                command: "node".to_string(),
                args: vec![],
                enabled: true,
                targets: vec!["codex".to_string()],
                env: None,
                disabled: None,
                auto_approve: None,
                startup_timeout_sec: Some(30),
                tool_timeout_sec: Some(60),
            }),
        );

        let result = transform_for_codex(&servers, &[]);
        assert!(result.is_ok());

        let toml_str = result.unwrap();
        assert!(toml_str.contains("startup_timeout_sec = 30"));
        assert!(toml_str.contains("tool_timeout_sec = 60"));
        // Should NOT contain Cursor-specific fields
        assert!(!toml_str.contains("disabled"));
        assert!(!toml_str.contains("autoApprove"));
    }

    #[test]
    fn test_transform_codex_http_server() {
        let mut servers = HashMap::new();
        servers.insert(
            "remote".to_string(),
            create_http_server(
                "https://api.example.com/mcp",
                Some("token123".to_string()),
                vec!["all".to_string()],
            ),
        );

        let result = transform_for_codex(&servers, &[]);
        assert!(result.is_ok());

        let toml_str = result.unwrap();
        assert!(toml_str.contains("[mcp_servers.remote]"));
        assert!(toml_str.contains("url = \"https://api.example.com/mcp\""));
        assert!(toml_str.contains("bearer_token = \"token123\""));
    }

    #[test]
    fn test_transform_codex_with_env() {
        let mut env_vars = HashMap::new();
        env_vars.insert("API_KEY".to_string(), "secret".to_string());

        let mut servers = HashMap::new();
        servers.insert(
            "test".to_string(),
            ServerConfig::Stdio(StdioServerConfig {
                command: "node".to_string(),
                args: vec![],
                enabled: true,
                targets: vec!["codex".to_string()],
                env: Some(env_vars),
                disabled: None,
                auto_approve: None,
                startup_timeout_sec: None,
                tool_timeout_sec: None,
            }),
        );

        let result = transform_for_codex(&servers, &[]);
        assert!(result.is_ok());

        let toml_str = result.unwrap();
        assert!(toml_str.contains("[mcp_servers.test.env]"));
        assert!(toml_str.contains("API_KEY = \"secret\""));
    }

    #[test]
    fn test_transform_codex_mixed_servers() {
        let mut servers = HashMap::new();
        servers.insert(
            "local".to_string(),
            create_stdio_server("npx", vec![], vec!["all".to_string()]),
        );
        servers.insert(
            "remote".to_string(),
            create_http_server("https://api.example.com", None, vec!["all".to_string()]),
        );

        let result = transform_for_codex(&servers, &[]);
        assert!(result.is_ok());

        let toml_str = result.unwrap();
        assert!(toml_str.contains("[mcp_servers.local]"));
        assert!(toml_str.contains("[mcp_servers.remote]"));
    }

    #[test]
    fn test_transform_codex_empty_servers() {
        let servers = HashMap::new();
        let result = transform_for_codex(&servers, &[]);
        assert!(result.is_ok());

        let toml_str = result.unwrap();
        // Empty config should have empty mcp_servers table
        assert!(toml_str.contains("mcp_servers"));
    }
}
