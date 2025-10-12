//! Cursor JSON transformer

use crate::config::types::{ServerConfig, ToolName};
use std::collections::HashMap;

/// Transform servers to Cursor JSON format (stub)
pub fn transform_for_cursor(
    _servers: &HashMap<String, ServerConfig>,
    _default_targets: &[String],
) -> Result<String, String> {
    Ok("{}".to_string())
}
