//! Multi-Agent-Config - Unified Configuration Manager for AI Coding Tools
//!
//! Command-line interface for managing AI coding tool configurations.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod cli;
mod completions;
mod doctor;

use cli::commands::{compile_command, diff_command, init_command, validate_command};

/// Application version from Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// License identifier from Cargo.toml
const LICENSE: &str = env!("CARGO_PKG_LICENSE");

/// CLI structure
#[derive(Parser)]
#[command(name = "multi-agent-config")]
#[command(version = VERSION)]
#[command(about = "Unified configuration manager for AI coding tools")]
#[command(long_about = None)]
struct Cli {
    /// Path to configuration file
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

/// CLI subcommands
#[derive(Subcommand)]
enum Commands {
    /// Initialize configuration with template
    Init {
        /// Overwrite existing configuration
        #[arg(short, long)]
        force: bool,
    },

    /// Validate configuration without writing
    Validate,

    /// Compile and write tool configurations
    Compile {
        /// Target specific tools (default: all with matching servers)
        #[arg(short, long)]
        tool: Vec<String>,

        /// Show what would be done without writing
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Show diff of what would change
    Diff {
        /// Target specific tools (default: all with matching servers)
        #[arg(short, long)]
        tool: Vec<String>,
    },

    /// Show version information
    Version,

    /// Show license information
    License,

    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },

    /// Run health check and diagnostics
    Doctor,
}

/// Get default config path
fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("multi-agent-config")
        .join("config.toml")
}

fn main() {
    let cli = Cli::parse();

    // Get config path, using default if not provided
    let config_path = cli.config.unwrap_or_else(default_config_path);

    let exit_code = match cli.command {
        Commands::Version => {
            version_command();
            0
        }
        Commands::License => {
            license_command();
            0
        }
        Commands::Init { force } => match init_command(&config_path, force) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("{}", e.format_with_suggestion());
                e.exit_code()
            }
        },
        Commands::Validate => match validate_command(&config_path, cli.verbose) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("{}", e.format_with_suggestion());
                e.exit_code()
            }
        },
        Commands::Compile { tool, dry_run } => {
            match compile_command(&config_path, &tool, dry_run, cli.verbose) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("{}", e.format_with_suggestion());
                    e.exit_code()
                }
            }
        }
        Commands::Diff { tool } => match diff_command(&config_path, &tool, cli.verbose) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("{}", e.format_with_suggestion());
                e.exit_code()
            }
        },
        Commands::Completions { shell } => {
            completions::generate_completions(shell);
            0
        }
        Commands::Doctor => {
            doctor::run_doctor();
            0
        }
    };

    std::process::exit(exit_code);
}

/// Print version information
fn version_command() {
    println!("multi-agent-config {VERSION}");
}

/// Print license information
fn license_command() {
    println!("multi-agent-config is licensed under {LICENSE}");
    println!();

    match LICENSE {
        "MIT" => {
            println!("MIT License - A permissive license that allows:");
            println!("• Commercial use");
            println!("• Modification");
            println!("• Distribution");
            println!("• Private use");
            println!();
            println!("Requires:");
            println!("• License and copyright notice");
        }
        "Apache-2.0" => {
            println!("Apache License 2.0 - A permissive license that allows:");
            println!("• Commercial use");
            println!("• Modification");
            println!("• Distribution");
            println!("• Patent use");
            println!("• Private use");
            println!();
            println!("Requires:");
            println!("• License and copyright notice");
            println!("• State changes");
        }
        _ => {
            println!("License: {LICENSE}");
            println!("See the LICENSE file for full terms and conditions.");
        }
    }

    println!();
    println!("For full license text, see LICENSE file in project root");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_path() {
        let path = default_config_path();
        assert!(path.to_string_lossy().contains("multi-agent-config"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }
}
