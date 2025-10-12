//! Configuration module for multi-agent-config
//!
//! This module handles parsing, validation, and manipulation of the unified
//! TOML configuration format.

pub mod parser;
pub mod types;

pub use parser::parse_config_file;
pub use types::*;
