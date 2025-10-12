//! Target filtering logic for servers

use crate::config::types::{ServerConfig, ToolName};
use std::collections::HashMap;

/// Filter servers for a specific tool based on targets
///
/// Implements the target filtering algorithm from specification Section 7.2.
///
/// # Arguments
///
/// * `servers` - All servers from configuration
/// * `tool_name` - Tool to filter for
/// * `default_targets` - Default targets from settings (empty if none specified)
///
/// # Returns
///
/// HashMap of servers that should be included for the specified tool
pub fn filter_servers_for_tool(
    servers: &HashMap<String, ServerConfig>,
    tool_name: ToolName,
    default_targets: &[String],
) -> HashMap<String, ServerConfig> {
    let mut filtered = HashMap::new();

    for (name, server) in servers {
        // Skip disabled servers
        if !is_server_enabled(server) {
            continue;
        }

        // Get targets for this server
        let targets = get_server_targets(server, default_targets);

        // Check if this tool is in the targets
        if should_include_server(&targets, tool_name) {
            filtered.insert(name.clone(), server.clone());
        }
    }

    filtered
}

/// Check if a server is enabled
fn is_server_enabled(server: &ServerConfig) -> bool {
    match server {
        ServerConfig::Stdio(stdio) => stdio.enabled,
        ServerConfig::Http(http) => http.enabled,
    }
}

/// Get targets for a server, using defaults if not specified
fn get_server_targets(server: &ServerConfig, default_targets: &[String]) -> Vec<String> {
    match server {
        ServerConfig::Stdio(stdio) => {
            if stdio.targets.is_empty() || stdio.targets == vec!["all"] {
                if default_targets.is_empty() {
                    vec!["all".to_string()]
                } else {
                    default_targets.to_vec()
                }
            } else {
                stdio.targets.clone()
            }
        }
        ServerConfig::Http(http) => {
            if http.targets.is_empty() || http.targets == vec!["all"] {
                if default_targets.is_empty() {
                    vec!["all".to_string()]
                } else {
                    default_targets.to_vec()
                }
            } else {
                http.targets.clone()
            }
        }
    }
}

/// Check if a server should be included for the given tool
fn should_include_server(targets: &[String], tool_name: ToolName) -> bool {
    // Check if "all" is in targets
    if targets.iter().any(|t| t == "all") {
        return true;
    }

    // Check if the specific tool name is in targets
    targets.iter().any(|t| {
        ToolName::from_str(t)
            .map(|parsed| parsed == tool_name)
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{HttpServerConfig, StdioServerConfig};

    fn create_stdio_server(targets: Vec<String>, enabled: bool) -> ServerConfig {
        ServerConfig::Stdio(StdioServerConfig {
            command: "npx".to_string(),
            args: vec![],
            enabled,
            targets,
            env: None,
            disabled: None,
            auto_approve: None,
            startup_timeout_sec: None,
            tool_timeout_sec: None,
        })
    }

    fn create_http_server(targets: Vec<String>, enabled: bool) -> ServerConfig {
        ServerConfig::Http(HttpServerConfig {
            url: "https://example.com".to_string(),
            bearer_token: None,
            enabled,
            targets,
        })
    }

    #[test]
    fn test_filter_servers_with_all_target() {
        let mut servers = HashMap::new();
        servers.insert("test".to_string(), create_stdio_server(vec!["all".to_string()], true));

        let filtered = filter_servers_for_tool(&servers, ToolName::Cursor, &[]);
        assert_eq!(filtered.len(), 1);
        assert!(filtered.contains_key("test"));
    }

    #[test]
    fn test_filter_servers_with_specific_target() {
        let mut servers = HashMap::new();
        servers.insert(
            "cursor-only".to_string(),
            create_stdio_server(vec!["cursor".to_string()], true),
        );
        servers.insert(
            "codex-only".to_string(),
            create_stdio_server(vec!["codex".to_string()], true),
        );

        let filtered = filter_servers_for_tool(&servers, ToolName::Cursor, &[]);
        assert_eq!(filtered.len(), 1);
        assert!(filtered.contains_key("cursor-only"));
        assert!(!filtered.contains_key("codex-only"));
    }

    #[test]
    fn test_filter_servers_disabled() {
        let mut servers = HashMap::new();
        servers.insert(
            "disabled".to_string(),
            create_stdio_server(vec!["all".to_string()], false),
        );
        servers.insert(
            "enabled".to_string(),
            create_stdio_server(vec!["all".to_string()], true),
        );

        let filtered = filter_servers_for_tool(&servers, ToolName::Cursor, &[]);
        assert_eq!(filtered.len(), 1);
        assert!(filtered.contains_key("enabled"));
    }

    #[test]
    fn test_filter_servers_no_matching_targets() {
        let mut servers = HashMap::new();
        servers.insert(
            "codex-only".to_string(),
            create_stdio_server(vec!["codex".to_string()], true),
        );

        let filtered = filter_servers_for_tool(&servers, ToolName::Cursor, &[]);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_servers_with_default_targets() {
        let mut servers = HashMap::new();
        servers.insert(
            "test".to_string(),
            create_stdio_server(vec![], true), // Empty targets, should use defaults
        );

        let default_targets = vec!["cursor".to_string(), "codex".to_string()];
        let filtered = filter_servers_for_tool(&servers, ToolName::Cursor, &default_targets);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_servers_empty_list() {
        let servers = HashMap::new();
        let filtered = filter_servers_for_tool(&servers, ToolName::Cursor, &[]);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_servers_http_for_cursor() {
        let mut servers = HashMap::new();
        servers.insert(
            "http".to_string(),
            create_http_server(vec!["cursor".to_string()], true),
        );

        // HTTP servers should be included in filtering (the transformer will handle exclusion)
        let filtered = filter_servers_for_tool(&servers, ToolName::Cursor, &[]);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_servers_multiple_targets() {
        let mut servers = HashMap::new();
        servers.insert(
            "multi".to_string(),
            create_stdio_server(vec!["cursor".to_string(), "codex".to_string()], true),
        );

        let cursor_filtered = filter_servers_for_tool(&servers, ToolName::Cursor, &[]);
        assert_eq!(cursor_filtered.len(), 1);

        let codex_filtered = filter_servers_for_tool(&servers, ToolName::Codex, &[]);
        assert_eq!(codex_filtered.len(), 1);

        let opencode_filtered = filter_servers_for_tool(&servers, ToolName::Opencode, &[]);
        assert_eq!(opencode_filtered.len(), 0);
    }
}
