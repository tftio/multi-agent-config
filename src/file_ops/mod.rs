//! File operations module
//!
//! This module handles safe file operations including atomic writes, backups,
//! state tracking, and diff generation.

pub mod backup;
pub mod state;
pub mod writer;

pub use backup::create_backup;
pub use state::{hash_file, GeneratedFile, StateFile, StateTracker};
pub use writer::write_file_atomic;
