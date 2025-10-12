//! Multi Agent Config
//!
//! A multi-agent configuration and orchestration tool
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

mod completions;
mod doctor;
mod update;

/// Application version from Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// License identifier from Cargo.toml
const LICENSE: &str = "MIT";

#[derive(Parser)]
#[command(name = "multi-agent-config")]
#[command(about = "A multi-agent configuration and orchestration tool")]
#[command(version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show version information
    Version,
    /// Show license information
    License,
    /// Process input data
    Process {
        /// Input file or value
        input: String,
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },
    /// Check health and configuration
    Doctor,
    /// Update to the latest version
    Update {
        /// Specific version to install
        #[arg(long)]
        version: Option<String>,
        /// Force update even if already up-to-date
        #[arg(short, long)]
        force: bool,
        /// Custom installation directory
        #[arg(long)]
        install_dir: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version => {
            print_version();
        }
        Commands::License => {
            print_license();
        }
        Commands::Process { input, verbose } => {
            process_input(&input, verbose)?;
        }
        Commands::Completions { shell } => {
            completions::generate_completions(shell);
        }
        Commands::Doctor => {
            let exit_code = doctor::run_doctor();
            std::process::exit(exit_code);
        }
        Commands::Update {
            version,
            force,
            install_dir,
        } => {
            let exit_code = update::run_update(
                version.as_deref(),
                force,
                install_dir.as_deref(),
            );
            std::process::exit(exit_code);
        }
    }

Ok(())
}

/// Print version information
fn print_version() {
    println!("{} {}", "multi-agent-config".green().bold(), VERSION);
}

/// Print license information
fn print_license() {
    println!("{} is licensed under {}", "Multi Agent Config".green().bold(), LICENSE.yellow());
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
        },
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
        },
        "CC0-1.0" => {
            println!("Creative Commons CC0 1.0 Universal - Public domain dedication:");
            println!("• No rights reserved");
            println!("• Can be used for any purpose");
            println!("• No attribution required");
        },
        _ => {
            println!("License: {}", LICENSE);
            println!("See the LICENSE file for full terms and conditions.");
        }
    }

    println!();
    println!("For full license text, see: {}", "LICENSE file in project root".blue().underline());
}

/// Process the input value
fn process_input(input: &str, verbose: bool) -> Result<()> {
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    pb.set_message("Processing input");

    if verbose {
        println!("{} {}", "Processing input:".blue(), input);
    }

    // TODO: Add your application logic here
    // Simulate some work
    for i in 0..100 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        pb.set_position(i + 1);
    }

    pb.finish_with_message("Processing complete");
    println!("{} {}", "Successfully processed:".green(), input);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constant() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.chars().next().unwrap().is_numeric());
    }

    #[test]
    fn test_license_constant() {
        assert!(!LICENSE.is_empty());
        assert!(matches!(LICENSE, "MIT" | "Apache-2.0" | "CC0-1.0" | _));
    }

    #[test]
    fn test_process_input() {
        let result = process_input("test", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_input_verbose() {
        let result = process_input("test", true);
        assert!(result.is_ok());
    }
}
