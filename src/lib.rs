//! Multi-Agent-Config - Unified Configuration Manager for AI Coding Tools
//!
//! This library provides functionality to compile a single unified TOML configuration
//! into tool-specific MCP (Model Context Protocol) server configurations.

pub mod config;
pub mod error;

pub use config::{parse_config_file, MultiAgentConfig, ServerConfig, Settings, ToolName};
pub use error::ConfigError;
