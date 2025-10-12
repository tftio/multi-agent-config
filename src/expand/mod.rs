//! Environment variable expansion module
//!
//! This module handles expansion of variable references in configuration
//! values:
//! - `${VAR}` - Shell environment variables
//! - `{VAR}` - Variables from [env] section

pub mod expander;

pub use expander::{Expander, ExpansionResult};
