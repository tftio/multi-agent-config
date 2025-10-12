//! Compile command implementation

use multi_agent_config::config::{ToolName, parse_and_expand_config, validate_config};
use multi_agent_config::error::MultiAgentError;
use multi_agent_config::file_ops::{StateTracker, create_backup, hash_file, write_file_atomic};
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

/// Compile and write tool configurations
///
/// # Arguments
///
/// * `config_path` - Path to unified configuration file
/// * `tools` - Specific tools to compile for (empty = all matching servers)
/// * `dry_run` - Show what would be done without writing
/// * `verbose` - Enable verbose output
///
/// # Returns
///
/// * `Ok(())` - Compilation successful
/// * `Err(MultiAgentError)` - Error during compilation
///
/// # Errors
///
/// Returns error if config invalid, transformation fails, or write fails
pub fn compile_command(
    config_path: &Path,
    tools: &[String],
    dry_run: bool,
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

    // Load state tracker
    let state_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("multi-agent-config")
        .join("state")
        .join("generated.json");
    let mut state_tracker = StateTracker::load(&state_path).map_err(|e| {
        MultiAgentError::FileOpError(multi_agent_config::file_ops::writer::FileOpError::Io(e))
    })?;

    // Compile for each tool
    let mut compiled_count = 0;

    for tool in target_tools {
        if verbose {
            println!("Compiling for {tool}...");
        }

        // Transform configuration
        let output_content = match tool {
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

        if dry_run {
            println!("Would write to: {}", output_path.display());
            if verbose {
                println!("Content preview (first 200 chars):");
                println!(
                    "{}...",
                    output_content.chars().take(200).collect::<String>()
                );
            }
        } else {
            // Create backup if file exists
            if let Ok(Some(backup_path)) = create_backup(&output_path) {
                if verbose {
                    println!("  Created backup: {}", backup_path.display());
                }
            }

            // Write file
            write_file_atomic(&output_path, &output_content, Some(0o600))?;

            // Compute hash and update state
            let hash = hash_file(&output_path).map_err(|e| {
                MultiAgentError::FileOpError(multi_agent_config::file_ops::writer::FileOpError::Io(
                    e,
                ))
            })?;
            state_tracker.add_generated_file(&tool.to_string(), output_path.clone(), hash);

            println!("  {} -> {}", tool, output_path.display());
            compiled_count += 1;
        }
    }

    // Save state
    if !dry_run {
        state_tracker.save().map_err(|e| {
            MultiAgentError::FileOpError(multi_agent_config::file_ops::writer::FileOpError::Io(e))
        })?;
    }

    if dry_run {
        println!("Dry run complete (no files written)");
    } else {
        println!("Successfully compiled {compiled_count} configuration(s)");
    }

    Ok(())
}
