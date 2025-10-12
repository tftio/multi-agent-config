//! Formatted output functions for CLI

use colored::Colorize;
use is_terminal::IsTerminal;
use std::io::{stderr, stdout};

/// Check if stdout is a TTY
#[allow(dead_code)]
pub fn is_tty() -> bool {
    stdout().is_terminal()
}

/// Check if stderr is a TTY
#[allow(dead_code)]
pub fn is_stderr_tty() -> bool {
    stderr().is_terminal()
}

/// Print an error message
#[allow(dead_code)]
pub fn print_error(message: &str) {
    if is_stderr_tty() {
        eprintln!("{} {}", "Error:".red().bold(), message);
    } else {
        eprintln!("Error: {message}");
    }
}

/// Print a warning message
#[allow(dead_code)]
pub fn print_warning(message: &str) {
    if is_stderr_tty() {
        eprintln!("{} {}", "Warning:".yellow().bold(), message);
    } else {
        eprintln!("Warning: {message}");
    }
}

/// Print an info message
#[allow(dead_code)]
pub fn print_info(message: &str) {
    if is_tty() {
        println!("{} {}", "Info:".blue().bold(), message);
    } else {
        println!("Info: {message}");
    }
}

/// Print a success message
#[allow(dead_code)]
pub fn print_success(message: &str) {
    if is_tty() {
        println!("{} {}", "Success:".green().bold(), message);
    } else {
        println!("Success: {message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_tty() {
        // Just verify the function doesn't panic
        let _ = is_tty();
    }

    #[test]
    fn test_is_stderr_tty() {
        // Just verify the function doesn't panic
        let _ = is_stderr_tty();
    }

    // Note: print_* functions can't be easily unit tested without capturing
    // output They're tested indirectly through integration tests
}
