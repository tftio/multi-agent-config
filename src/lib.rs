//! Multi-Agent-Config - Unified Configuration Manager for AI Coding Tools
//!
//! This library provides functionality to compile a single unified TOML configuration
//! into tool-specific MCP (Model Context Protocol) server configurations.

pub mod config;
pub mod error;
pub mod expand;
pub mod file_ops;
pub mod transform;

pub use config::{
    MultiAgentConfig, ServerConfig, Settings, ToolName, ValidationError, parse_and_expand_config,
    parse_config_file, validate_config,
};
pub use error::{
    ConfigError, EXIT_FILE_ERROR, EXIT_LOCK_ERROR, EXIT_PARTIAL_FAILURE, EXIT_SUCCESS,
    EXIT_VALIDATION_ERROR, MultiAgentError,
};
pub use expand::{Expander, ExpansionResult};
