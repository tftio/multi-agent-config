//! Integration tests for Multi Agent Config

use std::process::Command;
use tempfile::TempDir;

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
    assert!(stdout.contains("A multi-agent configuration and orchestration tool"));
    assert!(stdout.contains("SUBCOMMANDS") || stdout.contains("Commands"));
    assert!(stdout.contains("version"));
    assert!(stdout.contains("license"));
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
    assert!(stdout.contains("Multi Agent Config"));
    assert!(stdout.contains("licensed under"));
    assert!(stdout.contains("LICENSE file"));
}

/// Test help for specific subcommand
#[test]
fn test_subcommand_help() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "multi-agent-config", "--", "version", "--help"])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Show version information") || stdout.contains("version"));
    assert!(stdout.contains("Usage") || stdout.contains("USAGE"));
}

/// Test that no args shows help
#[test]
fn test_no_args_shows_help() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "multi-agent-config"])
        .output()
        .expect("Failed to execute binary");
    // With clap, should show help and exit successfully due to arg_required_else_help
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("A multi-agent configuration and orchestration tool"));
    assert!(stdout.contains("version"));
    assert!(stdout.contains("license"));
}

/// Test process subcommand with input
#[test]
fn test_process_with_input() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "multi-agent-config",
            "--",
            "process",
            "test-input"
        ])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Successfully processed: test-input"));
}

/// Test process subcommand with verbose flag
#[test]
fn test_process_verbose_flag() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "multi-agent-config",
            "--",
            "process",
            "--verbose",
            "test-input"
        ])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Processing input: test-input"));
    assert!(stdout.contains("Successfully processed: test-input"));
}

/// Test process subcommand help
#[test]
fn test_process_help() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "multi-agent-config",
            "--",
            "process",
            "--help"
        ])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Process input data") || stdout.contains("process"));
    assert!(stdout.contains("--verbose"));
    assert!(stdout.contains("input") || stdout.contains("<INPUT>"));
}

/// Test invalid subcommand
#[test]
fn test_invalid_subcommand() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "multi-agent-config",
            "--",
            "invalid"
        ])
        .output()
        .expect("Failed to execute binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("unrecognized"));
}