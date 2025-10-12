//! Init command implementation

use multi_agent_config::error::MultiAgentError;
use multi_agent_config::file_ops::{create_backup, write_file_atomic};
use std::path::Path;

/// Template configuration content
const TEMPLATE_CONFIG: &str = r#"# multi-agent-config - Unified Configuration for AI Coding Tools
# https://github.com/jfb/multi-agent-config

[settings]
version = "1.0"
default_targets = ["cursor", "opencode", "codex"]

# Environment variables section
# Variables defined here can be referenced as {VAR_NAME}
# Shell environment variables can be referenced as ${VAR_NAME}
[env]
# Example: GITHUB_TOKEN = "${GITHUB_PERSONAL_ACCESS_TOKEN}"
# Example: API_BASE = "https://api.example.com"

# MCP Servers Configuration
# Each server can target specific tools or use "all"

# Example STDIO server
[mcp.servers.example-stdio]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-example"]
enabled = true
targets = ["all"]

# Optional: environment variables for this server
# [mcp.servers.example-stdio.env]
# API_KEY = "{GITHUB_TOKEN}"

# Example: Cursor-specific server with auto-approve
# [mcp.servers.cursor-specific]
# command = "npx"
# args = ["-y", "package"]
# targets = ["cursor"]
# disabled = false
# autoApprove = ["tool_name"]

# Example: Codex-specific server with timeouts
# [mcp.servers.codex-specific]
# command = "node"
# args = ["server.js"]
# targets = ["codex"]
# startup_timeout_sec = 30
# tool_timeout_sec = 60

# Example: HTTP server
# [mcp.servers.remote-server]
# url = "https://api.example.com/mcp"
# bearer_token = "{API_TOKEN}"
# targets = ["opencode", "codex", "claude-code"]
# enabled = true
"#;

/// Initialize configuration file with template
///
/// # Arguments
///
/// * `config_path` - Path to configuration file
/// * `force` - Overwrite existing configuration
///
/// # Returns
///
/// * `Ok(())` - Config created successfully
/// * `Err(MultiAgentError)` - Error creating config
///
/// # Errors
///
/// Returns error if file exists without --force, or write operation fails
pub fn init_command(config_path: &Path, force: bool) -> Result<(), MultiAgentError> {
    // Check if config file exists
    if config_path.exists() && !force {
        return Err(MultiAgentError::CliError(format!(
            "Configuration file already exists: {}\nUse --force to overwrite",
            config_path.display()
        )));
    }

    // Create backup if file exists and force is true
    if config_path.exists() && force {
        match create_backup(config_path) {
            Ok(Some(backup_path)) => {
                eprintln!("Info: Created backup: {}", backup_path.display());
            }
            Ok(None) => {}
            Err(e) => {
                return Err(MultiAgentError::FileOpError(
                    multi_agent_config::file_ops::writer::FileOpError::IoError(e),
                ));
            }
        }
    }

    // Write template to config file
    write_file_atomic(config_path, TEMPLATE_CONFIG, Some(0o600))?;

    println!("Success: Configuration initialized: {}", config_path.display());
    println!();
    println!("Next steps:");
    println!("1. Edit the configuration file to add your MCP servers");
    println!("2. Run 'multi-agent-config validate' to check your config");
    println!("3. Run 'multi-agent-config compile' to generate tool configs");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_init_command_creates_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let result = init_command(&config_path, false);
        assert!(result.is_ok());
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[settings]"));
        assert!(content.contains("version = \"1.0\""));
        assert!(content.contains("[mcp.servers"));
    }

    #[test]
    fn test_init_command_existing_no_force() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create existing file
        fs::write(&config_path, "existing").unwrap();

        let result = init_command(&config_path, false);
        assert!(result.is_err());

        // Original file should be unchanged
        let content = fs::read_to_string(&config_path).unwrap();
        assert_eq!(content, "existing");
    }

    #[test]
    fn test_init_command_existing_with_force() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create existing file
        fs::write(&config_path, "existing").unwrap();

        let result = init_command(&config_path, true);
        assert!(result.is_ok());

        // Backup should exist
        let backup_path = config_path.with_extension("toml.backup");
        assert!(backup_path.exists());

        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, "existing");

        // New file should have template
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[settings]"));
    }

    #[test]
    fn test_init_command_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nested").join("dir").join("config.toml");

        let result = init_command(&config_path, false);
        assert!(result.is_ok());
        assert!(config_path.exists());
    }

    #[test]
    fn test_template_config_valid() {
        // Verify template is valid TOML
        let result: Result<toml::Value, _> = toml::from_str(TEMPLATE_CONFIG);
        assert!(result.is_ok());
    }
}
