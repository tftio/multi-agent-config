//! Claude Code JSON transformer
//!
//! Claude Code uses the same format as opencode.ai

use crate::config::types::{ServerConfig, ToolName};
use crate::transform::filter::filter_servers_for_tool;
use crate::transform::opencode::{OpencodeLocalServer, OpencodeRemoteServer, OpencodeServer};
use serde::Serialize;
use std::collections::HashMap;

/// Claude Code configuration output structure (same as opencode.ai)
#[derive(Debug, Serialize)]
struct ClaudeCodeConfig {
    /// MCP servers for Claude Code
    mcp: HashMap<String, OpencodeServer>,
}

/// Transform servers to Claude Code JSON format
///
/// Claude Code uses the same MCP configuration format as opencode.ai.
///
/// # Arguments
///
/// * `servers` - All servers from configuration
/// * `default_targets` - Default targets from settings
///
/// # Returns
///
/// * `Ok(String)` - JSON string formatted for Claude Code
/// * `Err(String)` - Transformation error
///
/// # Errors
///
/// Returns error if JSON serialization fails
pub fn transform_for_claude_code(
    servers: &HashMap<String, ServerConfig>,
    default_targets: &[String],
) -> Result<String, String> {
    // Filter servers for Claude Code
    let filtered = filter_servers_for_tool(servers, ToolName::ClaudeCode, default_targets);

    let mut claude_servers = HashMap::new();

    for (name, server) in filtered {
        let opencode_server = match server {
            ServerConfig::Stdio(stdio) => {
                OpencodeServer::Local(crate::transform::opencode::transform_stdio_server(&stdio))
            }
            ServerConfig::Http(http) => {
                OpencodeServer::Remote(crate::transform::opencode::transform_http_server(&http))
            }
        };
        claude_servers.insert(name, opencode_server);
    }

    let claude_config = ClaudeCodeConfig {
        mcp: claude_servers,
    };

    // Serialize to JSON with 2-space indentation
    serde_json::to_string_pretty(&claude_config)
        .map_err(|e| format!("JSON serialization error: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{HttpServerConfig, StdioServerConfig};

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

    #[test]
    fn test_transform_claude_code_stdio() {
        let mut servers = HashMap::new();
        servers.insert(
            "test".to_string(),
            create_stdio_server("npx", vec!["-y".to_string()], vec!["claude-code".to_string()]),
        );

        let result = transform_for_claude_code(&servers, &[]);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json.contains("mcp"));
        assert!(json.contains("test"));
        assert!(json.contains("\"type\": \"local\""));
        assert!(json.contains("npx"));
    }

    #[test]
    fn test_transform_claude_code_http() {
        let mut servers = HashMap::new();
        servers.insert(
            "remote".to_string(),
            ServerConfig::Http(HttpServerConfig {
                url: "https://example.com".to_string(),
                bearer_token: Some("token".to_string()),
                enabled: true,
                targets: vec!["all".to_string()],
            }),
        );

        let result = transform_for_claude_code(&servers, &[]);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json.contains("\"type\": \"remote\""));
        assert!(json.contains("https://example.com"));
    }

    #[test]
    fn test_transform_claude_code_filters_correctly() {
        let mut servers = HashMap::new();
        servers.insert(
            "claude-only".to_string(),
            create_stdio_server("npx", vec![], vec!["claude-code".to_string()]),
        );
        servers.insert(
            "cursor-only".to_string(),
            create_stdio_server("npx", vec![], vec!["cursor".to_string()]),
        );

        let result = transform_for_claude_code(&servers, &[]);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json.contains("claude-only"));
        assert!(!json.contains("cursor-only"));
    }

    #[test]
    fn test_transform_claude_code_empty() {
        let servers = HashMap::new();
        let result = transform_for_claude_code(&servers, &[]);
        assert!(result.is_ok());

        let json = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["mcp"].as_object().unwrap().is_empty());
    }
}
