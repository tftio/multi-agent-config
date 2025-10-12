//! Core data structures for multi-agent-config
//!
//! This module defines the types that represent the unified configuration
//! schema.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAgentConfig {
    /// Optional settings section
    #[serde(default)]
    pub settings: Option<Settings>,

    /// Optional environment variables section
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,

    /// Required MCP servers configuration
    pub mcp: McpConfig,
}

/// Settings section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Configuration version (required, validated as semver)
    pub version: String,

    /// Default target tools for servers
    #[serde(default = "default_targets")]
    pub default_targets: Vec<String>,
}

/// Default targets: cursor, opencode, codex
fn default_targets() -> Vec<String> {
    vec![
        "cursor".to_string(),
        "opencode".to_string(),
        "codex".to_string(),
    ]
}

/// MCP configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Map of server name to server configuration
    pub servers: HashMap<String, ServerConfig>,
}

/// Server configuration (STDIO or HTTP)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServerConfig {
    /// STDIO server configuration
    Stdio(StdioServerConfig),
    /// HTTP server configuration
    Http(HttpServerConfig),
}

/// STDIO server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdioServerConfig {
    /// Command to execute (executable name or path)
    pub command: String,

    /// Command arguments
    #[serde(default)]
    pub args: Vec<String>,

    /// Whether server is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Target tools for this server
    #[serde(default = "default_all_targets")]
    pub targets: Vec<String>,

    /// Environment variables for the server
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,

    /// Cursor-specific: whether server is disabled
    #[serde(default)]
    pub disabled: Option<bool>,

    /// Cursor-specific: tools to auto-approve
    #[serde(rename = "autoApprove", default)]
    pub auto_approve: Option<Vec<String>>,

    /// Codex-specific: startup timeout in seconds
    #[serde(default)]
    pub startup_timeout_sec: Option<u32>,

    /// Codex-specific: tool timeout in seconds
    #[serde(default)]
    pub tool_timeout_sec: Option<u32>,
}

/// HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpServerConfig {
    /// Server URL (must start with http:// or https://)
    pub url: String,

    /// Optional bearer token for authentication
    #[serde(default)]
    pub bearer_token: Option<String>,

    /// Whether server is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Target tools for this server
    #[serde(default = "default_all_targets")]
    pub targets: Vec<String>,
}

/// Default value for boolean fields: true
const fn default_true() -> bool {
    true
}

/// Default value for targets: `["all"]`
fn default_all_targets() -> Vec<String> {
    vec!["all".to_string()]
}

/// Supported tool names
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolName {
    /// Claude Code
    ClaudeCode,
    /// Cursor
    Cursor,
    /// opencode.ai
    Opencode,
    /// `OpenAI` Codex
    Codex,
    /// All tools
    All,
}

impl ToolName {
    /// Get all concrete tool names (excluding "all")
    #[must_use]
    pub fn concrete_tools() -> Vec<Self> {
        vec![Self::ClaudeCode, Self::Cursor, Self::Opencode, Self::Codex]
    }

    /// Parse from string
    ///
    /// Note: This is different from `FromStr` trait which returns `Result`
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "claude-code" => Some(Self::ClaudeCode),
            "cursor" => Some(Self::Cursor),
            "opencode" => Some(Self::Opencode),
            "codex" => Some(Self::Codex),
            "all" => Some(Self::All),
            _ => None,
        }
    }

    /// Convert to string
    #[must_use]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Cursor => "cursor",
            Self::Opencode => "opencode",
            Self::Codex => "codex",
            Self::All => "all",
        }
    }
}

impl std::fmt::Display for ToolName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ToolName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s).ok_or_else(|| format!("Invalid tool name: {s}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_targets() {
        let targets = default_targets();
        assert_eq!(targets.len(), 3);
        assert!(targets.contains(&"cursor".to_string()));
        assert!(targets.contains(&"opencode".to_string()));
        assert!(targets.contains(&"codex".to_string()));
    }

    #[test]
    fn test_default_true() {
        assert!(default_true());
    }

    #[test]
    fn test_default_all_targets() {
        let targets = default_all_targets();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0], "all");
    }

    #[test]
    fn test_tool_name_from_str() {
        assert_eq!(
            ToolName::from_str("claude-code"),
            Some(ToolName::ClaudeCode)
        );
        assert_eq!(ToolName::from_str("cursor"), Some(ToolName::Cursor));
        assert_eq!(ToolName::from_str("opencode"), Some(ToolName::Opencode));
        assert_eq!(ToolName::from_str("codex"), Some(ToolName::Codex));
        assert_eq!(ToolName::from_str("all"), Some(ToolName::All));
        assert_eq!(ToolName::from_str("invalid"), None);
    }

    #[test]
    fn test_tool_name_as_str() {
        assert_eq!(ToolName::ClaudeCode.as_str(), "claude-code");
        assert_eq!(ToolName::Cursor.as_str(), "cursor");
        assert_eq!(ToolName::Opencode.as_str(), "opencode");
        assert_eq!(ToolName::Codex.as_str(), "codex");
        assert_eq!(ToolName::All.as_str(), "all");
    }

    #[test]
    fn test_tool_name_display() {
        assert_eq!(format!("{}", ToolName::ClaudeCode), "claude-code");
        assert_eq!(format!("{}", ToolName::Cursor), "cursor");
    }

    #[test]
    fn test_tool_name_concrete_tools() {
        let tools = ToolName::concrete_tools();
        assert_eq!(tools.len(), 4);
        assert!(tools.contains(&ToolName::ClaudeCode));
        assert!(tools.contains(&ToolName::Cursor));
        assert!(tools.contains(&ToolName::Opencode));
        assert!(tools.contains(&ToolName::Codex));
        assert!(!tools.contains(&ToolName::All));
    }
}
