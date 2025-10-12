//! Diff command implementation

use multi_agent_config::config::{ToolName, parse_and_expand_config, validate_config};
use multi_agent_config::error::MultiAgentError;
use multi_agent_config::file_ops::generate_file_diff;
use multi_agent_config::transform::{
    transform_for_claude_code, transform_for_codex, transform_for_cursor, transform_for_opencode,
};
use std::path::{Path, PathBuf};

/// Get output path for a tool's configuration
fn get_tool_config_path(tool: ToolName) -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));

    match tool {
        ToolName::Cursor => config_dir
            .join("Cursor")
            .join("User")
            .join("globalStorage")
            .join("saoudrizwan.claude-dev")
            .join("settings")
            .join("mcp.json"),
        ToolName::Opencode => config_dir.join("opencode").join("mcp.json"),
        ToolName::Codex => config_dir.join("codex").join("mcp_config.toml"),
        ToolName::ClaudeCode => config_dir.join("claude").join("mcp.json"),
        ToolName::All => panic!("Cannot get path for 'all' tool"),
    }
}

/// Show diff of what would change
///
/// # Arguments
///
/// * `config_path` - Path to unified configuration file
/// * `tools` - Specific tools to show diff for (empty = all matching servers)
/// * `verbose` - Enable verbose output
///
/// # Returns
///
/// * `Ok(())` - Diff displayed successfully
/// * `Err(MultiAgentError)` - Error during diff generation
///
/// # Errors
///
/// Returns error if config invalid or transformation fails
pub fn diff_command(
    config_path: &Path,
    tools: &[String],
    verbose: bool,
) -> Result<(), MultiAgentError> {
    // Parse and expand configuration
    let config = parse_and_expand_config(config_path)?;

    // Validate
    if let Err(errors) = validate_config(&config) {
        eprintln!("Validation failed:");
        for error in &errors {
            eprintln!("  - {error}");
        }
        return Err(MultiAgentError::Config(
            multi_agent_config::error::ConfigError::ValidationError(format!(
                "{} error(s)",
                errors.len()
            )),
        ));
    }

    // Determine target tools
    let target_tools: Vec<ToolName> = if tools.is_empty() {
        ToolName::concrete_tools()
    } else {
        tools.iter().filter_map(|t| ToolName::from_str(t)).collect()
    };

    // Get default targets from settings
    let default_targets = config
        .settings
        .as_ref()
        .map(|s| s.default_targets.clone())
        .unwrap_or_default();

    // Generate diff for each tool
    for tool in target_tools {
        if verbose {
            println!("Generating diff for {tool}...");
        }

        // Transform configuration
        let new_content = match tool {
            ToolName::Cursor => transform_for_cursor(&config.mcp.servers, &default_targets)
                .map_err(MultiAgentError::TransformError)?,
            ToolName::Opencode => transform_for_opencode(&config.mcp.servers, &default_targets)
                .map_err(MultiAgentError::TransformError)?,
            ToolName::Codex => transform_for_codex(&config.mcp.servers, &default_targets)
                .map_err(MultiAgentError::TransformError)?,
            ToolName::ClaudeCode => {
                transform_for_claude_code(&config.mcp.servers, &default_targets)
                    .map_err(MultiAgentError::TransformError)?
            }
            ToolName::All => continue,
        };

        let output_path = get_tool_config_path(tool);

        // Generate and display diff
        let diff = generate_file_diff(&output_path, &new_content);

        println!("=== {} ({}) ===", tool, output_path.display());
        println!("{diff}");
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_diff_command_valid_config() {
        let toml_content = r#"
[settings]
version = "1.0"

[mcp.servers.test]
command = "npx"
targets = ["cursor"]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let result = diff_command(temp_file.path(), &[], false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_diff_command_specific_tool() {
        let toml_content = r#"
[settings]
version = "1.0"

[mcp.servers.test]
command = "npx"
targets = ["all"]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let result = diff_command(temp_file.path(), &["cursor".to_string()], false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_diff_command_invalid_config() {
        let toml_content = r#"
[settings]
version = "invalid"

[mcp.servers.test]
command = "npx"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let result = diff_command(temp_file.path(), &[], false);
        assert!(result.is_err());
    }
}
