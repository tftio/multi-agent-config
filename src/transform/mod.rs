//! Format transformation module
//!
//! This module handles transformation of unified configuration into
//! tool-specific formats.

pub mod claude_code;
pub mod codex;
pub mod cursor;
pub mod filter;
pub mod opencode;

pub use claude_code::transform_for_claude_code;
pub use codex::transform_for_codex;
pub use cursor::transform_for_cursor;
pub use filter::filter_servers_for_tool;
pub use opencode::{
    OpencodeLocalServer, OpencodeRemoteServer, OpencodeServer, transform_for_opencode,
};
