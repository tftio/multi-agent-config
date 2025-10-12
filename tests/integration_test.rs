//! Integration tests for Multi-Agent-Config

use std::process::Command;

/// Test that the binary exists and runs
#[test]
fn test_binary_exists() {
    let output = Command::new("cargo")
        .args(["build", "--bin", "multi-agent-config"])
        .output()
        .expect("Failed to execute cargo build");

    assert!(output.status.success(), "Failed to build binary");
}

/// Test version subcommand
#[test]
fn test_version_subcommand() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "multi-agent-config", "--", "version"])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("multi-agent-config"));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

/// Test global help flag
#[test]
fn test_help_flag() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "multi-agent-config", "--", "--help"])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Unified configuration manager for AI coding tools"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("version"));
    assert!(stdout.contains("license"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("validate"));
    assert!(stdout.contains("compile"));
}

/// Test license subcommand
#[test]
fn test_license_subcommand() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "multi-agent-config", "--", "license"])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("multi-agent-config is licensed under"));
    assert!(stdout.contains("LICENSE file"));
}

/// Test help for specific subcommand
#[test]
fn test_subcommand_help() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "multi-agent-config",
            "--",
            "version",
            "--help",
        ])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Show version information"));
    assert!(stdout.contains("Usage:"));
}

/// Test that no args shows help
#[test]
fn test_no_args_shows_help() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "multi-agent-config"])
        .output()
        .expect("Failed to execute binary");

    // No args with required subcommand shows error
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage:") || stderr.contains("required"));
}

/// Test init command
#[test]
fn test_init_command() {
    use tempfile::TempDir;
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "multi-agent-config",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "init",
        ])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Configuration initialized"));
    assert!(stdout.contains("Next steps"));

    // Verify file was created
    assert!(config_path.exists());
}

/// Test validate command
#[test]
fn test_validate_command() {
    use tempfile::TempDir;
    use std::fs;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Create valid config
    let valid_config = r#"
[settings]
version = "1.0"

[mcp.servers.test]
command = "npx"
args = ["-y", "package"]
"#;
    fs::write(&config_path, valid_config).unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "multi-agent-config",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "validate",
        ])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("valid"));
}

/// Test compile command with dry-run
#[test]
fn test_compile_command() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Create valid config
    let valid_config = r#"
[settings]
version = "1.0"

[mcp.servers.test]
command = "npx"
targets = ["cursor"]
"#;
    std::fs::write(&config_path, valid_config).unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "multi-agent-config",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "compile",
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Would write") || stdout.contains("Dry run"));
}

/// Test diff command (stub)
#[test]
fn test_diff_command() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "multi-agent-config", "--", "diff"])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("not yet implemented") || stdout.contains("Phase 5"));
}

/// Test doctor command
#[test]
fn test_doctor_command() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "multi-agent-config", "--", "doctor"])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("health check"));
}

/// Test completions command
#[test]
fn test_completions_command() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "multi-agent-config",
            "--",
            "completions",
            "bash",
        ])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should generate bash completion script
    assert!(stdout.contains("multi-agent-config") || stdout.contains("_multi"));
}

/// Test global verbose flag
#[test]
fn test_verbose_flag() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "multi-agent-config",
            "--",
            "--verbose",
            "version",
        ])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
}

/// Test global config flag
#[test]
fn test_config_flag() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "multi-agent-config",
            "--",
            "--config",
            "/tmp/test.toml",
            "version",
        ])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
}
