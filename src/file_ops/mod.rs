//! File operations module
//!
//! This module handles safe file operations including atomic writes, backups,
//! state tracking, and diff generation.

pub mod backup;
pub mod diff;
pub mod state;
pub mod writer;

pub use backup::create_backup;
pub use diff::{generate_diff, generate_file_diff};
pub use state::{GeneratedFile, StateFile, StateTracker, hash_file};
pub use writer::write_file_atomic;
