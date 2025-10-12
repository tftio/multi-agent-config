//! CLI command implementations

pub mod compile;
pub mod diff;
pub mod init;
pub mod validate;

pub use compile::compile_command;
pub use diff::diff_command;
pub use init::init_command;
pub use validate::validate_command;
