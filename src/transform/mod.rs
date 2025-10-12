//! Format transformation module
//!
//! This module handles transformation of unified configuration into tool-specific formats.

pub mod cursor;
pub mod filter;
pub mod opencode;

pub use cursor::transform_for_cursor;
pub use filter::filter_servers_for_tool;
pub use opencode::transform_for_opencode;
